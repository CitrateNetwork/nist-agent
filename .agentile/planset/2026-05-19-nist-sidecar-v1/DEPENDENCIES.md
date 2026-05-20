---
created: 2026-05-19T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
planset: nist-sidecar-v1
---

# Dependencies — nist-sidecar-v1

> What this planset consumes, what (if anything) consumes it, and
> what changes in the surrounding system when this planset closes.
> Pair-read with `citrate-federation/manifest.toml` — that file is
> the canonical SHA-pin source; this file documents the *shape* and
> *risk* of each dependency.

## Upstream — what nist-agent consumes

| Dependency | Kind | Pin | Risk / mitigation |
|---|---|---|---|
| `citrate-agent-runtime` (`citrate_agent_core` crate) | Cargo dep on federation-pinned rev | TBC at S-3 | **High coupling.** This is the engine. Mitigation: federation `drift-check.yml` catches rev drift; ALIGNMENT.md governs ownership boundary; PRs back to runtime are the primary contribution path. |
| `citrate-chain` (Solidity ABIs for `OrganizationSBT`, `AgentSBT`, `CapsuleRegistry`, `AnchorRegistry`, `BenchmarkRegistry`) | ABI artifact import | manifest-pinned | **Medium.** Contract upgrades are 2-of-3 timelocked (RFC §7.1) so ABI breakage is preceded by federation announcement. |
| `citrate-federation` (manifest, agentile rules, audit cadence) | Methodology / pin source | rolling main | **Low.** We mirror rules locally for offline reading; federation wins on drift (Rule 11). |
| Rust stable toolchain | Toolchain | pinned at S-3 via `rust-toolchain.toml` | **Low.** Standard. |
| `wasmtime` (Component Model linker) | Cargo dep (transitive via runtime) | inherits runtime pin (26.x as of 2026-05) | **Medium.** Capsule capability enforcement depends on linker behavior; spec compat reviewed at each wasmtime major. |
| `cucumber-rs` | Cargo dev-dep | latest 0.x at S-1 | **Low.** Test-time only. |
| `tla2tools.jar` | External tool | 1.8.0 (S-2 sets pin) | **Low.** Test-time only; CI installs at job-start. |
| `Slint` (UI toolkit) | Cargo dep at S-10 | latest 1.x at S-10 | **Medium-license.** GPLv3 / Royalty-Free / Commercial triple — the FedRAMP package needs commercial license. RFC §8.4. |
| `llama-cpp-4` + Gemma 4 E2B GGUF | Cargo dep + asset | model hash pinned in CONFIG.md | **Medium-supply-chain.** GGUF is a separately-hashable artifact (NIST SI-7); doctor verifies hash at startup. |
| `ed25519-dalek`, `serde_cbor`, `sha2` | Cargo crypto deps | inherits runtime pins | **High-crypto.** Verify FIPS 140-3 posture during S-13 (Trail of Bits prep). |
| `gh` / GitHub Actions runners | CI | latest stable | **Low.** Tripwire checks all run as scripts, portable. |

## Downstream — who consumes nist-agent

| Consumer | Kind | Status |
|---|---|---|
| (Future) operator deployments on Citrate Mainnet | Binary install | First pilot per overlay class in Phase 2. |
| (Future) third-party EVM deployments | Binary install | Enabled by S-4 (`trait ChainClient`). |
| (Future) `citrate-boeing-shell` or other federation agent surfaces | Cargo dep on nist-agent crates | Subject to federation manifest pinning once we publish a crate. |
| (Future) the `citrate-agent-runtime` itself | PRs back upstream | nist-agent's feature crates that get upstreamed (per ALIGNMENT.md exception clause) ultimately *land* in runtime; runtime then becomes a backwards consumer in the sense that it gains nist-agent-authored work. |

## Cross-repo work that crosses the federation line

Any sprint that touches a non-nist-agent repo must:

1. Open a federation sprint under
   `citrate-federation/agentile/sprints/active/<slug>.md` (per
   `AGENTILE_WORKFLOW.md` lines 156–171 from Citrate-Labs root).
2. Cite the federation sprint in this repo's sprint file.
3. Bump `citrate-federation/manifest.toml` if a SHA pin changes.
4. Run `./scripts/pin-bump.sh nist-agent` from inside
   `citrate-federation/` to propagate.
5. Pass `drift-check.yml`.

Sprints in this planset known to need a federation companion:

- **S-3** — opens manifest entry for `nist-agent`.
- **S-5** — PRs Agent loop + Model resolver to `citrate-agent-runtime`.
- **S-6** — PRs Policy bundle to runtime.
- **S-7** — PRs doctor checks 6–11 to runtime.
- **S-15** — Mainnet anchor first run requires AnchorRegistry policy
  sign-off in `citrate-chain`.

## What changes when this planset closes

- A `v1.0.0` tag exists on `nist-agent`.
- Federation manifest pins `nist-agent` at that tag.
- `citrate-agent-runtime` has merged the upstream PRs for Agent loop,
  Policy, Model resolver, and doctor checks 6–11.
- The five normative TLA+ specs are the canonical copy in
  `nist-agent/.agentile/formal/`; runtime references them.
- Phase-1 overlay bundles (CMMC L3, FERPA, COPPA, CIPA, HIPAA,
  HITECH, FedRAMP High) are signed and published.
- A Trail of Bits report exists in the archive.
- One pilot operator per overlay class has a signed deployment
  doctor report on chain.
