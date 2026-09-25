# Maps RFC-CIT-AGENT-0001 §3.2

Feature: citrate-agent-cli — one-shot command-line surface
  As an SRE
  I want a CLI binary that wraps the same library with no IPC
  So that scripted / cron workflows have a deterministic agent invocation

  Background:
    Given the cli binary is built and on PATH

  Scenario: One-shot input produces a single audit-anchored exchange
    When `citrate-agent-cli run --input "ls /data" --capsule list-files`
    Then exactly one Proposal AuditRecord is appended
    And the process exits 0 if approved, 2 if rejected, 3 if timeout

  Scenario: The CLI surface enforces the HIC gate just like Slint
    Given a tier-high action is proposed
    When the CLI runs without `--ci-no-tty`
    Then approval flows are prompted interactively
    With `--ci-no-tty`, the CLI exits 4 ("HIC approval required, no TTY available")

  Scenario: Doctor is a CLI subcommand
    When `citrate-agent-cli doctor` runs
    Then the same signed TOML report as the daemon's doctor is produced

# Sprint: S-12 distribution-and-runbooks
