//! `citrate-agent wizard` — launch the Slint concierge.
//!
//! Behind `--features feat-ui` at build time. When the feature
//! is disabled the subcommand still exists (so `--help` is
//! consistent) but emits a helpful error pointing at the
//! feature flag.

use anyhow::Result;

#[cfg(feature = "feat-ui")]
pub fn run() -> Result<i32> {
    println!("citrate-agent wizard: Slint concierge surface");
    println!(
        "note: the wizard's headless render models are wired (S-10a); the runtime \
         Slint window-up integration lands in S-12c. For now, use \
         `cargo test -p nist-agent-wizard --features feat-ui` to exercise the surface."
    );
    Ok(0)
}

#[cfg(not(feature = "feat-ui"))]
pub fn run() -> Result<i32> {
    eprintln!(
        "wizard requires --features feat-ui at build time. Rebuild with:\n  \
         cargo build --bin citrate-agent --features feat-ui"
    );
    Ok(2)
}
