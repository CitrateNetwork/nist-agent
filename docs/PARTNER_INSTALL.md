# nist-agent — Partner Install Guide

**Release:** `v1.0.0-rc.1`
**Audience:** evaluation partners standing up nist-agent on their own infrastructure.

This guide gets you from a downloaded release bundle to a running, policy-bound
agent harness. For what the release contains and what is preview vs final, read
**`docs/PARTNER_EVALUATION.md`** first.

---

## 1. What you receive

The release bundle (`nist-agent-v1.0.0-rc.1.tar.gz` + `.sha256`) contains:

- The `nist-agent` CLI daemon (signed).
- The Slint operator app (unsigned in this RC — see Evaluation guide).
- A bundled Gemma 4 E2B GGUF model for the first-run concierge (hash-pinned;
  the doctor verifies it at startup).
- The six compliance overlay bundles, each with a deployment runbook.
- A CycloneDX SBOM (`.sbom.json`).
- The signed release manifest (`.manifest.toml`) with per-artifact SHA-256 hashes.

## 2. Verify the bundle before you trust it

```bash
# 1. Checksum
sha256sum -c nist-agent-v1.0.0-rc.1.tar.gz.sha256

# 2. Confirm the manifest hashes match the extracted artifacts
tar xzf nist-agent-v1.0.0-rc.1.tar.gz
cd nist-agent-v1.0.0-rc.1
cat .manifest.toml          # per-artifact sha256 + signing key id
```

> The RC manifest is **soft-key signed** (Ed25519). The HSM-backed signing path
> for the GA release is documented in `docs/audit/HSM_SEAM.md`.

## 3. Requirements

- **OS:** Linux x86_64 (primary). macOS aarch64 and Windows x86_64 binaries are
  produced for the CLI; the desktop app is best evaluated on Linux for this RC.
- **Hardware key (recommended):** FIDO2 / PIV-CAC / TPM or Secure Enclave for
  real quorum sign-off. Software keys are accepted for a first look but do not
  represent the production trust model.
- **For on-chain anchoring (optional):** access to Citrate testnet
  (`https://rpc.citrate.ai`, chain 40204) **or** a local Anvil node. Anchoring is
  off by default; the harness runs fully air-gapped without it.
- **No outbound network is required.** nist-agent is air-gapped by default.

## 4. First run

```bash
# Run the doctor pre-flight first — it verifies the model hash, key backends,
# storage, and policy integrity before anything is allowed to run.
./nist-agent doctor

# Launch the operator app (or the CLI) and let the concierge walk you through
# first-run setup: role enrollment, policy selection, storage backend.
./nist-agent up           # CLI daemon
# or launch the bundled Slint operator app
```

On first run the concierge will:
1. Enroll your five roles (Operator, Reviewer, Compliance Officer, Security
   Officer, Auditor) to hardware-backed keys.
2. Let you select a compliance overlay (CMMC-L3 baseline + one of FERPA / COPPA /
   CIPA / HIPAA / FedRAMP High).
3. Initialize the hash-chained audit log + (optionally) configure an anchor
   strategy.

## 5. Run an evaluation

- **Approve an action:** submit a sample agent action and walk it through the
  five-role quorum. Observe separation-of-duties enforcement (the same key cannot
  satisfy two conflicting roles; no self-approval).
- **Dispatch a Capsule:** install the `hello` smoke Capsule and dispatch it;
  inspect the manifest in the Capsule Inspector and confirm capability
  enforcement at load.
- **Verify the audit chain:** run the audit verifier; tamper with a record and
  confirm the chain rejects it.
- **(Optional) Anchor:** anchor the audit root to testnet or Anvil and confirm
  only the commitment — never operator content — appears on-chain.

## 6. Per-overlay deployment

Each compliance overlay ships a runbook under
`docs/compliance/<overlay>/RUNBOOK.md` (cmmc-l3, ferpa, coppa, cipa, hipaa,
fedramp-high), each with a rollback section. Follow the one matching your
regulatory context.

## 7. Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| `doctor` fails on model hash | Bundled GGUF altered / partial download | Re-extract from the verified tarball |
| OS blocks the desktop app | Unsigned app in this RC | Use the signed CLI, or allow the app in your OS security settings |
| Quorum won't complete | Software keys / missing role enrollment | Enroll all five roles; use hardware keys for the real model |
| Anchor step times out | No testnet/Anvil reachable | Anchoring is optional — skip it, or point at a local Anvil |

---

Questions or problems? See **`docs/PARTNER_FEEDBACK.md`** for how to reach us.
