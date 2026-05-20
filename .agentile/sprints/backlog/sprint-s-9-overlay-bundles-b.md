---
created: 2026-05-20T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: backlog
sprint: S-9
---

# Sprint S-9: Overlay bundles B — HIPAA / HITECH + FedRAMP High + WORM/NFS-S3 sinks

**Goal.** Author signed PolicyBundles for HIPAA/HITECH and FedRAMP
High; land the WORM and NFS/S3 `AuditSink` backends; enroll
PIV-CAC-only signing path; gate Mobile signing per overlay.

**Why now.** Healthcare + federal are the Phase-1 pilot targets
adjacent to Phase 1's later sprints. Storage backends are required
by FedRAMP High.

**Predecessors.** S-6, S-8.

**Features owned.**
- `features/overlays/overlay-hipaa.feature`
- `features/overlays/overlay-fedramp-high.feature`
- `features/core/audit-storage-backends.feature` (WORM, NFS/S3)

**Cross-repo.** Upstream PR to runtime for the new `AuditSink` impls.

**Exit criteria.**
- Two signed PolicyBundles published.
- WORM and NFS/S3 sinks merged upstream.
- PIV-CAC enrollment flow exercised end-to-end on the daemon and Slint.
- Reference deployments pass `doctor` for each overlay.
- Runbooks landed at `docs/compliance/{hipaa,fedramp-high}/RUNBOOK.md`.
