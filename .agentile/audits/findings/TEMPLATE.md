---
created: {{YYYY-MM-DDTHH:MM:SSZ}}
branch: {{branch-on-which-finding-was-triaged}}
author: {{tob-finding-author-or-citrate-triager}}
status: {{open|in-remediation|fixed|signed-off|withdrawn}}
finding_id: {{TOB-NA-####}}
tier: {{tier-1|tier-2|tier-3}}
sprint: {{remediation-sprint-id}}
---

# {{TOB-NA-#### — short title}}

> Source: Trail of Bits, nist-agent v1.0-rc engagement
> (commit `30044d2`). Tier per
> [`RUBRIC.md`](RUBRIC.md). Embargo per
> [`docs/audit/CONTACT.md`](../../../docs/audit/CONTACT.md).

## Surface

Name the surface from
[`docs/audit/THREAT_MODEL.md`](../../../docs/audit/THREAT_MODEL.md)
this finding lives in. If the finding spans multiple
surfaces, list each.

## Summary

One paragraph — what's the bug, what's the impact.

## Reproducer

```rust
// Or shell, TOML, etc. — whatever reproduces the finding.
// Minimal; the auditor's full PoC lands in the report.
```

## Root cause

Pointer into the code: `path/to/file.rs:LINE` with the
specific function / branch responsible.

## Remediation plan

- [ ] Open a `fix/tob-{{finding_id}}-{{slug}}` branch.
- [ ] Land the code change.
- [ ] Add a test that pins the fix.
- [ ] Open a PR; tag the finding id in the description.
- [ ] On merge, re-export the audit log for evidence and
      append the PR + commit ids below.
- [ ] Request TOB sign-off.

### Tests added

- `crates/<crate>/src/<file>.rs::<test_name>` — pins the fix.
- (Optional) integration test under `tests/`.

### PR + commit

- PR: `https://github.com/CitrateNetwork/nist-agent/pull/####`
- Merge commit: `########`
- Sprint that closed the remediation: `{{sprint}}`

## TOB sign-off

- [ ] TOB confirms remediation closes the finding.
- Signed-off-by: `{{tob-signer}}`
- Date: `{{YYYY-MM-DD}}`
- Archive entry: `citrate-agentile-archive/audits/{{date-tob-nist-agent-v1}}/findings/{{finding_id}}.md`

## Disclosure timeline

| Event | Date |
|---|---|
| TOB report received | |
| Acknowledgment sent | |
| Triage complete | |
| Remediation merged | |
| TOB sign-off | |
| Embargo lifted | |

## Cross-references

- Related findings (if any): `TOB-NA-####`, `TOB-NA-####`.
- RFC clause this finding maps to: `RFC-CIT-AGENT-0001 §X.Y`.
- Feature scenario the test pins: `features/<path>.feature`.
- Formal spec (if any) that should be tightened:
  `.agentile/formal/specs/<Name>.tla`.
