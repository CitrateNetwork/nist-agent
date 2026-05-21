//! Context plumbed into every doctor check. The harness builds
//! one of these from its live state at doctor-run time.
//!
//! Each field is `Option<_>` so the harness can run a partial
//! pass when not all surfaces are wired (e.g. first-run when no
//! PolicyBundle is signed yet). Checks whose dependency is `None`
//! return `Severity::Pass` with `details: skipped`, so partial
//! contexts still produce meaningful reports — same shape as
//! upstream's `citrate_agent_core::doctor::DoctorContext`.

use ed25519_dalek::VerifyingKey;
use nist_agent_policy::RawSignedBundle;
use std::path::PathBuf;

pub struct NistDoctorContext {
    /// `now()` injected — pass a deterministic value in tests,
    /// `time::OffsetDateTime::now_utc().unix_timestamp()` (or
    /// equivalent) in production. Doctor checks that rely on
    /// validity-window math read this rather than calling the
    /// system clock directly.
    pub now_unix: i64,

    /// The signed PolicyBundle the harness is running with.
    /// `None` skips PolicyBundleValidity + RoleLattice checks
    /// (no bundle to validate). NetworkPosture skips too because
    /// it derives expected posture from the bundle.
    pub raw_bundle: Option<RawSignedBundle>,

    /// The SecurityOfficer's trusted public key used to verify
    /// `raw_bundle`. Operator-configured at deploy time.
    pub so_pubkey: Option<VerifyingKey>,

    /// Recent egress observations (what the harness's network code
    /// actually did). Pairs with PolicyBundle.egress_posture for the
    /// NetworkPostureCheck: if posture is `Disabled` and this is
    /// non-empty, the check is BLOCKER. The list is the operator's
    /// responsibility to populate; the harness owns the
    /// instrumentation, not the doctor.
    pub egress_observations: Vec<EgressObservation>,

    /// Bundled model file path. `None` skips ModelHashCheck.
    pub model_path: Option<PathBuf>,

    /// Expected SHA-256 (hex-encoded) of the model file at
    /// `model_path`. `None` skips. Comes from the release manifest
    /// in production.
    pub model_expected_sha256: Option<String>,

    /// Directory containing the TLA+ specs. Defaults to
    /// `.agentile/formal/specs/` when present. `None` skips
    /// TlaSpecsCurrentCheck.
    pub tla_specs_dir: Option<PathBuf>,

    /// Minimum number of TLA+ specs the harness is willing to run
    /// with. The S-2 baseline is 5 (the normative set). Doctor
    /// flags a count below this as BLOCKER. Same semantics as
    /// `scripts/ci/check_spec_ratchet.py`'s baseline.
    pub tla_specs_min: usize,
}

/// One observation of an outbound network attempt the harness made.
/// Surfaced into the doctor's NetworkPosture check; if the policy
/// posture is `Disabled` and `egress_observations` contains any
/// row, the check fails BLOCKER.
#[derive(Debug, Clone)]
pub struct EgressObservation {
    /// Wall-clock timestamp (unix seconds) of the attempt.
    pub observed_at_unix: i64,
    /// Destination host (DNS name or IP literal). Doctor reports
    /// this verbatim so an auditor can identify the violator.
    pub destination: String,
    /// Capsule context, if any. `None` if the attempt was from the
    /// harness itself (e.g. an anchor-write transaction) — those
    /// are still violations under `EgressPosture::Disabled`.
    pub capsule: Option<String>,
}

impl NistDoctorContext {
    /// Minimal empty context — every check skips. Used in tests
    /// that exercise the skip path without wiring real surfaces.
    pub fn empty(now_unix: i64) -> Self {
        Self {
            now_unix,
            raw_bundle: None,
            so_pubkey: None,
            egress_observations: Vec::new(),
            model_path: None,
            model_expected_sha256: None,
            tla_specs_dir: None,
            tla_specs_min: 5,
        }
    }
}
