---
created: 2026-05-31T18:00:00Z
branch: audit/audit-ref-2026-05-31
author: claude-opus-4-8 (audit wiring)
status: active
audit_id: 2026-05-31-federation-deep-audit
---

# Active audit reference — `nist-agent`

> This repo's two-way link into the centralized federation audit trail
> (Hybrid topology). It is what an outsider cloning *this* repo follows to the
> findings. The canonical audit home is the `citrate-security` repo.

This repo participates in the **Inaugural Federation Deep Audit** opened
2026-05-31 as a **Tier 1 — full audit** surface.

- Audit root: `citrate-security/audits/2026-05-31-federation-deep-audit/`
- This repo's folder: `.../per-repo/nist-agent/` — `MAP.md` (architecture + data-flow +
  two Mermaid diagrams) and `INVENTORY.md` (feature/app/function table)
- Findings roll-up: `.../06_FINDINGS.md`
- Audit index: `citrate-security/audits/AUDIT_INDEX.md`
- Standard: `citrate-security/.agentile/standard/AGENTILE_AUDIT_STANDARD.md`

## Local evidence (Hybrid topology)

Remediation diffs and post-fix e2e/regression evidence for findings against this
repo land **here**, next to the code, under
`.agentile/audits/2026-05-31-federation-deep-audit/` and are linked back from the
central finding.

## Scope for this repo

- Phase-1 mapping complete; see `per-repo/nist-agent/MAP.md` for the mapped surface and
  the high-risk areas queued for Phase-2 vuln-hunting.

## Subsequent audit — Federation-Wide Audit 2026-06-20 (chunk FWA-C9)

This repo was re-audited under the **federation-wide audit 2026-06-20**, chunk
**FWA-C9** (Agent runtime & WASM sandbox), at SHA `5d683dc` on
`docs/partner-eval`.

- Audit root: `citrate-security/audits/2026-06-20-federation-wide-audit/`
- Chunk report: `.../per-chunk/FWA-C9/REPORT.md` (+ `findings.json`,
  `evidence/test-runs.txt`)
- Central deferrals ledger: `.../DEFERRALS.md`
- Standard: Agentile-Audit Standard **v0.2**

### Findings touching this repo

- **FWA-C9-01 (Medium)** — PolicyBundle verified but never enforced at
  `dispatch()`. **DEFERRED-WITH-OWNER** (re-confirmed by enforcement-surface
  determination at HEAD: no gateable action surface exists). Local remediation
  record + tripwire-in-waiting:
  `.agentile/audits/2026-06-21-fwa-remediation/REMEDIATION_LOG.md`.
- (FWA-C9-02 Low + OBS-1/2 are scoped to `citrate-agent-runtime`, not this repo.)

### Carry-forward dispositions confirmed

- NIST_AGENT-001 *load/verify* half: **RESOLVED** fail-closed
  (`crates/nist-agent-daemon/src/policy.rs`, `tests/policy_enforcement.rs`).
- NIST_AGENT-002 SI-7 TOCTOU: **RESOLVED / NOT REPRODUCED** at HEAD.
- Prior "nist-agent 0/10": **STALE** — workspace builds, policy suites pass.
