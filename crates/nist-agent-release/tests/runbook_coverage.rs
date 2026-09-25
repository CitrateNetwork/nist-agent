//! Pins `dist-overlay-runbooks.feature`:
//!
//! - "A runbook exists for every Phase-1 overlay" — six paths.
//! - "Runbooks include rollback procedures" — every runbook has a
//!   `## Rollback` section.
//!
//! Lives in `nist-agent-release` because the runbooks are part of
//! the release bundle (`ArtifactKind::Runbook`). If a future
//! refactor moves a runbook, this test catches it before the
//! release manifest goes out the door.

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

const OVERLAY_RUNBOOKS: &[&str] = &[
    "docs/compliance/cmmc-l3/RUNBOOK.md",
    "docs/compliance/ferpa/RUNBOOK.md",
    "docs/compliance/coppa/RUNBOOK.md",
    "docs/compliance/cipa/RUNBOOK.md",
    "docs/compliance/hipaa/RUNBOOK.md",
    "docs/compliance/fedramp-high/RUNBOOK.md",
];

#[test]
fn every_phase1_overlay_has_a_runbook() {
    let root = repo_root();
    for rel in OVERLAY_RUNBOOKS {
        let p = root.join(rel);
        assert!(p.exists(), "missing runbook: {}", p.display());
    }
}

#[test]
fn every_runbook_has_a_rollback_section() {
    // RFC §11.1 + dist-overlay-runbooks.feature scenario
    // "Runbooks include rollback procedures": every runbook MUST
    // carry a `## Rollback` section with the three documented
    // steps (PolicyBundle revert, HIC-gated capsule uninstall,
    // final audit export).
    let root = repo_root();
    for rel in OVERLAY_RUNBOOKS {
        let p = root.join(rel);
        let body = fs::read_to_string(&p).unwrap_or_else(|_| panic!("read {}", p.display()));
        assert!(
            body.contains("## Rollback"),
            "{}: missing `## Rollback` section",
            p.display(),
        );
        // The three rollback items are content-bearing — confirm
        // the canonical phrases appear so a future edit can't
        // accidentally remove the load-bearing steps without
        // tripping a test.
        assert!(
            body.contains("PolicyBundle") || body.contains("policy bundle"),
            "{}: rollback should reference the PolicyBundle revert step",
            p.display(),
        );
        assert!(
            body.contains("HIC"),
            "{}: rollback should reference the HIC-gated uninstall step",
            p.display(),
        );
        assert!(
            body.to_lowercase().contains("audit"),
            "{}: rollback should reference the final audit export step",
            p.display(),
        );
    }
}

#[test]
fn every_runbook_references_supporting_artifacts() {
    // Scenario "Each runbook references the corresponding
    // overlay's policy bundle and capsule set" — confirm the
    // four canonical anchors appear at least once per runbook.
    let root = repo_root();
    for rel in OVERLAY_RUNBOOKS {
        let p = root.join(rel);
        let body = fs::read_to_string(&p).unwrap();
        let lower = body.to_lowercase();
        for needle in [
            "policybundle", // PolicyBundle reference
            "capsule", // capsule set (per-capsule content_hash invariant enforced in features/)
            "doctor",  // doctor checks
        ] {
            assert!(
                lower.contains(needle),
                "{}: missing reference to '{}'",
                p.display(),
                needle,
            );
        }
        // The feature scenario's "AnchorRegistry / WORM storage
        // configuration" row is satisfied by either keyword;
        // overlay runbooks typically inherit the CMMC baseline's
        // anchor strategy and reference WORM directly.
        assert!(
            lower.contains("anchor") || lower.contains("worm"),
            "{}: missing anchor/WORM storage reference",
            p.display(),
        );
    }
}
