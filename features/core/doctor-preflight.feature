# Maps RFC-CIT-AGENT-0001 §10.1, §10.2, §10.3

Feature: Doctor pre-flight and continuous monitoring
  As a Compliance Officer
  I want a single command that emits a signed report covering NIST CA-7 and SI-family controls
  So that I have an SSP evidence artifact by construction, not by retrofit

  Background:
    Given a fresh install with a signed PolicyBundle and at least one installed capsule

  Scenario: The eleven checks per RFC §10.2 are exhaustively executed
    When `citrate-agent doctor` is invoked
    Then the report includes exactly these checks:
      | check                                              |
      | capsule-signatures-verify                          |
      | manifest-wit-wasm-capability-match                 |
      | policy-bundle-signed-and-within-validity-window    |
      | audit-chain-intact-from-last-anchor                |
      | approver-hardware-key-bindings-valid               |
      | network-posture-matches-policy                     |
      | model-file-hash-matches-manifest                   |
      | tla-specs-current-for-tier-high-capsules           |
      | role-lattice-fully-assigned                        |
      | approval-queue-within-sla                          |
      | break-glass-affirmation-window-not-exceeded        |

  Scenario: BLOCKER outcome prevents the agent from running
    Given check "model-file-hash-matches-manifest" returns Err
    When the harness next attempts to start
    Then the start is refused with reason "doctor reports BLOCKER: model-file-hash-mismatch"

  Scenario: Signed TOML report and optional on-chain commitment
    When the doctor report is produced
    Then it is signed with the harness ed25519 surface key
    And if policy.doctor.anchor = true, a `DoctorReport` event is emitted on chain carrying sha256(report)

  Scenario: Freshness SLA refuses stale runs
    Given the last doctor report is older than the overlay's freshness SLA
    When the agent attempts to start
    Then it is refused with reason "doctor report stale"

# Sprint: S-7 doctor-checks-6-through-11
