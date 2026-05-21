--------------------------- MODULE AuditChainIntegrity ---------------------------
(***************************************************************************)
(* cit-agent — Audit Chain Integrity                                       *)
(*                                                                         *)
(* Normative TLA+ spec for RFC-CIT-AGENT-0001 §6 (Audit Chain and Record  *)
(* Keeping) and §9.1 row 2.                                                *)
(*                                                                         *)
(* Models the hash-chained, signed AuditRecord stream produced by the     *)
(* cit-agent harness. Each record carries a `previous_hash` linking it to *)
(* the prior record, a CBOR-canonical `payload`, and one or more         *)
(* `signatures`. The harness MUST guarantee:                               *)
(*                                                                         *)
(*   - Append-only: records are never deleted or reordered                *)
(*   - Hash contiguity: previous_hash on record N == hash(record N-1)     *)
(*   - Signature validity: every signature verifies against the canonical *)
(*     CBOR encoding of the record it signs                               *)
(*   - No dangling previous_hash: no record's previous_hash points        *)
(*     outside the chain                                                  *)
(*                                                                         *)
(* Crypto is abstracted: `hash` and `sigVerify` are uninterpreted         *)
(* functions modeled as injective identities over their inputs. The spec  *)
(* therefore checks the STRUCTURAL properties of the chain — that the    *)
(* link, signing, and append-only discipline are preserved by every     *)
(* admitted transition. Crypto-soundness is verified separately by aws-  *)
(* lc-rs / FIPS suite testing per RFC §3.3 and §2.2 SC-13.                *)
(*                                                                         *)
(* Status: draft — pending TLC verification by operator (CONFIG.md       *)
(* policy: Saul runs TLC locally).                                        *)
(*                                                                         *)
(* Supersedes (deprecation candidates — execution deferred to a later    *)
(* sprint, do NOT delete yet):                                            *)
(*   - AuditTrailIntegrity.tla : the prior spec models append-only +     *)
(*     monotonic timestamps but does NOT model the hash-chain link or    *)
(*     signature attestation. Keep until cit-agent v1.0 ships and the    *)
(*     legacy Hermes audit trail is decommissioned.                      *)
(*   - TrailAppendOnly.tla : strictly weaker than this spec; subsumed by *)
(*     the AppendOnly invariant here.                                     *)
(***************************************************************************)

EXTENDS Integers, Sequences, FiniteSets, TLC

CONSTANTS
    Payloads,    \* finite set of opaque payload identifiers
    Signers,     \* finite set of signer DIDs (subset of role lattice)
    MaxLen,      \* maximum chain length for bounded model checking
    GenesisHash  \* the all-zeros hash sentinel (record 0's `previous_hash`)

(***************************************************************************)
(* A Record is the in-spec abstraction of the on-the-wire AuditRecord.    *)
(* `previous_hash` references the prior record's identity hash, computed *)
(* via the abstract Hash() operator. `sigs` is the set of (signer,       *)
(* signed-over-record-hash) pairs.                                        *)
(***************************************************************************)
Record == [
    seq      : Nat,
    payload  : Payloads,
    prevHash : Nat,      \* abstract hash codomain
    selfHash : Nat,
    sigs     : SUBSET [signer : Signers, over : Nat]
]

VARIABLES
    chain,    \* Seq(Record) — the audit log
    nextHash  \* Nat — monotonic counter we use as the abstract hash codomain

vars == <<chain, nextHash>>

(***************************************************************************)
(* Hash(payload, prev, seq) is modeled as an injective identity returning *)
(* a fresh Nat. We implement this by allocating `nextHash` and            *)
(* incrementing it. Two different (payload, prev, seq) triples receive   *)
(* different hashes; the same triple appended twice would receive the    *)
(* same hash in reality, but the spec only admits append (not replay),   *)
(* so collision modeling is unnecessary.                                   *)
(***************************************************************************)

(***************************************************************************)
(* TypeOK avoids `chain \in Seq(Record)` because TLC would attempt to     *)
(* enumerate the infinite domain Nat. Instead we type-check each record  *)
(* field shape inline.                                                    *)
(***************************************************************************)
TypeOK ==
    /\ Len(chain) <= MaxLen
    /\ nextHash \in Nat
    /\ \A i \in 1..Len(chain) :
        /\ chain[i].seq = i
        /\ chain[i].payload \in Payloads
        /\ chain[i].selfHash < nextHash
        /\ chain[i].selfHash \in Nat
        /\ chain[i].prevHash \in Nat
        /\ \A s \in chain[i].sigs : s.signer \in Signers /\ s.over \in Nat

Init ==
    /\ chain = <<>>
    /\ nextHash = GenesisHash + 1

(***************************************************************************)
(* AppendRecord(p, S):                                                     *)
(*   - p \in Payloads, S \subseteq Signers                                 *)
(*   - The new record's previous_hash is the prior record's selfHash      *)
(*     (or GenesisHash if the chain is empty).                            *)
(*   - The new record's selfHash is freshly allocated.                    *)
(*   - Every signer in S signs over the new record's selfHash —          *)
(*     modeling that signatures bind to the canonical CBOR of the          *)
(*     record they sign.                                                  *)
(***************************************************************************)
AppendRecord(p, S) ==
    /\ Len(chain) < MaxLen
    /\ p \in Payloads
    /\ S \subseteq Signers
    /\ S /= {}     \* every record must carry at least one signature
    /\ LET prevHash == IF chain = <<>> THEN GenesisHash ELSE chain[Len(chain)].selfHash
           newSelfHash == nextHash
           rec == [
               seq |-> Len(chain) + 1,
               payload |-> p,
               prevHash |-> prevHash,
               selfHash |-> newSelfHash,
               sigs |-> { [signer |-> s, over |-> newSelfHash] : s \in S }
           ]
       IN /\ chain' = Append(chain, rec)
          /\ nextHash' = nextHash + 1

Next == \E p \in Payloads, S \in (SUBSET Signers) \ {{}} : AppendRecord(p, S)

Spec == Init /\ [][Next]_vars /\ WF_vars(Next)

(***************************************************************************)
(* SAFETY INVARIANTS                                                       *)
(***************************************************************************)

\* I1: Type safety.
TypeInvariant == TypeOK

\* I2: Append-only — the chain length only grows. Encoded as a temporal
\*     property since it ranges over the next-state relation.
AppendOnly ==
    [][Len(chain') >= Len(chain)]_vars

\* I3: Sequence numbers are contiguous starting at 1.
SequenceContiguity ==
    \A i \in 1..Len(chain) : chain[i].seq = i

\* I4: Hash contiguity — every record's prevHash equals the prior
\*     record's selfHash (or GenesisHash for record 1).
ChainContiguity ==
    \A i \in 1..Len(chain) :
        chain[i].prevHash = IF i = 1 THEN GenesisHash ELSE chain[i-1].selfHash

\* I5: Signature validity — every signature in a record signs over that
\*     record's selfHash. Models: signatures bind to the canonical CBOR
\*     of the record they attest.
SignatureValidity ==
    \A i \in 1..Len(chain) :
        \A sig \in chain[i].sigs : sig.over = chain[i].selfHash

\* I6: No record has a dangling prevHash — every prevHash references
\*     either GenesisHash or a selfHash that exists earlier in the chain.
NoDanglingPrevHash ==
    \A i \in 1..Len(chain) :
        \/ chain[i].prevHash = GenesisHash
        \/ \E j \in 1..(i-1) : chain[j].selfHash = chain[i].prevHash

\* I7: At least one signature per record (non-repudiation, RFC §6.1 +
\*     AU-10).
RecordsAreSigned ==
    \A i \in 1..Len(chain) : chain[i].sigs /= {}

(***************************************************************************)
(* LIVENESS                                                                *)
(***************************************************************************)

\* L1: As long as MaxLen has room and a payload is available, the chain
\*     can grow. Bounded liveness — eventually the chain reaches some
\*     positive length (or it terminates at MaxLen).
EventuallyGrows ==
    <> (Len(chain) > 0)

(***************************************************************************)
(* TLC bounded-model state constraint — referenced by .cfg via            *)
(* CONSTRAINT StateConstraint. TLC requires a named operator here; an     *)
(* inline expression in the .cfg is rejected as "constraint not defined". *)
(***************************************************************************)
StateConstraint == nextHash <= MaxLen + 1

(***************************************************************************)
(* TLC symmetry — referenced by AuditChainIntegrity.cfg via SYMMETRY.    *)
(***************************************************************************)
\* NOTE: only Payloads form a symmetry orbit. Signers are TLC strings
\* (not model values), so Permutations() on them is rejected with
\* "Symmetry function must have model values as domain and range".
\* If Signers ever needs to be in the symmetry group, declare it as
\* model values in the cfg (CONSTANTS Signers = {s_op, s_rv, s_co}).
Symmetry == Permutations(Payloads)

=============================================================================
