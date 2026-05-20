# Maps RFC-CIT-AGENT-0001 §7.1, §9.2

Feature: BenchmarkRegistry — capsule scenario pass-rate and TLA+ verification status
  As a Reviewer
  I want a capsule's Gherkin pass rate and TLA+ verification status discoverable on-chain
  So that the Slint Capsule Inspector can surface a trust signal before install

  Background:
    Given the BenchmarkRegistry contract is deployed

  Scenario: Posting a verification record requires publisher signature
    When publisher signs a record { capsule_id, gherkin_pass_n, gherkin_total_n, tla_status, evidence_hash }
    Then a `BenchmarkPosted` event is emitted

  Scenario: TLA+ status is one of: not_required, verified, failed, stale
    Then the encoded status enum is exactly { 0:not_required, 1:verified, 2:failed, 3:stale }

  Scenario: Inspector pulls the latest record at install time
    Given a capsule with multiple benchmark posts over time
    When the Slint Capsule Inspector queries the latest
    Then it receives the most-recent record by block number
    And displays gherkin pass-rate and tla_status in the install dialog

# Sprint: link citrate-chain
