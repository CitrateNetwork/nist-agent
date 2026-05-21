---
created: 2026-05-21T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: backlog
sprint: S-13a
---

# Sprint S-13a: TOB engagement tracking + per-finding remediation

**Goal.** Track the live Trail of Bits engagement against
nist-agent v1.0-rc (audit boundary `30044d2`, S-12 close).
Triage each finding as it arrives, open a per-finding
remediation branch + PR, obtain TOB sign-off. Close to "no
Tier-1 findings outstanding."

**Why now.** v1.0 tag is gated on this closing per the
Phase-1 exit criteria (planset ROADMAP exit #5). S-13 closed
the packet phase; S-13a is the receiving / remediation phase.

**Predecessors.** S-13 (packet delivery + ADR-012).

**Cross-repo.** Findings may originate cross-repo (citrate-
agent-runtime, citrate-chain). Per `docs/audit/CONTACT.md`'s
coordinated-disclosure clause, those land in the upstream
project's security inbox via the engagement lead.

**Engagement window.** Variable. Starts on the day TOB
acknowledges the packet (per the kickoff checklist in
`docs/audit/CONTACT.md`); ends when TOB delivers the signed
report **and** every Tier-1 finding has Citrate-side
remediation merged + TOB sign-off.

**Activity (per finding as it lands):**

1. TOB delivers finding write-up.
2. Triage per `.agentile/audits/findings/RUBRIC.md`:
   - Tier-1: same-day ack, 14-day remediation target.
   - Tier-2: 2-BD ack, 60-day target.
   - Tier-3: 5-BD ack, best-effort.
3. Drop a per-finding file under
   `.agentile/audits/findings/<id>-<slug>.md` using
   `TEMPLATE.md`.
4. Open `fix/tob-<id>-<slug>` branch, land the code change +
   pinning test, PR, admin-merge.
5. Update the per-finding file with PR + merge commit + test
   id.
6. Request TOB sign-off; record in the per-finding file.
7. Update `.agentile/audits/findings/INDEX.md`.

**Cross-repo.** Tier-1 findings may need federation-side
coordination (e.g., upstream PR in `citrate-agent-runtime`
+ federation manifest rev bump). The federation sprint shape
applies in that case.

**Exit criteria.**
- TOB final signed report delivered.
- All Tier-1 findings have remediation PRs merged + TOB
  sign-off.
- Per-finding files in `.agentile/audits/findings/` have
  status `signed-off` for every Tier-1.
- Final signed report archived to
  `citrate-agentile-archive/audits/<delivery-date>-tob-nist-agent-v1/`.
- `.agentile/audits/findings/INDEX.md` reflects the final
  state (sorted by tier; counts per tier in the header).
- Federation manifest pin for nist-agent updated to capture
  the post-remediation state (commit-rev today; `v1.0.0-rc`
  tag once the other v1.0 prerequisites also close).

**Carry to v1.0-tag sprint.**
- v1.0 PolicyBundle signing ceremony (SecurityOfficer +
  ComplianceOfficer co-sign).
- Federation pin update from commit-rev to `v1.0.0-rc` tag.

**Out of scope.**
- Tier-2 / Tier-3 findings may remediate post-v1.0 if
  appropriately tracked per RUBRIC.md SLAs.
- S-12b CI infrastructure (parallel track, independent of
  this sprint).
- S-14 pilot onboarding (parallel track).
