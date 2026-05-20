---
created: 2026-05-20T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: backlog
sprint: S-10
---

# Sprint S-10: Slint concierge + HITL UI + capsule inspector + marketplace

**Goal.** Build the Slint operator app per RFC §8: first-run
concierge with bundled Gemma 4 E2B, HITL approval queue UI,
Capsule Inspector (the AC-6 audit artifact for installs), chat
surface, and marketplace browser.

**Why now.** Operator onboarding is the Phase-2 pilot gate. Without
the Slint app, onboarding falls back to CLI editing of TOML, which
isn't an operator-grade experience.

**Predecessors.** S-3, S-5 (model resolver), S-6 (policy), S-8 (one overlay live).

**Features owned.**
- `features/surfaces/surface-slint-concierge.feature`
- `features/surfaces/surface-slint-capsule-inspector.feature`

**Cross-repo.** None unless we factor a shared widget crate.

**Licensing.** Slint GPLv3 default; commercial license needed for
FedRAMP package. Track at S-12.

**Exit criteria.**
- Concierge captures org identity, role assignments, hardware key enrollment, policy bundle selection, overlay activation, all with HITL-gated AuditRecords per step.
- Capsule Inspector renders every field per `features/surfaces/surface-slint-capsule-inspector.feature`.
- Install button disabled until all fields scrolled into view.
- Smoke pass on Linux + macOS; Windows track in S-12.
