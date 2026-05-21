---
created: 2026-05-21T00:00:00Z
branch: feat/s-8-overlay-bundles-a
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
overlay: coppa
---

# Deployment Runbook — COPPA Overlay

> K-12 deployments handling under-13 PII. Layers on top of
> CMMC-L3 baseline. Pair-read with
> [`cmmc-l3/RUNBOOK.md`](../cmmc-l3/RUNBOOK.md) and (commonly
> together with COPPA) [`ferpa/RUNBOOK.md`](../ferpa/RUNBOOK.md).

## Scope

| Field | Value |
|---|---|
| **Statute** | Children's Online Privacy Protection Act (15 U.S.C. § 6501 et seq.) |
| **Audit floor** | 5 years (`retention::retention_floor(Overlay::Coppa)`); effective floor with CMMC = 6 years |
| **Data class recognized** | `COPPA-restricted` (under-13 PII) |
| **Required pre-call token** | Verifiable parental-consent token (signed by the operator's parental-consent signer) |
| **Auto-escalation** | Any emission involving COPPA data is escalated to tier-high regardless of capsule declaration |

## Construction

```rust
use nist_agent_overlays::{coppa, OverlayBuilder};

let bundle = coppa(OverlayBuilder {
    role_assignments: build_school_roles(),
    not_before: now_unix,
    expires_at: now_unix + ONE_YEAR_SECONDS,
});
// ... sign as in CMMC-L3 runbook ...
```

For schools running both COPPA and FERPA (the common case),
activate both overlays:

```rust
let mut bundle = nist_agent_overlays::ferpa(opts);
bundle.overlays.add(nist_agent_prelude::Overlay::Coppa);
bundle.bundle_name = "ferpa-coppa".into();
```

## What COPPA adds over the CMMC-L3 baseline

- **Parental-consent token required at HITL time** for every
  capsule call that reads `COPPA-restricted` data. Without the
  token, the harness refuses with
  `PolicyError::COPPA: parental-consent missing` (S-8b capsule
  work — for v0.x, the lattice + HITL quorum is in place but
  the token verification path lands with the
  parental-consent-verifier capsule).
- **Verifiable consent** — tokens are signed by an org-specific
  signer whose cert chain is in the operator's trust roots.
  Token expiration is honored at HITL gate.
- **Tier escalation** — capsule's declared tier never wins for
  COPPA emits; the bundle's risk-tier map gets the escalation.

## Pre-deploy checklist

(Inherits CMMC-L3 baseline checklist, plus:)

- [ ] Org parental-consent signer cert is in trust roots
- [ ] Parental-consent-verifier capsule is installed (S-8b)
- [ ] Audit log captures `ParentalConsentTokenValidated` events
- [ ] Risk-tier map escalates capsules that read `COPPA-restricted`

## Decommissioning

Same shape as FERPA — workflow file + dual signature + final audit
export + `OverlayDecommissioned` record. The act of decommissioning
COPPA in a K-12 deployment is itself a sensitive event; the
operator's compliance officer SHOULD document a justification in
the workflow file beyond the formal signature.

## See also

- [`features/overlays/overlay-coppa.feature`](../../../features/overlays/overlay-coppa.feature)
- RFC-CIT-AGENT-0001 §2.3
