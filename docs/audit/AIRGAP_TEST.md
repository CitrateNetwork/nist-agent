---
created: 2026-05-21T00:00:00Z
branch: feat/s-12b-ci-infra-and-cli
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
audience: Operator validating nist-agent on prem before pilots / TOB engagement
---

# Air-Gap On-Prem Test — nist-agent v1.0-rc

> Step-by-step for validating the workspace end-to-end on an
> air-gapped host. Pre-cache deps once with network, disable
> network, then exercise: build → test → doctor → SI-7 verify
> → embedded inference (optional) → install flow.

## Why this exists

RFC §3.3 ("Network Posture and Air-Gap Discipline") makes
off-by-default the load-bearing posture. Before any pilot
exercises the harness, the operator should confirm:

1. The workspace **builds without network** once deps are
   pre-cached (Rule 1 of operational sanity).
2. The CLI binary boots, runs the 11 pre-flight checks, and
   exits cleanly.
3. The SI-7 GGUF integrity check refuses tampered models with
   the pinned wording.
4. The release-verifier installer refuses bad signatures
   with the pinned wording.
5. (Optional, with `feat-model-llamacpp`) embedded inference
   runs against a real GGUF.

If any of these fail on prem, every downstream consumer
(pilots, TOB, federation pin to `v1.0.0-rc`) is gated until
the failure is fixed.

## Phase 0 — One-time pre-cache (network ON)

Run on a host with internet, on the same OS / architecture as
the target air-gap host. The build cache moves to the target;
nothing else.

```sh
# 1. Clone at the audit-boundary commit.
git clone ssh://git@github.com/CitrateNetwork/nist-agent.git
cd nist-agent
git checkout main  # S-12b close (commit hash printed at sprint close)

# 2. Pre-cache every Cargo dep + the toolchain.
rustup show     # honors rust-toolchain.toml
cargo fetch     # populates ~/.cargo/registry from Cargo.lock

# 3. Build the workspace once to populate target/ + the
#    proc-macro / build-script caches.
cargo build --workspace --release

# 4. Build the CLI bin with embedded inference enabled. This
#    pulls in llama-cpp-2's C++ deps (cmake + clang); the
#    binary itself becomes a single self-contained executable.
cargo build --release --bin citrate-agent --features feat-model-llamacpp

# 5. (Optional) cargo vendor for strict air-gap.
cargo vendor vendor/
# Then add the .cargo/config.toml stanza vendor/ prints.

# 6. Tarball the workspace + cache for transport.
tar -czf nist-agent-airgap-bundle.tar.gz \
    Cargo.toml Cargo.lock rust-toolchain.toml \
    crates/ features/ docs/ scripts/ .agentile/ \
    vendor/ target/release/citrate-agent
```

Move the tarball to the air-gap host via approved media
(USB stick + chain-of-custody, internal mirror, etc.).

## Phase 1 — Bring up the air-gap host (network OFF)

```sh
# Confirm no network reaches the public internet.
ping -c1 -W2 1.1.1.1; echo "above MUST fail with packet loss"

# Extract the bundle.
mkdir -p ~/nist-agent && cd ~/nist-agent
tar -xzf /path/to/nist-agent-airgap-bundle.tar.gz

# If you vendored, point Cargo at it:
mkdir -p .cargo
cat > .cargo/config.toml <<'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

# Sanity check: confirm the CLI binary runs.
./target/release/citrate-agent --version
./target/release/citrate-agent status
```

If `cargo` is not yet installed on the air-gap host, install
the same `rustup` toolchain from a pre-downloaded
`rustup-init` script (out of scope here; standard rustup
docs).

## Phase 2 — Build + test without network

```sh
# Build everything offline.
cargo build --workspace --release --offline

# Run the test suite offline. Expect 200+ tests pass (the
# S-12b ratchet baseline; check .agentile/coverage/baseline.json).
cargo test --workspace --offline

# Lint.
cargo clippy --workspace --all-targets --offline -- -D warnings

# Format check.
cargo fmt --all -- --check
```

Every command above MUST succeed without reaching the network.
If any of them attempts a fetch, the pre-cache was incomplete
— go back to Phase 0.

## Phase 3 — Doctor pre-flight

```sh
./target/release/citrate-agent doctor

# Expect: 5 NIST §10.2 checks reported. With no PolicyBundle /
# model wired, the bundle-dependent checks return Pass with
# "skipped: no raw_bundle in context" — that's correct partial
# behavior. TLA-specs-current should return Pass at ≥ 5 specs.
```

JSON form for machine consumption:

```sh
./target/release/citrate-agent doctor --json | jq .
```

Exit code: `0` = Pass overall, `1` = Warn, `2` = Blocker.

## Phase 4 — SI-7 GGUF integrity (always)

This is the load-bearing check the audit cares about — even if
embedded inference isn't enabled.

```sh
# Generate a test GGUF + matching manifest.
tmp=$(mktemp -d)
echo "GGUF stub payload" > $tmp/model.gguf
sha=$(sha256sum $tmp/model.gguf | awk '{print $1}')

cat > $tmp/release.manifest.toml <<EOF
version = "0.1.0"
git_rev = "airgap-test"

[signature]
algorithm = "ed25519"
public_key_hex = ""
signature_hex = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"

[[artifact]]
kind = "model"
path = "model.gguf"
sha256 = "0x${sha}"
EOF

# Happy path → exit 0.
./target/release/citrate-agent model verify \
    --manifest $tmp/release.manifest.toml \
    --gguf $tmp/model.gguf
echo "exit: $?  # expect 0"

# Tamper → exit 2 with pinned wording.
echo "tampered" > $tmp/model.gguf
./target/release/citrate-agent model verify \
    --manifest $tmp/release.manifest.toml \
    --gguf $tmp/model.gguf
echo "exit: $?  # expect 2"
```

The pinned-wording refusal is `SI-7: model hash mismatch` —
that exact string is what
`dist-bundled-gemma4.feature` scenario "harness refuses to
load a GGUF whose hash doesn't match" requires.

## Phase 5 — Embedded inference (optional, requires `feat-model-llamacpp`)

Only run this phase if you built the CLI with
`--features feat-model-llamacpp` AND you have a real GGUF to
test against. A tiny test GGUF (e.g. a 1B-parameter model
quantized to Q4_K_M, ~600 MB) is enough to confirm the path.

```sh
# Drop a real GGUF here.
real_gguf=~/models/your-real-model.gguf

# Compute its hash + build a manifest.
sha=$(sha256sum "$real_gguf" | awk '{print $1}')
cat > /tmp/real.manifest.toml <<EOF
version = "0.1.0"
git_rev = "airgap-test"

[signature]
algorithm = "ed25519"
public_key_hex = ""
signature_hex = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"

[[artifact]]
kind = "model"
path = "model.gguf"
sha256 = "0x${sha}"
EOF

# Run inference.
./target/release/citrate-agent model infer \
    --manifest /tmp/real.manifest.toml \
    --gguf "$real_gguf" \
    --prompt "Define Bell-LaPadula in one sentence."
```

The SI-7 check runs before load; a tampered GGUF gets refused
before any inference work begins.

Note: the v1 embedded backend is a smoke-test path (greedy
decode up to 128 tokens). The full streaming integration with
the agent loop's HIC gate lands in S-12c.

## Phase 6 — Install flow refusal

Without a real signed bundle, we can still exercise the
refusal path:

```sh
# Dry-run against a bundle dir with a manifest but bogus key.
mkdir -p $tmp/bundle
cp $tmp/release.manifest.toml $tmp/bundle/

./target/release/citrate-agent install \
    --release-pubkey-hex 0000000000000000000000000000000000000000000000000000000000000000 \
    $tmp/bundle \
    --dry-run
echo "exit: $?  # expect 0 (dry-run lists artifacts)"

# Real check refuses because the bundle's signature is zero.
./target/release/citrate-agent install \
    --release-pubkey-hex 0000000000000000000000000000000000000000000000000000000000000000 \
    $tmp/bundle
echo "exit: $?  # expect 2; pinned wording 'release signature invalid'"
```

The pinned-wording refusal is `release signature invalid` per
`dist-signed-releases.feature`'s "refuses install on failure
with reason 'release signature invalid'" scenario.

## Phase 7 — Daemon smoke

The v1 daemon is a no-op skeleton (full event loop is S-12c).
We just confirm it boots cleanly.

```sh
./target/release/citrate-agent daemon --smoke
echo "exit: $?  # expect 0"
```

## Pass criteria (operator sign-off)

The on-prem validation passes when:

- ☐ Phase 2: `cargo build/test/clippy/fmt` all succeed offline.
- ☐ Phase 3: `doctor` exits 0 (or 1 with documented warns).
- ☐ Phase 4: SI-7 verify exits 0 on happy path; exits 2 with
  pinned wording on tamper.
- ☐ Phase 5: (if feature enabled) inference produces non-empty
  output against a real GGUF.
- ☐ Phase 6: install refuses bogus signature with pinned
  wording.
- ☐ Phase 7: daemon `--smoke` exits 0.

A failure on any of these is a S-12b regression. Open a
finding in `.agentile/audits/findings/` (tier per RUBRIC.md)
and remediate before pilots get the bundle.

## See also

- [`BUILD.md`](BUILD.md) — toolchain pins + build commands.
- [`DEPLOYMENT.md`](DEPLOYMENT.md) — reference deployment for
  hands-on exercise (this doc's on-prem narrower cousin).
- [`SCOPE.md`](SCOPE.md) — what's in/out of scope; air-gap
  posture is an in-scope invariant.
- [`HSM_SEAM.md`](HSM_SEAM.md) — how the soft-key release
  signing path swaps in an HSM.
- `crates/nist-agent-cli/src/` — CLI implementation; one
  subcommand per file.
