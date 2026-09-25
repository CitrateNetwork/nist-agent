---
created: 2026-05-21T00:00:00Z
branch: feat/s-8-overlay-bundles-a
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
overlay: cipa
---

# Deployment Runbook — CIPA Overlay

> Schools with E-Rate obligations. Adds AI-output content
> filtering on top of the CMMC-L3 baseline. Pair-read with
> [`cmmc-l3/RUNBOOK.md`](../cmmc-l3/RUNBOOK.md).

## Scope

| Field | Value |
|---|---|
| **Statute** | Children's Internet Protection Act (47 U.S.C. § 254(h)) |
| **Audit floor** | 5 years (`retention::retention_floor(Overlay::Cipa)`); effective floor with CMMC = 6 years |
| **Required capsule** | CIPA content filter — bundled-tier or managed-tier; intercepts every model token stream destined for a student-facing surface |
| **Filter ordering** | Token-stream filtering happens BEFORE the surface emits; blocked sequences never reach the user |

## Construction

```rust
use nist_agent_overlays::{cipa, OverlayBuilder};
let bundle = cipa(OverlayBuilder {
    role_assignments: build_school_roles(),
    not_before: now_unix,
    expires_at: now_unix + ONE_YEAR_SECONDS,
});
```

Schools typically combine CIPA with FERPA + COPPA:

```rust
let mut bundle = nist_agent_overlays::ferpa(opts);
bundle.overlays.add(nist_agent_prelude::Overlay::Coppa);
bundle.overlays.add(nist_agent_prelude::Overlay::Cipa);
bundle.bundle_name = "school-full".into();
```

## What CIPA adds over the CMMC-L3 baseline

- **CIPA content filter capsule** intercepts every model token
  stream that targets a student-facing surface. Required for
  E-Rate funding eligibility under 47 U.S.C. § 254(h)(5)(B).
- **Filter decision audit records** — every blocked sequence
  produces an AuditRecord of type `ContentFilterDecision` with:
  input hash, filter capsule id, rationale code. The doctor's
  retention check covers these records.
- **Surface-level integration** — the harness emits a "blocked by
  CIPA filter" notice to the user-facing surface; the underlying
  block is recorded in audit.

## Pre-deploy checklist

(Inherits CMMC-L3 baseline checklist, plus:)

- [ ] CIPA filter capsule installed (S-8b — bundles with v1.0
      release; for v0.x, the policy bundle is configured but the
      capsule install is the operator's choice from third-party
      managed capsules)
- [ ] `nist_agent_doctor` reports the filter capsule's
      `content_hash` + signing tier under the daily report
- [ ] Audit log persistence reaches the configured CIPA-eligible
      retention period (5 years; effective 6 with CMMC) — records
      land in the WORM sink and (per operator preference) anchor
      to the configured AnchorRegistry
- [ ] Internet safety policy document referenced in the
      `.agentile/policy/<overlay>/POLICY.md` (operator-authored)

## Decommissioning

Same pattern as FERPA / COPPA. CIPA decommissioning is rare in
practice because E-Rate funding requires the school maintain
the technology-protection-measure clause; decommissioning would
expose the district to ineligibility re-review.

## Rollback

- Revert the PolicyBundle to the previous signed version. The
  rollback is itself an audit event (`PolicyChange` record with
  AU-9(5) dual signature). For an E-Rate-funded district, also
  document the technology-protection-measure status change in
  the workflow file — CIPA rollback exposes the district to
  E-Rate re-review.
- Un-install the CIPA filter capsule through the standard
  HIC-gated capsule-uninstall flow. The harness's network
  posture remains `Disabled` by default after capsule removal.
- Export the audit log for the period during which the CIPA
  bundle applied; archive to the 5-year CIPA-eligible retention
  (effective 6-year under the CMMC-L3 baseline).

## See also

- [`features/overlays/overlay-cipa.feature`](../../../features/overlays/overlay-cipa.feature)
- RFC-CIT-AGENT-0001 §2.3
