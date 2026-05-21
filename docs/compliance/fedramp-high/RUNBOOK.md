---
created: 2026-05-21T00:00:00Z
branch: feat/s-9-overlay-bundles-b
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
overlay: fedramp-high
---

# Deployment Runbook — FedRAMP High Overlay

> Federal-system deployments under FedRAMP Rev 5 High baseline.
> Mandates PIV-CAC-only signing, WORM audit storage, no mobile
> companion. Layers on top of CMMC-L3 baseline. Pair-read with
> [`cmmc-l3/RUNBOOK.md`](../cmmc-l3/RUNBOOK.md).

## Scope

| Field | Value |
|---|---|
| **Baseline** | FedRAMP Rev 5 High (GSA / FedRAMP PMO) |
| **Audit floor** | 3 years per FedRAMP (`retention::retention_floor(Overlay::FedrampHigh)`); effective floor with CMMC = 6 years (CMMC wins via MAX) |
| **Signing surfaces** | PIV-CAC smart cards REQUIRED for all approver roles. FIDO2 hardware keys REJECTED at HITL gate. |
| **Mobile signing** | DISABLED (`Overlay::FedrampHigh.forbids_mobile_signing() == true`). The harness refuses mobile pairing requests. |
| **Audit storage** | WORM-eligible REQUIRED. Use `nist_agent_audit_sinks::WormFilesystemSink` or equivalent. |

## Construction

```rust
use nist_agent_audit_sinks::WormFilesystemSink;
use nist_agent_overlays::{fedramp_high, OverlayBuilder};

let bundle = fedramp_high(OverlayBuilder {
    role_assignments: build_federal_roles(),
    not_before: now_unix,
    expires_at: now_unix + ONE_YEAR_SECONDS,
});

// MANDATORY: WORM audit sink. Activation fails otherwise.
let sink = WormFilesystemSink::open("/var/lib/cit-agent/audit-worm")?;
```

## What FedRAMP High adds over the CMMC-L3 baseline

- **PIV-CAC enforcement.** Operator's harness rejects FIDO2 keys
  at HITL signing-surface check when `Overlay::FedrampHigh` is
  active. PIV certificates MUST chain to the operator's
  configured CA root + pass CRL/OCSP revocation check.
- **WORM audit storage.** `WormFilesystemSink` (this sprint)
  creates one file per audit record via `O_CREAT | O_EXCL`. A
  second write of the same sequence errors at the syscall layer
  with `EEXIST`. Each file is 0400 (owner read-only) after
  creation. Pair with POSIX ACLs / SELinux confinement for the
  combined WORM posture FedRAMP assessors review.
- **Mobile signing disabled.** Any pairing attempt fails;
  approvers MUST use Slint desktop + PIV-CAC.
- **Default anchor strategy: HybridNightlyPlusCapsule.**
  Operators wanting maximum on-chain fidelity dial up to
  per-approval (Strategy B) via the standard PolicyBundle
  update flow.

## Pre-deploy checklist

(Inherits CMMC-L3 baseline checklist, plus:)

- [ ] PIV-CAC CA root cert installed at the harness's configured
      trust path
- [ ] CRL/OCSP source reachable from every harness instance
- [ ] `WormFilesystemSink` configured at the audit storage path;
      doctor's audit-file-permissions check returns Pass
- [ ] Mobile companion disabled in operator config (it's
      disabled by overlay default; verify the config didn't
      override)
- [ ] FedRAMP-Moderate-or-higher hosting environment (the harness
      can run on-prem or on FedRAMP-authorized cloud only)
- [ ] System Security Plan (SSP) drafted referencing this runbook
      + the doctor's daily reports as continuous-monitoring
      evidence

## ATO submission notes

Doctor's daily TOML report serves as continuous-monitoring
evidence for FedRAMP's monthly delivery cadence. The signed
report + the WORM audit log (`./scripts/export-audit-window.sh`
exports a date range with hash-chain verification) together
form the FedRAMP-acceptable evidence packet.

## Decommissioning

FedRAMP-High decommissioning happens when an ATO is rescinded
or the system transitions to FedRAMP-Moderate. The workflow
file documents the ATO transition + final audit export covering
the FedRAMP-High period. The WORM sink stays intact (immutable
records); the decommissioning event is a new record in the
chain.

## Rollback

- Revert the PolicyBundle to the previous signed version. The
  rollback is itself an audit event (`PolicyChange` record with
  AU-9(5) dual signature). For an ATO-bearing system, the
  rollback must be a documented step in the ATO transition plan;
  rolling back without coordinating the Authorization-to-Operate
  state is an ATO violation.
- Un-install FedRAMP-High-overlay capsules through the standard
  HITL-gated capsule-uninstall flow. The WORM audit sink stays
  intact (immutable records); the capsule uninstall is itself an
  additional WORM record.
- Export the audit log for the period during which the
  FedRAMP-High bundle applied; archive per the FedRAMP
  continuous-monitoring evidence packet shape. The signed daily
  doctor reports for the period are part of this export.
- Mobile signing remained `Disabled` for the entire FedRAMP-High
  window; no mobile-related cleanup is required.

## See also

- [`features/overlays/overlay-fedramp-high.feature`](../../../features/overlays/overlay-fedramp-high.feature)
- [`nist-agent-audit-sinks::WormFilesystemSink`](../../../crates/nist-agent-audit-sinks/src/worm.rs)
- RFC-CIT-AGENT-0001 §2.3, §5.6, §6.2
