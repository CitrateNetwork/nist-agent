---
created: 2026-05-20T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: backlog
sprint: S-4
---

# Sprint S-4: EVM chain adapter trait

**Goal.** Define `trait ChainClient` that abstracts over Citrate L1
and any compatible EVM chain; implement the default Citrate impl
(thin wrapper over `citrate-agent-core`'s `AnchorRegistryClient`) and
a generic EVM impl reachable via operator-supplied RPC + contract
addresses. This is the sidecar value-add over `citrate-agent-runtime`.

**Why now.** The product proposition is "composable, distributable,
chain-agnostic." Without S-4 we ship a Citrate-only product.

**Predecessors.** S-3.

**Features owned.**
- `features/chain/chain-evm-adapter.feature`
- `features/chain/chain-privacy-guarantees.feature` (extended to verify across adapters)

**Cross-repo.** Likely upstream `trait ChainClient` itself to runtime
once the surface settles. The generic EVM impl stays in nist-agent.

**Exit criteria.**
- `trait ChainClient` with methods: `anchor`, `is_anchored`,
  `chain_id`, `read_clearance`, `read_capsule_registry`.
- Default Citrate impl: thin wrapper over runtime.
- Generic EVM impl: `ethers-rs` based, contract addresses from
  PolicyBundle.
- Integration test against a local Anvil with mock contracts.
- Adapter selection is auditable per `chain-evm-adapter.feature`.
- ADR landed at `.agentile/adrs/2026-XX-XX-evm-adapter-trait.md`.
