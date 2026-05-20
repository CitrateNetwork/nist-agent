# Maps RFC-CIT-AGENT-0001 §2.3

Feature: FedRAMP High overlay
  As a federal-system operator
  I want PIV-CAC-only signing and WORM audit storage
  So that the harness operates inside the FedRAMP Rev 5 High baseline

  Background:
    Given the active overlays include CMMC-L3 + FedRAMP-High

  Scenario: FIDO2 signatures are rejected
    When any approver attempts to sign via FIDO2
    Then the signature is rejected with reason "FedRAMP-High: PIV-CAC required"

  Scenario: PIV-CAC signatures verify against the operator's configured CA chain
    When SecurityOfficer signs with a PIV cert
    Then the signature MUST chain to the operator-configured CA root
    And the cert MUST not be revoked per the configured CRL/OCSP source

  Scenario: WORM storage backend is mandatory
    When activation is attempted with local-fs as the AuditSink
    Then activation fails with "FedRAMP-High requires WORM-eligible sink"

  Scenario: Mobile signing is disabled
    Given the active overlay is FedRAMP-High
    When Mobile is enabled in any role's signing-surface list
    Then PolicyBundle activation fails with "FedRAMP-High: Mobile signing forbidden"

# Sprint: S-9 overlay-bundles-b
