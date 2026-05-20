# Maps RFC-CIT-AGENT-0001 §8.2

Feature: Bundled Gemma 4 E2B GGUF as separately hashable artifact
  As a Security Officer
  I want the bundled concierge model to be a separately-hashable artifact
  So that NIST SI-7 model integrity can be verified without binary surgery

  Background:
    Given the release bundle ships `gemma-4-e2b-it-Q4_K_M.gguf` alongside the daemon

  Scenario: The release manifest declares the GGUF's sha256
    Then `release.manifest.toml` carries `[model] sha256 = "0x..."` for the GGUF

  Scenario: The harness refuses to load a GGUF whose hash doesn't match
    Given the GGUF is replaced on disk by a tampered copy
    When the harness attempts to load it
    Then load fails with Err("SI-7: model hash mismatch")

  Scenario: License terms are bundled with the model
    Then the release bundle includes the Gemma 4 license file alongside the GGUF

# Sprint: S-12 distribution-and-runbooks
