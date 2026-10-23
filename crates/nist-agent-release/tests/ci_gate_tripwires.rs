//! Tripwires for CI gates and supply-chain hygiene that `cargo test`
//! would otherwise never exercise.
//!
//! - The Rule-5 no-unwrap gate (`scripts/ci/check_no_unwraps.py`) must
//!   actually scan this workspace's `crates/*/src` layout in full-scan
//!   mode: a bare `.unwrap()` in a fixture production file is flagged,
//!   and one inside an in-file `#[cfg(test)] mod` is not.
//! - The TLA+ workflow fetches a pinned `tla2tools.jar` tag and
//!   checksum-verifies it (never `releases/latest`).
//! - Private-key and token buffers stay on the zeroize path.
//!
//! Fixture trees live under `target/` (never the source tree, never the
//! host), unique per test so parallel runs don't collide.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crate lives at crates/<name>")
        .to_path_buf()
}

fn read_repo_file(rel: &str) -> String {
    fs::read_to_string(repo_root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// Lines that are not YAML/shell comments.
fn effective_lines(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect()
}

// ── Rule-5 no-unwrap gate ───────────────────────────────────────────

/// Build a minimal agentile-equipped project containing a copy of the
/// gate and its helper, plus one fixture crate with `lib_rs` as its
/// `src/lib.rs`. Returns the fixture root.
fn unwrap_gate_fixture(tag: &str, lib_rs: &str) -> PathBuf {
    let dir = repo_root()
        .join("target")
        .join("tmp-ci-gate-tripwires")
        .join(format!("{tag}-{}", std::process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("clear stale fixture");
    }
    fs::create_dir_all(dir.join(".agentile")).expect("mk .agentile");
    fs::create_dir_all(dir.join("scripts/ci")).expect("mk scripts/ci");
    fs::create_dir_all(dir.join("scripts/index")).expect("mk scripts/index");
    fs::create_dir_all(dir.join("crates/demo/src")).expect("mk crate");
    fs::copy(
        repo_root().join("scripts/ci/check_no_unwraps.py"),
        dir.join("scripts/ci/check_no_unwraps.py"),
    )
    .expect("copy gate");
    fs::copy(
        repo_root().join("scripts/index/_common.py"),
        dir.join("scripts/index/_common.py"),
    )
    .expect("copy gate helper");
    fs::write(dir.join("crates/demo/src/lib.rs"), lib_rs).expect("write fixture");
    dir
}

fn run_unwrap_gate(root: &Path) -> (i32, String) {
    let out = Command::new("python3")
        .arg(root.join("scripts/ci/check_no_unwraps.py"))
        .current_dir(root)
        .output()
        .expect("python3 must be runnable (CI + dev hosts ship it)");
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.code().unwrap_or(-1), log)
}

#[test]
fn unwrap_gate_full_scan_flags_prod_unwrap_under_crates() {
    let root = unwrap_gate_fixture(
        "prod",
        "pub fn parse(s: &str) -> u32 {\n    s.parse::<u32>().unwrap()\n}\n",
    );
    let (code, log) = run_unwrap_gate(&root);
    assert_eq!(
        code, 1,
        "full-scan mode must flag a bare .unwrap() in crates/*/src; got {code}: {log}"
    );
    assert!(
        log.contains("crates/demo/src/lib.rs:2"),
        "violation must cite the file: {log}"
    );
    fs::remove_dir_all(&root).expect("clean fixture");
}

#[test]
fn unwrap_gate_ignores_unwrap_inside_in_file_test_module() {
    let root = unwrap_gate_fixture(
        "testmod",
        "pub fn one() -> u32 {\n    1\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {\n        let v: Option<u32> = Some(super::one());\n        assert_eq!(v.unwrap(), 1);\n    }\n}\n",
    );
    let (code, log) = run_unwrap_gate(&root);
    assert_eq!(
        code, 0,
        "test-module unwraps are not production code; got {code}: {log}"
    );
    fs::remove_dir_all(&root).expect("clean fixture");
}

#[test]
fn unwrap_gate_is_clean_on_this_workspace() {
    let out = Command::new("python3")
        .arg(repo_root().join("scripts/ci/check_no_unwraps.py"))
        .current_dir(repo_root())
        .output()
        .expect("python3 must be runnable");
    let log = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "workspace has production unwraps:\n{log}"
    );
}

// ── TLA+ toolchain pin ──────────────────────────────────────────────

#[test]
fn tla_tools_jar_is_tag_pinned_and_checksum_verified() {
    let wf = read_repo_file(".github/workflows/tla-verify.yml");
    let eff = effective_lines(&wf);
    assert!(
        !eff.iter().any(|l| l.contains("releases/latest")),
        "tla2tools.jar must not be fetched from releases/latest"
    );
    let tag_line = eff
        .iter()
        .find(|l| l.trim_start().starts_with("TLA_TOOLS_TAG:"))
        .expect("TLA_TOOLS_TAG must be pinned in env");
    assert!(
        tag_line
            .split(':')
            .nth(1)
            .is_some_and(|v| v.trim().starts_with('v')),
        "TLA_TOOLS_TAG must name a release tag: {tag_line}"
    );
    let sha_line = eff
        .iter()
        .find(|l| l.trim_start().starts_with("TLA_TOOLS_SHA256:"))
        .expect("TLA_TOOLS_SHA256 must be pinned in env");
    let digest = sha_line
        .split(':')
        .nth(1)
        .map(str::trim)
        .unwrap_or_default();
    assert!(
        digest.len() == 64 && digest.chars().all(|c| c.is_ascii_hexdigit()),
        "TLA_TOOLS_SHA256 must be a 64-hex digest: {sha_line}"
    );
    assert!(
        eff.iter()
            .any(|l| l.contains("releases/download/${TLA_TOOLS_TAG}/tla2tools.jar")),
        "download URL must use the pinned tag"
    );
    assert!(
        eff.iter()
            .any(|l| l.contains("${TLA_TOOLS_SHA256}") && l.contains("sha256sum -c")),
        "the fetched jar must be checked against the pinned digest"
    );
}

// ── zeroize path for key / token material ───────────────────────────

#[test]
fn key_and_token_material_stays_on_the_zeroize_path() {
    let ws = read_repo_file("Cargo.toml");
    let dalek = ws
        .lines()
        .find(|l| l.trim_start().starts_with("ed25519-dalek"))
        .expect("ed25519-dalek workspace dep");
    assert!(
        dalek.contains("\"zeroize\""),
        "ed25519-dalek must keep its zeroize feature (SigningKey wiped on drop): {dalek}"
    );

    let evm = read_repo_file("crates/nist-agent-chain/src/evm_generic.rs");
    assert!(
        evm.contains("zeroize::Zeroizing::new(hex::decode(stripped)"),
        "decoded secp256k1 key bytes must be held in a Zeroizing buffer"
    );
    assert!(
        evm.contains("zeroize::Zeroizing::new([0u8; 32])"),
        "the fixed-size key copy must be a Zeroizing buffer"
    );

    let pairing = read_repo_file("crates/nist-agent-mobile-pairing/src/pairing.rs");
    let drop_impl = pairing
        .split("impl Drop for PairingToken")
        .nth(1)
        .expect("PairingToken must implement Drop");
    let body = drop_impl.split("\n}\n").next().unwrap_or_default();
    assert!(
        body.contains("self.0.zeroize()"),
        "PairingToken::drop must zeroize the bearer token"
    );
}
