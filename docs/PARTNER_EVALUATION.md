# nist-agent — Partner Evaluation Guide

**Release:** `v1.0.0-rc.1` (partner-evaluation release candidate)
**Audience:** invited evaluation partners
**What this document is:** an honest map of what is real and tested, what is a
labelled preview, and what is deliberately deferred — so your feedback is
calibrated to the right baseline.

> **Read this before you install.** nist-agent is a release *candidate*. The
> compliance-critical surfaces are real, tested, and formally verified. A small
> number of features are explicit previews (clearly marked in the UI). We want
> your feedback on the real surfaces; the previews are flagged so you don't waste
> time reporting them as bugs.

---

## What nist-agent is

A NIST-compliant agent harness — a sidecar that lets an organization run AI
agents under hard, auditable controls. It is **air-gapped by default** (zero
outbound connectivity unless a signed Security Officer policy enables it), gates
every action through a five-role human quorum (Operator, Reviewer, Compliance
Officer, Security Officer, Auditor) with hardware-backed keys (FIDO2 / PIV-CAC /
Secure-Enclave-or-TPM), loads capabilities as signed WASM **Capsules** enforced
at load time, and maintains a hash-chained, signed audit log.

**Compliance baseline:** NIST SP 800-171 Rev 3 + CMMC Level 3.
**Overlays in this release:** FERPA, COPPA, CIPA, HIPAA/HITECH, FedRAMP High.

---

## Status at a glance

| Capability | Status | Notes |
|---|---|---|
| Five-role HITL approval quorum | ✅ **Real & tested** | Separation-of-duties, dedup, self-approval all enforced at the policy seam |
| Hash-chained signed audit log | ✅ **Real & tested** | Four storage backends; tamper-evident; verifiable replay |
| On-chain audit anchoring | ✅ **Real** (testnet) | Anchors commitments only — no operator content ever reaches the chain. Citrate L1 (chain 40204) or local Anvil |
| Signed Capsule dispatch (WASM) | ✅ **Real & tested** | Manifest enforced at the wasmtime linker; capability enforcement is load-time, not advisory |
| Doctor pre-flight checks (11) | ✅ **Real & tested** | Includes model-hash verification (SI-7) |
| Six compliance overlay bundles | ✅ **Real** | Each with a signed release runbook + rollback section |
| Five normative TLA+ specifications | ✅ **Verified in CI** | Approval state machine, audit-chain integrity, data-class lattice, capsule install gate, break-glass path |
| Slint operator app (concierge, HITL queue, Capsule Inspector, marketplace) | ✅ **Real**, 🟡 **unsigned** | The CLI is signed; the desktop app ships unsigned in this RC — see "Previews" |
| Autonomous agent loop (L0 chat) | 🟡 **Preview** | Runs a **local-model fallback**, clearly labelled. Real agent orchestration lands in v1.0 (upstream `CIT-AGENT-3`) |
| Mobile companion | 🟡 **Protocol only** | Render-model is real; native iOS/Android apps deferred to v1.1+ |
| External Tier-1 audit (Trail of Bits) | 🟡 **In progress** | Audit packet delivered; remediation track active. No Tier-1 findings outstanding as of this RC |

✅ real and tested · 🟡 preview / in-progress · all previews are labelled in-product.

---

## What we want your feedback on

These are the surfaces that are real and where your input is most valuable:

1. **The approval-quorum workflow.** Does the five-role model fit how your
   organization actually approves agent actions? Where does it feel too heavy or
   too loose?
2. **Compliance overlay fit.** If you operate under FERPA / COPPA / CIPA / HIPAA /
   FedRAMP High, does the overlay bundle + runbook match your control expectations?
   What's missing?
3. **The audit story.** Can your auditor reconstruct "who approved what, when, and
   under which policy" from the audit log and on-chain anchors?
4. **Doctor / first-run onboarding.** Does the concierge get a new operator to a
   working, policy-bound install without hand-holding?
5. **Air-gapped operation.** Does the default-deny network posture hold up in your
   environment, and is the opt-in egress path clear?

## What is a preview — please don't file these as defects

- **L0 chat runs a local-model fallback**, not the production agent loop. Use it
  to exercise the supervised HITL path, not to evaluate autonomous-agent quality.
  (Real orchestration arrives in v1.0 with upstream `CIT-AGENT-3`.)
- **The Slint desktop app is unsigned** in this RC. Your OS may warn on launch.
  The CLI daemon is signed and is the recommended interface for serious
  evaluation. Signed desktop installers ship in v1.0.
- **Production anchoring is deferred.** Anchor to testnet (chain 40204) or local
  Anvil for evaluation.

## Known limitations / deferred to v1.0+

- Production agent-loop orchestration (`CIT-AGENT-3`, upstream).
- HSM-backed release signing (current RC uses soft-key signing; the HSM swap-in
  path is documented and is a configuration change, not a code change).
- Native mobile apps (v1.1+).
- v1.1 overlays: DoD IL4/IL5, ITAR, CJIS, IRS 1075.
- The `v1.0.0` tag itself is gated on completion of the external audit.

---

## How to proceed

1. Install per **`docs/PARTNER_INSTALL.md`**.
2. Run an evaluation against your own infrastructure (testnet or air-gapped).
3. Send feedback per **`docs/PARTNER_FEEDBACK.md`** — structured findings,
   friction points, and "would this pass our auditor" judgments are all welcome.

Thank you for evaluating nist-agent. Your feedback feeds directly into the v1.0
release and the active audit-remediation track.
