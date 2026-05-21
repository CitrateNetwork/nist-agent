--------------------------- MODULE ApprovalStateMachine ---------------------------
(***************************************************************************)
(* cit-agent — Tiered-Risk + Role-Bound Quorum Approval State Machine     *)
(*                                                                         *)
(* Normative TLA+ spec for RFC-CIT-AGENT-0001 §5 (Human-in-the-Loop        *)
(* Approval) and §9.1 row 1. Models the per-action approval lifecycle:    *)
(*                                                                         *)
(*   Proposed -> AwaitingRole_i -> ... -> {Approved | Rejected}           *)
(*                                  \                                     *)
(*                                   -> Resumed -> Executed               *)
(*                                                                         *)
(* Builds on the FIFO + timeout shape of gui/ToolApprovalLifecycle.tla    *)
(* but generalises to:                                                     *)
(*   - Risk tiers Low | Medium | High | Critical (RFC §5.2)                *)
(*   - Five-role lattice Operator/Reviewer/Compliance/Security/Auditor    *)
(*     (RFC §5.3)                                                          *)
(*   - Per-tier required-role multisets (quorum)                          *)
(*   - State-managed interrupt with deterministic resumption (RFC §5.4)   *)
(*   - Edit-on-medium MUST NOT silently promote (RFC §9.1 row 1)          *)
(*                                                                         *)
(* Properties verified:                                                    *)
(*   Safety:                                                               *)
(*     - Termination: every Proposed action ultimately reaches            *)
(*       {Rejected, Executed}                                              *)
(*     - NoExecuteWithoutQuorum: an action Executed iff its required-role *)
(*       multiset signatures are all collected                            *)
(*     - AuditorNeverApproves: Auditor signatures never appear            *)
(*     - NoSilentPromotion: an action whose risk tier is edited after    *)
(*       proposal triggers a new approval cycle, not silent execution    *)
(*     - DeterministicResume: Resumed state is a function of the         *)
(*       checkpoint + approval payload alone                              *)
(*     - TerminalsAreSticky: {Rejected, Executed} are absorbing states   *)
(*   Liveness:                                                            *)
(*     - EventuallyTerminal: every Proposed action ~> terminal            *)
(*                                                                         *)
(* Status: draft — pending TLC verification by operator (CONFIG.md        *)
(* policy: Saul runs TLC locally; CI ratchet is .cfg-driven).             *)
(*                                                                         *)
(* Supersedes (deprecation candidates — execution deferred to a later     *)
(* sprint, do NOT delete yet):                                            *)
(*   - HermesSidecarGrantBoundary.tla : the Hermes-specific guided/operator *)
(*     grant-boundary spec is generalised here to the five-role quorum    *)
(*     under cit-agent. Once cit-agent supersedes Hermes-as-sidecar, that *)
(*     spec should move to agent/deprecated/.                             *)
(*   - AgentToolApproval.tla : the 4-tier risk model with autoApprove +   *)
(*     emergency stop is folded into this spec's tier+role lattice. Keep  *)
(*     until cit-agent ships v1.0 so the old harness retains coverage.   *)
(***************************************************************************)

EXTENDS Integers, Sequences, FiniteSets, TLC

CONSTANTS
    Actions,    \* finite set of action identifiers proposed by capsules
    Roles,      \* {Op, Rv, Co, So, Au} — five-role lattice (RFC §5.3)
    Tiers,      \* {Low, Medium, High, Critical} — risk tiers (RFC §5.2)
    Capsules    \* finite set of capsule identifiers (for tier source)

(***************************************************************************)
(* Quorum mapping (RFC §5.2). Defined in the spec, not the cfg, because  *)
(* TLC cfg files do not accept function-valued constants in record form. *)
(*                                                                         *)
(*   Low      -> {}              (auto-approve / log only)                *)
(*   Medium   -> {Op}            (single approver)                        *)
(*   High     -> {Op, Rv}        (N-of-M, default 2)                      *)
(*   Critical -> {Rv, Co, So}    (full quorum)                            *)
(***************************************************************************)
Quorum(t) ==
    CASE t = "Low"      -> {}
      [] t = "Medium"   -> {"Op"}
      [] t = "High"     -> {"Op", "Rv"}
      [] t = "Critical" -> {"Rv", "Co", "So"}

VARIABLES
    state,         \* [Actions -> State]
    tier,          \* [Actions -> Tiers]  — current tier (may be edited)
    proposedTier,  \* [Actions -> Tiers]  — tier at proposal time
    capsuleOf,     \* [Actions -> Capsules]
    signatures,    \* [Actions -> SUBSET Roles] — accumulated signatures
    checkpoint,    \* [Actions -> SUBSET Roles] — frozen at proposal; used for resumption determinism
    edited,        \* SUBSET Actions — actions whose tier was changed since proposal
    executed       \* SUBSET Actions — actions that have run

vars == <<state, tier, proposedTier, capsuleOf, signatures, checkpoint, edited, executed>>

(***************************************************************************)
(* State alphabet. AwaitingRole_r is encoded as the flat string            *)
(*   "Awaiting_<r>" (CIT-AGENT-2 WP-1 — was previously the record         *)
(*   [phase |-> "Awaiting", role |-> r]).                                  *)
(* The record encoding made `state \in [Actions -> States]` heterogeneous *)
(* over a function whose range mixed atoms with records, which TLC could  *)
(* not fingerprint. Flat strings keep the function homogeneous.            *)
(***************************************************************************)
PhasePlain     == {"Proposed", "Approved", "Rejected", "Resumed", "Executed"}
AwaitingState(r)   == "Awaiting_" \o r
AwaitingStates == { AwaitingState(r) : r \in Roles }
States         == PhasePlain \cup AwaitingStates
Terminal       == {"Rejected", "Executed"}

IsAwaiting(s) == s \in AwaitingStates
RoleOf(s)     == CHOOSE r \in Roles : s = AwaitingState(r)

TypeOK ==
    /\ state       \in [Actions -> States]
    /\ tier        \in [Actions -> Tiers]
    /\ proposedTier \in [Actions -> Tiers]
    /\ capsuleOf   \in [Actions -> Capsules]
    /\ signatures  \in [Actions -> SUBSET Roles]
    /\ checkpoint  \in [Actions -> SUBSET Roles]
    /\ edited      \subseteq Actions
    /\ executed    \subseteq Actions

(***************************************************************************)
(* Init: every action begins Proposed with its capsule's required-role     *)
(* multiset frozen into checkpoint (the deterministic-resume anchor).     *)
(***************************************************************************)
Init ==
    /\ capsuleOf \in [Actions -> Capsules]
    /\ tier \in [Actions -> Tiers]
    /\ proposedTier = tier
    /\ state = [a \in Actions |-> "Proposed"]
    /\ signatures = [a \in Actions |-> {}]
    /\ checkpoint = [a \in Actions |-> Quorum(tier[a])]
    /\ edited = {}
    /\ executed = {}

(***************************************************************************)
(* PickNextRole(a): move an action from Proposed/Awaiting to the next     *)
(* role in the quorum it has not yet collected. If the required multiset  *)
(* is empty (Low), it can go straight to Approved.                        *)
(***************************************************************************)
RemainingRoles(a) == checkpoint[a] \ signatures[a]

Begin(a) ==
    /\ state[a] = "Proposed"
    /\ \/ /\ RemainingRoles(a) = {}
          /\ state' = [state EXCEPT ![a] = "Approved"]
       \/ /\ RemainingRoles(a) /= {}
          /\ \E r \in RemainingRoles(a) :
                state' = [state EXCEPT ![a] = AwaitingState(r)]
    /\ UNCHANGED <<tier, proposedTier, capsuleOf, signatures, checkpoint, edited, executed>>

(***************************************************************************)
(* SignByRole(a, r): the operator with role r signs the action a. The     *)
(* signature is recorded; the action advances either to the next awaited *)
(* role or to Approved when the quorum is satisfied.                       *)
(*                                                                         *)
(* Auditor (Au) cannot sign — enforced both by the guard and by the      *)
(* AuditorNeverApproves invariant below.                                   *)
(***************************************************************************)
SignByRole(a, r) ==
    /\ r \in Roles
    /\ r /= "Au"
    /\ state[a] = AwaitingState(r)
    /\ r \in RemainingRoles(a)
    /\ signatures' = [signatures EXCEPT ![a] = signatures[a] \cup {r}]
    /\ LET newSigs == signatures[a] \cup {r}
           remaining == checkpoint[a] \ newSigs
       IN \/ /\ remaining = {}
             /\ state' = [state EXCEPT ![a] = "Approved"]
          \/ /\ remaining /= {}
             /\ \E r2 \in remaining :
                   state' = [state EXCEPT ![a] = AwaitingState(r2)]
    /\ UNCHANGED <<tier, proposedTier, capsuleOf, checkpoint, edited, executed>>

(***************************************************************************)
(* RejectByRole(a, r): any required role may reject; this terminates the *)
(* action in Rejected.                                                     *)
(***************************************************************************)
RejectByRole(a, r) ==
    /\ r \in Roles
    /\ r /= "Au"
    /\ state[a] = AwaitingState(r)
    /\ state' = [state EXCEPT ![a] = "Rejected"]
    /\ UNCHANGED <<tier, proposedTier, capsuleOf, signatures, checkpoint, edited, executed>>

(***************************************************************************)
(* EditTier(a, t2): the operator edits the action's tier post-proposal.   *)
(* RFC §9.1 row 1 requires this MUST NOT silently promote: it forces a   *)
(* new approval cycle (state returns to Proposed, signatures cleared,    *)
(* checkpoint refreshed). The `edited` set is sticky to enable the       *)
(* NoSilentPromotion safety check.                                        *)
(*                                                                         *)
(* Editing is restricted to Medium tier actions and only while           *)
(* Awaiting / Proposed (RFC §5.2 high/critical disallow edit).           *)
(***************************************************************************)
EditTier(a, t2) ==
    /\ t2 \in Tiers
    /\ proposedTier[a] = "Medium"
    /\ a \notin edited   \* edit-once: once edited, no further re-edits
    /\ \/ state[a] = "Proposed"
       \/ IsAwaiting(state[a])
    /\ tier[a] /= t2
    /\ tier' = [tier EXCEPT ![a] = t2]
    /\ signatures' = [signatures EXCEPT ![a] = {}]
    /\ checkpoint' = [checkpoint EXCEPT ![a] = Quorum(t2)]
    /\ state' = [state EXCEPT ![a] = "Proposed"]
    /\ edited' = edited \cup {a}
    /\ UNCHANGED <<proposedTier, capsuleOf, executed>>

(***************************************************************************)
(* Resume(a): once Approved, the harness deterministically resumes the   *)
(* agent loop from the frozen checkpoint. The resumed state depends only *)
(* on `checkpoint[a]` and `signatures[a]` (RFC §5.4 determinism).        *)
(***************************************************************************)
Resume(a) ==
    /\ state[a] = "Approved"
    /\ state' = [state EXCEPT ![a] = "Resumed"]
    /\ UNCHANGED <<tier, proposedTier, capsuleOf, signatures, checkpoint, edited, executed>>

Execute(a) ==
    /\ state[a] = "Resumed"
    /\ checkpoint[a] \subseteq signatures[a]
    /\ state' = [state EXCEPT ![a] = "Executed"]
    /\ executed' = executed \cup {a}
    /\ UNCHANGED <<tier, proposedTier, capsuleOf, signatures, checkpoint, edited>>

Next ==
    \/ \E a \in Actions : Begin(a)
    \/ \E a \in Actions, r \in Roles : SignByRole(a, r)
    \/ \E a \in Actions, r \in Roles : RejectByRole(a, r)
    \/ \E a \in Actions, t2 \in Tiers : EditTier(a, t2)
    \/ \E a \in Actions : Resume(a)
    \/ \E a \in Actions : Execute(a)

Spec == Init /\ [][Next]_vars
    /\ \A a1 \in Actions : WF_vars(Begin(a1) \/ Resume(a1) \/ Execute(a1))
    /\ \A a2 \in Actions, r \in Roles : WF_vars(SignByRole(a2, r) \/ RejectByRole(a2, r))

(***************************************************************************)
(* SAFETY INVARIANTS                                                       *)
(***************************************************************************)

\* I1: Type safety.
TypeInvariant == TypeOK

\* I2: No action executes without its full required-role multiset.
NoExecuteWithoutQuorum ==
    \A a \in executed : checkpoint[a] \subseteq signatures[a]

\* I3: Auditor never appears on any signature set (AC-5 separation of
\*     duties — RFC §5.3).
AuditorNeverApproves ==
    \A a \in Actions : "Au" \notin signatures[a]

\* I4: Editing a Medium tier action does not silently execute the
\*     pre-edit signatures — the signature set is cleared on edit, so an
\*     edited action cannot have been executed using its pre-edit quorum.
\*     We capture this as: an edited action that reaches Executed must
\*     have collected the post-edit quorum after the edit.
NoSilentPromotion ==
    \A a \in (edited \cap executed) :
        checkpoint[a] \subseteq signatures[a]

\* I5: Terminal states are absorbing (no resurrection from Rejected /
\*     Executed).
TerminalsAreSticky ==
    [][\A a \in Actions : state[a] \in Terminal => state'[a] = state[a]]_vars

\* I6: Determinism of resumption — the checkpoint captured at proposal
\*     time (or at edit time) is what gates Execute; no other variable
\*     can have promoted the action.
DeterministicResume ==
    \A a \in executed :
        /\ checkpoint[a] = Quorum(tier[a])
        /\ checkpoint[a] \subseteq signatures[a]

(***************************************************************************)
(* LIVENESS                                                                *)
(***************************************************************************)

\* L1: Every action eventually reaches a terminal state. Bounded by the
\*     CONSTRAINT on the cfg (no infinite EditTier loops).
EventuallyTerminal ==
    \A a \in Actions : state[a] = "Proposed" ~> state[a] \in Terminal

(***************************************************************************)
(* TLC bounded-model state constraint — referenced by .cfg via            *)
(* CONSTRAINT StateConstraint. Bounds the cardinality of the `edited`     *)
(* set so the EditTier action cannot loop indefinitely on Medium-tier     *)
(* actions. The inline Cardinality(...) expression in the .cfg is         *)
(* rejected by TLC ("constraint of Cardinality is equal to Java method"); *)
(* a named operator is required.                                           *)
(***************************************************************************)
StateConstraint == Cardinality(edited) <= 2

(***************************************************************************)
(* TLC symmetry — referenced by ApprovalStateMachine.cfg via SYMMETRY.    *)
(***************************************************************************)
Symmetry == Permutations(Actions) \cup Permutations(Capsules)

=============================================================================
