---
created: 2026-05-20T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: accepted
adr: 001
sprint: S-3
---

# ADR-001: Workspace shape and runtime consumption

| Field | Value |
|---|---|
| **ADR Number** | ADR-001 |
| **Date** | 2026-05-20 |
| **Status** | ACCEPTED |
| **Author** | Saul Loveman + Claude Opus 4.7 (1M context) |
| **Sprint** | S-3 |

## Context

nist-agent is positioned as a **sidecar consumer** of
`citrate-agent-runtime` per
[`.agentile/planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md`]. The
runtime canonically owns the `citrate-agent-core` crate. nist-agent
needs Rust crates that build on top of that core to add: an
EVM-chain-agnostic adapter trait (S-4), overlay-keyed policy bundles
(S-6, S-8, S-9), the Slint operator app (S-10), and the signed
distribution pipeline (S-12).

Three forces in tension at sprint kickoff:

1. **Federation Rule 1** — no mocks, stubs, or TODOs in production
   paths. The first crate we land must do something real with the
   runtime's public surface, not just compile.
2. **Federation Rule 11 + 12** — the federation manifest is canonical,
   and cross-repo deps follow the drift map. The Cargo dep on
   `citrate-agent-core` must use the manifest's rev pin and the
   `[[drift]]` constraint must exist before the Cargo.toml does.
3. **RFC §3.2** — the v1.0 public surface is frozen; the runtime
   names which types are part of it. Our prelude crate has to mirror
   that exact list, not re-invent it.

Additional constraints:

- The runtime workspace's manifest declares
  `package.repository = "https://github.com/CitrateNetwork/citrate-agent-runtime"`,
  but the runtime is `visibility = "private"` (per federation
  `manifest.toml`). Cargo fetches must therefore use an SSH alias
  with a deploy key — runtime itself uses this pattern for
  `citrate-wallet-core` via `github-citrate-chain`.
- The `~/.ssh/config` already has the aliases
  `github-citrate-agent-runtime` and `github-citrate-chain` mapped
  to per-repo readonly deploy keys at
  `~/.config/citrate-split-keys/`.
- Cargo's bundled libgit2 does not read `~/.ssh/config`; system git
  does. So `git-fetch-with-cli = true` is required in
  `.cargo/config.toml`.

## Decision

**We will stand up a virtual Cargo workspace at the repo root,
consume `citrate-agent-core` as a git-pinned dependency on the
manifest's canonical rev (no vendoring, no path-dep at sprint
close), and place every nist-agent crate under `crates/`.**

Concretely:

1. The workspace is **virtual** (no root `[package]` section). Each
   crate lives at `crates/<name>/` with its own `Cargo.toml`.
2. `citrate-agent-core` is referenced in `[workspace.dependencies]`
   as `{ git = "ssh://git@github-citrate-agent-runtime/CitrateNetwork/citrate-agent-runtime", rev = "<manifest rev>" }`.
   The rev value is the **single edit point** when the federation
   bumps the runtime pin via `pin-bump.sh`.
3. The first crate is `nist-agent-prelude`. It re-exports the exact
   set of types the RFC §3.2 public surface names
   (`ApprovalQueue`, `RecorderClient`, `PendingView`, `ToolCall`,
   `ToolResult`, `ApprovalOutcomePublic`) and adds the
   `Overlay` enum capturing the v1.0 overlay set (RFC §2.3).
4. `rust-toolchain.toml` pins `channel = "stable"` with `rustfmt`
   and `clippy` — byte-identical to runtime's pin.
5. `deny.toml` mirrors runtime's license/advisory/ban posture
   byte-identical except for an `allow-git` list authorizing the
   two SSH-aliased federation sources.
6. `.cargo/config.toml` sets `git-fetch-with-cli = true` so the SSH
   aliases resolve.

## Consequences

**Positive.**

- Test count baseline starts at 4 (vs. 0), satisfying Rule 2's
  monotone-non-decreasing invariant from S-3 onward without S-4
  having to bootstrap a test from zero.
- Federation drift-check turns green for the
  `nist-agent → citrate-agent-runtime` constraint as soon as this
  commit lands — no S-4 dependency.
- Future crates (`nist-agent-chain` in S-4, `nist-agent-policy` in
  S-6, etc.) inherit toolchain, deny posture, and workspace
  dependencies for free; their per-crate `Cargo.toml`s stay small.
- The "single edit point" property means future rev bumps are
  one-line edits with `pin-bump.sh` propagation.

**Negative.**

- nist-agent now requires the `github-citrate-agent-runtime` SSH
  alias to be configured on every developer machine and on CI. CI
  configuration follows what `citrate-agent-runtime/.github/workflows/ci.yml`
  already does (an `SSH_DEPLOY_KEY` secret per source repo); this
  is implementation work for S-3's WP-3.7 (CI wiring), deferred
  from this ADR's scope.
- The git-pinned dep slows clean `cargo build` materially the first
  time a builder runs (it pulls runtime + chain transitively, ~38s
  observed locally on the day this ADR was written). Subsequent
  builds hit the cargo cache. Acceptable.

**Neutral.**

- We commit `Cargo.lock` per Rust convention for binary-producing
  workspaces. Once a binary crate exists (S-12), this becomes
  load-bearing for reproducible builds.

## Alternatives considered

1. **Vendor `citrate-agent-core` as a git submodule.** Rejected:
   defeats federation Rule 11 (manifest is canonical) and creates
   two sources of truth for the rev pin.
2. **Path-dep against `/home/saul/Projects/Citrate-Labs/citrate-agent-runtime`.**
   Rejected for production: works only on the architect's machine.
   May still be used as a one-line override during local
   experimentation via `[patch."ssh://..."]` — explicitly NOT
   committed to `main`.
3. **Flat workspace (`Cargo.toml` at root is also the prelude
   crate).** Rejected: as crate count grows (S-4 adds chain, S-6
   adds policy, S-10 adds slint-shell, S-11 adds mobile-bridge),
   the flat layout becomes awkward.
4. **Re-export `citrate_agent_core::*` from the prelude.** Rejected:
   the RFC §3.2 public surface is named explicitly. A glob re-export
   would also pull in future-private items if runtime forgets to
   mark them.

## Open questions

- The runtime publishes under `license = "MIT"`. nist-agent
  publishes under `Apache-2.0`. This is compatible (Apache-2.0
  source can consume MIT deps) but creates a small license-bookkeeping
  surface for the Trail of Bits engagement in S-13. Tracked in
  S-13's prep checklist, not this ADR.
- When the runtime ships its first audited release to crates.io, we
  should swap the git dep for a crates.io dep. Tracked as a
  follow-up ADR placeholder; no action this sprint.

## References

- [`planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md`](../planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md)
- `citrate-federation/manifest.toml` — `[repos.citrate-agent-runtime]` and `[[drift]]` entries for nist-agent
- RFC-CIT-AGENT-0001 §3.2 (frozen public surface)
- `citrate-agent-runtime/agent/core/src/lib.rs` — canonical re-export list
- Federation rules 1, 11, 12
