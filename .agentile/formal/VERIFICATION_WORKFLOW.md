---
created: 2026-05-20T00:00:00Z
branch: feat/s-2-tla-spec-port
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
---

# TLA+ Verification Workflow

> Operator-facing guide for verifying the five normative TLA+ specs
> locally and in CI. Pair-read with [`README.md`](README.md) (the
> spec inventory) and [`ADR-002`](../adrs/ADR-002-tla-custody-in-nist-agent.md)
> (why nist-agent holds canonical custody).

## Why TLA+

The RFC names five subsystems whose safety properties cannot be
fully exercised by Gherkin scenarios alone — they're state-machine
properties that require model checking. TLA+ + TLC explore the
reachable state space and report whether any reachable state
violates an invariant. Rule 10 makes this a BLOCKER: a spec that
regresses to FAIL (or a new spec that fails on first run) blocks
the merge.

Gherkin captures *intent* (the operator-visible contract). TLA+
captures *safety* (the invariants that must hold in every reachable
state). The CI cross-checks them: scenarios that pass Gherkin but
violate a TLA+ invariant are the highest-priority bug class in
this repo.

## Prerequisites

- **Java 17+.** TLA+ tools run on the JVM. `actions/setup-java` in
  CI; `apt install openjdk-17-jdk` locally on Debian/Ubuntu.
- **`tla2tools.jar`** (the TLC binary). One file, ~10 MB.

### One-time local install

```bash
mkdir -p ~/.local/share
curl -L \
  https://github.com/tlaplus/tlaplus/releases/latest/download/tla2tools.jar \
  -o ~/.local/share/tla2tools.jar
```

Then add an alias for convenience:

```bash
# in ~/.bashrc or ~/.zshrc
alias tlc='java -jar ~/.local/share/tla2tools.jar'
```

## Verifying a single spec

From the repo root:

```bash
java -jar ~/.local/share/tla2tools.jar -workers auto \
  -config .agentile/formal/specs/agent/ApprovalStateMachine.cfg \
  .agentile/formal/specs/agent/ApprovalStateMachine.tla
```

Successful output ends with `Model checking completed. No error has
been found.` and reports the number of distinct states explored.
Compare against [`README.md`](README.md)'s state-count column — a
significant drop may indicate the `.cfg` lost a constant or the
spec accidentally narrowed.

## Verifying all five specs

```bash
for s in .agentile/formal/specs/agent/*.tla; do
  echo "=== $(basename $s .tla) ==="
  java -jar ~/.local/share/tla2tools.jar -workers auto \
    -config "${s%.tla}.cfg" "$s" \
    || { echo "FAILED: $s"; exit 1; }
done
```

## CI workflow

`.github/workflows/tla-verify.yml` runs the same loop on every PR.
The job:

1. Installs JDK 17 via `actions/setup-java`.
2. Downloads `tla2tools.jar` from the official tlaplus/tlaplus
   release artifact (cached across runs).
3. Runs TLC against every `.tla` file under
   `.agentile/formal/specs/`.
4. Exits non-zero on any spec's failure → BLOCKER per Rule 10.

The separate `spec-ratchet` job in `ratchet-check.yml` enforces the
*count* floor (Rule 10's secondary axis): the number of specs never
decreases below the baseline in `.agentile/coverage/baseline.json`.

## Adding a new spec

1. Author `<Name>.tla` and `<Name>.cfg` in
   `.agentile/formal/specs/agent/`.
2. Add the spec to [`README.md`](README.md)'s inventory table.
3. Bump `specs.count` in `.agentile/coverage/baseline.json`.
4. Verify locally with the command above.
5. PR — `tla-verify.yml` runs the spec; `spec-ratchet` enforces
   the new baseline.

## Removing or replacing a spec

Per Rule 10, a spec can only be removed if a corrected replacement
lands in the **same commit**. Concretely:

- Update `<Name>.tla` to the corrected version.
- Update the `README.md` row.
- The replacement MUST verify in CI.
- The PR description names the prior `.tla`'s file hash so the
  removal is traceable in `git log`.

A spec may NOT be removed because it's slow or flaky. Slow specs
get their `.cfg` tightened (bound state space) or a separate longer
CI budget — never deletion.

## TLC budgets and timeouts

The default CI budget is 180 seconds × 4 workers × 2,500 MB heap.
The five normative specs all fit within this budget per the archive
baseline:

- `ApprovalStateMachine` — 336,292 states (≈ 60 s)
- `AuditChainIntegrity` — 35,435 states (≈ 10 s)
- `CapsuleInstallGate` — 2,600 states (≈ 2 s)
- `DataClassLattice` — small bounded model (≈ 5 s)
- `BreakGlassPath` — small bounded model (≈ 5 s)

If a future spec exceeds the default budget, the options (in order
of preference) are:

1. Tighten the `.cfg` (lower CONSTANT cardinalities, add
   StateConstraint).
2. Add SYMMETRY where state-space-equivalent permutations exist.
3. Move the spec to a separate `tla-verify-slow.yml` workflow that
   runs nightly rather than per-PR.
4. Document as `BOUNDED_EXPLORATION` (TIMEOUT with zero violations
   after ≥ 4M states explored) — same pattern as the archive's
   `TLC_BASELINE.md` uses for `AuditTrailIntegrity` and others.

Option 4 is a last resort and requires a sprint-level decision.

## Troubleshooting

### TLC reports `Module name MyModule is inconsistent with file name`

The TLA+ module declaration `MODULE Foo` must match the filename
`Foo.tla`. Fix the `MODULE` line, not the filename — readers
search by RFC-aligned filename.

### TLC reports `Cannot find module`

The spec uses `EXTENDS` to import another spec. If the imported
spec isn't in the same directory or in `tla2tools.jar`'s standard
library, TLC can't find it. Move the import to the same directory
or add to `tla2tools.jar`'s classpath.

### Verification hangs past the CI budget

Either: tighten the `.cfg`, or add the spec to the slow-run track.
Don't increase the per-PR budget without a sprint-level decision —
slow PRs degrade developer experience and CI throughput.
