---
created: 2026-05-21T00:00:00Z
branch: feat/s-10a-concierge-wizard
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: completed
sprint: S-10a
closed: 2026-05-21T00:00:00Z
---

# Sprint S-10a: Slint concierge — first-run setup wizard

## Sprint Metadata

| Field | Value |
|---|---|
| **Sprint ID** | `S-10a` |
| **Sprint Name** | Concierge wizard — headless state machine + Slint scaffold |
| **Goal** | Land `crates/nist-agent-wizard` per RFC §8.1: headless state machine + optional Slint UI binding. State machine accepts org identity, role assignments, hardware keys, overlay selection, and emits a SecurityOfficer-signed PolicyBundle. Slint UI compiles when `feat-ui` is enabled. |
| **Branch** | `feat/s-10a-concierge-wizard` |
| **Start Date** | 2026-05-21 |
| **End Date** | 2026-05-21 |
| **Status** | `COMPLETE` |
| **Planset** | [`../../planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md`](../../planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md) |
| **Predecessors** | S-3, S-5, S-6, S-8, S-9 (all six overlay factories) |

## Why this sprint

Operator onboarding is the Phase-2 pilot gate. The concierge
wizard is the smallest standalone slice that delivers
operator-visible value — without HITL queue UI or marketplace,
operators can still complete first-run setup, sign a
PolicyBundle, and start the daemon.

Split from the original S-10 per the 2026-05-21 planset
refactor (S-10 → S-10a + S-10b + S-10c).

## Deliverables

- `crates/nist-agent-wizard/` — new workspace member.
  - `Wizard` state machine — 7 step states, 11 transitions,
    pure-Rust, no Slint dep in default build.
  - `StepAudit` typed sum (5 event variants).
  - `pick_bundle` — most-specific overlay routing (FedRAMP-High
    > HIPAA > FERPA > COPPA > CIPA > CMMC-L3 baseline).
  - Slint UI scaffold at `ui/wizard.slint` with five panes;
    compiles when `--features feat-ui`.
  - 9 unit tests covering transition guards + role-completeness
    + output-path collision + hardware-key validation +
    audit-log shape + overlay routing.
- ADR-008: wizard stays in nist-agent permanently. Product
  UI, not engine surface. First crate in the workspace with
  no upstream-PR plan.
- Test count: 101 → 110 (+9).

## Test Baseline (start of sprint)

| Metric | Count | Captured |
|---|---|---|
| Tests | 101 | 2026-05-21 |
| Formal specs | 5 | 2026-05-21 |
| CI tripwires | 7 | 2026-05-21 |
| Frontmatter coverage | 100% | 2026-05-21 |
| Drift constraints | 11/11 green | 2026-05-21 |
| Workspace crates | 8 | 2026-05-21 |

## Method

State machine as pure-Rust lib (no Slint dep). Slint UI behind
`feat-ui` feature gates the heavy renderer deps. Tests cover
the state machine; Slint binding exercised at S-12 build time.

## Work Packages

### WP-10a.1 — `Wizard` state machine (DONE)
7 step states (Idle → OrgIdentity → Roles → HardwareKeys →
Overlays → Review → Signed). Transitions guarded by
`assert_step`. Helpers for role completeness + hardware-key
validation.

### WP-10a.2 — `StepAudit` typed sum (DONE)
5 variants matching RFC §8.1 steps. Serializable; the harness
wraps into `AuditRecord` at integration time.

### WP-10a.3 — `pick_bundle` overlay routing (DONE)
Most-specific factory wins. Test
`picks_fedramp_high_when_active` pins the rule.

### WP-10a.4 — Slint UI scaffold (DONE)
`ui/wizard.slint` with five panes. `build.rs` gates the slint
codegen via `CARGO_FEATURE_FEAT_UI`. `src/ui.rs` wires
welcome → begin; remaining transitions scaffolded in the
.slint file but Rust-side callbacks defer to S-10b.

### WP-10a.5 — ADR-008 (DONE)
No upstream PR plan. Product UI, not engine surface.

### WP-10a.6 — Wire remaining Slint callbacks (DEFERRED to S-10b)

### WP-10a.7 — Gemma 4 E2B conversational copy (DEFERRED to S-10c)

## Daily updates

- 2026-05-21 — Kickoff and close in one session. 9 new tests
  bring the workspace to 110. fmt + clippy clean.

## Exit criteria

- [x] `Wizard` state machine + 9 tests pass
- [x] All transitions reject out-of-order calls
- [x] Role-completeness check errors on missing role
- [x] Output-path collision check errors on existing file
- [x] `sign_and_write` produces a verified `RawSignedBundle`
- [x] Most-specific overlay wins
- [x] Slint scaffold compiles with `--features feat-ui`
- [x] ADR-008 landed
- [x] Test ratchet 101 → 110
- [ ] Wire remaining Slint callbacks (DEFERRED)
- [ ] Conversational copy via Gemma 4 E2B (DEFERRED)

## Close note (2026-05-21)

The state machine is the load-bearing piece — testable,
headless, embeddable in any UI framework. The Slint scaffold
demonstrates how the same state machine drives a GUI; full
wiring lands alongside S-10b/c.

ADR-008 closes the "does every crate migrate upstream?"
question: product UI stays here permanently.

**Next sprint.** S-10b (HITL queue UI + Capsule Inspector).
