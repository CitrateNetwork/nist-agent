---
created: 2026-06-11
branch: audit/secrem02-nist-agent-pass1
author: Fable 5 (Claude Code) subagent
sprint: SECREM-02-followup-remediation
repo: nist-agent
baseline_test_count: 234
status: active
---

# SECREM-02 WP 6.1 — nist-agent first remediation pass

Remediates all ten findings from the 2026-05-31 inaugural federation
deep audit (`citrate-security/audits/2026-05-31-federation-deep-audit`),
re-confirmed STILL-OPEN at rev `773606b` by the 2026-06-09 follow-up
audit (`per-repo/nist-agent/REPORT.md`). Decision gate: GO — the WORM
audit sink backs the federation's AU-immutability compliance claim, so
these findings sit on a compliance-claim path.

Protocol: baseline suite → re-verify each finding in source → RED
adversarial tests (confirmed failing) → fail-closed fixes → suite
green (count ≥ baseline) → fmt + clippy (`-D warnings`, CI parity) →
mutation pass (each new check reverted in place; new tests confirmed
to fail; restored).

- Baseline: **234 passed / 0 failed** (30 suites) at `e207300`.
- Final: **266 passed / 0 failed** (32 suites). +32 tests.
- `cargo fmt --all --check` clean; `cargo clippy --workspace
  --all-targets --locked -- -D warnings` clean (CI gates on both).

## Findings table

| Finding | Sev | Red test(s) | Fix (file) | Suite | Mutation | Disposition + Notes |
|---|---|---|---|---|---|---|
| NIST_AGENT-2026-05-31-001 — daemon never loads/verifies PolicyBundle | HIGH | `tests/policy_enforcement.rs` (daemon crate): refuses missing bundle / untrusted signer / garbage pubkey / tampered bytes; valid bundle held in state. 4 behavioral RED failures confirmed pre-fix | `crates/nist-agent-daemon/src/policy.rs` (new startup gate: read wizard-JSON or CBOR `RawSignedBundle`, `verify_and_decode` vs SO trust root, `activate(now)`), wired in `daemon.rs::prepare` (fail-closed), bundle held in `DaemonState.policy`; new `DaemonError::PolicyRejected` | green | `load_policy` call replaced with `None` → 5 tests FAIL; restored | **FIXED (core)**. Remainder DEFERRED-WITH-OWNER (owner: nist-agent maintainer, target: SECREM-02 Phase 7): gating future egress/HITL/anchor *operations* through the held bundle — the v1 daemon exposes only read-only IPC (`Status`/`QueueDepth`/`RecentAudit`), so there is no egress-bearing operation to gate yet; the bundle is now verified, activated, and held for exactly that wiring |
| NIST_AGENT-2026-05-31-002 — SI-7 gate TOCTOU (hash bytes ≠ loaded bytes) | HIGH | `embedded.rs::si7_swap_after_verify_cannot_change_loaded_bytes` (swap file in check-to-use window; loader-visible bytes must equal verified bytes). Behavioral RED failure confirmed | `crates/nist-agent-model/src/embedded.rs`: new `load_verified_bytes(verified_bytes, source)` stages exact verified bytes into a private 0700 tempdir (copy 0400) held for backend lifetime; llama.cpp opens the staged copy, never the operator path; `load_verified(path)` now reads once + delegates. `crates/nist-agent-cli/src/commands/model.rs::run_infer` passes the same buffer it hashed. Binding pin: `load_verified_bytes_binds_hashed_bytes_to_loaded_bytes` | green | `model_path` mutated back to operator path → si7 test FAILS; restored | **FIXED**. Cost: one staged copy of the GGUF per inference run (correctness over disk). Doctor `ModelHashCheck` / installer variants covered by -006 fix + existing digest APIs |
| NIST_AGENT-2026-05-31-003 — unauthenticated local IPC socket | MED | `daemon.rs::bound_socket_mode_excludes_group_and_other` (behavioral RED failure confirmed: umask-default mode); `peer_gate_refuses_foreign_uid` (API-pin RED: helper absent → compile fail) | `crates/nist-agent-daemon/src/daemon.rs`: socket chmod 0600 after bind (fail-closed on chmod error); `peer_authorized` gate (own euid or root) checked via `UnixStream::peer_cred` BEFORE any request byte; platform refusing peer-cred disclosure → refused | green | (a) chmod removed → mode test FAILS; (b) gate forced `true` → peer test FAILS; restored both | **FIXED**. Cross-uid connect e2e not testable without root; gate unit-pinned + same-uid round-trip regression tests stay green |
| NIST_AGENT-2026-05-31-004 — `apply_directive` verifier-injectable, no replay binding | MED | `tests/egress_directive.rs` (release crate): garbage sig / tampered reason / wrong trust root / replayed nonce / stale nonce / expired directive all refused; `injectable_verifier_removed_from_api` source pin. API-pin RED: new signature → compile fail confirmed pre-fix | `crates/nist-agent-release/src/egress.rs`: `EgressDirective` gains `nonce` (strictly increasing) + `expires_at_unix`; canonical length-prefixed `signing_payload()`; Ed25519 verified in-crate; `apply_directive(self, directive, so_pubkey, last_consumed_nonce, now_unix) -> (posture, consumed_nonce)`; `FnOnce` injection removed; `disable()` added (fail-closed free). New `ReleaseError::EgressDirectiveReplayed` | green | verify+expiry+nonce checks skipped → 6 tests FAIL; restored | **FIXED**. Caller persists consumed nonce (API forces it). Breaking API change; sole pre-fix callers were tests. Mobile `SignedDecision` replay-within-TTL variant is out of this crate — noted for Phase 7 |
| NIST_AGENT-2026-05-31-005 — WORM sink doesn't fsync directory entry | MED | `worm.rs::append_fsyncs_the_directory_entry` — source pin (append must call `self.sync_parent_dir()`) + behavioral check helper succeeds on live sink. RED: compile fail (helper absent) confirmed | `crates/nist-agent-audit-sinks/src/worm.rs`: `sync_parent_dir()` (open dir fd + `sync_all`) called after `file.sync_all()`; stale comment corrected | green | `sync_parent_dir()` call removed → pin test FAILS; restored | **FIXED**. This is the finding that directly backs the AU-immutability/durability compliance claim. Crash-injection harness not portable in unit tests → source pin chosen per WP protocol. OS-level immutability (`chattr +i`/object-lock) remains a deployment-doc item (S-13) |
| NIST_AGENT-2026-05-31-006 — install hashes manifest-controlled paths pre-signature; path escape | MED | `install.rs` tests: `bad_signature_aborts_before_any_artifact_io`, `traversal_artifact_path_refused_even_when_signed`, `absolute_artifact_path_refused_even_when_signed` (+ `safe_bundle_path` unit pin). 3 behavioral RED failures confirmed (traversal/absolute previously exited 0!) | `crates/nist-agent-cli/src/commands/install.rs`: `verifier.verify(&manifest)` now runs BEFORE any artifact IO (pinned refusal wording kept); `safe_bundle_path()` refuses absolute paths and `..`/root/prefix components before joining | green | (a) early verify removed → bad-sig test FAILS; (b) `safe_bundle_path` accept-all → 3 tests FAIL; restored | **FIXED**. Library `Installer` already ordered signature-first; defect was the CLI wrapper |
| NIST_AGENT-2026-05-31-007 — placeholder mainnet addresses, no ship-guard | LOW | `citrate.rs::prod_ctor_refuses_placeholder_mainnet_addresses` (behavioral RED failure confirmed); `types.rs` placeholder-flag pins | `crates/nist-agent-chain/src/types.rs`: `ContractAddresses::is_placeholder()`; `crates/nist-agent-chain/src/citrate.rs`: `from_hex_key` refuses placeholder set (returns `None`); test fixture constructs directly | green | guard removed → test FAILS; restored | **FIXED**. Prod ctor is intentionally unusable until citrate-chain v1 addresses land; `citrate_mainnet()` stays the single update point |
| NIST_AGENT-2026-05-31-008 — IPC `to_line` panics via `.expect()` | LOW | `ipc.rs::wire_encoding_has_no_panic_path` (source pin, needle assembled) + `response_serialize_fallback_decodes_as_error`. RED: pin failed + helper absent confirmed | `crates/nist-agent-daemon/src/ipc.rs`: both `to_line`s now `unwrap_or_else`; response falls back to `serialize_error_line()` (always-valid `Error` JSON via `serde_json::Value::to_string`); request falls back to a line the daemon refuses | green | `.expect()` restored on response path → pin test FAILS; restored | **FIXED** |
| NIST_AGENT-2026-05-31-009 — two divergent `EgressPosture` enums | LOW | `tests/egress_directive.rs::policy_posture_maps_totally_and_broker_only_is_not_direct_egress` (exhaustive). RED: `From` impl absent → compile fail confirmed | `crates/nist-agent-release/src/egress.rs`: exhaustive `From<nist_agent_policy::types::EgressPosture>` — `Disabled→Disabled`, `BrokerOnly→Disabled` (broker-only is never direct egress), `Allowed→Enabled`; release crate now deps nist-agent-policy (no cycle; `cargo metadata` verified) | green | `BrokerOnly→Enabled` → mapping test FAILS; restored | **FIXED**. Policy enum stays the signed source of truth; adding a policy variant now forces a compile-time decision here |
| NIST_AGENT-2026-05-31-010 — `compute_hash` doc claims canonical CBOR | INFO | `action.rs::compute_hash_doc_describes_the_real_scheme` (source pin, needle assembled). Behavioral RED failure confirmed | `crates/nist-agent-loop/src/action.rs`: field + method docs now describe the u64-BE length-prefixed scheme actually implemented and warn recomputing layers | green | stale CBOR claim restored in doc → pin test FAILS; restored | **FIXED** (doc-only; implementation was sound and unchanged) |

## Mutation pass summary

11 mutations (one per fix surface; -003 had two): every mutation made
at least one new test fail; all restorations verified by a clean
`git status` and a final full-suite green run (266/0).

## Deviations / notes

- RED phase: findings whose fix required a new API (-004, -005 helper,
  -008 helper, -009 mapping, -003 peer gate) pinned the contract via
  tests that failed to compile pre-fix; all other reds failed
  behaviorally against the unmodified code. Both failure modes were
  captured before any fix landed.
- -001 is FIXED at its fail-closed core (verify/activate/hold/refuse at
  startup) with the runtime-operation gating remainder explicitly
  DEFERRED-WITH-OWNER (nist-agent maintainer, SECREM-02 Phase 7) — the
  v1 daemon has no egress-bearing operation to gate yet.
- IPC wire format and `IPC_PROTOCOL_VERSION` unchanged; unauthorized
  peers receive a standard `Error` response.
- `Cargo.lock` updated (libc + path-dep edges). FUA-NIST-AGENT-01..03
  (release-pipeline findings from the follow-up audit) are out of this
  WP's scope (inaugural-audit findings only) and remain open on the
  citrate-security team board.
