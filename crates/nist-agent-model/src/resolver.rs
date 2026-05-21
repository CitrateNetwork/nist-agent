//! ResolvedModel — the resolution-order machinery per RFC §3.3.
//!
//! `Model::resolve()` walks the configured sources in order and
//! returns the first reachable one. The list of sources is itself
//! the configuration surface; in production it comes from the
//! signed PolicyBundle (S-6), and in tests it's passed directly.

use crate::error::ModelError;
use crate::ollama::OllamaClient;
use crate::ModelBackend;
use std::path::PathBuf;
use std::sync::Arc;

/// Operator-configured model resolution surface. The harness's
/// PolicyBundle deserializes into this shape.
#[derive(Debug, Clone)]
pub struct ModelConfig {
    /// Name the operator picked for the target model (e.g.
    /// `"gemma4:e2b"`). Each backend uses this differently:
    /// Ollama looks it up in `/api/tags`, llama.cpp ignores it (the
    /// server is preconfigured), embedded loads from `model_path`.
    pub model_name: String,

    /// Ollama discovery endpoints. Empty disables the Ollama path.
    pub ollama_endpoints: Vec<(String, u16)>,

    /// llama.cpp server discovery endpoints. Empty disables.
    pub llamacpp_endpoints: Vec<(String, u16)>,

    /// Embedded GGUF file path (NIST SI-7 hash-verified separately
    /// at the doctor pre-flight check). `None` disables.
    pub embedded_model_path: Option<PathBuf>,

    /// Operator policy: is egress permitted? When `false`, remote
    /// sources (HuggingFace pull, on-chain registry) are refused
    /// before any DNS lookup.
    pub egress_allowed: bool,
}

impl ModelConfig {
    /// The v1 default — Ollama on localhost:11434 only. Air-gap
    /// safe. Equivalent to the bundled Slint concierge's starting
    /// posture before the operator configures anything (RFC §8.2).
    pub fn localhost_ollama(model_name: impl Into<String>) -> Self {
        Self {
            model_name: model_name.into(),
            ollama_endpoints: vec![("127.0.0.1".to_string(), 11434)],
            llamacpp_endpoints: Vec::new(),
            embedded_model_path: None,
            egress_allowed: false,
        }
    }
}

/// A resolved model — one of the supported backends, ready for
/// inference. The variant tells the caller which source resolved.
#[derive(Debug)]
pub enum ResolvedModel {
    Ollama(OllamaClient),
    // LlamaCppServer(LlamaCppClient) — lands in a follow-up commit
    //   when we have a real llama.cpp HTTP shape to depend on.
    //   For now nist-agent-model ships Ollama as the only built-in
    //   discovery target; ResolvedModel is non_exhaustive to keep
    //   the surface forward-compatible.
}

impl ResolvedModel {
    /// Provider id (e.g. `"ollama:gemma4:e2b@127.0.0.1:11434"`).
    /// Doctor reports include this string verbatim.
    pub fn id(&self) -> &str {
        match self {
            ResolvedModel::Ollama(c) => c.id(),
        }
    }

    /// Run inference. Convenience wrapper around the `ModelBackend`
    /// trait method; consumers that want streaming use
    /// `infer_stream` on the inner backend directly.
    pub async fn infer(&self, prompt: &str) -> Result<String, ModelError> {
        match self {
            ResolvedModel::Ollama(c) => c.infer(prompt).await,
        }
    }
}

/// The resolver entry point. Walks `cfg`'s sources in declaration
/// order and returns the first reachable backend. The order matters
/// per RFC §3.3 — air-gap discipline says we prefer Ollama (local
/// daemon) over llama.cpp server (local daemon) over embedded
/// (in-process). Operators on a sidecar deployment may invert by
/// supplying an empty `ollama_endpoints` list.
pub struct Model;

impl Model {
    pub async fn resolve(cfg: &ModelConfig) -> Result<ResolvedModel, ModelError> {
        let mut tried = Vec::new();

        for (host, port) in &cfg.ollama_endpoints {
            tried.push(format!("ollama://{host}:{port}"));
            match OllamaClient::try_discover(host, *port, &cfg.model_name).await {
                Ok(Some(c)) => return Ok(ResolvedModel::Ollama(c)),
                Ok(None) => continue,
                Err(e) => {
                    tracing::warn!("ollama discovery {host}:{port} hard-failed: {e}");
                    continue;
                }
            }
        }

        // llamacpp_endpoints and embedded fall here when their
        // implementations land. Both are NoOp today.

        Err(ModelError::NoModelAvailable {
            tried: tried.join(", "),
        })
    }
}

/// Wrap a `ResolvedModel` in an `Arc<dyn ModelBackend>` for the
/// agent loop, which holds models as trait objects so it can swap
/// backends across reconfigurations.
impl ResolvedModel {
    pub fn into_arc_dyn(self) -> Arc<dyn ModelBackend> {
        match self {
            ResolvedModel::Ollama(c) => Arc::new(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn empty_config_returns_no_model_available() {
        let cfg = ModelConfig {
            model_name: "anything".into(),
            ollama_endpoints: vec![],
            llamacpp_endpoints: vec![],
            embedded_model_path: None,
            egress_allowed: false,
        };
        let err = Model::resolve(&cfg)
            .await
            .expect_err("empty cfg must error");
        match err {
            ModelError::NoModelAvailable { tried } => assert_eq!(tried, ""),
            other => panic!("expected NoModelAvailable, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn unreachable_ollama_falls_through_to_no_model() {
        // 192.0.2.x is RFC 5737 TEST-NET-1 — guaranteed unreachable.
        let cfg = ModelConfig {
            model_name: "anything".into(),
            ollama_endpoints: vec![("192.0.2.1".into(), 11434)],
            llamacpp_endpoints: vec![],
            embedded_model_path: None,
            egress_allowed: false,
        };
        let err = Model::resolve(&cfg)
            .await
            .expect_err("unreachable must error");
        match err {
            ModelError::NoModelAvailable { tried } => {
                assert!(tried.contains("ollama://192.0.2.1:11434"));
            }
            other => panic!("expected NoModelAvailable, got {other:?}"),
        }
    }

    #[test]
    fn localhost_ollama_is_air_gap_default() {
        let cfg = ModelConfig::localhost_ollama("gemma4:e2b");
        assert!(!cfg.egress_allowed);
        assert_eq!(cfg.ollama_endpoints, vec![("127.0.0.1".to_string(), 11434)]);
        assert!(cfg.embedded_model_path.is_none());
    }
}
