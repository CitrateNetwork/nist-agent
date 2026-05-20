---
created: 2026-05-20T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: backlog
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
