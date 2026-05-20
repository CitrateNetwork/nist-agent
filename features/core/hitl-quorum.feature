# Maps RFC-CIT-AGENT-0001 §5.1, §5.2, §5.3
# TLA+: ApprovalStateMachine.tla

Feature: Tiered-risk plus role-bound quorum gate
  As a Compliance Officer
  I want every action gated by a quorum sized to its risk
  So that AC-3(2), AC-5, AC-6, AU-9(5) are enforced as one mechanism

  Background:
    Given the five base roles are assigned to at least one identity each
    And the risk-tier-to-quorum table is loaded from the active PolicyBundle

  Scenario Outline: Quorum size matches the risk tier
    Given a proposal at risk tier "<tier>"
    When approvers begin signing
    Then the gate clears at exactly "<roles>"
    And no earlier signature subset clears the gate
    Examples:
      | tier     | roles                                            |
      | low      | Operator                                         |
      | medium   | Operator + Reviewer                              |
      | high     | Reviewer + ComplianceOfficer                     |
      | critical | Reviewer + ComplianceOfficer + SecurityOfficer   |

  Scenario: Proposer cannot self-approve (AC-5)
    Given Alice (Operator role) proposed an action at tier "medium"
    When Alice attempts to add her own approval
    Then the approval is rejected with reason "self-approval forbidden"

  Scenario: Auditor role cannot also be an approver (AC-5)
    Given Bob holds Auditor role
    When Bob attempts to sign as a Reviewer on the same action
    Then the signature is rejected with reason "role-conflict: Auditor"

  Scenario: Duplicate signatures from the same identity are rejected
    Given Carol has signed as Reviewer
    When Carol attempts to add a second signature with a different surface
    Then the second signature is rejected as a duplicate

# Sprint: consume citrate-agent-runtime
