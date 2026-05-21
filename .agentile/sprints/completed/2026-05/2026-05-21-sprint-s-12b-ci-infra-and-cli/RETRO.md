---
created: 2026-05-21T00:00:00Z
branch: feat/s-12b-ci-infra-and-cli
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: complete
sprint: S-12b
---

# Retro — Sprint S-12b (CI infra + operator CLI)

## What landed

The bridge between "v1.0-rc Rust workspace exists" and "an
operator can verify it on prem." 14th workspace crate
`nist-agent-cli` (bin: `citrate-agent`); embedded llama.cpp
backend in `nist-agent-model` behind `feat-model-llamacpp`;
`docs/audit/AIRGAP_TEST.md` step-by-step; GitHub Actions
release pipeline on tag push; hermetic builder Dockerfile;
`docs/audit/HSM_SEAM.md` swap-in guide.

The defining design choice was the **soft-key signing seam**:
the workflow signs with a GHA-secret Ed25519 key today; the
HSM swap-in is a 30-line wrapper around the same `Signer`
trait. The verifier never knows or cares — that's ADR-011's
audit-boundary property paying off operationally.

## Metrics delta

| Metric | Before (S-13-kickoff) | After (S-12b) | Delta |
|---|---|---|---|
| `cargo test --workspace` | 202 | 211 | +9 |
| Workspace crates | 13 | 14 | +1 |
| ADRs | 12 | 12 | 0 |
| TLA+ specs | 5 | 5 | 0 |
| GitHub workflows | n (existing) | n+1 | +1 (release.yml) |
| Audit-packet docs | 10 | 12 | +2 (AIRGAP_TEST, HSM_SEAM) |

## What went well

- **`citrate-agent` bin came together cleanly.** Each
  subcommand is a thin wrapper around the relevant library
  crate; the bin owns dispatch, not logic. The library crates
  did the load-bearing work in S-3 through S-12; S-12b is
  the operator-facing veneer.
- **Pinned-wording refusals carry across the CLI.** The
  `dist-bundled-gemma4.feature`'s `"SI-7: model hash
  mismatch"` and `dist-signed-releases.feature`'s
  `"release signature invalid"` come through the CLI exit
  paths verbatim. The test I added to `commands/install.rs`
  asserts the Display impl as a final guard against drift.
- **Soft-key seam keeps the workflow honest about its
  signing posture.** The release notes literally call out
  "pre-release / not for production" when the
  `RELEASE_SIGNING_KEY_HEX` secret is set. When the HSM
  swap-in lands, that warning flips off. The verifier code
  doesn't change a line.
- **`build_manifest.py` is intentionally Python.** Pure
  stdlib, no dep tree, deterministic output, inspectable on
  the air-gap host without Cargo. Tools that gate releases
  should be simple enough to audit at a glance.
- **The smoke tests I ran during the sprint exercise every
  CLI surface.** `--help`, `status`, `doctor`, `model verify`
  happy + tamper, `build_manifest.py`, `sign-manifest`. If
  any of them break a future sprint, regression is loud.

## What was tricky

- **clap function shadowing.** Importing
  `nist_agent_doctor::run` and defining `pub async fn run`
  in the same module shadows. Caught at compile; fixed by
  renaming the import to `run_checks`. Worth remembering:
  any CLI subcommand module that exposes a `run` function
  should not also `use` a same-named function from a library
  crate.
- **`Result<T, _>::unwrap_err` requires `T: Debug`.** Caught
  in `nist-agent-model::EmbeddedLlamaCpp` because the
  `LlamaInner` field's `tokio::sync::Mutex<Option<LlamaInner>>`
  doesn't have `Debug` when the feature is on. Solved with
  a manual Debug impl that `finish_non_exhaustive()`s the
  inner field. Same trap S-12 hit on `ReleaseVerifier`.
- **`scripts/release/` had to be standalone.** Initially
  tried to be a workspace member; that pulled in the entire
  workspace's transitive deps for what's a 100-line signer.
  Moved to its own `[workspace]` declaration; ~5x faster
  to build standalone.
- **`tempfile` was a dev-dependency on `nist-agent-model`
  but not on `nist-agent-cli`** — the inline `mod tempfile`
  I'd added at the bottom of `commands/model.rs` was a
  cycle-import accident. Removed and added the dev-dep
  explicitly.
- **llama-cpp-2 isn't actually pulled in by default.** The
  feature gate works because `cargo build --workspace`
  doesn't enable per-crate features by default unless they're
  in the `default` set. Operator opts in at build time;
  default workspace builds stay light. Verified by checking
  `Cargo.lock` after the change — no `llama-cpp-2` entries
  there. (The workflow does NOT build with the feature in
  the release pipeline; it's an opt-in for the
  embedded-inference deployment shape.)

## What to carry into S-12c / v1.0-tag

- **Daemon event loop.** Today's `daemon` subcommand is a
  no-op skeleton. S-12c wires:
  - HITL queue tokio handles (the render models from S-10b
    are already wired; just need the runtime).
  - Audit anchor cadence (PolicyBundle's `anchor_strategy`
    drives nightly Merkle root / per-install events).
  - IPC socket for `citrate-agent status` to read live state.
- **Slint wizard window-up.** The render models (S-10a) and
  Slint `.slint` files exist. S-12c constructs the parent
  Window and binds the callbacks.
- **Two-machine bit-for-bit reproducibility test.** Add a
  matrix to the release workflow: build on two distinct GHA
  runners, assert sha256 equality on `target/release/citrate-agent`
  and `release.manifest.toml`.
- **Workflow validation locally.** I authored the workflow
  but couldn't validate it end-to-end without an actual tag
  push. First tag should be `v0.1.0-rc.0` to exercise the
  pipeline against the soft-key path before any real release.
- **HSM provisioning.** ADR-011's deferred work. The seam doc
  is ready; the ceremony itself is operator-side.

## Open follow-ups

- **`cargo deny` config cleanup** (carried from S-13-kickoff).
  Three stale license allowances + 44 duplicate-version
  warnings. Cosmetic; touch once before v1.0 tag.
- **`citrate-agent daemon` needs `--config` flag** when the
  real event loop lands. Today it just smoke-boots.
- **Workflow's SBOM step uses cargo-cyclonedx 0.5.7 pinned.**
  Bump-cadence policy not yet decided; lock the version for
  v1.0-rc and revisit at v1.1.
- **The CLI's `install` command computes per-artifact SHA-
  256s in-memory.** Fine for sub-GB bundles; large bundles
  (with bundled GGUF, ~2 GB total) should stream. Mark for
  S-12c.
- **GHA workflow does not yet exercise `cargo audit` as a
  blocking gate.** Today it's `continue-on-error` because
  the wasmtime transitive advisories are upstream-side. Once
  the federation rev bumps to a patched runtime, flip to
  blocking.
