# Maps RFC-CIT-AGENT-0001 §4.5, §7.2
# TLA+: CapsuleInstallGate.tla

Feature: Capsule install gate enforces capability and clearance at load
  As an auditor
  I want capability and clearance enforced before any bytecode runs
  So that a misconfigured capsule cannot reach a running state

  Background:
    Given an AgentSBT with on-chain clearance set to "CUI"

  Scenario: The four-step install ladder per RFC §7.2
    Given a capsule manifest with data_class.reads = ["CUI"]
    When install is proposed
    Then in order:
      | step | check                                                                |
      | 1    | read AgentSBT.clearance from chain (or cached if offline)           |
      | 2    | read capsule manifest data_class.reads                              |
      | 3    | verify clearance dominates every entry in reads under the lattice  |
      | 4    | route through HIC gate before adding to AgentSBT installed set    |
    And step ordering MUST be preserved (no parallel resolution)

  Scenario: A read that exceeds clearance is refused both locally and on-chain
    Given a capsule with data_class.reads = ["ITAR"]
    And AgentSBT.clearance = "CUI"
    When install is proposed
    Then the harness refuses with "Bell-LaPadula: no-read-up"
    And the CapsuleRegistry contract also refuses if reached out-of-band

  Scenario: Capability declarations bind the wasmtime linker
    Given a capsule with capability.network = "none"
    When the harness builds the per-capsule linker
    Then the linker host-function table contains NO socket imports
    And attempting to link the capsule with socket imports fails before instantiation

  Scenario: Install AuditRecord captures the full ladder outcome
    When install completes
    Then an AuditRecord of type "CapsuleInstalled" is appended carrying:
      | field                 |
      | capsule.content_hash  |
      | manifest.signing.tier |
      | AgentSBT.clearance    |
      | step_outcomes[1..4]   |

# Sprint: S-2 tla-spec-port
