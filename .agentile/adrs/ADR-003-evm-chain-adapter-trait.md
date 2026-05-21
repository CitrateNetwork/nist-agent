---
created: 2026-05-20T00:00:00Z
branch: feat/s-4-evm-chain-adapter
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: accepted
adr: 003
sprint: S-4
---

# ADR-003: EVM chain adapter trait — abstraction lives in nist-agent

| Field | Value |
|---|---|
| **ADR Number** | ADR-003 |
| **Date** | 2026-05-20 |
| **Status** | ACCEPTED |
| **Author** | Saul Loveman + Claude Opus 4.7 (1M context) |
| **Sprint** | S-4 |

## Context

RFC-CIT-AGENT-0001 §3.2 commits to "compose as a library, deploy as
a daemon, embed as a WASM component" — i.e. the harness MUST be
portable. The §11 sidecar-N1 carve-out goes further: nist-agent
MUST work on Citrate L1 (chain id 40204) *and* any compatible EVM
chain that hosts the five RFC §7.1 contracts (`OrganizationSBT`,
`AgentSBT`, `CapsuleRegistry`, `AnchorRegistry`, `BenchmarkRegistry`).

`citrate-agent-runtime`'s `AnchorRegistryClient`
(`agent/core/src/chain/anchor.rs`) currently takes a `chain_id`
parameter but hard-codes the contract address mapping at the call
site and exposes only the two AnchorRegistry methods. To service
RFC §7.2's Bell-LaPadula install gate (`AgentSBT.clearance`) and
RFC §7.1's capsule-registry reads (`CapsuleRegistry.entryOf`),
nist-agent needs a trait that:

1. Generalizes the five contracts behind one interface.
2. Lets operators on non-Citrate EVM chains supply addresses via
   their signed PolicyBundle.
3. Keeps the privacy guarantees (RFC §7.3) regardless of which
   chain is selected.

The open ownership question in
[`planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md`](../planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md):

> **`trait ChainClient` placement.** Is the trait an upstream
> concept (lives in `citrate-agent-core`) or a sidecar concept
> (lives in a nist-agent crate that *uses* `citrate-agent-core`)?
> Working assumption: upstream — the trait is core-shaped, the
> generic EVM impl is sidecar-shaped.

This ADR closes that question with the **opposite** decision from
the working assumption.

## Decision

**We will hold the `trait ChainClient` and both v1.0
implementations in a new nist-agent crate, `nist-agent-chain`.**
`citrate-agent-runtime` continues to own the concrete
`AnchorRegistryClient` (the original Citrate-bound client); the
Citrate impl in nist-agent-chain (`CitrateChainClient`) is a thin
wrapper that delegates anchor/is-anchored to the runtime's client
and adds the four-other-contract reads using the same RPC
machinery.

Concrete deliverables in S-4:

1. `crates/nist-agent-chain/` workspace member.
2. `trait ChainClient` with five methods: `chain_id`, `anchor`,
   `is_anchored`, `read_clearance`, `read_capsule_registry`.
3. `CitrateChainClient` — default impl, wraps the runtime's
   `AnchorRegistryClient` for writes, uses
   `citrate_wallet_core::chain::RpcClient` directly for the four
   other-contract reads. Contract addresses come from
   `ContractAddresses::citrate_mainnet()` (placeholder values
   pending citrate-chain v1 deployment).
4. `GenericEvmChainClient` — operator-configured impl. Takes RPC
   URL, chain id, and the full `ContractAddresses` struct at
   construction. Signing happens via the same
   `citrate_wallet_core::chain::TransactionBuilder` the runtime
   uses, so the cryptographic surface stays unified.
5. Federation drift constraint `nist-agent → citrate-chain`
   (`citrate-wallet-core` pin) honored — Cargo.toml landed at
   `crates/nist-agent-chain/Cargo.toml`. Drift-check turns green
   for the last open constraint.
6. Unit tests at 11 covering ABI selectors, encoding layouts,
   address derivation, trait-object dispatch across both impls,
   chain-id correctness, key-validation refusal.

## Why the working assumption was wrong

The working assumption in ALIGNMENT.md said the trait should live
in `citrate-agent-core` (upstream). On closer inspection:

1. **The trait's call shape is operator-policy-driven, not engine-
   driven.** Which contracts to read, which RPC URL to dial, which
   chain id to sign for — these are operator choices encoded in the
   signed PolicyBundle (S-6). The engine never needs to *select*
   among adapters; it just consumes a `&dyn ChainClient` passed in
   by the harness. The selection logic belongs near the policy
   bundle parsing, which lives in nist-agent.
2. **Adding the trait upstream creates a release-cadence coupling.**
   Future trait method additions (e.g. RFC v2's SVM namespace, or
   v1.1's `BenchmarkRegistry.postReceipt`) would require a runtime
   release. Holding the trait here lets nist-agent grow the surface
   on its own cadence.
3. **The original `AnchorRegistryClient` shape was over-fitted to
   Citrate.** It hard-codes the Citrate AnchorRegistry contract
   address at construction and the chain_id at call site. Lifting
   it into a general trait *upstream* would have meant a breaking
   change to runtime's public surface. Wrapping it *downstream*
   leaves runtime alone.

If a hypothetical second product (`gdpr-agent`, `solana-agent`)
needs the same trait shape, we lift it upstream then — that's an
ADR-revision, not a re-architecture.

## Consequences

**Positive.**

- The federation manifest's last open drift constraint
  (`nist-agent → citrate-chain` for `citrate-wallet-core`) is
  honored, taking drift-check to 11/11 green.
- Future overlay sprints (S-8 / S-9) and policy-bundle sprint
  (S-6) can bind to `&dyn ChainClient` instead of a concrete type,
  making overlay-driven adapter swaps testable.
- `citrate-agent-runtime` stays narrowly engine-shaped: it owns
  the canonical Citrate anchor client; nist-agent owns the
  abstraction.
- The address derivation function (`derive_address_hex`) duplicates
  the runtime's `audit::recorder::derive_address` *behavior* but
  is independent code. We pin it with a test against the runtime's
  fixture key/address pair, which forces a sync-or-fail discipline
  if either side mutates.

**Negative.**

- We now hold a duplicate of the ABI selector and encoding helpers
  (`anchor`, `isAnchored`). They are 5–10 lines each, byte-aligned
  to the Solidity ABI; the alternative (re-exporting runtime's
  internals) would have widened runtime's public surface for a
  helper that doesn't deserve it. Acceptable cost.
- `GenericEvmChainClient` introduces a code path that's never
  exercised on Citrate Mainnet. Integration testing requires
  spinning up Anvil or a comparable local EVM. S-4 lands the unit
  tests; a follow-up sprint adds Anvil-in-CI under
  `.github/workflows/evm-adapter-integration.yml`.
- The `ContractAddresses::citrate_mainnet()` defaults are
  placeholder values until citrate-chain tags v1 contracts.
  Tracked in S-15 (mainnet anchor).

**Neutral.**

- The runtime's `AnchorRegistryClient::from_hex_key` signature
  takes `chain_id: u64`, so its current shape composes cleanly
  inside `CitrateChainClient::from_hex_key`. No upstream change
  needed in S-4. If a later trait refinement requires upstream
  cooperation (e.g. exposing `inner.signer()` for shared signing),
  that's an upstream PR at that time.

## Alternatives considered

1. **Trait in `citrate-agent-core` (working assumption).**
   Rejected for the three reasons above.
2. **Two separate crates: `nist-agent-citrate` + `nist-agent-evm`.**
   Rejected: trait + impls in one crate is conventional Rust
   shape; splitting before there's a second product needing only
   one impl is premature.
3. **`ethers-rs` or `alloy-rs` instead of citrate-wallet-core.**
   Rejected: pulling in a third heavy ETH client doubles the
   build cost and complicates the supply-chain story (deny.toml
   review for an entire new dep tree). citrate-wallet-core
   already does what we need; we save 80+ transitive crates.
4. **Skip the abstraction; just expose `&AnchorRegistryClient`.**
   Rejected: the RFC's portability commitment requires the
   abstraction; without it, "deploy on any EVM chain" is fiction.

## Reversal conditions

The trait migrates upstream to `citrate-agent-core` if:

- A second product (`gdpr-agent`, `solana-agent`) needs the same
  shape and the duplicate maintenance cost exceeds the upstream-
  coupling cost.
- The runtime gains structural reasons to need the trait at its
  own boundary (e.g. an upstream agent loop that selects adapters
  based on capsule manifests).

If reversed, the migration is one commit per affected repo, plus a
manifest rev bump.

## References

- RFC-CIT-AGENT-0001 §3.2, §7.1, §7.2, §7.3, §11 sidecar-N1.
- [`.agentile/planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md`](../planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md) — "Open ownership questions" section.
- `crates/nist-agent-chain/src/lib.rs` — the trait.
- `citrate-agent-runtime/agent/core/src/chain/anchor.rs` — the concrete Citrate client this wraps.
- Federation rules 9 (one source of truth per topic), 11 (federation manifest canonical), 12 (drift map).
