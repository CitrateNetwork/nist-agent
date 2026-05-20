# Maps RFC-CIT-AGENT-0001 §3.3

Feature: Air-gapped default network posture
  As an operator
  I want zero outbound connectivity until I explicitly enable it
  So that compliance is the default, not the opt-in

  Background:
    Given a fresh install with no prior PolicyBundle

  Scenario: First-run egress is disabled
    When the harness starts
    Then any tool, capsule, or model call requiring network fails with "egress disabled by default"
    And no DNS lookup is issued
    And no TCP connection is opened

  Scenario: Capsule without declared network capability cannot link a socket import
    Given a capsule whose manifest has network = "none"
    And whose WASM imports `wasi:sockets/tcp`
    When the harness attempts to load it
    Then `wasmtime::Linker::instantiate` returns Err at link time
    And no instance is admitted to a running state

  Scenario: Doctor flags any egress without policy
    Given egress is disabled by policy
    When the harness observes any outbound TCP attempt
    Then an AuditRecord of type "EgressAttempt" is appended
    And `doctor` returns BLOCKER on the next run

# Sprint: S-7 doctor-checks-6-through-11
