---
created: 2026-05-21T00:00:00Z
branch: feat/s-6-policy-bundle
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: accepted
adr: 005
sprint: S-6
---

# ADR-005: PolicyBundle + data-class lattice land locally (third exception-clause exercise)

| Field | Value |
|---|---|
| **ADR Number** | ADR-005 |
| **Date** | 2026-05-21 |
| **Status** | ACCEPTED |
| **Author** | Saul Loveman + Claude Opus 4.7 (1M context) |
| **Sprint** | S-6 |

## Context

RFC-CIT-AGENT-0001 §3.1 names `PolicyBundle` as one of the
load-bearing types in `citrate_agent_core`'s public surface. RFC
§7.2 names the Bell-LaPadula data-class lattice as a normative
invariant the harness enforces at capsule install and on every
per-call data-class check. RFC §2.3 names the overlay activation
ratchet as a one-way rule.

`citrate-agent-runtime`'s `agent/core/src/policy/mod.rs` is an
empty scaffold (header comment only) — the same shape as the
`agent` and `model` modules S-5 filled. Their upstream sprint
(CIT-AGENT-4) has not been sequenced.

ADR-004 set the precedent: ALIGNMENT.md's exception clause says
"land locally as a feature crate, PR upstream within the same
sprint." ADR-005 is the third exercise of that pattern (after
ADR-004 for agent loop + model resolver). S-6 follows the same
shape; this ADR documents the specifics of where each piece lives
and how it bridges with the existing chain crate.

## Decision

**We will land the PolicyBundle, data-class lattice, role/tier/
overlay-state types, and Ed25519 signature verification in a new
`crates/nist-agent-policy` crate.** Module split:

- `types` — `RiskTier`, `Role`, `AnchorStrategy`, `EgressPosture`
- `lattice` — `DataClass` enum + `dominates()` operator
- `overlay_state` — `ActiveOverlays` ratchet + `DecommissioningWorkflow`
- `bundle` — `PolicyBundle`, canonical CBOR encode/decode,
  `RawSignedBundle` with Ed25519 verification
- `error` — `PolicyError` typed variants

Eventually moves into `citrate_agent_core::policy` (same pattern
as ADR-004's `::agent` and `::model`).

### Lattice unification with the chain crate

`nist-agent-chain::types::Clearance` already encodes the same
five levels (Public..Itar) with the same byte assignment because
S-4 needed it to read `AgentSBT.clearance` from on-chain. The
honest answer is that these are *the same lattice* viewed from
two layers:

- `DataClass` (policy-side) — what a data element *is*
- `Clearance` (chain-side) — what a subject is *cleared for*

They are byte-aligned (Public=0, Cui=1, Phi=2, Ferpa=3, Itar=4)
because the on-chain enum and the in-memory enum MUST agree at
the install gate. Two enums with the same shape would normally
violate Rule 9 ("one source of truth per topic"); the alternative
(make `DataClass` canonical in policy, have chain re-export)
would create a `chain → policy` dependency that didn't exist
before. We resolve this with **cross-layer test pinning**:

- `nist-agent-policy::lattice::tests::cross_layer_alignment_with_chain_clearance` pins the byte values 0..4.
- `nist-agent-chain::types::tests` (existing) pins the same byte values via its own enum.
- If either drifts, both tests fail.

When S-5b lands and we restructure to delete `nist-agent-loop`
and `nist-agent-model` on upstream merge, we will revisit this
duplication and consider lifting `DataClass`/`Clearance` to a
shared location (probably `citrate_agent_core::types` or a new
`nist-agent-types` crate). Tracked as a follow-up in the S-6
RETRO.

### Canonical CBOR for signing

The SecurityOfficer signs the canonical CBOR encoding of
`PolicyBundle`. RFC 8949 §4.2.1 deterministic encoding rules
mandate: shortest-form integer encoding, sorted map keys, no
indefinite-length items, lowercase floating-point representation.
ciborium produces this by default.

We do **defense-in-depth canonical verification**: the verifier
decodes the bundle, re-encodes it, and compares bytes against the
input. Any non-canonical encoding (e.g. trailing CBOR bytes the
decoder ignores) is rejected as `PolicyError::NonCanonicalCbor`.
This catches signature-replay attacks where an adversary
substitutes equivalent-but-different bytes the signature
mathematically validates but a strict re-encode wouldn't produce.

## Consequences

**Positive.**

- nist-agent v1.0 critical path proceeds — S-7 (Doctor) can read
  the PolicyBundle for its check #3 (policy bundle signed + in
  validity window); S-8 / S-9 can author overlay-specific bundles
  binding to the typed `ActiveOverlays` ratchet; S-10 (Slint
  concierge) can present `Role::ALL` in the role-enrollment
  pane.
- The canonical-CBOR defense-in-depth is a real protection — the
  `verify_rejects_non_canonical_cbor` test exercises it. Auditors
  can point at the test as evidence.
- The one-way ratchet enforces RFC §2.3's "removal requires a
  documented decommissioning workflow" structurally:
  `ActiveOverlays::remove_with_workflow` requires a typed proof
  parameter; `remove_without_workflow` is a function that always
  errors. The Rust type system makes the rule unbypassable.
- 25 new tests, bringing the workspace to 66. The ratchet floor
  rises.

**Negative.**

- A second `DataClass`/`Clearance` enum exists, with cross-layer
  pin tests. The duplication is rule-9-tolerant only because the
  test pinning is part of CI. If we ever lose the chain test, the
  duplication becomes a real source-of-truth violation. Tracked
  as a follow-up in the RETRO.
- `nist-agent-policy` doesn't depend on `nist-agent-loop` or
  `nist-agent-model`, but the loop + model will eventually
  consume the policy crate (for risk-tier escalation in the loop
  and for model-resolver config in the model). That's a
  follow-up wiring that lands when the upstream PRs merge.
- Bundle versions are encoded as a single `u32`. Future
  forward-compat (adding a field) is easy; future
  backward-compat (removing a field) requires a `bundle_version`
  bump. The `BUNDLE_VERSION = 1` constant marks the v1 wire
  shape.

**Neutral.**

- `RawSignedBundle::signature` is `Vec<u8>` rather than `[u8; 64]`
  because serde's default derive doesn't support arrays > 32
  bytes without a helper crate. The length is enforced at
  `Signature::from_slice` in `verify_and_decode`. Cosmetic;
  doesn't affect security.

## Alternatives considered

1. **Put `DataClass` in `nist-agent-chain` and have policy
   consume it.** Rejected: makes policy depend on chain
   transitively (and chain pulls reqwest, ethereum primitives,
   k256, etc.), which inflates the dependency tree for
   policy-only consumers. Policy is conceptually the higher
   layer; the dep should run the other way.
2. **Put `Clearance` in a new shared `nist-agent-types` crate.**
   Rejected for S-6 because it ripples through both existing
   crates and lengthens the sprint. Reconsidered when the
   upstream merges happen.
3. **Sign the JSON encoding instead of canonical CBOR.** Rejected:
   JSON has multiple legal encodings of the same value (whitespace,
   key order, number formatting), so the canonical-form
   enforcement would require a JSON canonicalizer (RFC 8785 JCS).
   CBOR's canonical form is simpler and matches the format
   citrate-agent-runtime already uses for audit records (per
   `audit::record::canonical_cbor`).

## Reversal conditions

ADR-005 is reversed when:

- The upstream PR for `citrate_agent_core::policy` merges; AND
- The federation manifest bumps the runtime rev; AND
- Our `Cargo.toml` re-pins `citrate-agent-core` to that rev; AND
- The 25 tests transfer cleanly to the upstream test runner.

A successor ADR records the deletion of `nist-agent-policy` and
the re-export changes in `nist-agent-prelude` (PolicyBundle joins
the existing re-exports).

## References

- RFC-CIT-AGENT-0001 §2.3 (overlay activation), §3.1 (PolicyBundle
  declaration), §5.2 (risk tiers), §5.3 (five-role lattice), §6.3
  (anchor strategies), §7.2 (Bell-LaPadula lattice).
- [`ADR-002`](ADR-002-tla-custody-in-nist-agent.md) — TLA+
  custody (the lattice spec lives here).
- [`ADR-003`](ADR-003-evm-chain-adapter-trait.md) — chain
  adapter (where `Clearance` currently lives).
- [`ADR-004`](ADR-004-agent-loop-and-model-resolver-placement.md)
  — the precedent for the exception-clause pattern.
- [`.agentile/planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md`](../planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md) — the exception clause.
- Federation rules 1, 9 (one source of truth — and the cross-layer
  test pinning that lets us tolerate the `DataClass`/`Clearance`
  duplication).
- `.agentile/formal/specs/agent/DataClassLattice.tla` — the TLA+
  spec the lattice tests echo (proves the dominates() ordering
  holds in the safety invariants).
- `nist-agent-chain/src/types.rs` — the existing `Clearance` enum
  whose byte assignments the new `DataClass` pins.
