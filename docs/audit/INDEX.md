---
created: 2026-05-21T00:00:00Z
branch: feat/s-13-tob-engagement
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
audience: Trail of Bits engagement team
---

# nist-agent v1.0-rc — Trail of Bits Audit Packet

> Cover sheet for the Trail of Bits engagement against
> `nist-agent` at the v1.0 release-candidate commit
> [`30044d2`](https://github.com/CitrateNetwork/nist-agent/commit/30044d26201daaabdeaaab8d7f25d950b336f023).
> Read these documents in order; each links forward to the
> next. Questions during the engagement go to the contacts in
> [`CONTACT.md`](CONTACT.md).

## What this packet is

This packet bootstraps the Trail of Bits engagement against the
`nist-agent` sidecar — a NIST-compliant agent harness
implementing [RFC-CIT-AGENT-0001](../rfcs/RFC-CIT-AGENT-0001.md).
The sidecar is a sidecar **consumer** (not a fork) of
`citrate-agent-runtime`; the engagement scope is this
workspace's 13 crates plus the policy bundle / overlay /
distribution documentation that constitutes the deployable
product.

## Reading order

| # | Document | What it answers |
|---|---|---|
| 1 | [`SCOPE.md`](SCOPE.md) | What's in scope, what's out of scope, file inventory, line counts. |
| 2 | [`THREAT_MODEL.md`](THREAT_MODEL.md) | Actor / asset / surface crosswalk + STRIDE per load-bearing surface. |
| 3 | [`BUILD.md`](BUILD.md) | Reproducible-build instructions, toolchain pins, hermetic environment. |
| 4 | [`DEPLOYMENT.md`](DEPLOYMENT.md) | How to stand up a hands-on reference deployment. |
| 5 | [`V1_READINESS.md`](V1_READINESS.md) | Phase-1 exit criteria cross-check (what's done, what's deferred). |
| 6 | [`CONTACT.md`](CONTACT.md) | Escalation paths, secure communication channels, embargo policy. |

## What you're being asked to find

Findings the engagement is sized to surface, in priority order:

1. **Quorum-bypass paths in HIC approval.** Any way for an
   action to reach `Agent::resume()` without the role-bound
   signatures the policy bundle requires.
2. **Overlay-ratchet bypass.** Any way to remove `CmmcL3` from
   the active overlay set, or to add a less-restrictive overlay
   without the `DecommissioningWorkflow` proof token.
3. **Audit-record integrity gaps.** Any path that writes to the
   audit chain without the dual-signature requirement (AU-9(5))
   or that mutates a WORM record after creation.
4. **Mobile-pairing token replay / attestation spoofing.** Any
   way to reach `PairingState::Active` without all three of:
   SecurityOfficer signature, allowlisted attestation, and
   matching pairing token.
5. **Release-verifier confused-deputy.** Any input that causes
   `Installer::verify_and_install()` to extract content despite
   a missing or mismatched signature.
6. **Egress-posture leak.** Any path that performs a network
   call while `EgressPosture::Disabled` is in effect.
7. **GGUF SI-7 bypass.** Any way to load a GGUF whose hash
   doesn't match `release.manifest.toml[model].sha256`.

The packet does NOT bound creativity — these are the surfaces
we *expect* findings on; if you find a class outside the list,
that's a higher-priority finding.

## Out-of-band questions

[`CONTACT.md`](CONTACT.md) has the response SLAs. Tier-1
clarifications: same-day. Tier-2/3: within two business days.

## Engagement output expectation

A signed report under
`citrate-agentile-archive/audits/<delivery-date>-tob-nist-agent-v1/`
following the federation's existing per-engagement folder
convention. The sprint S-13 close note in this repo links the
final folder once delivered.

## See also

- [`.agentile/adrs/ADR-012-tob-packet-here-engagement-external.md`](../../.agentile/adrs/ADR-012-tob-packet-here-engagement-external.md) — the scope decision behind this packet.
- [`.agentile/audits/findings/`](../../.agentile/audits/findings/) — local triage scaffolding; per-finding files land here once the engagement starts delivering.
- [RFC-CIT-AGENT-0001 §11.1](../rfcs/RFC-CIT-AGENT-0001.md) — the audit-posture clause that makes this engagement load-bearing for v1.0.
