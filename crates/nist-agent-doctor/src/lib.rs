//! nist-agent-doctor — pre-flight checks for the RFC §10.2
//! requirements not covered by `citrate_agent_core::doctor`.
//!
//! Upstream (CIT-AGENT-7a) shipped 5 of the 11 RFC §10.2 checks
//! using surfaces that existed at that sprint:
//!
//!   audit-chain-integrity            (RFC check #4)
//!   audit-file-permissions           (not in RFC list — bonus)
//!   approval-queue-depth             (RFC check #10)
//!   pending-break-glass              (RFC check #11)
//!   runtime-presence                 (not in RFC list — bonus)
//!
//! This crate adds 5 more, each binding to a surface S-2..S-6
//! built locally in nist-agent:
//!
//!   policy-bundle-validity           (RFC check #3) → nist-agent-policy
//!   network-posture-matches-policy   (RFC check #6) → nist-agent-policy
//!   model-file-hash-matches-manifest (RFC check #7) → nist-agent-model
//!   tla-specs-current                (RFC check #8) → .agentile/formal/
//!   role-lattice-fully-assigned      (RFC check #9) → nist-agent-policy
//!
//! Three RFC §10.2 checks remain UNIMPLEMENTED here AND upstream:
//!
//!   capsule-signatures-verify        (RFC check #1) → S-7b
//!   manifest-wit-wasm-capability     (RFC check #2) → S-7b
//!   approver-hardware-key-bindings   (RFC check #5) → S-11 (mobile)
//!
//! Per ADR-006, this crate lands locally per the
//! ALIGNMENT.md exception-clause pattern; upstream PR follows
//! same shape as ADR-004 / ADR-005.

pub mod checks;
pub mod context;
pub mod error;

pub use checks::{
    ModelHashCheck, NetworkPostureCheck, NistCheck, PolicyBundleValidityCheck, RoleLatticeCheck,
    TlaSpecsCurrentCheck,
};
pub use context::NistDoctorContext;
pub use error::DoctorError;

// Re-export the upstream CheckResult + Severity types so the
// caller doesn't need to know about citrate-agent-core to use
// our checks' output. Rule 9 — one source of truth (upstream
// owns the result type; we just produce it).
pub use citrate_agent_core::doctor::{CheckResult, Severity};

/// Run a set of nist-agent doctor checks against the supplied
/// context. Returns a list of results in the order the checks
/// were given. The overall severity is the max across results
/// (any Blocker = Blocker; else any Warn = Warn; else Pass).
///
/// The harness composes upstream's `doctor::run` results with
/// ours into a single `DoctorReport`; this function is the local
/// half.
pub fn run(ctx: &NistDoctorContext, checks: &[Box<dyn NistCheck>]) -> Vec<CheckResult> {
    checks.iter().map(|c| c.run(ctx)).collect()
}

/// Convenience: compose the full v1 set of new checks against the
/// supplied context. Tests and the operator-facing harness both
/// use this to avoid hand-building the Vec.
pub fn v1_checks() -> Vec<Box<dyn NistCheck>> {
    vec![
        Box::new(PolicyBundleValidityCheck),
        Box::new(NetworkPostureCheck),
        Box::new(ModelHashCheck),
        Box::new(TlaSpecsCurrentCheck),
        Box::new(RoleLatticeCheck),
    ]
}

/// Compute overall severity across a results vector. Mirrors
/// upstream's `compute_overall` function (private over there;
/// re-implemented here so the local half can report its own
/// summary without depending on the upstream private fn).
pub fn compute_overall(results: &[CheckResult]) -> Severity {
    if results
        .iter()
        .any(|r| matches!(r.severity, Severity::Blocker))
    {
        Severity::Blocker
    } else if results.iter().any(|r| matches!(r.severity, Severity::Warn)) {
        Severity::Warn
    } else {
        Severity::Pass
    }
}
