//! nist-agent-model — model resolver for the agent loop.
//!
//! RFC-CIT-AGENT-0001 §3.3 ("Network Posture and Air-Gap Discipline")
//! and §8.2 ("The Bundled Concierge Model") define the resolution
//! order the harness MUST follow:
//!
//!   1. local Ollama on port 11434
//!   2. local llama.cpp server on ports 8080 / 8000
//!   3. embedded GGUF via llama-cpp-4 bindings
//!   4. configured site-mirror
//!   5. on-chain registry pull (only if egress is permitted)
//!   6. external pull (only with explicit policy + SO signature)
//!
//! This crate implements (1) and (2) as the v1 default. The embedded
//! case (3) is gated behind a `feat-embedded` feature so the
//! heavyweight `llama-cpp-2`/native-build dep is opt-in — S-10 (Slint
//! concierge) needs it; the daemon and CLI can defer. The remote
//! sources (4–6) are operator-policy-driven and live in S-6's policy
//! bundle parsing, not here.
//!
//! Per ALIGNMENT.md's exception clause, the canonical home for these
//! types is `citrate_agent_core::model` (upstream). Because runtime
//! has not yet sequenced CIT-AGENT-3's model-resolver work, we land
//! locally and PR back; see ADR-004 for the migration plan.

pub mod embedded;
pub mod error;
pub mod hash;
pub mod ollama;
pub mod resolver;

pub use embedded::EmbeddedLlamaCpp;

pub use error::ModelError;
pub use hash::verify_sha256;
pub use ollama::OllamaClient;
pub use resolver::{Model, ModelConfig, ResolvedModel};

use async_trait::async_trait;
use futures::stream::BoxStream;

/// Streaming inference result. Each item is a chunk of generated
/// text (typically one token, but model-dependent — Ollama produces
/// multi-byte UTF-8 chunks that may not align on token boundaries).
pub type TokenStream<'a> = BoxStream<'a, Result<String, ModelError>>;

/// The minimum trait every resolved model implements. Mirrors what
/// upstream's `citrate_agent_core::model::Model` will define once
/// CIT-AGENT-3 lands the runtime-side version.
#[async_trait]
pub trait ModelBackend: Send + Sync {
    /// Operator-facing identifier (e.g. "ollama:gemma4:e2b",
    /// "llamacpp:8080", "embedded:gemma-4-e2b-it-Q4_K_M.gguf").
    fn id(&self) -> &str;

    /// One-shot inference. Returns the full completion.
    async fn infer(&self, prompt: &str) -> Result<String, ModelError>;

    /// Streaming inference. Caller drains the stream as the model
    /// produces chunks. The agent loop's token-by-token surface
    /// binds here.
    async fn infer_stream<'a>(&'a self, prompt: &'a str) -> Result<TokenStream<'a>, ModelError>;
}
