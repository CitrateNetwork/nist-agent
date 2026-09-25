# Maps RFC-CIT-AGENT-0001 §5.6, §8

Feature: Mobile companion — out-of-band signing surface
  As an approver
  I want to clear approvals from my phone when not at my desk
  So that the approval queue does not become a bottleneck without sacrificing key hygiene

  Background:
    Given the mobile app is paired with the daemon via mTLS over the operator's network
    And the active PolicyBundle declares per-overlay mobile signing eligibility

  Scenario: Pairing is HIC-gated
    When the operator initiates a new pairing
    Then the pairing requires SecurityOfficer signature
    And the pairing record is an AuditRecord

  Scenario Outline: Mobile eligibility by overlay
    Given the active overlay is "<overlay>"
    Then mobile signing is "<allowed>"
    Examples:
      | overlay     | allowed |
      | CMMC-L3     | yes     |
      | FERPA       | yes     |
      | HIPAA       | yes     |
      | FedRAMP-High| no      |
      | ITAR        | no      |

  Scenario: Mobile signatures have a 1-hour TTL by default
    When Reviewer signs from Mobile at time T
    Then the signature expires at T + 1 hour

  Scenario: Mobile device attestation chains are policy-allowlisted
    Given allowlist = ["apple-sep", "google-strongbox"]
    When a Samsung Knox attestation is presented
    Then the signing attempt is refused with "device attestation not on allowlist"

# Sprint: S-11 mobile-companion
