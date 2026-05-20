# Maps RFC-CIT-AGENT-0001 §7.3

Feature: On-chain surface contains no operator content
  As a privacy-regulated operator
  I want anchoring on a public chain to remain FERPA / HIPAA / ITAR / IRS-1075 compatible
  So that I can prove "the chain learned that something happened, in what shape, by which class of approver — never what the something was"

  Background:
    Given an active deployment under overlay set including FERPA, HIPAA, ITAR

  Scenario: AuditRecords never appear on chain
    When n records are anchored under any strategy
    Then no chain transaction contains a record's payload bytes

  Scenario: Approver identity is the DID only, never biographic data
    When approver Carol signs an action
    Then on-chain events for that action reference did:citrate:role:0x... derived from her PIV/FIDO key
    And no name, email, or government identifier appears in the on-chain calldata

  Scenario: Org / agent metadata is minimum required for clearance enforcement
    Then the only on-chain org/agent fields are:
      | field                |
      | org_did              |
      | overlay_set          |
      | agent_did            |
      | clearance            |
      | installed_capsules[] |

# Sprint: S-4 evm-chain-adapter-trait
