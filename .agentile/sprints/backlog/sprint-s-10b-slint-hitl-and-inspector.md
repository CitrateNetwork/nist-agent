---
created: 2026-05-21T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: backlog
sprint: S-10b
---

# Sprint S-10b: Slint HITL approval queue UI + Capsule Inspector pane

**Goal.** Add the two operator-facing surfaces that govern every
agent action: the HITL approval queue UI (pending proposals,
signature collection, sign/reject buttons) and the Capsule
Inspector pane (the AC-6 audit artifact rendered per
`features/surfaces/surface-slint-capsule-inspector.feature`).

**Why now.** S-10a's concierge can author a PolicyBundle but
operators can't yet review and approve actions. S-10b makes the
harness usable for actual agent work; without it, the daemon
runs but no approver can act.

**Predecessors.** S-10a (Slint shell), S-5 (Agent loop +
Checkpoint type — the queue surfaces its `Action` proposals),
S-4 (chain adapter — Capsule Inspector reads
`read_capsule_registry()` for the Gherkin pass-rate + TLA
status from BenchmarkRegistry).

**Features owned.**
- `features/surfaces/surface-slint-capsule-inspector.feature`
- `features/core/hitl-quorum.feature` (UI half)
- `features/core/hitl-state-managed-interrupt.feature` (UI half)

**Cross-repo.** None.

**Exit criteria.**
- Approval queue pane: list of pending `Action` proposals from
  upstream's `ApprovalQueue::pending()`. Each row shows: capsule
  name + version, function, data-class summary, required roles,
  current signatures. Sign / Reject buttons collect a hardware-
  backed signature via the configured surface (FIDO2 / PIV-CAC).
- Capsule Inspector pane: renders every field per
  `features/surfaces/surface-slint-capsule-inspector.feature` —
  content_hash, publisher DID, signing tier, declared
  capabilities, data classes, risk tier, required roles,
  certified overlays, TLA verification status, Gherkin pass
  rate, expandable procedure.md.
- Install button disabled until every field has been scrolled
  into the viewport (per scenario "Install button is disabled
  until the operator scrolls every field into view").
- No silent install path: CLI / daemon RPC / mobile also route
  install requests through the Inspector pane for
  confirmation.
- Resume-on-approval triggers the agent loop's `resume()` with
  the collected `ApprovalPayload`.

**Deferred to S-10c:**
- Marketplace browser (which capsules to install — Inspector
  consumes a single capsule's metadata regardless of source)
- Live chat surface
