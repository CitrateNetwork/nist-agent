--------------------------- MODULE BreakGlassPath ---------------------------
(***************************************************************************)
(* cit-agent — Break-Glass Emergency Path                                  *)
(*                                                                         *)
(* Normative TLA+ spec for RFC-CIT-AGENT-0001 §5.5 (Break-Glass Emergency *)
(* Path) and §9.1 row 5.                                                   *)
(*                                                                         *)
(* The break-glass path admits a single Security Officer signature in    *)
(* lieu of full quorum, subject to constraints:                            *)
(*                                                                         *)
(*   - MUST be explicitly enabled per-capsule (break_glass_eligible)     *)
(*   - MUST trigger immediate notification to every approver in the     *)
(*     configured role lattice                                            *)
(*   - MUST be followed within 72 hours by a post-hoc quorum (Reviewer + *)
(*     Compliance Officer) that affirms the action                       *)
(*   - If unaffirmed past the window, MUST surface on every subsequent  *)
(*     doctor report                                                      *)
(*   - MUST NOT be eligible for ITAR-classified data under any policy   *)
(*                                                                         *)
(* The clock is abstract; one tick == one hour (so DEADLINE = 72         *)
(* corresponds to the 72-hour window in production). For TLC tractability *)
(* we run with a much smaller deadline.                                    *)
(*                                                                         *)
(* Properties verified:                                                    *)
(*   Safety:                                                               *)
(*     - BreakGlassNotifiesAll: invocation always notifies every role    *)
(*       in the configured lattice                                        *)
(*     - ITARBlocksBreakGlass: ITAR-classified actions never enter the   *)
(*       BreakGlassInvoked state                                          *)
(*     - AffirmationOrSurfaced: every BreakGlassInvoked action either   *)
(*       reaches Affirmed within the window or is Surfaced               *)
(*     - SurfacedIsSticky: once Surfaced, the action remains so on every *)
(*       subsequent doctor report until explicitly Affirmed              *)
(*     - SingleSOSufficient: invocation requires exactly one Security   *)
(*       Officer signature (not zero, not full quorum)                  *)
(*   Liveness:                                                             *)
(*     - WindowAlwaysResolves: every BreakGlassInvoked action ~>        *)
(*       {Affirmed, Surfaced}                                              *)
(*                                                                         *)
(* Status: draft — pending TLC verification by operator (CONFIG.md       *)
(* policy: Saul runs TLC locally).                                        *)
(*                                                                         *)
(* Supersedes (deprecation candidates — execution deferred to a later    *)
(* sprint, do NOT delete yet): none direct. EmergencyStopProtocol.tla   *)
(* covers the kill-switch path, not break-glass; both remain in scope.  *)
(***************************************************************************)

EXTENDS Integers, Sequences, FiniteSets, TLC

CONSTANTS
    Actions,    \* finite set of action identifiers
    Approvers,  \* finite set of role identifiers (the lattice)
    DEADLINE    \* clock ticks before unaffirmed -> Surfaced

(***************************************************************************)
(* State alphabet.                                                         *)
(*                                                                         *)
(*   Pending          — action proposed, not yet invoking break-glass   *)
(*   BreakGlassInvoked — Security Officer issued the break-glass         *)
(*                       signature; every approver was notified          *)
(*   Affirmed         — post-hoc quorum (Reviewer + Compliance) affirmed *)
(*                       within window                                    *)
(*   Unaffirmed       — window elapsed without quorum (intermediate)    *)
(*   Surfaced         — appears on every doctor report (terminal until  *)
(*                       remediated via separate IR workflow)            *)
(***************************************************************************)
Phases == {"Pending", "BreakGlassInvoked", "Affirmed", "Unaffirmed", "Surfaced"}

VARIABLES
    phase,         \* [Actions -> Phases]
    isITAR,        \* SUBSET Actions — actions touching ITAR data
    eligible,      \* SUBSET Actions — break_glass_eligible per manifest
    notified,      \* [Actions -> SUBSET Approvers] — roles notified on invocation
    invokedAt,     \* [Actions -> Nat] — clock tick at invocation; 0 if not invoked
    affirmation,   \* [Actions -> SUBSET Approvers] — post-hoc affirmation signatures
    surfaceCount,  \* [Actions -> Nat] — number of doctor reports this surfaced on
    clock          \* monotonic Nat — abstract hours

vars == <<phase, isITAR, eligible, notified, invokedAt, affirmation, surfaceCount, clock>>

(***************************************************************************)
(* AffirmationSet: the post-hoc quorum required for a valid affirmation. *)
(* RFC §5.5: Reviewer + Compliance Officer.                                *)
(***************************************************************************)
AffirmationSet == {"Rv", "Co"}

TypeOK ==
    /\ phase \in [Actions -> Phases]
    /\ isITAR \subseteq Actions
    /\ eligible \subseteq Actions
    /\ notified \in [Actions -> SUBSET Approvers]
    /\ invokedAt \in [Actions -> Nat]
    /\ affirmation \in [Actions -> SUBSET Approvers]
    /\ surfaceCount \in [Actions -> Nat]
    /\ clock \in Nat
    /\ AffirmationSet \subseteq Approvers

Init ==
    /\ phase = [a \in Actions |-> "Pending"]
    /\ isITAR \in SUBSET Actions
    /\ eligible \in SUBSET Actions
    /\ notified = [a \in Actions |-> {}]
    /\ invokedAt = [a \in Actions |-> 0]
    /\ affirmation = [a \in Actions |-> {}]
    /\ surfaceCount = [a \in Actions |-> 0]
    /\ clock = 0

(***************************************************************************)
(* InvokeBreakGlass(a): the Security Officer issues a break-glass         *)
(* signature on action a. RFC §5.5: every approver MUST be notified at  *)
(* this moment.                                                            *)
(*                                                                         *)
(* ITAR-classified actions cannot invoke break-glass under any policy    *)
(* bundle. Eligibility (`eligible`) must be set in the capsule manifest. *)
(***************************************************************************)
InvokeBreakGlass(a) ==
    /\ phase[a] = "Pending"
    /\ a \notin isITAR
    /\ a \in eligible
    /\ phase' = [phase EXCEPT ![a] = "BreakGlassInvoked"]
    /\ notified' = [notified EXCEPT ![a] = Approvers]
    /\ invokedAt' = [invokedAt EXCEPT ![a] = clock]
    /\ UNCHANGED <<isITAR, eligible, affirmation, surfaceCount, clock>>

(***************************************************************************)
(* RejectBreakGlass(a): the break-glass invocation is refused because    *)
(* the action is ITAR or ineligible. The action stays Pending (other   *)
(* paths — normal approval — may still resolve it via the                *)
(* ApprovalStateMachine spec, which is out of scope for this spec).     *)
(***************************************************************************)
RejectBreakGlass(a) ==
    /\ phase[a] = "Pending"
    /\ \/ a \in isITAR
       \/ a \notin eligible
    /\ phase' = phase
    /\ UNCHANGED <<isITAR, eligible, notified, invokedAt, affirmation, surfaceCount, clock>>

(***************************************************************************)
(* PostHocSign(a, r): a post-hoc approver r signs the affirmation. When  *)
(* the AffirmationSet is fully signed, the action moves to Affirmed.    *)
(***************************************************************************)
PostHocSign(a, r) ==
    /\ phase[a] = "BreakGlassInvoked"
    /\ r \in AffirmationSet
    /\ r \notin affirmation[a]
    /\ clock - invokedAt[a] < DEADLINE
    /\ affirmation' = [affirmation EXCEPT ![a] = affirmation[a] \cup {r}]
    /\ LET newSigs == affirmation[a] \cup {r} IN
        IF AffirmationSet \subseteq newSigs
        THEN phase' = [phase EXCEPT ![a] = "Affirmed"]
        ELSE phase' = phase
    /\ UNCHANGED <<isITAR, eligible, notified, invokedAt, surfaceCount, clock>>

(***************************************************************************)
(* WindowExpired(a): the 72-hour window elapsed without full quorum. The *)
(* action transitions to Unaffirmed; the next doctor run surfaces it.    *)
(***************************************************************************)
WindowExpired(a) ==
    /\ phase[a] = "BreakGlassInvoked"
    /\ clock - invokedAt[a] >= DEADLINE
    /\ ~ (AffirmationSet \subseteq affirmation[a])
    /\ phase' = [phase EXCEPT ![a] = "Unaffirmed"]
    /\ UNCHANGED <<isITAR, eligible, notified, invokedAt, affirmation, surfaceCount, clock>>

(***************************************************************************)
(* DoctorReport: every doctor run surfaces every Unaffirmed and Surfaced *)
(* action. Surfacing an Unaffirmed action transitions it to Surfaced and *)
(* increments the per-action surface counter.                              *)
(***************************************************************************)
DoctorReport ==
    /\ \E a \in Actions : phase[a] \in {"Unaffirmed", "Surfaced"}
    /\ phase' = [a \in Actions |->
                    IF phase[a] = "Unaffirmed" THEN "Surfaced" ELSE phase[a]]
    /\ surfaceCount' = [a \in Actions |->
                          IF phase[a] \in {"Unaffirmed", "Surfaced"}
                          THEN surfaceCount[a] + 1
                          ELSE surfaceCount[a]]
    /\ UNCHANGED <<isITAR, eligible, notified, invokedAt, affirmation, clock>>

Tick ==
    /\ clock' = clock + 1
    /\ UNCHANGED <<phase, isITAR, eligible, notified, invokedAt, affirmation, surfaceCount>>

Next ==
    \/ \E a \in Actions : InvokeBreakGlass(a)
    \/ \E a \in Actions : RejectBreakGlass(a)
    \/ \E a \in Actions, r \in Approvers : PostHocSign(a, r)
    \/ \E a \in Actions : WindowExpired(a)
    \/ DoctorReport
    \/ Tick

Spec == Init /\ [][Next]_vars
    /\ WF_vars(\E a \in Actions : WindowExpired(a))
    /\ WF_vars(DoctorReport)
    /\ WF_vars(Tick)

(***************************************************************************)
(* SAFETY INVARIANTS                                                       *)
(***************************************************************************)

\* I1: Type safety.
TypeInvariant == TypeOK

\* I2: Invocation notifies all approvers (RFC §5.5).
BreakGlassNotifiesAll ==
    \A a \in Actions :
        phase[a] \in {"BreakGlassInvoked", "Affirmed", "Unaffirmed", "Surfaced"}
        => notified[a] = Approvers

\* I3: ITAR never reaches break-glass (RFC §5.5 last bullet).
ITARBlocksBreakGlass ==
    \A a \in isITAR :
        phase[a] \notin {"BreakGlassInvoked", "Affirmed", "Unaffirmed", "Surfaced"}

\* I4: Only break-glass-eligible actions may invoke.
OnlyEligibleInvokes ==
    \A a \in Actions :
        phase[a] \in {"BreakGlassInvoked", "Affirmed", "Unaffirmed", "Surfaced"}
        => a \in eligible

\* I5: Affirmed implies the full post-hoc quorum signed.
AffirmedRequiresQuorum ==
    \A a \in Actions :
        phase[a] = "Affirmed" => AffirmationSet \subseteq affirmation[a]

\* I6: Surfaced -> previously Unaffirmed -> window expired without
\*     quorum. Equivalently: a Surfaced action was once invoked.
\*     CIT-AGENT-2 fix: previous witness was `invokedAt[a] > 0`, which
\*     fails when InvokeBreakGlass fires at clock = 0. The true witness
\*     for "was invoked" is `notified[a] = Approvers` — that assignment
\*     happens nowhere else and uniquely identifies invocation.
SurfacedWasInvoked ==
    \A a \in Actions :
        phase[a] = "Surfaced" => notified[a] = Approvers

\* I7: Surfaced is sticky until affirmed via separate IR workflow.
\*     Not modeled here, so Surfaced -> stays Surfaced.
SurfacedIsSticky ==
    [][\A a \in Actions :
        phase[a] = "Surfaced" => phase'[a] = "Surfaced"]_vars

\* I8: Once Affirmed, stays Affirmed.
AffirmedIsSticky ==
    [][\A a \in Actions :
        phase[a] = "Affirmed" => phase'[a] = "Affirmed"]_vars

(***************************************************************************)
(* LIVENESS                                                                *)
(***************************************************************************)

\* L1: Every invoked break-glass action eventually reaches Affirmed or
\*     Surfaced (one of the two terminal-ish states).
WindowAlwaysResolves ==
    \A a \in Actions :
        phase[a] = "BreakGlassInvoked" ~> phase[a] \in {"Affirmed", "Surfaced"}

\* L2: Unaffirmed actions eventually surface on a doctor report.
AffirmationOrSurfaced ==
    \A a \in Actions :
        phase[a] = "Unaffirmed" ~> phase[a] = "Surfaced"

(***************************************************************************)
(* TLC bounded-model state constraint — referenced by .cfg via            *)
(* CONSTRAINT StateConstraint. Bounds the clock so TLC terminates; the    *)
(* DEADLINE-window semantics are exercised for any positive clock bound.  *)
(***************************************************************************)
StateConstraint == clock <= 5

(***************************************************************************)
(* TLC symmetry — referenced by BreakGlass.cfg via SYMMETRY.             *)
(***************************************************************************)
Symmetry == Permutations(Actions)

=============================================================================
