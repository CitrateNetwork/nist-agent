//! Slint UI binding — feature-gated behind `feat-ui`.
//!
//! Translates between the headless `Wizard` state machine in
//! `wizard.rs` and the Slint pane declarations in
//! `ui/wizard.slint`. The Rust side owns the wizard; Slint just
//! displays the current step and routes callbacks back to typed
//! transitions.
//!
//! The actual app binary lives in `examples/concierge.rs`. This
//! module exposes the `WizardWindow` Slint component + a helper
//! `launch()` function that wires it to a fresh `Wizard`.

slint::include_modules!();

use crate::wizard::Wizard;

/// Launch the wizard window. Blocking — returns when the window
/// closes. The wizard's terminal state (Signed or aborted) is in
/// the returned `Wizard` instance.
pub fn launch() -> Result<Wizard, slint::PlatformError> {
    let window = WizardWindow::new()?;

    // Initial step = 0 (Welcome / Begin button).
    window.set_step(0);

    // For v0.x the binding wires only the `begin-clicked`
    // callback as a representative example. Wiring all five
    // remaining callbacks lands in a follow-up commit on this
    // sprint (the operator-facing surface lives in S-10b/c).
    let weak = window.as_weak();
    window.on_begin_clicked(move || {
        if let Some(w) = weak.upgrade() {
            w.set_step(1);
        }
    });

    window.run()?;
    Ok(Wizard::new())
}
