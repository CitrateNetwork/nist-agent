//! Embedded llama.cpp backend — resolver step 3 per RFC §3.3.
//!
//! Gated behind the `feat-model-llamacpp` Cargo feature so the
//! default build stays light (the `llama-cpp-2` dep pulls in a
//! C++ toolchain). Operators on an air-gap host enable the
//! feature, build once with the toolchain warm, and ship.
//!
//! The SI-7 hash check on the GGUF runs *before* this module
//! loads anything. To close the check-to-use race
//! (NIST_AGENT-2026-05-31-002: hash over one read, llama.cpp
//! re-opens the path), the loader never re-opens the operator-
//! supplied path: [`EmbeddedLlamaCpp::load_verified_bytes`]
//! stages the exact verified bytes into a private 0700 tempdir
//! and llama.cpp opens that staged copy. A swap of the original
//! path after verification cannot reach inference. The verifier
//! itself lives in [`crate::hash::verify_sha256`] and in
//! `nist-agent-release::ModelIntegrity`; callers MUST run one of
//! those over the same bytes they pass in.

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
    // Private staging dir holding the verified GGUF copy. Held
    // for the backend's lifetime so the staged file survives the
    // lazy first-inference load; removed on drop. The dir is
    // 0700 + the copy 0400, so no co-tenant swap can land between
    // hash check and llama.cpp's open (SI-7 TOCTOU fix).
    _verified_stage: tempfile::TempDir,
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
    /// Construct from *hash-verified* GGUF bytes. The caller
    /// (CLI / harness) MUST have run the SI-7 hash check
    /// (`ModelIntegrity::verify_gguf` or `verify_sha256`) over
    /// `verified_bytes` — these exact bytes, not a separate read
    /// of the same path — before invoking this. The bytes are
    /// staged into a private 0700 tempdir owned by this backend;
    /// llama.cpp opens the staged copy, so a swap of `source`
    /// after verification cannot reach inference (closes the
    /// NIST_AGENT-2026-05-31-002 check-to-use race).
    pub fn load_verified_bytes(
        verified_bytes: &[u8],
        source: impl Into<PathBuf>,
    ) -> Result<Self, ModelError> {
        let source = source.into();
        let file_name = source
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.gguf")
            .to_string();
        let stage = tempfile::Builder::new()
            .prefix("nist-agent-si7-")
            .tempdir()
            .map_err(|e| ModelError::InvalidConfig {
                reason: format!("create SI-7 staging dir: {e}"),
            })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(stage.path(), std::fs::Permissions::from_mode(0o700))
                .map_err(|e| ModelError::InvalidConfig {
                    reason: format!("chmod SI-7 staging dir: {e}"),
                })?;
        }
        let staged_path = stage.path().join(&file_name);
        std::fs::write(&staged_path, verified_bytes).map_err(|e| ModelError::InvalidConfig {
            reason: format!("stage verified GGUF {}: {e}", staged_path.display()),
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&staged_path, std::fs::Permissions::from_mode(0o400))
                .map_err(|e| ModelError::InvalidConfig {
                    reason: format!("chmod staged GGUF: {e}"),
                })?;
        }
        Ok(Self {
            id: format!("embedded:{file_name}"),
            model_path: staged_path,
            _verified_stage: stage,
            #[cfg(feature = "feat-model-llamacpp")]
            inner: tokio::sync::Mutex::new(None),
        })
    }

    /// Construct from a *hash-verified* GGUF path. Reads the file
    /// exactly once and delegates to [`Self::load_verified_bytes`]
    /// — the loader never re-opens `path`, so post-read swaps of
    /// the operator path cannot reach inference. Callers that
    /// already hold the verified bytes (the SI-7 gate flow) MUST
    /// use `load_verified_bytes` directly so the hashed bytes and
    /// the loaded bytes are the same buffer.
    pub fn load_verified(path: impl Into<PathBuf>) -> Result<Self, ModelError> {
        let path = path.into();
        let bytes = std::fs::read(&path).map_err(|e| ModelError::InvalidConfig {
            reason: format!(
                "embedded GGUF path does not exist or is unreadable: {}: {e}",
                path.display()
            ),
        })?;
        Self::load_verified_bytes(&bytes, &path)
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
        // model_path is the private staged copy, never the
        // operator-supplied path (SI-7 TOCTOU fix); same file
        // name, different (0700-staged) directory.
        assert_ne!(b.model_path(), f.path());
        assert_eq!(b.model_path().file_name(), f.path().file_name());
    }

    #[test]
    fn load_verified_bytes_binds_hashed_bytes_to_loaded_bytes() {
        // The bytes the SI-7 gate hashed are byte-for-byte the
        // bytes at the loader-visible path.
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("model.gguf");
        std::fs::write(&p, b"ORIGINAL").unwrap();
        let verified = std::fs::read(&p).unwrap();
        let backend = EmbeddedLlamaCpp::load_verified_bytes(&verified, &p).unwrap();
        assert_eq!(std::fs::read(backend.model_path()).unwrap(), verified);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let dir_mode = std::fs::metadata(backend.model_path().parent().unwrap())
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(dir_mode & 0o077, 0, "staging dir must be private (0700)");
        }
    }

    #[test]
    fn si7_swap_after_verify_cannot_change_loaded_bytes() {
        // RED for NIST_AGENT-2026-05-31-002 (HIGH, SI-7 TOCTOU):
        // the gate hashed one read of the GGUF, then handed the
        // PATH to a loader that re-opened it — so a swap in the
        // check-to-use window reached llama.cpp with SI-7
        // "passing". Pin: whatever path the loader will open must
        // carry exactly the verified bytes, even after the
        // operator-supplied path is swapped post-verification.
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("model.gguf");
        std::fs::write(&p, b"GOOD MODEL BYTES").unwrap();

        // The SI-7 flow: read once, hash those bytes (elided —
        // any caller-side check), then construct the backend.
        let verified = std::fs::read(&p).unwrap();
        let backend = EmbeddedLlamaCpp::load_verified(&p).unwrap();

        // Attacker swaps the file inside the race window.
        std::fs::write(&p, b"EVIL MODEL BYTES").unwrap();

        let loaded = std::fs::read(backend.model_path()).unwrap();
        assert_eq!(
            loaded, verified,
            "loader-visible bytes must be the verified bytes, not the swapped file"
        );
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
