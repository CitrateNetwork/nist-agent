---
created: 2026-05-20T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: backlog
sprint: S-3
---

# Sprint S-3: Cargo workspace + consume citrate-agent-core

**Goal.** Stand up the Rust workspace, depend on `citrate-agent-core`
at a federation-canonical rev, mirror runtime's `deny.toml` posture,
and pin the toolchain via `rust-toolchain.toml`.

**Why now.** Everything from S-4 onward needs a place to land code.
We do this once, conservatively, so the rest of Phase 1 can move fast.

**Predecessors.** S-1, S-2.

**Features owned.** none directly (infrastructure sprint), but enables
all subsequent feature work.

**Cross-repo.** Opens `nist-agent` entry in
`citrate-federation/manifest.toml`. Federation sprint required.

**Exit criteria.**
- `Cargo.toml` workspace at root with member crates declared.
- `rust-toolchain.toml` pinned (matching runtime).
- `deny.toml` mirroring runtime's license + advisory posture.
- `citrate-agent-core` dep wired at runtime's tip rev.
- `cargo build --workspace` green; `cargo test --workspace` green (test count baseline captured to `.agentile/coverage/baseline.json`).
- Federation manifest entry merged.
