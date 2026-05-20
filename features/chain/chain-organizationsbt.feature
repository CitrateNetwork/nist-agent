# Maps RFC-CIT-AGENT-0001 §7.1

Feature: OrganizationSBT — soulbound org identity
  As a federation participant
  I want each deploying organization represented by a non-transferable on-chain identity
  So that org-scoped roles, overlays, and audit anchors have a single anchor point

  Background:
    Given the OrganizationSBT contract is deployed at its canonical address on the configured chain

  Scenario: Minting an OrganizationSBT requires governance signature
    When an org submits an enrollment request
    Then mint requires the 2-of-3 timelocked governance controller signature

  Scenario: The SBT is non-transferable
    When the holder attempts an ERC-721 transferFrom
    Then the call reverts with "OrganizationSBT: soulbound"

  Scenario: The SBT carries minimum metadata only
    Then the on-chain record stores only:
      | field            |
      | org_did          |
      | overlay_set      |
      | enrolled_at      |
      | governance_root  |
    And no biographical identity is stored on-chain

# Sprint: link citrate-chain
