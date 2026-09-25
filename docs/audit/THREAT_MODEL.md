---
created: 2026-05-21T00:00:00Z
branch: feat/s-13-tob-engagement
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
audience: Trail of Bits engagement team
---

# Threat Model — nist-agent v1.0-rc

> Companion to [`SCOPE.md`](SCOPE.md). Each section names a
> load-bearing surface, the actors / assets in scope, and the
> STRIDE classes that matter most for that surface. Cross-
> references to the formal specs and feature scenarios are
> kept tight so each claim is traceable to code.

## Actors

| Actor | Role | Surface they can reach |
|---|---|---|
| `Operator` | Day-to-day user of the harness. Initiates capsule calls. | Slint UI, chat surface, CLI. |
| `Reviewer` | Approves low/medium-risk capsule calls. | HIC approval queue (desk or mobile). |
| `ComplianceOfficer` | Co-signs medium/high-risk + audit deletions. | HIC queue, AuditChain. |
| `SecurityOfficer` | Signs policy bundles, mobile pairings, egress directives. | PolicyBundle, pairing flow, egress activation. |
| `Auditor` | Read-only on the audit chain; SoD-excluded from approvals. | AuditChain reads. |
| `Adjacent attacker` | Has network access to the daemon but no role binding. | mTLS endpoint, CapsuleRegistry-read interface. |
| `Compromised dependency` | Trojan in a Cargo dep or a tampered GGUF. | Whatever the dep's API touches. |
| `Compromised insider` | Holds one role's key but not the quorum. | Whatever their role permits *unilaterally* (which should be nothing security-critical). |

## Assets

| Asset | Where it lives | Why it matters |
|---|---|---|
| `SecurityOfficer` private key | HSM (production) / hardware key (operator-side). | Signs every PolicyBundle. Compromise = arbitrary overlay activation. |
| `Reviewer` / `ComplianceOfficer` keys | Hardware keys (FIDO2 / PIV-CAC). | Quorum members; per-action signatures. |
| Active PolicyBundle | Filesystem + (optionally) anchored on `AnchorRegistry`. | The harness's authority on what's permitted. |
| Audit log records | WORM filesystem sink + (optionally) anchored. | Immutable record of every decision. AU-9(5). |
| Capsule manifests | Filesystem; content_hash declared. | Define what a capsule may do; signing tier governs trust. |
| Release bundle | Operator's offline media. | The daemon + GGUF + manifests the operator runs. |
| Bundled GGUF | Filesystem; sha256 in `release.manifest.toml`. | The concierge model. SI-7 integrity is the load-bearing check. |
| Mobile-companion pairing record | Persisted as `AuditRecord`. | Establishes a phone as a signing surface. |

## Load-bearing surfaces — STRIDE per surface

### 1. HIC approval quorum (`crates/nist-agent-hitl` + upstream `ApprovalQueue`)

Formal spec: `.agentile/formal/specs/HITLQuorum.tla`.

| STRIDE | Concrete scenarios we expect to be defended against |
|---|---|
| **S**poofing | A non-`Reviewer` identity signs a low-risk approval. Defense: capsule manifest's `risk.required_roles` drives the signature check; identity → role binding lives in PolicyBundle. |
| **T**ampering | An attacker mutates `ApprovalRow` content between display and sign. Defense: signed payload covers a canonical encoding of the proposal, not the UI's display string. |
| **R**epudiation | An approver signs and later denies. Defense: signature + ISO-8601 timestamp persist as `AuditRecord`. |
| **I**nfo disclosure | Auditor exfiltrates a pending proposal's `args_pretty`. Defense: Auditor is read-only on the audit chain; pending queue is not in the audit chain. |
| **D**oS | Queue fills with rejected proposals. Defense: rate-limit at the capsule-call layer (upstream); rejected proposals close immediately. |
| **E**oP | A Reviewer triggers a high-risk action without ComplianceOfficer co-sign. Defense: SoD lattice + `roles_still_required` check at `approve()`. |

**Where to look hardest:** the seam between `ApprovalQueueView::from_pending` and the harness's signature-state source. The render model takes signature state as a parallel slice (not from upstream's `PendingView`); a harness bug that mis-supplies that slice is a quorum-bypass primitive.

### 2. Overlay activation ratchet (`nist-agent-policy::ActiveOverlays`)

Formal spec: `.agentile/formal/specs/OverlayRatchet.tla`.

| STRIDE | Concrete scenarios |
|---|---|
| **T**ampering | Operator forces `Overlay::CmmcL3` out of the active set to permit a forbidden action. Defense: `remove_with_workflow` refuses `Overlay::CmmcL3` regardless of workflow proof. |
| **E**oP | A second PolicyBundle is loaded that *replaces* the active set with a less-restrictive set. Defense: bundle activation runs through `ActiveOverlays::ratchet_into`, which requires every previously-active overlay to remain in the new set OR carry a valid `DecommissioningWorkflow` token. |
| **R**epudiation | A `OverlayDecommissioned` event is suppressed. Defense: decommissioning workflow is an HIC-gated proposal; the workflow's SHA-256 is itself part of the proof token, so the audit record is content-addressable. |

**Where to look hardest:** `ActiveOverlays::ratchet_into`'s edge cases — empty prior-set, prior-set with only `CmmcL3`, workflow-proof-but-wrong-overlay, time-of-check / time-of-use between two concurrent activation attempts.

### 3. Mobile-companion pairing (`crates/nist-agent-mobile-pairing`)

| STRIDE | Concrete scenarios |
|---|---|
| **S**poofing | A non-`SecurityOfficer` identity initiates a pairing. Defense: `apply_security_officer()` records the DID; the harness verifies the DID against the trust root. |
| **T**ampering | Pairing token replayed against a different `PairingRecord`. Defense: pairing id binds the token to one record; token cleared on activation. |
| **R**epudiation | A device denies signing. Defense: `SignedDecision.signature_hex` over a deterministic digest persists in audit. |
| **I**nfo disclosure | `PairingRecord` leaks the token in subsequent audit records. Defense: token cleared on activation; subsequent state transitions persist without the secret. |
| **D**oS | Adjacent attacker bombards the pairing endpoint with bogus tokens. Defense: `PairingTokenMismatch` is cheap to compute; rate-limiting belongs at the transport layer. |
| **E**oP | A device reaches `Active` without an allowlisted attestation. Defense: `apply_attestation()` is reachable only after `apply_security_officer()`; `AttestationAllowlist::validate_chain` runs before `apply_attestation()` in the harness flow. |

**Where to look hardest:** the constant-time-ness of `PairingToken::matches` (documented as not strictly constant-time), and any harness path that skips `validate_chain` between attestation receipt and `apply_attestation`.

### 4. Audit-chain append-only invariant (`crates/nist-agent-audit-sinks`)

Formal spec: `.agentile/formal/specs/AuditChainAppendOnly.tla`.

| STRIDE | Concrete scenarios |
|---|---|
| **T**ampering | An attacker with filesystem access modifies a prior record. Defense: WORM sink writes via `O_CREAT \| O_EXCL` on a `0600`-mode file in a sticky-bit dir; modify-after-write requires root or the harness's own UID (which the daemon should not have post-init). |
| **R**epudiation | Auditor disputes a record's authenticity. Defense: per-record hash chain — each record carries its predecessor's hash; periodic anchor to `AnchorRegistry` (RFC §6.3). |
| **D**oS | Audit sink filesystem fills. Defense: doctor pre-flight checks free space; rotation is operator-policy. |

**Where to look hardest:** the seam between in-memory `AuditRecord` construction and the WORM write. If the construction can fail mid-flight (e.g. signature error), does the partial state ever land on disk? Same question for the hash-chain link.

### 5. Release verifier (`crates/nist-agent-release`)

Formal spec: `.agentile/formal/specs/ReleaseVerifier.tla`.

| STRIDE | Concrete scenarios |
|---|---|
| **T**ampering | Manifest's artifact list mutated post-signature. Defense: signing payload covers the artifact list; signature verification fails. |
| **E**oP | Confused-deputy on the installer: cause `verify_and_install` to extract content before signature checks complete. Defense: signature check is the first statement; per-artifact hash check is after. |
| **I**nfo disclosure | An attacker extracts the public key from the manifest to forge a signature. Defense: public key is not secret; the secret is the operator-configured trust root. Manifest-declared public key must match trust root. |

**Where to look hardest:** the `signing_payload` reconstruction — if the TOML encoder produces non-deterministic output across machine A and machine B, the signature won't verify even on a valid bundle. We've pinned `signing_payload()` to use the pre-signature-zeroed form; verify the encoder's determinism in your environment.

### 6. Egress posture default + activation (`nist-agent-release::EgressPosture`)

Maps to RFC §3.3 G1.

| STRIDE | Concrete scenarios |
|---|---|
| **E**oP | A harness path performs a network call while posture is `Disabled`. Defense: posture is checked at every network-touching call site; capsules declaring `network` capability are filtered out of the marketplace. |
| **T**ampering | An `EgressDirective` is replayed to re-enable egress after a revert. Defense: the directive's `asserted_at_iso` is part of the signed payload; the harness records activation timestamps and refuses past-timestamped directives older than the most recent activation/revert. (Implementation lives in the operator's `verifier` callback per ADR-011.) |

**Where to look hardest:** the operator-supplied `verifier` callback for `EgressDirective`. The release crate accepts any `FnOnce(&EgressDirective) -> bool`; if an operator's callback returns `true` without checking the signature, egress flips on bogus input. The packet's reference deployment uses a callback that delegates to `nist-agent-policy::verify_security_officer`; deviate at your peril.

### 7. GGUF SI-7 integrity (`nist-agent-release::ModelIntegrity`)

| STRIDE | Concrete scenarios |
|---|---|
| **T**ampering | Bundled GGUF replaced on disk by a tampered copy. Defense: `verify_gguf()` hashes the bytes and refuses with `"SI-7: model hash mismatch"`. |
| **E**oP | A capsule reads the GGUF and uses cached embeddings without re-verification. Defense: the load path always re-verifies; cached embeddings are downstream of a verified load. |

**Where to look hardest:** any code path that loads model bytes without going through `ModelIntegrity::verify_gguf` or `verify_gguf_digest`. There should be none in the audit boundary; flag any that exist.

### 8. Agent loop + checkpoint store (`crates/nist-agent-loop`)

| STRIDE | Concrete scenarios |
|---|---|
| **T**ampering | A checkpoint is modified to alter the action proposal between `step` and `resume`. Defense: `Checkpoint::derive_id` is content-addressable over the action; resume against a mismatched id refuses. |
| **R**epudiation | Agent denies producing a particular action proposal. Defense: checkpoint persists with the full completion text + action. |
| **E**oP | The model's `<<ACTION ...>>` proposal syntax is forged in a way that bypasses HIC. Defense: `parse_action` is the single entry point; any `AgentOutcome::Pending` writes a checkpoint *before* returning. |

**Where to look hardest:** the `<<ACTION cap.fn args>>` parser. Anything in the model's free-form completion that contains those bytes becomes a proposal; consider injection where the operator's prompt contains the syntax to confuse the parser, or where the model echoes a prior prompt's syntax.

## Data-class lattice (Bell-LaPadula)

Formal spec: `.agentile/formal/specs/DataClassLattice.tla`.

`PUBLIC < CUI < PHI < FERPA < ITAR`. The lattice is the
no-read-up / no-write-down enforcer:

- A capsule whose `data_class.reads = [CUI]` cannot read `PHI`.
- A capsule whose `data_class.writes = [PHI]` cannot write `CUI`.
- An action whose `emits = [FERPA]` may not be ingested by a
  downstream capsule whose `reads ≠ [FERPA, ITAR]`.

**Where to look hardest:** the byte-alignment between
`DataClass` (used in capsule manifests) and `Clearance` (used
in role binding). They are cross-pinned by test (S-8); verify
the pin holds at every consumer.

## Threat-model boundaries we explicitly accept

- **Compromised RNG.** We trust the operator's host RNG. A
  compromised RNG breaks every protocol; not in scope for this
  engagement.
- **HSM compromise.** A compromised SecurityOfficer HSM defeats
  every downstream defense. Mitigated by HSM key custody policy
  (out of scope for this engagement; ADR-011).
- **Compromised GGUF supplier.** A tampered GGUF that
  reproduces a target sha256 is a SHA-256 collision; if you
  find a collision, we have a much bigger problem.
- **Supply-chain trojans in transitively pinned crates.** The
  federation manifest pins `citrate-agent-core` and
  `citrate-wallet-core` by rev; transitive deps are pinned by
  Cargo.lock. SBOM enumeration is in `BUILD.md`.
- **Side channels on shared hardware.** The reference
  deployment is single-tenant; multi-tenant deployments are an
  operator concern.

## See also

- [`SCOPE.md`](SCOPE.md) — file inventory + what's in/out.
- [`.agentile/formal/specs/`](../../.agentile/formal/specs/) — the five normative TLA+ specs.
- [`features/`](../../features/) — Gherkin features; each surface above has a corresponding feature file.
- RFC-CIT-AGENT-0001 §§3.3, 5, 6, 8 — the architecture chapters this threat model annotates.
