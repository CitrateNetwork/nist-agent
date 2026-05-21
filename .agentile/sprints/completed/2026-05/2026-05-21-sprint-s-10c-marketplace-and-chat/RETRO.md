---
created: 2026-05-21T00:00:00Z
branch: feat/s-10c-marketplace-and-chat
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: complete
sprint: S-10c
---

# Retro — Sprint S-10c (capsule marketplace + chat surface)

## What landed

The last of the three S-10 sub-sprints. The two surfaces that let
the operator *do* things now exist as headless render models +
`feat-ui`-gated Slint scaffolds.

- 11th workspace crate `nist-agent-marketplace`.
- `MarketplaceView` enforces the overlay-certification filter at
  construction; a downstream `validate_install()` re-checks for
  belt-and-braces and yields the same `CapsuleOverlayMismatch`
  error class for both code paths.
- On-Chain tab is a first-class concept of the render model, not a
  Slint conditional: switching to it returns an
  `OnChainTabDisabled { reason }` error when egress posture forbids,
  so the gate is testable without launching a UI runtime.
- `ChatSessionView` carries the three streaming-lifecycle states
  the agent loop produces (`Idle`, `Streaming`, `PausedAtAction`)
  with `input_enabled()` driving the input-box-enabled property in
  Slint. `validate_resume()` ensures resume signals only fire from
  paused state with a matching checkpoint id.
- Trajectory export gate matches RFC §12 Q3: FedRAMP High
  deployments hide the hint and `validate_export()` refuses with
  `TrajectoryExportForbidden`.

## Metrics delta

| Metric | Before (S-10b) | After (S-10c) | Delta |
|---|---|---|---|
| `cargo test --workspace` | 121 | 139 | +18 |
| Workspace crates | 10 | 11 | +1 |
| ADRs | 9 | 9 | 0 (covered by ADR-009) |
| TLA+ specs | 5 | 5 | 0 |

## What went well

- **Two-surface-per-crate pattern held.** Same shape as
  `nist-agent-hitl`: one crate, two render-model modules, one
  Slint file with two exported components, one `ui.rs` translation
  layer. Pattern reuse meant the Slint compile went green on the
  first try (modulo a clippy `Default` impl).
- **Belt-and-braces gates are cheap.** Pre-filtering listings at
  construction *and* re-validating at Install-time costs a
  one-liner per path but pins the rule in two places, so an
  out-of-band caller can't bypass it. Same shape as
  `check_install_gate()` in S-10b.
- **Stream-state enum was the right shape.** Tagged-`state` serde
  representation makes it easy to embed in PolicyBundle audit
  records (S-6) without a wire-format change.

## What was tricky

- **`Default` derive trap.** `MarketplaceFilter` derives `Default`,
  which transitively requires `MarketplaceSource: Default`. Solved
  by `#[derive(Default)] + #[default] Bundled` on the enum — same
  fix as the wizard's `Step` enum in S-10a. Pattern is worth
  remembering: any time a struct with `derive(Default)` carries a
  custom enum, the enum needs the same.
- **Slint pane-vs-window warnings.** Two warnings about
  `MarketplacePane` / `ChatPane` not inheriting `Window`. Same
  shape as S-10b; the integration site wraps panes inside a
  parent Window component owned by `nist-agent-wizard`. Warning
  is informational, not a build break, but worth a follow-up in
  S-12 (distribution) to silence cleanly.

## What to carry into S-11 / S-12

- Mobile companion deep-link integration (S-11) needs a chat-side
  hook to surface a "scan this on your phone" affordance when the
  active overlay set permits mobile signing. The
  `ChatSessionView::trajectory_export_allowed` pattern is the
  right shape — copy it for `mobile_signing_allowed`.
- The pane→Window wrap can be done once in `nist-agent-wizard`'s
  shell as part of S-12's reproducible-build packaging. Doing it
  earlier would scatter Window components across crates.

## Open follow-ups

- Tokio glue that pipes `Agent::step()`'s streaming model output
  into `ChatStreamState::Streaming { partial }` — deferred to
  S-12 alongside the rest of the bin-level wiring.
- An on-chain CapsuleRegistry adapter that emits the
  `Vec<CapsuleListing>` for the On-Chain tab — the `dyn
  ChainClient` interface exists (S-4); the marketplace just needs
  a `read_capsule_registry()` consumer wired in.
- Multi-tab chat sessions — explicitly out of v1.0.
