# Maps RFC-CIT-AGENT-0001 §7.2
# TLA+: DataClassLattice.tla

Feature: Bell-LaPadula data-class lattice across capsule composition
  As an auditor
  I want capsule reads/writes/emits constrained by a lattice
  So that information flow is monotone and non-bypassable

  Background:
    Given the lattice ordering: PUBLIC < CUI < PHI < FERPA < ITAR
    And operator clearance is set per role and per session

  Scenario Outline: Read access requires clearance dominance
    Given Operator clearance = "<clearance>"
    And capsule data_class.reads = "<reads>"
    Then access outcome is "<outcome>"
    Examples:
      | clearance | reads          | outcome   |
      | CUI       | [PUBLIC]       | permitted |
      | CUI       | [PUBLIC, CUI]  | permitted |
      | CUI       | [PHI]          | denied    |
      | PHI       | [CUI, PHI]     | permitted |
      | PHI       | [ITAR]         | denied    |

  Scenario: Output emission cannot label down below input class
    Given a capsule reads "FERPA-restricted" data
    When it attempts to emit class "PUBLIC"
    Then the emission is rejected unless redact-and-attest is invoked via a signed sub-capsule

  Scenario: Composition is transitively safe
    Given capsule A emits class "CUI"
    And capsule B reads class "CUI"
    When A's output is piped to B
    Then no lattice violation occurs
    And if B attempts to emit "PUBLIC" without redact-and-attest, it fails

# Sprint: S-2 tla-spec-port + S-6 policy-bundle-and-data-class-lattice
