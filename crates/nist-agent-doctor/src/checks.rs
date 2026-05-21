//! The five new checks. Each is a unit struct implementing
//! `NistCheck`; the harness composes them into a `Vec<Box<dyn
//! NistCheck>>` and runs them sequentially via the top-level
//! `run` function.

use crate::context::NistDoctorContext;
use citrate_agent_core::doctor::{CheckResult, Severity};
use nist_agent_policy::types::Role;
use std::collections::BTreeMap;

/// The trait every doctor check implements. Send + Sync so a
/// single `Vec<Box<dyn NistCheck>>` can be shared across threads.
/// Shape mirrors upstream's `citrate_agent_core::doctor::checks::Check`
/// trait — the upstream PR will unify them.
pub trait NistCheck: Send + Sync {
    fn name(&self) -> &str;
    fn run(&self, ctx: &NistDoctorContext) -> CheckResult;
}

fn pass(name: &str, message: impl Into<String>) -> CheckResult {
    CheckResult {
        name: name.into(),
        severity: Severity::Pass,
        message: message.into(),
        details: BTreeMap::new(),
    }
}

fn warn(name: &str, message: impl Into<String>) -> CheckResult {
    CheckResult {
        name: name.into(),
        severity: Severity::Warn,
        message: message.into(),
        details: BTreeMap::new(),
    }
}

fn blocker(name: &str, message: impl Into<String>) -> CheckResult {
    CheckResult {
        name: name.into(),
        severity: Severity::Blocker,
        message: message.into(),
        details: BTreeMap::new(),
    }
}

// ── #3 PolicyBundleValidityCheck ────────────────────────────────

/// RFC §10.2 check 3 — policy bundle is signed by SecurityOfficer
/// AND is within its signed validity window.
///
/// BLOCKER iff:
///   - signature does not verify against the trusted SO pubkey
///   - now < not_before  (clock skew / premature activation)
///   - now > expires_at  (expired)
///   - any one of the five base roles has zero identities assigned
///   - bundle_version exceeds harness max
pub struct PolicyBundleValidityCheck;

impl NistCheck for PolicyBundleValidityCheck {
    fn name(&self) -> &str {
        "policy-bundle-validity"
    }
    fn run(&self, ctx: &NistDoctorContext) -> CheckResult {
        let Some(raw) = &ctx.raw_bundle else {
            return pass(self.name(), "skipped: no raw_bundle in context");
        };
        let Some(pk) = &ctx.so_pubkey else {
            return blocker(
                self.name(),
                "raw_bundle present but no SecurityOfficer pubkey in context",
            );
        };
        let bundle = match raw.verify_and_decode(pk) {
            Ok(b) => b,
            Err(e) => {
                return blocker(self.name(), format!("bundle verify failed: {e}"));
            }
        };
        match bundle.activate(ctx.now_unix, &BTreeMap::new()) {
            Ok(()) => pass(
                self.name(),
                format!(
                    "bundle '{}' valid; not_before={}, expires_at={}",
                    bundle.bundle_name, bundle.not_before, bundle.expires_at
                ),
            ),
            Err(e) => blocker(self.name(), format!("activate failed: {e}")),
        }
    }
}

// ── #6 NetworkPostureCheck ──────────────────────────────────────

/// RFC §10.2 check 6 — observed network behavior matches the
/// active PolicyBundle's `egress_posture`.
///
/// Logic:
///   - `Disabled` + any observation → BLOCKER
///   - `BrokerOnly` (we have no broker-vs-direct distinction in v1
///     observations yet) → WARN with the observation count, telling
///     the operator the harness can't currently verify the
///     constraint. Upstream PR for richer observations is tracked
///     in S-7b RETRO.
///   - `Allowed` → PASS regardless of observation count.
pub struct NetworkPostureCheck;

impl NistCheck for NetworkPostureCheck {
    fn name(&self) -> &str {
        "network-posture-matches-policy"
    }
    fn run(&self, ctx: &NistDoctorContext) -> CheckResult {
        use nist_agent_policy::types::EgressPosture;

        let Some(raw) = &ctx.raw_bundle else {
            return pass(self.name(), "skipped: no raw_bundle in context");
        };
        let Some(pk) = &ctx.so_pubkey else {
            return blocker(self.name(), "raw_bundle present but no SO pubkey");
        };
        let bundle = match raw.verify_and_decode(pk) {
            Ok(b) => b,
            Err(e) => return blocker(self.name(), format!("bundle verify failed: {e}")),
        };

        let obs_count = ctx.egress_observations.len();

        match bundle.egress_posture {
            EgressPosture::Disabled => {
                if obs_count == 0 {
                    pass(self.name(), "posture=Disabled and zero egress observations")
                } else {
                    blocker(
                        self.name(),
                        format!(
                            "posture=Disabled but {obs_count} egress observation(s) recorded \
                             — first destination: {}",
                            ctx.egress_observations[0].destination
                        ),
                    )
                }
            }
            EgressPosture::BrokerOnly => warn(
                self.name(),
                format!(
                    "posture=BrokerOnly; {obs_count} observation(s) recorded — \
                     direct-vs-broker discrimination not yet implemented \
                     (S-7b: richer EgressObservation)"
                ),
            ),
            EgressPosture::Allowed => pass(
                self.name(),
                format!("posture=Allowed; {obs_count} observation(s) recorded"),
            ),
        }
    }
}

// ── #7 ModelHashCheck ───────────────────────────────────────────

/// RFC §10.2 check 7 — bundled model file's SHA-256 matches the
/// release manifest's declared digest. NIST SI-7.
pub struct ModelHashCheck;

impl NistCheck for ModelHashCheck {
    fn name(&self) -> &str {
        "model-file-hash-matches-manifest"
    }
    fn run(&self, ctx: &NistDoctorContext) -> CheckResult {
        let Some(path) = &ctx.model_path else {
            return pass(self.name(), "skipped: no model_path in context");
        };
        let Some(expected) = &ctx.model_expected_sha256 else {
            return pass(self.name(), "skipped: no model_expected_sha256 in context");
        };
        match nist_agent_model::verify_sha256(path, expected) {
            Ok(()) => pass(
                self.name(),
                format!(
                    "{} sha256 matches expected {}",
                    path.display(),
                    &expected[..16.min(expected.len())]
                ),
            ),
            Err(nist_agent_model::ModelError::HashMismatch { expected, actual }) => blocker(
                self.name(),
                format!("hash mismatch — expected {expected}, got {actual}"),
            ),
            Err(e) => blocker(self.name(), format!("verify failed: {e}")),
        }
    }
}

// ── #8 TlaSpecsCurrentCheck ─────────────────────────────────────

/// RFC §10.2 check 8 — TLA+ specifications for installed tier-high
/// and tier-critical capsules have a current verification record.
///
/// Today's bound: we check that the `.agentile/formal/specs/`
/// directory contains at least `tla_specs_min` `.tla` files (the
/// S-2 baseline of 5). Reading per-capsule TLA+ verification from
/// BenchmarkRegistry is S-7b work — the BenchmarkRegistry contract
/// doesn't have a deployment yet (S-15 mainnet anchor).
pub struct TlaSpecsCurrentCheck;

impl NistCheck for TlaSpecsCurrentCheck {
    fn name(&self) -> &str {
        "tla-specs-current"
    }
    fn run(&self, ctx: &NistDoctorContext) -> CheckResult {
        let Some(dir) = &ctx.tla_specs_dir else {
            return pass(self.name(), "skipped: no tla_specs_dir in context");
        };
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) => {
                return blocker(
                    self.name(),
                    format!("cannot read tla_specs_dir {}: {e}", dir.display()),
                );
            }
        };
        let mut count = 0usize;
        for entry in entries.flatten() {
            let path = entry.path();
            // Recurse one level into subdirs (agent/, chain/, etc.).
            if path.is_dir() {
                if let Ok(sub) = std::fs::read_dir(&path) {
                    for s in sub.flatten() {
                        let sp = s.path();
                        if sp.extension().and_then(|x| x.to_str()) == Some("tla") {
                            count += 1;
                        }
                    }
                }
            } else if path.extension().and_then(|x| x.to_str()) == Some("tla") {
                count += 1;
            }
        }
        if count < ctx.tla_specs_min {
            blocker(
                self.name(),
                format!(
                    "found {count} .tla file(s) under {} — below minimum {}",
                    dir.display(),
                    ctx.tla_specs_min
                ),
            )
        } else {
            pass(
                self.name(),
                format!(
                    "{count} .tla file(s) under {} (≥ {})",
                    dir.display(),
                    ctx.tla_specs_min
                ),
            )
        }
    }
}

// ── #9 RoleLatticeCheck ─────────────────────────────────────────

/// RFC §10.2 check 9 — required role assignments are filled. Every
/// role in `Role::ALL` MUST have at least one identity assigned.
///
/// This is a subset of `PolicyBundleValidityCheck`'s activate-time
/// check, but doctor reports it separately so operators see which
/// role specifically failed. The audit trail benefits from the
/// granularity.
pub struct RoleLatticeCheck;

impl NistCheck for RoleLatticeCheck {
    fn name(&self) -> &str {
        "role-lattice-fully-assigned"
    }
    fn run(&self, ctx: &NistDoctorContext) -> CheckResult {
        let Some(raw) = &ctx.raw_bundle else {
            return pass(self.name(), "skipped: no raw_bundle in context");
        };
        let Some(pk) = &ctx.so_pubkey else {
            return blocker(self.name(), "raw_bundle present but no SO pubkey");
        };
        let bundle = match raw.verify_and_decode(pk) {
            Ok(b) => b,
            Err(e) => return blocker(self.name(), format!("bundle verify failed: {e}")),
        };
        let mut missing: Vec<Role> = Vec::new();
        for r in Role::ALL {
            let assigned = bundle.role_assignments.get(r).map(|v| v.len()).unwrap_or(0);
            if assigned == 0 {
                missing.push(*r);
            }
        }
        if missing.is_empty() {
            pass(self.name(), "all 5 base roles have ≥1 identity")
        } else {
            blocker(self.name(), format!("missing role identities: {missing:?}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use nist_agent_policy::{PolicyBundle, RawSignedBundle};

    fn fixture_key() -> SigningKey {
        SigningKey::from_bytes(&[7u8; 32])
    }

    fn signed_bundle(bundle: &PolicyBundle, sk: &SigningKey) -> RawSignedBundle {
        let cbor = bundle.encode_canonical().expect("encode");
        let sig = sk.sign(&cbor).to_bytes().to_vec();
        RawSignedBundle {
            bundle_cbor: cbor,
            signature: sig,
        }
    }

    // ── PolicyBundleValidityCheck ──

    #[test]
    fn policy_check_skips_without_bundle() {
        let ctx = NistDoctorContext::empty(0);
        let r = PolicyBundleValidityCheck.run(&ctx);
        assert!(matches!(r.severity, Severity::Pass));
        assert!(r.message.contains("skipped"));
    }

    #[test]
    fn policy_check_passes_on_valid_bundle() {
        let sk = fixture_key();
        let bundle = PolicyBundle::minimal_template();
        let raw = signed_bundle(&bundle, &sk);
        let mut ctx = NistDoctorContext::empty(0);
        ctx.raw_bundle = Some(raw);
        ctx.so_pubkey = Some(sk.verifying_key());
        let r = PolicyBundleValidityCheck.run(&ctx);
        assert!(matches!(r.severity, Severity::Pass), "got {r:?}");
    }

    #[test]
    fn policy_check_blocks_on_wrong_so_pubkey() {
        let sk = fixture_key();
        let bundle = PolicyBundle::minimal_template();
        let raw = signed_bundle(&bundle, &sk);
        let wrong_pk = SigningKey::from_bytes(&[42u8; 32]).verifying_key();
        let mut ctx = NistDoctorContext::empty(0);
        ctx.raw_bundle = Some(raw);
        ctx.so_pubkey = Some(wrong_pk);
        let r = PolicyBundleValidityCheck.run(&ctx);
        assert!(matches!(r.severity, Severity::Blocker), "got {r:?}");
        assert!(r.message.contains("verify failed"));
    }

    #[test]
    fn policy_check_blocks_on_expired_bundle() {
        let sk = fixture_key();
        let mut bundle = PolicyBundle::minimal_template();
        bundle.not_before = 100;
        bundle.expires_at = 200;
        let raw = signed_bundle(&bundle, &sk);
        let mut ctx = NistDoctorContext::empty(500); // > expires_at
        ctx.raw_bundle = Some(raw);
        ctx.so_pubkey = Some(sk.verifying_key());
        let r = PolicyBundleValidityCheck.run(&ctx);
        assert!(matches!(r.severity, Severity::Blocker));
        assert!(r.message.contains("activate failed"));
    }

    // ── NetworkPostureCheck ──

    #[test]
    fn network_check_passes_when_disabled_and_no_observations() {
        let sk = fixture_key();
        let bundle = PolicyBundle::minimal_template(); // posture=Disabled by default
        let raw = signed_bundle(&bundle, &sk);
        let mut ctx = NistDoctorContext::empty(0);
        ctx.raw_bundle = Some(raw);
        ctx.so_pubkey = Some(sk.verifying_key());
        let r = NetworkPostureCheck.run(&ctx);
        assert!(matches!(r.severity, Severity::Pass), "got {r:?}");
    }

    #[test]
    fn network_check_blocks_when_disabled_but_observed_egress() {
        use crate::context::EgressObservation;
        let sk = fixture_key();
        let bundle = PolicyBundle::minimal_template();
        let raw = signed_bundle(&bundle, &sk);
        let mut ctx = NistDoctorContext::empty(0);
        ctx.raw_bundle = Some(raw);
        ctx.so_pubkey = Some(sk.verifying_key());
        ctx.egress_observations.push(EgressObservation {
            observed_at_unix: 10,
            destination: "example.com".into(),
            capsule: None,
        });
        let r = NetworkPostureCheck.run(&ctx);
        assert!(matches!(r.severity, Severity::Blocker), "got {r:?}");
        assert!(r.message.contains("example.com"));
    }

    // ── ModelHashCheck ──

    #[test]
    fn model_hash_check_passes_on_known_empty_file_hash() {
        use std::io::Write;
        use tempfile::NamedTempFile;
        let f = NamedTempFile::new().expect("tempfile");
        // Don't write — file is empty; sha256 of "" is the known
        // constant we pin.
        let mut ctx = NistDoctorContext::empty(0);
        ctx.model_path = Some(f.path().to_path_buf());
        ctx.model_expected_sha256 =
            Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into());
        let r = ModelHashCheck.run(&ctx);
        assert!(matches!(r.severity, Severity::Pass), "got {r:?}");
        // Suppress unused-warning on `Write` import.
        let _ = NamedTempFile::new().and_then(|mut t| t.write_all(b"").map(|_| t));
    }

    #[test]
    fn model_hash_check_blocks_on_mismatch() {
        use tempfile::NamedTempFile;
        let f = NamedTempFile::new().expect("tempfile");
        let mut ctx = NistDoctorContext::empty(0);
        ctx.model_path = Some(f.path().to_path_buf());
        ctx.model_expected_sha256 =
            Some("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".into());
        let r = ModelHashCheck.run(&ctx);
        assert!(matches!(r.severity, Severity::Blocker), "got {r:?}");
        assert!(r.message.contains("hash mismatch"));
    }

    // ── TlaSpecsCurrentCheck ──

    #[test]
    fn tla_check_skips_without_dir() {
        let ctx = NistDoctorContext::empty(0);
        let r = TlaSpecsCurrentCheck.run(&ctx);
        assert!(matches!(r.severity, Severity::Pass));
    }

    #[test]
    fn tla_check_blocks_when_below_min() {
        use tempfile::TempDir;
        let tmp = TempDir::new().expect("tempdir");
        // 0 specs in the dir, but min=5.
        let mut ctx = NistDoctorContext::empty(0);
        ctx.tla_specs_dir = Some(tmp.path().to_path_buf());
        ctx.tla_specs_min = 5;
        let r = TlaSpecsCurrentCheck.run(&ctx);
        assert!(matches!(r.severity, Severity::Blocker), "got {r:?}");
    }

    #[test]
    fn tla_check_passes_at_or_above_min_with_subdir_recursion() {
        use std::fs;
        use tempfile::TempDir;
        let tmp = TempDir::new().expect("tempdir");
        let sub = tmp.path().join("agent");
        fs::create_dir(&sub).expect("subdir");
        for name in ["a.tla", "b.tla", "c.tla", "d.tla", "e.tla"] {
            fs::write(sub.join(name), b"").expect("write");
        }
        let mut ctx = NistDoctorContext::empty(0);
        ctx.tla_specs_dir = Some(tmp.path().to_path_buf());
        ctx.tla_specs_min = 5;
        let r = TlaSpecsCurrentCheck.run(&ctx);
        assert!(matches!(r.severity, Severity::Pass), "got {r:?}");
        assert!(r.message.contains("5"));
    }

    // ── RoleLatticeCheck ──

    #[test]
    fn role_lattice_check_passes_on_complete_template() {
        let sk = fixture_key();
        let bundle = PolicyBundle::minimal_template(); // all five seeded
        let raw = signed_bundle(&bundle, &sk);
        let mut ctx = NistDoctorContext::empty(0);
        ctx.raw_bundle = Some(raw);
        ctx.so_pubkey = Some(sk.verifying_key());
        let r = RoleLatticeCheck.run(&ctx);
        assert!(matches!(r.severity, Severity::Pass), "got {r:?}");
    }

    #[test]
    fn role_lattice_check_blocks_when_role_unfilled() {
        let sk = fixture_key();
        let mut bundle = PolicyBundle::minimal_template();
        bundle.role_assignments.remove(&Role::Auditor);
        let raw = signed_bundle(&bundle, &sk);
        let mut ctx = NistDoctorContext::empty(0);
        ctx.raw_bundle = Some(raw);
        ctx.so_pubkey = Some(sk.verifying_key());
        let r = RoleLatticeCheck.run(&ctx);
        assert!(matches!(r.severity, Severity::Blocker), "got {r:?}");
        assert!(r.message.contains("Auditor"));
    }

    // ── Composite run() ──

    #[test]
    fn v1_checks_run_against_empty_context_all_skip() {
        let ctx = NistDoctorContext::empty(0);
        let results = crate::run(&ctx, &crate::v1_checks());
        assert_eq!(results.len(), 5);
        for r in &results {
            assert!(matches!(r.severity, Severity::Pass), "{}: {r:?}", r.name);
        }
        assert!(matches!(crate::compute_overall(&results), Severity::Pass));
    }
}
