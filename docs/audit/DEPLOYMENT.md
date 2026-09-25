---
created: 2026-05-21T00:00:00Z
branch: feat/s-13-tob-engagement
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
audience: Trail of Bits engagement team
---

# Deployment — nist-agent v1.0-rc Reference Stand-Up

> How to bring up a hands-on reference deployment that
> exercises every load-bearing surface in
> [`THREAT_MODEL.md`](THREAT_MODEL.md). Auditors who want to
> reproduce a specific feature scenario should start here.

## Topology

Single host. The reference deployment is single-tenant; multi-
tenant deployments are an operator concern and out of scope.

```
+-----------------------------------------------------------+
| Operator host (Linux, x86_64)                             |
|                                                            |
|  +---------------+   +-----------------+                  |
|  | citrate-agent |<->| WORM audit sink |                  |
|  |   daemon      |   | (/var/lib/...)  |                  |
|  +---------------+   +-----------------+                  |
|         ^                                                  |
|         | mTLS (deferred S-12b for real mTLS;             |
|         |       reference deployment uses plaintext       |
|         |       loopback in the test config)              |
|         v                                                  |
|  +---------------+    +-----------------+                 |
|  | Slint wizard  |    | Mobile pairing  |                 |
|  | + HIC  UI     |    | (Rust render-   |                 |
|  | (feat-ui)     |    |  model only;    |                 |
|  +---------------+    |  native apps    |                 |
|                       |  deferred)      |                 |
|                       +-----------------+                 |
|                                                            |
+-----------------------------------------------------------+
        |
        | (anchoring; optional)
        v
+--------------------------+
|   Citrate L1 testnet     |
|   chain_id = 40204       |
|   (or any compatible EVM)|
+--------------------------+
```

## Prerequisites

- Linux x86_64 host. The mobile-pairing render model and the
  Slint UI build on macOS / Windows too; the audit-sink WORM
  guarantee depends on POSIX `O_CREAT | O_EXCL` and is
  Linux-tested.
- Rust toolchain per [`BUILD.md`](BUILD.md).
- A POSIX filesystem with ACL + sticky-bit dir support for the
  WORM sink. tmpfs is acceptable for ephemeral test runs.
- For the chain-adapter exercises: an RPC endpoint for any
  EVM-compatible chain. The reference config uses a Citrate
  testnet RPC; substituting a local Anvil / Hardhat instance is
  documented below.

## Reference config

Three operator-side artifacts are needed to bring up the
daemon. Each lives under `./reference-deployment/` in the audit
packet's worktree.

### 1. PolicyBundle (CMMC-L3 baseline)

```sh
# Construct from the overlay builder.
cargo run -p nist-agent-overlays --example build-cmmc-l3-baseline \
    --features build-fixtures \
    > reference-deployment/policy-bundle.cbor
```

The example wires a deterministic SecurityOfficer test key (do
not use in production; this is for hands-on exercise). The
resulting `PolicyBundle` carries:

- Five role assignments (Operator, Reviewer, ComplianceOfficer,
  SecurityOfficer, Auditor) bound to deterministic test DIDs.
- `ActiveOverlays = [CmmcL3]`.
- Retention floor: 6 years (CMMC default).
- Anchor strategy: `Hybrid` (nightly Merkle root + per-install).

### 2. Trust roots

The reference deployment ships a `trust-roots.toml` listing the
test SecurityOfficer Ed25519 public key. Auditors who want to
exercise the "wrong signer is refused" path: edit the file to a
different key and observe the doctor's `Severity::Blocker`
return.

### 3. Release manifest

```sh
# Build a signed release manifest against the local workspace
# binary + a placeholder GGUF.
cargo run -p nist-agent-release --example sign-reference-manifest \
    --features build-fixtures \
    > reference-deployment/release.manifest.toml
```

The example uses the same deterministic test key. Auditors who
want to exercise the install-refusal path: tamper with the
resulting TOML and re-run the installer.

## Bring-up

```sh
# 1. Doctor pre-flight (expect Pass on every check).
cargo run --bin citrate-agent -- doctor \
    --config reference-deployment/config.toml

# 2. Daemon start.
cargo run --bin citrate-agent -- daemon \
    --config reference-deployment/config.toml &

# 3. Wizard (Slint UI; optional, requires feat-ui build).
cargo run --bin citrate-agent --features feat-ui -- wizard

# 4. CLI smoke test.
cargo run --bin citrate-agent -- status
```

The daemon binary aggregates the workspace crates; the actual
binary entry point lands as part of S-12b's CI workflow. For
the audit, exercise the crates individually via the per-crate
examples + the test suite.

## What to exercise

A non-exhaustive list of surfaces an auditor should poke at to
confirm the defenses described in [`THREAT_MODEL.md`](THREAT_MODEL.md):

### HIC queue + Capsule Inspector

```sh
cargo test -p nist-agent-hitl install_gate_blocks_until_every_field_viewed
cargo test -p nist-agent-hitl from_pending_builds_rows_in_order
```

Then manually walk the install gate via the wizard UI (if you
built `feat-ui`) — confirm the Install button is disabled until
every field has been scroll-tracked.

### Overlay activation ratchet

```sh
cargo test -p nist-agent-policy overlay_state
```

Try to construct a bundle that activates `[Ferpa]` without
`CmmcL3`. The `ActiveOverlays::ratchet_into` API should refuse.

### Mobile pairing state machine

```sh
cargo test -p nist-agent-mobile-pairing
```

Walk the happy path + each refusal path. The pairing record
shape mirrors an `AuditRecord` — confirm token clearance on
activation.

### Release verifier

```sh
cargo test -p nist-agent-release
```

Then run the integration test that verifies all 6 runbooks
carry the Rollback section:

```sh
cargo test -p nist-agent-release --test runbook_coverage
```

### WORM audit sink

```sh
cargo test -p nist-agent-audit-sinks
```

Manually attempt to overwrite a written record:

```sh
echo "tamper" > /tmp/worm-test/000001.record  # expect EEXIST
```

## What's NOT in the reference deployment

- Real mTLS for the mobile-pairing endpoint. Deferred to S-12b.
- HSM-backed signing. The reference uses soft Ed25519 test
  keys; production replaces them at the HSM seam (`ReleaseSigner`
  trait, `nist-agent-policy::SignedBundle::sign`).
- Native iOS / Android apps. Sibling repos; ADR-010.
- A reproducibly-built binary tarball. CI infrastructure;
  ADR-011.
- A chain anchor. The reference config has `anchor_strategy =
  "none"` for offline operation; switch to `"hybrid"` and point
  at a real RPC to exercise the chain adapter.

## Tear-down

```sh
pkill citrate-agent
rm -rf reference-deployment/  # if you want to start fresh
```

The WORM audit records remain in `/var/lib/citrate-agent/audit/`
(or wherever the config pointed). Auditors should examine them
post-run; the records cannot be modified, only appended to.

## See also

- [`BUILD.md`](BUILD.md) — toolchain + build steps.
- [`THREAT_MODEL.md`](THREAT_MODEL.md) — the surfaces this
  deployment is designed to let you exercise.
- [`INSTALL.md`](../../INSTALL.md) — operator-facing install
  walkthrough (a softer version of this document for
  non-auditor users).
