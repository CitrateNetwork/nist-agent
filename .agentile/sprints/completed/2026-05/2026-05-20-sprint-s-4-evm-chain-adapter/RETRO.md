---
created: 2026-05-20T00:00:00Z
branch: feat/s-4-evm-chain-adapter
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: completed
sprint: S-4
---

# Sprint S-4 — Retrospective

## Outcome

| Field | Value |
|---|---|
| **Goal achieved?** | YES |
| **WPs planned / closed** | 8 / 7 (WP-4.8 explicitly deferred) |
| **Carry-forward WPs** | WP-4.8 (Anvil-in-CI integration test) → S-6 close |
| **Closing branch** | `feat/s-4-evm-chain-adapter` (PR pending) |

## Metrics delta

| Axis | Start | End | Δ |
|---|---|---|---|
| Tests | 4 | 15 | **+11** |
| Formal specs | 5 | 5 | 0 (unrelated; S-4 is adapter work) |
| CI tripwires | 7 | 7 | 0 |
| Frontmatter coverage | 100% | 100% | maintained |
| Federation drift constraints green | 10/11 | 11/11 (post-merge) | **+1** |
| Workspace crates | 1 | 2 | +1 (`nist-agent-chain`) |

The 11/11 line is the load-bearing milestone for the planset: every
declared federation cross-repo dependency is now honored by a
real Cargo.toml pin.

## What worked

- **Wrapping `AnchorRegistryClient` rather than re-implementing.**
  The runtime's client is already well-tested (5 unit tests in the
  archive). Wrapping it costs one struct field and ~10 lines of
  trait impl per method. Reimplementing would have meant
  duplicating the ABI work and the signing path.
- **Re-exporting `AnchorKind` from `citrate_agent_core::audit`**
  rather than re-defining it. Rule 9 ("one source of truth per
  topic") said this; doing it kept the cross-layer alignment with
  the Solidity contract enum order.
- **Same fixture key in our address-derivation test as runtime's.**
  This is the smoke that pins our `derive_address_hex` to runtime's
  `audit::recorder::derive_address`. If either side mutates, both
  fixtures fail simultaneously — the right failure mode.
- **`citrate-wallet-core` exposes `RpcClient` and `TransactionBuilder`
  as public-API primitives.** This let us implement the generic EVM
  client without adding `ethers-rs` or `alloy-rs` as a new dep.
  Saved ~80 transitive crates and a deny.toml review cycle.

## What didn't work

- **Reaching for the cargo-deny / clippy / fmt cycle to catch
  formatting issues before the first push.** Local `cargo fmt
  --all -- --check` would have caught the multi-line argument
  rewraps that rustfmt did automatically; instead, I had to run
  `cargo fmt --all` after the first build. Friction, not bug.
- **The `ContractAddresses::citrate_mainnet()` placeholder values.**
  Returning 0x01..0x05 padded addresses is honest (they're
  obviously placeholder) but not Rule-1-clean. The right answer:
  return `None` until the real addresses are in. Deferred to S-15
  because the real addresses don't exist yet.

## What surprised us

- **`citrate-wallet-core` 0.4.0's `RpcClient` and
  `TransactionBuilder` are exactly the surface we needed.** No
  bridging code, no shimming, no wrapping. The original defense_prime
  shell sprint (BFR-INT-12b) carved out this exact API months
  ago; we inherited the dividend.
- **The `trait ChainClient` settled at 5 methods on the first
  draft.** The S-1 backlog stub listed exactly these five and
  they were the right five — no method needed adding or removing
  after writing both impls. That's the value of authoring the
  Gherkin feature inventory before the trait.

## Lessons for future sprints

1. **When a runtime adapter is already shaped well, wrap; don't
   re-implement.** S-4's whole work fit in one day because we
   didn't redesign the wire format. Future sprints touching
   runtime's adapters should do the same.
2. **`workspace.dependencies` lookup makes drift constraints
   trivial.** When a federation `[[drift]]` entry says "this crate
   uses citrate-X at version Y", landing it is one line in
   `[workspace.dependencies]` and one line in the consumer's
   `Cargo.toml`. Federation Rule 12 is cheap to honor at this
   shape.
3. **Defer `unimplemented!()` traps by returning a typed error
   variant** instead. `ChainError::ZeroAddress(name)` keeps us
   honest about the configuration boundary without using `panic!`
   in production paths.

## Open follow-ups

1. Replace `ContractAddresses::citrate_mainnet()` placeholders
   with real Citrate Mainnet v1 contract addresses (blocked on
   citrate-chain v1 deployment; tracked at S-15).
2. Add `.github/workflows/evm-adapter-integration.yml` running
   Anvil + the trait against mock contracts (blocked on S-6
   PolicyBundle parsing for end-to-end selection).
3. Consider lifting `derive_address` shared code into runtime's
   public surface so we delete our copy. Not S-4 scope.

## Next sprint(s)

- **S-5 (Agent loop + Model resolver)** unblocked.
- **S-6 (PolicyBundle + data-class lattice)** unblocked; the
  bundle will define how operators select between
  `CitrateChainClient` and `GenericEvmChainClient` at startup.
