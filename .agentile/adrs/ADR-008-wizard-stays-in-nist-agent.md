---
created: 2026-05-21T00:00:00Z
branch: feat/s-10a-concierge-wizard
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: accepted
adr: 008
sprint: S-10a
---

# ADR-008: Wizard stays in nist-agent permanently (no upstream migration)

| Field | Value |
|---|---|
| **ADR Number** | ADR-008 |
| **Date** | 2026-05-21 |
| **Status** | ACCEPTED |
| **Author** | Saul Loveman + Claude Opus 4.7 (1M context) |
| **Sprint** | S-10a |

## Context

ADRs 002 / 004 / 005 / 006 / 007 documented five exception-clause
exercises where the local landing migrates upstream to
`citrate-agent-core` after the upstream PR lands. The pattern is:
build it locally in nist-agent because we need it now; PR it
upstream because it belongs there; delete the local crate on
merge.

S-10a's `nist-agent-wizard` is structurally different. The
wizard is the **product UI**, not the **engine**. It:

1. Composes typed factories from `nist-agent-overlays` (which is
   product-shaped — overlays are nist-agent's compliance posture
   contribution, not the engine's).
2. Targets a specific operator audience (DIB / K-12 / FedRAMP)
   shaped by nist-agent's product proposition.
3. Embeds product copy (org name prompts, role descriptions in
   the .slint file) that's nist-agent-branded.
4. Uses Slint, which is a GUI toolkit, not an engine concern.

ADR-005 noted "ADR-008's wizard placement is the natural sibling"
when discussing why PolicyBundle types belong here; this ADR
makes that explicit.

## Decision

**`nist-agent-wizard` lives canonically and permanently in
`nist-agent`. There is no upstream PR plan.** The crate is part
of the product, not the runtime.

If a future product (`gdpr-agent`, `solana-agent`, hypothetical
`cmmc-l5-agent`) needs its own first-run wizard, it forks
`nist-agent-wizard` or builds its own. The state machine pattern
is reusable; the specific wizard's copy + flow is not.

### Headless lib + optional Slint UI

The wizard ships as a single crate with two layers:

1. **Default build: headless state machine.** Pure Rust, no Slint
   dep. Tests run in CI without system libs. The
   `Wizard` struct + `Step` enum + `WizardState` are the
   contract. Operators integrating into a non-Slint surface
   (CLI, web UI, browser-WASM) drive the state machine
   directly.
2. **`feat-ui` feature: Slint binding.** Adds the `slint` runtime
   crate + `slint-build` codegen invocation. The
   `crates/nist-agent-wizard/ui/wizard.slint` file declares the
   five panes; `src/ui.rs` binds them to the state machine.
   Built when operators want the GUI binary (S-12 distribution
   pipeline always builds with feat-ui).

`slint-build` stays as an unconditional `[build-dependencies]`
entry because it's small (codegen-only) and feature-gating it
makes build.rs harder to write portably. The actual codegen
invocation is gated by `CARGO_FEATURE_FEAT_UI` env-var check at
build time.

### What S-10a ships vs defers

S-10a delivers the **state machine and the Slint scaffold**. The
five-pane UI compiles when `feat-ui` is on; the `launch()`
helper opens a window with the welcome pane and the Begin
button. Wiring the remaining four panes to live state
transitions is S-10b/c follow-up work — the wizard's *logic* is
complete; its *UI plumbing* is a sketch.

This is deliberate: S-10a's job is to prove the state machine
is testable + extensible. S-10b lands when the HITL approval
queue UI ships, and the same Slint scaffolding gets the same
wiring treatment.

## Consequences

**Positive.**

- The headless state machine is testable in CI without any
  system-lib setup. 9 tests cover transition guards, role
  completeness, output collision, hardware-key validation,
  audit-log shape.
- Operators building a CLI-only deployment can drive the
  wizard from a `read_line` loop without pulling in Slint.
- Future products that fork the wizard get a clean starting
  point — the `Step`/`Wizard`/`WizardState` triad is the
  copy-and-edit-able pattern.
- ADR-008 closes the open question "does every nist-agent
  crate migrate upstream eventually?" — no. Product-shaped
  crates stay.

**Negative.**

- The Slint scaffold in S-10a is intentionally partial. An
  operator running `cargo run --features feat-ui` today
  sees the welcome pane and a Begin button that advances to
  the org-identity pane, but the remaining transitions
  aren't wired to the state machine. The wiring is
  mechanical (each callback calls a wizard method); it's
  just not S-10a's scope.
- The `nist-agent-wizard` crate name doesn't make the
  product/engine split obvious to a casual reader of
  workspace.members. ADR-008 documents the distinction so
  the convention isn't accidentally violated.

**Neutral.**

- `nist-agent-wizard` depends on `nist-agent-overlays` and
  `nist-agent-policy`. Both of those still have upstream-PR
  plans (per ADR-005 they'd move into `citrate-agent-core`).
  When those migrations happen, this crate updates its `use`
  statements but doesn't itself migrate.

## Alternatives considered

1. **Plan to migrate `nist-agent-wizard` upstream like the
   others.** Rejected: the upstream `citrate-agent-core` is
   the engine; UI components don't belong there. Other
   federation crates (citrate-gui-native, citrate-defense_prime-shell)
   demonstrate the same product/engine split.
2. **Put the wizard in `citrate-gui-native` instead of
   nist-agent.** Rejected: citrate-gui-native is the
   federation's general-purpose desktop GUI; nist-agent's
   wizard is product-specific. Sharing widgets is fine;
   sharing the wizard's specific shape would force one
   product's flow on another.
3. **Skip Slint entirely; ship CLI-only for v0.x.** Rejected:
   RFC §8 names the Slint app as v1.0 deliverable. The
   headless + feature-gated split lets us deliver both shapes
   from one crate.

## Reversal conditions

ADR-008 is reversed when:

- A second nist-agent-shaped product needs the same wizard
  flow AND
- The federation decides to lift it into a shared crate AND
- The shared crate's home (citrate-gui-native or a new
  citrate-onboarding-kit) is identified.

Until then, the wizard stays here.

## References

- RFC-CIT-AGENT-0001 §8.1, §8.2 (concierge spec), §3.2 (UI is
  product-shaped, engine surface is library-shaped).
- ADRs 002/004/005/006/007 — the five exception-clause
  exercises that DO migrate upstream.
- `crates/nist-agent-wizard/` — the crate this ADR governs.
- `crates/nist-agent-overlays/` — the factory crate the
  wizard composes.
- Federation rule 9 (one source of truth — product-shaped
  crates are the product's source of truth).
