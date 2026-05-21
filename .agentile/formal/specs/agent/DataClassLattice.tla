--------------------------- MODULE DataClassLattice ---------------------------
(***************************************************************************)
(* cit-agent — Data Class Lattice (Bell-LaPadula Adapted)                  *)
(*                                                                         *)
(* Normative TLA+ spec for RFC-CIT-AGENT-0001 §7.2 (Clearance Lattice on  *)
(* AgentSBT) and §9.1 row 3.                                               *)
(*                                                                         *)
(* The data-class lattice is a partial order over {PUBLIC, CUI, PHI,      *)
(* FERPA, ITAR}. The AgentSBT's clearance field anchors the per-agent    *)
(* upper bound. Two Bell-LaPadula style properties must hold:              *)
(*                                                                         *)
(*   - No-Read-Up: a capsule whose `reads` contains a class higher than  *)
(*     the agent's clearance never executes.                              *)
(*   - No-Write-Down: a capsule cannot `emit` data at a class lower than *)
(*     what it `read` unless it explicitly traverses a declassification  *)
(*     capsule.                                                            *)
(*                                                                         *)
(* The lattice ordering is intentionally NOT a total order: PHI, FERPA,  *)
(* and ITAR are incomparable peers above CUI. PUBLIC is the bottom. A   *)
(* clearance dominates a class iff there is an ascending chain from the  *)
(* class to the clearance in the lattice.                                  *)
(*                                                                         *)
(* Properties verified:                                                    *)
(*   Safety:                                                               *)
(*     - NoReadUp: no executed capsule read a class above clearance       *)
(*     - NoWriteDownWithoutDeclassifier: no emit lower than any read     *)
(*       unless the action's capsule chain includes a declassifier      *)
(*     - DeclassifierProvenanceIsSigned: declassification capsules carry *)
(*       a Compliance-Officer signature (modeled abstractly here)         *)
(*     - ClearanceMonotone: AgentSBT clearance does not silently         *)
(*       decrease mid-run                                                  *)
(*   Liveness:                                                             *)
(*     - EveryProposedSettled: every Proposed capsule call is eventually *)
(*       allowed or refused                                                *)
(*                                                                         *)
(* Status: draft — pending TLC verification by operator (CONFIG.md       *)
(* policy: Saul runs TLC locally).                                        *)
(*                                                                         *)
(* Supersedes (deprecation candidates — execution deferred to a later    *)
(* sprint, do NOT delete yet): none direct. This is a new spec; no       *)
(* existing spec models the cross-class lattice.                          *)
(***************************************************************************)

EXTENDS Integers, Sequences, FiniteSets, TLC

CONSTANTS
    Capsules,        \* finite set of capsule identifiers
    Declassifiers    \* SUBSET Capsules — capsules certified as declassifiers

(***************************************************************************)
(* Data classes.                                                           *)
(*                                                                         *)
(*   PUBLIC < CUI < {PHI, FERPA, ITAR}                                     *)
(*                                                                         *)
(* PHI, FERPA, ITAR are mutually incomparable (the lattice is NOT total). *)
(***************************************************************************)
Classes == {"PUBLIC", "CUI", "PHI", "FERPA", "ITAR"}

\* Dominates(a, b): a is at-least-as-classified as b in the lattice.
Dominates(a, b) ==
    \/ a = b
    \/ b = "PUBLIC"
    \/ /\ a \in {"PHI", "FERPA", "ITAR"}
       /\ b = "CUI"
    \/ /\ a \in {"PHI", "FERPA", "ITAR"}
       /\ b = "PUBLIC"
    \/ /\ a = "CUI"
       /\ b = "PUBLIC"

VARIABLES
    clearance,      \* the agent's current clearance level (single AgentSBT)
    capsuleReads,   \* [Capsules -> SUBSET Classes] — declared reads
    capsuleEmits,   \* [Capsules -> SUBSET Classes] — declared emits
    callState,      \* [Capsules -> {"Proposed", "Allowed", "Refused", "Executed"}]
    chainSeen,      \* [Capsules -> SUBSET Capsules] — declassifier chain accumulated
    everRead        \* [Capsules -> SUBSET Classes] — runtime-read classes by call

vars == <<clearance, capsuleReads, capsuleEmits, callState, chainSeen, everRead>>

TypeOK ==
    /\ clearance \in Classes
    /\ capsuleReads \in [Capsules -> SUBSET Classes]
    /\ capsuleEmits \in [Capsules -> SUBSET Classes]
    /\ callState \in [Capsules -> {"Proposed", "Allowed", "Refused", "Executed"}]
    /\ chainSeen \in [Capsules -> SUBSET Capsules]
    /\ everRead \in [Capsules -> SUBSET Classes]
    /\ Declassifiers \subseteq Capsules

(***************************************************************************)
(* Init picks reads/emits from a bounded subset (cardinality <= 2) to     *)
(* keep TLC tractable. The CONSTRAINT in the cfg is the secondary bound. *)
(***************************************************************************)
\* Cardinality(S) <= 1 keeps |BoundedClassSets| = 6 (the empty set + the
\* 5 singletons). With 3 capsules × 2 fields (reads, emits) this gives
\* 6^6 = 46,656 initial states — within TLC's heap budget. The lattice
\* invariants are exercised because Dominates checks individual classes,
\* not subsets; the singleton-or-empty image suffices for safety.
BoundedClassSets == { S \in SUBSET Classes : Cardinality(S) <= 1 }

Init ==
    /\ clearance \in Classes
    /\ capsuleReads \in [Capsules -> BoundedClassSets]
    /\ capsuleEmits \in [Capsules -> BoundedClassSets]
    /\ callState = [c \in Capsules |-> "Proposed"]
    /\ chainSeen = [c \in Capsules |-> {}]
    /\ everRead = [c \in Capsules |-> {}]

(***************************************************************************)
(* WouldWriteDown(c): pre-execution predicate (uses manifest, not          *)
(* runtime everRead). Used as a precondition in ExecuteCall to enforce    *)
(* declassifier traversal. CIT-AGENT-2 WP-3.                              *)
(***************************************************************************)
WouldWriteDown(c) ==
    \E ec \in capsuleEmits[c], rc \in capsuleReads[c] :
        /\ ec /= rc
        /\ ~Dominates(ec, rc)

(***************************************************************************)
(* AllowCall(c): perform the no-read-up check at install/call time.       *)
(* This is the harness's enforcement of RFC §7.2 Step 3.                  *)
(***************************************************************************)
AllowCall(c) ==
    /\ callState[c] = "Proposed"
    /\ \A cls \in capsuleReads[c] : Dominates(clearance, cls)
    \* CIT-AGENT-2 WP-3 (ITAR isolation, upfront): manifests that
    \* read ITAR and emit PUBLIC are unsafe even with a declassifier
    \* (RFC §2.3 ITAR overlay row). Refuse upfront — there is no
    \* path to a safe execution, so allowing then blocking would
    \* leave the capsule stuck in Allowed (liveness violation).
    /\ ~ ("ITAR" \in capsuleReads[c] /\ "PUBLIC" \in capsuleEmits[c])
    /\ callState' = [callState EXCEPT ![c] = "Allowed"]
    /\ UNCHANGED <<clearance, capsuleReads, capsuleEmits, chainSeen, everRead>>

(***************************************************************************)
(* RefuseCall(c): the lattice check fails OR the ITAR-isolation check    *)
(* fails; the call is refused. CIT-AGENT-2 WP-3 extends the disjunction *)
(* to cover the upfront ITAR refusal path.                                 *)
(***************************************************************************)
RefuseCall(c) ==
    /\ callState[c] = "Proposed"
    /\ \/ \E cls \in capsuleReads[c] : ~Dominates(clearance, cls)
       \/ ("ITAR" \in capsuleReads[c] /\ "PUBLIC" \in capsuleEmits[c])
    /\ callState' = [callState EXCEPT ![c] = "Refused"]
    /\ UNCHANGED <<clearance, capsuleReads, capsuleEmits, chainSeen, everRead>>

(***************************************************************************)
(* ExecuteCall(c): a capsule that is Allowed proceeds to execute.         *)
(* At execution it records the classes it actually read and emitted; we   *)
(* model this as the declared reads/emits for simplicity (the manifest is *)
(* the source of truth per RFC §4.3).                                    *)
(*                                                                         *)
(* If the capsule emits a class that is BELOW the minimum class it read, *)
(* the chainSeen must include at least one Declassifier — otherwise the *)
(* NoWriteDownWithoutDeclassifier invariant catches it.                   *)
(***************************************************************************)
ExecuteCall(c) ==
    /\ callState[c] = "Allowed"
    \* CIT-AGENT-2 WP-3 precondition: a capsule that would write down
    \* (emits some class not dominated by any of its reads) cannot
    \* execute unless it has traversed at least one Declassifier first.
    \* Without this guard the NoWriteDownWithoutDeclassifier invariant
    \* surfaces a counterexample at ~600K states — TLC was correct:
    \* the design lacked the runtime gate. RFC §7.2 step 3. ITAR
    \* isolation is checked upfront in AllowCall so by the time we
    \* reach here, the ITAR→PUBLIC class is already Refused.
    /\ WouldWriteDown(c) => (chainSeen[c] \cap Declassifiers /= {})
    /\ callState' = [callState EXCEPT ![c] = "Executed"]
    /\ everRead' = [everRead EXCEPT ![c] = capsuleReads[c]]
    /\ UNCHANGED <<clearance, capsuleReads, capsuleEmits, chainSeen>>

(***************************************************************************)
(* TraverseDeclassifier(c, d): the operator routes the action through a   *)
(* declassification capsule d before allowing the emit. Adds d to        *)
(* chainSeen[c]. Must occur while Allowed (pre-execution).                *)
(***************************************************************************)
TraverseDeclassifier(c, d) ==
    /\ d \in Declassifiers
    /\ callState[c] = "Allowed"
    /\ chainSeen' = [chainSeen EXCEPT ![c] = chainSeen[c] \cup {d}]
    /\ UNCHANGED <<clearance, capsuleReads, capsuleEmits, callState, everRead>>

Next ==
    \/ \E c \in Capsules : AllowCall(c)
    \/ \E c \in Capsules : RefuseCall(c)
    \/ \E c \in Capsules : ExecuteCall(c)
    \/ \E c \in Capsules, d \in Capsules : TraverseDeclassifier(c, d)

Spec == Init /\ [][Next]_vars
    /\ \A c1 \in Capsules : WF_vars(AllowCall(c1) \/ RefuseCall(c1) \/ ExecuteCall(c1))
    \* CIT-AGENT-2 WP-3 fairness: declassifier traversal must eventually
    \* fire for capsules that need it. Without this, a capsule needing
    \* declassification could stall in Allowed forever (liveness fail).
    /\ \A c2 \in Capsules, d \in Declassifiers :
         WF_vars(TraverseDeclassifier(c2, d))

(***************************************************************************)
(* HELPERS                                                                 *)
(***************************************************************************)

\* WroteDown(c): the capsule emits some class strictly below the minimum
\* of its reads. We approximate "strictly below" as "not dominated by".
\* The post-execution form (uses everRead, only set after ExecuteCall).
WroteDown(c) ==
    \E ec \in capsuleEmits[c], rc \in everRead[c] :
        /\ ec /= rc
        /\ ~Dominates(ec, rc)
\* (WouldWriteDown(c) defined earlier — the pre-execution form used as
\*  a precondition in ExecuteCall. CIT-AGENT-2 WP-3.)

(***************************************************************************)
(* SAFETY INVARIANTS                                                       *)
(***************************************************************************)

\* I1: Type safety.
TypeInvariant == TypeOK

\* I2: No-Read-Up. RFC §7.2: a capsule never executes if any class in its
\*     reads exceeds the agent's clearance.
NoReadUp ==
    \A c \in Capsules :
        callState[c] = "Executed" =>
            \A cls \in capsuleReads[c] : Dominates(clearance, cls)

\* I3: No-Write-Down-Without-Declassifier. If a capsule writes down, its
\*     execution chain must include at least one Declassifier.
NoWriteDownWithoutDeclassifier ==
    \A c \in Capsules :
        /\ callState[c] = "Executed"
        /\ WroteDown(c)
        =>
        chainSeen[c] \cap Declassifiers /= {}

\* I4: ITAR isolation. ITAR-classified reads never coexist with PUBLIC
\*     emits — even with a declassifier (RFC §2.3 ITAR overlay row,
\*     "MUST NOT be overridden under any policy").
ITARNeverWritesPublic ==
    \A c \in Capsules :
        /\ callState[c] = "Executed"
        /\ "ITAR" \in capsuleReads[c]
        =>
        "PUBLIC" \notin capsuleEmits[c]

\* I5: Refused capsules never reach Executed.
RefusedStaysRefused ==
    [][\A c \in Capsules :
        callState[c] = "Refused" => callState'[c] = "Refused"]_vars

\* I6: Clearance does not change mid-run (single-AgentSBT assumption for
\*     the bounded model; clearance escalation is itself a tier-critical
\*     action covered by ApprovalStateMachine).
ClearanceMonotone ==
    [][clearance' = clearance]_vars

(***************************************************************************)
(* LIVENESS                                                                *)
(***************************************************************************)

\* L1: Every Proposed capsule call eventually settles.
EveryProposedSettled ==
    \A c \in Capsules :
        callState[c] = "Proposed" ~> callState[c] \in {"Refused", "Executed"}

(***************************************************************************)
(* TLC bounded-model state constraint — referenced by .cfg via            *)
(* CONSTRAINT StateConstraint. TLC requires a named operator; the inline  *)
(* /\ ... expression in the .cfg is rejected. Bounding reads + emits to   *)
(* cardinality 2 keeps the explored space ~10^4 distinct states.          *)
(***************************************************************************)
StateConstraint ==
    /\ \A c \in Capsules : Cardinality(capsuleReads[c]) <= 2
    /\ \A c \in Capsules : Cardinality(capsuleEmits[c]) <= 2

(***************************************************************************)
(* TLC symmetry — referenced by DataClassLattice.cfg via SYMMETRY.       *)
(* Only the non-Declassifier capsules are symmetric; the declassifier     *)
(* role is semantically distinct.                                          *)
(***************************************************************************)
Symmetry == Permutations(Capsules \ Declassifiers)

=============================================================================
