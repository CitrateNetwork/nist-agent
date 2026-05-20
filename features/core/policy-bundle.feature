# Maps RFC-CIT-AGENT-0001 §2.3, §3.1
# TLA+: DataClassLattice.tla

Feature: Signed PolicyBundle with overlay activation
  As a Security Officer
  I want the operator policy expressed as a single signed CBOR object
  So that overlay activation, risk-tier mapping, and role lattice are tamper-evident

  Background:
    Given a five-role lattice "Operator, Reviewer, ComplianceOfficer, SecurityOfficer, Auditor"
    And a data-class lattice "PUBLIC < CUI < PHI < FERPA < ITAR"

  Scenario: PolicyBundle parses canonical CBOR and verifies SO signature
    When the harness loads policy-bundle.cbor
    Then the bundle parses as canonical CBOR per RFC 8949 §4.2.1
    And the SecurityOfficer ed25519 signature verifies over the canonical encoding

  Scenario Outline: Overlay activation is a one-way ratchet
    Given the active bundle has overlays "<active>"
    When the Security Officer publishes a new bundle with overlays "<requested>"
    Then activation outcome is "<outcome>"
    Examples:
      | active            | requested                     | outcome  |
      | CMMC-L3           | CMMC-L3, FERPA                | accepted |
      | CMMC-L3, FERPA    | CMMC-L3                       | rejected |
      | CMMC-L3, HIPAA    | CMMC-L3, HIPAA, FedRAMP-High  | accepted |

  Scenario: Risk-tier mapping cannot de-escalate per-capsule tier
    Given a capsule with manifest risk.tier = "high"
    When the policy bundle attempts to map this capsule to "medium"
    Then the bundle fails verification
    And the harness retains the previous bundle

  Scenario: Bundle rotation is recorded as an audit event
    When a new PolicyBundle is activated
    Then an AuditRecord of type "PolicyChange" is appended
    And the record is signed by SecurityOfficer + ComplianceOfficer (dual auth per AU-9(5))

# Sprint: S-6 policy-bundle-and-data-class-lattice
