//! `citrate-agent doctor` — runs the 11 RFC §10.2 pre-flight
//! checks against the local environment.
//!
//! Composes `nist_agent_doctor::v1_checks()` with a context
//! built from the operator's config (which path is the policy
//! bundle, where the GGUF lives, etc.). The full
//! configuration-binding path is operator-side; for v1 we
//! accept enough flags to drive the doctor in the air-gap
//! test runbook ([`docs/audit/AIRGAP_TEST.md`](../../docs/audit/AIRGAP_TEST.md)).

use anyhow::{Context, Result};
use clap::Args;
use nist_agent_doctor::{
    compute_overall, run as run_checks, v1_checks, CheckResult, NistDoctorContext, Severity,
};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct DoctorArgs {
    /// Path to the signed PolicyBundle. Skipped when absent.
    #[arg(long)]
    pub policy_bundle: Option<PathBuf>,

    /// Path to the embedded GGUF model file. Skipped when absent.
    /// SI-7 check requires both this and --expected-model-sha256.
    #[arg(long)]
    pub model_path: Option<PathBuf>,

    /// Expected hex sha256 of the GGUF file (from the release
    /// manifest's `[model].sha256`). Skipped when absent.
    #[arg(long)]
    pub expected_model_sha256: Option<String>,

    /// Path to the TLA+ specs directory. Defaults to
    /// `.agentile/formal/specs/` when present.
    #[arg(long)]
    pub tla_specs_dir: Option<PathBuf>,

    /// Emit results as JSON instead of a human table.
    #[arg(long)]
    pub json: bool,
}

pub async fn run(args: DoctorArgs) -> Result<i32> {
    let now_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let tla = args.tla_specs_dir.or_else(|| {
        let default = PathBuf::from(".agentile/formal/specs");
        default.exists().then_some(default)
    });

    let mut ctx = NistDoctorContext::empty(now_unix);
    ctx.model_path = args.model_path;
    ctx.model_expected_sha256 = args.expected_model_sha256;
    ctx.tla_specs_dir = tla;

    let checks = v1_checks();
    let mut results = run_checks(&ctx, &checks);

    // NA2-B-001: `--policy-bundle` must NOT be silently discarded.
    // v1.0 decode wiring (RawSignedBundle::from_path + a trusted
    // SecurityOfficer pubkey surface) is not canonized yet, so the
    // doctor cannot actually verify a supplied bundle. Rather than
    // dropping the argument on the floor — which let the operator
    // believe their bundle had been checked and produced an all-Pass
    // report — we emit an explicit, machine-readable Warn so the
    // overall severity and exit code reflect that the supplied bundle
    // was NOT verified.
    if let Some(bundle_path) = &args.policy_bundle {
        let mut details = BTreeMap::new();
        details.insert(
            "skipped".to_string(),
            "policy-bundle supplied but doctor cannot decode/verify it \
             (no RawSignedBundle::from_path wiring / no trusted SecurityOfficer \
             pubkey surface configured)"
                .to_string(),
        );
        details.insert("path".to_string(), bundle_path.display().to_string());
        results.push(CheckResult {
            name: "policy-bundle-wired".to_string(),
            severity: Severity::Warn,
            message: format!(
                "policy-bundle {} supplied but not verified — nothing was checked against it",
                bundle_path.display()
            ),
            details,
        });
    }

    let overall = compute_overall(&results);

    if args.json {
        let arr = serde_json::json!({
            "results": results.iter().map(|r| serde_json::json!({
                "name": r.name,
                "severity": format!("{:?}", r.severity),
                "message": r.message,
                "details": r.details,
            })).collect::<Vec<_>>(),
            "overall": format!("{:?}", overall),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&arr).context("serialize doctor json")?
        );
    } else {
        println!("{:<40} {:<8} details", "check", "severity");
        println!("{}", "-".repeat(80));
        for r in &results {
            println!(
                "{:<40} {:<8} {}",
                r.name,
                format!("{:?}", r.severity),
                r.message,
            );
            for (k, v) in &r.details {
                println!("    {k}: {v}");
            }
        }
        println!("{}", "-".repeat(80));
        println!("overall: {:?}", overall);
    }

    Ok(severity_to_exit(overall))
}

fn severity_to_exit(s: Severity) -> i32 {
    match s {
        Severity::Pass => 0,
        // A skipped check did not run (e.g. an environment-gated probe) — it is
        // not a failure, so it maps to success like Pass.
        Severity::Skipped => 0,
        Severity::Warn => 1,
        Severity::Blocker => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_to_exit_pinned() {
        // Shell scripts depend on these exact codes; pin them.
        assert_eq!(severity_to_exit(Severity::Pass), 0);
        assert_eq!(severity_to_exit(Severity::Warn), 1);
        assert_eq!(severity_to_exit(Severity::Blocker), 2);
    }

    /// NA2-B-001: supplying `--policy-bundle` must NOT silently
    /// produce an all-Pass, exit-0 report. With a bundle path set but
    /// no decode wiring, the doctor emits a Warn, so the overall exit
    /// code is 1 (non-zero), not 0.
    #[tokio::test]
    async fn supplying_policy_bundle_does_not_silently_pass() {
        let args = DoctorArgs {
            policy_bundle: Some(PathBuf::from("/nonexistent/bundle.cbor")),
            model_path: None,
            expected_model_sha256: None,
            tla_specs_dir: None,
            json: true,
        };
        let exit = run(args).await.expect("doctor run");
        assert_ne!(exit, 0, "a supplied --policy-bundle must not yield exit 0");
    }

    /// Tripwire: every declared `DoctorArgs` field must be read on
    /// some code path in `run`. This destructure forces the test to
    /// name every field; if a new field is added it fails to compile
    /// until it is accounted for here (and, by review, wired in
    /// `run`). It guards against a repeat of the `let _ =
    /// args.policy_bundle;` silent-discard bug.
    #[test]
    fn every_doctor_args_field_is_accounted_for() {
        let args = DoctorArgs {
            policy_bundle: Some(PathBuf::from("p")),
            model_path: Some(PathBuf::from("m")),
            expected_model_sha256: Some("s".into()),
            tla_specs_dir: Some(PathBuf::from("t")),
            json: true,
        };
        // Exhaustive destructure — compile error if a field is added
        // without being considered here.
        let DoctorArgs {
            policy_bundle,
            model_path,
            expected_model_sha256,
            tla_specs_dir,
            json,
        } = args;
        assert!(policy_bundle.is_some());
        assert!(model_path.is_some());
        assert!(expected_model_sha256.is_some());
        assert!(tla_specs_dir.is_some());
        assert!(json);
    }
}
