//! The `Agent` struct — the single loop that drives every surface.
//!
//! Lifecycle:
//!
//!   1. Caller creates the Agent with a model backend + checkpoint
//!      store.
//!   2. Caller invokes `step(input)`. The Agent runs the model on
//!      the input, accumulates tokens, and watches for proposed
//!      actions in the model's output.
//!   3. If an action is proposed, the Agent writes a Checkpoint to
//!      the store and returns `AgentOutcome::Pending`. Control
//!      returns to the harness, which surfaces the action to the
//!      HITL queue.
//!   4. The HITL queue eventually calls `resume(approval_payload)`
//!      with the surface's decision. The Agent matches the payload
//!      to the outstanding checkpoint and returns
//!      `AgentOutcome::Completed` with the action result, or
//!      `AgentOutcome::Rejected` if the surface refused.

use crate::action::{Action, ApprovalPayload};
use crate::checkpoint::{Checkpoint, CheckpointStore};
use crate::error::AgentLoopError;
use nist_agent_model::ModelBackend;
use std::sync::Arc;

/// Result of a step or resume call.
#[derive(Debug)]
pub enum AgentOutcome {
    /// The step produced a free-form text completion with no
    /// side-effecting action proposed. The harness emits the text
    /// to the surface and is done with this step.
    Text(String),

    /// The agent proposed an action. The Checkpoint is already
    /// written; the harness MUST route the action through the HITL
    /// queue and call `resume` when the surface returns an
    /// `ApprovalPayload`.
    Pending {
        checkpoint_id: String,
        action: Action,
    },

    /// The surface returned an approved payload; the agent has
    /// resumed from the checkpoint and completed.
    Completed {
        checkpoint_id: String,
        result: String,
    },

    /// The surface rejected. The checkpoint has been removed and
    /// no execution occurred. The harness emits a rejection audit
    /// record (handled in citrate-agent-core::audit).
    Rejected { checkpoint_id: String },
}

pub struct Agent {
    model: Arc<dyn ModelBackend>,
    checkpoint_store: Arc<dyn CheckpointStore>,
    /// Operator-configured conversation prefix for checkpoint id
    /// derivation. Empty string is a valid prefix (means no
    /// disambiguation needed in single-session deployments).
    conversation_prefix: String,
}

impl Agent {
    pub fn new(model: Arc<dyn ModelBackend>, checkpoint_store: Arc<dyn CheckpointStore>) -> Self {
        Self {
            model,
            checkpoint_store,
            conversation_prefix: String::new(),
        }
    }

    pub fn with_conversation_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.conversation_prefix = prefix.into();
        self
    }

    /// Drive the model on `input`. If the model's output contains a
    /// special-syntax action proposal, write a checkpoint and return
    /// `Pending`; otherwise return the completion as `Text`.
    ///
    /// The proposal syntax is `<<ACTION capsule.function args_json>>`
    /// appearing anywhere in the output. The action's args are
    /// everything after `function` up to the closing `>>`. This is
    /// the v0.1 wire format; future versions (RFC §11.3 v2) may
    /// extend it for sub-agent delegation.
    pub async fn step(&self, input: &str) -> Result<AgentOutcome, AgentLoopError> {
        let completion = self.model.infer(input).await?;

        if let Some(action) = parse_action(&completion) {
            let cp_id = Checkpoint::derive_id(&action, &self.conversation_prefix);
            let cp = Checkpoint {
                id: cp_id.clone(),
                action: action.clone(),
                capsule_manifest_snapshot: String::new(), // populated by caller via capsule loader
                agent_local_state_json: serde_json::json!({
                    "completion_so_far": completion,
                })
                .to_string(),
                conversation_prefix: self.conversation_prefix.clone(),
                created_at: now_iso(),
            };
            self.checkpoint_store
                .put(&cp)
                .await
                .map_err(AgentLoopError::Checkpoint)?;
            return Ok(AgentOutcome::Pending {
                checkpoint_id: cp_id,
                action,
            });
        }

        Ok(AgentOutcome::Text(completion))
    }

    /// Resume from a previously-written checkpoint with the
    /// surface's approval/rejection. The determinism property of
    /// RFC §5.4 requires: given the same `checkpoint_id` and the
    /// same `payload.proposal_hash`, the resumed agent MUST take
    /// the same first action it would have taken in any prior run.
    pub async fn resume(
        &self,
        checkpoint_id: &str,
        payload: ApprovalPayload,
    ) -> Result<AgentOutcome, AgentLoopError> {
        let cp = self
            .checkpoint_store
            .get(checkpoint_id)
            .await
            .map_err(AgentLoopError::Checkpoint)?
            .ok_or_else(|| {
                AgentLoopError::ApprovalMismatch(format!("no checkpoint with id {checkpoint_id}"))
            })?;

        if cp.action.proposal_hash != payload.proposal_hash {
            return Err(AgentLoopError::ApprovalMismatch(format!(
                "proposal hash mismatch: checkpoint {} vs payload {}",
                hex::encode(cp.action.proposal_hash),
                hex::encode(payload.proposal_hash),
            )));
        }

        if !payload.approved {
            self.checkpoint_store
                .remove(checkpoint_id)
                .await
                .map_err(AgentLoopError::Checkpoint)?;
            return Ok(AgentOutcome::Rejected {
                checkpoint_id: checkpoint_id.to_string(),
            });
        }

        // Approved path — the harness's capsule dispatcher executes
        // the action and returns the result. The Agent itself does
        // NOT execute capsules (capability enforcement is the
        // wasmtime linker's job per RFC §4.5). For this loop, we
        // signal completion; the caller (the harness or a
        // CapsuleDispatch wrapper) computes the real result.
        self.checkpoint_store
            .remove(checkpoint_id)
            .await
            .map_err(AgentLoopError::Checkpoint)?;
        Ok(AgentOutcome::Completed {
            checkpoint_id: checkpoint_id.to_string(),
            result: format!(
                "approved: {} {} (caller dispatches)",
                cp.action.capsule, cp.action.function
            ),
        })
    }
}

/// Parse the `<<ACTION ... >>` syntax out of model output.
///
/// Returns `Some(Action)` iff the output contains exactly one such
/// marker. Multiple markers is a sign of model misbehavior or
/// prompt injection — handled by returning `None` (the harness
/// treats the output as plain text and emits an audit warning).
fn parse_action(output: &str) -> Option<Action> {
    const OPEN: &str = "<<ACTION ";
    const CLOSE: &str = ">>";
    let start = output.find(OPEN)?;
    let body_start = start + OPEN.len();
    let close_rel = output[body_start..].find(CLOSE)?;
    let body = &output[body_start..body_start + close_rel];

    // Confirm no second `<<ACTION ` marker after the close.
    let after = &output[body_start + close_rel + CLOSE.len()..];
    if after.contains(OPEN) {
        return None;
    }

    // body = "capsule.function args_json"
    // Allow whitespace between function and args_json.
    let mut parts = body.splitn(2, ' ');
    let head = parts.next()?.trim();
    let (capsule, function) = head.split_once('.')?;
    let args_json = parts.next().unwrap_or("{}").trim();
    Some(Action::new(capsule, function, args_json))
}

fn now_iso() -> String {
    // SystemTime → RFC 3339-ish. Avoiding a chrono dep; we get a
    // millisecond timestamp and format it manually. Determinism is
    // not required for this field (it's audit metadata, not
    // proposal-hashed).
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("@{secs}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checkpoint::InMemoryCheckpointStore;
    use async_trait::async_trait;
    use futures::stream::BoxStream;
    use nist_agent_model::{ModelError, TokenStream};
    use std::sync::Arc;

    /// Test double: a model that returns a fixed completion.
    struct FixedCompletionModel(String);

    #[async_trait]
    impl ModelBackend for FixedCompletionModel {
        fn id(&self) -> &str {
            "test:fixed"
        }
        async fn infer(&self, _prompt: &str) -> Result<String, ModelError> {
            Ok(self.0.clone())
        }
        async fn infer_stream<'a>(
            &'a self,
            _prompt: &'a str,
        ) -> Result<TokenStream<'a>, ModelError> {
            let s: BoxStream<'a, Result<String, ModelError>> =
                Box::pin(futures::stream::once(async { Ok(self.0.clone()) }));
            Ok(s)
        }
    }

    fn make_agent(completion: &str) -> Agent {
        Agent::new(
            Arc::new(FixedCompletionModel(completion.to_string())),
            Arc::new(InMemoryCheckpointStore::new()),
        )
    }

    #[tokio::test]
    async fn step_returns_text_when_no_action_in_output() {
        let agent = make_agent("the answer is 42");
        match agent.step("what is the answer?").await.expect("step") {
            AgentOutcome::Text(t) => assert_eq!(t, "the answer is 42"),
            other => panic!("expected Text, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn step_returns_pending_when_output_contains_action() {
        let agent = make_agent(r#"plan: <<ACTION fs.read {"path":"/tmp/x"}>> then ..."#);
        match agent.step("read tmp").await.expect("step") {
            AgentOutcome::Pending { action, .. } => {
                assert_eq!(action.capsule, "fs");
                assert_eq!(action.function, "read");
                assert_eq!(action.args_json, r#"{"path":"/tmp/x"}"#);
            }
            other => panic!("expected Pending, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn pending_writes_a_checkpoint() {
        let store = Arc::new(InMemoryCheckpointStore::new());
        let agent = Agent::new(
            Arc::new(FixedCompletionModel(r#"<<ACTION cap.fn {}>>"#.to_string())),
            store.clone(),
        );
        let outcome = agent.step("hello").await.expect("step");
        match outcome {
            AgentOutcome::Pending { checkpoint_id, .. } => {
                let cp = store
                    .get(&checkpoint_id)
                    .await
                    .expect("get")
                    .expect("present");
                assert_eq!(cp.action.capsule, "cap");
                assert_eq!(cp.action.function, "fn");
            }
            other => panic!("expected Pending, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn resume_with_matching_payload_completes() {
        let agent = make_agent(r#"<<ACTION cap.fn {}>>"#);
        let outcome = agent.step("go").await.expect("step");
        let (id, hash) = match outcome {
            AgentOutcome::Pending {
                checkpoint_id,
                action,
            } => (checkpoint_id, action.proposal_hash),
            other => panic!("expected Pending, got {other:?}"),
        };
        let payload = ApprovalPayload {
            proposal_hash: hash,
            approved: true,
            signatures: vec![],
        };
        match agent.resume(&id, payload).await.expect("resume") {
            AgentOutcome::Completed { checkpoint_id, .. } => assert_eq!(checkpoint_id, id),
            other => panic!("expected Completed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn resume_with_rejection_removes_checkpoint() {
        let store = Arc::new(InMemoryCheckpointStore::new());
        let agent = Agent::new(
            Arc::new(FixedCompletionModel(r#"<<ACTION cap.fn {}>>"#.to_string())),
            store.clone(),
        );
        let outcome = agent.step("go").await.expect("step");
        let (id, hash) = match outcome {
            AgentOutcome::Pending {
                checkpoint_id,
                action,
            } => (checkpoint_id, action.proposal_hash),
            other => panic!("expected Pending, got {other:?}"),
        };
        let payload = ApprovalPayload {
            proposal_hash: hash,
            approved: false,
            signatures: vec![],
        };
        match agent.resume(&id, payload).await.expect("resume") {
            AgentOutcome::Rejected { checkpoint_id } => assert_eq!(checkpoint_id, id),
            other => panic!("expected Rejected, got {other:?}"),
        }
        // Checkpoint must be gone.
        assert!(store.get(&id).await.expect("get").is_none());
    }

    #[tokio::test]
    async fn resume_with_wrong_hash_errors() {
        let agent = make_agent(r#"<<ACTION cap.fn {}>>"#);
        let outcome = agent.step("go").await.expect("step");
        let id = match outcome {
            AgentOutcome::Pending { checkpoint_id, .. } => checkpoint_id,
            other => panic!("expected Pending, got {other:?}"),
        };
        let payload = ApprovalPayload {
            proposal_hash: [0xff; 32],
            approved: true,
            signatures: vec![],
        };
        let err = agent.resume(&id, payload).await.expect_err("must error");
        match err {
            AgentLoopError::ApprovalMismatch(msg) => {
                assert!(msg.contains("proposal hash mismatch"))
            }
            other => panic!("expected ApprovalMismatch, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn determinism_property_same_input_same_hash() {
        // RFC §5.4 determinism: same checkpoint state + same
        // approval payload → same first action. We exercise the
        // forward direction here (same input → same proposal hash);
        // the resume direction is exercised by the round-trip test
        // above.
        let agent_a = make_agent(r#"<<ACTION cap.fn {"x":1}>>"#);
        let agent_b = make_agent(r#"<<ACTION cap.fn {"x":1}>>"#);
        let a = match agent_a.step("go").await.expect("step") {
            AgentOutcome::Pending { action, .. } => action.proposal_hash,
            o => panic!("{o:?}"),
        };
        let b = match agent_b.step("go").await.expect("step") {
            AgentOutcome::Pending { action, .. } => action.proposal_hash,
            o => panic!("{o:?}"),
        };
        assert_eq!(a, b);
    }

    #[test]
    fn parse_action_returns_none_on_plain_text() {
        assert!(parse_action("hello world").is_none());
    }

    #[test]
    fn parse_action_returns_none_on_multiple_markers() {
        let out = "<<ACTION a.b {}>> and <<ACTION c.d {}>>";
        assert!(parse_action(out).is_none());
    }

    #[test]
    fn parse_action_extracts_first_marker() {
        let a = parse_action("<<ACTION cap.fn {}>>").expect("parsed");
        assert_eq!(a.capsule, "cap");
        assert_eq!(a.function, "fn");
    }
}
