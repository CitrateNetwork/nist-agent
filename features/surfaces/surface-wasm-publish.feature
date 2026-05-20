# Maps RFC-CIT-AGENT-0001 §3.2

Feature: citrate-agent-wasm — Component Model publish target
  As a host application author
  I want to embed the harness as a WebAssembly component
  So that browser-side and embedded-side surfaces can run the same agent

  Background:
    Given the wasm publish target builds reproducibly via `cargo component build --release`

  Scenario: Wasm artifact instantiates under wasmtime with the same WIT contract
    When a host instantiates citrate-agent-wasm
    Then the wit-bindgen contract matches the Rust library's exports
    And the artifact passes the same Gherkin scenarios that the native library passes

  Scenario: Network capability must be host-granted
    Given the wasm artifact's manifest declares network = "none"
    When the host attempts to grant wasi:sockets
    Then the link step refuses

  Scenario: Wasm artifact carries the harness's content_hash in its custom section
    Then `wasmparser` can extract the hash without instantiation
    And the hash matches the published release manifest

# Sprint: S-12 distribution-and-runbooks
