---
created: 2026-05-21T00:00:00Z
branch: feat/s-5-agent-loop-and-model
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: completed
sprint: S-5
---

# Sprint S-5 — Retrospective

## Outcome

| Field | Value |
|---|---|
| **Goal achieved?** | YES (in-scope: local landing); WP-5.5 (upstream PR) and WP-5.6 (llama.cpp + embedded) explicitly deferred to S-5b |
| **WPs planned / closed** | 6 / 4 in scope; 2 deferred |
| **Carry-forward WPs** | WP-5.5 (upstream PR on citrate-agent-runtime), WP-5.6 (llama.cpp server + embedded GGUF) |
| **Closing branch** | `feat/s-5-agent-loop-and-model` (PR pending) |

## Metrics delta

| Axis | Start | End | Δ |
|---|---|---|---|
| Tests | 15 | 41 | **+26** |
| Formal specs | 5 | 5 | 0 (S-2 territory) |
| CI tripwires | 7 | 7 | 0 |
| Frontmatter coverage | 100% | 100% | maintained |
| Federation drift constraints | 11/11 green | 11/11 green | 0 (no new pins) |
| Workspace crates | 2 | 4 | +2 (`nist-agent-model`, `nist-agent-loop`) |

The 41-tests baseline is the new floor — three months of future
S-5b/S-6/S-7 work can't decrease it.

## What worked

- **The ALIGNMENT.md exception-clause cite carried weight.** When
  the question "wait on runtime or land here" came up, the
  documented exception clause (added in S-1's ALIGNMENT.md) gave
  the call without re-litigating. ADR-004 cites the clause and
  records the worked example. Future sprints in the same shape
  now have a template.
- **Mirroring upstream's eventual module shape** (`agent` + `model`
  as siblings, not bundled) means the upstream PR is a directory
  rename, not a refactor. Less review friction when the PR opens.
- **Length-prefix framing in `Action::compute_hash`.** The
  `(capsule_len_u64 || capsule || function_len_u64 || function ||
  args_len_u64 || args)` shape makes the disambiguation property
  testable in a one-line property test. Caught the would-be
  collision in `proposal_hash_disambiguates_by_length_prefix`.
- **Returning `Ok(None)` on transport-refused** for Ollama
  discovery instead of bubbling the error. Lets the resolver fall
  through cleanly to the next source; matches how the RFC §3.3
  resolution order is described.

## What didn't work

- **First streaming impl was over-clever.** I tried to write a
  byte-stream NDJSON parser inline using `scan` + `flat_map` +
  `bytes::Bytes` and got tangled in lifetime + type-inference
  errors (4 compile errors). The clean version (collect body,
  split lines, parse each) is more code but obviously correct.
  Lesson: don't reach for `futures::stream` combinators inside
  closures unless you've drawn the type diagram first.
- **Forgot `#[derive(Debug)]` on `ResolvedModel`.** Tests use
  `expect_err()` which requires Debug. Cargo's error message was
  clear; cost ~30 seconds. Pin for future: every public enum that
  appears in a `Result` ought to derive Debug.
- **The `serde_json`-as-dev-dep dance recurred.** Same gotcha as
  S-3: a test using `serde_json` needs the dev-dep declared
  explicitly. I avoided it this time by not using it in test code,
  but I should add a Semgrep rule that flags `use serde_json` in
  `#[cfg(test)]` blocks of crates that don't list it. Tracked.

## What surprised us

- **`citrate_agent_core::audit::AnchorKind` is re-exportable from
  the prelude** (already proven in S-4), and the same shape works
  for the model crate's eventual `AnchorKind` reuse. The
  upstream's `pub use audit::...` discipline is the cheap
  cross-crate sharing pattern; we should adopt it everywhere
  nist-agent grows a new public enum.
- **Ollama's `/api/tags` schema is permissive.** Real-world tag
  entries have many fields beyond `name` (size, digest, modified
  timestamp). Using `#[serde(default)]` on `TagsResponse.models`
  and a narrow `TagEntry { name: String }` ignored the rest —
  good. Future fragility: if Ollama renames `name` we silently
  see "no models available". Worth a serde-strict toggle in
  doctor's pre-flight check.

## Lessons for future sprints

1. **For "fill an empty upstream mod.rs" sprints, the exception-
   clause path is the right answer.** ADR-004 is the template.
2. **Tests pay for themselves when they verify *properties*, not
   just *outcomes*.** The determinism property test
   (`determinism_property_same_input_same_hash`) is one assertion;
   it guards a load-bearing RFC §5.4 invariant. Adding it cost ~5
   lines; removing it would let any future drift in the action
   hashing or checkpoint id derivation slip past CI.
3. **Always run `cargo test --workspace 2>&1 | grep "test result"`
   to see the per-binary count.** A single test_result line that
   ends in "0 passed" can be a doc-test summary; a glance reads
   it as a problem when it's not.

## Pending follow-ups (S-5b candidates)

1. **Upstream PR on `citrate-agent-runtime`.** Lift
   `nist-agent-{loop,model}` into `agent/core/src/{agent,model}/`.
   Single commit; tests transfer verbatim. ADR-004 has the plan.
2. **`LlamaCppServerClient`.** Same shape as `OllamaClient` but
   targets llama.cpp's HTTP API (slightly different endpoints,
   no `/api/tags` equivalent — discovery is a `GET /health` + a
   `POST /completion` ping).
3. **Embedded GGUF via `llama-cpp-2`.** Heavyweight native-build
   dep (cmake, clang). Feature-gated behind `feat-embedded`.
   Adds ~5 minutes to cold CI build but enables the Slint
   concierge's air-gap path (S-10).
4. **Filesystem `CheckpointStore`.** Persists JSON files under an
   operator-configured directory. Half-sprint of work.
5. **Doctor pre-flight check #7 (model hash verification)** —
   bind `verify_sha256` from `nist-agent-model::hash` to the
   doctor's runtime check. Goes into S-7.
6. **Semgrep tripwire: `use serde_json` in #[cfg(test)] blocks
   without `serde_json` in `[dev-dependencies]`.** Prevents the
   recurring fail-then-fix cycle from S-3 and now S-5.

## Next sprint(s)

- **S-6 (PolicyBundle + data-class lattice)** unblocked. The
  bundle deserializes into the `ModelConfig` shape we landed here,
  so the S-6 work has a real binding target.
- **S-5b** (upstream PR + llama.cpp + embedded + filesystem
  store) can run in parallel to S-6 — they touch different
  crates.
