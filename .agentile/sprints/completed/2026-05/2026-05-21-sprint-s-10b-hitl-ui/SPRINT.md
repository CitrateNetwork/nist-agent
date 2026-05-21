---
created: 2026-05-21T00:00:00Z
branch: feat/s-10b-hitl-ui
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: completed
sprint: S-10b
closed: 2026-05-21T00:00:00Z
---

# Sprint S-10b: HITL approval queue UI + Capsule Inspector pane

## Sprint Metadata

| Field | Value |
|---|---|
| **Sprint ID** | `S-10b` |
| **Sprint Name** | HITL approval queue UI + Capsule Inspector pane (render models + Slint scaffold) |
| **Goal** | Land `crates/nist-agent-hitl` per RFC §8.3: render models for the approval queue and Capsule Inspector + Slint UI scaffold + the scroll-tracking install gate. Headless models tested in CI; UI compiles behind `feat-ui`. |
| **Branch** | `feat/s-10b-hitl-ui` |
| **Start Date** | 2026-05-21 |
| **End Date** | 2026-05-21 |
| **Status** | `COMPLETE` |
| **Planset** | [`../../planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md`](../../planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md) |
| **Predecessors** | S-10a (Slint scaffold pattern + ADR-008), S-5 (Agent loop + ToolCall), upstream HITL queue |

## Why this sprint

S-10a closed the concierge wizard. S-10b closes the two
surfaces every agent action passes through after the daemon is
running: the approval queue (sign/reject pending proposals)
and the Capsule Inspector (the AC-6 audit artifact rendered
before any install). Without these, the harness daemon runs
but no approver can act on it.

## Deliverables

- `crates/nist-agent-hitl/` — new workspace member.
  - `ApprovalQueueView` + `ApprovalRow` + `ApprovalRowSeverity`
    + `ApprovalAction` (render model + typed UI action sum).
  - `CapsuleInspectorView` with the 12 RFC §8.3 mandatory
    fields + `InspectorField` scroll-trackable rows +
    `mark_viewed()` + `check_install_gate()`.
  - `CapabilityRow`, `SigningTierBadge`, `OverlayCertification`
    typed display elements.
  - Slint UI scaffold at `ui/hitl.slint` with `ApprovalQueuePane`
    + `CapsuleInspectorPane`; `src/ui.rs` converts render
    models to Slint visual structs.
  - `HitlUiError` typed errors.
- ADR-009: HITL UI stays in nist-agent permanently (sibling of
  ADR-008's wizard decision).
- Test count: 110 → 121 (+11).

## Test Baseline (start of sprint)

| Metric | Count | Captured |
|---|---|---|
| Tests | 110 | 2026-05-21 |
| Formal specs | 5 | 2026-05-21 |
| CI tripwires | 7 | 2026-05-21 |
| Frontmatter coverage | 100% | 2026-05-21 |
| Drift constraints | 11/11 green | 2026-05-21 |
| Workspace crates | 9 | 2026-05-21 |
| ADRs | 8 | 2026-05-21 |

## Method

Same headless-render-model-plus-Slint-scaffold pattern
established by S-10a. `ApprovalQueueView::from_pending`
consumes upstream `PendingView` slices; the harness side
sources signature state from the queue's internal API.
`CapsuleInspectorView::check_install_gate` is the load-bearing
property — pin it with the test that walks through every
field's view-mark.

## Work Packages

### WP-10b.1 — `ApprovalQueueView` + `ApprovalRow` (DONE)

5 tests covering severity mapping (canonical + unmappable +
empty), row construction from `PendingView`, signature-state
zip, action call-id round-trip.

### WP-10b.2 — `CapsuleInspectorView` + install gate (DONE)

`InspectorField` scroll-trackable rows, `mark_viewed`,
`check_install_gate`. 6 tests covering fixture completeness,
gate-blocks-until-every-field-viewed, mark-viewed idempotence
+ unknown-id no-op, kebab-case serialization for SigningTierBadge,
overlay-certification round-trip.

### WP-10b.3 — Slint UI scaffold (DONE)

Two panes in `ui/hitl.slint` — `ApprovalQueuePane` (severity-
colored rows + Sign/Reject buttons) + `CapsuleInspectorPane`
(scrollable field list + install button gated on
`fields-viewed == fields-total`). `src/ui.rs` converts
render models to Slint visual structs (`ApprovalRowVisual`,
`InspectorFieldVisual`).

### WP-10b.4 — ADR-009 (DONE)

Mirrors ADR-008's decision; HITL UI stays permanently.

### WP-10b.5 — Wire wizard's deferred callbacks (DEFERRED to S-10c)

S-10a deferred `org-identity-submitted`, `role-add-clicked`,
etc. The wiring is mechanical (each callback calls a wizard
method); since S-10b is already a focused sprint, push the
wiring into S-10c when the unified app shell consolidates all
Slint panes.

### WP-10b.6 — Live tokio integration with upstream ApprovalQueue (DEFERRED)

S-10b ships the render models + Slint scaffold. The harness
integration that bridges the Slint UI's Sign/Reject callbacks
to `ApprovalQueue::approve()` / `reject()` lives at the
daemon boundary (S-12 distribution).

## Daily updates

- 2026-05-21 — Kickoff and close in one session. WP-10b.1
  through 10b.4 done; 10b.5 + 10b.6 deferred to S-10c / S-12.
  11 new tests bring the workspace to 121. fmt + clippy clean.

## Exit criteria

- [x] `ApprovalQueueView::from_pending` builds rows in upstream
      `PendingView` order
- [x] Severity-bucket mapping handles canonical strings +
      unknown / empty
- [x] `CapsuleInspectorView::fixture` carries all 12 RFC §8.3
      mandatory fields
- [x] Install gate blocks until `fields_viewed == fields_total`
- [x] `mark_viewed` is idempotent + unknown-id is a no-op
- [x] Slint scaffold compiles with `--features feat-ui`
- [x] ADR-009 landed
- [x] Test ratchet up (110 → 121)
- [x] Frontmatter coverage 100%
- [ ] Wire wizard's deferred callbacks (DEFERRED)
- [ ] Live tokio integration with upstream ApprovalQueue (DEFERRED)

## Close note (2026-05-21)

The install-gate test is the most load-bearing piece — it
encodes the AC-6 audit artifact's behavior in 30 lines and
catches regressions before they reach a Trail of Bits
assessor. The pattern transfers to any future "must view N
fields before proceeding" gate.

**Next sprint.** S-10c (marketplace browser + chat surface).
Closes the Slint app's surface area for v1.0.
