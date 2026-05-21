---
created: 2026-05-21T00:00:00Z
branch: feat/s-8-overlay-bundles-a
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: completed
sprint: S-8
---

# Sprint S-8 — Retrospective

## Outcome

| Field | Value |
|---|---|
| **Goal achieved?** | YES (4 PolicyBundle factories + retention table + runbooks); WIT/WASM supporting capsules deferred to S-8b |
| **WPs planned / closed** | 7 / 5 in scope; 2 deferred |
| **Carry-forward WPs** | WP-8.6 (WIT/WASM capsules), WP-8.7 (bundle v2 retention) |
| **Closing branch** | `feat/s-8-overlay-bundles-a` (PR pending) |

## Metrics delta

| Axis | Start | End | Δ |
|---|---|---|---|
| Tests | 80 | 93 | **+13** |
| Formal specs | 5 | 5 | 0 |
| CI tripwires | 7 | 7 | 0 |
| Frontmatter coverage | 100% | 100% | maintained |
| Drift constraints | 11/11 green | 11/11 green | 0 |
| Workspace crates | 6 | 7 | +1 (`nist-agent-overlays`) |
| Compliance runbooks | 0 | 4 | **+4** |
| Overlay factories | 0 | 4 | **+4** |

The "runbooks" axis is new this sprint — auditor-facing artifacts
that aren't tests but are evidence. Worth tracking explicitly as
S-9 / S-12 grow more of them.

## What worked

- **S-6's API made the factories trivial.** Each overlay factory is
  ~10 lines: build the baseline, add the overlay variant, rename
  the bundle. The PolicyBundle type was designed in S-6 with
  factories in mind; the work paid off here.
- **Cross-pinning the retention table against the feature file.**
  `audit-retention.feature` had the canonical 6/6/3/5 values; the
  `retention_matches_audit_retention_feature` test pins them. If
  the feature file ever drifts (or the table does), CI catches it.
- **`effective_floor()`'s MAX semantics.** When multiple overlays
  are active, the most-restrictive floor wins. School deployments
  running CMMC + FERPA + COPPA + CIPA effectively get CMMC's 6
  years across the board — which is what RFC §2.3's "overlays MAY
  restrict but MAY NOT relax" rule implies.
- **Runbooks reference the typed factories.** Each runbook shows
  the actual Rust code an operator writes. This is the kind of
  evidence Trail of Bits + DCMA-DIBCAC assessors prefer: not
  prose, but executable instructions.

## What didn't work

- **The capsules deferral felt heavy.** RFC §2.3 names redact-and-
  attest, parental-consent-verifier, CIPA filter as the
  load-bearing capsule set for school deployments. We have the
  POLICY bundle but not the CAPABILITY bundle — the operator
  can configure the overlay but not yet ACT under it. The
  defer-decision was right for sprint scope, but means S-8b is
  required before any K-12 pilot can run end-to-end.
- **Retention is in a constant table, not the bundle itself.** RFC
  §6.4 says retention "expressed in the policy bundle". Today our
  bundle just lists overlays; the floor is derived. Acceptable for
  v0.x; tracked for v2 bundle. Documented in the
  `retention.rs` module comment.

## What surprised us

- **Most of the S-8 work was prose, not code.** The factories are
  ~80 lines of Rust total; the runbooks are ~600 lines of
  markdown. The ratio shifted from prior sprints (where code
  dominated). Suggests Phase 2 sprints (S-12 distribution
  runbooks, S-13 ToB packet) will be even more prose-heavy.
- **`OverlayBuilder` collapsed three constructor params into one
  struct.** Each factory takes `OverlayBuilder` rather than
  three positional args (`role_assignments`, `not_before`,
  `expires_at`). Future S-9 factories inherit the same shape;
  adding a fourth common parameter doesn't break callers.

## Lessons for future sprints

1. **Auditor-facing artifacts count as deliverables.** S-8 closed
   with 4 new runbooks; they're as load-bearing as code in the
   v1.0 evidence packet. Future sprints (S-9, S-12, S-13) should
   track runbook output explicitly.
2. **Cross-pinning tests against feature files** is the right
   discipline for keeping prose + code in sync. When the runbook
   says "5 years retention floor" and the feature says "5 years"
   and the constant says `5 * ONE_YEAR_SECONDS`, the test
   `retention_matches_audit_retention_feature` keeps them aligned.
3. **Factory + builder pattern is the natural fit for overlay
   configuration.** Each overlay is the baseline plus a delta;
   the factory expresses the delta as one line. S-9 will reuse
   this exact shape.

## Pending follow-ups (S-8b candidates)

1. **WIT/WASM capsule bundles** for redact-and-attest (FERPA),
   parental-consent-verifier (COPPA), CIPA content filter.
   Needs cargo-component setup + per-capsule manifest authoring
   + gherkin scenarios + signing.
2. **PolicyBundle v2** with embedded retention field. Bumps
   `BUNDLE_VERSION` to 2; adds `#[serde(default)]` for
   backward-compat read on v1 bundles.
3. **Per-overlay risk-tier escalation tables** populated by the
   factory's `risk_tier_map`. Today each runbook documents the
   escalation prose; encoding it in the factory makes it
   testable.
4. **Operator-facing template generators.** A CLI subcommand
   `citrate-agent-cli generate-bundle --overlay ferpa` that
   prints the canonical CBOR + the signing prompt. Sized as
   half-sprint; lands in S-10 / S-12.
5. **Pilot deployment per overlay.** Tracked in S-14 close-out.

## Next sprint(s)

- **S-9 (Overlay bundles B — HIPAA + FedRAMP-High)** unblocked.
  Adds two more factories + the FedRAMP-High posture deltas
  (PIV-CAC required, Mobile disabled, WORM audit sink) + the
  HIPAA minimum-necessary check + the breach-notification
  countdown. Sized as one focused sprint.
- **S-8b** (capsules + bundle v2 + tier escalation) runs in
  parallel; touches a different crate (`capsules/*`) so no merge
  conflict.
