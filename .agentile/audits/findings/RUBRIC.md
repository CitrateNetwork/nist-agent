---
created: 2026-05-21T00:00:00Z
branch: feat/s-13-tob-engagement
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
sprint: S-13
---

# Finding-tier classification rubric

> Applied to every Trail of Bits finding when it lands.
> Drives the response SLA in [`docs/audit/CONTACT.md`](../../../docs/audit/CONTACT.md)
> and the v1.0 release gate (Tier-1 must be remediated and
> signed off before v1.0 tag per the planset's Phase-1
> exit criteria).

## Tier-1 — Critical / High

Triggers if **any** of the following apply:

- Quorum bypass: an action reaches `Agent::resume()` without
  the role-bound signatures the policy bundle requires.
- Overlay-ratchet bypass: `CmmcL3` removed from the active
  set, or a less-restrictive overlay activated without a valid
  `DecommissioningWorkflow` token.
- Audit-record integrity gap: WORM record modified after
  creation; or a `PolicyChange` / `OverlayDecommissioned`
  event suppressed.
- Mobile-pairing privilege escalation: `PairingState::Active`
  reached without all three of SecurityOfficer signature,
  allowlisted attestation, matching pairing token.
- Release verifier confused-deputy: bundle content extracted
  despite missing or mismatched signature.
- GGUF SI-7 bypass: tampered model loaded without
  `"SI-7: model hash mismatch"` refusal.
- Egress posture violation: outbound network call performed
  while `EgressPosture::Disabled`.
- Cryptographic break: signature forgery, key-extraction
  primitive, or canonical-encoding ambiguity that affects
  signed payloads.
- RCE on the daemon from an untrusted (Operator-rank or below)
  input.

**Response.** Same-day acknowledgment. Triage within 2 BD.
Remediation target 14 calendar days. Embargoed from public
disclosure until v1.0 tag + 30 days.

**v1.0 gate.** Tier-1 must be remediated and signed off by TOB
before v1.0 tag. The planset's Phase-1 exit criteria are
explicit on this.

## Tier-2 — Medium

Triggers if **any** of the following apply:

- A defense-in-depth layer is missing or weakened, but no
  Tier-1 invariant is reachable without an additional
  attacker-side prerequisite (e.g. compromised role key).
- DoS that requires sustained adjacent-network access.
- Information disclosure of metadata (record counts, queue
  sizes) but not of asset content.
- Wire-format ambiguity that does not affect signed payloads
  but complicates multi-language consumers.
- A policy-document gap that doesn't have a code consequence
  (e.g. a runbook omits a step that is mandatory in the
  feature scenario).
- Constant-time-ness gaps in non-load-bearing comparisons.
- Side-channel observations on shared hardware (single-tenant
  deployment is the reference, but operators may multi-tenant
  in practice).

**Response.** Acknowledgment within 2 BD. Triage within 5 BD.
Remediation target 60 calendar days. May land post-v1.0 if
appropriately tracked.

## Tier-3 — Low / Informational

Triggers if **any** of the following apply:

- Code-quality gap with no security consequence (e.g. clippy
  hint, doc-comment typo, redundant clone).
- A test could be tighter; not a coverage gap.
- A panic path that's reachable only with `unwrap` over a
  truly-infallible invariant (e.g. constants).
- A deprecation note for a dep that has no current
  vulnerability.
- A suggestion for a future-sprint improvement.

**Response.** Acknowledgment within 5 BD. Triage within 10 BD.
Best-effort remediation in the next sprint cycle.

## Out-of-band: TOB Discretion

If TOB's findings team judges a finding warrants a tier other
than what this rubric would assign, TOB's classification wins.
The rubric exists to set expectations, not to override
auditor judgment.

## Tier promotion / demotion

Tier may be revised on remediation analysis if:

- **Promote** (e.g. Tier-2 → Tier-1): Citrate-side analysis
  during remediation surfaces a Tier-1-triggering reachability
  path that TOB's initial review didn't see.
- **Demote** (e.g. Tier-1 → Tier-2): TOB's PoC turns out to
  require a Tier-1-only prerequisite (e.g. HSM compromise) and
  the surfaced finding is the *consequence*, not the primitive.

Any tier revision is documented in the per-finding file with
explicit Citrate + TOB sign-off.

## See also

- [`TEMPLATE.md`](TEMPLATE.md) — per-finding file shape.
- [`INDEX.md`](INDEX.md) — index of findings as they arrive.
- [`docs/audit/CONTACT.md`](../../../docs/audit/CONTACT.md) —
  the SLA table this rubric drives.
- [`.agentile/planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md`](../../planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md) —
  Phase-1 exit criteria that cite this rubric.
