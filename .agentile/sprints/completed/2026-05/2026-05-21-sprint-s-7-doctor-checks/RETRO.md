---
created: 2026-05-21T00:00:00Z
branch: feat/s-7-doctor-checks
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: completed
sprint: S-7
---

# Sprint S-7 — Retrospective

## Outcome

| Field | Value |
|---|---|
| **Goal achieved?** | YES (5/8 in-scope checks); 3 RFC checks deferred to S-7b / S-11 (surfaces still unbuilt) |
| **WPs planned / closed** | 10 / 8 in scope; 2 deferred |
| **Carry-forward WPs** | WP-7.9 (upstream PR), WP-7.10 (RFC checks #1, #2, #5) |
| **Closing branch** | `feat/s-7-doctor-checks` (PR pending) |

## Metrics delta

| Axis | Start | End | Δ |
|---|---|---|---|
| Tests | 66 | 80 | **+14** |
| Formal specs | 5 | 5 | 0 |
| CI tripwires | 7 | 7 | 0 |
| Frontmatter coverage | 100% | 100% | maintained |
| Drift constraints | 11/11 green | 11/11 green | 0 |
| Workspace crates | 5 | 6 | +1 (`nist-agent-doctor`) |
| RFC §10.2 checks covered | 3/11 (upstream) | **8/11** (5 here + 3 upstream) | +5 |

The RFC §10.2 coverage delta is the most load-bearing number —
it directly affects the Trail of Bits (S-13) checklist.

## What worked

- **Reusing upstream's `CheckResult` and `Severity`.** Rule 9
  (one source of truth) holds: the upstream types own the
  result shape; our checks produce them. The harness can merge
  upstream + nist-agent results into a single `Vec<CheckResult>`
  without translation.
- **`Option<_>` skip semantics.** Each `NistDoctorContext` field
  is `Option<_>`; when `None`, the corresponding check returns
  `Severity::Pass` with `message = "skipped: <reason>"`. Mirrors
  upstream's CIT-AGENT-7a pattern. Lets operators run a partial
  doctor pass during first-run setup before everything is wired.
- **Test density per check.** Each check has at least 2 tests
  (pass + blocker) and a skip-path test where applicable. Total
  14 tests for 5 checks; the test count almost equals the check
  count × invariant count. Catches regressions early.
- **`NetworkPostureCheck` returning WARN for `BrokerOnly`** is
  honest about the v1 observation type's limitation. Better than
  pretending we can verify the constraint we can't yet.

## What didn't work

- **The model-hash test had a residual `Write` import warning.**
  I imported `std::io::Write` for a stub-write at the bottom of
  the test, but the actual hash check used an empty file. The
  warning was harmless but ugly; rustfmt didn't catch it because
  it wasn't a fmt issue. Resolved by deleting the unused write
  statement (the test still exercises the empty-file canonical
  hash, which is what matters).
- **The TlaSpecsCurrentCheck filesystem traversal** recurses only
  one level. A future addition of `.agentile/formal/specs/wallet/`
  or other subtrees would need traversal depth ≥ 2. Documented in
  the check's doc comment; tracked for S-7b.
- **Doctor `compute_overall()` is duplicated** between upstream
  (private) and our copy. The upstream PR will delete our copy
  and re-export upstream's. Small duplication; flagged but not
  fixed in S-7.

## What surprised us

- **The five checks needed minimal scaffolding.** Each is ~30-50
  lines of straightforward logic. The bulk of the crate is the
  context struct (~100 lines including docs) and the tests (~250
  lines). The implementation-to-test ratio is 1:2, which feels
  right for safety-critical code.
- **The `BrokerOnly` egress posture has no v1 observability path.**
  RFC §3.3 names broker-vs-direct as distinguishable in principle;
  the harness instrumentation doesn't yet record which mode an
  observation came from. WARN is the right intermediate state;
  upstream PR adds the discrimination.

## Lessons for future sprints

1. **Sprint scope correctness improves with prior sprint context.**
   S-6 left a clean policy crate, S-5 left a clean model crate,
   S-2 left a clean spec dir. S-7's checks read those three
   surfaces almost verbatim — the sprint took half a day instead
   of two because the bindings were already there.
2. **"Skip not error" is the right default for missing context.**
   The skip-path tests caught at least one situation where I'd
   have otherwise written a `panic!("must configure")` — the
   skip is Rule-1-clean and lets first-run setups produce
   meaningful reports.
3. **Doctor checks are the cheapest place to bind audit evidence
   to code.** Each check is ~50 lines but takes one RFC
   requirement from "we promise" to "the runtime asserts every
   24h." The marginal cost of adding another check (S-7b) is
   trivial; the marginal compliance value is huge.

## Pending follow-ups (S-7b / future)

1. **Upstream PR on `citrate-agent-runtime`.** Lifts
   `nist-agent-doctor` into `citrate_agent_core::doctor`. Unifies
   `NistCheck` + `Check` traits; merges `NistDoctorContext` into
   `DoctorContext`. Deletes our `compute_overall` copy.
2. **RFC §10.2 #1 + #2** (capsule signatures, WIT/WASM match).
   Requires upstream's capsule loader to expose a doctor-time
   re-verify entry point. Upstream-only work; S-7b prerequisite.
3. **RFC §10.2 #5** (approver hardware key bindings). Requires
   S-11's mobile companion + PIV verifier. S-11 dependency.
4. **NetworkPostureCheck's BrokerOnly discrimination.** Add a
   `via_broker: bool` field to `EgressObservation`; promote the
   WARN to PASS/BLOCKER based on the observation vs posture.
5. **TlaSpecsCurrentCheck depth-2 traversal.** Trivial; bundle
   with the BenchmarkRegistry per-capsule verification reading
   into one S-7b commit.
6. **`details` map population.** Today all checks pass
   `BTreeMap::new()` as `details`. Auditor-facing reports benefit
   from structured detail (count, age, paths). Tracked as a
   doctor-quality follow-up.

## Next sprint(s)

- **S-8 (Overlay bundles A)** unblocked. Authors signed
  PolicyBundle instances for CMMC-L3 + FERPA + COPPA + CIPA, plus
  supporting capsules (FERPA redact-and-attest, COPPA parental-
  consent verifier, CIPA content filter). Each overlay's reference
  deployment runs doctor with 8/11 checks armed.
- S-7b (upstream PR + RFC #1/#2/#5) can run in parallel to S-8.
