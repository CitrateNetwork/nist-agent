//! `citrate-agent status` — minimal daemon status.
//!
//! v1 ships a build-time identity readout so operators can
//! confirm what binary they're running. The full live-daemon
//! status (queue depth, last anchor, doctor severity) lands
//! in S-12c once the daemon's IPC socket exists.

use anyhow::Result;

pub fn run() -> Result<i32> {
    println!("citrate-agent {}", env!("CARGO_PKG_VERSION"));
    println!("workspace: nist-agent");
    println!(
        "features: feat-ui={}, feat-model-llamacpp={}",
        cfg!(feature = "feat-ui"),
        cfg!(feature = "feat-model-llamacpp"),
    );
    println!("note: live-daemon status (queue depth, anchor lag) lands in S-12c");
    Ok(0)
}
