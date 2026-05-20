# Maps RFC-CIT-AGENT-0001 §8.1, §8.2

Feature: Slint concierge — first-run setup wizard
  As a new operator
  I want a guided concierge that walks me through org identity, role assignments, and overlay selection
  So that first-run setup is auditable and tamper-evident from the first keystroke

  Background:
    Given a fresh install with no prior policy bundle
    And the bundled Gemma 4 E2B GGUF is present and hash-verified

  Scenario: Concierge captures org identity, roles, and hardware keys
    When the operator launches the Slint app for the first time
    Then the concierge guides through:
      | step                         |
      | org identity (DID minting)   |
      | five-role assignment         |
      | hardware key enrollment      |
      | policy bundle selection      |
      | overlay activation           |
    And every step's completion is recorded as an AuditRecord

  Scenario: The bundled model drives the conversation
    When the concierge runs
    Then it loads Gemma 4 E2B via embedded llama-cpp-4
    And no network call is issued during the wizard

  Scenario: Concierge refuses to complete with an unfilled role
    Given the five base roles are not all assigned
    When the operator clicks "Finish"
    Then the concierge refuses and highlights the missing role

# Sprint: S-10 slint-concierge
