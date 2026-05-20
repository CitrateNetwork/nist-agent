# Maps RFC-CIT-AGENT-0001 §5.6

Feature: Hardware-backed signing surfaces for approvers
  As an operator
  I want each approver signature to come from a hardware-bound key
  So that AC-3(2), IA-5, and overlay-specific identity requirements are satisfied

  Background:
    Given the PolicyBundle declares the permitted surfaces per role

  Scenario Outline: Surface eligibility by overlay
    Given the active overlay set is "<overlay>"
    And the role is "<role>"
    When the approver attempts to sign on surface "<surface>"
    Then the outcome is "<outcome>"
    Examples:
      | overlay      | role             | surface       | outcome  |
      | CMMC-L3      | Operator         | FIDO2         | accepted |
      | CMMC-L3      | ComplianceOfficer| FIDO2         | accepted |
      | FedRAMP-High | ComplianceOfficer| FIDO2         | rejected |
      | FedRAMP-High | ComplianceOfficer| PIV-CAC       | accepted |
      | ITAR         | any              | Mobile        | rejected |
      | FERPA        | Reviewer         | Mobile        | accepted |
      | FERPA        | Reviewer         | OS-Keychain   | rejected |

  Scenario: Mobile signature TTL is shorter than desktop
    Given a FERPA-overlay deployment
    When Reviewer signs from Mobile at time T
    Then the signature expires at T + 1 hour
    And the same Reviewer signing from Slint Desktop expires at T + 24 hours

  Scenario: PIV expiration is caught by doctor pre-flight
    Given a PIV cert binding for SecurityOfficer expires in 3 days
    When `doctor` runs
    Then the report includes a "WARN: piv_cert_expiring_soon" entry

# Sprint: consume citrate-agent-runtime + S-11 mobile-companion
