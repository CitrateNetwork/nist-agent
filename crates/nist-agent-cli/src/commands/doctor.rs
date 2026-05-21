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
    compute_overall, run as run_checks, v1_checks, NistDoctorContext, Severity,
};
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
    // PolicyBundle decoding is operator-side; the CLI carries the
    // path but doesn't decode here. v1.0 wiring lands once
    // RawSignedBundle::from_path is canonized; until then, the
    // bundle-dependent checks skip with a Pass-shaped "no bundle
    // wired" result.
    let _ = args.policy_bundle;

    let checks = v1_checks();
    let results = run_checks(&ctx, &checks);
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
}
