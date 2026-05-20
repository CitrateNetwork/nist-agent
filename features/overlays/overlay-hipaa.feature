# Maps RFC-CIT-AGENT-0001 §2.3

Feature: HIPAA / HITECH overlay for PHI
  As a covered entity or business associate
  I want PHI access gated and minimum-necessary-enforced
  So that the harness produces evidence sufficient for HIPAA audits

  Background:
    Given the active overlays include CMMC-L3 + HIPAA
    And the data-class lattice recognizes "PHI"

  Scenario: PHI reads require ComplianceOfficer + Operator quorum even at tier-low
    Given a capsule reads PHI
    When invocation is attempted at risk.tier "low"
    Then the gate is escalated to require ComplianceOfficer + Operator

  Scenario: Emissions outside the minimum-necessary set are rejected
    Given a capsule reads PHI fields { name, dob, mrn, full_history }
    And the active minimum-necessary policy declares { mrn, dob }
    When the capsule attempts to emit { name, full_history }
    Then the emission is rejected with "minimum-necessary violation"

  Scenario: Breach notification timer starts on any unauthenticated PHI access
    Given an unauthenticated PHI access is detected
    Then a 60-day breach notification countdown is recorded
    And `doctor` enumerates open countdowns until closed

  Scenario: Retention floor is 6 years
    Given an AuditRecord under HIPAA
    Then deletion is refused for at least 6 years from write

# Sprint: S-9 overlay-bundles-b
