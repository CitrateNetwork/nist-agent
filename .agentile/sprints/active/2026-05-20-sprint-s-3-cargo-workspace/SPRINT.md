---
created: 2026-05-20T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
sprint: S-3
---

# Sprint S-3: Cargo workspace + consume citrate-agent-core

## Sprint Metadata

| Field | Value |
|---|---|
| **Sprint ID** | `S-3` |
| **Sprint Name** | Cargo workspace + consume citrate-agent-core |
| **Goal** | Stand up the Rust workspace at the repo root, depend on `citrate-agent-core` at a federation-pinned rev, mirror `citrate-agent-runtime`'s `deny.toml` license + advisory posture, pin the toolchain via `rust-toolchain.toml`, and capture the test-count baseline. |
| **Branch** | `main` |
| **Start Date** | 2026-05-20 |
| **End Date (target)** | 2026-05-27 |
| **Status** | `KICKED OFF` |
| **Planset** | [`../../planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md`](../../planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md) |
| **Predecessors** | S-1 (closed). S-2 runs in parallel (no mutual blocker). |

## Why this sprint

Every sprint from S-4 onward (`evm-chain-adapter-trait`,
`agent-loop-and-model-resolver`, `policy-bundle`, `doctor-checks`,
overlay bundles, Slint, mobile, distribution) needs a place to land
Rust code. We do this once, conservatively, so the rest of Phase 1
can move fast without re-litigating workspace structure or
dependency posture each time.

The workspace must consume `citrate-agent-core` (in
`citrate-agent-runtime`) as a Cargo dependency, not vendor or copy
it, per [`ALIGNMENT.md`](../../planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md).
The exact rev pin is sourced from `citrate-federation/manifest.toml`
(which S-1 close opened an entry in).

## Deliverables

- `Cargo.toml` at repo root — virtual workspace.
- `rust-toolchain.toml` — pinned to the same channel + components as `citrate-agent-runtime/rust-toolchain.toml`.
- `deny.toml` — mirrors `citrate-agent-runtime/deny.toml` (license allow-list, advisory rules, ban rules).
- `crates/nist-agent-prelude/` — first nist-agent crate; depends on `citrate-agent-core`; re-exports the prelude with a thin overlay-aware wrapper. Establishes the *shape* of subsequent crates.
- `crates/nist-agent-chain/` — second crate (stub for now); will hold the EVM-adapter trait in S-4.
- `.agentile/coverage/baseline.json` — `tests.command` populated; baseline count captured.
- `.github/workflows/ci.yml` — updated to add `cargo test --workspace`, `cargo deny check`, `cargo fmt --check`, `cargo clippy -D warnings`.
- ADR: `.agentile/adrs/2026-XX-XX-workspace-and-runtime-consumption.md`.

## Test Baseline (start of sprint)

| Metric | Count | Captured | Canonical command |
|---|---|---|---|
| **Tests** | 0 | 2026-05-20 | (to be captured by this sprint into `baseline.json`) |
| **Formal specs** | 0 (S-2 in flight) | 2026-05-20 | `scripts/ci/check_spec_ratchet.py` |
| **CI tripwires** | 7 | 2026-05-20 | `scripts/ci/check_tripwire_ratchet.py` |
| **Frontmatter coverage** | 100% | 2026-05-20 | `scripts/ci/check_frontmatter.py` |

## Method

Infrastructure sprint — no TLA+ required, but BDD scenarios from
`features/distribution/` shape the deliverables. The work is:

1. Read `citrate-agent-runtime`'s `Cargo.toml`, `rust-toolchain.toml`,
   `deny.toml` to learn the canonical posture.
2. Author our workspace `Cargo.toml` mirroring it.
3. Add `citrate-agent-core` as a path dep first (for local testing
   against `/home/saul/Projects/Citrate-Labs/citrate-agent-runtime/`),
   then flip to a git pin from `citrate-federation/manifest.toml`
   before sprint close.
4. Stand up the first two crates (`nist-agent-prelude`,
   `nist-agent-chain`) with smoke tests so the test ratchet has a
   non-zero baseline.
5. Wire CI.
6. Capture baseline.

## Work Packages

### WP-3.1 — Mirror runtime toolchain + deny posture

Acceptance: `rust-toolchain.toml` at repo root, channel + components
match runtime; `deny.toml` at root, license/advisory/ban rules match
runtime byte-for-byte modulo project-name fields.

### WP-3.2 — Author workspace `Cargo.toml`

Acceptance: virtual workspace declares `crates/*` members; `[workspace.dependencies]` block lists `citrate-agent-core` and shared deps with versions pinned to runtime's.

### WP-3.3 — `crates/nist-agent-prelude`

Acceptance: crate compiles with `cargo build -p nist-agent-prelude`;
re-exports `citrate_agent_core` prelude items; adds a trivial smoke
test asserting `citrate_agent_core::audit::AuditRecord` can be
constructed. Test count baseline ≥ 1 after this WP.

### WP-3.4 — `crates/nist-agent-chain` stub

Acceptance: empty crate that compiles, exists to be filled by S-4
with the `trait ChainClient`.

### WP-3.5 — Flip path-dep to git pin

Acceptance: `Cargo.toml` references
`citrate-agent-core` via git URL + rev from
`citrate-federation/manifest.toml` (rev =
`2591ed2d28780ca7938befe853be7cd1620029cc` at S-1 close);
`cargo build --workspace` green; `Cargo.lock` committed.

### WP-3.6 — Capture test-count baseline

Acceptance: `.agentile/coverage/baseline.json` has
`tests.command = "cargo test --workspace 2>&1 | grep 'test result:' | awk '{s+=$4}END{print s}'"`;
`tests.count` matches the post-WP-3.3 count.

### WP-3.7 — CI wiring

Acceptance: `.github/workflows/ci.yml` runs
`cargo build`, `cargo test --workspace`, `cargo deny check`,
`cargo fmt --check`, `cargo clippy --workspace -- -D warnings`,
`scripts/ci/check_test_ratchet.py`,
`scripts/ci/check_frontmatter.py`,
`scripts/ci/check_tripwire_ratchet.py`. Green on a PR.

### WP-3.8 — ADR

Acceptance: `.agentile/adrs/2026-XX-XX-workspace-and-runtime-consumption.md`
captures: why path-dep was used during development and flipped to
git pin at close; why the workspace is virtual not flat; how the
manifest pin is bumped (link to `pin-bump.sh`).

## Daily updates

- 2026-05-20 — Kickoff. Manifest entry for nist-agent landed in
  Citrate-Labs federation. Path-dep to local runtime is the first
  step; git-pin flip happens at sprint close.

## Exit criteria

- [ ] Workspace builds (`cargo build --workspace`)
- [ ] Tests pass (`cargo test --workspace`); count ≥ 1; baseline.json populated
- [ ] `cargo deny check` green
- [ ] `cargo fmt --check` green
- [ ] `cargo clippy -D warnings` green
- [ ] Manifest pin flipped from path-dep to git-rev
- [ ] CI workflow updated and green on a PR
- [ ] ADR landed
- [ ] Frontmatter coverage stays at 100%
- [ ] Sprint moves to `sprints/completed/2026-05/` (or `2026-06/` if it slips)

## Close note

(filled in at close)
