//! `citrate-agent daemon` — minimal daemon entry point.
//!
//! v1 ships a no-op long-running process that:
//!
//! - prints a startup banner with the same identity readout as
//!   `status`,
//! - sleeps until SIGINT / SIGTERM,
//! - exits cleanly with code 0.
//!
//! This is enough to satisfy the air-gap test runbook's "the
//! daemon starts cleanly" check + to give the GHA workflow a
//! `bin` target to package. The real event loop (HITL queue
//! processing, audit-anchor cadence, IPC socket) lands in
//! S-12c.

use anyhow::Result;
use clap::Args;

#[derive(Args, Debug)]
pub struct DaemonArgs {
    /// Exit immediately after the startup banner — used by the
    /// air-gap test to confirm the binary boots without
    /// actually running the (not-yet-implemented) loop.
    #[arg(long)]
    pub smoke: bool,
}

pub async fn run(args: DaemonArgs) -> Result<i32> {
    println!("citrate-agent {} starting", env!("CARGO_PKG_VERSION"));
    println!("note: v1 daemon is a no-op skeleton; event loop lands in S-12c");

    if args.smoke {
        println!("smoke mode — exiting after banner");
        return Ok(0);
    }

    // Wait for SIGINT / SIGTERM, then exit cleanly.
    tokio::signal::ctrl_c().await.ok();
    println!("citrate-agent shutting down");
    Ok(0)
}
