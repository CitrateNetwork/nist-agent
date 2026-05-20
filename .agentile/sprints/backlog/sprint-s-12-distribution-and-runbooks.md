---
created: 2026-05-20T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: backlog
sprint: S-12
---

# Sprint S-12: Distribution and runbooks

**Goal.** Make every artifact byte-reproducible, every release HSM-
signed, the bundled GGUF separately hashable, the air-gap install
working from a single signed bundle, and Phase-1 overlay runbooks
landed.

**Why now.** Until S-12 closes, "we ship a sidecar" is aspirational.
Operators need an installer they can verify offline.

**Predecessors.** S-3, S-5 (model bundling), S-8 + S-9 (overlay
bundles to ship), S-10 (Slint to ship), S-11 (mobile to ship).

**Features owned.**
- `features/distribution/dist-reproducible-builds.feature`
- `features/distribution/dist-signed-releases.feature`
- `features/distribution/dist-bundled-gemma4.feature`
- `features/distribution/dist-airgap-install.feature`
- `features/distribution/dist-overlay-runbooks.feature`
- `features/surfaces/surface-cli.feature`
- `features/surfaces/surface-daemon.feature`
- `features/surfaces/surface-wasm-publish.feature`

**Cross-repo.** None unless the GGUF artifact moves to a separate
release repo (revisit at sprint kickoff).

**Exit criteria.**
- Two-machine reproducibility test passes.
- Release pipeline produces signed bundle + SBOM on tag push.
- Installer verifies release signature; refuses on failure.
- Doctor verifies running daemon hash against release manifest.
- Six overlay runbooks landed under `docs/compliance/<overlay>/RUNBOOK.md`.
- Air-gap install completes end-to-end on a host with no network.
