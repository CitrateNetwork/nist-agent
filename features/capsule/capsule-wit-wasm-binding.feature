# Maps RFC-CIT-AGENT-0001 §4.5

Feature: WIT interface binds the wasmtime linker per-capsule
  As a security engineer
  I want capability enforcement at link time, not runtime
  So that a malicious capsule cannot reach a guarded host function under any input

  Background:
    Given a capsule loaded with a per-capsule wasmtime::Engine

  Scenario: Host functions table is built FROM the manifest, not filtered
    Given a manifest declaring capability.filesystem = ["read:/data/students"]
    When the linker is built
    Then the linker exposes exactly one filesystem host function: read-only over /data/students
    And no other filesystem host functions are present in the table

  Scenario: Mismatched WIT exports fail load
    Given the manifest declares `redact-student-record` is an export
    But the WIT does not declare a matching exported function
    When load is attempted
    Then load fails with "WIT/manifest mismatch: declared export missing"

  Scenario: Argument data-class is checked at call time
    Given a capsule declared data_class.reads = ["FERPA-restricted"]
    When `capsule.call(args)` is invoked with an argument whose data_class is "ITAR"
    Then the call fails with "data-class lattice violation"
    And the violation is recorded as an AuditRecord

# Sprint: consume citrate-agent-runtime
