---
created: 2026-05-21T00:00:00Z
branch: feat/s-7-doctor-checks
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: accepted
adr: 006
sprint: S-7
---

# ADR-006: Doctor checks land locally; companion to upstream's CIT-AGENT-7a

| Field | Value |
|---|---|
| **ADR Number** | ADR-006 |
| **Date** | 2026-05-21 |
| **Status** | ACCEPTED |
| **Author** | Saul Loveman + Claude Opus 4.7 (1M context) |
| **Sprint** | S-7 |

## Context

RFC §10.2 enumerates 11 checks the doctor pre-flight MUST run.
`citrate-agent-runtime`'s CIT-AGENT-7a sprint shipped 5 checks
built on surfaces that existed at that time (audit chain, audit
file permissions, approval queue depth, pending break-glass,
runtime presence). Of those 5, three map cleanly to RFC §10.2's
numbered list (#4 audit chain, #10 approval queue, #11 break-glass);
the other two (audit-file-permissions and runtime-presence) are
useful bonuses but not on the RFC checklist.

Eight RFC §10.2 checks remain unimplemented:

| # | RFC name | Depends on | Land where |
|---|---|---|---|
| 1 | capsule-signatures-verify | runtime capsule loader (CIT-AGENT-3b done) | upstream when sequenced; we don't have a Doctor entry yet |
| 2 | manifest-wit-wasm-capability-match | runtime capsule loader | same |
| 3 | policy-bundle-signed-and-within-validity-window | `nist-agent-policy` (S-6) | **here** |
| 5 | approver-hardware-key-bindings-valid | mobile companion (S-11) + PIV verifier | S-11 |
| 6 | network-posture-matches-policy | `nist-agent-policy` + harness instrumentation | **here** |
| 7 | model-file-hash-matches-manifest | `nist-agent-model::verify_sha256` (S-5) | **here** |
| 8 | tla-specs-current-for-tier-high-capsules | `.agentile/formal/specs/` (S-2) + BenchmarkRegistry | **here (filesystem subset)** |
| 9 | role-lattice-fully-assigned | `nist-agent-policy::Role::ALL` (S-6) | **here** |

S-7 lands checks 3, 6, 7, 8, 9 — five that bind to surfaces
nist-agent already built. The other three (#1, #2, #5) wait for
surfaces that aren't ready yet.

## Decision

**We will create `crates/nist-agent-doctor` containing the five
new checks as `NistCheck` impls.** The crate is a companion to
upstream's `citrate_agent_core::doctor`, not a replacement: the
harness composes results from both halves into a single
`DoctorReport`. ADR-006 is the fourth exercise of the ALIGNMENT.md
exception-clause pattern (after ADR-002/004/005).

### Why not extend upstream's `Check` trait directly?

The `Check` trait in `citrate_agent_core::doctor::checks` takes a
`DoctorContext` that doesn't carry the policy bundle, the model
manifest, the egress observations, or the TLA spec directory. To
implement our five checks against that trait, we'd need to:

1. Extend `DoctorContext` (upstream change), OR
2. Wrap our context as one of `DoctorContext`'s fields (awkward
   shape since the upstream context is concretely typed), OR
3. Define our own `NistCheck` trait + `NistDoctorContext` and let
   the harness compose results from both.

Option 3 is the local landing. The upstream PR (S-7b) unifies the
two traits and contexts in one move — cleaner than landing the
extension upstream first and then writing our checks against an
in-flight trait.

### What the checks produce

Each `NistCheck` returns `citrate_agent_core::doctor::CheckResult`
(re-exported here). The `Severity` enum is shared. Format
compatibility: when the harness merges results from upstream's
checks and ours, the merged `DoctorReport` has a homogeneous
`Vec<CheckResult>`.

### Skip semantics

Every check has an `Option<_>` dependency in `NistDoctorContext`.
When the dependency is `None`, the check returns `Severity::Pass`
with `message = "skipped: <reason>"`. The exception is
**PolicyBundleValidityCheck + RoleLatticeCheck + NetworkPostureCheck**:
when `raw_bundle` is `Some` but `so_pubkey` is `None`, all three
return `Severity::Blocker` ("bundle present but no SO pubkey
configured"). The harness MUST configure the trust roots before
a bundle can be present in the context; the skip mechanism is
for "no bundle yet" only.

This mirrors upstream's skip semantics in CIT-AGENT-7a and lets
operators run a partial doctor pass during first-run setup
(before the bundle is signed).

## Consequences

**Positive.**

- Five of RFC §10.2's outstanding checks are now implemented;
  combined with upstream's 3 that map to the RFC list, the
  harness covers 8/11. Plus 2 upstream bonus checks (audit-file-
  perms, runtime-presence). Total: 10 health observations per
  doctor run today.
- Each check binds to one specific surface (S-2 specs dir, S-5
  hash verifier, S-6 policy crate). Future regression in any of
  those surfaces surfaces in the doctor report, not just in unit
  tests.
- `compute_overall` aggregates correctly per RFC §10.3 (any
  BLOCKER → BLOCKER; else any WARN → WARN; else PASS).
- 14 new tests covering skip paths, pass paths, and at least one
  blocker path per check.

**Negative.**

- Three checks (#1, #2, #5) stay unimplemented. The harness can
  still run, but a Trail of Bits assessor will note the missing
  evidence. Tracked in S-7b RETRO: capsule signature + WIT/WASM
  match wait on upstream Doctor extension; approver-hardware-key
  waits on S-11 (mobile).
- The `NetworkPostureCheck` for `BrokerOnly` posture returns
  `Severity::Warn` — we don't yet have a way to tell direct from
  brokered traffic in the observation type. Documented in the
  check's doc comment + tracked in S-7b.
- The `TlaSpecsCurrentCheck` is a filesystem count, not a
  per-capsule verification record from BenchmarkRegistry. The
  RFC implies the latter; we deliver the former because the
  BenchmarkRegistry contract isn't deployed yet (S-15 work).
  The check is honest about this in its doc comment.

**Neutral.**

- The crate uses `Box<dyn NistCheck>` in `v1_checks()` and the
  top-level `run()`. Dynamic dispatch adds ~one indirect call
  per check; doctor isn't perf-critical (runs at start and once
  per 24h), so the overhead is invisible.

## Alternatives considered

1. **Extend upstream's `DoctorContext` first, then write our
   checks against the unified shape.** Rejected: requires the
   upstream PR to merge before S-7 can start; serializes work
   the exception clause exists to parallelize.
2. **Make our checks operate on JSON or TOML config rather than
   typed `NistDoctorContext`.** Rejected: loses the type safety
   that makes Rule-1 cheap to honor. Typed contexts mean a
   misspelled field is a compile error, not a doctor report that
   silently says "skipped".
3. **Implement only #7 (model hash) and #9 (role lattice) in
   S-7; defer the rest.** Rejected: would leave the doctor
   meaningfully incomplete and force S-8 (overlay bundles) to
   land a half-finished doctor for its acceptance criteria. The
   marginal cost of adding #3, #6, #8 in the same crate is small
   because they share `NistDoctorContext`.

## Reversal conditions

ADR-006 is reversed when:

- The upstream PR for `citrate_agent_core::doctor` accepts our
  five checks AND
- The federation manifest bumps the runtime rev AND
- Our `Cargo.toml` re-pins `citrate-agent-core` to that rev AND
- The 14 tests transfer to the upstream test runner.

A successor ADR records the deletion of `nist-agent-doctor` and
the new `citrate_agent_core::doctor` re-exports.

## References

- RFC-CIT-AGENT-0001 §10 (Doctor pre-flight check), §10.2 (11
  checks), §10.3 (output + severity).
- [`ADR-002`](ADR-002-tla-custody-in-nist-agent.md),
  [`ADR-004`](ADR-004-agent-loop-and-model-resolver-placement.md),
  [`ADR-005`](ADR-005-policy-bundle-placement.md) — the three
  prior exception-clause exercises.
- [`.agentile/planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md`](../planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md) — the exception clause.
- `citrate-agent-runtime/agent/core/src/doctor/` — the upstream
  CIT-AGENT-7a framework we compose with.
- Federation rules 1 (no stubs in production paths), 9 (one
  source of truth — `CheckResult` and `Severity` live upstream),
  10 (TLA+ specs verify in CI — TlaSpecsCurrentCheck binds
  here).
