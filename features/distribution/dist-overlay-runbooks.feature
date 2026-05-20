# Maps RFC-CIT-AGENT-0001 §11.1

Feature: Per-overlay deployment runbooks
  As a Compliance Officer for a Phase-1 overlay
  I want a step-by-step runbook for my overlay
  So that deployment is reproducible across operators and reviewable by assessors

  Background:
    Given each Phase-1 overlay has a runbook under `docs/compliance/<overlay>/RUNBOOK.md`

  Scenario Outline: A runbook exists for every Phase-1 overlay
    Then a file exists at "<path>"
    Examples:
      | path                                       |
      | docs/compliance/cmmc-l3/RUNBOOK.md         |
      | docs/compliance/ferpa/RUNBOOK.md           |
      | docs/compliance/coppa/RUNBOOK.md           |
      | docs/compliance/cipa/RUNBOOK.md            |
      | docs/compliance/hipaa/RUNBOOK.md           |
      | docs/compliance/fedramp-high/RUNBOOK.md    |

  Scenario: Each runbook references the corresponding overlay's policy bundle and capsule set
    Then each RUNBOOK.md links to:
      | item                                              |
      | the overlay's signed PolicyBundle artifact        |
      | the capsule list with content_hash for each       |
      | the AnchorRegistry / WORM storage configuration   |
      | the doctor checks that MUST pass post-install     |

  Scenario: Runbooks include rollback procedures
    Then each runbook includes a "Rollback" section with:
      | step                                              |
      | revert PolicyBundle (record the AuditRecord)      |
      | un-install added capsules (HITL-gated)            |
      | final audit export of the period                  |

# Sprint: S-12 distribution-and-runbooks
