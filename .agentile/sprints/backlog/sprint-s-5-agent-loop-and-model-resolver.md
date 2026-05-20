---
created: 2026-05-20T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: backlog
sprint: S-5
---

# Sprint S-5: Agent loop + Model resolver

**Goal.** Fill the empty `citrate-agent-core::agent` and `::model`
modules: token-by-token streaming loop with state-managed
interrupt, and a model resolver covering local Ollama (11434),
llama.cpp (8080/8000), and embedded GGUF via llama-cpp-4. PR back to
`citrate-agent-runtime` upstream.

**Why now.** Without the loop, nothing executes capsules. Without
the resolver, the bundled Gemma 4 E2B has no path.

**Predecessors.** S-3.

**Features owned.**
- `features/core/agent-loop.feature`
- `features/core/model-resolver.feature`

**Cross-repo.** Code lands upstream. We carry locally in a feature
crate only if runtime's review takes longer than this sprint.

**Exit criteria.**
- `agent::Agent::step()` ratchets the token stream and pauses at HITL
  gates per the state-managed interrupt pattern.
- Determinism property: same checkpoint + same approval → same first
  action. Verified by property test.
- Model resolver covers all four sources in the documented preference
  order; `Model::resolve()` returns the chosen variant.
- Upstream PR opened on `citrate-agent-runtime`.
- `cargo test --workspace` ratchet up.
