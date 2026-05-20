# Maps RFC-CIT-AGENT-0001 §7.1, §6.3

Feature: AnchorRegistry — operator-scoped audit anchors
  As an operator
  I want my audit chain anchored on-chain at operator-chosen cadence
  So that tamper-evidence is third-party-verifiable without exposing record content

  Background:
    Given the AnchorRegistry contract is deployed
    And the operator's AgentSBT exists

  Scenario: Anchor writes a 32-byte root only, never payloads
    When the harness calls anchor(commit_root, strategy_tag)
    Then the on-chain call data carries exactly 32 bytes of root and a strategy tag
    And no plaintext payload bytes are reachable from any chain RPC

  Scenario: is_anchored() query verifies a historical commitment
    Given commit_root 0xRRRR was anchored at block N
    When `is_anchored(0xRRRR)` is queried at block N+10
    Then the result is true

  Scenario: Anchor strategy tag is observable for audit replay
    Given strategies PerCapsule, PerApproval, NightlyMerkle
    When 100 anchors are written under each
    Then off-chain audit tooling can group anchors by strategy via the tag

# Sprint: consume citrate-agent-runtime
