---
created: 2026-05-21T00:00:00Z
branch: feat/s-5-agent-loop-and-model
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: accepted
adr: 004
sprint: S-5
---

# ADR-004: Agent loop + Model resolver land locally, then PR upstream

| Field | Value |
|---|---|
| **ADR Number** | ADR-004 |
| **Date** | 2026-05-21 |
| **Status** | ACCEPTED |
| **Author** | Saul Loveman + Claude Opus 4.7 (1M context) |
| **Sprint** | S-5 |

## Context

RFC-CIT-AGENT-0001 §3.1 names two of the eight harness subsystems
as load-bearing: the **Agent Loop** (single-loop, token-by-token
streaming, state-managed interrupt for HIC) and the **Model
Resolver** (Ollama / llama.cpp / embedded GGUF, with NIST SI-7
hash verification). Both are normative-core surface — their
canonical home is `citrate_agent_core::agent` and
`citrate_agent_core::model` upstream.

When we surveyed `citrate-agent-runtime` at S-1, both modules
existed as scaffolded `mod.rs` placeholders with only header
comments. The upstream planset (`citrate-agent-runtime/.agentile/`)
named them as CIT-AGENT-3 work but had not yet sequenced the
sprint. `citrate-agent-runtime` cannot ship its v1.0 without these
modules; neither can `nist-agent`.

[`planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md`](../planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md)'s
"Decision protocol when ownership is unclear" exception clause
says:

> 2. **Exception: roadmap-critical and runtime-unsequenced.** If
>    the work is on the nist-agent v1.0 path and runtime has not
>    yet sequenced it in their planset, land in a feature crate
>    locally in nist-agent. Open the upstream PR within the same
>    sprint.

S-5 is the exception case. The question this ADR resolves is:
**which crates do we create, and how do we name them so the
eventual upstream migration is clean?**

## Decision

**We will land the agent loop in `crates/nist-agent-loop` and the
model resolver in `crates/nist-agent-model`.** Both crates depend
on `citrate-agent-core` for shared types (none today; future for
`AnchorKind`-shape items). The eventual upstream migration moves
the contents into `citrate_agent_core::agent` and
`citrate_agent_core::model` respectively; per ALIGNMENT.md's
"Never copy code" rule, the upstream PR is *the* canonical move
and we *delete* our local crates on its merge.

### Naming

`nist-agent-loop` (not `nist-agent-agent`) — two reasons:

1. Cargo discourages `crate-thing-thing` patterns where one of the
   "things" is the same word; `nist-agent-agent` reads badly.
2. RFC §3.1 names the subsystem "Agent Loop"; "loop" is the
   distinguishing word.

`nist-agent-model` for the model resolver — direct mirror of
upstream's `citrate_agent_core::model`.

### Crate boundaries

- **`nist-agent-model`** owns the Model trait, the resolver
  (`Model::resolve()`), the Ollama client, the llama.cpp client
  (stub today, ships in S-5b), the SI-7 hash verifier. No
  dependency on `nist-agent-loop`.
- **`nist-agent-loop`** owns the Agent struct, the
  Action/Checkpoint types, the parse-action wire format, the
  CheckpointStore trait + in-memory impl. Depends on
  `nist-agent-model` for the `ModelBackend` trait.
- Neither crate depends on `nist-agent-chain` or
  `nist-agent-prelude`. The runtime engine doesn't need to know
  about the chain in v0.x; the HIC signing path uses the chain
  only at audit-anchoring time, and that's the harness's job, not
  the agent loop's.

### Upstream-PR plan

S-5 closes locally. The upstream PR lifts the entire surface into
`citrate-agent-core` as two new modules:

- `citrate_agent_core::model` → from `nist-agent-model`
- `citrate_agent_core::agent` → from `nist-agent-loop`

The PR will:

1. Replace the empty `mod.rs` files with the real implementations.
2. Re-export the public types from `citrate_agent_core` per the
   RFC §3.2 frozen surface (`Agent`, `Capsule`, `PolicyBundle`,
   `AuditChain` — `Agent` joins the existing re-exports).
3. Add the workspace deps (reqwest, sha2, futures, async-trait,
   url) to runtime's Cargo.toml.
4. Migrate our test suite verbatim (41 → 41 transitions).

After the PR merges and the federation bumps
`[repos.citrate-agent-runtime].rev`, we:

5. Re-pin `citrate-agent-core` in our `Cargo.toml` to the new rev.
6. Delete `crates/nist-agent-{loop,model}` from the workspace.
7. Update `nist-agent-prelude` to re-export `Agent`, `Action`,
   `Checkpoint` from upstream.

## Consequences

**Positive.**

- nist-agent v1.0's critical path proceeds without waiting on
  runtime's CIT-AGENT-3 scheduling. S-6 (PolicyBundle) and S-10
  (Slint concierge) can build on the loop + resolver landing here.
- The implementation gets tested + clippy-cleaned + cargo-deny'd
  in nist-agent's CI before the upstream review starts. By the
  time the PR opens, the code has cleared three independent
  rule-gates.
- The 41-test baseline (up from 15) gives us a real ratchet floor.
- ALIGNMENT.md's exception clause now has a worked example. Future
  sprints that hit the same situation cite ADR-004 + RETRO-S-5.

**Negative.**

- Until the upstream PR merges, there are two homes for the agent
  loop / model resolver: the empty upstream `mod.rs` files and our
  local crates. A reader landing in runtime expecting code will
  find header comments only; the README in
  `citrate-agent-runtime/agent/core/src/agent/mod.rs` should grow
  a "current implementation: nist-agent-loop" pointer when the
  upstream PR opens. Tracked as a follow-up to this ADR.
- nist-agent's binary size grows by ~3 MB (the reqwest + rustls
  dep tree). This was already paid for by `nist-agent-chain`
  (via `citrate-wallet-core`'s transitive reqwest pin), so the
  marginal cost is small.
- The CheckpointStore today has only an in-memory implementation.
  Production deployments need filesystem-backed persistence. That
  ships in S-5b or as a follow-up commit on this branch — tracked
  in the S-5 RETRO.

**Neutral.**

- The `<<ACTION ... >>` wire format for action parsing is a
  nist-agent invention; the upstream migration may revise it
  (e.g. to a JSON-mode-aware extraction matching what Hermes /
  OpenAI function-calling produce). That's a v2 RFC concern, not
  a v1.0 blocker.

## Alternatives considered

1. **Wait for runtime to schedule and ship CIT-AGENT-3.** Rejected:
   blocks our v1.0 critical path indefinitely. ALIGNMENT.md's
   exception clause exists precisely for this case.
2. **Author directly in `/home/saul/Projects/Citrate-Labs/
   citrate-agent-runtime/` and PR from there.** Rejected: bypasses
   nist-agent's CI gates; the code wouldn't get a deny.toml or
   ratchet review until upstream review starts. The "land locally
   first, PR upstream" sequence is exactly what the exception
   clause specifies.
3. **One crate (`nist-agent-engine`) housing both loop + resolver.**
   Rejected: the upstream split is by module (`agent` vs `model`).
   Mirroring that split locally makes the eventual move a 1:1
   directory rename instead of a refactor.

## Reversal conditions

This ADR is reversed when:

- The upstream PR merges into `citrate-agent-runtime` AND
- The federation manifest bumps the runtime rev AND
- Our `Cargo.toml` re-pins `citrate-agent-core` to that rev AND
- The 41 tests transfer cleanly to the upstream test runner.

A successor ADR records the deletion of `nist-agent-{loop,model}`
and the re-export changes in `nist-agent-prelude`.

## References

- RFC-CIT-AGENT-0001 §3.1 (Agent loop, Model resolver), §3.3
  (Network posture), §5.4 (State-managed interrupt), §8.2
  (Bundled concierge model).
- [`.agentile/planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md`](../planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md) — "Decision protocol when ownership is unclear" exception clause.
- [`ADR-002`](ADR-002-tla-custody-in-nist-agent.md) and
  [`ADR-003`](ADR-003-evm-chain-adapter-trait.md) for the two
  prior ownership-resolution ADRs.
- Federation rules 1 (no stubs in production paths) and 9 (one
  source of truth per topic).
