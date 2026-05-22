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
}

pub async fn run(args: DaemonArgs) -> Result<i32> {
    let config = match (args.config, args.smoke) {
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
