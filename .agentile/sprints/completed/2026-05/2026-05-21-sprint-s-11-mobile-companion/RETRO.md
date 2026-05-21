---
created: 2026-05-21T00:00:00Z
branch: feat/s-11-mobile-companion
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: complete
sprint: S-11
---

# Retro — Sprint S-11 (mobile companion protocol)

## What landed

The Rust portion of the mobile signing surface — 12th workspace
crate `nist-agent-mobile-pairing`. The rule layer (pairing state
machine, attestation allowlist, per-overlay eligibility + TTL)
and wire formats now live in code that compiles and tests
without ever opening a phone.

ADR-010 records the explicit scope split: this repo owns the
protocol; the native iOS / Android apps land in sibling repos
when those toolchains' work begins. That keeps mobile-app-store
process out of the Rust workspace and gives Trail of Bits a
small, focused crate to audit (RFC §11.1).

The feature scenarios from `surface-mobile-companion.feature`
pin 1:1 to tests:

- "Pairing is HITL-gated" — `apply_security_officer` refuses
  unless state is `Initiated`; the `PairingRecord` is shaped
  for AuditRecord persistence.
- "Mobile eligibility by overlay" — the table maps to
  `from_active_default` cases for CMMC-L3 / FERPA / HIPAA
  (allowed) and FedRAMP-High (refused).
- "1-hour TTL by default" — `SignatureTtl::DEFAULT.secs() == 3600`
  + `check_not_expired` at the T+ttl boundary is the pinning test.
- "Samsung Knox refused when not allowlisted" —
  `AttestationAllowlist::v1_default().validate_chain(samsung)`
  returns `AttestationNotAllowlisted { chain_id: "samsung-knox" }`.

## Metrics delta

| Metric | Before (S-10c) | After (S-11) | Delta |
|---|---|---|---|
| `cargo test --workspace` | 139 | 167 | +28 |
| Workspace crates | 11 | 12 | +1 |
| ADRs | 9 | 10 | +1 (ADR-010) |
| TLA+ specs | 5 | 5 | 0 |

## What went well

- **State machine pattern is clean.** Six states, four explicit
  transitions; the `InvalidPairingTransition { expected,
  observed }` error makes mis-wirings loud and recoverable. The
  test that walks the happy path is short; the tests that pin
  each refusal are short. Worth porting this shape to the next
  protocol crate that needs it.
- **TTL boundary test paid for itself.** The feature scenario's
  "expires at T + 1 hour" left ambiguity at the boundary; the
  test (`signature_at_ttl_boundary_is_accepted`) decides it
  explicitly. Future-me arguing about whether the comparison
  should be `>` or `>=` has a documented answer.
- **Allowlist as Vec<ChainId> + serde stays simple.** Operators
  can extend the allowlist by editing the PolicyBundle without
  a Rust release. Pinning Samsung Knox as opt-in (rather than
  inverting the rule) keeps the default safe.
- **Token cleared on activation.** The `PairingRecord.token`
  field is `None` after the device's first successful contact,
  so subsequent state transitions (Active → Revoked) don't
  re-persist the secret in audit records. Caught at write-time
  by re-reading the RFC's audit-data minimization principle.

## What was tricky

- **Constant-time token comparison.** The current implementation
  short-circuits on length, which isn't strictly constant-time.
  Documented inline because the daemon sees one token per
  pairing flow and the side-channel is principally hygiene, not
  a defense-in-depth gap. If the protocol ever moves to a
  many-tokens-per-second context, the comparison becomes load-
  bearing and needs to land on `subtle::ConstantTimeEq`.
- **`Duration` serde shape.** `#[serde(transparent)]` on
  `SignatureTtl` inherits `Duration`'s default
  `{"secs":...,"nanos":...}` form. Acceptable for now; if a
  native app's JSON consumer prefers integer seconds, we'll
  switch to a custom `Serialize`/`Deserialize`. The test pins
  the secs field so the contract is observable.
- **Out-of-scope vs deferred.** The feature scenario lists "iOS
  + Android apps build reproducibly" as exit criteria, but
  that's a different toolchain entirely. Calling it
  **deferred** (with the ADR documenting where to) rather than
  shrinking the scenario keeps the exit criteria honest. Same
  pattern S-10b used for "wire the queue's tokio handle".

## What to carry into S-12

- Daemon-side wiring: an HTTP route that accepts
  `SignedDecision` JSON, runs `MobileEligibility::check_not_expired`
  + `AttestationAllowlist::validate_chain`, then routes to
  `ApprovalQueue::approve()` / `reject()`.
- Persistence: serialize `PairingRecord` transitions via the
  existing `nist-agent-audit-sinks` crate's WORM sink.
- Doctor check #12 (new): "mobile-pairing-allowlist-not-empty"
  — warn the operator if mobile signing is enabled by overlay
  but the allowlist is empty.

## What to carry into the future native-app repos

- The Rust crate is the canonical source of truth. The apps
  must not re-implement the eligibility / TTL / allowlist rules
  — they consume them via FFI or against the serde-pinned JSON
  contract.
- A fixture corpus under `tests/fixtures/` should land in S-12
  so the native apps have something to test their decoder
  against without running the daemon.

## Open follow-ups

- Replace short-circuit token compare with `subtle::ConstantTimeEq`
  if mobile-companion-protocol ever serves multiple concurrent
  pairings per daemon (currently 1:1).
- Custom Duration serde for `SignatureTtl` if a native-app
  decoder prefers integer seconds over the default
  `{secs,nanos}` shape.
- Cryptographic verification of device-attestation chains —
  routes through this crate's `validate_chain` allowlist step
  but the cryptographic verifier itself is deferred.
