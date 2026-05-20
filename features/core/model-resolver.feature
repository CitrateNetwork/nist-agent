# Maps RFC-CIT-AGENT-0001 §3.3, §8.2

Feature: Air-gap-friendly model resolution
  As an operator in a disconnected environment
  I want the harness to find a model locally without ever touching the network
  So that air-gap discipline is preserved by default

  Background:
    Given the active PolicyBundle has egress = disabled

  Scenario Outline: Resolution order is deterministic
    Given availability of model sources is "<sources>"
    When `Model::resolve()` is called
    Then the chosen source is "<chosen>"
    Examples:
      | sources                                                | chosen           |
      | ollama:11434(gemma4:e2b)                               | ollama:11434     |
      | ollama:11434(none), llamacpp:8080(gemma4:e2b)          | llamacpp:8080    |
      | none, embedded(gemma-4-e2b-it-Q4_K_M.gguf)             | embedded         |
      | none                                                   | error("no model available") |

  Scenario: Model file hash is verified before first use
    Given the embedded model is selected
    And the manifest declares sha256 = 0xABCD...
    When the GGUF file's actual sha256 is 0xWXYZ...
    Then `Model::resolve()` returns Err("SI-7: model hash mismatch")
    And no token is ever sampled from the unverified file

  Scenario: Network resolution requires explicit policy + per-pull SO signature
    Given egress is enabled by policy
    And an external HuggingFace pull is requested
    When the request is issued
    Then the harness blocks pending a per-pull SecurityOfficer signature
    And after signing, the pulled artifact's hash is recorded as an AuditRecord

# Sprint: S-5 agent-loop-and-model-resolver
