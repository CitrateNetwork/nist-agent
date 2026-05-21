---
created: 2026-05-21T00:00:00Z
branch: feat/s-13-kickoff
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: draft
audience: Trail of Bits intake / engagement team
---

# Outbound engagement letter — DRAFT

> Draft outbound message from Citrate Network Inc. to Trail of
> Bits requesting a security audit of `nist-agent v1.0-rc`.
> Treat the body below as a starting point — review, edit, and
> send from your own inbox over your preferred channel (TOB's
> intake form at trailofbits.com/services, or a direct email
> if you have a prior contact). This document stays in the
> repo as the artifact of the kickoff turn; once the engagement
> begins, supersede it with the signed SoW from TOB.

## Suggested subject line

`Citrate Network — nist-agent v1.0-rc security audit (RFC-CIT-AGENT-0001)`

## Body

---

Hi Trail of Bits team,

I'm Saul Loveman, principal architect at Citrate Network Inc.
We're approaching the v1.0 release-candidate of **nist-agent**,
a NIST-compliant agent harness implementing
[RFC-CIT-AGENT-0001](https://github.com/CitrateNetwork/nist-agent/blob/main/docs/rfcs/RFC-CIT-AGENT-0001.md).
Per the RFC's §11.1 audit posture, v1.0 is gated on a clean
external audit, and we're writing to ask whether your team has
availability to scope the engagement.

### What it is

`nist-agent` is a composable sidecar — a Rust workspace that
sits next to organizations' existing infrastructure and gates
every agent action through a cryptographically-signed, tiered-
risk, role-bound HITL quorum. It defaults to air-gapped
operation; it anchors tamper-evident audit state on Citrate L1
(or any EVM-compatible chain via a generic adapter trait); it
ships per-overlay policy bundles for NIST SP 800-171 / CMMC L3
baseline plus FERPA / HIPAA / FedRAMP-High / COPPA / CIPA
configurations.

The sidecar is the *consumer* of an upstream agent engine
(`citrate-agent-runtime`); that runtime has its own audit
lineage and is out of scope for this engagement. We're asking
you to look at the sidecar layer: ~9.6 kLOC of Rust across 13
workspace crates, with a focused load-bearing surface
(release verifier, HITL queue, overlay activation ratchet,
mobile-companion pairing, WORM audit sink, agent loop).

### Why we think it's a fit

- **Small, focused audit surface.** ~9.6 kLOC pure Rust, no
  `unsafe`, no FFI in scope, no async-runtime sharp corners
  beyond standard tokio.
- **Pre-existing formal specs.** Five TLA+ specs cover the
  load-bearing invariants (HITLQuorum, OverlayRatchet,
  DataClassLattice, AuditChainAppendOnly, ReleaseVerifier);
  your team has historically been strong on bridging Rust
  + TLA+ + property testing, and we'd like to lean on that.
- **Threat model and packet pre-authored.** We've shipped a
  full audit packet at
  [`docs/audit/`](https://github.com/CitrateNetwork/nist-agent/tree/main/docs/audit)
  on the repo so you can scope the engagement without playing
  scavenger hunt. The packet's
  [`INDEX.md`](https://github.com/CitrateNetwork/nist-agent/blob/main/docs/audit/INDEX.md)
  is the cover sheet.

### What we'd want from you

- A scoping conversation to size the engagement (we estimate
  3–6 weeks of audit calendar; happy to be wrong).
- A formal SoW + engagement letter.
- A point-of-contact for routine clarifications + an urgent
  channel for any Tier-1 findings as they emerge.

We're flexible on calendar; if your team can start in the next
4–6 weeks, that would land the engagement comfortably ahead
of our v1.0 target (Q4 2026).

### What we're offering for kickoff

- A read-only deploy key on the repo for your IP, if useful.
- A reference deployment standing up per
  [`docs/audit/DEPLOYMENT.md`](https://github.com/CitrateNetwork/nist-agent/blob/main/docs/audit/DEPLOYMENT.md).
- An engagement-comms PGP key (fingerprint
  `CCAB 2E1A 46A8 8B8A 7B3C  D377 FCFC DDF8 4584 16A1`,
  Ed25519, expires 2028-05-20; ASCII-armored at
  [`docs/audit/PGP_PUBKEY.asc`](PGP_PUBKEY.asc)) for any
  sensitive communication. Verify the fingerprint out of
  band at kickoff.

### Logistics

The audit boundary commit is `30044d2`
([nist-agent#02c01b9 main](https://github.com/CitrateNetwork/nist-agent/commit/02c01b91cca104a711a6b84cbda891ae492e8455)
ships the packet docs themselves; remediation lands on top).
The federation pin is currently at the packet-delivery commit;
we'll update to a `v1.0.0-rc` tag once the engagement closes
and the other v1.0 prerequisites are met.

We're at the early stage where the right answer is a 30-minute
call to decide if there's a fit and what the scope looks like.
Reply at your convenience.

Best,
Saul Loveman
Principal Architect, Citrate Network Inc.
`saulweiloveman@gmail.com` (engagement comms)
`security@citratenetwork.org` (org security inbox)

---

## Attachments to send

1. **`docs/audit/SCOPE.md`** — what's in / out of scope.
2. **`docs/audit/THREAT_MODEL.md`** — load-bearing surfaces +
   STRIDE.
3. **`docs/audit/V1_READINESS.md`** — Phase-1 exit criteria
   cross-check.
4. **`docs/audit/PGP_FINGERPRINT.txt`** — engagement-comms PGP
   public key fingerprint (generated this turn; full ASCII-
   armored key in
   [`docs/audit/PGP_PUBKEY.asc`](PGP_PUBKEY.asc)).

Or, simpler: send the link to the
[`docs/audit/INDEX.md`](INDEX.md) cover sheet and let them
crawl from there.

## After TOB responds

When TOB acknowledges:

1. Set the `S-13a` sprint from `backlog/` to `active/` and
   stamp the kickoff date.
2. Update `docs/audit/CONTACT.md` with the TOB-side names and
   channels you agreed on (currently placeholders).
3. Complete the kickoff checklist at the bottom of
   `docs/audit/CONTACT.md`.
4. First sync call scheduled.

## See also

- [`INDEX.md`](INDEX.md) — packet cover sheet.
- [`CONTACT.md`](CONTACT.md) — channels + SLAs + kickoff checklist.
- [`.agentile/sprints/backlog/sprint-s-13a-tob-remediation.md`](../../.agentile/sprints/backlog/sprint-s-13a-tob-remediation.md) — the sprint that becomes active once TOB acks.
