# Maps RFC-CIT-AGENT-0001 §8.3

Feature: Capsule Inspector — the AC-6 audit artifact for install
  As an Operator about to install a capsule
  I want every capability and overlay implication surfaced in human-readable form
  So that informed consent is the gate, not the marketplace icon

  Background:
    Given a candidate capsule under review

  Scenario: Inspector surfaces every required field before install
    When the Inspector renders
    Then it shows:
      | field                                                |
      | capsule name, version, content_hash                  |
      | publisher DID and signing tier                       |
      | declared capabilities (network, fs, chain, subagent) |
      | declared data classes (reads, writes, emits)         |
      | risk tier and required approver roles                |
      | certified overlays and explicit not-certified list   |
      | TLA+ spec presence and verification status           |
      | Gherkin pass rate from BenchmarkRegistry             |
      | expandable view of procedure.md                      |

  Scenario: Install button is disabled until the operator scrolls every field into view
    When fields exceed the viewport
    Then the install button is disabled until "fields_viewed = fields_total"

  Scenario: No silent install path exists from any other surface
    Given any other surface (CLI, daemon RPC, mobile)
    When install is requested
    Then the surface still routes through the Inspector for confirmation

# Sprint: S-10 slint-concierge
