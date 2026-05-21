--------------------------- MODULE CapsuleInstallGate ---------------------------
(***************************************************************************)
(* cit-agent — Capsule Install Lifecycle                                   *)
(*                                                                         *)
(* Normative TLA+ spec for RFC-CIT-AGENT-0001 §4.5 (Capability            *)
(* Enforcement via WIT/WASM) and §9.1 row 4.                               *)
(*                                                                         *)
(* A capsule installs only after the harness has:                          *)
(*   1. Verified the manifest signature against the trusted publisher    *)
(*      key for its tier (bundled / managed / workspace)                  *)
(*   2. Cross-validated the WIT interface against the WASM imports vs    *)
(*      the manifest's [capability] declaration                          *)
(*   3. Traversed the HITL approval gate at the manifest's risk tier     *)
(*                                                                         *)
(* RFC §4.5 requires these to be LOAD-TIME invariants, not runtime       *)
(* guards. No codepath exists by which the capsule reaches `Installed`   *)
(* without all three predecessors.                                         *)
(*                                                                         *)
(* Properties verified:                                                    *)
(*   Safety:                                                               *)
(*     - NoCallableUninstalled: a capsule is callable only when its       *)
(*       state = Installed                                                 *)
(*     - NoPartialInstall: Installed => ManifestVerified ^ WitMatched ^   *)
(*       approval traversed                                                *)
(*     - RejectedNeverInstalls: once Rejected, the capsule never reaches *)
(*       Installed                                                         *)
(*     - SignatureBeforeWit: WIT cross-validation only begins after      *)
(*       manifest signature verification succeeds                          *)
(*     - WitBeforeApproval: HITL approval only begins after WIT matches  *)
(*   Liveness:                                                             *)
(*     - EveryProposedSettles: every Proposed install reaches Installed  *)
(*       or Rejected                                                       *)
(*                                                                         *)
(* This spec composes with ApprovalStateMachine.tla — the approval step *)
(* is abstracted here to {ApprovalApprove, ApprovalReject} actions; the *)
(* full quorum semantics live in that companion spec.                    *)
(*                                                                         *)
(* Status: draft — pending TLC verification by operator (CONFIG.md       *)
(* policy: Saul runs TLC locally).                                        *)
(*                                                                         *)
(* Supersedes (deprecation candidates — execution deferred to a later    *)
(* sprint, do NOT delete yet):                                            *)
(*   - McpCapabilityBoundary.tla : the prior MCP-layer capability       *)
(*     boundary spec is partially subsumed by this spec's manifest +    *)
(*     WIT cross-check, but it covers MCP-server-specific semantics not *)
(*     in scope here. Re-evaluate when MCP-as-capsule is implemented.    *)
(***************************************************************************)

EXTENDS Integers, Sequences, FiniteSets, TLC

CONSTANTS
    Capsules,    \* finite set of capsule identifiers
    Tiers        \* {bundled, managed, workspace} — signing tiers (RFC §4.4)

(***************************************************************************)
(* Install state phases. The order is forced — see the per-action guards. *)
(***************************************************************************)
Phases ==
    {"Proposed",
     "ManifestVerified",
     "WitMatched",
     "AwaitingApproval",
     "Installed",
     "Rejected"}

VARIABLES
    phase,           \* [Capsules -> Phases]
    signedOk,        \* SUBSET Capsules — manifest signature verified
    witMatched,      \* SUBSET Capsules — WIT cross-check passed
    approved,        \* SUBSET Capsules — HITL approval received
    callable,        \* SUBSET Capsules — runtime says this capsule may be called
    tierOf           \* [Capsules -> Tiers]

vars == <<phase, signedOk, witMatched, approved, callable, tierOf>>

TypeOK ==
    /\ phase \in [Capsules -> Phases]
    /\ signedOk \subseteq Capsules
    /\ witMatched \subseteq Capsules
    /\ approved \subseteq Capsules
    /\ callable \subseteq Capsules
    /\ tierOf \in [Capsules -> Tiers]

Init ==
    /\ phase = [c \in Capsules |-> "Proposed"]
    /\ signedOk = {}
    /\ witMatched = {}
    /\ approved = {}
    /\ callable = {}
    /\ tierOf \in [Capsules -> Tiers]

(***************************************************************************)
(* VerifyManifest(c): RFC §4.5 #1. Must succeed BEFORE any WIT parsing.   *)
(***************************************************************************)
VerifyManifest(c) ==
    /\ phase[c] = "Proposed"
    /\ signedOk' = signedOk \cup {c}
    /\ phase' = [phase EXCEPT ![c] = "ManifestVerified"]
    /\ UNCHANGED <<witMatched, approved, callable, tierOf>>

(***************************************************************************)
(* ManifestFails(c): the signature verification fails; the capsule is    *)
(* terminally rejected.                                                    *)
(***************************************************************************)
ManifestFails(c) ==
    /\ phase[c] = "Proposed"
    /\ phase' = [phase EXCEPT ![c] = "Rejected"]
    /\ UNCHANGED <<signedOk, witMatched, approved, callable, tierOf>>

(***************************************************************************)
(* CrossValidateWit(c): RFC §4.5 #2 #3. WIT imports MUST match the       *)
(* manifest's declared capability set. Only admissible AFTER signature   *)
(* verification.                                                          *)
(***************************************************************************)
CrossValidateWit(c) ==
    /\ phase[c] = "ManifestVerified"
    /\ c \in signedOk
    /\ witMatched' = witMatched \cup {c}
    /\ phase' = [phase EXCEPT ![c] = "WitMatched"]
    /\ UNCHANGED <<signedOk, approved, callable, tierOf>>

WitFails(c) ==
    /\ phase[c] = "ManifestVerified"
    /\ phase' = [phase EXCEPT ![c] = "Rejected"]
    /\ UNCHANGED <<signedOk, witMatched, approved, callable, tierOf>>

(***************************************************************************)
(* SubmitForApproval(c): WIT matched; the install now traverses the      *)
(* HITL gate per ApprovalStateMachine.                                    *)
(***************************************************************************)
SubmitForApproval(c) ==
    /\ phase[c] = "WitMatched"
    /\ c \in witMatched
    /\ phase' = [phase EXCEPT ![c] = "AwaitingApproval"]
    /\ UNCHANGED <<signedOk, witMatched, approved, callable, tierOf>>

(***************************************************************************)
(* ApprovalApprove(c) / ApprovalReject(c): the abstracted outcomes of the *)
(* full ApprovalStateMachine. The harness composes these with the        *)
(* tier-appropriate quorum; here we only model the binary outcome.       *)
(***************************************************************************)
ApprovalApprove(c) ==
    /\ phase[c] = "AwaitingApproval"
    /\ approved' = approved \cup {c}
    /\ phase' = [phase EXCEPT ![c] = "Installed"]
    /\ callable' = callable \cup {c}
    /\ UNCHANGED <<signedOk, witMatched, tierOf>>

ApprovalReject(c) ==
    /\ phase[c] = "AwaitingApproval"
    /\ phase' = [phase EXCEPT ![c] = "Rejected"]
    /\ UNCHANGED <<signedOk, witMatched, approved, callable, tierOf>>

Next ==
    \/ \E c \in Capsules : VerifyManifest(c)
    \/ \E c \in Capsules : ManifestFails(c)
    \/ \E c \in Capsules : CrossValidateWit(c)
    \/ \E c \in Capsules : WitFails(c)
    \/ \E c \in Capsules : SubmitForApproval(c)
    \/ \E c \in Capsules : ApprovalApprove(c)
    \/ \E c \in Capsules : ApprovalReject(c)

Spec == Init /\ [][Next]_vars
    /\ \A c1 \in Capsules :
         WF_vars(VerifyManifest(c1) \/ ManifestFails(c1))
    /\ \A c2 \in Capsules :
         WF_vars(CrossValidateWit(c2) \/ WitFails(c2))
    /\ \A c3 \in Capsules :
         WF_vars(SubmitForApproval(c3))
    /\ \A c4 \in Capsules :
         WF_vars(ApprovalApprove(c4) \/ ApprovalReject(c4))

(***************************************************************************)
(* SAFETY INVARIANTS                                                       *)
(***************************************************************************)

\* I1: Type safety.
TypeInvariant == TypeOK

\* I2: No callable uninstalled — RFC §9.1: a capsule is callable only
\*     when its phase is Installed.
NoCallableUninstalled ==
    \A c \in callable : phase[c] = "Installed"

\* I3: No partial install — Installed implies all predecessor checks
\*     succeeded.
NoPartialInstall ==
    \A c \in Capsules :
        phase[c] = "Installed" =>
            /\ c \in signedOk
            /\ c \in witMatched
            /\ c \in approved

\* I4: Rejected is terminal — once rejected, the capsule never installs.
RejectedNeverInstalls ==
    [][\A c \in Capsules :
        phase[c] = "Rejected" => phase'[c] = "Rejected"]_vars

\* I5: Ordering — WIT cross-check only happens after signature
\*     verification. Encoded as a state invariant: any capsule with WIT
\*     matched also has signedOk.
SignatureBeforeWit ==
    witMatched \subseteq signedOk

\* I6: Approval only happens after WIT match.
WitBeforeApproval ==
    approved \subseteq witMatched

\* I7: Installed is terminal (no resurrection of state).
InstalledIsSticky ==
    [][\A c \in Capsules :
        phase[c] = "Installed" => phase'[c] = "Installed"]_vars

(***************************************************************************)
(* LIVENESS                                                                *)
(***************************************************************************)

\* L1: Every Proposed capsule eventually reaches Installed or Rejected.
EveryProposedSettles ==
    \A c \in Capsules :
        phase[c] = "Proposed" ~> phase[c] \in {"Installed", "Rejected"}

(***************************************************************************)
(* TLC symmetry — referenced by CapsuleInstall.cfg via SYMMETRY.         *)
(***************************************************************************)
Symmetry == Permutations(Capsules)

=============================================================================
