# Maps RFC-CIT-AGENT-0001 §6.1
# TLA+: AuditChainIntegrity.tla

Feature: Hash-chained append-only audit log
  As an auditor
  I want every event recorded as a hash-chained record with signatures
  So that integrity is verifiable offline from genesis to head

  Background:
    Given an empty AuditChain initialized at time T0
    And the genesis record's previous_hash is 32 bytes of zero

  Scenario: Each new record's previous_hash equals sha256 of the prior record
    When n records are appended over time
    Then for every k in [1..n], record[k].previous_hash == sha256(canonical_cbor(record[k-1]))

  Scenario: Canonical CBOR encoding is deterministic
    Given the same logical record encoded twice
    Then the byte output is identical

  Scenario: Tampering with any field invalidates the chain from that point
    Given a chain of 100 records
    When byte 7 of record[42].payload is flipped
    Then `AuditChain::verify_integrity()` returns Err at index 42
    And records 43..100 are reported as orphan

  Scenario: RoleSignature is detached over canonical CBOR of the record
    Given a record with two RoleSignatures
    Then each signature verifies over canonical_cbor(record minus signatures)

# Sprint: consume citrate-agent-runtime
