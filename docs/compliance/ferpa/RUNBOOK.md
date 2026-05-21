---
created: 2026-05-21T00:00:00Z
branch: feat/s-8-overlay-bundles-a
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
overlay: ferpa
---

# Deployment Runbook — FERPA Overlay

> School-deployed agents handling student records. CMMC-L3
> baseline plus FERPA-specific quorum + retention. Pair-read
> with [`cmmc-l3/RUNBOOK.md`](../cmmc-l3/RUNBOOK.md).

## Scope

| Field | Value |
|---|---|
| **Statute** | Family Educational Rights and Privacy Act (20 U.S.C. § 1232g) |
| **Audit floor** | 5 years (`retention::retention_floor(Overlay::Ferpa)`); effective floor with CMMC = 6 years (MAX) |
| **Data classes recognized** | `FERPA-restricted` (student records), `FERPA-directory` (directory information) |
| **Mobile signing** | Permitted with 1-hour TTL (vs 24h on desktop). Both `Overlay::Ferpa.forbids_mobile_signing()` returns `false`. |
| **ComplianceOfficer signature** | REQUIRED before any FERPA-restricted emit, regardless of risk tier |

## Construction

```rust
use nist_agent_overlays::{ferpa, OverlayBuilder};
use std::collections::BTreeMap;

let mut role_assignments = BTreeMap::new();
// ... populate with the school's five-role identities ...

let bundle = ferpa(OverlayBuilder {
    role_assignments,
    not_before: now_unix,
    expires_at: now_unix + ONE_YEAR_SECONDS,
});

// SecurityOfficer signs:
let canonical = bundle.encode_canonical()?;
let signature = security_officer_key.sign(&canonical).to_bytes().to_vec();
let raw = RawSignedBundle { bundle_cbor: canonical, signature };
```

## What FERPA adds over the CMMC-L3 baseline

- **ComplianceOfficer-mandatory emit** — every capsule call that
  reads FERPA-restricted data and proposes to emit MUST collect
  a ComplianceOfficer signature in addition to the risk-tier
  quorum.
- **Redact-and-attest gating** — emitting FERPA-directory data as
  PUBLIC requires invoking a signed redact-and-attest sub-capsule;
  the attestation is recorded as a separate AuditRecord.
- **5-year retention floor** — CMMC-L3 baseline already enforces
  6 years, so the effective floor is 6. The doctor's retention
  check reads the per-overlay floor + takes MAX.

## Pre-deploy checklist

(Inherits CMMC-L3 baseline checklist, plus:)

- [ ] Configure the redact-and-attest capsule from the bundled
      tier (delivers with nist-agent v1.0; not yet in v0.x —
      tracked in S-8b)
- [ ] Verify that the data-class lattice includes `FERPA-restricted`
      and `FERPA-directory` (S-6's lattice covers PUBLIC..ITAR;
      FERPA-specific sub-classes are bundle-level, not lattice-level)
- [ ] Mobile companion (S-11) configured with 1-hour TTL when
      enrolled by an approver

## Pilot

Phase-1 pilot identification: at least one K-12 school district
deployment exercises the full bundle + the redact-and-attest
capsule + the per-FERPA quorum. Tracked in S-14 (pilot onboarding).

## Decommissioning

FERPA overlay is removable via the documented decommissioning
workflow per RFC §2.3:

1. SecurityOfficer + ComplianceOfficer co-sign a workflow file at
   `.agentile/sprints/active/overlay-decom-<date>.md`.
2. The workflow's SHA-256 goes into the `DecommissioningWorkflow`
   proof token consumed by `ActiveOverlays::remove_with_workflow`.
3. Final audit export of the period during which FERPA applied;
   archive per CMMC-L3 baseline.
4. A `OverlayDecommissioned` AuditRecord is appended.

## See also

- [`features/overlays/overlay-ferpa.feature`](../../../features/overlays/overlay-ferpa.feature)
- RFC-CIT-AGENT-0001 §2.3
