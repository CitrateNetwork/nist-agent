//! Embedded llama.cpp backend — resolver step 3 per RFC §3.3.
//!
//! Gated behind the `feat-model-llamacpp` Cargo feature so the
//! default build stays light (the `llama-cpp-2` dep pulls in a
//! C++ toolchain). Operators on an air-gap host enable the
//! feature, build once with the toolchain warm, and ship.
//!
//! The SI-7 hash check on the GGUF file runs *before* this
//! module loads anything — `EmbeddedLlamaCpp::load_verified()`
//! takes a pre-verified path so the verification gate cannot
//! be skipped from inside this module. The verifier itself
//! lives in [`crate::hash::verify_sha256`] and in
//! `nist-agent-release::ModelIntegrity`; callers MUST run one
//! of those before calling [`load_verified`].

use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use futures::stream::{self, BoxStream};

use crate::{ModelBackend, ModelError, TokenStream};

/// Embedded GGUF backend. Holds a llama.cpp model + context.
///
/// The `id` is derived from the GGUF filename and is what
/// `ResolvedModel::id()` returns + what doctor reports surface.
pub struct EmbeddedLlamaCpp {
    id: String,
    model_path: PathBuf,
    // The actual llama-cpp-2 handle is constructed lazily on
    // first inference so the doctor's "model resolves" check
    // doesn't pay the cost of loading the GGUF. Wrapped in
    // tokio::sync::Mutex so async callers don't deadlock.
    #[cfg(feature = "feat-model-llamacpp")]
    inner: tokio::sync::Mutex<Option<LlamaInner>>,
}

#[cfg(feature = "feat-model-llamacpp")]
struct LlamaInner {
    backend: llama_cpp_2::llama_backend::LlamaBackend,
    model: llama_cpp_2::model::LlamaModel,
}

impl std::fmt::Debug for EmbeddedLlamaCpp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The llama.cpp inner types don't implement Debug; skip the
        // `inner` field so this impl is feature-flag-agnostic.
        f.debug_struct("EmbeddedLlamaCpp")
            .field("id", &self.id)
            .field("model_path", &self.model_path)
            .finish_non_exhaustive()
    }
}

impl EmbeddedLlamaCpp {
    /// Construct from a *hash-verified* GGUF path. The caller
    /// (CLI / harness) MUST have run the SI-7 hash check
    /// (`ModelIntegrity::verify_gguf` or `verify_sha256`) before
    /// invoking this. This module does not re-verify; the
    /// contract is documented at the type level.
    pub fn load_verified(path: impl Into<PathBuf>) -> Result<Self, ModelError> {
        let path = path.into();
        if !path.exists() {
            return Err(ModelError::InvalidConfig {
                reason: format!("embedded GGUF path does not exist: {}", path.display()),
            });
        }
        let id = format!(
            "embedded:{}",
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown.gguf")
        );
        Ok(Self {
            id,
            model_path: path,
            #[cfg(feature = "feat-model-llamacpp")]
            inner: tokio::sync::Mutex::new(None),
        })
    }

    pub fn model_path(&self) -> &Path {
        &self.model_path
    }

    /// Ensure the underlying llama.cpp model is loaded. Idempotent.
    #[cfg(feature = "feat-model-llamacpp")]
    async fn ensure_loaded(&self) -> Result<(), ModelError> {
        use llama_cpp_2::{
            llama_backend::LlamaBackend,
            model::{params::LlamaModelParams, LlamaModel},
        };
        let mut guard = self.inner.lock().await;
        if guard.is_some() {
            return Ok(());
        }
        let backend = LlamaBackend::init().map_err(|e| ModelError::InvalidConfig {
            reason: format!("llama backend init: {e}"),
        })?;
        let params = LlamaModelParams::default();
        let model =
            LlamaModel::load_from_file(&backend, &self.model_path, &params).map_err(|e| {
                ModelError::InvalidConfig {
                    reason: format!("llama model load: {e}"),
                }
            })?;
        *guard = Some(LlamaInner { backend, model });
        Ok(())
    }
}

#[async_trait]
impl ModelBackend for EmbeddedLlamaCpp {
    fn id(&self) -> &str {
        &self.id
    }

    #[cfg(feature = "feat-model-llamacpp")]
    async fn infer(&self, prompt: &str) -> Result<String, ModelError> {
        use llama_cpp_2::{
            context::params::LlamaContextParams, llama_batch::LlamaBatch, model::AddBos,
            token::data_array::LlamaTokenDataArray,
        };
        self.ensure_loaded().await?;
        let guard = self.inner.lock().await;
        let inner = guard.as_ref().expect("loaded above");

        // Tokenize prompt.
        let tokens = inner
            .model
            .str_to_token(prompt, AddBos::Always)
            .map_err(|e| ModelError::Inference {
                reason: format!("tokenize: {e}"),
            })?;

        let ctx_params = LlamaContextParams::default().with_n_ctx(std::num::NonZeroU32::new(2048));
        let mut ctx = inner
            .model
            .new_context(&inner.backend, ctx_params)
            .map_err(|e| ModelError::Inference {
                reason: format!("context init: {e}"),
            })?;

        let mut batch = LlamaBatch::new(2048, 1);
        let last = tokens.len() - 1;
        for (i, t) in tokens.iter().enumerate() {
            batch
                .add(*t, i as i32, &[0], i == last)
                .map_err(|e| ModelError::Inference {
                    reason: format!("batch add: {e}"),
                })?;
        }
        ctx.decode(&mut batch).map_err(|e| ModelError::Inference {
            reason: format!("prompt decode: {e}"),
        })?;

        // Greedy decode up to 128 tokens; this is a smoke-test
        // inference path, not a production serving loop. The
        // agent loop's real streaming surface lands in S-12c.
        let mut output = String::new();
        let mut n_cur = tokens.len() as i32;
        for _ in 0..128 {
            let candidates = LlamaTokenDataArray::from_iter(ctx.candidates(), false);
            let next = ctx.sample_token_greedy(candidates);
            if inner.model.is_eog_token(next) {
                break;
            }
            let piece = inner
                .model
                .token_to_str(next, llama_cpp_2::model::Special::Tokenize)
                .map_err(|e| ModelError::Inference {
                    reason: format!("token_to_str: {e}"),
                })?;
            output.push_str(&piece);

            batch.clear();
            batch
                .add(next, n_cur, &[0], true)
                .map_err(|e| ModelError::Inference {
                    reason: format!("batch add: {e}"),
                })?;
            n_cur += 1;
            ctx.decode(&mut batch).map_err(|e| ModelError::Inference {
                reason: format!("decode: {e}"),
            })?;
        }
        Ok(output)
    }

    #[cfg(not(feature = "feat-model-llamacpp"))]
    async fn infer(&self, _prompt: &str) -> Result<String, ModelError> {
        Err(ModelError::InvalidConfig {
            reason: "embedded inference requires --features feat-model-llamacpp at build time; \
                 SI-7 hash verification still works without it"
                .into(),
        })
    }

    async fn infer_stream<'a>(&'a self, prompt: &'a str) -> Result<TokenStream<'a>, ModelError> {
        // v1: synthesize a stream from the one-shot result. The
        // real streaming integration (yielding tokens as
        // llama.cpp produces them) lands in S-12c.
        let full = self.infer(prompt).await?;
        Ok(Box::pin(stream::once(async move { Ok(full) })) as BoxStream<'_, _>)
    }
}

/// Convenience: wrap as `Arc<dyn ModelBackend>` for the agent
/// loop.
pub fn into_arc_dyn(backend: EmbeddedLlamaCpp) -> Arc<dyn ModelBackend> {
    Arc::new(backend)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn load_verified_rejects_missing_path() {
        let err = EmbeddedLlamaCpp::load_verified("/does/not/exist.gguf").unwrap_err();
        match err {
            ModelError::InvalidConfig { reason } => {
                assert!(reason.contains("does not exist"))
            }
            other => panic!("expected InvalidConfig, got {other:?}"),
        }
    }

    #[test]
    fn load_verified_records_id_from_filename() {
        let f = NamedTempFile::new().unwrap();
        let b = EmbeddedLlamaCpp::load_verified(f.path()).unwrap();
        assert!(b.id().starts_with("embedded:"));
        assert_eq!(b.model_path(), f.path());
    }

    #[cfg(not(feature = "feat-model-llamacpp"))]
    #[tokio::test]
    async fn infer_without_feature_returns_helpful_error() {
        // Operators who build without the feature must get a
        // diagnostic that tells them how to enable it, not a
        // generic NotImplemented.
        let f = NamedTempFile::new().unwrap();
        let b = EmbeddedLlamaCpp::load_verified(f.path()).unwrap();
        let err = b.infer("hello").await.unwrap_err();
        match err {
            ModelError::InvalidConfig { reason } => {
                assert!(reason.contains("feat-model-llamacpp"));
                assert!(reason.contains("SI-7 hash verification still works"));
            }
            other => panic!("expected InvalidConfig, got {other:?}"),
        }
    }
}
