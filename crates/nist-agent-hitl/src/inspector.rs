//! Capsule Inspector render model — RFC §8.3.
//!
//! Renders a capsule manifest as a list of typed
//! `InspectorField` values + tracks which fields the operator
//! has scrolled into view. The Install button stays disabled
//! until `fields_viewed == fields_total` — the load-bearing
//! AC-6 (Least Privilege) audit artifact per the feature
//! scenario "Install button is disabled until the operator
//! scrolls every field into view."

use crate::error::HitlUiError;
use nist_agent_prelude::Overlay;
use serde::{Deserialize, Serialize};

/// A row in the Capsule Inspector list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectorField {
    /// Stable id used for scroll-tracking. Same id from
    /// successive `Vec<InspectorField>` snapshots represents the
    /// same field.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Display value (already formatted for the UI).
    pub value: String,
    /// Whether the operator has scrolled this field into view.
    /// The UI updates this on viewport intersection events; the
    /// install gate reads it.
    pub viewed: bool,
}

impl InspectorField {
    pub fn new(id: impl Into<String>, label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value: value.into(),
            viewed: false,
        }
    }
}

/// One declared capability with display sugar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityRow {
    /// "network" / "filesystem" / "chain_calls" / "subagent_spawn".
    pub category: String,
    /// Operator-facing value (e.g. "none", "read:/data/students",
    /// "model_inference:0x0101", "false").
    pub display: String,
}

/// Signing-tier badge color/label combo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SigningTierBadge {
    Bundled,
    Managed,
    Workspace,
}

/// Per-overlay certification flag the manifest declares.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverlayCertification {
    pub overlay: Overlay,
    /// True = manifest claims compatibility; false = explicitly
    /// listed under `not_certified`.
    pub certified: bool,
}

/// Top-level Inspector view. Built from a parsed capsule manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleInspectorView {
    pub capsule_name: String,
    pub capsule_version: String,
    pub content_hash_hex: String,
    pub publisher_did: String,
    pub signing_tier: SigningTierBadge,
    pub capabilities: Vec<CapabilityRow>,
    pub data_class_reads: Vec<String>,
    pub data_class_writes: Vec<String>,
    pub data_class_emits: Vec<String>,
    pub risk_tier: String,
    pub required_roles: Vec<String>,
    pub overlay_certifications: Vec<OverlayCertification>,
    pub tla_spec_present: bool,
    pub tla_verification_status: String,
    pub gherkin_pass_count: u32,
    pub gherkin_total_count: u32,
    pub procedure_md: String,
    /// All fields as scroll-trackable rows. The order is
    /// deterministic so the Slint `ListView`'s viewport
    /// intersection signals can update the same field across
    /// re-renders.
    pub fields: Vec<InspectorField>,
}

impl CapsuleInspectorView {
    /// Total field count for the install-gate math.
    pub fn fields_total(&self) -> usize {
        self.fields.len()
    }

    /// How many fields the operator has scrolled into view.
    pub fn fields_viewed(&self) -> usize {
        self.fields.iter().filter(|f| f.viewed).count()
    }

    /// Mark a field viewed by id. Idempotent.
    pub fn mark_viewed(&mut self, id: &str) {
        if let Some(f) = self.fields.iter_mut().find(|f| f.id == id) {
            f.viewed = true;
        }
    }

    /// Returns Ok(()) iff every field has been viewed. RFC §8.3
    /// gate: the install button stays disabled until this passes.
    pub fn check_install_gate(&self) -> Result<(), HitlUiError> {
        let total = self.fields_total();
        let viewed = self.fields_viewed();
        if viewed < total {
            return Err(HitlUiError::InstallGateUnseenFields {
                unseen: total - viewed,
                total,
            });
        }
        Ok(())
    }

    /// Test fixture: a minimal capsule with deterministic
    /// fields. Used by tests + by `examples/inspector-headless`
    /// in S-12.
    pub fn fixture(name: &str) -> Self {
        let capsule_name = name.to_string();
        let capsule_version = "0.1.0".to_string();
        let content_hash_hex = "0".repeat(64);
        let publisher_did = "did:citrate:agent:0xfixture".to_string();
        let signing_tier = SigningTierBadge::Bundled;
        let capabilities = vec![
            CapabilityRow {
                category: "network".into(),
                display: "none".into(),
            },
            CapabilityRow {
                category: "filesystem".into(),
                display: "[]".into(),
            },
        ];
        let data_class_reads = vec!["PUBLIC".to_string()];
        let data_class_writes = vec![];
        let data_class_emits = vec!["PUBLIC".to_string()];
        let risk_tier = "low".to_string();
        let required_roles = vec!["Operator".to_string()];
        let overlay_certifications = vec![OverlayCertification {
            overlay: Overlay::CmmcL3,
            certified: true,
        }];
        let tla_spec_present = false;
        let tla_verification_status = "not_required".to_string();
        let gherkin_pass_count = 0;
        let gherkin_total_count = 0;
        let procedure_md = "## Hello\n\nA fixture capsule.".to_string();

        // Build the fields list in display order so scroll
        // tracking is deterministic.
        let fields = vec![
            InspectorField::new(
                "name-version",
                "Name + version",
                format!("{capsule_name} {capsule_version}"),
            ),
            InspectorField::new("content-hash", "Content hash", content_hash_hex.clone()),
            InspectorField::new("publisher", "Publisher DID", publisher_did.clone()),
            InspectorField::new(
                "signing-tier",
                "Signing tier",
                format!("{signing_tier:?}").to_lowercase(),
            ),
            InspectorField::new(
                "capabilities",
                "Capabilities",
                format!("{} declared", capabilities.len()),
            ),
            InspectorField::new(
                "data-classes",
                "Data classes",
                format!(
                    "reads={:?} writes={:?} emits={:?}",
                    data_class_reads, data_class_writes, data_class_emits
                ),
            ),
            InspectorField::new("risk-tier", "Risk tier", risk_tier.clone()),
            InspectorField::new(
                "required-roles",
                "Required roles",
                required_roles.join(", "),
            ),
            InspectorField::new(
                "overlays",
                "Overlay certifications",
                format!("{} declared", overlay_certifications.len()),
            ),
            InspectorField::new(
                "tla",
                "TLA+ verification",
                if tla_spec_present {
                    tla_verification_status.clone()
                } else {
                    "not_required".to_string()
                },
            ),
            InspectorField::new(
                "gherkin",
                "Gherkin pass rate",
                format!("{gherkin_pass_count}/{gherkin_total_count}"),
            ),
            InspectorField::new("procedure", "procedure.md", procedure_md.clone()),
        ];

        Self {
            capsule_name,
            capsule_version,
            content_hash_hex,
            publisher_did,
            signing_tier,
            capabilities,
            data_class_reads,
            data_class_writes,
            data_class_emits,
            risk_tier,
            required_roles,
            overlay_certifications,
            tla_spec_present,
            tla_verification_status,
            gherkin_pass_count,
            gherkin_total_count,
            procedure_md,
            fields,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_carries_every_required_field() {
        // RFC §8.3's minimum-display list:
        //   name + version, content_hash, publisher DID, signing
        //   tier, capabilities, data classes, risk tier,
        //   required roles, overlay certifications, TLA status,
        //   Gherkin pass rate, procedure.md (expandable).
        let v = CapsuleInspectorView::fixture("hello");
        let ids: Vec<&str> = v.fields.iter().map(|f| f.id.as_str()).collect();
        for required in [
            "name-version",
            "content-hash",
            "publisher",
            "signing-tier",
            "capabilities",
            "data-classes",
            "risk-tier",
            "required-roles",
            "overlays",
            "tla",
            "gherkin",
            "procedure",
        ] {
            assert!(
                ids.contains(&required),
                "missing field {required}; got {ids:?}"
            );
        }
    }

    #[test]
    fn install_gate_blocks_until_every_field_viewed() {
        let mut v = CapsuleInspectorView::fixture("hello");
        let total = v.fields_total();
        // Start: 0 viewed → InstallGateUnseenFields with unseen=total.
        match v.check_install_gate() {
            Err(HitlUiError::InstallGateUnseenFields { unseen, total: t }) => {
                assert_eq!(unseen, total);
                assert_eq!(t, total);
            }
            other => panic!("expected InstallGateUnseenFields, got {other:?}"),
        }
        // Mark each in turn; gate stays closed until the last.
        let ids: Vec<String> = v.fields.iter().map(|f| f.id.clone()).collect();
        for (i, id) in ids.iter().enumerate() {
            v.mark_viewed(id);
            let viewed = v.fields_viewed();
            assert_eq!(viewed, i + 1);
            if i + 1 < total {
                assert!(matches!(
                    v.check_install_gate(),
                    Err(HitlUiError::InstallGateUnseenFields { .. })
                ));
            }
        }
        // Last field viewed: gate opens.
        v.check_install_gate()
            .expect("after all fields viewed, gate should open");
    }

    #[test]
    fn mark_viewed_is_idempotent_and_unknown_id_no_op() {
        let mut v = CapsuleInspectorView::fixture("hello");
        v.mark_viewed("name-version");
        v.mark_viewed("name-version"); // again
        assert_eq!(v.fields_viewed(), 1);
        v.mark_viewed("does-not-exist"); // no-op
        assert_eq!(v.fields_viewed(), 1);
    }

    #[test]
    fn signing_tier_serializes_kebab_case() {
        let s = serde_json::to_string(&SigningTierBadge::Workspace).unwrap();
        assert_eq!(s, "\"workspace\"");
        let round: SigningTierBadge = serde_json::from_str("\"managed\"").expect("de");
        assert_eq!(round, SigningTierBadge::Managed);
    }

    #[test]
    fn overlay_certification_round_trips() {
        let c = OverlayCertification {
            overlay: Overlay::Ferpa,
            certified: true,
        };
        let s = serde_json::to_string(&c).unwrap();
        assert!(s.contains("\"overlay\":\"ferpa\""));
        assert!(s.contains("\"certified\":true"));
    }
}
