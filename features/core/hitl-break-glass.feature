# Maps RFC-CIT-AGENT-0001 §5.5
# TLA+: BreakGlassPath.tla

Feature: Break-glass emergency authorization path
  As a Security Officer responding to an incident
  I want to authorize a tightly-scoped emergency action alone
  So that incident response is not blocked by quorum coordination
  And the action remains fully auditable post-hoc

  Background:
    Given the active PolicyBundle permits break-glass for tier-high actions
    And a capsule "containment-shutdown" has manifest.break_glass_eligible = true

  Scenario: Single-SO break-glass clears the gate immediately
    When SecurityOfficer Dana signs break-glass on a "containment-shutdown" proposal
    Then the proposal clears the HITL gate at signature count 1
    And every approver in the role lattice receives an immediate notification

  Scenario: Post-hoc affirmation closes the loop within 72 hours
    Given a break-glass action was taken at time T
    When Reviewer + ComplianceOfficer affirm within 72 hours
    Then the action is recorded as "BreakGlassAffirmed"

  Scenario: Unaffirmed break-glass is flagged every doctor run
    Given a break-glass action exists at time T
    And 73 hours have passed without affirmation
    When `doctor` runs
    Then the report enumerates the action under "unaffirmed_break_glass"
    And severity is "WARN" (escalates to "BLOCKER" after 7 days per overlay default)

  Scenario: ITAR overlay forbids break-glass for ITAR-classified data
    Given the active PolicyBundle includes overlay "ITAR"
    And the data-class of a proposed action dominates "ITAR"
    When break-glass invocation is attempted
    Then the harness refuses with reason "ITAR forbids break-glass per RFC §5.5"

# Sprint: S-2 tla-spec-port + S-6 policy-bundle-and-data-class-lattice
