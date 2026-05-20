# Maps RFC-CIT-AGENT-0001 §4.3

Feature: Capsule manifest is the policy source of truth
  As a harness implementer
  I want every runtime policy decision sourced from the signed manifest
  So that the WIT, the WASM, and procedure.md cannot lie

  Background:
    Given a capsule manifest with sections [capsule] [capability] [data_class] [risk] [overlay] [procedure] [provenance] [signing]

  Scenario: Policy decisions read only the manifest
    Given a capsule call is proposed
    When the harness determines required quorum, data-class checks, and overlay eligibility
    Then no decision reads bytes from capsule.wasm
    And no decision reads procedure.md
    And no decision reads capsule.wit content beyond cross-validation against the manifest

  Scenario Outline: Manifest cross-validation against WIT and WASM at load
    Given manifest capability.network = "<declared>"
    And the WIT imports "<wit_import>"
    Then load outcome is "<outcome>"
    Examples:
      | declared      | wit_import         | outcome  |
      | none          | (no socket import) | accepted |
      | none          | wasi:sockets/tcp   | rejected |
      | egress-allowed| wasi:sockets/tcp   | accepted |
      | broker-only   | wasi:sockets/tcp   | rejected |

  Scenario: Required fields are non-optional
    Given a manifest missing [capsule].content_hash
    When loaded
    Then load fails with "manifest schema error: content_hash required"

# Sprint: consume citrate-agent-runtime
