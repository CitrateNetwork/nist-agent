# Maps RFC-CIT-AGENT-0001 §3.2

Feature: citrate-agentd — long-running daemon surface
  As an operator
  I want a long-running daemon that other tools can speak to via gRPC
  So that the agent is a shared resource across the org's tooling

  Background:
    Given citrate-agentd is running with a Unix-domain-socket gRPC endpoint

  Scenario: Local UDS gRPC requires operating-system permissions
    When a client connects to the UDS
    Then the kernel-checked uid/gid mapping is required to be in the configured allow set

  Scenario: mTLS endpoint is opt-in for the mobile companion
    Given mTLS is disabled in PolicyBundle
    When a client attempts to connect to the mTLS port
    Then the connection is refused (port closed)

  Scenario: Daemon health and queue depth are observable
    When `gRPC: Health.GetStatus` is called
    Then the response includes queue_depth, oldest_pending_age, doctor_last_run_ts, doctor_last_severity

  Scenario: Daemon refuses to start if doctor reports BLOCKER
    Given the most recent doctor report has severity BLOCKER
    When the daemon attempts to start
    Then it exits with code 5 and writes a refusal AuditRecord

# Sprint: S-12 distribution-and-runbooks
