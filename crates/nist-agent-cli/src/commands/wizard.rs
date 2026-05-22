//! `citrate-agent wizard` — launch the Slint concierge.
//!
//! Behind `--features feat-ui` at build time. When the feature
//! is disabled the subcommand still exists (so `--help` is
//! consistent) but emits a helpful error pointing at the
//! feature flag.

use anyhow::Result;

#[cfg(feature = "feat-ui")]
pub fn run() -> Result<i32> {
    // The wizard's Slint window is in nist-agent-wizard::ui::launch.
    // Blocking call; returns when the window closes.
    match nist_agent_wizard::ui::launch() {
        Ok(_wizard) => Ok(0),
        Err(e) => {
            eprintln!("wizard window failed: {e}");
            Ok(2)
        }
    }
}

#[cfg(not(feature = "feat-ui"))]
pub fn run() -> Result<i32> {
    eprintln!(
        "wizard requires --features feat-ui at build time. Rebuild with:\n  \
         cargo build --bin citrate-agent --features feat-ui"
    );
    Ok(2)
}
