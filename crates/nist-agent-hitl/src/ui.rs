//! Slint UI binding for the HITL surfaces. Feature-gated.
//!
//! `feat-ui` pulls in the slint runtime + the .slint codegen
//! output via `slint::include_modules!()`. The Rust-side
//! conversions translate between the headless render models
//! (`ApprovalQueueView`, `CapsuleInspectorView`) and the
//! Slint-side visual structs (`ApprovalRowVisual`,
//! `InspectorFieldVisual`).

slint::include_modules!();

use crate::{ApprovalQueueView, CapsuleInspectorView};

impl ApprovalQueueView {
    /// Convert the render model into the Slint-side visual model
    /// for `ApprovalQueuePane.rows`. The integration site
    /// converts the resulting `Vec<ApprovalRowVisual>` into a
    /// `slint::VecModel<ApprovalRowVisual>` via
    /// `slint::ModelRc::new`.
    pub fn to_visual(&self) -> Vec<ApprovalRowVisual> {
        self.rows
            .iter()
            .map(|r| ApprovalRowVisual {
                call_id: r.call_id.clone().into(),
                call_display: r.call_display.clone().into(),
                description: r.description.clone().into(),
                severity_label: format!("{:?}", r.severity).to_lowercase().into(),
                args_pretty: r.args_pretty.clone().into(),
                current_sigs: r.current_signatures.join(", ").into(),
                roles_still_required: r.roles_still_required.join(", ").into(),
            })
            .collect()
    }
}

impl CapsuleInspectorView {
    /// Convert the render model into the Slint-side visual model
    /// for `CapsuleInspectorPane.fields`.
    pub fn to_visual(&self) -> Vec<InspectorFieldVisual> {
        self.fields
            .iter()
            .map(|f| InspectorFieldVisual {
                id: f.id.clone().into(),
                label: f.label.clone().into(),
                value: f.value.clone().into(),
                viewed: f.viewed,
            })
            .collect()
    }
}
