---
created: 2026-05-20T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: completed
sprint: S-1
closed: 2026-05-20T00:00:00Z
---

# Sprint S-1: RFC canonization + planset + feature inventory + backlog seed

## Sprint Metadata

| Field | Value |
|---|---|
| **Sprint ID** | `S-1` |
| **Sprint Name** | RFC canonization + planset land + features + backlog |
| **Goal** | Land the structural docs: RFC as markdown, the v1.0 planset, the full Gherkin feature inventory, and 12 backlog sprint stubs — so subsequent sprints (S-2 onward) can be kicked off mechanically. |
| **Branch** | `main` |
| **Start Date** | 2026-05-19 |
| **End Date (target)** | 2026-05-20 |
| **Status** | `COMPLETE` |
| **Planset** | [`../../planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md`](../../planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md) |
| **Predecessors** | S-0 bootstrap (commit `cebd5d7`) |

## Why this sprint

The repo was bootstrapped from the agentile skeleton. Before any
real engineering work begins, we need: (a) the RFC accessible inside
the repo with Rule-12 frontmatter so it's grep-able and indexable;
(b) a planset that declares the multi-sprint workstream and
crosswalks against `citrate-agent-runtime` so we don't duplicate work;
(c) a Gherkin feature file per RFC normative section so every future
sprint has a green-or-not target; (d) a backlog of sprint stubs that
ROADMAP.md lists. Without S-1, S-2 onward have nothing to point at.

## Deliverables

- `docs/rfcs/RFC-CIT-AGENT-0001.md` — RFC v0.1 with Rule-12 frontmatter.
- `.agentile/AGENT_ENTRY.md` — federation-pointer style entry.
- `.agentile/CONFIG.md` and `.agentile/PRODUCT_SPEC.md` — completed.
- `.agentile/planset/2026-05-19-nist-sidecar-v1/{OVERVIEW,ROADMAP,ALIGNMENT,FEATURE_INVENTORY,DEPENDENCIES}.md`.
- `features/{core,capsule,chain,overlays,surfaces,distribution}/*.feature` — 44 Gherkin files.
- `.agentile/sprints/backlog/S-2..S-13` — 12 backlog stubs.

## Test Baseline (start of sprint)

| Metric | Count | Captured | Canonical command |
|---|---|---|---|
| **Tests** | 0 | 2026-05-20 | (set in S-3 alongside cargo workspace) |
| **Formal specs** | 0 | 2026-05-20 | `scripts/ci/check_spec_ratchet.py` |
| **CI tripwires** | (skeleton defaults) | 2026-05-20 | `scripts/ci/check_tripwire_ratchet.py` |
| **Frontmatter coverage** | (per `.agentile/INDEX/`) | 2026-05-20 | `scripts/ci/check_frontmatter.py` |

## Work Packages

### WP-1.1 — Convert RFC to markdown

Acceptance: `docs/rfcs/RFC-CIT-AGENT-0001.md` exists with Rule-12
frontmatter; the original docx is retained at the repo root as the
canonical artifact. Status: **DONE**.

### WP-1.2 — Replace bootstrap AGENT_ENTRY with federation-pointer style

Acceptance: `.agentile/AGENT_ENTRY.md` mirrors
`citrate-agent-runtime/.agentile/AGENT_ENTRY.md` structure, names
this repo's role (sidecar consumer), and lists what lives where.
Status: **DONE**.

### WP-1.3 — Fill CONFIG.md and PRODUCT_SPEC.md

Acceptance: every `<placeholder>` in CONFIG.md is replaced with a
real value; PRODUCT_SPEC.md states v1.0 scope, v1.1 additions, v2.0
additions, indefinite out-of-scope. Status: **DONE**.

### WP-1.4 — Build the planset

Acceptance: `.agentile/planset/2026-05-19-nist-sidecar-v1/` contains
five files (OVERVIEW, ROADMAP, ALIGNMENT, FEATURE_INVENTORY,
DEPENDENCIES) with Rule-12 frontmatter. Status: **DONE**.

### WP-1.5 — Author Gherkin feature inventory

Acceptance: `features/` contains one `.feature` file per RFC
normative section, totaling 44; each file references its RFC section
and its sprint stub. Status: **DONE**.

### WP-1.6 — Seed 12 backlog sprint stubs

Acceptance: `.agentile/sprints/backlog/<slug>.md` exists for S-2
through S-13 (12 files); each carries Rule-12 frontmatter, goal,
features-owned list, predecessors, exit criteria. Status: **DONE**.

### WP-1.7 — Re-run index and frontmatter ratchet

Acceptance: `scripts/sprint.sh index` regenerates
`.agentile/INDEX/INDEX_CHRONOLOGICAL.md`;
`scripts/ci/check_frontmatter.py` reports 100% coverage. Status:
**DONE** — frontmatter coverage 50/50 (100.0%) at close.

### WP-1.8 — Federation manifest opens an entry for nist-agent

Acceptance: an entry exists in `citrate-federation/manifest.toml`
for `nist-agent` at the current HEAD. Status: **DONE in same close
window** — manifest entry + `repos/nist-agent/{owners.md,deps.md}`
+ federation sprint file landed in Citrate-Labs/citrate-federation/
on 2026-05-20 (commit pending from user side).

## Daily updates

- 2026-05-19 — Kickoff. RFC analysis, methodology survey, planset
  drafted, decision points resolved with user, bootstrap.sh run.
- 2026-05-20 — All WPs except 1.6, 1.7, 1.8 are DONE. WP-1.6 in
  flight (12 backlog stubs). WP-1.7 will run at sprint close.
- 2026-05-20 (close) — WP-1.6 landed (12 stubs in `sprints/backlog/`);
  WP-1.7 ran (50/50 frontmatter); WP-1.8 landed in same window
  (federation manifest entry + repos/nist-agent/ + federation
  sprint authored). Sprint moved to `sprints/completed/2026-05/`.

## Exit criteria

- [x] RFC in markdown with frontmatter
- [x] AGENT_ENTRY federation-pointer style
- [x] CONFIG and PRODUCT_SPEC filled
- [x] Planset complete
- [x] 44 feature files
- [x] 12 backlog stubs
- [x] Index regenerated, frontmatter coverage 100%
- [x] Sprint moved to `sprints/completed/2026-05/`
- [x] Federation manifest entry opened (WP-1.8 absorbed into close)

## Close note (2026-05-20)

S-1 closed cleanly. Three commits in `nist-agent/main`:
`f856ec1` (skeleton import), `cebd5d7` (bootstrap), `a5a43ac`
(federation alignment). One commit pending on `citrate-federation`
(manifest entry + repos/nist-agent + federation sprint file).

**What shipped vs. plan.** Everything in the plan, plus WP-1.8
(originally deferred to S-3). The deferral was the right call at
plan-time — once S-1's structural docs were in, opening the
federation manifest entry was cheap. Pulling it forward keeps S-3
focused on the Cargo workspace.

**What changed mid-sprint.** Nothing material; the planset froze
on day 1. The bootstrap CONFIG template left `<ONE_LINE>` as a
placeholder even though `--description` was passed (minor skeleton
bug); patched manually in WP-1.3. Worth filing upstream against
`citrate-federation/agentile-skeleton` after S-3.

**Lessons for S-2 / S-3.**
- The skeleton-shipped LICENSE/README/CHANGELOG/INSTALL were all
  about the skeleton itself. We replaced LICENSE + README and moved
  the other two to `docs/agentile-skeleton/`. Worth flagging on
  bootstrap so future projects don't carry the skeleton's prose.
- 44 features × ~50 lines each was the right granularity — wide
  enough that each is a clean spec target, narrow enough that no
  feature needs decomposition for its owning sprint.
- The ALIGNMENT.md crosswalk against `citrate-agent-runtime` is the
  load-bearing doc going forward. Every sprint kickoff should
  read it before scoping.

**Next sprint(s).** S-2 (TLA+ port) and S-3 (Cargo workspace)
activate together; they have no mutual blocker.
