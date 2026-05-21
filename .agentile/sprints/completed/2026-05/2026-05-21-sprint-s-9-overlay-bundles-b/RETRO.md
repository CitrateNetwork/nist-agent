---
created: 2026-05-21T00:00:00Z
branch: feat/s-9-overlay-bundles-b
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: completed
sprint: S-9
---

# Sprint S-9 — Retrospective

## Outcome

| Field | Value |
|---|---|
| **Goal achieved?** | YES (in-scope: 2 factories + WORM sink + 2 runbooks); NFS + S3 sinks deferred to S-9b |
| **WPs planned / closed** | 7 / 5 in scope; 2 deferred |
| **Carry-forward WPs** | WP-9.6 (NFS sink), WP-9.7 (S3 sink) |
| **Closing branch** | `feat/s-9-overlay-bundles-b` (PR pending) |

## Metrics delta

| Axis | Start | End | Δ |
|---|---|---|---|
| Tests | 93 | 101 | **+8** |
| Formal specs | 5 | 5 | 0 |
| CI tripwires | 7 | 7 | 0 |
| Frontmatter coverage | 100% | 100% | maintained |
| Drift constraints | 11/11 green | 11/11 green | 0 |
| Workspace crates | 7 | 8 | +1 (`nist-agent-audit-sinks`) |
| Overlay factories | 4/6 | **6/6** | +2 |
| Compliance runbooks | 4 | 6 | +2 |

The 6/6 line is the meaningful one — every v1.0 RFC §2.3 overlay
has a typed factory now.

## What worked

- **The S-8 factory pattern transferred verbatim.** Two new
  factories were ~5 lines of Rust each. The OverlayBuilder shape
  + the `cmmc_l3_baseline` base layer paid off again.
- **`O_CREAT | O_EXCL` as the WORM mechanism.** One Rust line
  (`OpenOptions::new().create_new(true)`) delivers syscall-level
  write-once. Cleaner than building a Rust-level guard that
  could race against concurrent writers. The
  `second_write_of_same_sequence_errors_worm` test catches the
  property directly.
- **Cross-layer pin for mobile signing.** The
  `fedramp_high_marks_mobile_signing_forbidden` test asserts
  that the bundle's overlay set carries a member whose
  `forbids_mobile_signing()` returns true. Catches drift
  between the prelude's predicate and the overlay factory.
- **Re-exporting `AuditSink` from the new crate.** Consumers
  can write `use nist_agent_audit_sinks::AuditSink` without
  knowing about citrate-agent-core. Rule 9 holds because the
  trait still has one canonical home upstream; we just provide
  a convenience re-export.

## What didn't work

- **First build failed on `AgentError` variant naming.** I
  guessed `AgentError::AuditSink(String)`; upstream actually
  uses `AgentError::Audit(String)`. One-line fix but a reminder
  to look before guessing. The runtime survey at S-5/S-6 was
  the canonical place to learn the variant set — I should have
  re-consulted it.
- **`AuditRecord.payload: Vec<u8>` not `ciborium::Value`.** I
  assumed `payload` was a structured CBOR Value because the
  doc comment talks about "event-specific payload as a CBOR
  value." Reading the actual struct definition would've saved
  a build cycle. Tracked as a general lesson: read the type,
  not the comment.

## What surprised us

- **`mode(0o400)` on `OpenOptions` is `os::unix::fs::OpenOptionsExt`.**
  The portable API caps at `mode(0o600)` because Windows doesn't
  understand unix bits. Our sink is unix-only at this layer;
  future cross-platform parity (if the harness ships a Windows
  daemon) would need a per-platform shim. RFC §3.2's surfaces
  list doesn't include Windows daemon explicitly — likely OK to
  stay unix-only here.
- **CMMC-L3's 6-year floor wins over FedRAMP-High's 3-year
  floor.** The `effective_floor` MAX semantic means FedRAMP-High
  deployments retain 6 years anyway (because CMMC-L3 is always
  active). The runbook notes this so operators don't
  inadvertently shorten retention.

## Lessons for future sprints

1. **Read the upstream type before guessing.** Both build errors
   in S-9 were avoidable by reading the actual definitions in
   the runtime crate before writing test code. Future sprints
   that touch new upstream APIs: grep first, write second.
2. **`O_EXCL`-shaped immutability is a small primitive with
   big audit weight.** Trail of Bits (S-13) will likely call out
   the syscall-level enforcement as evidence-grade. Worth
   reusing the same pattern for any future "this can't be
   modified" semantics (e.g. PolicyBundle activation history
   entries, anchor receipts).
3. **Six factories now form a complete overlay matrix.** The
   pattern is provably scalable; S-11 (mobile) and S-12
   (distribution) can author their per-overlay deltas
   confidently.

## Pending follow-ups (S-9b / future)

1. **NFS sink** — `fcntl(F_SETLK)` byte-range locks for
   multi-host serialization. Mid-sized; one focused sprint.
2. **S3 sink** — `aws-sdk-rust` adoption + deny.toml review for
   ~50 new transitive crates. Larger sprint.
3. **`chattr +i`** invocation hook so post-write kernel
   immutability is a one-line addition for operators who want
   it.
4. **Cross-platform mode bits** — if S-12's Windows distribution
   target lands, the sink's `mode(0o400)` needs a Windows
   equivalent (ACL with deny-write on owner).
5. **HIPAA capsules** — PHI redactor + minimum-necessary
   verifier. Land with the FERPA/COPPA/CIPA capsule sprint.

## Next sprint(s)

- **S-10 (Slint concierge)** is the next critical-path sprint —
  the operator-facing onboarding + chat surface. Multiple
  sessions of work; the design phase alone is non-trivial.
- **S-11 (mobile companion)** can run in parallel to S-10.
- **S-9b** (NFS + S3 sinks) can run in parallel; touches
  `nist-agent-audit-sinks` only.
