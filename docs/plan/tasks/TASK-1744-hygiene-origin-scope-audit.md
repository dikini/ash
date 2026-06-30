# TASK-1744: Audit hygiene, origin, and scope boundary seams

## Status: ✅ Complete

## Description

Audit the live parser, expansion, lowering, engine/module-loader, and typechecker seams that must participate in Phase 171 hygiene. The audit must map current carriers and bypass paths before implementation tasks add or change any hygiene data structures.

## Specification Reference

- PLAN-171: `docs/plan/PLAN-171-MACRO-NOTATION-HYGIENE-AND-EXPANSION-BOUNDARIES.md`
- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`
- Phase 170 closeout: `docs/plan/PLAN-170-EXPANDED-SURFACE-INTEGRATION-AND-NOTATION-SCOPING.md`

## Dependencies

- ✅ TASK-1743: Phase 171 plan packet (complete)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Full macro hygiene | SPEC-095c §6, PLAN-170 follow-ons | Macro execution absent | No | Audit required carriers only | Audit must mark macro execution out of scope |
| Imported notation propagation | TASK-1740 | Summary carriers not ready | No | Preserve local-only unless audit proves otherwise | Negative leakage tests assigned to TASK-1747/TASK-1749 |

## Requirements

1. Inspect live code paths in:
   - `crates/ash-parser/src/surface.rs`
   - `crates/ash-parser/src/parse_expr.rs`
   - `crates/ash-parser/src/parse_module.rs`
   - `crates/ash-parser/src/lower.rs`
   - `crates/ash-engine/src/module_loader.rs`
   - relevant `crates/ash-typeck` consumers if surface-origin metadata reaches type checking.
2. Identify every current carrier for `SurfaceOrigin`, generated nodes, notation declarations, raw operator tokens, and expanded modules.
3. Identify every high-level API that can consume parsed surface without expanded-surface validation.
4. Produce `docs/audit/phase-171-hygiene-origin-scope-audit.md` with:
   - current-state map,
   - required target-state invariants,
   - implementation owner task for every gap,
   - positive visibility and negative leakage tests needed downstream.
5. Patch `SPEC-095c` or `SPEC-098c` only if the audit finds a missing normative invariant; do not add speculative macro execution semantics.

## TDD / audit steps

### Step 1: Symbol and call-path discovery

Use rust-analyzer for Rust symbols before broad text search. Trace `ExpandedSurfaceModule`, `SurfaceOrigin`, `OperatorSection`, notation declarations, and module-loader export collection.

### Step 2: Write audit artifact

**File:** `docs/audit/phase-171-hygiene-origin-scope-audit.md`

The artifact must have sections for parser carriers, expansion pass, module loading, type checking, diagnostics, and explicit non-goals.

### Step 3: Patch tasks if needed

If the audit finds a gap not owned by TASK-1745 through TASK-1749, patch the relevant task before marking TASK-1744 complete.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - python3 -c 'from pathlib import Path; s=Path("docs/audit/phase-171-hygiene-origin-scope-audit.md").read_text(); assert "SurfaceOrigin" in s; assert "ExpandedSurfaceModule" in s; assert "negative leakage" in s; assert "macro execution" in s and "out of scope" in s'
checklist:
  - [x] Audit artifact exists.
  - [x] Every gap has an owner task.
  - [x] Macro execution remains explicitly out of scope.
  - [x] Positive visibility and negative leakage tests are assigned.
```

## Completion Evidence

Created `docs/audit/phase-171-hygiene-origin-scope-audit.md`. The audit maps current `SurfaceOrigin`, `ExpandedSurfaceModule`, local notation, expansion, lowering, engine/module-loader, and typechecker seams; assigns every Phase 171 gap to TASK-1745 through TASK-1750; preserves macro execution as out of scope; and lists downstream positive visibility plus negative leakage tests.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 25
toolsets: [terminal, file]
```

## Dependencies for Next Task

The audit resolves D1–D4 inputs for TASK-1745 through TASK-1749.
