---
created: 2026-05-20T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: backlog
sprint: S-8
---

# Sprint S-8: Overlay bundles A — CMMC L3 + FERPA + COPPA + CIPA

**Goal.** Author signed PolicyBundles for the first four overlays
(CMMC L3 baseline, FERPA, COPPA, CIPA) and the supporting capsules
(redact-and-attest, parental-consent verifier, CIPA content filter).
Deploy reference doctor runs to prove each passes.

**Why now.** K-12 + DIB are the primary Phase-1 pilot targets.

**Predecessors.** S-6 (PolicyBundle schema), S-2 (specs).

**Features owned.**
- `features/overlays/overlay-cmmc-l3-baseline.feature`
- `features/overlays/overlay-ferpa.feature`
- `features/overlays/overlay-coppa.feature`
- `features/overlays/overlay-cipa.feature`
- `features/core/audit-retention.feature` (CMMC + FERPA rows)

**Cross-repo.** None expected; bundles + capsules live here.

**Exit criteria.**
- Four signed PolicyBundles published to `.agentile/policy/`.
- Three new capsules signed at bundled tier.
- Reference deployments pass `doctor` on a clean install for each overlay.
- Compliance runbooks landed for each at `docs/compliance/<overlay>/RUNBOOK.md`.
