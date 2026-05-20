# Maps RFC-CIT-AGENT-0001 §2.3

Feature: COPPA overlay for under-13 user data
  As a K-12 operator
  I want capsule reads of under-13 PII gated by parental-consent record
  So that COPPA's verifiable parental consent requirement is enforced by the harness

  Background:
    Given the active overlays include CMMC-L3 + COPPA
    And the data-class lattice recognizes "COPPA-restricted" (under-13 PII)

  Scenario: A read of COPPA-restricted data without a parental-consent token fails
    Given a capsule reads COPPA-restricted data
    And no parental-consent token is supplied
    When invocation is attempted
    Then the harness refuses with "COPPA: parental-consent missing"

  Scenario: Parental-consent token is verifiable
    Given a consent token signed by the org's parental-consent signer
    When the harness validates it
    Then signature MUST chain to the configured signer cert
    And the token MUST not be expired

  Scenario: Emissions involving COPPA data are tier-high regardless of capsule declaration
    Given a capsule reads COPPA-restricted data
    When it proposes any emission
    Then risk-tier is escalated to "high" for that action

# Sprint: S-8 overlay-bundles-a
