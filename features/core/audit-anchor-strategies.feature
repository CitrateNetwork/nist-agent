# Maps RFC-CIT-AGENT-0001 §6.3
# TLA+: AuditChainIntegrity.tla

Feature: Selectable on-chain anchoring strategy
  As an operator
  I want to choose the fidelity / cost tradeoff for chain anchoring
  So that high-scrutiny and budget-constrained deployments are both served

  Background:
    Given an AnchorRegistry contract reachable on the configured chain

  Scenario: Strategy A — per-capsule anchors only at install and run-receipt
    Given anchor strategy "A" is active
    When a capsule installs, runs once, and emits an audit record per action
    Then exactly two on-chain events are emitted: CapsuleInstall, RunReceipt
    And no per-action events are emitted

  Scenario: Strategy B — per-approval anchors
    Given anchor strategy "B" is active
    When 5 approvals are recorded in a day
    Then exactly 5 on-chain events are emitted

  Scenario: Strategy C — nightly Merkle root only
    Given anchor strategy "C" is active
    And 1000 audit records are appended on day D
    When the nightly anchor job runs at 00:05 UTC on day D+1
    Then exactly one on-chain event is emitted carrying a Merkle root over the 1000 records
    And every record's Merkle inclusion proof verifies offline

  Scenario: Strategy change is itself an audit event
    When the active strategy moves from C to "C + A for tier-high"
    Then a "StrategyChange" AuditRecord is appended
    And it is dual-signed by SecurityOfficer + ComplianceOfficer

# Sprint: consume citrate-agent-runtime + S-4 evm-chain-adapter-trait
