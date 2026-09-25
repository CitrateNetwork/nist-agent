---
created: 2026-05-21T00:00:00Z
branch: feat/s-13-tob-engagement
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
audience: Trail of Bits engagement team
---

# Scope — nist-agent v1.0-rc Trail of Bits Audit

> Commit at audit boundary: [`30044d26201daaabdeaaab8d7f25d950b336f023`](https://github.com/CitrateNetwork/nist-agent/commit/30044d26201daaabdeaaab8d7f25d950b336f023) (S-12 close).

## In scope

Every Rust crate in the workspace, every policy/overlay
artifact, and every operator-facing document that constitutes
the deployable product. **All paths are repo-relative.**

### Workspace crates (13)

| Crate | Lines (`*.rs`) | Surface |
|---|---|---|
| `crates/nist-agent-prelude` | 116 | Shared `Overlay` enum + re-exports. |
| `crates/nist-agent-chain` | 750 | `dyn ChainClient` EVM adapter trait + Citrate / generic-EVM bindings. |
| `crates/nist-agent-model` | 631 | Model-backend trait + GGUF loader interface. |
| `crates/nist-agent-loop` | 752 | `Agent::step()` + `resume()` loop + `Checkpoint` store. |
| `crates/nist-agent-policy` | 986 | `PolicyBundle` canonical-CBOR + Ed25519 SecurityOfficer signing + `ActiveOverlays` ratchet. |
| `crates/nist-agent-overlays` | 543 | Per-overlay bundle builders (CMMC-L3 / FERPA / COPPA / CIPA / HIPAA / FedRAMP-High). |
| `crates/nist-agent-doctor` | 715 | 11 doctor pre-flight checks. |
| `crates/nist-agent-audit-sinks` | 245 | WORM filesystem sink (`O_CREAT \| O_EXCL`). |
| `crates/nist-agent-wizard` | 817 | Slint concierge wizard render models + UI scaffold (`feat-ui`). |
| `crates/nist-agent-hitl` | 714 | HIC approval queue + Capsule Inspector render models + Slint scaffold. |
| `crates/nist-agent-marketplace` | 895 | Marketplace browser + chat surface render models + Slint scaffold. |
| `crates/nist-agent-mobile-pairing` | 1021 | Mobile pairing state machine + attestation allowlist + TTL + wire formats. |
| `crates/nist-agent-release` | 1378 | Release manifest + Ed25519 verifier + installer + GGUF SI-7 check + daemon-hash check + egress posture. |
| **Total** | **9563** | |

Approximately 9.6 kLOC of Rust across 13 crates. Roughly
40% of that is test code (see `cargo test --workspace` count
of 202).

### TLA+ specs (`/.agentile/formal/specs/`)

5 normative specs, each with a `.cfg` and BOUNDED_EXPLORATION
parameters:

- `DataClassLattice.tla` — Bell-LaPadula no-read-up / no-write-down.
- `HITLQuorum.tla` — quorum signature collection (Reviewer + ComplianceOfficer + SecurityOfficer).
- `OverlayRatchet.tla` — overlay activation is one-way until decommissioning.
- `AuditChainAppendOnly.tla` — WORM invariant on the audit sink.
- `ReleaseVerifier.tla` — signature-before-extract invariant.

### Gherkin features (`/features/`)

~40 `.feature` files covering operator surfaces, overlays,
distribution, and core invariants. Each feature maps to a
specific RFC section; the mapping is in the file header.

### Compliance docs (`/docs/compliance/<overlay>/`)

6 overlay runbooks (one per Phase-1 overlay) with rollback
procedures, control crosswalks, and deployment artifacts.

### Architecture decisions (`/.agentile/adrs/`)

12 ADRs (001–012). Read ADR-001 first (workspace + runtime
consumption) and ADR-011 (release verifier scope split) for the
load-bearing structural decisions.

## Out of scope

The engagement does **not** cover:

| Out-of-scope item | Reason | Where it does live |
|---|---|---|
| `citrate-agent-runtime` upstream code | Separate audit lineage; consumed via Cargo dep at pinned rev. | `CitrateNetwork/citrate-agent-runtime`, separate engagement. |
| `citrate-chain` smart contracts | Separate audit lineage; chain-side review. | `CitrateNetwork/citrate-chain`, separate engagement. |
| The actual HSM key custody | Procurement + physical security concern. | Deferred to S-12b; ADR-011. |
| GitHub Actions release workflow | Deferred CI infrastructure. | Deferred to S-12b; ADR-011. |
| Native iOS / Android apps | Different toolchains; sibling repos. | `nist-agent-mobile-{ios,android}` (deferred); ADR-010. |
| GGUF binary content | The verifier checks a sha256; the model file itself is an external artifact. | Operator-side; per overlay RUNBOOK. |
| Operator-side deployment runbooks for non-Phase-1 overlays | ITAR / IL4-5 / CJIS / IRS 1075 are post-v1.0. | Future plansets. |
| FFI bindings for the mobile companion | Deferred to native-app repos; not yet authored. | Deferred. |

## Dependency lineage

The workspace pins two cross-repo upstreams via git rev (rather
than crates.io versions) to keep the federation manifest
authoritative:

- `citrate-agent-core` ← `citrate-agent-runtime` @ `2591ed2d28780ca7938befe853be7cd1620029cc`.
- `citrate-wallet-core` ← `citrate-chain` @ `0f2d16b486a9ec1ed8a0394440dac480075d6942`.

The federation's [`drift-check.yml`](https://github.com/CitrateNetwork/citrate-federation/blob/main/.github/workflows/drift-check.yml) enforces that these revs match the federation manifest entries; mismatch fails CI.

Third-party crates worth flagging by audit relevance:

| Crate | Why it matters |
|---|---|
| `ed25519-dalek 2.1` (`std` feature) | Detached-signature verification on `PolicyBundle` + release manifest. |
| `ciborium 0.2` | Canonical CBOR (RFC 8949 §4.2.1) for `PolicyBundle` signing payloads. |
| `sha2 0.10` | SHA-256 for SI-7 model integrity + daemon-hash check + artifact-entry hashes. |
| `tokio 1.40` | Async runtime for the loop / HIC queue. |
| `rustls` (via `reqwest`) | TLS for chain RPC + (future) mTLS for mobile pairing. |
| `toml 0.8` | `release.manifest.toml` decode. |
| `slint 1.13` | UI runtime — feature-gated (`feat-ui`); not on the audit hot path. |

## What we already know

The sprint sequence S-1 through S-12 left a trail of known
sharp edges; flagging them so the engagement isn't surprised:

- **`PairingToken::matches` short-circuits on length** rather
  than running a strictly-constant-time comparison. Documented
  in `crates/nist-agent-mobile-pairing/src/pairing.rs` with the
  reasoning (one token per pairing flow; principally hygiene,
  not side-channel defense). Worth a finding if the threat
  model judges this load-bearing.
- **`SignatureTtl` serializes as `Duration` default form**
  (`{secs, nanos}`) rather than integer seconds. May complicate
  native-app decoder authoring (deferred to S-11b). Not a
  security finding, but worth flagging if the wire form's
  multi-language consumption is in scope.
- **Slint `MarketplacePane` + `ChatPane` emit Window-inheritance
  warnings** when built with `feat-ui`. Cosmetic; integration
  site wraps panes in a parent Window.
- **The `EgressDirective` signature verification is callback-
  injected** rather than baked into `nist-agent-release`. The
  callback's correctness is the operator's responsibility per
  ADR-011; the callback contract is in the egress module's
  doc comment.
- **No FFI surface yet.** All audit-relevant code is pure Rust;
  no `unsafe` in the workspace.

## Reference deployment artifact set

For hands-on exploration, the engagement should build the
reference deployment per [`BUILD.md`](BUILD.md) and stand it up
per [`DEPLOYMENT.md`](DEPLOYMENT.md). The reference deployment
exercises the CMMC-L3 baseline overlay; the other overlays are
configuration-only changes from there.

## See also

- [`INDEX.md`](INDEX.md) — packet cover sheet.
- [`THREAT_MODEL.md`](THREAT_MODEL.md) — per-surface STRIDE.
- [`.agentile/planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md`](../../.agentile/planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md) — the planset this engagement closes.
