//! Tripwire-in-waiting for FWA-C9-01 (MEDIUM, DEFERRED-WITH-OWNER).
//!
//! Finding: the daemon loads + signature-verifies the `PolicyBundle`
//! fail-closed and holds it in `DaemonState.policy`, but `dispatch()`
//! never ENFORCES it — policy decisions are passive. The audit
//! (2026-06-20 federation-wide, chunk FWA-C9) deferred the
//! *enforcement* half because at HEAD `dispatch()` exposes NO egress
//! or action surface to gate: the IPC request set is exactly the
//! read-only `{Status, QueueDepth, RecentAudit}` plus a reserved
//! `Shutdown` that is refused. There is literally nothing for the
//! policy to allow/deny.
//!
//! This test is the **trigger** that re-opens the deferral. It pins
//! two facts:
//!
//! 1. The dispatched `IpcRequest` surface is exactly the known
//!    read-only set + reserved `Shutdown`. The day someone adds a
//!    new request variant (an action / egress / broker request), the
//!    exhaustiveness assertion below stops compiling OR the source
//!    pin below fails — forcing the author to come back here.
//! 2. IF `dispatch()` ever performs egress/an action (detected via a
//!    source scan for the enforcement seam), it MUST consult
//!    `state.policy`. A new action arm that does NOT reference the
//!    policy fails this test.
//!
//! Discharge condition (from `.agentile/audits/2026-06-21-fwa-remediation/`):
//! when `dispatch()` gains an action surface, wire
//! `state.policy.egress_posture` (via the `nist-agent-release`
//! `EgressPosture` runtime gate) into that arm and convert this
//! tripwire-in-waiting into a live "a policy-denied action is blocked"
//! test.

use nist_agent_daemon::ipc::IpcRequest;

/// (1) Pin the dispatched request surface. This match is the
/// canonical inventory of every `IpcRequest` the daemon answers.
/// Adding a variant to `IpcRequest` makes this match non-exhaustive
/// and BREAKS COMPILATION — the new-variant author is forced to land
/// here, classify the variant as read-only vs. action, and (if it is
/// an action) wire policy enforcement before this test will compile
/// + pass again.
#[test]
fn ipc_request_surface_is_exactly_the_known_read_only_set() {
    // A representative value of every variant. If `IpcRequest` grows
    // a variant, the `match` below fails to compile (non-exhaustive),
    // which is the tripwire firing at build time.
    let surface = [
        IpcRequest::Status,
        IpcRequest::QueueDepth,
        IpcRequest::RecentAudit { count: 1 },
        IpcRequest::Shutdown,
    ];

    for req in surface {
        // Exhaustive classification. Every variant is either a
        // read-only status RPC or the reserved (refused) Shutdown.
        // A new variant forces a decision here.
        let is_gateable_action = match req {
            // Read-only status surface — nothing to gate.
            IpcRequest::Status => false,
            IpcRequest::QueueDepth => false,
            IpcRequest::RecentAudit { .. } => false,
            // Reserved; dispatch() refuses it (no action performed).
            IpcRequest::Shutdown => false,
            // NOTE TO THE AUTHOR OF A NEW VARIANT: if your variant
            // performs egress or any state-changing action, return
            // `true` here AND wire `state.policy.egress_posture`
            // enforcement into its dispatch arm. Then promote this
            // tripwire to a live deny test. Per FWA-C9-01.
        };
        assert!(
            !is_gateable_action,
            "a gateable action variant was added to IpcRequest without policy \
             enforcement at dispatch() — re-open FWA-C9-01 and wire \
             state.policy.egress_posture (see \
             .agentile/audits/2026-06-21-fwa-remediation/REMEDIATION_LOG.md)"
        );
    }
}

/// (2) Source-level enforcement guarantee. If `dispatch()` ever
/// performs egress/an action (heuristic: it references the
/// egress/action enforcement seam), it MUST also reference
/// `state.policy`. Today neither is present — both invariants below
/// hold vacuously, which is the deferred state. The day an action
/// arm is added WITH an egress call but WITHOUT a policy read, this
/// fails.
#[test]
fn dispatch_action_arm_must_consult_policy() {
    let src = include_str!("../src/daemon.rs");

    // Isolate the `dispatch` fn body so we don't trip on unrelated
    // code (e.g. DaemonState field docs).
    let dispatch_body = extract_fn_body(src, "async fn dispatch(")
        .expect("dispatch() must exist in daemon.rs (FWA-C9-01 anchor)");

    // Enforcement-seam markers: any of these in the dispatch body
    // means dispatch is performing (or gating) a real action.
    let action_markers = ["apply_directive", "is_enabled", ".infer(", "EgressPosture", "egress"];
    let performs_action = action_markers.iter().any(|m| dispatch_body.contains(m));

    // Policy-consultation marker.
    let consults_policy = dispatch_body.contains("state.policy") || dispatch_body.contains(".policy");

    if performs_action {
        assert!(
            consults_policy,
            "dispatch() now performs an egress/action but does NOT consult \
             state.policy — FWA-C9-01 enforcement gap is live. Wire \
             state.policy.egress_posture into the action arm."
        );
    }
    // else: deferred state — no action surface, nothing to gate.
    // This branch intentionally asserts nothing (vacuously true).
}

/// Extract the body of the first fn whose signature starts with
/// `sig_prefix`, by brace-matching from the first `{` after it.
fn extract_fn_body<'a>(src: &'a str, sig_prefix: &str) -> Option<&'a str> {
    let start = src.find(sig_prefix)?;
    let after = &src[start..];
    let open = after.find('{')?;
    let bytes = after.as_bytes();
    let mut depth = 0usize;
    let mut i = open;
    let body_start = open + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&after[body_start..i]);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}
