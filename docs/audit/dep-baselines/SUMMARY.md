---
created: 2026-05-21T00:00:00Z
branch: feat/s-13-kickoff
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
audience: Trail of Bits engagement team
---

# Dependency baseline — nist-agent v1.0-rc

> Snapshot of the workspace dependency tree at the audit
> boundary commit. Captured 2026-05-21; reproducible with the
> commands in [`../BUILD.md`](../BUILD.md). Re-run before
> handing the bundle to TOB if anything has shifted.

## At a glance

| Metric | Value |
|---|---|
| Audit boundary commit | `30044d26201daaabdeaaab8d7f25d950b336f023` (S-12 close) |
| Workspace crates | 13 |
| `Cargo.lock` dependency count | 982 |
| RustSec advisory DB size | 1096 advisories (loaded 2026-05-20) |
| **`cargo audit`** | 16 vulnerabilities + 6 unmaintained warnings — see breakdown below |
| **`cargo deny advisories`** | clean (different policy than `cargo audit`) |
| **`cargo deny bans`** | FAILED — 44 duplicate-crate warnings (`Cargo.lock`-level diamond) |
| **`cargo deny licenses`** | FAILED — 3 `license-not-encountered` warnings (config drift, not actual license violations) |
| **`cargo deny sources`** | clean |

## What it means

### Vulnerabilities — read carefully before sizing the engagement

All 16 `cargo audit` vulnerabilities are in **transitive
dependencies** pulled in via the upstream crates:

| Crate | Count | Where pulled from | Remediation owner |
|---|---|---|---|
| `wasmtime` | 14 | `citrate-agent-core` (via the WASM capsule sandbox) | Upstream `citrate-agent-runtime` — federation manifest bump |
| `tracing-subscriber 0.2.x` | 1 | Indirect dep; some upstream still pulls 0.2 alongside 0.3 | Investigate at upstream; ANSI-escape log poisoning |
| Unmaintained warnings | 6 (bincode, derivative, fxhash, paste, rand_os) | All transitive | Best-effort upstream upgrades |

**Two of the wasmtime advisories are CVSS 9.0 critical**
(`RUSTSEC-2026-0095` Winch backend sandbox escape and
`RUSTSEC-2026-0096` aarch64 Cranelift sandbox escape). They
do not apply to nist-agent's audit-boundary code paths — the
sidecar does not execute untrusted WASM in the engagement-
relevant code surfaces (WASM capsules are an upstream
runtime concern). Still, the federation manifest pin to
upstream `citrate-agent-runtime` should bump to a wasmtime-
patched rev before v1.0 tag.

**Outcome for the TOB engagement:** these are out-of-scope
findings per [`SCOPE.md`](../SCOPE.md)'s "out of scope" table
(citrate-agent-runtime upstream is a separate audit lineage).
TOB should note them in the report's appendix; remediation
lives upstream.

### `cargo deny bans` — duplicate crates

44 warnings, every one a `Cargo.lock`-level duplicate (e.g.
`autocfg 0.1.8` + `1.5.0`). These are diamond-dependency
artifacts of pulling in two upstream crates with diverging
sub-dep ranges. Cosmetic, not a security finding.

**Outcome:** the deny.toml policy is "warn on duplicates";
the CI gate fails because the policy is configured to fail
on warnings. We can soften the policy to `multiple-versions
= "warn"` if the duplicate-noise becomes load-bearing for
v1.0; for the audit, the duplicates are pinned by `Cargo.lock`
and reproducible.

### `cargo deny licenses` — config drift

3 `license-not-encountered` warnings for licenses listed in
`deny.toml` (`Unicode-DFS-2016`, `OpenSSL`, `ring` exception)
that no current dep uses. These are stale allowances from
the original `deny.toml` (cloned from upstream); cleanup is a
S-13a / S-12b follow-up.

**Outcome:** not a security finding. The actual license
posture is clean — every dep's license is enumerated and
explicitly allowed.

## Files in this directory

- `SUMMARY.md` — this file.
- `cargo-audit.txt` — full `cargo audit` human-readable output
  (582 lines).
- `cargo-audit.json` — same data in JSON for tooling.
- `cargo-deny-summary.txt` — tail of `cargo deny check` showing
  the result line and last warnings.

The full `cargo deny check` output is ~400 KB (every duplicate
crate dumps its full dep tree); we keep the summary form here.
Re-generate the full output with `cargo deny check 2>&1` from
the workspace root if needed for engagement reference.

## Reproducer commands

```sh
# At audit boundary commit.
git checkout 30044d26201daaabdeaaab8d7f25d950b336f023

# Update + run.
cargo install cargo-audit --locked
cargo install cargo-deny --locked
cargo audit
cargo audit --json > docs/audit/dep-baselines/cargo-audit.json
cargo deny check
```

## See also

- [`../SCOPE.md`](../SCOPE.md) — what's in / out (the
  upstream-runtime caveat that applies to the wasmtime
  advisories).
- [`../THREAT_MODEL.md`](../THREAT_MODEL.md) — surfaces in
  scope; the wasmtime sandbox is not on the audit hot path.
- [`../BUILD.md`](../BUILD.md) — toolchain + build steps;
  same commands as above.
