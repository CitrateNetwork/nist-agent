//! `citrate-agent model` — SI-7 verify + (optional) inference.
//!
//! Two subcommands:
//!
//! - `verify` — runs `ModelIntegrity::verify_gguf()` against a
//!   manifest's `[model].sha256` and the on-disk GGUF. Refuses
//!   with the pinned `"SI-7: model hash mismatch"` wording on
//!   mismatch. Air-gap-safe; no inference performed.
//! - `infer` — runs a one-shot inference. Behind
//!   `--features feat-model-llamacpp` at build time. The SI-7
//!   check runs *before* load so a tampered GGUF cannot reach
//!   the inference path.

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use nist_agent_model::EmbeddedLlamaCpp;
use nist_agent_release::{ModelIntegrity, ReleaseManifest};
use std::fs;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum ModelCmd {
    /// SI-7 hash check: verify a GGUF's sha256 against a
    /// release manifest's `[model].sha256`.
    Verify(VerifyArgs),

    /// Run a single inference against the bundled GGUF.
    /// Requires --features feat-model-llamacpp at build.
    Infer(InferArgs),
}

#[derive(Args, Debug)]
pub struct VerifyArgs {
    /// Path to the release.manifest.toml carrying the expected
    /// `[model].sha256`.
    #[arg(long)]
    pub manifest: PathBuf,
    /// Path to the GGUF file on disk.
    #[arg(long)]
    pub gguf: PathBuf,
}

#[derive(Args, Debug)]
pub struct InferArgs {
    /// Path to the GGUF file on disk. Must already pass SI-7.
    #[arg(long)]
    pub gguf: PathBuf,
    /// Path to the release.manifest.toml for the SI-7 pre-load
    /// hash check.
    #[arg(long)]
    pub manifest: PathBuf,
    /// Prompt text. Use --prompt-file for multi-line input.
    #[arg(long)]
    pub prompt: Option<String>,
    /// Path to a file containing the prompt. Mutually exclusive
    /// with --prompt.
    #[arg(long)]
    pub prompt_file: Option<PathBuf>,
}

pub async fn run(cmd: ModelCmd) -> Result<i32> {
    match cmd {
        ModelCmd::Verify(args) => run_verify(args),
        ModelCmd::Infer(args) => run_infer(args).await,
    }
}

fn run_verify(args: VerifyArgs) -> Result<i32> {
    let toml_str = fs::read_to_string(&args.manifest)
        .with_context(|| format!("read manifest {}", args.manifest.display()))?;
    let manifest = ReleaseManifest::from_toml(&toml_str)
        .with_context(|| format!("parse manifest {}", args.manifest.display()))?;
    let gguf_bytes =
        fs::read(&args.gguf).with_context(|| format!("read GGUF {}", args.gguf.display()))?;

    match ModelIntegrity::verify_gguf(&manifest, &gguf_bytes) {
        Ok(()) => {
            println!("SI-7: model hash OK ({})", args.gguf.display());
            Ok(0)
        }
        Err(e) => {
            // Pinned-wording refusal flows through Display.
            eprintln!("{e}");
            Ok(2)
        }
    }
}

async fn run_infer(args: InferArgs) -> Result<i32> {
    // SI-7 first. The embedded backend's load_verified_bytes()
    // relies on the caller having checked the hash over the SAME
    // bytes it passes in; we do that here so the inference path
    // can never load a tampered GGUF — not even via a path swap
    // in the check-to-use window.
    let toml_str = fs::read_to_string(&args.manifest)
        .with_context(|| format!("read manifest {}", args.manifest.display()))?;
    let manifest = ReleaseManifest::from_toml(&toml_str)
        .with_context(|| format!("parse manifest {}", args.manifest.display()))?;
    let gguf_bytes =
        fs::read(&args.gguf).with_context(|| format!("read GGUF {}", args.gguf.display()))?;
    if let Err(e) = ModelIntegrity::verify_gguf(&manifest, &gguf_bytes) {
        eprintln!("{e}");
        return Ok(2);
    }
    // SI-7 TOCTOU fix (NIST_AGENT-2026-05-31-002): hand the
    // loader the exact bytes the hash check just verified — the
    // backend stages them privately and never re-opens
    // `args.gguf`, so a swap of the operator path between check
    // and load cannot reach inference.
    let backend = EmbeddedLlamaCpp::load_verified_bytes(&gguf_bytes, &args.gguf)
        .with_context(|| format!("load GGUF {}", args.gguf.display()))?;
    drop(gguf_bytes);

    let prompt = resolve_prompt(args.prompt, args.prompt_file)?;

    use nist_agent_model::ModelBackend;
    match backend.infer(&prompt).await {
        Ok(text) => {
            println!("{text}");
            Ok(0)
        }
        Err(e) => {
            eprintln!("inference failed: {e}");
            Ok(2)
        }
    }
}

fn resolve_prompt(prompt: Option<String>, prompt_file: Option<PathBuf>) -> Result<String> {
    match (prompt, prompt_file) {
        (Some(p), None) => Ok(p),
        (None, Some(f)) => {
            Ok(fs::read_to_string(&f)
                .with_context(|| format!("read prompt-file {}", f.display()))?)
        }
        (Some(_), Some(_)) => {
            anyhow::bail!("--prompt and --prompt-file are mutually exclusive")
        }
        (None, None) => Ok(String::from("Hello, world.")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn resolve_prompt_inline() {
        let p = resolve_prompt(Some("hi".into()), None).unwrap();
        assert_eq!(p, "hi");
    }

    #[test]
    fn resolve_prompt_from_file() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "from file").unwrap();
        let p = resolve_prompt(None, Some(f.path().to_path_buf())).unwrap();
        assert_eq!(p.trim(), "from file");
    }

    #[test]
    fn resolve_prompt_default_when_neither_supplied() {
        let p = resolve_prompt(None, None).unwrap();
        assert!(!p.is_empty());
    }

    #[test]
    fn resolve_prompt_refuses_both_inputs() {
        let f = NamedTempFile::new().unwrap();
        let r = resolve_prompt(Some("x".into()), Some(f.path().to_path_buf()));
        assert!(r.is_err());
    }
}
