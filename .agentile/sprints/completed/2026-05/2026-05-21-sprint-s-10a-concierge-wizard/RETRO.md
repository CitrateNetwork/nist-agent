---
created: 2026-05-21T00:00:00Z
branch: feat/s-10a-concierge-wizard
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: completed
sprint: S-10a
---

# Sprint S-10a — Retrospective

## Outcome

| Field | Value |
|---|---|
| **Goal achieved?** | YES (state machine + Slint scaffold); WP-10a.6 (full Slint wiring) + WP-10a.7 (model-driven copy) deferred to S-10b / S-10c |
| **WPs planned / closed** | 7 / 5 in scope; 2 deferred |
| **Carry-forward WPs** | WP-10a.6 (Slint callback wiring), WP-10a.7 (Gemma 4 E2B copy) |
| **Closing branch** | `feat/s-10a-concierge-wizard` (PR pending) |

## Metrics delta

| Axis | Start | End | Δ |
|---|---|---|---|
| Tests | 101 | 110 | **+9** |
| Formal specs | 5 | 5 | 0 |
| Workspace crates | 8 | 9 | +1 (`nist-agent-wizard`) |
| ADRs | 7 | 8 | +1 |
| Frontmatter coverage | 100% | 100% | maintained |
| Drift constraints | 11/11 green | 11/11 green | 0 |

## What worked

- **The headless + feature-gated split.** Slint is heavyweight
  (system lib deps + ~30s codegen). Putting the state machine
  in the lib and the UI behind `feat-ui` keeps CI fast and gives
  operators a CLI-only embedding path. The pattern transfers to
  S-10b / S-10c.
- **`pick_bundle` precedence routing.** Operators activate
  overlays in any order; the wizard picks the most-specific
  factory automatically. Test
  `picks_fedramp_high_when_active` pins the rule (most
  restrictive wins) so future addition of a v1.1 overlay
  (IL5 / ITAR) doesn't accidentally demote FedRAMP-High.
- **Audit events as a typed sum.** `StepAuditKind` is a Rust
  enum that serializes via serde; the harness's audit
  pipeline converts to `AuditRecord` at integration time.
  Decouples the wizard from the upstream audit chain crate.
- **Step ordering as a compile-helper.** `Step` derives
  `PartialOrd` so the output-path-allowed-on-or-before-Review
  check reads naturally as `if self.step >= Step::Signed`.

## What didn't work

- **Aspirational `Serialize_repr` derives in the first draft.**
  I wrote `#[derive(Serialize_repr, Deserialize_repr)]` thinking
  the proc-macros existed in scope; they don't (they live in
  the `serde_repr` crate we didn't pull in). Caught at first
  build; corrected with plain derives (Step doesn't need to
  serialize across the wire anyway). Reminder: when you reach
  for a proc-macro, verify it's actually a derive and not just
  named-similarly.
- **Slint build script + `optional = true` on slint-build.**
  Marking `slint-build` as optional in `[build-dependencies]`
  doesn't gate it at the build-script compile level — `build.rs`
  always compiles and references `slint_build::compile` would
  fail without the dep present. Fixed by making slint-build
  unconditional (it's a small codegen-only crate) and gating
  the actual codegen call via `CARGO_FEATURE_FEAT_UI` env-var
  check at build time.
- **Forgot the `#[default]` attribute on the Step enum's Idle
  variant.** Wrote a manual `impl Default for Step` which
  clippy correctly flagged as `this impl can be derived`.
  Replaced with `#[derive(Default)]` + `#[default]` on
  `Idle = 0`.

## What surprised us

- **The state machine fits in ~270 lines** including doc
  comments. The previous estimate "Slint UI design is
  non-trivial" was right — but the UI design is the .slint
  file (~150 lines, mostly declarative), and the *state
  machine* underneath is the small piece. Future S-10b /
  S-10c are likely similar shapes: medium-sized .slint files
  + small Rust orchestration.
- **`nist_agent_overlays::OverlayBuilder::placeholder()`** was
  the perfect test helper. Authored in S-8 without anticipating
  S-10a's tests; now S-10a uses it constantly. Worth a note in
  future sprint kickoffs: scan recent crates' `#[cfg(test)]`
  modules for reusable fixtures.

## Lessons for future sprints

1. **Feature-gate heavy UI deps.** The pattern landed here —
   default build is headless, `feat-ui` adds the GUI — should
   propagate to S-11 (mobile companion will have iOS / Android
   build deps), S-10b, S-10c.
2. **Audit events as a typed sum decouple wizard-style flows
   from the audit chain.** Future wizards / flows in
   nist-agent (overlay decommissioning UI, capsule install
   flow in S-10b) should adopt the same shape.
3. **ADR-008 closed the "does every nist-agent crate migrate
   upstream?" question.** Product-shaped crates stay; engine-
   shaped crates migrate. Future sprints citing the exception
   clause should also cite ADR-008 to clarify which side of
   the line they're on.

## Pending follow-ups

1. **Wire remaining Slint callbacks** — `org-identity-submitted`,
   `role-add-clicked`, etc. → call corresponding wizard
   methods. Sized as ~half-day inside S-10b.
2. **Gemma 4 E2B conversational copy** — replace the static
   prompts with model-rewritten copy via
   `nist_agent_model::Model::resolve()`. Lands in S-10c
   alongside the chat surface.
3. **CI Slint feature build** — needs libfontconfig +
   libxkbcommon on the runner. Belongs in S-12 distribution
   track because the same system-lib chain is what the actual
   binary needs.
4. **Wizard → AuditChain integration.** The wizard accumulates
   `StepAudit` events; the harness must convert them to
   `AuditRecord` and call `AuditSink::append`. Wired into the
   daemon startup in S-10b or S-12.

## Next sprint(s)

- **S-10b** (HITL approval queue UI + Capsule Inspector pane).
  Reuses the Slint scaffold pattern from S-10a; adds two more
  panes; wires the wizard's deferred callbacks at the same
  time.
- **S-10c** (Marketplace browser + chat surface). Last of the
  S-10 sub-sprints; wires Gemma 4 E2B for chat + the wizard's
  prompt rewriting.
- S-11 (mobile companion) can run parallel to S-10b/c.
