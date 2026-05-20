# Maps RFC-CIT-AGENT-0001 §11.1

Feature: Signed reproducible release pipeline
  As a release engineer
  I want every release artifact carry a detached signature from a hardware-backed release key
  So that operators can verify provenance independent of the distribution channel

  Background:
    Given the release signing key is held on a HSM
    And the public verification cert ships with the operator's CitrateNetwork/.github org defaults

  Scenario: A release tag triggers signed artifact production
    Given a git tag matching `v\d+\.\d+\.\d+`
    When the release workflow runs
    Then each artifact in the bundle has a detached signature next to it

  Scenario: Operator install verifies signatures before extracting
    When the operator runs `citrate-agent install <bundle>`
    Then the installer verifies the detached signature against the configured release cert
    And refuses install on failure with reason "release signature invalid"

  Scenario: SBOM accompanies every release
    Then the release bundle includes an SBOM in SPDX or CycloneDX
    And the SBOM is itself signed

# Sprint: S-12 distribution-and-runbooks
