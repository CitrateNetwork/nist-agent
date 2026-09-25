---
created: 2026-05-14T00:00:00Z
branch: main
author: Saul Loveman (Citrate Inc.) + Claude (Anthropic)
status: active
rfc: CIT-AGENT-0001
version: v0.1
custodian: Citrate Inc.
source: RFC-CIT-AGENT-0001-v0.1(1).docx
---

RFC-CIT-AGENT-0001

Citrate Agent

Architecture Reference for a NIST-Compliant Agent Harness on the Citrate Network

Status: v0.1 — Draft for Quorum Review

Date: 2026-05-14

Authors: Saul Loveman (Citrate Inc.), with Claude (Anthropic)

Custodian: Citrate Inc.

Compliance Baseline: NIST SP 800-171 + CMMC L3 (CUI/DIB)

Overlays: FERPA · COPPA · CIPA · HIPAA · HITECH · FedRAMP High · DoD IL4–IL5 · ITAR · CJIS · IRS 1075

Default Network Posture: Air-Gapped, Opt-In Egress

# Abstract

This document specifies the architecture of Citrate Agent (cit-agent), a Rust-implemented, NIST-compliant agent harness designed to operate inside organizational nodes on the Citrate Network (chain id 40204) and as a standalone bolt-on for arbitrary on-premise or air-gapped environments. The harness composes three substrate technologies — the WebAssembly Component Model for capability-sandboxed skills, TLA+ for formal verification of safety-critical control flow, and the Citrate L1 chain for tamper-evident attestation of agent state — into a single coherent product that satisfies NIST SP 800-171 + CMMC Level 3 as its baseline, with first-class overlay profiles for FERPA, HIPAA, FedRAMP High, ITAR, and adjacent regulatory regimes.

The design draws structural lessons from two prior-art systems — NousResearch's Hermes Agent (single-loop agent core, signed-skill compatibility) and the OpenClaw project (manifest-driven plugin architecture, pre-flight compliance checking) — and reimplements their patterns in a memory-safe, formally specified, FIPS-friendly form. The result is an agent that can operate under DCMA DIBCAC assessment scrutiny without sacrificing the ergonomics that made its inspirations useful.

Capabilities, procedures, and the cryptographic record of their use are unified as Capsules: signed WIT/WASM components bundled with their procedural rules of engagement and their formal specifications. Every action the agent proposes traverses a tiered-risk, role-bound quorum approval gate before execution; every approval is cryptographically signed, hash-chained locally, and optionally anchored on-chain as a Merkle root. Sensitive content never touches the chain; only commits do.

This specification is normative for the v1.0 release of cit-agent and the contracts at canonical addresses on Citrate Mainnet. Sections marked as Informative provide design rationale and may evolve.

# Document Status


Required signatories before v1.0-rc promotion:

- Saul Loveman / Larry Klosowski — Principal Architect, Citrate Network
- Taurien Buffaloe — Legal review (Greenberg Traurig)
- Lauren Mendenhall — IP / publication review
- Reed Rissberger or Kurt McKelvey — Dandi Health / healthcare overlay review
- External: Trail of Bits engagement lead — security review

# Conventions Used in This Document

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in BCP 14 (RFC 2119 and RFC 8174) when, and only when, they appear in all capitals.

Code blocks, configuration excerpts, and identifiers use a monospace font. On-chain addresses are written in lowercase hexadecimal with the 0x prefix. Hashes shown in examples are illustrative and not canonical.

This document follows the IETF RFC tradition of separating Normative requirements from Informative commentary. Sections explicitly labeled Informative may be cited but not relied upon as compliance evidence.

# 1. Introduction

## 1.1 Problem Statement

Organizations operating in regulated environments — defense industrial base contractors handling CUI, school districts handling FERPA-protected records, healthcare providers handling PHI, and government agencies handling classified-adjacent information — need to deploy AI agents to capture the productivity gains that have become competitively significant in 2026. The available open-source agent frameworks (Hermes Agent, OpenClaw, LangGraph, and others) provide compelling user experience but were not designed to a compliance bar. Their architectures assume environments their target deployments cannot legally inhabit: cloud-hosted memory, vendor-controlled model endpoints, plaintext message bus telemetry, and skill-creation loops that mutate the agent's authorized capability surface after deployment.

The available compliance-oriented agent platforms (proprietary, typically built on managed cloud services like AWS GovCloud or Azure Government) are either too expensive, too coarse-grained in their policy model, or too dependent on vendor lock-in to serve the wider regulated population — particularly smaller defense subcontractors and resource-constrained public-sector deployments like school districts.

The result is a population of organizations that are either deploying agents in violation of their compliance posture, declining to deploy at all, or building bespoke wrappers that they cannot afford to maintain. Citrate Agent exists to give this population a single, formally specified, open-core agent harness that satisfies NIST SP 800-171 + CMMC Level 3 as a baseline and admits overlay profiles for the surrounding regulatory family without re-architecture.

## 1.2 Goals

Citrate Agent v1.0 SHALL satisfy the following goals:

- G1 Operate by default in an air-gapped configuration. No outbound network connectivity is required at first run, at any subsequent run, or during any standard workflow. Egress is policy-gated and per-action.
- G2 Treat human approval as a first-class typed effect, not an out-of-band convention. Every action the agent proposes traverses a state-managed interrupt with cryptographic sign-off from designated roles, scaled by risk tier and data class.
- G3 Encode capabilities, procedures, and formal proofs as a single signed unit (a Capsule). A Capsule cannot grant itself capability beyond what its signed manifest declares, and the harness physically cannot execute one whose declared capabilities exceed the operator's policy.
- G4 Produce auditor-ready evidence by construction, not by retrofit. A nightly pre-flight check (“doctor”) and an audit-time grader produce signed reports that map directly onto NIST 800-171 / CMMC L3 assessment objectives.
- G5 Compose as a library, deploy as a daemon, embed as a WASM component. The Citrate node embeds the harness with no IPC overhead; standalone operators run a daemon with mTLS; future browser and embedded targets compile to the Component Model directly.
- G6 Anchor optionally and minimally to the Citrate L1. On-chain footprint is operator-selectable across three strategies. No CUI, PHI, FERPA-protected, ITAR-controlled, or other regulated content ever touches the chain — only commits and Merkle roots.
- G7 Verify, do not assume. Safety-critical control flow (the HIC approval state machine, the audit chain integrity property, the data-class lattice, the capsule install gate, the break-glass path) is formally specified in TLA+ and model-checked in CI as a BLOCKER ratchet per Agentile rule 10.
## 1.3 Non-Goals

The following are explicitly out of scope for v1.0 and SHALL NOT be implemented:

- N1 Free-form subagent delegation. A capsule SHALL NOT spawn arbitrary subagents at its own discretion. v2 may admit policy-gated subagent delegation; v1 does not.
- N2 Autonomous capsule creation. The agent SHALL NOT mint new capsules at runtime. Capsule authoring is a separate engineering process subject to its own Agentile-governed signing pipeline.
- N3 Persistent cross-session learning that mutates capability surface. The agent MAY accumulate facts in a memory store, but those facts SHALL NOT alter the set of capabilities the agent is authorized to invoke.
- N4 Solana / SVM-native skills. v1 is EVM-only. An SVM compatibility appendix is reserved for a future RFC.
- N5 Multi-tenant cloud hosting. The harness is single-operator-per-instance by design. SaaS deployments by third parties are permitted under the commercial license but are not the primary product.
## 1.4 Relationship to Prior Art

Citrate Agent is informed by, and consciously distinct from, two prior-art systems in the open-source agent landscape.


From Hermes Agent (NousResearch), Citrate Agent adopts:

- The single AIAgent loop pattern — one Agent type drives every surface (CLI, daemon, WASM host, Slint app).
- Skill manifest compatibility with the agentskills.io open standard at the metadata layer (file-name and frontmatter compatibility), allowing operators to import existing Hermes-format skill bundles after capability-signing them as Capsules.
- Trajectory export discipline for training-data generation, subject to the policy bundle and explicit data-class declarations.
From Hermes Agent, Citrate Agent consciously diverges on:

- Subagent delegation — Hermes ships freewheeling delegation by default; cit-agent forbids it in v1.
- Skill self-improvement at runtime — Hermes treats this as a feature; cit-agent treats it as a capability-surface mutation that fails the CMMC L3 baseline.
- Implementation language — Hermes is Python; cit-agent is Rust to support FIPS-validated crypto and memory-safety arguments to assessors.
From OpenClaw, Citrate Agent adopts:

- The doctor pre-flight pattern (highest-ranked adoption). openclaw doctor surfaces risky or misconfigured policies before runtime. In cit-agent, this becomes the daily continuous-monitoring artifact for CA-family (Assessment, Authorization, Monitoring) and SI-family (System and Information Integrity) controls.
- Bundled / managed / workspace skill tiering. Three signing tiers for capsules: bundled (Citrate-signed), managed (org-procurement-signed), workspace (operator-signed).
- Manifest-driven capability contracts. Adopted in a stronger form via WIT/WASM, where capability declarations are enforced by the linker, not the runtime.
From OpenClaw, Citrate Agent consciously diverges on:

- TypeScript implementation — replaced with Rust for the reasons above.
- Channel multiplicity — OpenClaw fans out across 20+ messaging platforms; cit-agent's surfaces are four (Slint, CLI, local web UI, mobile companion), all driving the same local Approval Service.
- Plugin auto-installation — cit-agent capsule installation requires HIC approval. No silent install paths exist.
# 2. Compliance Mapping

## 2.1 Baseline

Citrate Agent's baseline compliance posture is NIST SP 800-171 Revision 3 + CMMC Level 3 (32 CFR § 170.14(c)(4)). This baseline applies to all deployments regardless of overlay selection. The baseline encompasses 110 NIST SP 800-171 requirements at CMMC L2 plus 24 enhanced requirements from NIST SP 800-172 that constitute CMMC L3.

Selection rationale: CMMC L3 is the most stringent generally-applicable baseline for nonfederal organizations handling CUI; satisfying it positions the harness for downstream FedRAMP, ITAR, and HIPAA overlays with overlay-specific delta work only, not architectural rework. Operating CMMC L3 by default means an operator who needs less stringent compliance can dial it down through the policy bundle; an operator who needs more (e.g., DoD IL5) makes a smaller jump than from a FedRAMP-Moderate baseline.

## 2.2 Control Family Coverage

The following table maps the most load-bearing NIST SP 800-53 Revision 5 control families to the cit-agent components that implement them. This mapping is normative for v1.0 and is the artifact reviewed by external assessors.



In addition to the baseline mapping above, the architecture is informed by NIST's COSAiS (Control Overlays for Securing AI Systems) project, whose gap analysis identified AC, IA, AU, and SI as the families with the most severe deficiencies when standard SP 800-53 controls are applied to AI agent deployments. Citrate Agent's design prioritizes these families.

## 2.3 Overlay Profiles

Each regulatory overlay is implemented as a signed policy bundle that augments the baseline. Overlays MAY restrict the baseline (forbid actions the baseline would permit), MAY require additional role signatures, and MAY require additional audit evidence — but MAY NOT relax baseline controls.



Overlay activation is a one-way ratchet within a deployment: an overlay may be added by the Security Officer through the standard policy bundle update flow, but removal requires a documented decommissioning workflow and a final audit export of the period during which the overlay applied. This prevents operator-introduced gaps in the evidence chain.

# 3. Architecture Overview

## 3.1 High-Level Topology

Citrate Agent decomposes into seven major subsystems plus the on-chain registry surface. The library crate citrate-agent-core is the canonical implementation; all other subsystems either embed it or thinly wrap it.

+----------------------------------------------------------------------+
|                       Surfaces (thin clients)                        |
|   Slint App   |   CLI   |   Local Web UI   |   Mobile Companion      |
+------+------------+------------+------------+------------------------+
       |            |            |            |
       +------------+------------+------------+
                    | (in-process API or mTLS)
                    v
+----------------------------------------------------------------------+
|                       citrate-agent-core  (Rust library)             |
|                                                                      |
|   +-------------+ +------------+ +-------------+ +----------------+  |
|   | Agent Loop  | | Policy Eng | | HIC Queue   | | Audit Chain    |  |
|   | (single)    | | + Data     | | + State Mgr | | (hash-chained, |  |
|   |             | |   Class    | | + Approval  | |  per-record    |  |
|   |             | |   Lattice  | |   Signing   | |  signed)       |  |
|   +-------------+ +------------+ +-------------+ +----------------+  |
|                                                                      |
|   +-------------+ +------------+ +-------------+ +----------------+  |
|   | Capsule     | | Model      | | Chain       | | Doctor /       |  |
|   | Loader      | | Resolver   | | Client      | | Continuous     |  |
|   | (WIT/WASM)  | | (Ollama /  | | (EVM,       | | Monitoring     |  |
|   |             | |  llama.cpp)| |  AnchorReg) | | Reporter       |  |
|   +-------------+ +------------+ +-------------+ +----------------+  |
+----------------------------------------------------------------------+
                    |                            |
                    v                            v
+----------------------------------+  +-------------------------------+
|       wasmtime Sandbox           |  |       Citrate L1 Chain        |
|   (per-capsule, capability-      |  |   OrganizationSBT             |
|    typed, deterministic exec)    |  |   AgentSBT                    |
|                                  |  |   CapsuleRegistry (ERC-1155)  |
|   Secondary: nsjail / Apptainer  |  |   AnchorRegistry              |
|   for host-process capsules      |  |   BenchmarkRegistry           |
+----------------------------------+  +-------------------------------+

## 3.2 The citrate-agent-core Library Crate

citrate-agent-core is a Rust library crate (cdylib + rlib) implementing every load-bearing element of the harness. It exposes:

- Agent — the single agent type, parameterized by a Model trait, a Policy, and an AuditSink. Driven by step() in a token-by-token streaming loop.
- Capsule — the type representing a loaded, signature-verified capsule. Capsule::call(...) returns a Future that traverses the HIC gate before executing the underlying WASM.
- PolicyBundle — a signed, versioned configuration object containing risk-tier mappings, role lattice, overlay activations, anchor strategy, and egress posture.
- ApprovalQueue — the local state-managed interrupt store. Persists pending actions to operator-configured storage; surfaces them to the four surfaces; resumes the agent loop on approval.
- AuditChain — append-only hash-chained record store. Every entry: { previous_hash, payload, signatures[], anchor_root? }.
- Doctor — the pre-flight check runner. Produces a signed report and an on-chain commitment when policy allows.
citrate-agentd (the daemon binary) wraps citrate-agent-core behind a Unix-domain-socket gRPC interface for ops use and an optional mTLS endpoint for the mobile companion. citrate-agent-cli wraps the same library for one-shot command-line use. citrate-agent-slint embeds the library directly with no IPC.

## 3.3 Network Posture and Air-Gap Discipline

By default and unless overridden by an explicit signed policy directive, cit-agent operates with no outbound network connectivity. Concretely:

- At first run and on every subsequent start, the binary establishes that the configured egress broker (if any) is disabled and refuses any tool, capsule, or model call requiring network until an operator policy explicitly enables it.
- Model resolution prefers, in order: local Ollama or llama.cpp instance, embedded bundled model, configured site-mirror, on-chain registry pull (only if egress is permitted), external pull (only if egress AND an explicit external-source policy AND a per-pull Security Officer signature).
- Capsule installation prefers, in order: bundled (shipped with the binary), local site-mirror (installer-configured at deployment), CapsuleRegistry pull (only if egress permitted).
- Chain anchoring (AnchorRegistry writes) is the only outbound traffic that the harness ever generates without an in-flight capsule context, and it is governed by the operator's chosen anchor strategy.
- Any network attempt by a capsule that does not declare network capability in its manifest causes the wasmtime sandbox to refuse the import at link time; the capsule fails to load.
Air-gap mode is the assumed baseline. Operators in connected environments configure their way out of it; operators in disconnected environments do not have to configure their way into it.

# 4. The Capsule Model

## 4.1 Definition

A Capsule is the unified representation of a capability and its procedural rules of engagement, packaged as a single content-addressed, cryptographically signed unit. Capsules are the only mechanism by which the agent can perform actions that have side effects.

Capsules MUST be the unit of distribution, the unit of signing, the unit of capability declaration, and the unit of HIC gating. There SHALL be no out-of-band mechanism by which an agent gains capability.

## 4.2 Capsule Composition

A Capsule is a content-addressed bundle (.cps extension) containing the following files at fixed paths:

<capsule-name>-<version>.cps
+-- capsule.wit              # WIT interface — typed inputs/outputs/imports
+-- capsule.wasm             # compiled WASM component, deterministic build
+-- procedure.md             # SOP — human-authored, gates the agent
+-- manifest.toml            # signed metadata (the policy source of truth)
+-- gherkin/                 # *.feature scenarios (Agentile rule 10 evidence)
+-- specs/                   # optional TLA+ specs (REQUIRED for tier-high)
+-- SIGNATURES/
    +-- publisher.sig        # detached signature over the bundle hash
    +-- reviewer.sig         # REQUIRED for tier-high; signed by second key

## 4.3 Manifest Schema

The capsule manifest is the source of truth for every runtime policy decision. The harness MUST NOT make policy decisions based on the WASM bytecode, the WIT interface, or the procedure.md — only on the signed manifest. The WIT and the WASM are validated against the manifest at load time and rejected if they do not match.

[capsule]
name = "ferpa-redact-student-record"
version = "0.3.1"
content_hash = "sha256:..."

[capability]
network = "none"                    # none | broker-only | egress-allowed
filesystem = ["read:/data/students"]
chain_calls = ["model_inference:0x0101"]
subagent_spawn = false              # always false in v1

[data_class]
reads = ["FERPA-restricted", "FERPA-directory"]
writes = ["FERPA-directory"]
emits = ["PUBLIC"]

[risk]
tier = "high"                       # low | medium | high | critical
required_roles = ["Reviewer", "ComplianceOfficer"]
break_glass_eligible = false

[overlay]
certified = ["FERPA", "COPPA"]
not_certified = ["ITAR"]

[procedure]
gates = [
    { step = "before_read", role = "Operator" },
    { step = "before_emit", role = "ComplianceOfficer" },
]

[provenance]
publisher = "did:citrate:agent:0xab12..."
build_reproducible = true
agentile_sprint = "2026-05-14-sprint-7-ferpa-redact"
tla_spec = "specs/FerpaRedact.tla"

[signing]
tier = "bundled"                    # bundled | managed | workspace

## 4.4 The Three Signing Tiers

Capsules are classified by the trust tier of their signature chain. The tier governs what overlays the capsule may be activated under and what additional checks the harness performs at load.


## 4.5 Capability Enforcement via WIT/WASM

Capability enforcement in cit-agent is physical, not advisory. The capsule's WIT interface declares its imports. The wasmtime linker is configured to provide host functions only for imports that the capsule's manifest authorizes. A capsule whose WASM imports wasi:sockets but whose manifest declares network = "none" fails to instantiate; there is no codepath that admits the capsule to a running state.

The harness MUST treat capability enforcement as a load-time invariant, not a runtime guard. Specifically:

- Manifest signature verification MUST succeed before WASM parsing begins.
- WIT interface MUST match the declared capability set; mismatches fail load with a typed error.
- wasmtime engine MUST be configured per-capsule with a host functions table built from the manifest, not from a default table that is then filtered.
- The capsule's exported functions MUST be called only with arguments whose data_class is dominated by the operator's clearance level and the capsule's declared reads.
# 5. Human In Control (HIC) Approval

## 5.1 Approval Model

Citrate Agent implements a Tiered-Risk + Role-Bound Quorum approval model with state-managed interrupts. Every proposed action passes through the model before execution; no codepath exists by which an action with side effects can be executed without traversing the model.

The model is the union of four NIST SP 800-53 controls, implemented as one mechanism:

- AC-3(2) Dual Authorization — two authorized individuals required to execute designated privileged commands.
- AC-5 Separation of Duties — roles are designed so collusion is hard; audit administrators cannot also be approvers.
- AC-6 Least Privilege — every action is gated to the minimum role that needs it.
- AU-9(5) Dual Authorization for Audit — audit-log modifications require two-person approval.
## 5.2 Risk Tiers

Every action is assigned a risk tier, derived primarily from the calling capsule's manifest [risk] section, optionally escalated by the policy bundle, and never de-escalated. The tier determines the number and roles of required approvers.


## 5.3 The Five-Role Lattice

Citrate Agent defines a five-role minimum lattice. Operators MAY define additional org-specific roles; the harness MUST refuse to operate without all five base roles assigned.


## 5.4 State-Managed Interrupt

When the agent loop encounters an action that requires HIC approval, it does not block the thread or busy-wait. The loop persists the full action context — the proposed call, its arguments, the capsule context, the agent's local state, the inputs that led to the proposal — to the operator-configured Approval Queue storage, returns control to the harness, and surfaces the pending approval to all configured surfaces (Slint, CLI, web, mobile).

When the approval (or rejection) returns, the harness resumes the agent loop from the checkpoint with the approver's decision and signatures wired into the action's audit record. The resumed agent does not re-plan from scratch; it continues exactly from the proposal point. This is the LangGraph-style state-managed interrupt pattern, made cryptographically auditable.

Resumption MUST be deterministic with respect to the checkpoint: given the same checkpoint state and the same approval payload, the resumed agent MUST produce the same first action. This determinism is what allows the audit chain to record the proposal hash and the resumption hash and have them verifiable offline.

## 5.5 Break-Glass Emergency Path

CMMC L3 and NIST SP 800-172 explicitly contemplate emergency response scenarios where dual authorization itself could be a hazard (e.g., active incident containment). Citrate Agent provides a break-glass path subject to the following constraints:

- Break-glass invocation MUST be explicitly enabled per-action in the calling capsule's manifest (break_glass_eligible = true).
- Break-glass approval MAY be issued by a single Security Officer.
- Break-glass actions MUST trigger an immediate notification to every approver in the configured role lattice.
- Within a configurable window (default 72 hours), a post-hoc quorum (Reviewer + Compliance Officer) MUST affirm the action or it is flagged on the next doctor report as an unaffirmed break-glass.
- Unaffirmed break-glass actions MUST be enumerated on every subsequent doctor report until either affirmed or formally remediated through an incident-response workflow.
- Break-glass capsules MUST NOT be eligible to touch ITAR-classified data under any policy bundle.
## 5.6 Signing Surfaces

Approvers sign approvals using any of three hardware-backed paths, configured by the operator's policy bundle:

- Hardware Security Keys (FIDO2 / YubiKey 5 series). Default for low- and medium-risk; acceptable for high-risk under most overlays except ITAR / IL5.
- PIV / CAC Smart Cards. Required for FedRAMP High, DoD IL4/IL5, ITAR, CJIS, IRS 1075 overlays. Verified against operator's configured CA chain.
- Local OS Keychain (Secure Enclave / TPM / StrongBox). Permitted only for low-risk actions on the Operator role; never sufficient for Compliance Officer or Security Officer signatures.
The Slint app, CLI, local web UI, and mobile companion all surface the same Approval Queue and route signatures through the same hardware abstraction. Mobile signatures are restricted by overlay: ITAR / IL5 overlays MUST disable mobile signing; FERPA / HIPAA / 800-171 overlays MAY enable it with a per-overlay TTL on approvals from mobile (default: 1 hour vs 24 hours on desktop).

# 6. Audit Chain and Record Keeping

## 6.1 Audit Chain Structure

Every event of interest — proposed action, approval, rejection, capsule install, policy change, break-glass invocation, doctor report — produces a record in the hash-chained audit log. Records have the following normative structure:

AuditRecord {
    sequence:        u64,                 // monotonic per-agent
    timestamp:       i64,                 // unix epoch nanos, monotonic
    previous_hash:   [u8; 32],            // sha256 of previous record (or zeros for genesis)
    event_type:      EventType,           // enum, exhaustive
    payload:         serde_cbor::Value,   // event-specific
    actor:           AgentId,             // who proposed (the AgentSBT id)
    signatures:      Vec<RoleSignature>,  // approver signatures
    chain_anchor:    Option<AnchorRef>,   // present if anchored on-chain
}

RoleSignature {
    signer:      DID,            // did:citrate:role:0x... (PIV cert or FIDO key)
    role:        Role,
    signed_at:   i64,
    signature:   Vec<u8>,        // detached signature over canonical CBOR of record
    surface:     SigningSurface, // Slint | CLI | LocalWeb | Mobile
}

## 6.2 Storage Targets

Records live in operator-configured storage. The harness ships with four storage backends, all selectable by the policy bundle:

- Local filesystem — the default; air-gap-friendly; supports posix-acl-based access control.
- NFS / S3-compatible object store — for on-prem multi-host deployments.
- WORM storage (write-once-read-many) — required for some FedRAMP High and CJIS deployments; verified at write time by the storage backend.
- Citrate L1 chain — anchors only, never the full records. Privacy default.
Records themselves MUST NEVER leave the operator's chosen storage in plaintext. Anchors to the chain MUST be root hashes of Merkle trees over local records, not the records themselves. No CUI, PHI, FERPA-protected, or ITAR-controlled content MAY be written to chain in any anchor strategy.

## 6.3 Anchor Strategies

The operator's policy bundle selects one of three anchoring strategies, or a hybrid:

- Strategy A — Per-Capsule. On-chain events for capsule install and run-receipt only. No per-action chatter. Lowest fidelity, lowest cost.
- Strategy B — Per-Approval. Every approval emits its own on-chain event. Highest fidelity, highest cost. Suitable for tier-critical-only or for high-scrutiny deployments.
- Strategy C — Nightly Merkle Root. Daily batch: Merkle root of the day's audit-log entries written as a single on-chain event. Lowest cost, suitable for most overlays.
- Hybrid C + A for tier-high. Default for FedRAMP / CMMC deployments: nightly Merkle root for baseline traffic, plus per-capsule install event for tier-high actions.
Strategies MAY be changed by the Security Officer through the standard policy-bundle update flow, but the change MUST itself be a recorded audit event signed by Security Officer + Compliance Officer.

## 6.4 Retention

Retention periods are overlay-driven, expressed in the policy bundle, and enforced by the harness's audit-record retention scheduler:


Records MUST NOT be deleted before their overlay-derived retention period. Deletion of a record after expiration is itself an audited event requiring AU-9(5) dual authorization.

# 7. On-Chain Surface

## 7.1 Contracts at v1

Citrate Agent's on-chain surface comprises five contracts deployed on Citrate Mainnet (chain id 40204). Contracts are UUPS-upgradeable with a 2-of-3 timelocked controller: one key held by Citrate Network governance, one by an external security council, one by an emergency-pause role with no upgrade authority.


## 7.2 The Clearance Lattice on AgentSBT

The AgentSBT's clearance field is the on-chain anchor for the data-class lattice. When a capsule install is proposed, the harness performs the following four steps in order:

- Step 1. The harness reads the AgentSBT clearance from chain (or cached if anchor strategy is offline).
- Step 2. The harness reads the capsule manifest's data_class.reads from CapsuleRegistry.
- Step 3. If clearance does not dominate every entry in reads under the lattice ordering, the install is refused — both on-chain and locally.
- Step 4. If clearance does dominate, the install proceeds through the standard HIC gate before being added to AgentSBT's installed-capsule set.
This is the on-chain manifestation of Bell-LaPadula no-read-up. The check is enforced both by the harness and by the CapsuleRegistry contract itself; an out-of-band attempt to bypass the harness still fails at the contract level.

## 7.3 Privacy Guarantees

The on-chain surface deliberately contains no operator content. Specifically:

- Audit records exist only in operator-configured storage. The chain stores only Merkle roots.
- Capsule inputs and outputs are committed to (hashed) but never stored.
- Approver identities on-chain are DIDs derived from PIV / FIDO keys; the chain stores the DID, not the underlying biographic identity.
- Org and agent metadata is the minimum required for clearance enforcement and overlay verification.
This design is what permits the harness to anchor on a public chain while remaining compatible with FERPA, HIPAA, ITAR, and IRS 1075. The chain learns that something happened, in what shape, by which class of approver — never what the something was.

# 8. The Slint Onboarding and Operations App

## 8.1 Scope

The Slint app is the polished local surface for operators. It performs four roles simultaneously:

- Concierge — first-run setup wizard with a bundled tiny model driving the conversation. Captures org identity, role assignments, policy bundle selection, hardware key enrollment.
- HIC Approval Surface — the canonical UI for the Approval Queue. Capsule Inspector pane shows the full manifest before any install; approval pane shows action context, proposing capsule, signatures held, and signatures required.
- Chat — the interactive interface to the live Agent. Streams tokens; pauses at HIC gates; shows the audit-record-being-formed inline with the conversation.
- Capsule Marketplace Browser — browse, search, and install capsules from the operator's configured sources (bundled, site mirror, on-chain registry). Installation is HIC-gated; the marketplace pane proposes, the approval pane authorizes.
## 8.2 The Bundled Concierge Model

The Slint app ships with Google Gemma 4 E2B (Effective 2B parameters, Apache 2.0, released April 2026) as its bundled concierge model. The model file is distributed as a separately hashable artifact (gemma-4-e2b-it-Q4_K_M.gguf) alongside the binary, not embedded in it — NIST SI-7 requires the model artifact to be independently verifiable.

On first run, the Slint app resolves its model in the following order:

- Step 1. Local Ollama instance on standard port (11434); checks for gemma4:e2b, gemma4:e4b, qwen3:8b, llama4:8b in that preference order.
- Step 2. Local llama.cpp server on standard ports (8080, 8000).
- Step 3. Embedded bundled model via llama-cpp-4 with the bundled GGUF.
The model file's SHA-256 is verified against the manifest before first use. The harness refuses to load a model whose hash does not match. The bundled model has explicit function-calling support, 256K context, and is sufficient to drive the onboarding flow and provide useful baseline conversational behavior in air-gapped environments.

## 8.3 The Capsule Inspector

Before any capsule installs, the Slint app surfaces a Capsule Inspector view that translates the manifest into operator-readable form. The view is the AC-6 (Least Privilege) audit artifact for the install. No capsule installs without the operator seeing this view; the harness will not provide a bypass.

The Capsule Inspector view MUST present, at minimum:

- Capsule name, version, content hash.
- Publisher DID and signing tier (bundled / managed / workspace).
- Declared capabilities: network, filesystem, chain calls, subagent spawn.
- Declared data classes: reads, writes, emits.
- Risk tier and required approver roles.
- Certified overlays and explicit not-certified list.
- Whether a TLA+ specification exists, and its verification status.
- Gherkin scenario pass rate from the BenchmarkRegistry.
- An expandable view of procedure.md.
## 8.4 Licensing Note

Slint is licensed under a triple-license scheme: GPLv3, Royalty-Free, or Commercial. Citrate Agent's dual-license posture (Apache-2.0 OSS + commercial license for FedRAMP package) aligns naturally with Slint's commercial license for the FedRAMP track. The open-source distribution ships under Slint's GPLv3 terms, which is compatible with the open-core thesis but means any fork of the harness UI must publish its changes.

# 9. Formal Verification

## 9.1 The Five Load-Bearing Specs

Citrate Agent's safety-critical control flow is formally specified in TLA+ and model-checked in CI as a BLOCKER under Agentile rule 10. Five specs are normative for v1.0 and must verify in CI before any merge to main:


## 9.2 Gherkin + TLA+ Dual Channel

Capsules at tier-high or tier-critical MUST ship both a Gherkin scenario suite (intent-level behavior) and a TLA+ specification (safety invariants). The CI cross-checks them: scenarios that pass Gherkin but violate a TLA+ invariant are a BLOCKER and prevent the capsule from being signed.

This dual-channel discipline is Agentile rule 11 made operational at the capsule level: intent is captured in human-readable scenarios that double as executable tests; safety is captured in machine-checkable TLA+ that models the full reachable state space. Neither alone is sufficient; together they are the strongest correctness argument a small team can produce in 2026.

# 10. The Doctor Pre-Flight Check

## 10.1 Purpose

citrate-agent doctor is a pre-flight and continuous-monitoring command. It produces a signed report, optionally anchored on-chain, that serves as the System Security Plan evidence artifact and the daily continuous-monitoring (NIST SP 800-53 CA-7) report for the harness. Doctor is the highest-leverage tool in the harness for satisfying CA-family and SI-family controls without writing a separate compliance product.

## 10.2 Checks Performed

doctor MUST run, at minimum, the following checks on every invocation:

- All installed capsule signatures verify against trusted publisher keys for their tier.
- All capsule manifests' declared capabilities match their WASM imports under the WIT linker.
- Policy bundle is signed by Security Officer key and is not past its signed validity window.
- Audit chain is intact: hash-chain verifies from genesis (or last anchored root) to current head.
- All approver hardware key bindings remain valid: PIV certificates not expired, YubiKey serials registered, mobile pairings not revoked.
- Network posture matches policy: air-gapped sites verify no egress occurred since last run.
- Model file hash matches its manifest declaration.
- TLA+ specifications for installed tier-high and tier-critical capsules have a current verification record in BenchmarkRegistry.
- Required role assignments are filled: every role in the five-role lattice has at least one identity assigned.
- Approval queue is not backed up beyond its SLA (default: no high-risk approval pending more than 48 hours).
- No break-glass actions are pending post-hoc affirmation past their 72-hour window.
## 10.3 Output

doctor produces a signed TOML report at the operator's configured output path, plus an optional on-chain commitment of its content hash. The report enumerates every check, its outcome, and supporting evidence. Failed checks are categorized as BLOCKER (the agent SHALL NOT run until remediation) or WARN (the agent MAY run but the issue is enumerated on the next report).

# 11. Roadmap

## 11.1 v1.0 — Mainnet-Ready (Target: Q2 2027)

v1.0 ships the full architecture specified in this document: citrate-agent-core library, citrate-agentd daemon, citrate-agent-cli, citrate-agent-slint app, citrate-agent-wasm publish target, mobile companion (iOS + Android), five-contract on-chain surface, FERPA + HIPAA + CMMC L3 baseline overlay set, Gemma 4 E2B bundled, Trail of Bits audit complete.

## 11.2 v1.1 — IL5 and ITAR Hardened (Q1 2027)

DoD IL5 and ITAR overlays graduate from beta to production. PIV-only enforcement available as a global policy. CJIS overlay added. NIAP / Common Criteria evaluation initiated.

## 11.3 v2.0 — Subagent Delegation and SVM (target: 2027)

Policy-gated subagent delegation introduced. SVM compatibility appendix promoted to first-class support. A capsule's chain_calls capability gains an svm: namespace. Federated learning between cit-agent instances on the Citrate cooperative becomes a supported workload class.

## 11.4 Out of Scope Indefinitely

- Free-form runtime skill creation by the agent itself.
- Cross-org agent collaboration outside the cooperative's signed-capsule channels.
- Cloud-hosted multi-tenant deployment (third parties may offer it under the commercial license; Citrate Inc. will not operate it).
# 12. Open Questions for Quorum Resolution

The following items remain open after the v0.1 design interview and require resolution before v1.0-rc:

## Q1. Mobile companion attestation chains

Which device attestation chains (Apple SEP, Google StrongBox, Samsung Knox) are acceptable for which overlays? A policy-bundle whitelist exists; the default content of that whitelist for each overlay needs explicit Security Officer review.

## Q2. The off-chain identity bridge

Approvers identified by PIV / CAC are anchored to a real-world federal credentialing system; FIDO2 approvers are anchored to nothing outside the device. For overlays that require background-investigation evidence on approvers (IRS 1075, ITAR), how does the harness verify the binding between a FIDO key and an investigated identity? Working assumption: PIV / CAC required for those overlays; FIDO not sufficient. Needs explicit confirmation.

## Q3. Trajectory export and the training-data residency question

Hermes-style trajectory export for SFT/RL is technically straightforward but creates a new data-class question: a trajectory may contain CUI as part of the agent's observed actions. Working assumption: trajectory export is a tier-critical action and capsule-gated. Specific export-capsule manifest schema needs design.

## Q4. The cooperative-publishing channel

The cooperative-publishing model envisioned in the Citrate thesis (operators sharing curated capsules) needs an explicit signing-and-curation pipeline. Working assumption: capsules are published as ERC-1155 mints on CapsuleRegistry with publisher signatures; cooperative endorsement is a separate event from another signed party. Detailed flow needs design.

## Q5. The benchmark adversary

Capsule benchmarks (Gherkin + TLA+) cover declared behavior. They do not cover adversarial-input behavior — what happens if a capsule receives unusual or malicious input from another capsule's output? Working assumption: capsule composition is a runtime concern handled by data-class enforcement plus per-capsule contract testing in CI. Whether to add a dedicated adversarial-fuzz benchmark to BenchmarkRegistry needs decision.

## Q6. The doctor report's freshness SLA

How fresh does the most recent doctor report need to be before the harness refuses to run? Working assumption: 24 hours, with operator override down to 7 days under medium-risk overlays. Needs validation against assessor expectations.

# 13. Glossary


# Appendix A. References


Normative References

- NIST SP 800-53 Revision 5 — Security and Privacy Controls for Information Systems and Organizations
- NIST SP 800-171 Revision 3 — Protecting Controlled Unclassified Information in Nonfederal Systems
- NIST SP 800-172 — Enhanced Security Requirements for Protecting CUI
- CMMC Assessment Guide Level 3 v2.13 — Department of Defense
- 32 CFR § 170 — Cybersecurity Maturity Model Certification
- DFARS 252.204-7012 — Safeguarding Covered Defense Information and Cyber Incident Reporting
- FedRAMP Rev 5 Baselines — General Services Administration
- RFC 2119 / RFC 8174 — Key words for use in RFCs to Indicate Requirement Levels

Informative References

- NIST COSAiS Project — Control Overlays for Securing AI Systems
- NIST AI Agent Standards Initiative (launched February 2026)
- WebAssembly Component Model — Bytecode Alliance
- Citrate Network Documentation — citrate.ai/docs
- Agentile Methodology — github.com/CitrateNetwork/agentile
- Hermes Agent — github.com/NousResearch/hermes-agent
- OpenClaw — github.com/openclaw/openclaw
- TLA+ Home Page — Leslie Lamport, Microsoft Research

---

## Tables (extracted from source docx)

### Table 1

| Status: Draft for Quorum Review This is v0.1 of the Citrate Agent architecture RFC, produced 2026-05-14 as the first artifact of the design interview between the Citrate Network team and the Anthropic model assisting on the project. Subsequent versions will incorporate review from the named co-authors and quorum signatories below before promotion to v1.0-rc. |
| --- |

### Table 2

| Family | Key Controls | How cit-agent satisfies it | Component |
| --- | --- | --- | --- |
| AC | AC-2, AC-3, AC-3(2), AC-5, AC-6 | Tiered-risk + role-bound quorum approval engine; capability tags on every capsule; data-class lattice enforced by linker; dual authorization on tier-high actions; separation of duties via five-role minimum (Operator, Reviewer, Compliance Officer, Security Officer, Auditor). | policy + hitl + capsule loader |
| AU | AU-2, AU-9, AU-9(5), AU-10 | Hash-chained signed JSONL audit log; per-record per-signer signatures; AU-9(5) dual-auth on audit deletion enforced contractually (auditor role has no approval rights); AU-10 non-repudiation via approver hardware keys. | audit chain + key manager |
| IA | IA-2, IA-4, IA-5 | PIV / CAC / FIDO2 hardware authentication for approvers; per-agent SBT identity on-chain; per-org SBT identity; per-capsule manifest identity by content hash. | Slint signing flow + AgentSBT + OrganizationSBT |
| SI | SI-7, SI-7(1), SI-7(15) | Capsule signatures verified against trusted publisher keys before load; WIT linker check that imports match declared capabilities; model file hash verified against manifest before inference. | capsule loader + doctor |
| SC | SC-7, SC-8, SC-13 | Air-gapped by default; opt-in egress is policy-bundle-controlled and per-action; FIPS 140-3 validated crypto via aws-lc-rs; mTLS for daemon and mobile bridge. | egress broker + crypto module |
| CA | CA-2, CA-7 | Nightly doctor report produces signed continuous-monitoring artifact; on-chain anchoring of artifact hash supports CA-7 evidence chain. | doctor + chain anchor |
| CM | CM-5, CM-5(4) | Capsule install requires HIC approval (CM-5); tier-high capsule install requires dual authorization (CM-5(4) Dual Authorization for Privileged Changes). | capsule install gate |
| IR | IR-4 | Break-glass path with post-hoc quorum affirmation requirement; quarantine state on AgentSBT freezes capability surface pending IR-4 incident response. | break-glass + AgentSBT lifecycle |

### Table 3

| Overlay | Triggers | Key Restrictions | Required Evidence |
| --- | --- | --- | --- |
| FERPA + COPPA + CIPA | Capsule reads classified as student-directory or student-restricted | No external model pull; no cloud egress; no audio/video to external recipients; age-screening on COPPA-applicable flows | Annual Compliance Officer attestation; FERPA-redact capsule signature chain; CIPA filter capsule present |
| HIPAA + HITECH | Capsule reads or writes PHI | BAA-class signing chain required on capsule publishers; encryption at rest + in transit (FIPS 140-3); audit log retention 6 years minimum | BAA on file; HHS-OCR ready audit export; minimum-necessary justification per access |
| FedRAMP High + DoD IL4/IL5 | Operator-declared at install | FIPS 140-3 validated crypto only; U.S. persons only on approver role lattice (operator-policy enforced); no foreign IP allowed on any egress; container hardening to STIG | ATO package; continuous monitoring report; POA&M for any open finding |
| ITAR | Capsule reads classified as ITAR-controlled | U.S. persons only on every signature; air-gap MUST NOT be overridden under any policy; export-classification capsule required before egress | DDTC registration evidence; export classification record per affected capsule |
| CJIS | Capsule touches CJI | FIPS 140-3 crypto; CSO-approved personnel only; advanced authentication on approver flows | CJIS Security Addendum; personnel security records; logging configuration evidence |
| IRS 1075 | Capsule touches FTI | Background investigation evidence on approvers; physical access controls inherited from host facility; no commingling of FTI and non-FTI capsules | SSR (Safeguard Security Report); annual self-assessment |

### Table 4

| Tier | Signed By | Trust Implication | Installation Path |
| --- | --- | --- | --- |
| bundled | Citrate Inc. (canonical publisher key, FIPS HSM) | Maximal trust; safe for any overlay; reviewed and audited by Citrate before publication | Ships with the harness binary; updated via signed release |
| managed | Org's procurement-chain CA (org SBT controller) | Trusted for the org's policy bundle; subject to org's overlay restrictions | Installed via site mirror or chain pull; HIC approval required |
| workspace | Operator's local signing key | Trusted only for the operator's agent; cannot enter managed tier without re-signing | Local-only; mandatory dual approval (Operator + Security Officer); never anchored to chain |

### Table 5

| Tier | Examples | Approval Requirement | State Persistence |
| --- | --- | --- | --- |
| low | Read public data; idempotent searches; status queries | Auto-approve; log only | In-memory |
| medium | Internal write with rollback path; LLM call on non-CUI data | 1 approver from required-role set; approve / edit / reject | Checkpoint to operator storage |
| high | External email; deletion of business record; CUI write; LLM call on CUI data | N-of-M approvers (default 2) from required-role set; approve / reject only (no edit) | Checkpoint + on-chain anchor (per policy) |
| critical | Audit-log modification; policy-bundle change; capsule signature key rotation | Full quorum (Security Officer + Compliance Officer + Reviewer); approve / reject; mandatory written justification | Checkpoint + on-chain anchor (always) |

### Table 6

| Role | Authority | Constraints (AC-5) |
| --- | --- | --- |
| Operator | Run the agent; approve low-risk actions; propose actions for higher-risk review | Cannot approve actions they themselves proposed; cannot serve as audit-log reader for actions they signed off on |
| Reviewer | Second signature on medium- and high-risk actions; peer of Operator | Cannot be the same person as the Operator for any given action |
| Compliance Officer | Required signature on any action touching a protected data class (CUI, PHI, FERPA, ITAR, etc.) | Cannot also hold the Security Officer role for the same agent |
| Security Officer | Required signature on capability changes, capsule installs, policy-bundle updates, and break-glass actions | Cannot also hold the Compliance Officer role; cannot approve audit-log queries |
| Auditor | Read-only access to audit chain; can export evidence; can verify signature chain offline | Cannot approve ANY action; cannot modify any record |

### Table 7

| Overlay | Minimum Retention | Source |
| --- | --- | --- |
| NIST 800-171 Baseline | 3 years | Org's standard policy |
| CMMC L3 | 6 years post-contract-termination | DFARS 252.204-7012 |
| HIPAA | 6 years from creation or last effective date | 45 CFR § 164.530(j) |
| FERPA | 5 years post-student-departure | Org's record schedule + 34 CFR § 99 |
| ITAR | 5 years from final export action | 22 CFR § 122.5 |
| IRS 1075 | Until destruction authorized by Disclosure Officer | IRS Pub 1075 § 9.4 |

### Table 8

| Contract | Role |
| --- | --- |
| OrganizationSBT | Soulbound identity for an org. Holds compliance overlays attained, multisig controller, jurisdiction, founding date. Non-transferable; revocable only by quorum of its own controller. |
| AgentSBT | Soulbound identity for a deployed agent instance. Holds parent org id, harness version, clearance level (PUBLIC | CUI | PHI | FERPA | ITAR), status (provisioning | active | quarantined | retired), latest audit-chain head, set of installed capsule IDs. |
| CapsuleRegistry (ERC-1155) | Capsule registry. Token id = keccak256(name, version, manifest_hash). Per-id metadata: content hash, certified overlays, required role set, risk tier, publisher DID, reviewer signatures, benchmark anchor. |
| AnchorRegistry | Three event shapes (per-capsule, per-approval, nightly Merkle) for the three anchor strategies. Operator policy selects which events get emitted. |
| BenchmarkRegistry | Per-capsule Gherkin pass rates and TLA+ verification commitments. Separate from MODEL_BENCHMARK precompile (0x0105) which ranks model providers. |

### Table 9

| Spec | Property Verified |
| --- | --- |
| ApprovalStateMachine.tla | Every proposed action terminates in approve or reject. No action executes without the role-set its manifest requires. The state-managed interrupt resumes deterministically from any checkpoint. Edit-on-medium does not silently promote to a different action. |
| AuditChainIntegrity.tla | The hash chain is contiguous: previous_hash on record N equals sha256 of record N-1. Signatures verify against the canonical CBOR of the record they sign. No path admits a record whose previous_hash points outside the chain. |
| DataClassLattice.tla | Bell-LaPadula no-read-up: a capsule whose reads contains a class higher than the agent's clearance never executes. No-write-down: a capsule cannot emit data at a class lower than what it read unless it explicitly traverses a declassification capsule. |
| CapsuleInstall.tla | A capsule installs only after manifest signature verification + WIT-WASM-manifest cross-validation + HIC approval traversal. No partial-install state exists in which the capsule is callable but its install record is incomplete. |
| BreakGlass.tla | Break-glass invocation always notifies every approver. The 72-hour post-hoc affirmation window either resolves to affirmed or surfaces on every subsequent doctor report. ITAR-classified data never reaches a break-glass codepath. |

### Table 10

| Term | Definition |
| --- | --- |
| Agentile | Institutional methodology for human-agent software engineering, codified in the CitrateNetwork/agentile repository. Source of the four CI ratchets and the 13 enforceable rules referenced throughout this document. |
| AgentSBT | Soulbound (non-transferable) ERC-721-style token representing an agent's on-chain identity. Holds clearance, status, and audit chain head. |
| Anchor | An on-chain commitment to an off-chain artifact. Anchors are Merkle roots or content hashes, never the artifacts themselves. |
| ATO | Authority to Operate. The formal authorization an organization receives after a successful security assessment. cit-agent is designed to be ATO-friendly under multiple overlays. |
| Break-Glass | Emergency override path that permits a single Security Officer signature in lieu of full quorum, subject to post-hoc affirmation. |
| Capsule | The unified unit of capability and procedure in cit-agent. A signed WIT/WASM component bundled with its manifest, procedure, scenarios, and optionally a TLA+ spec. |
| CMMC | Cybersecurity Maturity Model Certification. DoD's tiered certification for DIB contractors. cit-agent's baseline targets CMMC L3. |
| COSAiS | Control Overlays for Securing AI Systems. NIST project mapping SP 800-53 controls to AI deployment categories. The architectural backbone for cit-agent's compliance posture. |
| CUI | Controlled Unclassified Information. Defined in 32 CFR Part 2002. The primary information category cit-agent's CMMC L3 baseline protects. |
| DID | Decentralized Identifier. Used to identify orgs, agents, and approver roles in the on-chain registry. |
| Doctor | The pre-flight and continuous-monitoring command. Produces signed reports that serve as the SSP evidence artifact. |
| FERPA | Family Educational Rights and Privacy Act. Overlay applicable to school-deployed agents. |
| HIC | Human In Control. The approval gate every proposed action traverses. |
| OrganizationSBT | Soulbound token representing an organization's on-chain identity. |
| Overlay | A signed policy bundle that augments the baseline compliance posture with overlay-specific restrictions and evidence requirements. |
| TLA+ | Specification language developed by Leslie Lamport. cit-agent's five load-bearing safety properties are written and model-checked in TLA+. |
| WIT | WebAssembly Interface Types. The typed interface description language for the WebAssembly Component Model. Capsules express their imports and exports in WIT. |

