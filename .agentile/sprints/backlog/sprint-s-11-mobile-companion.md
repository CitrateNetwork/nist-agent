---
created: 2026-05-20T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: backlog
sprint: S-11
---

# Sprint S-11: Mobile companion — iOS + Android signing surface

**Goal.** Ship a small mobile app for out-of-band approvals: pair
with the daemon via mTLS, fetch pending approvals, sign with Secure
Enclave / StrongBox, with overlay-driven enable/disable and TTL.

**Why now.** Approvers are not always at their desk; without
mobile, tier-low approvals queue up and the daemon SLA slips. RFC
§5.6 calls this out explicitly.

**Predecessors.** S-3 (workspace), S-6 (policy with mobile-eligibility flag).

**Features owned.**
- `features/surfaces/surface-mobile-companion.feature`
- `features/core/hitl-signing-surfaces.feature` (mobile rows)

**Cross-repo.** Pairing protocol may need a small runtime crate
addition — file upstream PR if so.

**Exit criteria.**
- Pairing flow is HITL-gated; pairing record is itself an AuditRecord.
- iOS + Android apps build reproducibly.
- Per-overlay enablement honored (FedRAMP-High/ITAR disable mobile; FERPA/HIPAA allow with TTL).
- Device attestation chains policy-allowlistable.
- End-to-end test: pair, fetch, sign, surface clears the HITL gate.
