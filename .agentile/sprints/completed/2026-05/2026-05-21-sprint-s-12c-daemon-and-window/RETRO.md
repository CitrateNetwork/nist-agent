---
created: 2026-05-21T00:00:00Z
branch: feat/s-12c-daemon-and-window
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: complete
sprint: S-12c
---

# Retro — Sprint S-12c (daemon + window-up + reproducibility CI + polish)

## What landed

A daemon you can actually run, a `wizard` subcommand that
opens a Slint window, a release pipeline that fails on
reproducibility drift, a clean license posture, and a
streaming installer. Closes the v1.0 daemon-runs-on-prem
story; the only thing left between here and v1.0 tag is the
HSM ceremony + first tag push (both operator-side).

15th workspace crate `nist-agent-daemon`. The end-to-end
smoke this sprint demonstrated — `citrate-agent daemon`
backgrounded, then `citrate-agent status` connects via Unix
socket and reads live state — is the load-bearing proof that
the architecture composes.

## Metrics delta

| Metric | Before (S-12b) | After (S-12c) | Delta |
|---|---|---|---|
| `cargo test --workspace` | 211 | 234 | +23 |
| Workspace crates | 14 | 15 | +1 |
| ADRs | 12 | 12 | 0 |
| `cargo deny check licenses` | FAILED | ok | flipped |
| `release.yml` reproducibility gate | absent | present | new |

## What went well

- **State-machine + IPC pattern is right-shaped.** `Daemon`
  holds `Arc<DaemonState>` — readers grab the Arc, take a
  short Mutex lock for the field they need, release. No
  cross-field locking discipline to enforce; no deadlock
  surface beyond what the per-field Mutex already constrains.
- **Test coverage for the daemon is dense.** 21 tests pin
  config validation, IPC wire-form, anchor recording, ring
  capacity, stale-socket cleanup, malformed-request handling.
  The end-to-end IPC test (`status_request_round_trip_via_socket`)
  exercises real Unix-socket plumbing in-process.
- **Static vs daemon-driven status is graceful.** `citrate-
  agent status` tries the socket; if unreachable, falls back
  to the build-time readout with a one-line note. Operator
  doesn't have to think about whether the daemon is running.
- **Protocol versioning was cheap to add.** Every IPC response
  carries `protocol_version: "1.0"`. A future gRPC v2.0
  migration sees the field on the wire and can decide whether
  to fall back to JSON; CLI clients see it and can warn on
  mismatch.
- **License posture is finally clean.** The `unused-allowed-
  license = "allow"` setting is the right knob for a config
  that mirrors upstream — drift from upstream's allow list
  doesn't fail our build.

## What was tricky

- **`#[serde(default)]` on a struct field uses the type's
  `Default` impl, not the field's `default = "fn"`.** Caught
  by `minimal_toml_parses_with_defaults` failing because
  `AnchorSection::default()` left `nightly_at_iso` empty.
  Fixed with manual `Default` impl. Worth remembering for any
  config struct with a serde-default fn on a field.
- **`unwrap_err` requires Debug on the Ok type.** Same trap
  as S-12 hit on `ReleaseVerifier`. The `Daemon` struct
  doesn't need Debug for the same reason — its `listener`
  field doesn't expose Debug-shaped state. Avoided by not
  using `unwrap_err` in daemon tests.
- **Slint dep-tree pulls in GPL-3.0 / royalty-free licenses.**
  Pre-existing license-allowance gap. Surfaced when I added
  `nist-agent-cli` as a non-feat-ui-only crate; the
  `slint-build` build-dep ships in `Cargo.lock` even when
  `feat-ui` is off. Added the LicenseRef-Slint-* lines to the
  allow list with rationale. If we ever want to keep slint's
  build deps out of the dep tree by default, that's a
  separate refactor.
- **`tempfile::TempDir::into_path` is deprecated in 3.20+.**
  Switched to `keep()`. Minor; the deprecation only fired
  with the daemon subcommand's `--smoke` path that uses a
  leaked scratch dir.
- **The wizard window doesn't actually run in a non-display
  context.** S-12c's `citrate-agent wizard` only verifies
  the binding compiles + dispatches to `launch()`. Running
  the actual window in CI requires a Slint platform backend
  (xvfb / software renderer); deferred. The Slint surface is
  exercised by `cargo test -p nist-agent-wizard --features
  feat-ui` which works headless.

## What to carry into the v1.0-tag sprint

- **Bind `queue_depth` to upstream `ApprovalQueue`.** Today
  the field is operator-driven via `Daemon::set_queue_depth`;
  the future wiring reads `ApprovalQueue::peek_n_pending()`
  on a cadence. Trivial once the binding lands.
- **Real anchor-write integration.** The daemon's
  `record_anchor` is the right shape; the cadence ticker
  (tokio task that fires nightly + on per-install events)
  needs to drive it against a `dyn ChainClient` from S-4.
- **HSM ceremony + first tag push.** Out of scope for the
  Rust workspace; documented in `docs/audit/HSM_SEAM.md`
  + the v1.0-tag sprint stub.
- **xvfb-shaped CI for the Slint window.** If we want
  `citrate-agent wizard` to be exercised end-to-end in CI,
  add a `feat-ui` job to release.yml running under xvfb.
  Today the wizard test surface is headless-only.

## Open follow-ups

- **`citrate-agent daemon --reload`** for hot PolicyBundle
  reload without restart. Not v1.0; nice-to-have for pilots.
- **`citrate-agent queue` subcommand** that lists pending HITL
  proposals via IPC. The `IpcRequest::QueueDepth` shape
  already supports it; just needs a new subcommand wrapper +
  a richer queue-list response variant.
- **WORM sink + recent-audit ring drift detection.** The
  daemon's in-memory ring and the on-disk WORM records should
  agree; the daemon should periodically reconcile and surface
  drift. Mark for v1.1.
- **gRPC migration path.** When the federation's multi-product
  IPC story matures, the `IPC_PROTOCOL_VERSION = "1.0"` field
  is what gates the migration. Don't promote to 2.0 until
  there's a concrete cross-product consumer.
