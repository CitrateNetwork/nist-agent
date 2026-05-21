---
created: 2026-05-21T00:00:00Z
branch: feat/s-9-overlay-bundles-b
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: completed
sprint: S-9
closed: 2026-05-21T00:00:00Z
---

# Sprint S-9: Overlay bundles B (HIPAA + FedRAMP-High + WORM audit sink)

## Sprint Metadata

| Field | Value |
|---|---|
| **Sprint ID** | `S-9` |
| **Sprint Name** | Overlay bundles B — HIPAA / HITECH + FedRAMP-High factories + WORM filesystem audit sink |
| **Goal** | Close the remaining two v1.0 overlay factories (HIPAA, FedRAMP-High) + ship a real WORM audit-storage backend so the FedRAMP-High overlay's `WORM storage mandatory` scenario passes. NFS / S3 sinks defer to S-9b. |
| **Branch** | `feat/s-9-overlay-bundles-b` |
| **Start Date** | 2026-05-21 |
| **End Date** | 2026-05-21 |
| **Status** | `COMPLETE` |
| **Planset** | [`../../planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md`](../../planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md) |
| **Predecessors** | S-6 (PolicyBundle), S-8 (overlay factory pattern) |

## Why this sprint

S-8 closed 4 of 6 v1.0 overlays. S-9 closes the remaining 2 plus
the WORM audit storage backend FedRAMP-High mandates.

After S-9, all six v1.0 overlays have typed signed PolicyBundle
factories and a real audit-sink choice satisfying each
overlay's storage requirements.

## Deliverables

- `crates/nist-agent-overlays/src/profiles.rs` — extended with
  `hipaa(opts)` and `fedramp_high(opts)` factories.
- `crates/nist-agent-audit-sinks/` — new workspace member.
  - `WormFilesystemSink` — `O_CREAT | O_EXCL` enforced WORM,
    0o400 read-only mode after close, sequence-ordered `iter()`.
  - Re-exports upstream's `AuditSink` trait.
- `docs/compliance/{hipaa,fedramp-high}/RUNBOOK.md`.
- ADR-007.
- Test count: 93 → 101 (+8).

## Test Baseline (start of sprint)

| Metric | Count | Captured |
|---|---|---|
| Tests | 93 | 2026-05-21 |
| Formal specs | 5 | 2026-05-21 |
| CI tripwires | 7 | 2026-05-21 |
| Frontmatter coverage | 100% | 2026-05-21 |
| Drift constraints | 11/11 green | 2026-05-21 |
| Workspace crates | 7 | 2026-05-21 |
| Overlay factories | 4 | 2026-05-21 |
| Compliance runbooks | 4 | 2026-05-21 |

## Method

Factories follow S-8's pattern exactly. The WORM sink uses
`OpenOptions::create_new(true)` (i.e. `O_CREAT | O_EXCL`) for
syscall-level write-once. ADR-007 records why this is sufficient
for FedRAMP-High when paired with POSIX ACLs + confinement.

## Work Packages

### WP-9.1 — HIPAA factory (DONE)
### WP-9.2 — FedRAMP-High factory (DONE)
### WP-9.3 — WormFilesystemSink (DONE) — 5 tests
### WP-9.4 — Two runbooks (DONE)
### WP-9.5 — ADR-007 (DONE)
### WP-9.6 — NFS sink (DEFERRED to S-9b)
### WP-9.7 — S3 sink (DEFERRED to S-9b)

## Daily updates

- 2026-05-21 — Kickoff and close in one session. 8 new tests
  bring the workspace to 101. fmt, clippy, deny clean.

## Exit criteria

- [x] `hipaa()` + `fedramp_high()` factories land
- [x] Both round-trip through canonical CBOR + Ed25519
- [x] FedRAMP-High pins `forbids_mobile_signing() == true`
- [x] `WormFilesystemSink` implements `AuditSink` end-to-end
- [x] WORM-violation test passes
- [x] Two new runbooks
- [x] ADR-007 landed
- [x] Test ratchet 93 → 101
- [x] All overlay factories pass doctor checks
- [ ] NFS + S3 sinks (DEFERRED)

## Close note (2026-05-21)

All six v1.0 overlays now have typed signed PolicyBundle
factories backed by a real audit-sink choice.

| Overlay | Factory | Runbook | Audit sink |
|---|---|---|---|
| CMMC-L3 baseline | ✅ S-8 | ✅ S-8 | filesystem (upstream) |
| FERPA | ✅ S-8 | ✅ S-8 | filesystem |
| COPPA | ✅ S-8 | ✅ S-8 | filesystem |
| CIPA | ✅ S-8 | ✅ S-8 | filesystem |
| HIPAA / HITECH | ✅ S-9 | ✅ S-9 | filesystem |
| FedRAMP-High | ✅ S-9 | ✅ S-9 | **WORM (S-9)** |

**Next sprint.** S-10 (Slint concierge).
