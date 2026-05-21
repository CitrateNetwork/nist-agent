---
created: 2026-05-21T00:00:00Z
branch: feat/s-6-policy-bundle
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: completed
sprint: S-6
---

# Sprint S-6 — Retrospective

## Outcome

| Field | Value |
|---|---|
| **Goal achieved?** | YES (in-scope); WP-6.7 (upstream PR) and WP-6.8 (wire into loop + model) deferred to S-6b |
| **WPs planned / closed** | 8 / 6 in scope; 2 deferred |
| **Carry-forward WPs** | WP-6.7 (upstream PR), WP-6.8 (PolicyBundle wiring into loop + model resolver) |
| **Closing branch** | `feat/s-6-policy-bundle` (PR pending) |

## Metrics delta

| Axis | Start | End | Δ |
|---|---|---|---|
| Tests | 41 | 66 | **+25** |
| Formal specs | 5 | 5 | 0 |
| CI tripwires | 7 | 7 | 0 |
| Frontmatter coverage | 100% | 100% | maintained |
| Federation drift constraints | 11/11 green | 11/11 green | 0 |
| Workspace crates | 4 | 5 | +1 (`nist-agent-policy`) |

66-tests baseline. The `DataClass` cross-layer alignment test +
the non-canonical CBOR rejection test are the two highest-value
adds — each guards a load-bearing invariant that would otherwise
silently drift.

## What worked

- **Reusing the ADR-004 pattern.** ADR-005 follows ADR-002/004's
  shape closely (placement decision, alternatives, reversal
  conditions). Authoring took ~10 minutes because the template
  was clear. The exception-clause pattern is now a documented
  pipeline.
- **Canonical CBOR + re-encode verification.** The defense-in-depth
  check (re-encode the decoded bundle, compare bytes to the
  signed input) is one line of test code and catches an entire
  class of signature-replay attacks. Trail of Bits will likely
  approve.
- **One-way ratchet enforced by the type system.** `ActiveOverlays::remove_with_workflow`
  consumes a `DecommissioningWorkflow` proof token by *type*, not
  by runtime check. There's no path to bypass that doesn't go
  through the compiler, which makes RFC §2.3's "removal requires
  a documented workflow" rule unbypassable by accident. The
  `remove_without_workflow` function exists only to let test code
  exercise the negative without needing to construct the proof
  type.
- **Cross-layer alignment test** for `DataClass` vs `Clearance`.
  Both sides pin the byte values explicitly; if either drifts,
  both tests fail in the same PR. This is the rule-9 violation
  that's tolerable because the test pinning replaces the
  source-of-truth discipline.

## What didn't work

- **The first build cycle exposed missing derives.** `Overlay`
  needed `Ord`/`PartialOrd` for `BTreeSet<Overlay>`. `Role`
  needed them for `BTreeMap<Role, ...>`. Three quick fixes, but
  three round-trips through the compiler. Lesson: when authoring
  a new crate that uses sorted collections, derive `Ord` on
  every enum at the start.
- **Serde and `[u8; 64]` mismatch.** ed25519-dalek signatures are
  64 bytes; serde's default derive only handles arrays up to 32.
  Switched to `Vec<u8>` with length validation at deserialize-
  use time. Cleaner alternative was `serde-big-array` but adding
  a helper crate for one field wasn't worth the supply-chain
  surface. Tracked: revisit when other 64-byte fields appear.
- **Two `Read` failures** on Cargo.toml editing — the first
  edits applied before the tool's read-tracking caught up. Self-
  inflicted; recovered with re-reads.
- **`Edit` failed twice on the Cargo.toml workspace member +
  deps changes.** The tool's "must read first" enforcement
  caught race conditions in my own command sequence. Cost ~1
  minute; lesson is to always Read before Edit on files I've
  modified earlier in the session.

## What surprised us

- **ciborium's deterministic-encoding default is exactly what
  RFC 8949 §4.2.1 specifies.** No flags to set; the encoder
  produces sorted-key maps, shortest-form integers, and so on
  out of the box. Other CBOR libraries require explicit
  configuration. This makes the canonical-form story
  Rule-1-clean: no `// TODO: turn on deterministic encoding`
  hiding anywhere.
- **The PolicyBundle struct's field count grew to 9 without
  feeling complicated.** Bundle version, name, validity window,
  overlay state, risk tier map, role assignments, anchor strategy,
  egress posture. Each field maps 1:1 to an RFC section, which
  made review trivial. Lesson: keep struct fields parallel to
  RFC structure where possible.

## Lessons for future sprints

1. **Three ADRs (002/004/005) now exercise the exception-clause
   pattern.** Future sprints that hit the same situation
   (runtime-scaffolded-but-unsequenced) should cite the trio as
   precedent and adopt the same crate-then-upstream-PR shape.
2. **Cross-layer alignment tests are cheap insurance.** When two
   layers MUST agree on a byte mapping (Solidity enum ↔ Rust
   enum), pin both with explicit byte-value tests. The cost is
   ~5 lines; the value is catching the drift the moment it
   happens.
3. **Type-level enforcement beats runtime checks for one-way
   rules.** `remove_with_workflow(proof: &DecommissioningWorkflow)`
   forces every caller to supply the proof; `remove(force: bool)`
   would let a future caller silently bypass it. Rule 10 is
   easier to honor when the compiler does the work.

## Pending follow-ups (S-6b candidates)

1. **Upstream PR on `citrate-agent-runtime`.** Lift
   `nist-agent-policy` into `agent/core/src/policy/`. Same shape
   as the S-5b upstream-PR follow-up.
2. **Wire PolicyBundle into agent loop + model resolver.** Loop
   honors risk-tier escalation map; model resolver reads
   `egress_posture` to refuse egress-required sources.
3. **Unify `DataClass` and `Clearance`.** Pick canonical
   location — `citrate_agent_core::types` is the strongest
   candidate since both `nist-agent-chain` and `nist-agent-policy`
   would consume it without circular deps.
4. **Overlay-state history with timestamps.** Replace
   `BTreeSet<Overlay>` with `Vec<OverlayActivation>` carrying
   activated_at / decommissioned_at / workflow_ref. Lands in S-8
   when the first real overlay bundle gets signed.
5. **`serde-big-array`** vs `Vec<u8>` for the 64-byte signature
   field. Mostly cosmetic; revisit if other 64-byte fields appear
   (e.g. session keys, BLS signatures in v2).

## Next sprint(s)

- **S-7 (Doctor checks 6–11)** unblocked. Doctor's check #3
  ("Policy bundle is signed by Security Officer key and is not
  past its signed validity window") binds directly to
  `RawSignedBundle::verify_and_decode` and `PolicyBundle::activate`.
- S-6b (upstream PR + wiring + lattice unification) can run in
  parallel.
