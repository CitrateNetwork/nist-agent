//! Approval queue render model.
//!
//! `ApprovalQueueView` converts upstream's `Vec<PendingView>`
//! into a UI-friendly list of `ApprovalRow` values. The UI
//! (Slint or CLI) iterates and renders; user actions
//! (Sign / Reject) come back through `ApprovalAction` which the
//! integration layer routes to `ApprovalQueue::approve()` /
//! `reject()`.

use citrate_agent_core::hitl::PendingView;
use serde::{Deserialize, Serialize};

/// Severity bucket for color-coding in the UI. Maps from
/// `PendingView.risk_level` strings the queue emits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApprovalRowSeverity {
    Low,
    Medium,
    High,
    Critical,
    /// Unknown / unmappable risk-level string. The UI displays
    /// this with a warning indicator and the raw string in
    /// `ApprovalRow::risk_level_raw`.
    Unknown,
}

impl ApprovalRowSeverity {
    /// Map a `PendingView.risk_level` string to a typed bucket.
    /// Comparison is case-insensitive so upstream's free-form
    /// string doesn't accidentally fall through.
    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "low" => Self::Low,
            "medium" | "med" => Self::Medium,
            "high" => Self::High,
            "critical" | "crit" => Self::Critical,
            _ => Self::Unknown,
        }
    }
}

/// One pending approval as rendered for the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalRow {
    /// Unique id from `ToolCall.call_id`; the UI uses this when
    /// routing Sign / Reject actions back to the queue.
    pub call_id: String,
    /// Capsule + function display string (e.g. "ferpa-redact:0.3.1 / redact").
    pub call_display: String,
    /// Operator-facing description (from PendingView.description).
    pub description: String,
    /// Typed severity.
    pub severity: ApprovalRowSeverity,
    /// Raw risk-level string from upstream — shown in detail
    /// view for transparency.
    pub risk_level_raw: String,
    /// Pretty-printed args (PendingView.args_pretty verbatim).
    pub args_pretty: String,
    /// Roles that have already signed (caller-supplied — upstream
    /// `ApprovalQueue` tracks signatures separately from PendingView).
    pub current_signatures: Vec<String>,
    /// Roles still required for quorum.
    pub roles_still_required: Vec<String>,
}

/// User action routed back to the queue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalAction {
    Sign {
        call_id: String,
        /// Signing identity (DID).
        signer_did: String,
        /// Role the signer is acting as.
        role: String,
    },
    Reject {
        call_id: String,
        /// Free-form reason — recorded in the audit trail.
        reason: String,
    },
}

impl ApprovalAction {
    pub fn call_id(&self) -> &str {
        match self {
            Self::Sign { call_id, .. } | Self::Reject { call_id, .. } => call_id,
        }
    }
}

/// The top-level view model. Built from the harness's snapshot
/// of the approval queue + signature state; the UI re-renders
/// when this changes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalQueueView {
    pub rows: Vec<ApprovalRow>,
    /// Operator-facing counter (e.g. "3 pending"). Computed.
    pub pending_count: usize,
}

impl ApprovalQueueView {
    /// Construct from upstream `PendingView` records + a parallel
    /// mapping of signature state.
    ///
    /// `signatures` is `Vec<(call_id, current, still_required)>`.
    /// Caller (the harness) sources these from its `ApprovalQueue`
    /// internal state; this constructor just zips them.
    pub fn from_pending(
        pendings: &[PendingView],
        signatures: &[(String, Vec<String>, Vec<String>)],
    ) -> Self {
        // Build a quick lookup so we don't make the harness
        // pre-sort signatures into the same order as pendings.
        // We key on PendingView.name because that's the closest
        // thing to a stable id in the upstream type (call_id is
        // internal to PendingEntry, not PendingView).
        let sig_by_name: std::collections::HashMap<&str, (Vec<String>, Vec<String>)> = signatures
            .iter()
            .map(|(name, cur, req)| (name.as_str(), (cur.clone(), req.clone())))
            .collect();

        let rows: Vec<ApprovalRow> = pendings
            .iter()
            .map(|p| {
                let (current_signatures, roles_still_required) = sig_by_name
                    .get(p.name.as_str())
                    .cloned()
                    .unwrap_or_default();
                ApprovalRow {
                    call_id: p.name.clone(),
                    call_display: p.name.clone(),
                    description: p.description.clone(),
                    severity: ApprovalRowSeverity::parse(&p.risk_level),
                    risk_level_raw: p.risk_level.clone(),
                    args_pretty: p.args_pretty.clone(),
                    current_signatures,
                    roles_still_required,
                }
            })
            .collect();
        let pending_count = rows.len();
        Self {
            rows,
            pending_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use citrate_agent_core::hitl::PendingView;
    use serde_json;

    fn pv(name: &str, risk: &str) -> PendingView {
        PendingView {
            name: name.into(),
            description: format!("desc for {name}"),
            risk_level: risk.into(),
            args_pretty: "{\n  \"x\": 1\n}".into(),
        }
    }

    #[test]
    fn severity_maps_canonical_strings() {
        assert_eq!(ApprovalRowSeverity::parse("low"), ApprovalRowSeverity::Low);
        assert_eq!(
            ApprovalRowSeverity::parse("Medium"),
            ApprovalRowSeverity::Medium
        );
        assert_eq!(
            ApprovalRowSeverity::parse("HIGH"),
            ApprovalRowSeverity::High
        );
        assert_eq!(
            ApprovalRowSeverity::parse("critical"),
            ApprovalRowSeverity::Critical
        );
    }

    #[test]
    fn severity_unknown_for_unmappable_string() {
        // Defensive: upstream's PendingView.risk_level is free-form;
        // an unexpected value should produce Unknown, not panic.
        assert_eq!(
            ApprovalRowSeverity::parse("urgent"),
            ApprovalRowSeverity::Unknown
        );
        assert_eq!(ApprovalRowSeverity::parse(""), ApprovalRowSeverity::Unknown);
    }

    #[test]
    fn from_pending_builds_rows_in_order() {
        let pendings = vec![
            pv("redact_pii", "high"),
            pv("read_log", "low"),
            pv("anchor_root", "medium"),
        ];
        let sigs = vec![
            (
                "redact_pii".into(),
                vec!["Reviewer".into()],
                vec!["ComplianceOfficer".into()],
            ),
            ("read_log".into(), vec![], vec!["Operator".into()]),
        ];
        let v = ApprovalQueueView::from_pending(&pendings, &sigs);
        assert_eq!(v.rows.len(), 3);
        assert_eq!(v.pending_count, 3);
        assert_eq!(v.rows[0].call_id, "redact_pii");
        assert_eq!(v.rows[0].current_signatures, vec!["Reviewer".to_string()]);
        assert_eq!(
            v.rows[0].roles_still_required,
            vec!["ComplianceOfficer".to_string()]
        );
        // Row without signature info gets defaults (empty vecs).
        assert_eq!(v.rows[2].current_signatures, Vec::<String>::new());
    }

    #[test]
    fn empty_pending_produces_empty_view() {
        let v = ApprovalQueueView::from_pending(&[], &[]);
        assert_eq!(v.pending_count, 0);
        assert!(v.rows.is_empty());
    }

    #[test]
    fn action_call_id_round_trip() {
        let sign = ApprovalAction::Sign {
            call_id: "c1".into(),
            signer_did: "did:role:1".into(),
            role: "Operator".into(),
        };
        let reject = ApprovalAction::Reject {
            call_id: "c2".into(),
            reason: "data class violation".into(),
        };
        assert_eq!(sign.call_id(), "c1");
        assert_eq!(reject.call_id(), "c2");
    }

    #[test]
    fn approval_row_serializes_kebab_case_severity() {
        // PolicyBundle audit records embed ApprovalRow snapshots
        // for the persisted decision history; pin the on-wire
        // form so serde refactors don't drift.
        let row = ApprovalRow {
            call_id: "c1".into(),
            call_display: "cap.fn".into(),
            description: "d".into(),
            severity: ApprovalRowSeverity::Critical,
            risk_level_raw: "critical".into(),
            args_pretty: "{}".into(),
            current_signatures: vec![],
            roles_still_required: vec![],
        };
        let json = serde_json::to_string(&row).expect("ser");
        assert!(json.contains("\"severity\":\"critical\""));
    }
}
