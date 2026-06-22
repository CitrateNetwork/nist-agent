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
//! ## What this file is — and what BV-NIST-02 fixed
//!
//! BV-NIST-02 (LOW, tripwire-partially-effective): the prior version
//! of this file could be DODGED. Its test (2) only checked that the
//! dispatch source *mentions* the substring `state.policy`, and it
//! decided "is this an action" with a small renameable string
//! allowlist (`egress`/`EgressPosture`/`is_enabled`/`apply_directive`/
//! `.infer(`). A blind-verify PoC showed an adversarial "fix" could
//! add an `Egress` arm with a FAIL-OPEN default-allow body and stay
//! GREEN — by either (a) renaming the action so no marker matched, or
//! (b) printing the word `state.policy` in a comment without enforcing
//! it. Mention != enforcement; a string allowlist is evadable.
//!
//! This rewrite removes both bypasses by making the check
//! **structural** and **closed-world**:
//!
//! 1. `IpcRequest`'s variant inventory is pinned by an exhaustive
//!    `match`. Adding ANY variant breaks compilation here (good) and
//!    forces the author to classify it read-only vs. action.
//! 2. The set of `IpcRequest::<Variant>` arms that `dispatch()`
//!    actually handles is extracted STRUCTURALLY from the source and
//!    asserted to equal EXACTLY the known read-only allowlist. A new
//!    or RENAMED arm — whatever its identifier — changes that set and
//!    trips the test. There is no string allowlist to rename around.
//! 3. Fail-closed enforcement, not mention: because C9-01 enforcement
//!    is deferred, the invariant that holds TODAY is "dispatch has no
//!    non-read-only arm." The moment a non-read-only arm exists, the
//!    structural check (2) fires, and the assert message instructs the
//!    author to route it through a deny-default policy gate. The test
//!    cannot be satisfied by a fail-open arm that merely names the
//!    policy — the arm must not exist until it is gated.
//!
//! Discharge condition (from `.agentile/audits/2026-06-21-fwa-remediation/`):
//! when `dispatch()` gains an action surface, add the new variant to
//! the read-only set ONLY after wiring `state.policy.egress_posture`
//! (via the `nist-agent-release` `EgressPosture` runtime gate) into
//! that arm as a deny-default decision, and convert this
//! tripwire-in-waiting into a live "a policy-denied action is blocked"
//! test.

use nist_agent_daemon::ipc::IpcRequest;

/// The canonical, closed-world inventory of IPC requests that
/// `dispatch()` is allowed to answer WITHOUT a policy gate. Every
/// entry here is read-only (a status RPC) or the reserved, refused
/// `Shutdown`. This is the single source of truth shared by both the
/// exhaustiveness pin and the structural arm-set check below.
///
/// To ADD an entry you must consciously assert (in code review, and
/// by editing this list) that the variant performs no egress and no
/// state mutation — OR that it routes through a fail-closed policy
/// decision. Do not add an action variant here to silence the test.
const READ_ONLY_DISPATCH_ARMS: &[&str] = &["Status", "QueueDepth", "RecentAudit", "Shutdown"];

/// (1) Pin the `IpcRequest` variant inventory exhaustively. Adding a
/// variant to `IpcRequest` makes this `match` non-exhaustive and
/// BREAKS COMPILATION — the new-variant author is forced to land here,
/// classify the variant as read-only vs. action, and (if it is an
/// action) wire fail-closed policy enforcement before this test will
/// compile + pass again.
#[test]
fn ipc_request_surface_is_exactly_the_known_read_only_set() {
    // A representative value of every variant. If `IpcRequest` grows a
    // variant, the `match` below fails to compile (non-exhaustive),
    // which is the tripwire firing at build time.
    let surface = [
        IpcRequest::Status,
        IpcRequest::QueueDepth,
        IpcRequest::RecentAudit { count: 1 },
        IpcRequest::Shutdown,
    ];

    for req in surface {
        // Exhaustive classification. Every variant is either a
        // read-only status RPC or the reserved (refused) Shutdown. A
        // new variant forces a decision here.
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
            // enforcement (deny-default) into its dispatch arm. Then
            // promote this tripwire to a live deny test. Per
            // FWA-C9-01.
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

/// (2) STRUCTURAL closed-world check on `dispatch()`'s arms — the
/// BV-NIST-02 fix. Instead of asking "does the source mention
/// `state.policy`?" (a dodgeable substring) or "does it contain one of
/// these action markers?" (a renameable allowlist), we extract the set
/// of `IpcRequest::<Variant>` arm heads that `dispatch()` actually
/// matches on, and assert it equals EXACTLY `READ_ONLY_DISPATCH_ARMS`.
///
/// Why this catches the class the old test missed:
///   * A NEW arm (e.g. `IpcRequest::Egress => ...`) adds a variant to
///     the extracted set → set != allowlist → FAIL, regardless of what
///     the arm body says.
///   * A RENAMED fail-open arm (e.g. `IpcRequest::Transmit => ...`,
///     `IpcRequest::Send => ...`) is just as much a new arm name → it
///     too is in the extracted set but not the allowlist → FAIL. There
///     is no marker string to evade.
///   * Removing/renaming a read-only arm also trips it (the inventory
///     drifted), forcing a conscious update of the allowlist.
///
/// The only way to make this pass is to keep dispatch's handled-arm
/// set identical to the audited read-only inventory. Adding any action
/// surface REQUIRES the author to (a) edit READ_ONLY_DISPATCH_ARMS
/// AND/OR (b) discharge FWA-C9-01 with a deny-default policy gate — a
/// conscious act, not an accident a renamed arm slips past.
#[test]
fn dispatch_handles_exactly_the_read_only_arm_set_or_gates_policy() {
    let src = include_str!("../src/daemon.rs");

    let dispatch_body = extract_fn_body(src, "async fn dispatch(")
        .expect("dispatch() must exist in daemon.rs (FWA-C9-01 anchor)");

    // Structurally enumerate the variants dispatch matches on.
    let handled = handled_ipc_request_variants(dispatch_body);

    assert!(
        !handled.is_empty(),
        "could not extract any `IpcRequest::<Variant>` arms from dispatch() — \
         the match shape changed; re-derive this structural check before \
         trusting it (BV-NIST-02)"
    );

    let expected: std::collections::BTreeSet<String> = READ_ONLY_DISPATCH_ARMS
        .iter()
        .map(|s| (*s).to_string())
        .collect();

    // Anything dispatch handles that is NOT in the audited read-only
    // set is, by definition, an arm that was added or renamed without
    // updating the inventory — i.e. a candidate fail-open action arm.
    let unexpected: Vec<&String> = handled.difference(&expected).collect();
    assert!(
        unexpected.is_empty(),
        "dispatch() handles IpcRequest arm(s) {unexpected:?} that are NOT in the \
         audited read-only inventory {READ_ONLY_DISPATCH_ARMS:?}. A new/renamed \
         dispatch arm appeared. If it performs egress or any action it MUST route \
         through a fail-closed (deny-default) policy decision on \
         state.policy.egress_posture before being added to the inventory — \
         FWA-C9-01. A fail-open default-allow arm is NOT acceptable. \
         (BV-NIST-02 catches renamed/fail-open arms structurally.)"
    );

    // Read-only arms that vanished mean the inventory drifted; force a
    // conscious reconciliation (don't let the closed-world set rot).
    let missing: Vec<&String> = expected.difference(&handled).collect();
    assert!(
        missing.is_empty(),
        "dispatch() no longer handles audited read-only arm(s) {missing:?}; the \
         IPC surface drifted. Reconcile READ_ONLY_DISPATCH_ARMS with dispatch() \
         and re-confirm no action surface slipped in (BV-NIST-02)."
    );
}

/// (3) Fail-closed posture pin: enforcement, not mention. Because
/// C9-01 enforcement is deferred, the invariant that holds at HEAD is
/// that dispatch performs NO egress/action — so every arm is purely a
/// read of `state` returning an `IpcResponse`, and the reserved
/// `Shutdown` arm REFUSES rather than acts.
///
/// This guards the *posture* the structural set check assumes: if
/// someone keeps the arm NAMES identical to the allowlist (dodging
/// check 2) but mutates an existing read-only arm into a fail-open
/// action (e.g. turning `Shutdown` from a refusal into an actual
/// shutdown without a policy gate), we still want a tripwire. We
/// assert that the reserved `Shutdown` arm continues to refuse — it
/// must not become an executed action without policy enforcement.
#[test]
fn reserved_shutdown_arm_still_refuses_not_acts() {
    let src = include_str!("../src/daemon.rs");
    let dispatch_body = extract_fn_body(src, "async fn dispatch(")
        .expect("dispatch() must exist in daemon.rs (FWA-C9-01 anchor)");

    let shutdown_arm = extract_match_arm_body(dispatch_body, "IpcRequest::Shutdown")
        .expect("dispatch() must still have an `IpcRequest::Shutdown` arm (read-only inventory)");

    // The reserved arm must REFUSE: it returns an Error response and
    // does not perform a real shutdown / process exit / signal raise.
    // If C9-01 is ever discharged to make Shutdown actionable, that
    // action must be gated on state.policy — at which point update this
    // pin to assert the deny-default gate instead of refusal.
    assert!(
        shutdown_arm.contains("IpcResponse::Error"),
        "the reserved IpcRequest::Shutdown arm must REFUSE (return \
         IpcResponse::Error), not act. If you are wiring real shutdown, gate it \
         on state.policy (deny-default) — FWA-C9-01 / BV-NIST-02."
    );
    let forbidden_action_calls = ["process::exit", "std::process::exit", "raise(", "abort("];
    for call in forbidden_action_calls {
        assert!(
            !shutdown_arm.contains(call),
            "the reserved IpcRequest::Shutdown arm performs an action (`{call}`) \
             without a fail-closed policy gate — FWA-C9-01 enforcement gap is \
             live. Route it through state.policy.egress_posture (deny-default) \
             before acting. (BV-NIST-02)"
        );
    }
}

/// Extract the set of `IpcRequest::<Variant>` identifiers that appear
/// as match-arm heads inside `body`. We scan for the literal pattern
/// `IpcRequest::` and read the following Rust identifier. This is
/// closed-world: any arm head, whatever it is named, is captured.
fn handled_ipc_request_variants(body: &str) -> std::collections::BTreeSet<String> {
    const PAT: &str = "IpcRequest::";
    let mut out = std::collections::BTreeSet::new();
    let bytes = body.as_bytes();
    let mut search_from = 0usize;
    while let Some(rel) = body[search_from..].find(PAT) {
        let id_start = search_from + rel + PAT.len();
        // Read the identifier that follows `IpcRequest::`.
        let mut i = id_start;
        while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
            i += 1;
        }
        if i > id_start {
            out.insert(body[id_start..i].to_string());
        }
        search_from = id_start.max(i);
    }
    out
}

/// Extract the body text of a single match arm whose pattern head
/// matches `arm_pattern` (e.g. `"IpcRequest::Shutdown"`). Returns the
/// text from the arm's `=>` up to the start of the next arm (or the
/// end of the match block), which is enough to inspect what the arm
/// does. Brace-aware so a block-bodied arm is captured whole.
fn extract_match_arm_body<'a>(body: &'a str, arm_pattern: &str) -> Option<&'a str> {
    let pat_at = body.find(arm_pattern)?;
    let after_pat = &body[pat_at..];
    let arrow = after_pat.find("=>")?;
    let arm_start = arrow + 2;
    let region = &after_pat[arm_start..];
    let bytes = region.as_bytes();

    // Walk forward, tracking brace depth. The arm ends at the first
    // top-level (depth 0) comma OR when we return to negative depth
    // (the closing `}` of the match). A block-bodied arm opens with
    // `{`; an expression arm ends at the next top-level comma.
    let mut depth: i32 = 0;
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                if depth == 0 {
                    // Closing brace of the enclosing match.
                    return Some(&region[..i]);
                }
                depth -= 1;
            }
            b',' if depth == 0 => return Some(&region[..i]),
            _ => {}
        }
        i += 1;
    }
    Some(region)
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
