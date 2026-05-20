# Maps RFC-CIT-AGENT-0001 §2.1, §2.2

Feature: CMMC Level 3 baseline overlay
  As a DIB contractor handling CUI
  I want the harness to enforce the CMMC L3 control set out of the box
  So that DCMA-DIBCAC assessment is satisfiable from doctor reports alone

  Background:
    Given the active PolicyBundle includes overlay "CMMC-L3"
    And the AC, AU, IA, SI control families are mapped to harness components per RFC §2.2

  Scenario: AC-3(2) dual authorization fires on every privileged command
    Given a capsule with risk.tier ≥ "medium"
    When a privileged action is proposed
    Then the HITL quorum requires at least two distinct authorized identities

  Scenario: AU-9(5) dual authorization protects audit-log modifications
    When deletion of an expired AuditRecord is attempted
    Then SecurityOfficer + ComplianceOfficer signatures are required

  Scenario: SI-7 model file integrity is verified at every start
    Given the bundled model GGUF has a manifest-declared sha256
    When the harness starts
    Then the actual sha256 is recomputed and compared
    And mismatch is a BLOCKER

  Scenario: CA-7 continuous monitoring artifact is produced daily
    Then `doctor` runs on a 24h cadence (per CONFIG.md)
    And each report is appended to the AuditChain and optionally anchored

# Sprint: S-8 overlay-bundles-a
