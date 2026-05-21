---
created: 2026-05-21T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: backlog
sprint: S-10c
---

# Sprint S-10c: Slint capsule marketplace browser + chat surface

**Goal.** Add the two remaining S-10 surfaces: browse-and-search
the capsule marketplace (bundled, local site mirror, on-chain
CapsuleRegistry) and the interactive chat surface that drives
the agent loop with operator prompts.

**Why now.** S-10a + S-10b cover setup + approval. S-10c lets
operators DO things — install new capsules and converse with
the agent. Closes the v1.0 Slint app surface.

**Predecessors.** S-10a (shell), S-10b (Capsule Inspector — the
marketplace's "install" button routes through it), S-4 (chain
adapter — marketplace reads CapsuleRegistry via `dyn
ChainClient`), S-5 (Agent loop — chat surface drives `step()` +
streams tokens).

**Features owned.**
- `features/surfaces/surface-slint-concierge.feature` (chat
  surface scenarios that didn't land in S-10a)
- `features/chain/chain-capsuleregistry.feature` (UI half)
- `features/chain/chain-benchmarkregistry.feature` (UI half)

**Cross-repo.** None.

**Exit criteria.**
- Marketplace pane: list of capsules from operator-configured
  sources. Three tabs: Bundled (ships with the binary), Site
  Mirror (operator-configured path), On-Chain (read from
  CapsuleRegistry; only enabled when egress posture allows).
- Search + filter by overlay certification (so a FERPA-only
  deployment hides capsules without `certified = ["FERPA"]`).
- Install button on each row routes to the Capsule Inspector
  pane (S-10b) for the HITL-gated install flow.
- Chat surface: live text input → `Agent::step()` → streamed
  token display. Pauses at every `AgentOutcome::Pending`, shows
  the action proposal inline with the conversation, awaits
  resume from the HITL queue, continues streaming.
- Trajectory export hint surfaced when the operator's overlay
  allows it (Hermes-style training data; tier-critical capsule
  per RFC §12 Q3).

**Deferred to S-10d / future:**
- Mobile companion deep-link integration (S-11)
- Per-capsule SHA-256 verification UI (the inspector already
  surfaces content_hash; explicit "verify" button is nice-to-have)
- Multi-tab chat sessions (one conversation at a time in v1.0)
