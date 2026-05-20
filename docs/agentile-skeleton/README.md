---
created: 2026-05-20T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
---

# agentile-skeleton — upstream reference

The files in this directory describe the upstream **agentile
methodology skeleton** that `nist-agent` was bootstrapped from. They
are not about `nist-agent` itself; keeping them here for newcomers
to understand the bootstrap genealogy and to make upstream-update
comparisons easier.

| File | What it describes |
|---|---|
| [`CHANGELOG.md`](CHANGELOG.md) | The agentile skeleton's own release notes (v0.x → v1.0.0-rc1). Useful when upgrading the skeleton itself. |
| [`INSTALL.md`](INSTALL.md) | Step-by-step skeleton install walkthrough — Options A/B/C, `bootstrap.sh` invocation, ratchet wiring. Useful when bringing a new federated repo online. |

The skeleton itself lives upstream at
[`github.com/citratenetwork/agentile`](https://github.com/citratenetwork/agentile)
and was cloned into the repo root before `bootstrap.sh` flattened it.

## Updating from upstream

When the skeleton releases a new version:

```bash
git remote add agentile https://github.com/CitrateNetwork/agentile.git
git fetch agentile main
git diff agentile/main -- .agentile/rules/ scripts/ .claude/
# cherry-pick what's safe to take
```

See `INSTALL.md`'s "Updating the skeleton in your project" section
for the canonical upgrade path.

## What's nist-agent-specific lives elsewhere

- Repo identity, scope, license: [`README.md`](../../README.md), [`LICENSE`](../../LICENSE) at the repo root.
- Agent entry point: [`.agentile/AGENT_ENTRY.md`](../../.agentile/AGENT_ENTRY.md).
- Canonical project constants: [`.agentile/CONFIG.md`](../../.agentile/CONFIG.md).
- Product scope: [`.agentile/PRODUCT_SPEC.md`](../../.agentile/PRODUCT_SPEC.md).
- Planset (the current workstream): [`.agentile/planset/2026-05-19-nist-sidecar-v1/`](../../.agentile/planset/2026-05-19-nist-sidecar-v1/).
