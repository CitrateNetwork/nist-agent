//! Interactive chat surface render model — RFC §8.3.
//!
//! The chat surface drives `nist_agent_loop::Agent::step()` with
//! operator prompts and renders the agent's reply. Three lifecycle
//! states matter to the UI:
//!
//! 1. **Streaming.** Tokens arrive over time; the UI appends them
//!    to the in-progress assistant turn.
//! 2. **Paused at action.** The model emitted an
//!    `<<ACTION ...>>` proposal — the loop returns
//!    `AgentOutcome::Pending` with the action and a checkpoint id.
//!    The chat pane displays the proposal inline (so the operator
//!    sees context for why they're being asked to sign) and waits
//!    for the HIC queue (S-10b) to return a decision.
//! 3. **Resumed.** The queue returned a decision; the harness calls
//!    `Agent::resume(...)`. The chat pane reads either `Completed`
//!    or `Rejected` and continues from there.
//!
//! Trajectory export (Hermes-style training data per RFC §12 Q3) is
//! a tier-critical capability — only surfaced when the active
//! overlay set permits it. FedRAMP High deployments treat
//! trajectories as covered data and forbid export.

use nist_agent_prelude::Overlay;
use serde::{Deserialize, Serialize};

use crate::error::MarketplaceUiError;

/// Which side of the conversation produced this turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChatRole {
    Operator,
    Agent,
    /// System-emitted note (e.g. "Action <<x>> approved by Operator
    /// at 2026-05-21T15:04:05Z"). These are always rendered but
    /// never sent back to `Agent::step()`.
    System,
}

/// One completed (or in-progress) turn in the chat transcript.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatTurn {
    pub role: ChatRole,
    pub content: String,
    /// ISO-8601 timestamp. Harness fills this in; render model is
    /// agnostic to clock source.
    pub timestamp_iso: String,
}

/// Streaming state of the current (in-progress) agent turn, if any.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum ChatStreamState {
    /// No turn in progress. Operator's input box is enabled.
    Idle,
    /// Tokens are arriving. The `partial` field is the
    /// accumulated text so far; the UI re-renders the in-progress
    /// turn every time this updates.
    Streaming { partial: String },
    /// The model emitted `<<ACTION ...>>` — the harness wrote a
    /// checkpoint, returned `Pending`, and routed the action to the
    /// HIC queue. The chat surface displays the proposal inline
    /// and disables further operator input until the queue returns.
    PausedAtAction {
        checkpoint_id: String,
        action_summary: String,
    },
}

/// User-driven action returned from the chat UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatAction {
    /// Send a fresh operator prompt to `Agent::step()`.
    Submit { text: String },
    /// HIC queue returned an approval payload, harness should
    /// call `Agent::resume(checkpoint_id, payload)`. The chat
    /// surface forwards the queue's decision rather than producing
    /// the payload itself; this variant is the UI's "go ahead"
    /// signal.
    Resume { checkpoint_id: String },
    /// Operator cancelled the in-progress turn (Esc / Cancel
    /// button). The harness should drop the streaming model call.
    CancelStreaming,
    /// Export the trajectory as Hermes-style training data. Only
    /// emitted when `ChatSessionView::trajectory_export_allowed`
    /// is true.
    ExportTrajectory,
}

/// Top-level chat session view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatSessionView {
    pub turns: Vec<ChatTurn>,
    pub stream_state: ChatStreamState,
    /// True iff every active overlay permits Hermes-style
    /// trajectory export. Drives the visibility of the export
    /// hint in the toolbar. RFC §12 Q3 calls this out as a
    /// tier-critical capability.
    pub trajectory_export_allowed: bool,
    /// Operator's current draft text (re-displayed across re-renders
    /// so input isn't lost when the view re-builds).
    pub input_draft: String,
}

impl ChatSessionView {
    /// Construct an empty session. `active_overlays` controls the
    /// trajectory-export hint; if any active overlay forbids it,
    /// the hint stays hidden.
    pub fn new(active_overlays: &[Overlay]) -> Self {
        Self {
            turns: Vec::new(),
            stream_state: ChatStreamState::Idle,
            trajectory_export_allowed: Self::compute_export_allowed(active_overlays),
            input_draft: String::new(),
        }
    }

    /// Per RFC §12 Q3: trajectory export is forbidden under any
    /// overlay that treats trajectories as covered data. FedRAMP
    /// High is the load-bearing v1.0 example.
    fn compute_export_allowed(active_overlays: &[Overlay]) -> bool {
        !active_overlays
            .iter()
            .any(|o| matches!(o, Overlay::FedrampHigh))
    }

    /// Append a finalized turn (operator submit, agent completion,
    /// or a system note). The caller is responsible for filling in
    /// the timestamp.
    pub fn push_turn(&mut self, turn: ChatTurn) {
        self.turns.push(turn);
    }

    /// Update the streaming state. Harness drives this from the
    /// model backend's token stream.
    pub fn set_stream(&mut self, state: ChatStreamState) {
        self.stream_state = state;
    }

    /// Mark the session paused at an action proposal. Convenience
    /// wrapper that's clearer than building the enum directly at
    /// call sites.
    pub fn pause_at_action(
        &mut self,
        checkpoint_id: impl Into<String>,
        action_summary: impl Into<String>,
    ) {
        self.stream_state = ChatStreamState::PausedAtAction {
            checkpoint_id: checkpoint_id.into(),
            action_summary: action_summary.into(),
        };
    }

    /// Whether the operator can submit a new prompt right now.
    /// False while streaming and while paused at an action.
    pub fn input_enabled(&self) -> bool {
        matches!(self.stream_state, ChatStreamState::Idle)
    }

    /// Validate a resume action. Returns the checkpoint id if the
    /// session is in a paused state matching the supplied id.
    /// Surfaces a programmer error if the harness mis-wires the
    /// resume path.
    pub fn validate_resume(&self, checkpoint_id: &str) -> Result<&str, MarketplaceUiError> {
        match &self.stream_state {
            ChatStreamState::PausedAtAction {
                checkpoint_id: cp, ..
            } if cp == checkpoint_id => Ok(cp.as_str()),
            _ => Err(MarketplaceUiError::ResumeWithoutPause),
        }
    }

    /// Validate a trajectory-export action. Returns the turn
    /// snapshot the harness should hand to the export sink.
    pub fn validate_export(&self) -> Result<&[ChatTurn], MarketplaceUiError> {
        if !self.trajectory_export_allowed {
            return Err(MarketplaceUiError::TrajectoryExportForbidden);
        }
        Ok(&self.turns)
    }

    /// Fixture: two-turn conversation with the session idle.
    pub fn fixture() -> Self {
        let mut v = Self::new(&[Overlay::CmmcL3, Overlay::Ferpa]);
        v.push_turn(ChatTurn {
            role: ChatRole::Operator,
            content: "redact PII from the gradebook export".into(),
            timestamp_iso: "2026-05-21T15:00:00Z".into(),
        });
        v.push_turn(ChatTurn {
            role: ChatRole::Agent,
            content: "Proposing the redaction. The action proposal will require approval.".into(),
            timestamp_iso: "2026-05-21T15:00:02Z".into(),
        });
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_allowed_under_non_fedramp_overlays() {
        let v = ChatSessionView::new(&[Overlay::CmmcL3, Overlay::Ferpa, Overlay::HipaaHitech]);
        assert!(v.trajectory_export_allowed);
    }

    #[test]
    fn export_forbidden_when_fedramp_high_active() {
        // RFC §12 Q3 pin: any deployment with FedRAMP High in the
        // active set must hide the trajectory-export hint.
        let v = ChatSessionView::new(&[Overlay::CmmcL3, Overlay::FedrampHigh]);
        assert!(!v.trajectory_export_allowed);
        match v.validate_export() {
            Err(MarketplaceUiError::TrajectoryExportForbidden) => {}
            other => panic!("expected TrajectoryExportForbidden, got {other:?}"),
        }
    }

    #[test]
    fn input_disabled_while_streaming() {
        let mut v = ChatSessionView::new(&[Overlay::CmmcL3]);
        assert!(v.input_enabled());
        v.set_stream(ChatStreamState::Streaming {
            partial: "thinking…".into(),
        });
        assert!(!v.input_enabled());
        v.set_stream(ChatStreamState::Idle);
        assert!(v.input_enabled());
    }

    #[test]
    fn input_disabled_while_paused_at_action() {
        let mut v = ChatSessionView::new(&[Overlay::CmmcL3]);
        v.pause_at_action("cp-1", "redact_pii(gradebook.csv)");
        assert!(!v.input_enabled());
    }

    #[test]
    fn validate_resume_matches_paused_checkpoint_id() {
        let mut v = ChatSessionView::new(&[Overlay::CmmcL3]);
        v.pause_at_action("cp-1", "x");
        v.validate_resume("cp-1").expect("matching id");
        // Mismatched id is a programmer error.
        assert!(matches!(
            v.validate_resume("cp-2"),
            Err(MarketplaceUiError::ResumeWithoutPause)
        ));
    }

    #[test]
    fn validate_resume_rejects_when_session_not_paused() {
        let v = ChatSessionView::new(&[Overlay::CmmcL3]);
        assert!(matches!(
            v.validate_resume("cp-1"),
            Err(MarketplaceUiError::ResumeWithoutPause)
        ));
    }

    #[test]
    fn stream_state_serializes_with_tagged_state_field() {
        // PolicyBundle audit records snapshot the chat state when a
        // checkpoint is written; pin the wire form so a serde
        // refactor can't silently drift.
        let s = serde_json::to_string(&ChatStreamState::Idle).unwrap();
        assert!(s.contains("\"state\":\"idle\""));
        let s = serde_json::to_string(&ChatStreamState::Streaming {
            partial: "x".into(),
        })
        .unwrap();
        assert!(s.contains("\"state\":\"streaming\""));
        assert!(s.contains("\"partial\":\"x\""));
    }

    #[test]
    fn role_serializes_kebab_case() {
        let s = serde_json::to_string(&ChatRole::Operator).unwrap();
        assert_eq!(s, "\"operator\"");
    }

    #[test]
    fn fixture_session_has_two_turns_and_is_idle() {
        let v = ChatSessionView::fixture();
        assert_eq!(v.turns.len(), 2);
        assert_eq!(v.turns[0].role, ChatRole::Operator);
        assert_eq!(v.turns[1].role, ChatRole::Agent);
        assert!(v.input_enabled());
    }
}
