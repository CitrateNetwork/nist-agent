---
created: 2026-05-21T00:00:00Z
branch: feat/s-10b-hitl-ui
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: accepted
adr: 009
sprint: S-10b
---

# ADR-009: HITL UI + Capsule Inspector stay in nist-agent permanently

| Field | Value |
|---|---|
| **ADR Number** | ADR-009 |
| **Date** | 2026-05-21 |
| **Status** | ACCEPTED |
| **Author** | Saul Loveman + Claude Opus 4.7 (1M context) |
| **Sprint** | S-10b |

## Context

ADR-008 closed the question of whether every nist-agent crate
migrates upstream by establishing that **product UI crates
stay**. The wizard (`nist-agent-wizard`) was the first such crate.

S-10b adds two more operator-facing surfaces — the HITL approval
queue UI and the Capsule Inspector pane — and consolidates them
in one crate (`nist-agent-hitl`). This ADR documents the
placement decision (consistent with ADR-008 + the
product/engine dividing line).

## Decision

**`nist-agent-hitl` lives canonically and permanently in
`nist-agent`. There is no upstream PR plan.** Same shape as
ADR-008's decision for `nist-agent-wizard`.

### Why combine queue + inspector in one crate

These two surfaces share the same lifecycle: a capsule is
proposed (queue) → its manifest is inspected (inspector) →
the operator either signs the install (queue resumes) or
rejects. Keeping the render models in one crate lets them share
internal helpers (severity-bucket mapping, the
`InspectorField` scroll-tracking primitive) without inter-crate
plumbing. Future split — if S-10c's marketplace browser pulls
in a third surface — can carve a `nist-agent-ui-shared` crate
at that point; not now.

### What S-10b ships vs defers

S-10b lands the **render models** and a **Slint scaffold** for
each pane. Same shape as S-10a:

- `ApprovalQueueView` from a slice of upstream `PendingView` +
  parallel signature state.
- `ApprovalRow` typed values the UI iterates over.
- `ApprovalAction` typed sum the UI emits back (Sign / Reject)
  that the integration layer routes to upstream's
  `ApprovalQueue::approve()` / `reject()`.
- `CapsuleInspectorView` with the 12 RFC §8.3 mandatory fields
  + a scroll-tracking gate (`mark_viewed` + `check_install_gate`).
- `SigningTierBadge`, `OverlayCertification`, `CapabilityRow`
  for typed display elements.
- `.slint` UI declarations for both panes; conversions in
  `src/ui.rs` produce the Slint-side visual structs from the
  render models.

Wiring the harness-side integration (tokio handles for the
ApprovalQueue's approve/reject, capsule-manifest parsing into a
CapsuleInspectorView, the actual binary entry point) is S-10c
+ S-12 territory.

## Consequences

**Positive.**

- The install-gate property (`Install button is disabled until
  the operator scrolls every field into view`) is now testable
  in headless CI. The
  `install_gate_blocks_until_every_field_viewed` test pins the
  AC-6 audit artifact's behavior; future regression is caught
  before it reaches an operator.
- The render models are serializable, so the harness can
  snapshot UI state into audit records (e.g.
  "operator viewed inspector for capsule X at time T") without
  going through Slint.
- Upstream's `PendingView` shape is consumed but not extended;
  Rule 9 holds.

**Negative.**

- The `ApprovalQueueView::from_pending` constructor takes a
  parallel `signatures` slice rather than reading from the
  upstream `ApprovalQueue` directly. Upstream's queue stores
  signature state on the private `PendingEntry`, not the
  public `PendingView`. The harness's integration layer needs
  to source signatures from the queue's internal API.
  Tracked as S-10c follow-up: extend upstream's `PendingView`
  with `current_signatures` and `roles_still_required` fields.
- The `.slint` UI files (this crate + wizard) duplicate some
  shared elements (button styling, error pane). A future
  `nist-agent-ui-shared` crate could carve these out;
  premature today.

**Neutral.**

- `nist-agent-hitl` depends on `citrate-agent-core` directly
  (for `PendingView`) and on `nist-agent-prelude` (for the
  Overlay enum). When the upstream PRs for other crates land
  and the federation rev bumps, this crate compiles against
  the new types unchanged.

## Alternatives considered

1. **Two separate crates: `nist-agent-approval-queue` +
   `nist-agent-inspector`.** Rejected — the surfaces share
   lifecycle and internal primitives; one crate is cleaner
   for v0.x. Split when a third surface forces the
   refactor.
2. **Put the HITL UI in `citrate-agent-core::hitl::ui`.**
   Rejected per ADR-008 — UI is product-shaped.
3. **Skip the headless render models; write the UI directly in
   Slint.** Rejected — that ties testability to having a GUI
   runtime, which kills CI coverage. The split-render-model
   pattern from ADR-008 ports cleanly here.

## Reversal conditions

Same as ADR-008's reversal conditions: if a second
nist-agent-shaped product needs the same HITL surfaces, and the
federation decides on a shared crate, the migration target
becomes a `citrate-onboarding-kit` or similar. Until then, the
crate stays here.

## References

- RFC-CIT-AGENT-0001 §5 (HITL approval model), §8.1 (operator
  surfaces), §8.3 (Capsule Inspector).
- ADR-008 — the wizard's product-UI stays-here decision; this
  ADR is its sibling.
- `crates/nist-agent-hitl/` — the crate this ADR governs.
- `features/surfaces/surface-slint-capsule-inspector.feature`
  — the install-gate scenario the test pins.
- Federation rule 9 (one source of truth — product-shaped
  crates are the product's source of truth).
