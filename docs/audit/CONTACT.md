---
created: 2026-05-21T00:00:00Z
branch: feat/s-13-tob-engagement
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
audience: Trail of Bits engagement team
---

# Contact — nist-agent v1.0-rc Trail of Bits Engagement

> Communication channels, escalation paths, and response SLAs
> for the engagement. Treat anything Tier-1 as embargoed
> until coordinated disclosure per the policy below.

## Primary contacts

| Role | Name | Channel |
|---|---|---|
| Engagement lead (Citrate side) | Saul Loveman | PGP-encrypted email to `saulweiloveman@gmail.com` |
| Sprint custodian | (rotating; current = Saul Loveman) | Same as above |
| Security inbox | `security@citratenetwork.org` (org-level) | Per `CitrateNetwork/.github/SECURITY.md` |

Operator-facing security reporting policy is at
[`CitrateNetwork/.github/SECURITY.md`](https://github.com/CitrateNetwork/.github/blob/main/SECURITY.md);
this packet inherits its embargo + acknowledgment rules.

## Communication channels

### Routine (Tier-2, Tier-3, clarifications)

- Email with PGP encryption for sensitive content.
- The engagement's per-week sync call (cadence agreed at
  kickoff; default Tuesday 10:00 ET).

### Urgent (Tier-1, suspected active exploit)

- Encrypted Signal (number exchanged at kickoff).
- Fallback: PGP-encrypted email with subject prefix
  `[TOB-NIST-AGENT-URGENT]`.

PGP keys are exchanged out-of-band at kickoff. The
SecurityOfficer key used for `PolicyBundle` signing is **not**
the same key as the engagement-comms PGP key; do not conflate
them.

**Citrate-side engagement-comms PGP key.** Fingerprint:

  `CCAB 2E1A 46A8 8B8A 7B3C  D377 FCFC DDF8 4584 16A1`

Ed25519, expires 2028-05-20. ASCII-armored public key at
[`PGP_PUBKEY.asc`](PGP_PUBKEY.asc); supporting metadata +
verification instructions at
[`PGP_FINGERPRINT.txt`](PGP_FINGERPRINT.txt). Verify the
fingerprint out of band (voice call at first sync, or a
signed message from a previously-trusted Citrate channel)
before trusting.

## Response SLAs (Citrate side)

Counted from receipt by the engagement lead, business days,
US Eastern.

| Tier | First acknowledgment | Triage decision | Remediation target |
|---|---|---|---|
| Tier-1 (critical / high) | Same business day | Within 2 BD | Within 14 calendar days |
| Tier-2 (medium) | Within 2 BD | Within 5 BD | Within 60 calendar days |
| Tier-3 (low / informational) | Within 5 BD | Within 10 BD | Best-effort, next sprint cycle |

Triage rubric is in
[`.agentile/audits/findings/RUBRIC.md`](../../.agentile/audits/findings/RUBRIC.md).

## Embargo policy

- All findings are embargoed from public disclosure until the
  remediation lands and the v1.0 tag is published.
- Tier-1 findings remain embargoed for an additional 30 days
  after v1.0 tag to allow operator-side patching cadence.
- The federation audit archive (under
  `citrate-agentile-archive/audits/`) publishes the final
  report after the embargo window closes.
- If a finding overlaps with an in-the-wild incident discovered
  during the engagement, the Citrate-side incident response
  takes precedence and the engagement lead coordinates the
  joint disclosure timing.

## Coordinated disclosure (if a finding overlaps with another project)

Findings touching `citrate-agent-runtime` upstream or
`citrate-chain` smart contracts are forwarded to those projects'
security inboxes by the engagement lead; the TOB report carries
a cross-reference and the affected project owns its own
disclosure timing.

## What goes where

| Item | Destination |
|---|---|
| Per-finding draft writeup | `.agentile/audits/findings/<id>-<slug>.md` in this repo |
| Triaged finding index | `.agentile/audits/findings/INDEX.md` |
| Final signed report from TOB | `citrate-agentile-archive/audits/<date>-tob-nist-agent-v1/` |
| Remediation PR | Branch `fix/tob-<finding-id>-<slug>` |
| Sign-off email from TOB | Archive folder above, alongside the report |

## Engagement kickoff checklist

- [ ] PGP key exchange complete.
- [ ] Signal contact verified.
- [ ] Weekly sync cadence agreed.
- [ ] Read-only deploy key issued for `nist-agent` repo
      access (TOB-side IP if requested).
- [ ] TOB confirms receipt of this packet.
- [ ] First sync call scheduled within 1 week of receipt.
- [ ] Reference deployment stood up per
      [`DEPLOYMENT.md`](DEPLOYMENT.md).

## Acknowledgments

Findings will be acknowledged in the final v1.0 release notes
and (with TOB's permission) in the federation's public
[`AUDIT_INDEX.md`](https://github.com/CitrateNetwork/citrate-agentile-archive/blob/main/audits/AUDIT_INDEX.md).
Per the federation's existing practice, the auditing
organization is credited; individual auditors are credited by
name only with their explicit consent.

## See also

- [`INDEX.md`](INDEX.md) — packet cover sheet.
- [`CitrateNetwork/.github/SECURITY.md`](https://github.com/CitrateNetwork/.github/blob/main/SECURITY.md) — org-wide security disclosure policy.
- [`.agentile/audits/findings/RUBRIC.md`](../../.agentile/audits/findings/RUBRIC.md) — tier classification rubric.
