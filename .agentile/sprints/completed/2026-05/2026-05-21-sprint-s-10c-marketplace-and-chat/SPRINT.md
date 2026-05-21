---
created: 2026-05-21T00:00:00Z
branch: feat/s-10c-marketplace-and-chat
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: complete
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

## Close note — 2026-05-21

Status: **COMPLETE**.

**Delivered.**
- `crates/nist-agent-marketplace` (11th workspace crate).
- `browser.rs`: `MarketplaceView` with three tabs (Bundled / Site
  Mirror / On-Chain), per-overlay pre-filter at construction, AND-
  semantics `required_overlays` filter, case-insensitive search,
  `validate_install()` belt-and-braces gate, `switch_tab()` that
  refuses on-chain when the egress posture forbids it.
- `chat.rs`: `ChatSessionView` with three-state lifecycle (Idle /
  Streaming / PausedAtAction), input-enabled gating, resume-id
  validation, FedRAMP-High-aware trajectory-export gate per RFC
  §12 Q3.
- `ui/marketplace.slint`: `MarketplacePane` (TabWidget +
  per-listing Install button) and `ChatPane` (turns list +
  streaming/paused inline state + submit/cancel/resume/export
  buttons), `feat-ui`-gated.
- `src/ui.rs`: `to_visual()` conversions for `CapsuleListing`,
  `ChatTurn`, and `ChatStreamState`.

**Metrics.**
- `cargo test --workspace` 121 → 139 (+18).
- Workspace crates 10 → 11.
- ADRs unchanged (the no-upstream-migration decision was already
  recorded in ADR-009 covering the entire UI surface).
- Slint compiles clean under `--features feat-ui` (two
  Window-inheritance warnings carry over from S-10b; pane-style
  intentional — harness wraps panes in a Window).

**Exit criteria — final state.**
- ✅ Three-tab marketplace with per-source listings.
- ✅ Search + overlay-certification filter.
- ✅ Install routes to Capsule Inspector via `validate_install()` +
  the S-10b `CapsuleInspectorView`.
- ✅ Chat surface lifecycle covers Idle / Streaming / Paused-at-
  action / Resume.
- ✅ Trajectory-export hint conditioned on overlay set.

Wizard callback wiring + tokio integration of the chat surface
into a running `Agent::step()` stream are deferred to S-12
(distribution + runbooks) — the render model is the load-bearing
testable layer; the streaming glue is a thin tokio adapter.
