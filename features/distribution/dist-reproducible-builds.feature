# Maps RFC-CIT-AGENT-0001 §11.1; capsule manifest [provenance].build_reproducible

Feature: Reproducible builds across surfaces and capsules
  As a procurement officer
  I want every shipped artifact to be byte-reproducible
  So that supply-chain provenance can be re-verified by any downstream auditor

  Background:
    Given a hermetic build environment defined by `rust-toolchain.toml` + `Cargo.lock`

  Scenario: Building the same source on two clean machines yields the same artifact
    When the same git rev is built on machine A and machine B
    Then sha256(artifact_A) == sha256(artifact_B)

  Scenario: Capsule manifests assert reproducibility
    Then every shipped capsule manifest has [provenance].build_reproducible = true
    And the build-script produces an SBOM alongside the .cps

  Scenario: Doctor verifies the running daemon's hash matches the published manifest
    When `doctor` runs against a deployed daemon
    Then the daemon's executable sha256 is compared against the release manifest
    And mismatch is BLOCKER

# Sprint: S-12 distribution-and-runbooks
