# Maps RFC-CIT-AGENT-0001 §6.2

Feature: Pluggable audit storage backends
  As an operator
  I want to choose storage that matches my compliance posture
  So that air-gapped, on-prem multi-host, and WORM requirements are all served

  Background:
    Given the AuditSink trait is implemented for four backends:
      | backend         | use_case                                  |
      | local-fs        | default, air-gap-friendly                 |
      | nfs-s3          | on-prem multi-host                        |
      | worm            | FedRAMP High, CJIS — write-once-verified  |
      | chain-anchors   | anchors only, never full records          |

  Scenario: Local filesystem backend respects posix ACLs
    Given local-fs backend is selected
    When a record is written
    Then the file mode is 0600 by default
    And the directory has sticky bit set

  Scenario: WORM backend rejects re-write attempts at storage layer
    Given worm backend is selected
    When the harness attempts to overwrite record N
    Then the backend returns Err("WORM violation") before any I/O occurs
    And no partial state is left on disk

  Scenario: Chain anchors are root hashes only, never plaintext
    Given chain-anchors backend is selected
    When records 1..100 are anchored under Strategy C (nightly Merkle)
    Then the on-chain event contains a single 32-byte Merkle root
    And no payload bytes reach the chain

  Scenario: Storage choice is recorded in the PolicyBundle and audited at change
    Given an active deployment using local-fs
    When the SecurityOfficer + ComplianceOfficer activate a new bundle with worm
    Then a "StorageBackendChange" AuditRecord is appended before any record is written to the new backend

# Sprint: S-9 overlay-bundles-b (WORM + NFS/S3 sinks)
