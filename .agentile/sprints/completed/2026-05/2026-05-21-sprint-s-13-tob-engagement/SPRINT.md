---
created: 2026-05-20T00:00:00Z
branch: feat/s-13-tob-engagement
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: complete
sprint: S-13
---

# Sprint S-13: Trail of Bits engagement preparation + remediation

**Goal.** Engage Trail of Bits, deliver the audit packet, receive
findings, and close to "no Tier-1 findings outstanding."

**Why now.** v1.0 release is contingent on a closed external audit
per RFC §11.1.

**Predecessors.** S-2 → S-12 all closed.

**Features owned.** none directly — this sprint runs the gauntlet
against the *full* feature inventory.

**Cross-repo.** Audit findings may originate cross-repo (runtime,
chain). Federation sprint required if so.

**Exit criteria.**
- Audit packet delivered (scope, artifacts, deployment instructions, threat model).
- Findings received and triaged.
- Tier-1 findings remediated and signed off by Trail of Bits.
- Audit report archived to `citrate-agentile-archive/audits/<date>-tob-nist-agent-v1/`.
- PolicyBundle for v1.0 release signed by SecurityOfficer + ComplianceOfficer.
- Sprint closes with the v1.0 tag candidate.

## Close note — 2026-05-21

Status: **COMPLETE** (packet phase). Engagement runs externally
per ADR-012; remediation phase tracked as S-13a.

ADR-012 codifies the explicit phase split. This sprint lands
the **packet** that bootstraps the TOB engagement; the actual
engagement runs on TOB's schedule with their own
infrastructure; findings + remediation become S-13a (per-
finding remediation sprints) once findings arrive.

**Delivered.**
- `docs/audit/INDEX.md` — packet cover sheet + reading order.
- `docs/audit/SCOPE.md` — what's in/out of scope; full crate
  inventory (13 crates / ~9.6 kLOC Rust) + line counts + dep
  lineage + known sharp edges.
- `docs/audit/THREAT_MODEL.md` — actor/asset matrix + STRIDE
  per load-bearing surface (HITL quorum, overlay ratchet,
  mobile pairing, WORM audit, release verifier, egress
  posture, GGUF SI-7, agent loop). Cross-referenced to formal
  specs + feature scenarios.
- `docs/audit/BUILD.md` — toolchain pins, build commands,
  formal-spec verification, reproducibility status, test
  surface map (12 invariants → test locations).
- `docs/audit/DEPLOYMENT.md` — reference deployment topology +
  bring-up + per-surface exercise commands.
- `docs/audit/CONTACT.md` — PGP/Signal channels, response SLAs
  (same-day Tier-1, 2-BD Tier-2, 5-BD Tier-3), embargo policy,
  kickoff checklist.
- `docs/audit/V1_READINESS.md` — Phase-1 exit criteria cross-
  check (4/6 ✅, 2/6 🟡; gates to v1.0 tag enumerated).
- `.agentile/audits/findings/RUBRIC.md` — Tier classification
  rubric + per-tier remediation SLAs.
- `.agentile/audits/findings/TEMPLATE.md` — per-finding file
  shape with frontmatter, surface mapping, reproducer,
  remediation tracker, sign-off, disclosure timeline.
- `.agentile/audits/findings/INDEX.md` — index of findings
  (currently empty; engagement not yet kicked off).
- `.agentile/adrs/ADR-012-tob-packet-here-engagement-external.md`
  — placement decision.

**Metrics.**
- Workspace crates: 13 → 13 (docs-only sprint).
- `cargo test --workspace`: 202 → 202 (Rule 3 satisfied:
  monotone non-decreasing).
- ADRs: 11 → 12 (added ADR-012).
- Audit-packet docs: 0 → 6 (under `docs/audit/`).
- Findings-tracker docs: 0 → 3 (under `.agentile/audits/findings/`).
- Phase-1 exit criteria status: 4/6 ✅, 2/6 🟡 (TOB engagement
  + pilot onboarding).

**Exit criteria — final state.**
- ✅ Audit packet delivered (six documents under `docs/audit/`).
- ⏸ Findings received and triaged — **DEFERRED to S-13a**
  (engagement not yet kicked off).
- ⏸ Tier-1 findings remediated and signed off — **DEFERRED
  to S-13a** (gates v1.0 tag).
- ⏸ Audit report archived — **DEFERRED to S-13a** (post-
  delivery; lands in
  `citrate-agentile-archive/audits/<date>-tob-nist-agent-v1/`).
- ⏸ v1.0 PolicyBundle signed — **DEFERRED** (gated on TOB
  sign-off + S-12b CI completion).
- ⏸ v1.0 tag — **DEFERRED to v1.0-tag sprint** (Phase-1 exit
  criteria 4 + 5 + 6 must close first).

**Carried into S-13a / S-14.**
- TOB kickoff: PGP key exchange, deploy key issuance, first
  sync call. Tracked in [`CONTACT.md`](../../../docs/audit/CONTACT.md)'s kickoff checklist.
- Per-finding remediation sprints — open as findings land,
  one branch + PR per Tier-1 finding.
- Pilot onboarding for at least one operator per Phase-1
  overlay class (S-14).
- S-12b: CI workflow + HSM ceremony (parallel track,
  independent of TOB findings).

**v1.0 tag prerequisites** (cross-referenced in
[`V1_READINESS.md`](../../../docs/audit/V1_READINESS.md)):
1. TOB engagement closes with no Tier-1 outstanding.
2. S-12b delivers signed reproducible bundle via CI.
3. S-14 closes at least one pilot per overlay.
4. Federation pin updates to `v1.0.0-rc` tag.
