# Maps RFC-CIT-AGENT-0001 §7.1, §7.2

Feature: AgentSBT — soulbound agent identity with clearance
  As a Compliance Officer
  I want each agent represented by a soulbound token carrying its data-class clearance
  So that capsule install eligibility is verifiable on-chain

  Background:
    Given an OrganizationSBT exists for the deploying org
    And the AgentSBT contract is deployed

  Scenario: Minting an AgentSBT records clearance and installed capsule set
    When an AgentSBT is minted under an OrganizationSBT
    Then the record stores:
      | field                |
      | agent_did            |
      | org_did              |
      | clearance            |
      | installed_capsules[] |

  Scenario: Clearance is the on-chain anchor for the data-class lattice
    Given AgentSBT.clearance = "CUI"
    When a CapsuleInstall message is sent with manifest reads = ["PHI"]
    Then the CapsuleRegistry contract reverts with "Bell-LaPadula: no-read-up"

  Scenario: Clearance changes require dual on-chain authorization
    When the org governance issues a clearance-bump from "CUI" to "PHI"
    Then the bump requires SecurityOfficer + ComplianceOfficer signatures encoded in the transaction
    And the change is itself an event indexable as `AgentClearanceChanged`

# Sprint: link citrate-chain
