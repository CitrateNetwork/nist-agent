# Maps RFC-CIT-AGENT-0001 §2.3

Feature: FERPA overlay for school-deployed agents
  As a school district IT lead
  I want FERPA-specific quorum and data-class behavior
  So that student records are protected by harness invariants, not by operator vigilance

  Background:
    Given the active overlays include CMMC-L3 + FERPA
    And the data-class lattice recognizes "FERPA-restricted" and "FERPA-directory"

  Scenario: ComplianceOfficer signature required before any FERPA-restricted emit
    Given a capsule reads FERPA-restricted records
    When it proposes to emit any output
    Then a ComplianceOfficer signature is required regardless of risk tier

  Scenario: FERPA-directory data may emit as PUBLIC after redact-and-attest
    Given a capsule reads FERPA-directory data
    When it invokes a signed redact-and-attest sub-capsule
    Then emission of PUBLIC output is permitted
    And the attestation is recorded as an AuditRecord

  Scenario: Mobile signing is permitted under FERPA with 1-hour TTL
    Given the active overlay is FERPA
    When Reviewer signs from Mobile at time T
    Then the signature expires at T + 1 hour

  Scenario: Retention floor is 5 years
    Given an AuditRecord written under FERPA
    Then deletion is refused for at least 5 years from write

# Sprint: S-8 overlay-bundles-a
