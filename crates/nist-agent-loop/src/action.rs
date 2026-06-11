//! `Action` and `ApprovalPayload` — the wire shapes between the
//! agent loop and the HITL approval surface.
//!
//! The agent loop never executes actions directly. It proposes
//! them; the operator's surface (Slint, CLI, daemon RPC, mobile)
//! collects signatures; the harness routes approvals back to the
//! loop. Both directions of this conversation are typed below.

use serde::{Deserialize, Serialize};

/// A proposed side-effecting call. The shape mirrors the call sites
/// in `citrate_agent_core::capsule::CapsuleDispatch`; the loop
/// produces these for capsule invocations, the surface consumes them
/// for display + approval routing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Action {
    /// Capsule name + version (e.g. `"ferpa-redact-student-record:0.3.1"`).
    pub capsule: String,
    /// Function name within the capsule WIT export.
    pub function: String,
    /// Args, JSON-encoded for surface display; the harness
    /// deserializes per-capsule WIT at execution time.
    pub args_json: String,
    /// SHA-256 over the deterministic length-prefixed encoding of
    /// {capsule, function, args}: each field is prefixed with its
    /// byte length as a u64 big-endian word and concatenated (see
    /// [`Action::compute_hash`]). The HITL gate signs this hash;
    /// the audit chain records it. Determinism property: same
    /// inputs → same hash, always. Any layer recomputing this
    /// hash MUST use the same length-prefixed scheme.
    pub proposal_hash: [u8; 32],
}

/// Approval payload returned by the surface. The signatures here
/// are detached signatures over `Action::proposal_hash`; the
/// harness verifies them against the configured role lattice in
/// `citrate_agent_core::hitl`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalPayload {
    /// Echoed proposal hash — caller MUST match it against the
    /// outstanding action's hash. Disagreement is a hard error.
    pub proposal_hash: [u8; 32],
    /// Whether the surface accepted (`true`) or rejected (`false`).
    /// Rejection is also a valid resume signal — the loop emits a
    /// rejection audit record and returns control without executing.
    pub approved: bool,
    /// Signatures collected by the surface. The HITL quorum check
    /// in citrate-agent-core's `ApprovalQueue` validates whether
    /// this set satisfies the action's required-roles multiset.
    pub signatures: Vec<u8>, // opaque to the loop; HITL verifies
}

impl Action {
    /// Compute the proposal hash: SHA-256 over each field
    /// length-prefixed (u64 big-endian) and concatenated. NOT
    /// CBOR — the prefix scheme below is the normative encoding
    /// (NIST_AGENT-2026-05-31-010). Pure function — the
    /// determinism property of RFC §5.4 hinges on this being
    /// byte-stable across builds.
    pub fn compute_hash(capsule: &str, function: &str, args_json: &str) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        // Length-prefix each field so concatenation can't ambiguate
        // (e.g. capsule="a", function="bc" vs capsule="ab",
        // function="c" would otherwise hash identically).
        h.update((capsule.len() as u64).to_be_bytes());
        h.update(capsule.as_bytes());
        h.update((function.len() as u64).to_be_bytes());
        h.update(function.as_bytes());
        h.update((args_json.len() as u64).to_be_bytes());
        h.update(args_json.as_bytes());
        let out: [u8; 32] = h.finalize().into();
        out
    }

    /// Build an Action with the hash filled in.
    pub fn new(
        capsule: impl Into<String>,
        function: impl Into<String>,
        args_json: impl Into<String>,
    ) -> Self {
        let capsule = capsule.into();
        let function = function.into();
        let args_json = args_json.into();
        let proposal_hash = Self::compute_hash(&capsule, &function, &args_json);
        Self {
            capsule,
            function,
            args_json,
            proposal_hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proposal_hash_is_deterministic() {
        let a = Action::new("cap1", "fn", r#"{"x":1}"#);
        let b = Action::new("cap1", "fn", r#"{"x":1}"#);
        assert_eq!(a.proposal_hash, b.proposal_hash);
    }

    #[test]
    fn proposal_hash_disambiguates_by_length_prefix() {
        // Without length-prefixing, ("a","bc","") would hash the same
        // as ("ab","c",""). Confirms our framing prevents that.
        let a = Action::new("a", "bc", "");
        let b = Action::new("ab", "c", "");
        assert_ne!(a.proposal_hash, b.proposal_hash);
    }

    #[test]
    fn proposal_hash_changes_with_args() {
        let a = Action::new("cap", "fn", r#"{"x":1}"#);
        let b = Action::new("cap", "fn", r#"{"x":2}"#);
        assert_ne!(a.proposal_hash, b.proposal_hash);
    }

    #[test]
    fn compute_hash_doc_describes_the_real_scheme() {
        // RED for NIST_AGENT-2026-05-31-010 (DOC-STALE): the doc
        // claimed `proposal_hash` is a canonical-CBOR hash while
        // the implementation hashes u64-BE length-prefixed raw
        // bytes. Any layer recomputing the hash *from the doc*
        // would mismatch. Pin: the stale claim must not reappear
        // in this module. (Needle assembled so this test's own
        // source can't satisfy it.)
        let src = include_str!("action.rs");
        let needle = ["canonical", "CBOR"].join(" ");
        assert!(
            !src.contains(&needle),
            "doc must describe the length-prefixed scheme actually implemented"
        );
        assert!(
            src.contains("length-prefix"),
            "doc must name the length-prefixed scheme"
        );
    }
}
