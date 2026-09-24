---
created: 2026-05-20T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: completed
sprint: S-3
---

# Sprint S-3 — Retrospective

## Outcome

| Field | Value |
|---|---|
| **Goal achieved?** | YES |
| **WPs planned / closed** | 8 / 7 (3.4 explicitly deferred to S-4) |
| **Carry-forward WPs** | WP-3.4 (nist-agent-chain stub) → S-4 scope |
| **Closing branch** | `main` at commit (this close commit) |

## Metrics delta

| Axis | Start | End | Δ |
|---|---|---|---|
| Tests | 0 | 4 | +4 |
| Formal specs | 0 | 0 | 0 (S-2 territory) |
| CI tripwires | 7 | 7 | 0 |
| Frontmatter coverage | 52/52 (100%) | 53/53 (100%) | maintained |
| Federation drift constraints green | 9/11 | 10/11 | +1 (citrate-agent-core pin) |

Test ratchet baselined at 4. Every future sprint that touches Rust
code must keep `cargo test --workspace 2>&1 | grep 'test result:' |
awk '{s+=$4}END{print s}'` at 4 or higher.

## What worked

- **Skipping the path-dep → git-pin two-stage flip.** The original
  WP-3.5 imagined a path-dep stage for local development, then a
  flip to git-pin at sprint close. In practice, the git-pin worked
  on first try (the SSH alias was pre-configured for runtime/chain
  deploy keys) so we never needed the path-dep intermediate state.
  Saved a half-WP's worth of work.
- **Mirroring runtime's `deny.toml` byte-for-byte plus an `allow-git`
  extension.** Runtime's posture is already audit-reviewed; adopting
  it wholesale removes a degree of freedom from the Trail of Bits
  engagement (S-13).
- **The prelude's smoke tests as a manifest-rev tripwire.** The
  `citrate_agent_core_reexports_are_constructible` test does
  nothing functional, but if a future runtime rev removes one of
  the RFC §3.2 named types, the test will fail at the next
  `cargo build` — surfacing the breaking change at compile time
  instead of at runtime. Cheap insurance.
- **Pairing CI wiring (WP-3.7) with the close commit rather than a
  separate post-sprint PR.** Keeps the manifest-rev → drift-check
  → green-CI feedback loop in one sprint boundary.

## What didn't work

- **Forgetting `serde_json` as a dev-dep** caused one build failure
  cycle. Trivial to fix, but a reminder that smoke tests using JSON
  serde need the dep in `[dev-dependencies]` even though `serde` is
  in `[dependencies]`. A `cargo check --tests` step locally would
  have caught this faster than `cargo test`.
- **`cargo-deny` not being installed locally** meant the
  supply-chain check is a CI-only gate. That's defensible — CI is
  authoritative — but a developer who breaks `deny.toml` finds out
  on the PR rather than at commit time. Worth installing on the
  architect's machine via `cargo install cargo-deny` as a one-time
  setup.
- **The CI workflows reference secrets that don't exist on the
  GitHub repo yet.** First PR will fail at the SSH-config step
  until `AGENT_RUNTIME_DEPLOY_KEY` and `CHAIN_DEPLOY_KEY` are
  configured. Manual operator step; see "Pending CI setup" below.

## What surprised us

- **Cold `cargo build` took 37.77s** — most of it pulling and
  compiling wasmtime + cranelift transitively. Hot rebuild is
  sub-second. The CI cache step is meaningful; without it every PR
  would pay the cold-build cost.
- **`citrate-agent-core` re-exports `wasmtime`** (see
  `runtime/agent/core/src/lib.rs:38`) "so the capsule-dispatch
  consumer can pass typed `Val` args ... without taking a direct
  wasmtime dep." This means our future `nist-agent-chain` and
  `nist-agent-policy` crates pick up wasmtime types via the
  prelude's re-export — we should NEVER add a direct `wasmtime`
  dep to those crates, to avoid version skew. Worth a tripwire
  rule (suggest a Semgrep rule in S-4: forbid `wasmtime =` in any
  `nist-agent-*/Cargo.toml` outside the prelude).

## Lessons for future sprints

1. **The skeleton's `ratchet-check.yml` test-ratchet job is a real
   integration point, not just a Python script wrapper.** Any
   sprint that adds a new language/runtime to the canonical test
   command needs to extend that job with the appropriate toolchain
   setup. S-10 (Slint) and S-11 (mobile) will both touch this.
2. **SSH-aliased git deps imply CI secrets.** Every future
   federation cross-repo dep we adopt needs a corresponding deploy
   key + secret. Worth a section in `.agentile/CONFIG.md` listing
   required secrets so the matrix is discoverable.
3. **Rule-12-first really does work cleanly.** The federation
   `[[drift]]` entries landed at 2026-05-20 morning. By afternoon,
   one of them was already honored by S-3's commit. The other
   stays as a deadline signal for S-4 — exactly what Rule 12 is
   designed to produce.

## Pending CI setup (manual operator step)

The CI workflows need two repo secrets on
`github.com/CitrateNetwork/nist-agent`:

```bash
# From a machine with gh CLI + the deploy keys readable:
gh secret set AGENT_RUNTIME_DEPLOY_KEY \
  -R CitrateNetwork/nist-agent \
  < ~/.config/citrate-split-keys/citrate-agent-runtime-fed-ro

gh secret set CHAIN_DEPLOY_KEY \
  -R CitrateNetwork/nist-agent \
  < ~/.config/citrate-split-keys/citrate-chain-readonly
```

The `-fed-ro` variant of the runtime key is the federation-scoped
readonly key; the chain key is shared across runtime, gui-native,
defense_prime-shell consumers. Both already exist; they just need to be
copied to the new repo as secrets.

## Next sprint(s)

- **S-2 (TLA+ spec port)** stays active in parallel.
- **S-4 (EVM chain adapter trait)** is unblocked. It opens with
  the `nist-agent-chain` crate that satisfies the second federation
  drift constraint.
