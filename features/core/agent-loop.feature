# Maps RFC-CIT-AGENT-0001 §3.1, §5.4
# TLA+: ApprovalStateMachine.tla

Feature: Single-loop agent core with state-managed interrupt
  As an operator running nist-agent
  I want a single deterministic agent loop
  So that every proposed action is checkpointable, resumable, and auditable

  Background:
    Given a configured Agent with a Model, a Policy, and an AuditSink
    And an empty ApprovalQueue
    And no pending proposals

  Scenario: Loop streams tokens until a side-effect is proposed
    When the agent receives an input "summarize report.txt"
    And the model produces tokens token-by-token
    Then the loop emits each token to the surface in order
    And the loop does not call any capsule before all tokens are emitted

  Scenario: Side-effect proposal pauses the loop deterministically
    Given a capsule "redact-pii" registered with risk tier "high"
    When the model proposes calling capsule "redact-pii" with args { file: "report.txt" }
    Then the loop checkpoints to ApprovalQueue with the full proposal context
    And the loop returns control to the harness without blocking
    And the proposal hash matches sha256(canonical_cbor(proposal))

  Scenario: Resumption from checkpoint is deterministic
    Given a checkpoint was persisted for proposal hash 0xPP
    When approval payload signed by Reviewer + ComplianceOfficer is returned
    Then the loop resumes from the exact checkpoint
    And the resumed agent produces the same first action it would have produced
    And both the proposal hash 0xPP and the resumption hash 0xRR are recorded in the AuditChain

  Scenario: Loop refuses to execute a side effect without HIC approval
    Given a proposal exists in ApprovalQueue with no signatures
    When some code path attempts to call capsule.execute() directly
    Then the call is refused at compile-time or with a typed runtime error
    And the AuditChain records a "BypassAttempt" event

# Sprint: S-5 agent-loop-and-model-resolver
