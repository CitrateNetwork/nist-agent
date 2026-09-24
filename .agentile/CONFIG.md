---
created: 2026-05-20T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
---

# CONFIG.md — Canonical Project Constants

> **Source of truth** for every immutable constant in this project
> (versioned IDs, default ports, environment names, well-known
> addresses, etc.). Other docs may reference these values; they may
> not redefine them. Changing a value here is a planset-grade event
> (see Stability guarantees, below).

## Project identity

| Field | Value |
|---|---|
| Project name | `nist-agent` |
| Product code | `cit-agent` (per RFC-CIT-AGENT-0001) |
| One-line description | NIST-compliant agent harness — composable sidecar for Citrate Network and EVM chains |
| Repository | `https://github.com/CitrateNetwork/nist-agent` (TBC at publish) |
| License | `Apache-2.0` (open-core; commercial license available for FedRAMP package per RFC §8.4) |
| Initial author | Saul Loveman (Citrate Inc.) + Claude (Anthropic) |
| Federation tier | T1 (full external audit required before v1.0.0) |

## Compliance baselines

| Baseline | Status | Source |
|---|---|---|
| NIST SP 800-171 Rev 3 | **baseline** | RFC §2.1 |
| CMMC Level 3 (32 CFR § 170.14(c)(4)) | **baseline** | RFC §2.1 |
| FERPA | overlay | RFC §2.3 |
| COPPA | overlay | RFC §2.3 |
| CIPA | overlay | RFC §2.3 |
| HIPAA / HITECH | overlay | RFC §2.3 |
| FedRAMP High | overlay | RFC §2.3 |
| DoD IL4 / IL5 | overlay (v1.1) | RFC §11.2 |
| ITAR | overlay (v1.1) | RFC §11.2 |
| CJIS | overlay (v1.1) | RFC §11.2 |
| IRS 1075 | overlay (v1.1) | RFC §11.2 |

## Languages and toolchains

| Language | Toolchain | Test command (canonical) |
|---|---|---|
| `rust` | Rust stable pinned via `rust-toolchain.toml` (to be added in Sprint S-4) | `cargo test --workspace 2>&1 \| grep 'test result:' \| awk '{s+=$4}END{print s}'` |
| `tla` | TLA+ Toolbox / `tla2tools.jar` v1.8.0+ | `scripts/ci/check_spec_ratchet.py` |
| `gherkin` | `cucumber-rs` (consumer of features/) | counted via `scripts/ci/check_test_ratchet.py` once wired |

The `tests.command` field in `.agentile/coverage/baseline.json` is the
authoritative invocation; the row above documents it for humans.

## Default network posture

| Setting | Default | Source |
|---|---|---|
| Egress | **disabled** (air-gapped) | RFC §3.3, G1 |
| Egress override | requires signed Security Officer policy directive | RFC §3.3 |
| Model resolution order | local Ollama (11434) → llama.cpp (8080/8000) → embedded GGUF → site mirror → on-chain registry (only with egress) → external (only with explicit policy) | RFC §3.3, §8.2 |
| Capsule install source order | bundled → local site mirror → CapsuleRegistry (only with egress) | RFC §3.3 |

## Environments

| Environment | Purpose | RPC / endpoint | Chain ID | Stable since |
|---|---|---|---|---|
| `local` | Developer machine, no chain | `127.0.0.1:11434` (Ollama) | n/a | Day 0 |
| `citrate-mainnet` | Production anchor target | (per `citrate-federation/manifest.toml`) | `40204` | Per RFC §7.1 |
| `citrate-testnet` | Pre-prod anchor target | TBC | TBC | TBC |
| `evm-adapter-test` | Generic EVM adapter integration tests | local Anvil / Hardhat | configurable | Phase 1 |

## Versioned constants

| Name | Value | Why it cannot change without migration |
|---|---|---|
| Citrate L1 chain ID | `40204` | Pinned in five on-chain contracts; changing it means redeployment. |
| Capsule archive extension | `.cps` | Manifest-driven loader matches on this; any change breaks all existing capsules. |
| Bundled concierge model | `gemma-4-e2b-it-Q4_K_M.gguf` | Hash-pinned per NIST SI-7; changing model requires fresh hash + reproducible-build evidence. |
| Five-role lattice | `Operator, Reviewer, ComplianceOfficer, SecurityOfficer, Auditor` | Quorum math + TLA+ specs depend on this exact tuple. |
| Three signing tiers | `bundled, managed, workspace` | CapsuleRegistry / capsule loader / doctor checks all switch on tier. |
| Three anchor strategies | `PerCapsule, PerApproval, NightlyMerkle` | TLA+ spec `AuditChainIntegrity.tla` enumerates exactly these. |
| Frontmatter spec | `created, branch, author, status` (Rule-12) | Every CI ratchet relies on this shape. |

## Stability guarantees

Anything in this file is on the **stable surface**. Changing a value
requires:

1. A versioned migration document under `.agentile/planset/<new-frame>/MIGRATION.md`.
2. A sprint that lands the migration with a rollback plan, signed off by Compliance Officer + Security Officer.
3. A note in `CHANGELOG.md` flagging the change as breaking (or not).
4. If the change crosses repos (e.g. Citrate chain ID): a federation sprint under `citrate-federation/agentile/sprints/active/`.

If you find yourself wanting to change a value here mid-sprint, that's
a signal to file a planset, not to edit this file silently.
