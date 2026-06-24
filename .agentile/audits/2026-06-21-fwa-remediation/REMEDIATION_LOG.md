---
created: 2026-06-21T00:00:00Z
branch: remediation/fwa-2026-06
author: Claude Opus 4.8 (1M context) — RM-MISC/nist-agent remediation agent
status: active
audit_id: 2026-06-20-federation-wide-audit
chunk: FWA-C9
standard: Agentile-Audit Standard v0.2
---

# FWA-C9 remediation log — nist-agent

Remediation pass for the **federation-wide audit 2026-06-20**, chunk
**FWA-C9** (Agent runtime & WASM sandbox). Scope assigned to this repo:
**FWA-C9-01** (Medium).

Central report:
`citrate-security/audits/2026-06-20-federation-wide-audit/per-chunk/FWA-C9/REPORT.md`
Central deferrals ledger:
`citrate-security/audits/2026-06-20-federation-wide-audit/DEFERRALS.md`

Pinned SHA for nist-agent at audit time: `5d683dc` (branch `docs/partner-eval`).
Remediation branch: `remediation/fwa-2026-06`.

---

## FWA-C9-01 — PolicyBundle verified but never enforced at `dispatch()` (Medium)

**Final disposition: DEFERRED-WITH-OWNER (confirmed by enforcement-surface
re-determination).**

### Enforcement-surface determination (the job)

The remediation brief required confirming the deferral by determining whether a
real enforcement/egress surface exists at HEAD that the policy *should* gate.

**Determination: NO gateable action surface exists at `dispatch()` at HEAD.**
The verified `PolicyBundle` is held passively in `DaemonState.policy`; there is
no dispatched action for it to allow/deny. Evidence:

| Fact | Evidence (file:line @ `5d683dc`) |
|---|---|
| `dispatch()` answers exactly `Status`, `QueueDepth`, `RecentAudit`, `Shutdown` | `crates/nist-agent-daemon/src/daemon.rs:275-301` |
| `Status` / `QueueDepth` / `RecentAudit` are read-only in-memory status RPCs (no egress, no state change) | `daemon.rs:277-295` |
| `Shutdown` is **refused** — reserved for v1.1, performs no action | `daemon.rs:296-299` |
| The `IpcRequest` enum has no action/egress/broker variant | `crates/nist-agent-daemon/src/ipc.rs:30-42` |
| `dispatch()` never reads `state.policy` (nothing to consult) | `daemon.rs:275-301` (no `state.policy` reference) |
| The IPC socket is 0600 owner-only; peer-uid gated | `daemon.rs:120-125`, `:225-240` |
| Egress-enforcement machinery exists but lives in a **different crate**, unwired to the daemon | `crates/nist-agent-release/src/egress.rs:64-157` (`EgressPosture::apply_directive`, `is_enabled`, `From<policy::EgressPosture>`) |
| The only consumer of an egress decision is the model resolver's `egress_allowed`, which `dispatch()` does **not** call | `crates/nist-agent-model/src/resolver.rs:37,50` |
| Doctor reads `bundle.egress_posture` for a pre-flight CHECK (not a runtime gate) | `crates/nist-agent-doctor/src/checks.rs:128-151` |

Conclusion: the audit's **DEFERRED-WITH-OWNER** disposition is **correct**. The
policy object has nothing to gate at HEAD; FWA-C9-01 is the *enforcement* half of
prior NIST_AGENT-001 — the *load/verify/activate* half is already fixed
fail-closed (`policy.rs:25-51`, `policy_enforcement.rs` 6/6 pass).

### Deferral row (for the central DEFERRALS ledger)

| Field | Value |
|---|---|
| **Finding** | FWA-C9-01 — nist-agent PolicyBundle verified but never enforced at runtime |
| **Severity** | MEDIUM (CIT-SEV: asset=infra, impact=authbypass / policy-not-applied) |
| **Disposition** | DEFERRED-WITH-OWNER |
| **Owner** | `_nist-agent owner_` (named-person placeholder — must be signed per v0.2) |
| **Risk accepted by** | `_federation lead_` (placeholder — must be signed) |
| **SLA / target** | 90d (FedRAMP MED) — re-check **on egress wiring**, not on a calendar tick |
| **Rationale** | No runtime egress/action surface exists at `dispatch()` to enforce against at HEAD. The daemon's v1.0 IPC surface is read-only status RPCs over a 0600 owner-only Unix socket. The verified `PolicyBundle.egress_posture` is held passively in `DaemonState.policy`; no dispatched action can bypass it because none exists. |
| **Compensating control** | (1) Default `EgressPosture::Disabled` floor in the policy + `minimal_template` (`bundle.rs:73`). (2) Egress is opt-in only via a signed SecurityOfficer directive verified in-crate (`egress.rs:136-157`). (3) The IPC socket is owner-only (0600) + peer-uid gated. (4) The model resolver defaults `egress_allowed=false` (`resolver.rs:50`). |
| **Trigger that re-opens** | The day `dispatch()` (or any IPC request handler) gains an action/egress/broker arm that performs a gateable action. At that point the policy MUST be consulted and the action DENIED on a policy-deny. |
| **Tripwire-in-waiting** | `crates/nist-agent-daemon/tests/policy_enforcement_tripwire.rs` (2 tests, added this branch). See below. |

### Tripwire-in-waiting (the permanent guard)

Added `crates/nist-agent-daemon/tests/policy_enforcement_tripwire.rs` — two
tests that FAIL the day an enforcement surface is added without policy
enforcement:

1. **`ipc_request_surface_is_exactly_the_known_read_only_set`** — pins the
   dispatched `IpcRequest` inventory via an exhaustive `match`. Adding a variant
   to `IpcRequest` makes the match non-exhaustive and **breaks compilation**,
   forcing the new-variant author to classify it (read-only vs. action) and, if
   it is an action, return `true` in the classifier — which then trips the
   in-test assertion until policy enforcement is wired.
2. **`dispatch_action_arm_must_consult_policy`** — source-scans the `dispatch()`
   fn body. If it ever references the egress/action enforcement seam
   (`apply_directive`, `is_enabled`, `EgressPosture`, `egress`, `.infer(`) it
   MUST also reference `state.policy` / `.policy`. Today neither marker is
   present (vacuously deferred); the day an action arm is added without a policy
   read, this fails with the FWA-C9-01 re-open message.

**Tripwire validity proof (RED→GREEN of the guard itself):** injecting an
`apply_directive` marker into the `dispatch()` body without a `state.policy`
reference made `dispatch_action_arm_must_consult_policy` FAIL
(`tripwire ... panicked: dispatch() now performs an egress/action but does NOT
consult state.policy`); restoring HEAD returned it to GREEN. The guard is not
vacuously passing.

### Close-gate accounting (deferral track)

- **RED (denied action currently proceeds):** N/A — no action surface exists to
  deny against. The design-level NEEDS-REPRO in the central report stands.
- **DEFERRAL recorded:** this row + the central ledger note.
- **TRIPWIRE-IN-WAITING:** the two tests above; proven to fire.
- **DISCHARGE CONDITION:** when `dispatch()` gains an action surface, wire
  `state.policy.egress_posture` through the `nist-agent-release` `EgressPosture`
  runtime gate (`From<policy::EgressPosture>` → `is_enabled()`), DENY on a
  policy-deny, and promote the tripwire-in-waiting to a live
  "a policy-denied action is blocked" test.

### CODE-QUALITY

No production code changed (correct for a deferral). The added test file is
self-contained, references only the public `nist_agent_daemon::ipc::IpcRequest`
surface plus an `include_str!` source scan, and adds zero dependencies. The
brace-matching `extract_fn_body` helper is local to the test. Test count is
monotone-increasing (+2), consistent with Rule 2 / the test-count ratchet.

### DOCUMENTATION

The finding, the enforcement-surface determination, the deferral row, the
tripwire-in-waiting, and the discharge condition are all recorded here and
cross-referenced to the central report + ledger. The tripwire test file carries
a module doc that states the finding, the deferred state, and the discharge
condition inline next to the guard, so a future engineer touching `dispatch()`
meets the rationale at the point of change.

---

## Note for the central DEFERRALS ledger

`citrate-security/.../2026-06-20-federation-wide-audit/DEFERRALS.md` already
lists FWA-C9-01 (MED, owner `_nist-agent owner_`, 90d, "no runtime egress
surface exists yet", re-check "on egress wiring"). This remediation **confirms**
that row by independent re-determination at HEAD and **upgrades** the compensating
control + trigger detail above, and supplies the concrete **tripwire-in-waiting**
file path the central row was missing. The `_owner_` / `_federation lead_`
placeholders remain unsigned — per v0.2 an unaccepted deferral blocks audit close,
so this row is PROVISIONAL until named.
