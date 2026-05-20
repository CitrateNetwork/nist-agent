---
created: 2026-05-20T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: completed
sprint: S-1
---

# Sprint S-1 — Retrospective

## Outcome

| Field | Value |
|---|---|
| **Goal achieved?** | YES |
| **WPs planned / closed** | 8 / 8 |
| **Carry-forward WPs** | none |
| **Closing branch** | `main` (final commit pending the close commit) |

## Metrics delta

| Axis | Start | End | Δ |
|---|---|---|---|
| Tests | 0 | 0 | 0 (Rust workspace not yet stood up — captured in S-3) |
| Formal specs | 0 | 0 | 0 (specs land in S-2) |
| CI tripwires | (skeleton defaults: 7) | 7 | 0 |
| Frontmatter coverage | n/a | 50/50 (100.0%) | new baseline |
| Gherkin features | 0 | 44 | +44 |
| Sprint backlog stubs | 0 | 12 | +12 |

No axis regressed. Tests and specs are intentionally 0 at S-1 close
— S-1's scope was structural docs, not code. They become ratcheted
axes starting S-2 (specs) and S-3 (tests).

## What worked

- **The federation-pointer style AGENT_ENTRY.md was the right call.**
  Mirroring `citrate-agent-runtime/.agentile/AGENT_ENTRY.md` gave us
  a 90-line entry that points at the federation control plane rather
  than duplicating it. The bootstrap-generated entry was generic and
  would have drifted from federation conventions.
- **Crosswalking against the runtime in `ALIGNMENT.md` before
  scoping any sprint** — this caught at least three potential
  duplications (HITL queue, audit chain, capsule loader) and turned
  them into "consume runtime" rows. Without this, S-4/S-5/S-6 would
  have been ~3× the work.
- **Full feature inventory in one sprint instead of growing it
  per-sprint** — it took ~2h of authoring but now every future sprint
  has a green-or-not target file. The alternative (author features
  inside each sprint's kickoff) would have made S-2 onwards slower.
- **The five-file planset structure** (OVERVIEW / ROADMAP /
  ALIGNMENT / FEATURE_INVENTORY / DEPENDENCIES) is a good template
  for future plansets. Each file has a clear purpose; cross-links
  let readers navigate in <30 seconds.

## What didn't work

- **The bootstrap CONFIG template ignored the `--description` flag**
  passed to `bootstrap.sh --non-interactive`. The placeholder
  `<ONE_LINE>` survived into the committed file and had to be
  patched in WP-1.3. Filed against
  `github.com/citratenetwork/agentile` as a follow-up.
- **The skeleton's root-level LICENSE / README / CHANGELOG / INSTALL
  files leaked into nist-agent's identity.** They were all about the
  skeleton, not about us. WP-1.6 patched README + LICENSE; the close
  commit moved CHANGELOG and INSTALL into `docs/agentile-skeleton/`.
  Skeleton bootstrap should probably blank these or move them itself
  on first run.
- **WP-1.8 (federation manifest entry) was originally deferred to
  S-3 but was clearly closer to S-1 than to S-3.** Pulling it
  forward into the S-1 close window was free; the original deferral
  was over-conservative planning. Note for future sprints: a WP
  that's "open the federation entry for this work" almost always
  belongs in the sprint that opens the workstream.

## What surprised us

- **citrate-agent-runtime is much further along than the user
  initially conveyed.** ~12k LOC, TLA+ already verified for 2 of 5
  normative specs, 10 production capsules, AnchorRegistry adapter
  live. This shifted the whole planset: nist-agent is not a
  greenfield build, it's a *packaging and EVM-generalization* layer
  on top of a mostly-finished engine.
- **The skeleton ships `.agentile/SOUL.md` / `SPIRIT.md` / `AGENT.md`**
  — speculative rule-of-meaning docs. They don't appear in federation
  conventions and may or may not survive. Left as-is for now; revisit
  in S-2 if the federation rule set diverges.

## Lessons for future sprints

1. **Read `ALIGNMENT.md` at every sprint kickoff** before scoping
   any work that could plausibly live upstream.
2. **Federation-coupled WPs belong in the sprint that creates them**,
   not in a later "infrastructure" sprint.
3. **Bootstrap-generated files are starting points, not products.**
   Audit every file the skeleton creates and decide whether to keep,
   rewrite, or relocate it before the next sprint opens.
4. **44 features × ~50 lines was the right granularity.** Use this
   as the default for future feature-inventory sprints in adjacent
   plansets (e.g. `nist-sidecar-v1.1`).

## Next sprint(s)

- **S-2 (TLA+ spec port)** and **S-3 (Cargo workspace)** activate
  together with no mutual blocker.
- **S-4 (EVM adapter trait)** is the first sprint that ships unique
  nist-agent value over `citrate-agent-runtime`; it's where the
  product proposition starts being demonstrable.
