---
created: 2026-05-21T00:00:00Z
branch: feat/s-9-overlay-bundles-b
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
overlay: hipaa-hitech
---

# Deployment Runbook — HIPAA / HITECH Overlay

> Covered entities and business associates handling PHI. Layers
> on top of CMMC-L3 baseline. Pair-read with
> [`cmmc-l3/RUNBOOK.md`](../cmmc-l3/RUNBOOK.md).

## Scope

| Field | Value |
|---|---|
| **Statutes** | Health Insurance Portability and Accountability Act (45 CFR Part 164) + HITECH Act |
| **Audit floor** | 6 years (`retention::retention_floor(Overlay::HipaaHitech)`); effective floor with CMMC = 6 years (already at the limit) |
| **Data class recognized** | `PHI` (Protected Health Information) |
| **Required quorum for PHI emits** | ComplianceOfficer + Operator, regardless of declared risk tier |
| **Minimum-necessary policy** | Each PHI-reading capsule MUST declare the subset of fields it consumes; emissions outside that subset are rejected |
| **Breach-notification countdown** | 60 days from any unauthenticated PHI access; doctor reports open countdowns until closed |

## Construction

```rust
use nist_agent_overlays::{hipaa, OverlayBuilder};

let bundle = hipaa(OverlayBuilder {
    role_assignments: build_covered_entity_roles(),
    not_before: now_unix,
    expires_at: now_unix + ONE_YEAR_SECONDS,
});
// ... SecurityOfficer signs the canonical CBOR ...
```

## What HIPAA / HITECH adds over the CMMC-L3 baseline

- **PHI quorum escalation.** Any capsule call that reads PHI
  requires ComplianceOfficer + Operator signatures at the HIC
  gate, even at risk.tier = "low". The bundle just declares the
  overlay; the escalation is enforced at the HIC quorum check
  reading the active overlay set.
- **Minimum-necessary enforcement.** A PHI-reading capsule's
  manifest declares a `data_class.reads` field-list. The
  capsule call must include only those fields; emissions that
  expand beyond the list error before execution.
- **Breach-notification countdown.** Unauthenticated PHI access
  recorded as a `BreachNotificationOpened` audit event with a
  60-day countdown timestamp. Doctor enumerates open countdowns
  until a `BreachNotificationClosed` event closes them.

## Pre-deploy checklist

(Inherits CMMC-L3 baseline checklist, plus:)

- [ ] Each PHI-reading capsule has its minimum-necessary field
      list declared in `data_class.reads`
- [ ] BreachNotificationOpened / Closed event handlers wired in
      the audit chain
- [ ] ComplianceOfficer identity rotation procedure documented
      (HIPAA Security Rule requires periodic review)
- [ ] Business Associate Agreements (BAAs) executed with all
      vendors whose harness instance touches PHI
- [ ] Audit log persistence reaches the 6-year retention floor;
      records persist to the WORM sink and (per operator choice)
      anchor to the AnchorRegistry for tamper-evidence

## Decommissioning

Same shape as FERPA. HIPAA decommissioning is rare for covered
entities; more common for business associates whose contracts
expire. The workflow file documents the BAA termination + the
final audit export covers the contract's effective period.

## Rollback

- Revert the PolicyBundle to the previous signed version. The
  rollback is itself an audit event (`PolicyChange` record with
  AU-9(5) dual signature). For a covered entity, document the
  BAA-status implications in the workflow file before signing —
  HIPAA rollback affects the entity's compliance posture, not
  just the harness configuration.
- Un-install PHI-touching capsules through the standard
  HIC-gated capsule-uninstall flow. The minimum-necessary
  `data_class.reads` declarations are recorded in the install
  audit; the uninstall audit closes the loop.
- Export the audit log for the period during which the HIPAA
  bundle applied; archive to the 6-year retention floor. The
  export must include any `BreachNotificationOpened` /
  `BreachNotificationClosed` events from the period.

## See also

- [`features/overlays/overlay-hipaa.feature`](../../../features/overlays/overlay-hipaa.feature)
- RFC-CIT-AGENT-0001 §2.3
