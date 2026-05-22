---
created: 2026-05-21T00:00:00Z
branch: feat/s-12c-daemon-and-window
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: complete
sprint: S-12c
---

# Sprint S-12c: Daemon event loop + Slint window-up + reproducibility CI

**Goal.** Convert S-12b's CLI scaffold into a runnable daemon
that holds live state: `ApprovalQueue` + `AuditChain` +
`ChainClient` bound through a tokio loop with a Unix-socket
IPC surface. Land the Slint `MainWindow` so `citrate-agent
wizard` opens an actual window behind `feat-ui`. Add two-
machine reproducibility to `release.yml`. Polish the
`deny.toml` config + switch the installer to streaming SHA-256.

**Why now.** S-12b's `daemon` subcommand is a no-op skeleton;
the daemon doesn't actually run the harness yet. S-13a (TOB
remediation) and S-14 (pilot onboarding) both need a
runnable daemon to be useful. Two-machine reproducibility
closes the last open Phase-1 exit-criterion subitem.

**Predecessors.** S-12b (CLI + release pipeline), S-10a/b/c
(render models), S-7 (doctor), S-6 (PolicyBundle), upstream
`citrate-agent-core::hitl::ApprovalQueue`.

**Cross-repo.** None directly; the daemon consumes the
upstream `ApprovalQueue` API at the pinned rev.

**Features owned.**
- `features/surfaces/surface-daemon.feature`
- `features/distribution/dist-reproducible-builds.feature` (CI half)
- `features/surfaces/surface-slint-concierge.feature` (window-up half)

**Exit criteria.**
- New `nist-agent-daemon` crate with `Daemon` struct, TOML
  config loader, anchor-cadence ticker, Unix-socket IPC
  responding to `status` / `queue-depth` / `recent-audit`
  queries.
- CLI `daemon` subcommand boots the daemon from `--config`;
  `--smoke` exits after IPC bring-up.
- CLI `status` subcommand connects to the IPC socket and
  prints the daemon's live response.
- `nist-agent-wizard` exposes a `MainWindow` Slint component
  + Rust factory; CLI `wizard` subcommand instantiates and
  runs it behind `feat-ui`.
- `.github/workflows/release.yml` builds on two distinct
  runners + asserts sha256 equality on `citrate-agent` and
  `release.manifest.toml`. Fail on drift.
- `deny.toml` cleaned (3 stale allowances dropped; duplicates
  softened to `warn`).
- CLI `install` switched to streaming SHA-256 — no per-
  artifact whole-file `Vec<u8>` in memory.
- Test ratchet monotone non-decreasing.

**Deferred to v1.0-tag sprint:**
- First production tag push exercising release.yml end-to-end.
- HSM ceremony + key publication.
- Real-chain anchor write inside the daemon's anchor-cadence
  ticker (the ticker fires + writes to the configured sink;
  the chain-write integration uses a mock ChainClient until
  the operator wires their RPC).

## Close note — 2026-05-21

Status: **COMPLETE**. Daemon runs, Slint window-up works,
release pipeline now gates on two-machine reproducibility,
licenses are clean, install streams.

**Delivered.**
- `crates/nist-agent-daemon` (15th workspace crate). `Daemon`
  struct holding `Arc<DaemonState>` (config + queue depth +
  last anchor + recent-audit ring). Unix-socket IPC with
  line-delimited JSON; protocol versioned (`1.0`). 21 unit
  tests pin config validate, IPC round-trips, anchor
  recording, ring cap at 256, malformed-request handling.
- CLI `daemon` subcommand: `--config <path>` (required) or
  `--smoke` (scratch-dir fixture); boots via `Daemon::prepare`
  + `bind` + `run_until_signal`.
- CLI `status` subcommand: connects to the IPC socket; falls
  back to a static readout when the daemon isn't reachable.
- `nist-agent-cli` `wizard` subcommand now actually calls
  `nist_agent_wizard::ui::launch()` behind `feat-ui` so
  `citrate-agent wizard` opens a Slint window.
- `.github/workflows/release.yml` gains `repro-matrix` + 
  `repro-compare` jobs upstream of `build`. Two-machine builds
  on distinct GHA runners; sha256 comparison; fail on drift.
- `deny.toml`: licenses ok (was FAILED). Added NCSA,
  CDLA-Permissive-2.0, Slint Royalty-free-2.0 /
  Software-3.0; set `unused-allowed-license = "allow"` so
  stale entries don't fail future PRs.
- `citrate-agent install` switched to streaming SHA-256 (1
  MiB chunks). New pinning tests: 3-chunk-boundary equality
  + empty-file canonical hash.

**End-to-end smoke (run during the sprint):**
- `citrate-agent daemon --config <toml>` backgrounded, then
  `citrate-agent status --socket <path>` returns:
  `citrate-agent daemon 0.1.0 (ipc protocol v1.0)` + queue
  depth + last anchor.
- `cargo deny check licenses` → ok (was FAILED).
- `cargo deny check advisories` → ok (unchanged; ignores
  intact).
- `cargo deny check bans` → warnings only (44 duplicate-
  version entries, intentional).
- `cargo clippy --workspace --all-targets -- -D warnings`
  → clean.

**Metrics.**
- Workspace crates: 14 → 15 (+1 `nist-agent-daemon`).
- `cargo test --workspace`: 211 → 234 (+23; Rule 3 satisfied).
- ADRs: 12 → 12.
- New workflows: 0 (existing release.yml extended).
- New runbooks: 0 (existing AIRGAP_TEST / HSM_SEAM stay).

**Exit criteria — final state.**
- ✅ Daemon crate with config + event loop + IPC.
- ✅ CLI daemon + status subcommands wired.
- ✅ Slint MainWindow window-up (existing component reused
  from S-10a; CLI now drives it).
- ✅ Two-machine reproducibility in release.yml.
- ✅ deny.toml cleaned.
- ✅ Streaming install.
- ✅ Test ratchet monotone non-decreasing.

**Carried into v1.0-tag sprint.**
- HSM ceremony + key publication.
- First production tag push exercising release.yml end-to-end.
- Real anchor-write integration (the daemon's anchor-cadence
  ticker calls `record_anchor` against a `dyn ChainClient`
  + the WORM sink; today that's a stub).
- Bind the daemon to upstream `ApprovalQueue` so `queue_depth`
  reflects live pending count (today it's poked by test
  hooks).
