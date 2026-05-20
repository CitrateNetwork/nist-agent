# Maps RFC-CIT-AGENT-0001 §4.4

Feature: Bundled / Managed / Workspace signing tiers
  As an operator
  I want each capsule classified by trust tier
  So that overlay eligibility and load-time checks scale with provenance

  Background:
    Given the harness ships with the bundled-tier publisher root certificate

  Scenario Outline: Eligibility under each overlay by signing tier
    Given a capsule with signing.tier = "<tier>"
    And the active overlay is "<overlay>"
    When install is attempted
    Then outcome is "<outcome>"
    Examples:
      | tier      | overlay      | outcome  |
      | bundled   | CMMC-L3      | accepted |
      | managed   | CMMC-L3      | accepted |
      | workspace | CMMC-L3      | accepted-with-warning |
      | workspace | FedRAMP-High | rejected |
      | workspace | ITAR         | rejected |

  Scenario: Bundled tier requires Citrate-signed root
    Given a capsule claims signing.tier = "bundled"
    But the publisher signature does not chain to the bundled-root cert
    When load is attempted
    Then load fails with "bundled tier signature does not chain to root"

  Scenario: Managed tier requires org-procurement signature
    Given a capsule claims signing.tier = "managed"
    And the operator has not enrolled the procurement signer
    When load is attempted
    Then load fails with "managed tier requires enrolled org signer"

  Scenario: Workspace tier requires operator's own key
    Given workspace tier
    When load is attempted by Operator Alice
    Then the signature MUST verify against an Operator-role key
    And the install AuditRecord notes the tier as "workspace"

# Sprint: consume citrate-agent-runtime
