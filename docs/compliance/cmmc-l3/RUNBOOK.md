---
created: 2026-05-21T00:00:00Z
branch: feat/s-8-overlay-bundles-a
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
overlay: cmmc-l3
---

# Deployment Runbook — CMMC Level 3 Baseline

> The baseline overlay every nist-agent deployment carries
> regardless of additional overlay selection (RFC §2.1).
> Operationally: every operator running nist-agent runs this
> runbook first; other overlays' runbooks layer on top.

## Scope

| Field | Value |
|---|---|
| **Compliance baseline** | NIST SP 800-171 Rev 3 + CMMC Level 3 (32 CFR § 170.14(c)(4)) |
| **Enhanced requirements** | NIST SP 800-172 (CUI handling) |
| **Audit cadence** | Doctor pre-flight runs every start + 24h continuous-monitoring; AuditRecord retention floor 6 years (`retention::retention_floor(Overlay::CmmcL3)`) |
| **Anchor strategy** | Hybrid: nightly Merkle root + per-capsule install events (RFC §6.3 default) |
| **Egress posture** | Disabled by default; opt-in via signed Security Officer policy directive (RFC §3.3 G1) |
| **Mobile signing** | Permitted for low/medium-risk Operator approvals; PIV-CAC required for high-risk (RFC §5.6) |
| **Required signing surfaces** | FIDO2 hardware keys for low/medium; PIV-CAC accepted for all tiers |

## Deployment artifacts

| Artifact | How to obtain |
|---|---|
| Signed PolicyBundle | Construct via `nist_agent_overlays::cmmc_l3_baseline(OverlayBuilder { role_assignments, not_before, expires_at })`, sign with the operator's SecurityOfficer Ed25519 key, distribute the `RawSignedBundle` to every harness instance |
| Capsule set | Bundled-tier capsules ship with the nist-agent release; managed-tier capsules from your org's procurement signer; workspace-tier from per-operator keys |
| Doctor report | Run `citrate-agent doctor` on each instance after install; the report is signed and optionally anchored on chain (operator's `anchor_strategy`) |

## Pre-deploy checklist

- [ ] All five base roles (Operator, Reviewer, ComplianceOfficer, SecurityOfficer, Auditor) have at least one identity assigned. Verify with `nist_agent_doctor::RoleLatticeCheck`.
- [ ] SecurityOfficer's Ed25519 public key is distributed to every harness instance as a trust root.
- [ ] `policy_bundle.activate(now, &prior_tiers)` returns `Ok(())` on a reference deployment.
- [ ] `nist_agent_doctor::v1_checks()` returns `Severity::Pass` overall on a reference deployment.
- [ ] Audit storage path is on a filesystem with POSIX ACLs supporting 0600 mode + sticky-bit directory.
- [ ] Anchor RPC endpoint configured (Citrate Mainnet or any compatible EVM chain per nist-agent-chain's `trait ChainClient`).

## What you commit to by adopting CMMC-L3 baseline

- **AC-3(2) Dual Authorization** — privileged commands require two authorized identities. Enforced by HIC quorum (RFC §5.1).
- **AC-5 Separation of Duties** — Auditor role cannot also be an approver. Enforced by `nist-agent-policy::Role` lattice + ApprovalQueue's SoD check.
- **AC-6 Least Privilege** — every action gated to the minimum role that needs it. Enforced via capsule manifest `risk.required_roles`.
- **AU-9(5) Dual Authorization for Audit** — audit-log modifications require two-person approval. Enforced via the audit chain's RoleSignature requirement.
- **CA-7 Continuous Monitoring** — daily doctor report. Run is operator-scheduled (cron + `citrate-agent doctor --anchor`).
- **SI-7 Software, Firmware, and Information Integrity** — model file SHA-256 verified at every start. Enforced via `nist_agent_doctor::ModelHashCheck`.

## Retention

Audit records written under this overlay have a 6-year retention
floor. Deletion before the floor is rejected at the
`AuditChain::delete()` call site. Deletion after expiration is
itself an audit event requiring AU-9(5) dual authorization.

## Decommissioning

CMMC-L3 baseline is the only **non-removable** overlay. Per
`nist_agent_policy::overlay_state::ActiveOverlays::remove_with_workflow`,
attempting to remove it returns
`PolicyError::OverlayRemovalForbidden(Overlay::CmmcL3)` regardless
of the supplied workflow proof.

## Rollback

- Revert the PolicyBundle to the previous signed version. The
  rollback is itself an audit event (`PolicyChange` record with
  AU-9(5) dual signature).
- Un-install added capsules through the standard HIC-gated
  capsule-uninstall flow.
- Export the audit log for the period during which the bundle
  applied; archive to WORM storage per
  `nist_agent_doctor::NetworkPostureCheck` evidence.

## See also

- [`features/overlays/overlay-cmmc-l3-baseline.feature`](../../../features/overlays/overlay-cmmc-l3-baseline.feature)
- [`features/core/audit-retention.feature`](../../../features/core/audit-retention.feature)
- `.agentile/adrs/ADR-005-policy-bundle-placement.md`
- RFC-CIT-AGENT-0001 §2.1, §2.2 control-family map
