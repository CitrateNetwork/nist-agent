# Maps RFC-CIT-AGENT-0001 §4.1, §4.2

Feature: Content-addressed Capsule bundle composition
  As a capsule author
  I want my capability packaged as a single .cps bundle
  So that distribution, signing, and HIC gating all operate on one indivisible unit

  Background:
    Given the .cps extension is reserved for capsule archives

  Scenario: A valid .cps contains the canonical file set at fixed paths
    Given a bundle "ferpa-redact-student-record-0.3.1.cps"
    When the loader inspects it
    Then the following paths exist:
      | path                          |
      | capsule.wit                   |
      | capsule.wasm                  |
      | procedure.md                  |
      | manifest.toml                 |
      | gherkin/                      |
      | SIGNATURES/publisher.sig      |

  Scenario: A bundle missing manifest.toml is refused
    Given a bundle with no manifest.toml
    When the loader inspects it
    Then `Capsule::from_archive` returns Err("missing manifest.toml")

  Scenario: A tier-high capsule MUST carry a TLA+ spec in specs/
    Given a manifest with risk.tier = "high"
    And no `specs/` directory in the bundle
    When the loader inspects it
    Then `Capsule::from_archive` returns Err("tier-high requires specs/*.tla")

  Scenario: A capsule's content_hash is computed deterministically
    Given the same canonical file set produced by two reproducible builds
    Then the manifest.content_hash matches between builds

# Sprint: consume citrate-agent-runtime
