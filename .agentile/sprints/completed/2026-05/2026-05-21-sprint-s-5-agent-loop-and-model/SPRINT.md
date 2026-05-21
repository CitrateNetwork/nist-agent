---
created: 2026-05-21T00:00:00Z
branch: feat/s-5-agent-loop-and-model
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: completed
sprint: S-5
closed: 2026-05-21T00:00:00Z
---

# Sprint S-5: Agent loop + Model resolver

## Sprint Metadata

| Field | Value |
|---|---|
| **Sprint ID** | `S-5` |
| **Sprint Name** | Agent loop + Model resolver (local landing; upstream PR pending) |
| **Goal** | Fill the empty `citrate-agent-core::agent` and `::model` slots — single-loop token-streaming with state-managed interrupt, plus Ollama / llama.cpp / embedded model resolver with NIST SI-7 hash verification — locally in `nist-agent-loop` and `nist-agent-model`. Open upstream PR to lift into `citrate-agent-core`. |
| **Branch** | `feat/s-5-agent-loop-and-model` |
| **Start Date** | 2026-05-21 |
| **End Date** | 2026-05-21 |
| **Status** | `COMPLETE` |
| **Planset** | [`../../planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md`](../../planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md) |
| **Predecessors** | S-3 (Cargo workspace), S-4 (chain adapter — for prelude conventions) |

## Why this sprint

Per ALIGNMENT.md exception clause: runtime's CIT-AGENT-3 has the
agent loop + model resolver scaffolded but unsequenced. nist-agent's
v1.0 critical path needs them now — S-6 (PolicyBundle) binds to
the model resolver's resolution config, S-10 (Slint concierge)
binds to the agent loop's surface. ADR-004 records the placement
decision.

## Deliverables

- `crates/nist-agent-model/` — workspace member.
  - `ModelBackend` trait
  - `OllamaClient` with `try_discover()` + `infer()` + `infer_stream()`
  - `Model::resolve()` walking ollama → llamacpp → embedded
  - SHA-256 hash verifier (NIST SI-7)
  - `ModelError` typed errors
- `crates/nist-agent-loop/` — workspace member.
  - `Agent` struct with `step()` + `resume()`
  - `Action` + `ApprovalPayload` wire types with length-prefixed
    proposal-hash determinism
  - `Checkpoint` + `CheckpointStore` trait + `InMemoryCheckpointStore`
  - `parse_action()` for the `<<ACTION capsule.function args_json>>`
    wire format
  - `AgentLoopError` typed errors
- ADR-004: agent loop + model resolver placement.
- Workspace deps: reqwest, futures, sha2, tokio-stream, url added.
- Test count baseline 15 → 41 (+26).

## Test Baseline (start of sprint)

| Metric | Count | Captured |
|---|---|---|
| Tests | 15 | 2026-05-21 |
| Formal specs | 5 | 2026-05-21 |
| CI tripwires | 7 | 2026-05-21 |
| Frontmatter coverage | 100% | 2026-05-21 |
| Drift constraints | 11/11 green | 2026-05-21 |
| Workspace crates | 2 | 2026-05-21 |

## Method

Engineering sprint — bind to runtime's existing public surface
(ModelBackend trait shape mirrors what upstream will accept on PR
back), implement what RFC §3.1 / §5.4 specify, prove the
determinism property by property test.

## Work Packages

### WP-5.1 — `nist-agent-model` crate (DONE)

`ModelBackend` trait, `OllamaClient`, `Model::resolve()`,
SHA-256 hash verifier per NIST SI-7. 10 tests covering hash
verification (empty + hello-world + 0x-prefix + uppercase +
mismatch), Ollama discovery (unreachable, refused, missing
model), resolver fallthrough.

### WP-5.2 — `nist-agent-loop` crate (DONE)

`Agent`, `Action`, `Checkpoint`, `parse_action`,
`InMemoryCheckpointStore`. 16 tests covering proposal-hash
determinism, length-prefix disambiguation, step→pending→resume
round-trips, rejection path, wrong-hash mismatch error, parse
edge cases (plain text, multiple markers).

### WP-5.3 — Workspace deps + Cargo.toml (DONE)

reqwest 0.12 (rustls), futures 0.3, sha2 0.10, tokio-stream 0.1,
url 2.5 added to `[workspace.dependencies]`. Both new crates
consume via `workspace = true`.

### WP-5.4 — ADR-004 (DONE)

Records ALIGNMENT.md exception-clause exercise and the upstream-
PR migration plan. Reversal conditions named.

### WP-5.5 — Upstream PR on `citrate-agent-runtime` (DEFERRED)

A follow-up operator action. The PR replaces the empty mod.rs
files in `agent/core/src/{agent,model}/` with the contents of
`nist-agent-{loop,model}`. After merge, the federation rev bump
on nist-agent re-pins citrate-agent-core to the new rev; a
successor PR on nist-agent deletes the two local crates. Tracked
as S-5b in the RETRO.

### WP-5.6 — llama.cpp server client + embedded GGUF (DEFERRED to S-5b)

The Ollama path is the v1.0 critical-path resolver. llama.cpp
HTTP server and the embedded llama-cpp-2 binding land in S-5b
once the heavyweight native-build deps (clang, cmake) are wired
into CI.

## Daily updates

- 2026-05-21 — Kickoff and close in one session. WP-5.1 → 5.4
  closed; WP-5.5 + 5.6 explicitly deferred. Build green; 41 tests
  pass; clippy + fmt + deny clean locally; baseline.json bumped.

## Exit criteria

- [x] `nist-agent-model` crate compiles + tests pass + clippy clean
- [x] `nist-agent-loop` crate compiles + tests pass + clippy clean
- [x] Determinism property covered: same checkpoint + same payload
      → identical `Completed` outcome (test in `agent.rs`)
- [x] `cargo test --workspace` ratchet up (15 → 41)
- [x] ADR-004 landed
- [x] `cargo deny check` passes (no new advisories beyond the
      triaged register)
- [x] `cargo fmt --check`, `cargo clippy -D warnings` green
- [x] Frontmatter coverage 100%
- [ ] Upstream PR on `citrate-agent-runtime` (DEFERRED — operator
      action, tracked as S-5b)
- [ ] llama.cpp server + embedded GGUF (DEFERRED — S-5b)

## Close note (2026-05-21)

S-5 closed in a single session with the model resolver and agent
loop landing as `nist-agent-model` and `nist-agent-loop`
respectively. The upstream-PR step (WP-5.5) is a follow-up
operator action — the code is ready to lift wholesale into
`citrate-agent-core::{model,agent}` once we branch on the runtime
side.

**Sprint outcome.** 41 tests passing (up from 15) including the
RFC §5.4 determinism property test (`determinism_property_same_input_same_hash`).
SHA-256 hash verifier for NIST SI-7 lands with 4 round-trip tests
against known canonical hashes. The Ollama HTTP client correctly
handles three failure modes (unreachable, refused, missing-model)
as `Ok(None)` so the resolver can fall through.

**Pending follow-ups.**

1. **WP-5.5 upstream PR.** Branch on runtime, move
   `nist-agent-{loop,model}/src/*` into `agent/core/src/{agent,model}/`,
   update runtime's `Cargo.toml` workspace.dependencies, open PR
   referencing ADR-004.
2. **WP-5.6 llama.cpp + embedded.** S-5b sprint. Adds the
   `LlamaCppServerClient` (HTTP shape similar to Ollama) and the
   feature-gated `EmbeddedModel` via `llama-cpp-2`. Requires
   cmake/clang in CI for the embedded build.
3. **Filesystem CheckpointStore.** The in-memory store ships for
   tests; production deployments need disk persistence. Sized as a
   half-sprint task; tracked in the S-5 RETRO.
4. **Streaming wire format.** `infer_stream` currently collects
   the full body before splitting NDJSON. True byte-level streaming
   matters for Slint chat surfaces with visible cadence; revisit
   in S-10 if needed.

**Next sprint.** S-6 (PolicyBundle + data-class lattice) is now
unblocked. S-6 will define how operators select between the
ModelConfig sources (Ollama / llamacpp / embedded) via the signed
PolicyBundle.
