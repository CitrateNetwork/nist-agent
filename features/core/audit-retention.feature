# Maps RFC-CIT-AGENT-0001 §6.4

Feature: Overlay-driven audit record retention
  As a Compliance Officer
  I want each overlay to dictate its own retention period
  So that retention is enforced by the harness, not by operator memory

  Background:
    Given the active PolicyBundle declares retention per overlay

  Scenario Outline: Overlay sets the floor; harness refuses early deletion
    Given an AuditRecord written under overlay "<overlay>" at time T
    When deletion is attempted at time T + "<offset>"
    Then the deletion is "<outcome>"
    Examples:
      | overlay        | offset       | outcome  |
      | CMMC-L3        | 6 years - 1d | rejected |
      | CMMC-L3        | 6 years + 1d | permitted-with-dual-auth |
      | HIPAA          | 6 years - 1d | rejected |
      | FedRAMP-High   | 3 years - 1d | rejected |
      | FERPA          | 5 years - 1d | rejected |

  Scenario: Post-expiration deletion is itself a dual-authorized audit event
    Given an AuditRecord past its retention floor
    When deletion is initiated by Operator and signed by SecurityOfficer + ComplianceOfficer
    Then a "RecordDeleted" AuditRecord is appended carrying the deleted record's sha256

  Scenario: Retention period reduction requires planset migration
    When the SecurityOfficer attempts to shorten retention for FERPA from 5y to 3y
    Then the PolicyBundle activation fails
    And the harness reports "retention reduction requires planset migration"

# Sprint: S-8 overlay-bundles-a + S-9 overlay-bundles-b
