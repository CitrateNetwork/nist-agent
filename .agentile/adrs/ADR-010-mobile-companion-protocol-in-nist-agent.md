---
created: 2026-05-21T00:00:00Z
branch: feat/s-11-mobile-companion
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: accepted
adr: 010
sprint: S-11
---

# ADR-010: Mobile-companion protocol + render models in nist-agent; native apps deferred

| Field | Value |
|---|---|
| **ADR Number** | ADR-010 |
| **Date** | 2026-05-21 |
| **Status** | ACCEPTED |
| **Author** | Saul Loveman + Claude Opus 4.7 (1M context) |
| **Sprint** | S-11 |

## Context

RFC §5.6 specifies an out-of-band mobile signing surface so
approvers are not chained to their desk. The mobile companion has
two distinct layers:

1. **Pairing + wire protocol + eligibility rules + attestation
   allowlist + render models.** Pure Rust. Belongs in the
   nist-agent workspace alongside the other product surfaces.
2. **Native iOS app + Android app.** Swift + Kotlin (or Compose
   Multiplatform). Builds reproducibly, ships through TestFlight /
   Play Internal Testing, links Secure Enclave / StrongBox APIs.

Building (2) inside this Rust workspace is a category error —
it's a different toolchain, a different release cadence, and a
different audit surface (mobile-app-store review vs Trail of
Bits Rust review).

The S-10 surfaces (S-10a wizard, S-10b HITL, S-10c marketplace +
chat) all followed the same playbook: ship the **render models +
protocol layer** in a Rust crate so the rules are testable in
headless CI; defer the rendering integration (Slint for desktop,
Swift/Kotlin for mobile) to a downstream packaging step. S-11
does the same.

## Decision

**S-11 lands `nist-agent-mobile-pairing` — a Rust crate that
owns the pairing state machine, wire formats, per-overlay
eligibility rules, attestation allowlist, and the render models
the mobile UIs will consume.** The crate is the canonical source
of truth for what counts as a valid pairing, what counts as an
eligible signing surface, and what counts as an allowed device
attestation. Both the desktop daemon and the future native apps
read from it (the apps via either FFI / WASM or a JSON-pinned
contract — pick at packaging time).

**The native iOS and Android apps are deferred** to their own
repos under `CitrateNetwork/nist-agent-mobile-ios` and
`-mobile-android`. Reasons:

- App-store builds need their own CI lanes, signing certificates,
  device-farm tests — none of which belong in the Rust workspace.
- Treating each app as a separate consumer of the protocol crate
  forces the contract to be language-clean. If the apps were in
  this workspace they'd be tempted to reach into Rust internals,
  which would defeat the purpose.
- The protocol-only crate is what Trail of Bits will audit (per
  RFC §11.1); native-app pen-testing is a separate engagement
  with mobile specialists.

### What S-11 ships

- `pairing.rs` — `PairingRecord`, `PairingState` machine
  (Initiated → SecurityOfficerSigned → DeviceAttested →
  Active|Rejected), `PairingToken` (short-lived nonce).
- `attestation.rs` — `DeviceAttestation` enum (AppleSep,
  GoogleStrongbox, etc.) + `AttestationAllowlist` policy filter
  per the feature scenario "Samsung Knox refused when not on
  allowlist".
- `eligibility.rs` — `MobileEligibility` per active overlay set.
  Mirrors `Overlay::forbids_mobile_signing()` and adds TTL math
  (default 1 hour per the feature scenario).
- `wire.rs` — Wire formats: `ApprovalSnapshot` (daemon →
  device) and `SignedDecision` (device → daemon).
- `error.rs` — Typed errors for every rejection path.

### What S-11 defers

- Native iOS app (Swift + SwiftUI + SecKey/SEP).
- Native Android app (Kotlin + Compose + StrongBox).
- The mTLS transport itself — handled by the daemon's existing
  rustls stack (S-12 wiring). The pairing crate produces the
  *credentials* and *audit records*; transport is downstream.
- Device-attestation chain verification cryptography — the
  allowlist accepts a chain envelope and routes verification to
  the underlying crypto crate (deferred to S-12 packaging or a
  future runtime-side contribution).

## Consequences

**Positive.**

- Mobile eligibility per overlay is testable from `cargo test`,
  not from an iOS simulator. The feature scenario's example
  table maps 1:1 to a `#[test]` table.
- TTL math is one function; the daemon, the desktop UI, and the
  mobile apps all read the same answer.
- Attestation allowlist is config-driven (a `Vec<String>` of
  trusted chain ids), so an operator can add a new attestation
  vendor without a Rust release.
- The crate is small enough that Trail of Bits can audit it in
  a single review pass.

**Negative.**

- A consumer (the iOS app, the Android app) must either embed
  the Rust crate via FFI or implement the rules a second time
  against a JSON contract. We accept the cost — pinning the
  wire form via serde + a `*.json` fixture corpus is cheap
  enough that drift is detectable in CI.
- We commit to maintaining the protocol crate's stability;
  every breaking change forces both apps to rev.

**Neutral.**

- The crate depends on `nist-agent-prelude` (for `Overlay`) and
  `citrate-agent-core` (for `AuditRecord`, when the pairing
  record is persisted). Same pattern as `nist-agent-hitl`.

## Alternatives considered

1. **Ship a minimal iOS + Android app in this sprint.** Rejected
   — that's a multi-week native-dev effort and the wrong
   toolchain for this workspace. Doing it badly in one sprint
   would burn audit budget.
2. **Put the protocol upstream in `citrate-agent-core`.**
   Rejected for the same reason as the HITL UI (ADR-009): the
   protocol is product-shaped (per-overlay rules, vendor
   allowlist, TTL defaults) — those decisions belong in the
   product, not the engine. Upstream's role is the generic
   `ApprovalQueue` + `AuditRecord`; this crate produces the
   AuditRecord but does not extend the engine's contract.
3. **Build it as part of `nist-agent-hitl` since both are
   approval surfaces.** Rejected — mobile is an out-of-band
   surface with its own lifecycle (pairing, attestation, TTL
   expiry). Keeping it in its own crate makes the protocol
   audit-able in isolation.

## Reversal conditions

If the federation later decides that the mobile protocol is
generic enough to host multiple non-nist-agent products (e.g.,
another sidecar wants to share a mobile signing surface), the
crate migrates upstream to a new `citrate-mobile-companion`
crate. Until that demand is concrete, the crate stays here.

## References

- RFC-CIT-AGENT-0001 §5.6 (mobile signing surface), §8 (HITL).
- ADR-008 / ADR-009 — sibling product-stays-here decisions.
- `crates/nist-agent-mobile-pairing/` — the crate this ADR
  governs.
- `features/surfaces/surface-mobile-companion.feature` — the
  scenarios this crate's tests pin.
- Federation rule 9 (one source of truth).
