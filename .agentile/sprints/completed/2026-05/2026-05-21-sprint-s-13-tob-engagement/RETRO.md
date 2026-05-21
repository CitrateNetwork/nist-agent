---
created: 2026-05-21T00:00:00Z
branch: feat/s-13-tob-engagement
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: complete
sprint: S-13
---

# Retro — Sprint S-13 (TOB audit packet)

## What landed

The packet that bootstraps the Trail of Bits engagement. Six
audience-tailored documents under `docs/audit/` + three
findings-tracker scaffolds under `.agentile/audits/findings/` +
ADR-012 codifying the phase split. No new Rust code — this is
the sprint that converts "code is complete" into "audit-ready,"
which is documentation work by definition.

The packet is structured so an external auditor can read
`INDEX.md` cold and reach a hands-on reference deployment in
under an hour: scope → threat model → build → deploy → exercise.
The findings-tracker scaffolds + tier rubric let triage start
the day the first finding lands rather than improvising under
deadline pressure.

## Metrics delta

| Metric | Before (S-12) | After (S-13) | Delta |
|---|---|---|---|
| `cargo test --workspace` | 202 | 202 | 0 (docs sprint) |
| Workspace crates | 13 | 13 | 0 |
| ADRs | 11 | 12 | +1 (ADR-012) |
| TLA+ specs | 5 | 5 | 0 |
| Audit packet docs | 0 | 6 | +6 |
| Findings-tracker docs | 0 | 3 | +3 |

Rule 3 (test count monotone non-decreasing) holds trivially.

## What went well

- **Phase split via ADR was the right call.** "Trail of Bits
  engagement preparation + remediation" reads like one sprint;
  doing the packet AND the engagement AND the remediation in
  one sprint would have either marked-complete with no actual
  audit run (dishonest) or stretched the sprint over months
  (dilutes the cadence). ADR-012 makes the boundary explicit
  and the close note keeps it honest.
- **Threat model is traceable.** Every claim in
  `THREAT_MODEL.md` cross-references a formal spec, a feature
  file, or a specific test. If a future remediation tightens
  one of the listed surfaces, the affected docs are obvious
  from the cross-references.
- **Test surface map in BUILD.md pays for itself.** Twelve
  load-bearing invariants → exact test locations. Auditors who
  want to confirm a defense exists can grep for the test name
  and run it. Lowers the audit-cost-per-invariant.
- **Findings tracker frontmatter is sprint-ratchet-compatible.**
  Every per-finding file will carry the same Rule-12
  frontmatter shape as sprints / ADRs / journals; the
  frontmatter ratchet check (Rule 12) catches an omission
  without a tracker-specific check.

## What was tricky

- **Sizing the "out of scope" section.** Easy to either (a)
  drown an auditor in disclaimers or (b) under-document and
  force them to guess. Settled on a tight table with one row
  per excluded item + the ADR that documents why. If TOB asks
  about anything not in the table, it's a packet bug worth
  fixing.
- **Reference deployment without a daemon binary entry point.**
  The audit boundary commit doesn't ship a runnable `bin`
  binary — that's S-12b. DEPLOYMENT.md works around this by
  pointing auditors at per-crate examples + the test suite,
  with a note that the binary entry point is deferred. Honest
  but slightly awkward; S-13a's remediation work may add a
  minimal `bin` for auditor convenience.
- **Coordinated disclosure policy needed a paragraph.**
  Cross-repo findings (touching `citrate-agent-runtime` or
  `citrate-chain`) are entirely plausible from the verifier
  surface; CONTACT.md spells out the routing so a TOB-side
  finding doesn't sit in an unaddressed inbox.
- **V1_READINESS as a public-facing checklist is honest about
  what's missing.** 4/6 ✅, 2/6 🟡. Resisted the urge to mark
  the TOB criterion as 🟡 with "in progress" since the
  engagement isn't kicked off yet; the table separates
  "packet shipped" from "engagement closed" which is the
  meaningful distinction.

## What to carry into S-13a / S-14

- **S-13a (per-finding remediation).** One branch + PR per
  Tier-1 finding, named `fix/tob-<id>-<slug>`. The
  TEMPLATE.md remediation checklist is the source of truth
  for what "done" looks like.
- **S-14 (pilot onboarding).** At least one operator per
  Phase-1 overlay class. The audit packet's reference
  deployment is the starting point — if anything in the
  packet trips a pilot, that's a packet bug.
- **S-12b (CI infra).** Parallel track. Independent of TOB
  findings; can start now.

## Open follow-ups

- **Daemon binary entry point.** Audit packet works without
  it, but a minimal `citrate-agent` bin would make the
  reference deployment more turnkey. Score for S-12b or a
  small S-13a-style cleanup sprint.
- **`cargo audit` + `cargo deny` baseline.** BUILD.md mentions
  both; we should run them at the audit boundary commit and
  archive the output so TOB inherits a clean dep-vuln baseline.
- **PGP key publication.** CONTACT.md references key exchange
  at kickoff; we should publish the engagement-comms PGP key
  fingerprint somewhere durable (federation `.github` repo?)
  so TOB can verify our identity out of band.
- **Federation rev bump to S-13 close.** Doc-only commits
  rarely move the pin, but for posterity the federation
  manifest should capture the "audit-ready" state — see the
  `V1_READINESS.md` note.
