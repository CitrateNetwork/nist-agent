# Maps RFC-CIT-AGENT-0001 §5.4
# TLA+: ApprovalStateMachine.tla

Feature: State-managed interrupt for human approval
  As a harness implementer
  I want the agent loop to pause and resume around HIC gates
  So that long-running approvals do not block threads and audits remain deterministic

  Background:
    Given a configured ApprovalQueue with filesystem-backed storage

  Scenario: Pause persists full proposal context
    When a tier-high action is proposed at time T
    Then a checkpoint is written containing:
      | field                | shape                |
      | proposed_call        | { capsule, args }    |
      | capsule_context      | manifest snapshot    |
      | agent_local_state    | serialized           |
      | inputs_that_led_to_it| chat-window prefix   |
    And the harness control returns to its caller within 100 ms

  Scenario: Resume from checkpoint is deterministic across processes
    Given a checkpoint persisted at time T
    When the harness restarts and the approval is signed at time T+1d
    Then resumption produces the identical next action it would have produced at T

  Scenario: Rejection writes an immutable AuditRecord and discards the checkpoint
    When the approval is rejected by ComplianceOfficer
    Then an AuditRecord of type "ProposalRejected" is appended
    And the checkpoint is securely deleted (overwritten then removed)

# Sprint: consume citrate-agent-runtime
