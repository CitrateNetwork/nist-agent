//! `citrate-agent daemon` — load config, bind IPC, run the
//! event loop until SIGINT / SIGTERM.
//!
//! Composition: this command owns argument parsing + config
//! loading + tracing init; the actual daemon lifecycle lives in
//! `nist-agent-daemon::Daemon`. The `--smoke` flag stops after
//! `bind()` so the air-gap test can confirm the daemon boots
//! without entering the run loop.

use anyhow::{Context, Result};
use clap::Args;
use nist_agent_daemon::{Daemon, DaemonConfig};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct DaemonArgs {
    /// Path to the daemon's TOML config. Required.
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Exit cleanly after IPC socket bind — used by the air-
    /// gap test to confirm the daemon comes up without
    /// entering the run loop. When set, --config is optional
    /// and a scratch-dir fixture is used.
    #[arg(long)]
    pub smoke: bool,

    /// Explicitly permit booting with no policy bundle (air-gap
    /// smoke mode). Forces `[policy] allow_no_policy = true` on the
    /// loaded config (NA2-B-002). Without a bundle path or this
    /// flag, the daemon refuses to start.
    #[arg(long)]
    pub insecure_no_policy: bool,

    /// Permit running the real IPC loop from a DEBUG build. Debug
    /// builds inherit the upstream HIC dev fail-open (self-minted
    /// signer keys accepted without a roster; NA2-B-023), so a
    /// production run must be a release build unless this flag is
    /// passed. Ignored for `--smoke` (which never serves approvals).
    #[arg(long)]
    pub insecure_dev: bool,
}

pub async fn run(args: DaemonArgs) -> Result<i32> {
    // NA2-B-023: a debug build inherits the upstream HIC dev
    // fail-open. Refuse the real run loop unless the operator
    // explicitly acknowledges it. `--smoke` is exempt (it exits
    // before serving anything).
    if cfg!(debug_assertions) && !args.smoke && !args.insecure_dev {
        eprintln!(
            "refusing to run the daemon from a DEBUG build (HIC dev fail-open, NA2-B-023); \
             build with --release or pass --insecure-dev to override"
        );
        return Ok(2);
    }

    let mut config = match (args.config.clone(), args.smoke) {
        (Some(path), _) => DaemonConfig::from_path(&path)
            .with_context(|| format!("load daemon config {}", path.display()))?,
        (None, true) => {
            let scratch = tempfile::Builder::new()
                .prefix("citrate-agent-smoke-")
                .tempdir()
                .context("create smoke scratch dir")?;
            // Leak the tempdir so the socket survives serve_once.
            // It cleans up on process exit.
            let path = scratch.keep();
            DaemonConfig::fixture(&path)
        }
        (None, false) => {
            eprintln!("--config is required (or use --smoke for a scratch run)");
            return Ok(2);
        }
    };

    // NA2-B-002: surface the no-policy opt-in as a CLI flag so an
    // operator does not have to hand-edit the TOML to reach the
    // (still explicit) air-gap smoke mode.
    if args.insecure_no_policy {
        config.policy.allow_no_policy = true;
    }

    let mut daemon = Daemon::prepare(config).context("prepare daemon")?;
    let socket = daemon.bind().context("bind IPC socket")?;
    println!(
        "citrate-agent daemon {} listening at {}",
        env!("CARGO_PKG_VERSION"),
        socket.display(),
    );

    if args.smoke {
        println!("smoke mode — IPC bound; exiting before run loop");
        return Ok(0);
    }

    daemon.run_until_signal().await.context("run loop")?;
    println!("citrate-agent daemon shutting down");
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A debug build refuses the real run loop unless the operator
    /// passes `--insecure-dev`; the refusal happens before any config
    /// is read (so a missing config path still yields exit 2, not a
    /// config error).
    #[cfg(debug_assertions)]
    #[tokio::test]
    async fn debug_build_refuses_run_loop_without_insecure_dev() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let args = DaemonArgs {
            config: Some(tmp.path().join("absent.toml")),
            smoke: false,
            insecure_no_policy: false,
            insecure_dev: false,
        };
        let code = run(args).await.expect("refusal is an exit code, not an error");
        assert_eq!(code, 2, "debug build must refuse without --insecure-dev");
    }

    /// With `--insecure-dev` the gate is passed and the command goes
    /// on to read the config (here: absent, so it errors).
    #[cfg(debug_assertions)]
    #[tokio::test]
    async fn insecure_dev_passes_the_debug_gate() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let args = DaemonArgs {
            config: Some(tmp.path().join("absent.toml")),
            smoke: false,
            insecure_no_policy: false,
            insecure_dev: true,
        };
        assert!(run(args).await.is_err(), "past the gate, an absent config must error");
    }
}
