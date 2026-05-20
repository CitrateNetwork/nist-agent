# Maps RFC-CIT-AGENT-0001 §3.3, G1

Feature: Air-gapped install
  As an operator in a disconnected environment
  I want to install nist-agent from a single signed bundle on offline media
  So that the network posture is "off by default, opt-in by policy"

  Background:
    Given an offline target host with no outbound connectivity

  Scenario: Bundle is self-sufficient for v1.0 baseline overlays
    When the operator copies the signed release bundle onto offline media
    And runs the installer on the target
    Then no network access is required
    And the resulting deployment passes `doctor` end-to-end

  Scenario: First-run wizard works without network
    When the Slint concierge runs on the offline target
    Then it completes the onboarding flow using only the bundled GGUF and local hardware keys

  Scenario: Egress remains disabled after install
    When the harness next starts
    Then egress is disabled
    And any capsule with network capability is greyed-out in the marketplace pane

# Sprint: S-12 distribution-and-runbooks
