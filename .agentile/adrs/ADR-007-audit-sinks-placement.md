---
created: 2026-05-21T00:00:00Z
branch: feat/s-9-overlay-bundles-b
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: accepted
adr: 007
sprint: S-9
---

# ADR-007: WORM audit sink lands locally; NFS/S3 deferred to S-9b

| Field | Value |
|---|---|
| **ADR Number** | ADR-007 |
| **Date** | 2026-05-21 |
| **Status** | ACCEPTED |
| **Author** | Saul Loveman + Claude Opus 4.7 (1M context) |
| **Sprint** | S-9 |

## Context

RFC §6.2 names four audit storage backends as v1.0 deliverables:

1. **Local filesystem** — default; air-gap-friendly. Done upstream
   in `citrate_agent_core::audit::sink::FilesystemSink` (CIT-AGENT-5a).
2. **NFS / S3-compatible object store** — on-prem multi-host.
   Not yet implemented anywhere.
3. **WORM (write-once-read-many)** — FedRAMP-High + CJIS
   requirement. Not yet implemented anywhere.
4. **Citrate L1 chain anchors** — anchors only, never the full
   records. Done in `citrate_agent_core::audit::sink::ChainAnchorSink`
   (CIT-AGENT-6).

S-9 needs the WORM backend now (the FedRAMP-High overlay
mandates it; `features/overlays/overlay-fedramp-high.feature`
scenario "WORM storage backend is mandatory" pins the
activation-fails-without-WORM semantic). NFS/S3 is on the v1.0
roadmap but not on S-9's critical path — the harness can ship
the FedRAMP-High overlay with WORM-only storage.

## Decision

**We will create `crates/nist-agent-audit-sinks` containing the
WORM filesystem sink as a `citrate_agent_core::audit::AuditSink`
impl. NFS and S3 sinks defer to S-9b.** Fifth exercise of the
ALIGNMENT.md exception-clause pattern (after ADR-002/004/005/006).

### WORM implementation strategy

`WormFilesystemSink` stores one audit record per file, named by
the record's sequence number, zero-padded to 20 digits for
sort-stable directory listings (`00000000000000000042.cbor`).

Append calls `std::fs::OpenOptions::new().create_new(true)`,
which sets `O_CREAT | O_EXCL` on Unix and `CREATE_NEW` on
Windows. The OS-level effect: opening fails with `EEXIST` if
the path exists. A second write of the same sequence is therefore
rejected at the syscall, not at any Rust-level guard.

After write, the file is closed via `sync_all()` (durability)
and mode is set to `0o400` (owner read-only). Subsequent reads
go through `iter()`, which reads each file's CBOR contents in
sequence order.

### Stronger immutability is platform-specific

Linux `chattr +i` (immutable attribute) gives kernel-level
write-protection but requires root. macOS `chflags uchg` is
similar. Both are non-portable and not exposed through the
Rust standard library; consumers needing them can call them
out-of-band (e.g. a `chattr +i` invocation in the operator's
systemd unit) without changing the sink's API. This sink delivers
"harness-level WORM" — sufficient for FedRAMP-High when paired
with POSIX ACLs + AppArmor / SELinux confinement, which the
Trail of Bits engagement (S-13) will assess.

### Why NFS / S3 wait for S-9b

- **NFS** needs distributed-lock semantics across multiple
  harness instances writing to the same shared directory.
  Implementations vary (POSIX `fcntl(F_SETLK)`, NFSv4 byte-range
  locks, dotlock fallback); doing it right is a sprint of
  testing in itself.
- **S3** needs an HTTP client + AWS-SigV4 signer + retry
  semantics + multi-region awareness. `aws-sdk-rust` adds ~50
  transitive crates to the deny.toml review surface; worth
  doing once, deliberately, not bundled with S-9.

Both defer to S-9b. The PolicyBundle factory for FedRAMP-High
(`fedramp_high(opts)`) doesn't choose the sink; the harness's
start-up code selects from `WormFilesystemSink` today and
adds Nfs/S3 in S-9b.

## Consequences

**Positive.**

- FedRAMP-High overlay ships with a real WORM backend, not a
  stub. The Trail of Bits assessor (S-13) walks into a
  filesystem with one immutable file per record and `O_EXCL`-
  enforced write-once.
- The sink's `AuditSink` impl reuses upstream's trait directly
  (Rule 9). The harness composes our WORM sink + upstream's
  FilesystemSink + ChainAnchorSink in one `Vec<Arc<dyn AuditSink>>`
  for multi-sink writes.
- The `O_EXCL`-rejects-second-write test is the load-bearing
  property under FedRAMP §AU-9(5); regression in that property
  fails CI immediately.

**Negative.**

- NFS / S3 sinks land in S-9b, not S-9. The first FedRAMP-High
  deployment (S-14 pilot) is single-host until S-9b lands. For
  pilots this is acceptable; for prod, S-9b is prerequisite.
- The WORM-via-O_EXCL approach trusts the operator hasn't
  pre-created stub files at the target sequence numbers.
  Mitigation: the harness's first write enumerates the directory
  and refuses to start if non-sequence-shaped files are present.
  Tracked as a follow-up tightening; not done in S-9.

**Neutral.**

- Mode 0o400 (read-only after close) is best-effort; root or the
  owner can `chmod` it back to writable. The append step's
  O_EXCL is the real WORM lock; the mode bits are belt-and-
  suspenders.

## Alternatives considered

1. **`std::fs::OpenOptions` without `create_new`** + a Rust-level
   "does it exist?" check. Rejected: not atomic against concurrent
   writers. `O_EXCL` is the syscall-level atomic.
2. **`tempfile` + `rename`** (write to temp, atomically rename
   into place). Rejected: rename overwrites the destination
   silently — defeats WORM.
3. **Single growing append-only file with monotonic offsets**
   (like a WAL). Rejected: a partial write at process kill leaves
   the file in a torn state; recovery semantics are harder than
   one-file-per-record.
4. **Defer WORM to S-9b alongside NFS/S3.** Rejected: FedRAMP-
   High overlay's `WORM storage backend is mandatory` scenario
   wants a real implementation now. Without it, S-9 can't claim
   FedRAMP-High as deliverable.

## Reversal conditions

ADR-007 is reversed when:

- The upstream PR for `citrate_agent_core::audit::sink::worm`
  merges (lifts our impl into runtime).
- The federation manifest bumps the runtime rev to that commit.
- Our `Cargo.toml` re-pins `citrate-agent-core`.
- The 5 sink tests transfer to the upstream test runner.

A successor ADR records the deletion of `nist-agent-audit-sinks`'s
WORM module and the new `citrate_agent_core::audit::sink`
re-exports. NFS/S3 sinks land first in `nist-agent-audit-sinks`
per the same exception-clause pattern; their upstream PR is
their own follow-up.

## References

- RFC-CIT-AGENT-0001 §6.2 (storage backends), `features/overlays/overlay-fedramp-high.feature`.
- `citrate_agent_core::audit::sink::{AuditSink, FilesystemSink, ChainAnchorSink}` — the upstream trait + sibling impls.
- ADR-002/004/005/006 — the four prior exception-clause exercises.
- Federation rules 1 (no stubs in production paths — `O_EXCL` is real, not advisory), 9 (one source of truth — `AuditSink` trait stays upstream).
