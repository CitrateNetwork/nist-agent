//! SECREM-02 WP 7.3 tripwires — reproducible, locked, hash-bound
//! release pipeline (FUA-NIST-AGENT-01, -02, -03; 2026-06-09
//! follow-up audit, `per-repo/nist-agent/REPORT.md`).
//!
//! The release pipeline's whole assurance story is "the binary you
//! download is the binary the two-machine reproducibility gate
//! verified, built from the committed `Cargo.lock`". That property
//! lives in `.github/workflows/release.yml` + `Dockerfile.builder` +
//! `scripts/release/build_manifest.py` — none of which the compiler
//! sees. These source-pin tripwires read those files from the repo
//! root and fail the workspace suite if the pipeline regresses:
//!
//! - FUA-01: every `cargo build/test/clippy/run` in the release
//!   workflow carries `--locked` (a stale `Cargo.lock` then fails
//!   the build instead of silently re-resolving), the publish job
//!   `needs:` the reproducibility gate, the binary is built exactly
//!   once (in the gated matrix), and the staged artifact is the
//!   downloaded gate artifact, hash-compared against both machines.
//! - FUA-02: GitHub host keys are pinned literally (Meta API
//!   values), `StrictHostKeyChecking yes`; no `ssh-keyscan` TOFU,
//!   no `accept-new`.
//! - FUA-03: the advisory gate is blocking (no `continue-on-error`
//!   anywhere in the release workflow) and the release path performs
//!   no network `cargo install` — the SBOM tool is pre-cached in
//!   `Dockerfile.builder`.
//!
//! Lives in `nist-agent-release` because this crate owns the release
//! manifest format the pipeline produces (same placement rationale
//! as `runbook_coverage.rs`).

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR points at the crate; the repo root is two
    // levels up (`crates/<name>/Cargo.toml`).
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn read_repo_file(rel: &str) -> String {
    let path = repo_root().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

fn release_workflow() -> String {
    read_repo_file(".github/workflows/release.yml")
}

/// True iff `needle` occurs on an *effective* line of `text` — i.e.
/// not on a YAML / embedded-shell comment line. A `continue-on-error:`
/// key or an `ssh-keyscan` invocation only takes effect on a
/// non-comment line; comments are allowed to *mention* the banned
/// constructs (e.g. to document why they are banned).
fn effective_contains(text: &str, needle: &str) -> bool {
    text.lines()
        .map(str::trim_start)
        .filter(|l| !l.starts_with('#'))
        .any(|l| l.contains(needle))
}

/// FUA-NIST-AGENT-01 (locked closure): every cargo invocation in the
/// release workflow that resolves dependencies must pass `--locked`,
/// so the shipped artifact's dependency closure is exactly the
/// committed `Cargo.lock` — and an out-of-date lockfile fails the
/// release instead of being silently re-resolved.
#[test]
fn every_release_cargo_invocation_is_locked() {
    let wf = release_workflow();
    // Subcommands that resolve/build the dependency graph. `cargo
    // fmt` takes no `--locked`; `cargo audit` reads Cargo.lock
    // directly; `cargo install` is banned outright by
    // `release_path_needs_no_network_tool_install`.
    let must_lock = ["cargo build", "cargo test", "cargo clippy", "cargo run"];
    let mut violations = Vec::new();
    for (idx, line) in wf.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue; // YAML / embedded-shell comment
        }
        for needle in must_lock {
            if trimmed.contains(needle) && !trimmed.contains("--locked") {
                violations.push(format!("release.yml:{}: {}", idx + 1, trimmed));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "release workflow cargo invocations missing --locked \
         (FUA-NIST-AGENT-01):\n{}",
        violations.join("\n")
    );
}

/// FUA-NIST-AGENT-01 (hash binding): the artifact that ships must BE
/// the artifact the reproducibility gate verified — same job
/// artifact, hash-compared, never a third rebuild between the check
/// and the publish.
#[test]
fn shipped_binary_is_the_repro_gated_artifact() {
    let wf = release_workflow();

    // The publish job must depend on the reproducibility gate.
    assert!(
        wf.contains("needs: repro-compare"),
        "release.yml: the build/sign/publish job no longer `needs: \
         repro-compare` — publish is decoupled from the \
         reproducibility gate (FUA-NIST-AGENT-01)"
    );

    // The release binary is built exactly once — inside the gated
    // repro matrix. A second `cargo build --release --bin
    // citrate-agent` means an uncompared third binary ships.
    let builds = wf
        .matches("cargo build --release --bin citrate-agent")
        .count();
    assert_eq!(
        builds, 1,
        "release.yml: expected exactly one `cargo build --release \
         --bin citrate-agent` (the gated matrix build); found \
         {builds}. A rebuild outside the matrix ships an uncompared \
         binary (FUA-NIST-AGENT-01)"
    );

    // The gate's machine-A binary is carried forward as a workflow
    // artifact (uploaded by the matrix, downloaded by the publish
    // job)...
    assert!(
        wf.matches("bin-machine-A").count() >= 2,
        "release.yml: the gated binary artifact `bin-machine-A` is \
         not both uploaded by the repro matrix and downloaded by the \
         publish job (FUA-NIST-AGENT-01)"
    );

    // ...the publish job re-hashes it against BOTH machine hashes
    // before staging...
    assert!(
        wf.contains(r#"[ "$sha_bin" != "$sha_a" ]"#),
        "release.yml: publish job no longer hash-compares the \
         downloaded gated binary against the machine-A/B \
         reproducibility hashes (FUA-NIST-AGENT-01)"
    );

    // ...and the staged binary is that downloaded artifact, not a
    // local target/ build product.
    assert!(
        wf.contains("gated-bin/citrate-agent release-staging/bin/"),
        "release.yml: the staged release binary is not the \
         downloaded reproducibility-gated artifact \
         (FUA-NIST-AGENT-01)"
    );
}

/// FUA-NIST-AGENT-01 (manifest binding): the release manifest builder
/// is told the gate hash and refuses to write a manifest whose staged
/// binary hashes differently.
#[test]
fn manifest_step_binds_the_gated_hash() {
    let wf = release_workflow();
    assert!(
        wf.contains("--expect \"bin/citrate-agent=${{ steps.gate.outputs.gated_sha256 }}\""),
        "release.yml: build_manifest.py is no longer passed \
         `--expect bin/citrate-agent=<gated sha>` — the signed \
         manifest is not bound to the reproducibility-gate hash \
         (FUA-NIST-AGENT-01)"
    );
}

/// FUA-NIST-AGENT-02: the deploy-key SSH trust root is pinned, not
/// TOFU. GitHub's published host keys (Meta API,
/// <https://api.github.com/meta>) are written literally and host-key
/// checking is strict; `ssh-keyscan` and `accept-new` trust whatever
/// answers on the wire during the run.
#[test]
fn github_host_keys_are_pinned_not_tofu() {
    let wf = release_workflow();
    assert!(
        !effective_contains(&wf, "ssh-keyscan"),
        "release.yml: ssh-keyscan reintroduced — host key taken on \
         trust from the network at run time (FUA-NIST-AGENT-02)"
    );
    assert!(
        !effective_contains(&wf, "accept-new"),
        "release.yml: StrictHostKeyChecking accept-new reintroduced \
         — first connection trusts an unverified key \
         (FUA-NIST-AGENT-02)"
    );
    assert!(
        wf.contains("StrictHostKeyChecking yes"),
        "release.yml: StrictHostKeyChecking is not `yes` \
         (FUA-NIST-AGENT-02)"
    );
    // GitHub's published Ed25519 host key — the literal pin.
    assert!(
        wf.contains("AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl"),
        "release.yml: pinned GitHub Ed25519 host key missing from \
         known_hosts (FUA-NIST-AGENT-02)"
    );
}

/// FUA-NIST-AGENT-03 (blocking advisories): no step in the release
/// workflow may be `continue-on-error` — in particular the
/// `cargo audit` gate. Advisories block a tagged release; accepting
/// one requires an explicit ignore in the audit configuration, not a
/// soft-fail step.
#[test]
fn advisory_gate_blocks_release() {
    let wf = release_workflow();
    assert!(
        !effective_contains(&wf, "continue-on-error"),
        "release.yml: a continue-on-error step reappeared — the \
         release can ship past a failing gate (FUA-NIST-AGENT-03)"
    );
}

/// FUA-NIST-AGENT-03 (no network installs): the release path performs
/// no `cargo install` — every tool it needs (cargo-audit, cargo-deny,
/// cargo-cyclonedx) is pre-cached in the hermetic builder image.
#[test]
fn release_path_needs_no_network_tool_install() {
    let wf = release_workflow();
    assert!(
        !effective_contains(&wf, "cargo install"),
        "release.yml: a release-time `cargo install` reappeared — \
         network tool fetch inside the signing pipeline \
         (FUA-NIST-AGENT-03)"
    );
    let dockerfile = read_repo_file("Dockerfile.builder");
    assert!(
        dockerfile.contains("cargo install cargo-cyclonedx --locked"),
        "Dockerfile.builder: cargo-cyclonedx is no longer pre-cached \
         in the builder image — the SBOM step would need a network \
         install at release time (FUA-NIST-AGENT-03)"
    );
}

// ---------------------------------------------------------------
// Behavioral pin: scripts/release/build_manifest.py `--expect`
// hash binding (FUA-NIST-AGENT-01 manifest half).
// ---------------------------------------------------------------

mod build_manifest_expect {
    use super::repo_root;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    /// Fresh scratch dir under target/ (kept out of the source tree;
    /// unique per test so parallel runs don't collide).
    fn scratch(tag: &str) -> PathBuf {
        let dir = repo_root()
            .join("target")
            .join("tmp-release-tripwires")
            .join(format!("{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("staging/bin")).unwrap();
        dir
    }

    fn run_build_manifest(dir: &Path, expect: Option<&str>) -> (bool, String) {
        let script = repo_root().join("scripts/release/build_manifest.py");
        let mut cmd = Command::new("python3");
        cmd.arg(script)
            .args(["--version", "v0.0.0-test"])
            .args(["--git-rev", "deadbeef"])
            .arg("--staging")
            .arg(dir.join("staging"))
            .arg("--out")
            .arg(dir.join("release.manifest.toml"));
        if let Some(e) = expect {
            cmd.args(["--expect", e]);
        }
        let out = cmd
            .output()
            .expect("python3 must be runnable (CI + dev hosts ship it)");
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        (out.status.success(), combined)
    }

    fn sha256_hex(bytes: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        hex::encode(Sha256::digest(bytes))
    }

    #[test]
    fn expect_with_matching_hash_succeeds_and_manifest_carries_it() {
        let dir = scratch("match");
        let body = b"gated binary bytes";
        fs::write(dir.join("staging/bin/citrate-agent"), body).unwrap();
        let sha = sha256_hex(body);

        let (ok, log) = run_build_manifest(&dir, Some(&format!("bin/citrate-agent={sha}")));
        assert!(ok, "matching --expect must succeed; output:\n{log}");

        let manifest = fs::read_to_string(dir.join("release.manifest.toml")).unwrap();
        assert!(
            manifest.contains(&format!("sha256 = \"0x{sha}\"")),
            "manifest must carry the gated hash; got:\n{manifest}"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn expect_with_mismatched_hash_refuses_to_write_a_manifest() {
        let dir = scratch("mismatch");
        fs::write(
            dir.join("staging/bin/citrate-agent"),
            b"tampered after gate",
        )
        .unwrap();
        let wrong = "0".repeat(64);

        let (ok, log) = run_build_manifest(&dir, Some(&format!("bin/citrate-agent={wrong}")));
        assert!(
            !ok,
            "mismatched --expect must fail the manifest build \
             (FUA-NIST-AGENT-01); output:\n{log}"
        );
        assert!(
            !dir.join("release.manifest.toml").exists(),
            "no manifest may be written on a gate-hash mismatch"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn expect_for_a_path_missing_from_staging_fails() {
        let dir = scratch("missing");
        // Staging contains no bin/ at all — the expected artifact
        // never arrived; the manifest must not paper over it.
        let sha = "1".repeat(64);
        let (ok, log) = run_build_manifest(&dir, Some(&format!("bin/citrate-agent={sha}")));
        assert!(
            !ok,
            "--expect for an absent staged path must fail \
             (FUA-NIST-AGENT-01); output:\n{log}"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
