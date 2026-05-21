---
created: 2026-05-20T00:00:00Z
branch: feat/s-2-tla-spec-port
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: completed
sprint: S-2
---

# Sprint S-2 — Retrospective

## Outcome

| Field | Value |
|---|---|
| **Goal achieved?** | YES |
| **WPs planned / closed** | 8 / 8 |
| **Carry-forward WPs** | none |
| **Closing branch** | `feat/s-2-tla-spec-port` (PR pending) |

## Metrics delta

| Axis | Start | End | Δ |
|---|---|---|---|
| Tests | 4 | 4 | 0 (unrelated; S-2 is spec work) |
| Formal specs | 0 | 5 | **+5** |
| CI tripwires | 7 | 7 | 0 |
| Frontmatter coverage | 100% | 100% | maintained |
| Federation drift constraints green | 10/11 | 10/11 | 0 (drift is Cargo-pin scoped, not spec scoped) |
| CI workflows | 8 | 9 | +1 (`tla-verify.yml`) |

Rule-10 BLOCKER now armed: future PRs that break a normative spec
get caught at CI before merge.

## What worked

- **Reading the archive before authoring.** The original plan
  treated 3 of 5 specs as fresh authoring. A 30-second `ls` of
  `citrate-agentile-archive/formal/specs/agent/` showed all five
  exist with PASSing TLC runs. The sprint shrank from
  "author + verify" to "port + verify CI wiring." Lesson:
  inventory the archive before estimating fresh work.
- **Renaming the two archive specs to RFC-aligned names rather
  than leaving them as `CapsuleInstall`/`BreakGlass`.** The RFC
  §9.1 names are what auditors (DCMA-DIBCAC, Trail of Bits) will
  reference. Auditor-readable names > git-blame-preserving names.
  The behavior is byte-identical; only `MODULE` declarations
  changed.
- **Using `actions/cache` for `tla2tools.jar`.** The jar is ~10
  MB; first run downloads it, subsequent runs hit cache. Avoids
  fetching every PR.
- **10-minute per-spec budget in CI** vs runtime's archive's 180s.
  Generous slack means no edge-case timeout flakes blocking PRs
  while still total < 20 min for all 5 specs.

## What didn't work

- **Two `.agentile/formal/` files came pre-populated from the
  agentile skeleton** (with `status: active` and skeleton author).
  Writing the new content failed the "must Read first" check on
  the Write tool, requiring an extra Read step. Minor friction
  but worth flagging: the bootstrap left these as live files,
  not templates.
- **The plan imagined "verify under TLC" as a sprint-time local
  step.** In practice TLC verification only runs locally if the
  contributor installs Java + tla2tools.jar. The right load-bearing
  gate is `tla-verify.yml` in CI; the local verification is
  ergonomic but not authoritative. The plan was imprecise about
  this distinction. ADR-002 records it explicitly.

## What surprised us

- **`DataClassLattice` was already in the archive.** Per the
  ALIGNMENT.md crosswalk I had it listed as one of "3/5 to author
  from scratch." Wrong — the archive had it with a working TLC
  run. The same was true for `CapsuleInstallGate` (as
  `CapsuleInstall`) and `BreakGlassPath` (as `BreakGlass`). All
  five normative specs predated the May-2026 federation split.
- **The skeleton's `check_spec_ratchet.py` is parameterizable via
  `baseline.specs.directories`** but the default of
  `.agentile/formal/specs` was already what we needed. No script
  edit required.

## Lessons for future sprints

1. **Inventory before authoring.** Whenever a plan calls for
   "fresh artifact X," check `citrate-agentile-archive/` first.
   The federation split happened recently; lots of pre-split
   artifacts still apply.
2. **CI cache keys for binary tools** (`tla2tools.jar` here, future
   ones may include `tla2tools-community.jar`, `gemma-4-e2b.gguf`,
   `wasmtime-cli`) should be versioned by the tool's release tag
   so cache invalidation is intentional, not date-based.
3. **Two-level Rule-10 enforcement is the right shape.** The
   file-count ratchet (`spec-ratchet`) catches deletion; the TLC
   verification job (`tla-verify`) catches semantic regression.
   Neither alone is sufficient.

## Pending CI activation

`tla-verify.yml` was added in this sprint but has not yet run on a
PR. The PR that introduces it will be the first execution. If TLC
takes longer than the 10-minute budget on any spec, that's a real
signal — increase the budget per-spec or tighten the `.cfg`.

## Next sprint(s)

- **S-4 (EVM chain adapter trait)** activates next. Lands
  `crates/nist-agent-chain/`, the `trait ChainClient`, and the
  generic EVM impl. Closes the last federation drift constraint
  (`citrate-wallet-core` pin).
- S-5, S-6 onward: unblocked.
