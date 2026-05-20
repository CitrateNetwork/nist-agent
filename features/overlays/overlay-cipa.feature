# Maps RFC-CIT-AGENT-0001 §2.3

Feature: CIPA overlay for school internet filtering
  As a school IT lead with E-Rate obligations
  I want every model output checked against an internet-content-policy filter
  So that CIPA's filtering requirement extends to AI-generated content

  Background:
    Given the active overlays include CMMC-L3 + CIPA
    And a CIPA content filter capsule is installed and signed at the managed tier

  Scenario: Every model token stream passes through the CIPA filter capsule
    When the agent loop emits tokens to a student-facing surface
    Then tokens transit the CIPA filter before reaching the surface
    And filter decisions are recorded as audit events of type "ContentFilterDecision"

  Scenario: A blocked emission is logged with rationale, not silently dropped
    When the filter blocks a token sequence
    Then an AuditRecord captures: input hash, filter capsule id, rationale code
    And the user-facing surface receives a clean "blocked by CIPA filter" notice

  Scenario: Doctor reports the CIPA filter's active version
    When `doctor` runs under a CIPA overlay
    Then the report includes the filter capsule's content_hash and signing tier

# Sprint: S-8 overlay-bundles-a
