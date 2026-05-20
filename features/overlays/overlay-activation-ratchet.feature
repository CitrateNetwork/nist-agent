# Maps RFC-CIT-AGENT-0001 §2.3

Feature: Overlay activation is a one-way ratchet
  As an auditor
  I want overlays to be add-only within a deployment lifetime
  So that operator-introduced gaps in the evidence chain are impossible

  Background:
    Given an active PolicyBundle with overlays "<active>"

  Scenario Outline: Overlay removal requires a decommissioning workflow
    Given active overlays "<active>"
    When the SecurityOfficer attempts to publish a bundle with overlays "<requested>"
    Then activation outcome is "<outcome>"
    Examples:
      | active                    | requested              | outcome  |
      | CMMC-L3                   | CMMC-L3, FERPA         | accepted |
      | CMMC-L3, FERPA            | CMMC-L3                | rejected |
      | CMMC-L3, FERPA            | (decommission FERPA)   | requires-decommissioning-workflow |

  Scenario: A documented decommissioning workflow is recognized
    Given an open decommissioning workflow file under .agentile/sprints/active/
    And a final audit export for the period during which the overlay applied
    When the SecurityOfficer + ComplianceOfficer co-sign the activation
    Then the overlay is removed from the active bundle
    And a "OverlayDecommissioned" AuditRecord is appended

  Scenario: Removal is itself recorded immutably
    When an overlay is decommissioned
    Then the AuditChain carries the workflow file's sha256
    And the chain anchor for that record references it in the next nightly Merkle root

# Sprint: S-6 policy-bundle-and-data-class-lattice
