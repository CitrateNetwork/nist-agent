---
created: 2026-05-21T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: backlog
sprint: S-10a
---

# Sprint S-10a: Slint concierge — first-run setup wizard

**Goal.** Land the Slint app shell + the bundled-model-driven
first-run concierge wizard per RFC §8.1 + §8.2. Operator opens
the binary, the concierge guides through org identity, role
assignment, hardware-key enrollment, overlay selection, and
PolicyBundle authoring. The result is a signed bundle written
to the operator's configured storage path.

**Why now.** Operator onboarding is the Phase-2 pilot gate. The
wizard is the smallest standalone slice that delivers
operator-visible value — without HITL queue UI or marketplace,
operators can still complete first-run setup.

**Predecessors.** S-3 (workspace), S-5 (model resolver — Ollama
+ embedded GGUF path), S-6 (PolicyBundle), S-8/S-9 (overlay
factories — wizard picks one).

**Features owned.**
- `features/surfaces/surface-slint-concierge.feature`

**Cross-repo.** None.

**Licensing.** Slint GPLv3 default; commercial license needed
for FedRAMP package, tracked at S-12.

**Exit criteria.**
- New `crates/nist-agent-slint` crate with the app shell.
- Concierge wizard panes: org identity → five-role enrollment →
  hardware-key enrollment → overlay selection (one of the six
  factories from S-8/S-9) → review → sign + write bundle.
- Gemma 4 E2B drives the conversation copy (resolved via S-5's
  `Model::resolve()` — Ollama localhost first, embedded GGUF if
  S-5b's llama-cpp-2 path lands).
- Each completed step writes an AuditRecord (typed event:
  `OrgIdentitySet`, `RoleAssigned`, `HardwareKeyEnrolled`,
  `OverlayActivated`, `PolicyBundleSigned`).
- "Finish" disabled until all five base roles assigned (mirrors
  `PolicyBundle.activate()`'s role-completeness check).
- Smoke pass on Linux; macOS in S-12 (Slint runs on both but the
  binary build infrastructure for macOS lives in distribution).

**Deferred to S-10b / S-10c:**
- HITL approval queue UI
- Capsule Inspector pane
- Marketplace browser
- Chat surface (live agent interaction)
