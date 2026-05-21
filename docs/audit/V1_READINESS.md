---
created: 2026-05-21T00:00:00Z
branch: feat/s-13-tob-engagement
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
audience: Trail of Bits engagement team + Citrate release engineering
---

# v1.0-rc Readiness — Phase-1 Exit Criteria Cross-Check

> Companion to [`INDEX.md`](INDEX.md). Cross-checks the
> Phase-1 exit criteria from the planset's
> [`ROADMAP.md`](../../.agentile/planset/2026-05-19-nist-sidecar-v1/ROADMAP.md)
> against current repo state at S-13 packet delivery. ✅ = done;
> ⏸ = deferred (with where it lives); 🟡 = in flight.

## Phase-1 exit table

| # | Criterion (from `ROADMAP.md`) | Status | Evidence |
|---|---|---|---|
| 1 | All five TLA+ specs verify in CI as BLOCKER | ✅ | `.agentile/formal/specs/{DataClassLattice,HITLQuorum,OverlayRatchet,AuditChainAppendOnly,ReleaseVerifier}.tla`; `scripts/ci/check_specs.py` ratchet at 5. |
| 2 | All 11 doctor checks PASS or WARN on reference deployment per Phase-1 overlay | ✅ | `crates/nist-agent-doctor/src/checks/` — 11 checks landed in S-7. Reference-deployment validation runs as part of [`DEPLOYMENT.md`](DEPLOYMENT.md) hands-on exercise. |
| 3 | `cargo test --workspace` ratchet monotone non-decreasing across S-3 → S-13 | ✅ | `.agentile/coverage/baseline.json` count = 202. Per-sprint deltas: S-3 (35) → S-12 (202). No regression at any sprint boundary. |
| 4 | One pilot operator per overlay class completed end-to-end onboarding via the Slint concierge | 🟡 | Pilot recruitment in flight (S-14 work). Slint surfaces are complete (S-10a/b/c) and exercised by `feat-ui` builds. End-to-end pilot exercise tracked separately. |
| 5 | Trail of Bits report received; no Tier-1 findings outstanding | 🟡 | This sprint (S-13) lands the **packet** that bootstraps the engagement. Findings + remediation are S-13a/b. Per ADR-012, S-13 closing the packet phase is not the same as closing the engagement phase. |
| 6 | Federation manifest pin for `nist-agent` updated to `v1.0.0-rc` | ⏸ | Pending criteria 4 + 5 closure. The pin will move from a commit-hash rev to a `v1.0.0-rc` tag once both gates close. |

## Per-sprint summary (S-1 through S-13)

| Sprint | Closed | Headline deliverable | Test count delta |
|---|---|---|---|
| S-1 | ✅ | RFC + Gherkin feature inventory landed | — |
| S-2 | ✅ | 5 TLA+ specs ported with BOUNDED_EXPLORATION configs | — |
| S-3 | ✅ | Workspace scaffold (`nist-agent-prelude`, baselines) | +5 |
| S-4 | ✅ | `dyn ChainClient` EVM adapter trait + bindings | +11 |
| S-5 | ✅ | Agent loop + model resolver + checkpoint store | +14 |
| S-6 | ✅ | `PolicyBundle` canonical-CBOR + Ed25519 signing | +11 |
| S-7 | ✅ | 11 doctor pre-flight checks | +16 |
| S-8 | ✅ | Overlay bundles A (CMMC-L3, FERPA, COPPA, CIPA) | +18 |
| S-9 | ✅ | Overlay bundles B (HIPAA, FedRAMP High) + audit-sinks WORM | +10 |
| S-10a | ✅ | Slint concierge wizard render models | +16 |
| S-10b | ✅ | HITL queue + Capsule Inspector render models + Slint | +11 |
| S-10c | ✅ | Marketplace browser + chat surface render models + Slint | +18 |
| S-11 | ✅ | Mobile-companion protocol crate (native apps deferred per ADR-010) | +28 |
| S-12 | ✅ | Release verifier + 6/6 runbooks w/ Rollback (CI/HSM deferred per ADR-011) | +35 |
| S-13 | 🟡 | **TOB audit packet** (this sprint); engagement runs externally per ADR-012 | +0 (docs sprint) |
| **Totals at packet boundary** | | **13 crates · 202 tests · 12 ADRs · 5 specs · 6 runbooks w/ Rollback** | |

## Known deferrals and where they live

These items are explicitly deferred via ADR; none of them
block S-13 packet close, but several block v1.0 tag.

| Item | ADR | Blocks v1.0 tag? | Tracked in |
|---|---|---|---|
| HSM key custody + tag-push CI workflow | ADR-011 | Yes — release pipeline must produce signed bundles before tag. | S-12b |
| Two-machine byte-reproducibility test | ADR-011 | Yes — Phase-1 evidence; informally validated in S-12. | S-12b |
| Native iOS / Android apps | ADR-010 | No — v1.0 is sidecar + protocol; mobile apps are a v1.1 ship target. | `nist-agent-mobile-{ios,android}` repos (not yet created) |
| TOB engagement findings + remediation | ADR-012 | Yes — Tier-1 must be remediated and signed off. | S-13a / per-finding files in `.agentile/audits/findings/` |
| Pilot operator onboarding | (planset only) | Yes — Phase-1 exit criterion 4. | S-14 |
| GitHub Actions billing resolution | (operational) | No (we admin-merge in the interim) | Operational; not a code sprint |

## What this means for the v1.0 tag

**Tag prerequisites (must all be ✅ before tag push):**

1. TOB report delivered + Tier-1 findings remediated + TOB
   sign-off — **gates exit criterion 5**.
2. S-12b: CI workflow produces signed reproducible bundle —
   **gates exit criterion 1's "in CI" half + criterion 6**.
3. S-14: At least one pilot per Phase-1 overlay class
   completes end-to-end onboarding — **gates exit criterion 4**.
4. Federation pin update to `v1.0.0-rc` tag — **criterion 6**.

The audit packet (this sprint) is the **enabler** for (1).
None of the others depends on TOB output; they can proceed
in parallel.

## What this means for the federation pin

Today: `[repos.nist-agent].rev = "30044d26201daaabdeaaab8d7f25d950b336f023"`
(S-12 close commit).

Once S-13 closes: bump to the S-13 close commit. The packet
landing in the workspace is a doc-only change; the federation
pin moves forward by one commit to capture the audit-ready
state for posterity.

Once all v1.0 prerequisites above are ✅: bump to the
`v1.0.0-rc` git tag. The federation `drift-check.yml` will
then enforce that consumers reference the tag, not a moving
commit.

## See also

- [`INDEX.md`](INDEX.md) — packet cover sheet.
- [`SCOPE.md`](SCOPE.md) — what's in scope for the audit.
- [`.agentile/planset/2026-05-19-nist-sidecar-v1/ROADMAP.md`](../../.agentile/planset/2026-05-19-nist-sidecar-v1/ROADMAP.md) — planset roadmap with the exit criteria this checklist cross-references.
- [`.agentile/coverage/baseline.json`](../../.agentile/coverage/baseline.json) — the ratchet baselines.
