//! `Checkpoint` — the persisted shape of a paused agent loop.
//!
//! RFC §5.4 specifies the four fields a checkpoint MUST carry:
//! the proposed call, its arguments, the capsule context, the
//! agent's local state, and the inputs that led to the proposal.
//! Together those let the loop resume from exactly the point of
//! interrupt with deterministic semantics.

use crate::action::Action;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

/// A snapshot of the agent loop at the moment it proposes an action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Stable id (the SHA-256 of the action's `proposal_hash` plus
    /// the conversation prefix). Used to retrieve the checkpoint
    /// on resume; also the audit-record's identifier for this
    /// pause/resume cycle.
    pub id: String,

    /// The action being proposed.
    pub action: Action,

    /// Capsule manifest snapshot (caller-supplied opaque string,
    /// typically a TOML or JSON serialization). Surfaced to the
    /// approver for the Capsule Inspector view (RFC §8.3).
    pub capsule_manifest_snapshot: String,

    /// The agent's local state at pause — opaque to the
    /// checkpoint store. JSON-serialized so the surface can
    /// inspect it without the agent's binary being present.
    pub agent_local_state_json: String,

    /// The conversation prefix that led to this proposal —
    /// truncated to the operator-configured window (default 4 KB).
    pub conversation_prefix: String,

    /// UTC timestamp (RFC 3339) when the checkpoint was written.
    pub created_at: String,
}

impl Checkpoint {
    /// Derive the checkpoint id from the action's proposal hash
    /// plus a conversation-prefix hash. Pure function; idempotent.
    pub fn derive_id(action: &Action, conversation_prefix: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(action.proposal_hash);
        h.update(b"||");
        h.update(conversation_prefix.as_bytes());
        hex::encode(h.finalize())
    }
}

/// Storage for checkpoints. In production, the operator's policy
/// bundle declares a filesystem path or an S3 bucket. For tests
/// and embedded use, the in-memory implementation below suffices.
#[async_trait]
pub trait CheckpointStore: Send + Sync {
    /// Persist `cp`. Returns an error iff durable storage failed —
    /// per RFC §5.4, the loop MUST NOT proceed without a successful
    /// write (audit chain integrity).
    async fn put(&self, cp: &Checkpoint) -> Result<(), String>;

    /// Retrieve by id. Returns `None` if not present.
    async fn get(&self, id: &str) -> Result<Option<Checkpoint>, String>;

    /// Securely delete after the action completes or is rejected.
    /// Idempotent — repeated deletes are no-ops, not errors.
    async fn remove(&self, id: &str) -> Result<(), String>;
}

/// In-memory store — for tests, embedded WASM hosts where
/// persistence isn't needed, and the bundled Slint concierge's
/// first-run flow (where the policy bundle hasn't been signed yet).
pub struct InMemoryCheckpointStore {
    inner: Mutex<HashMap<String, Checkpoint>>,
}

impl Default for InMemoryCheckpointStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryCheckpointStore {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl CheckpointStore for InMemoryCheckpointStore {
    async fn put(&self, cp: &Checkpoint) -> Result<(), String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| format!("checkpoint store poisoned: {e}"))?;
        guard.insert(cp.id.clone(), cp.clone());
        Ok(())
    }

    async fn get(&self, id: &str) -> Result<Option<Checkpoint>, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|e| format!("checkpoint store poisoned: {e}"))?;
        Ok(guard.get(id).cloned())
    }

    async fn remove(&self, id: &str) -> Result<(), String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| format!("checkpoint store poisoned: {e}"))?;
        guard.remove(id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;

    #[tokio::test]
    async fn put_get_remove_round_trip() {
        let store = InMemoryCheckpointStore::new();
        let action = Action::new("cap", "fn", r#"{"x":1}"#);
        let cp = Checkpoint {
            id: Checkpoint::derive_id(&action, "hi"),
            action,
            capsule_manifest_snapshot: "manifest".into(),
            agent_local_state_json: "{}".into(),
            conversation_prefix: "hi".into(),
            created_at: "2026-05-21T00:00:00Z".into(),
        };
        store.put(&cp).await.expect("put");
        let got = store.get(&cp.id).await.expect("get").expect("present");
        assert_eq!(got.id, cp.id);
        store.remove(&cp.id).await.expect("remove");
        assert!(store.get(&cp.id).await.expect("get post-remove").is_none());
    }

    #[test]
    fn derive_id_is_deterministic() {
        let action = Action::new("cap", "fn", r#"{"x":1}"#);
        let id_a = Checkpoint::derive_id(&action, "conversation prefix");
        let id_b = Checkpoint::derive_id(&action, "conversation prefix");
        assert_eq!(id_a, id_b);
    }

    #[test]
    fn derive_id_differs_with_conversation_prefix() {
        // RFC §5.4 determinism property: same proposal but different
        // conversation prefix yields different checkpoint ids, so
        // resumption can never accidentally pick up a stale prefix.
        let action = Action::new("cap", "fn", r#"{"x":1}"#);
        let id_a = Checkpoint::derive_id(&action, "prefix a");
        let id_b = Checkpoint::derive_id(&action, "prefix b");
        assert_ne!(id_a, id_b);
    }
}
