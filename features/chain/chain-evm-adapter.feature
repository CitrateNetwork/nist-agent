# Maps RFC-CIT-AGENT-0001 §3.2, §11 sidecar-N1
# Owned end-to-end by nist-agent — the sidecar value-add over citrate-agent-runtime.

Feature: Pluggable EVM chain adapter
  As an operator on a non-Citrate EVM chain
  I want to anchor nist-agent's audit trail on my own chain
  So that the sidecar works inside my existing infrastructure without depending on Citrate L1

  Background:
    Given the `trait ChainClient` is defined in nist-agent
    And the default implementation wraps citrate-agent-runtime's AnchorRegistryClient

  Scenario: The default Citrate adapter is selected by absence of configuration
    Given no `[chain]` section in the operator's policy bundle
    When the harness starts
    Then `ChainClient` resolves to the bundled Citrate impl
    And chain_id reports 40204

  Scenario Outline: A generic EVM adapter is selected when configured
    Given the operator's policy bundle declares `[chain] kind = "evm-generic"` with rpc "<rpc>" and chain_id "<chain_id>"
    When the harness starts
    Then `ChainClient` resolves to the generic EVM impl pointed at "<rpc>"
    And reported chain_id is "<chain_id>"
    Examples:
      | rpc                    | chain_id |
      | http://anvil.local:8545 | 31337    |
      | https://corp-evm.example/rpc | 412412 |

  Scenario: All five contracts have configurable addresses in the policy bundle
    Given a generic EVM adapter
    Then the bundle MUST supply addresses for OrganizationSBT, AgentSBT, CapsuleRegistry, AnchorRegistry, BenchmarkRegistry
    And startup fails if any address is missing or is zero

  Scenario: Privacy guarantees hold across all adapters
    Given any selected adapter
    When records are anchored
    Then no payload bytes reach the chain
    And the test corpus from `chain-privacy-guarantees.feature` runs green against every adapter

  Scenario: Adapter selection is itself an audited event
    When the active adapter changes
    Then a `ChainAdapterChange` AuditRecord is appended dual-signed by SecurityOfficer + ComplianceOfficer

# Sprint: S-4 evm-chain-adapter-trait
