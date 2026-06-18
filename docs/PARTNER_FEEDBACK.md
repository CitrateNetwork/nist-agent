# nist-agent — Partner Feedback

Thank you for evaluating nist-agent `v1.0.0-rc.1`. Your feedback feeds directly
into the v1.0 release and the active external-audit remediation track. This page
tells you how to reach us and what kind of feedback is most useful.

> **⟦SAUL TO CONFIRM⟧ the channels below before sending to partners.** Replace the
> placeholders with the real destinations and delete this note.
> - Email alias: `feedback@citratenetwork.org` *(confirm or replace)*
> - GitHub Discussions: enable on `CitrateNetwork/nist-agent` *(or a dedicated
>   `nist-agent-feedback` repo)*
> - Real-time channel: Slack / Discord invite for the partner cohort *(optional)*

---

## How to send feedback

**Preferred — structured findings:**
Open a GitHub Discussion (or issue) on the repo using the template below, or email
the alias with the same fields.

**For anything sensitive** (a suspected security issue, or content you can't post
publicly): email the alias directly and write `SECURITY` in the subject. Do not
file security issues in public Discussions.

## What's most useful to us

In priority order:

1. **"Would this pass our auditor?"** — concrete gaps between what nist-agent
   produces (audit log, anchors, overlay runbooks) and what your compliance
   regime actually requires.
2. **Workflow fit** — where the five-role quorum is too heavy, too loose, or
   doesn't map to your org's real approval chain.
3. **Overlay accuracy** — for your regulatory context (FERPA / COPPA / CIPA /
   HIPAA / FedRAMP High), what controls are missing or mis-stated.
4. **First-run friction** — anywhere the doctor or concierge left you stuck.
5. **Bugs on the real surfaces** (see the Evaluation guide for what's real vs
   preview — please don't file previews as bugs).

## Feedback template

```
Title: [area] short summary

Environment:
  - nist-agent version: v1.0.0-rc.1
  - OS / arch:
  - Interface: CLI / desktop app
  - Overlay in use: cmmc-l3 / ferpa / coppa / cipa / hipaa / fedramp-high
  - Key backend: FIDO2 / PIV-CAC / TPM / Secure Enclave / software

Category: compliance-fit / workflow / overlay-accuracy / onboarding / bug / other

What happened (or what's missing):

What you expected:

Steps to reproduce (if a bug):

Severity (your judgment): blocker / major / minor / nit

Would this block adoption in your org? yes / no / with-changes
```

## What happens to your feedback

- **Acknowledgement:** within 2 business days.
- **Triage:** within 7 business days, with a category and rough disposition
  (fix-for-v1.0 / backlog / by-design / already-tracked).
- **Security reports:** acknowledged within 1 business day and handled privately.

Compliance-fit and overlay-accuracy findings are reviewed against the active
Trail of Bits audit remediation and the v1.0 scope.

## Scope reminder

This is a **release candidate**. Some features are labelled previews (autonomous
agent loop, unsigned desktop app, mobile native apps) — see
**`docs/PARTNER_EVALUATION.md`**. Feedback on those is welcome as forward-looking
input, but they are known-deferred, not defects.
