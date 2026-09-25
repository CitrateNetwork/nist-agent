//! nist-agent-loop — the single-loop agent core.
//!
//! RFC-CIT-AGENT-0001 §3.1 ("Agent Loop") and §5.4 ("State-Managed
//! Interrupt") together define:
//!
//! 1. **One Agent type.** A single struct drives every surface
//!    (CLI, daemon, WASM host, Slint app). No per-surface forks.
//! 2. **Token-by-token streaming.** Output flows to the surface as
//!    the model produces it.
//! 3. **State-managed interrupt for HIC.** When the agent proposes
//!    an `Action` (a side-effecting call), the loop persists the
//!    full proposal context to the operator-configured
//!    `CheckpointStore`, returns control, and waits to be resumed
//!    with an approval payload.
//! 4. **Deterministic resumption.** Given the same checkpoint state
//!    and the same approval payload, the resumed agent MUST produce
//!    the same first action it would have produced. This determinism
//!    is what makes the audit chain verifiable offline.
//!
//! The canonical home for these types is
//! `citrate_agent_core::agent` (upstream). Because runtime has not
//! yet sequenced CIT-AGENT-3's agent-loop work, we land locally and
//! PR back; see ADR-004.

pub mod action;
pub mod agent;
pub mod checkpoint;
pub mod error;

pub use action::{Action, ApprovalPayload};
pub use agent::{Agent, AgentOutcome};
pub use checkpoint::{Checkpoint, CheckpointStore, InMemoryCheckpointStore};
pub use error::AgentLoopError;
