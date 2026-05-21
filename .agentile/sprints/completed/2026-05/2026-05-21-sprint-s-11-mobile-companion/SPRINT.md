---
created: 2026-05-20T00:00:00Z
branch: feat/s-11-mobile-companion
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: complete
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

## Close note — 2026-05-21

Status: **COMPLETE** (Rust protocol + render-model layer).

ADR-010 records the placement decision: the **protocol** lands in
nist-agent; the **native iOS / Android apps** are deferred to
their own repos under `CitrateNetwork/nist-agent-mobile-{ios,android}`.
This matches the pattern S-10a/b/c used for Slint — the rule
layer is Rust, the renderer is downstream.

**Delivered.**
- `crates/nist-agent-mobile-pairing` (12th workspace crate).
- `pairing.rs` — `PairingRecord` + `PairingState` machine
  (Initiated → SecurityOfficerSigned → DeviceAttested → Active;
  pre-terminal → Rejected; Active → Revoked). HITL-gated +
  AuditRecord-shaped per the feature scenario.
- `attestation.rs` — `DeviceAttestation` envelope + operator-
  configurable `AttestationAllowlist`. Default permits
  `apple-sep` + `google-strongbox`; Samsung Knox refused unless
  the operator opts in (pinning the feature scenario verbatim).
- `eligibility.rs` — `MobileEligibility` per active overlay set
  + `SignatureTtl::DEFAULT` (1 hour per RFC §5.6).
  `check_not_expired()` enforces the TTL on inbound decisions.
- `wire.rs` — `ApprovalSnapshot` (daemon → device) and
  `SignedDecision` (device → daemon) serde-pinned wire forms.

**Metrics.**
- `cargo test --workspace` 139 → 167 (+28).
- Workspace crates 11 → 12.
- ADRs 9 → 10 (added ADR-010).

**Exit criteria — final state.**
- ✅ Pairing flow HITL-gated; record is AuditRecord-shaped.
- ⏸ iOS + Android apps build reproducibly — **DEFERRED** to
  `nist-agent-mobile-ios` / `-android` sibling repos per ADR-010.
- ✅ Per-overlay enablement honored (`MobileEligibility::from_active`).
- ✅ Device attestation chains policy-allowlistable
  (`AttestationAllowlist::validate_chain`).
- ⏸ End-to-end pair-fetch-sign test — requires the daemon's mTLS
  transport (S-12 wiring). The render model is fully testable in
  isolation; transport-level e2e is downstream.

**Carried into S-12.**
- Wire `SignedDecision`'s arrival into the daemon's HTTP/mTLS
  handler; route through `MobileEligibility::check_not_expired`
  before handing to `ApprovalQueue::approve()`.
- Persist `PairingRecord` transitions as `AuditRecord`s via the
  existing audit-sinks crate.
- Open the native-app repos and import this crate via FFI or
  contract-pinned JSON fixtures.
