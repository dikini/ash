# TASK-1747: Harden notation and macro scope-table boundaries

## Status: ✅ Complete

## Description

Make notation and macro scope boundaries explicit and testable. Phase 171 keeps Phase 170's conservative local-only notation rule unless real summary carriers are introduced. Macro scopes may be represented for diagnostics, but macro activation/execution remains fail-closed.

## Specification Reference

- PLAN-171: `docs/plan/PLAN-171-MACRO-NOTATION-HYGIENE-AND-EXPANSION-BOUNDARIES.md`
- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`
- Phase 170 closeout: `docs/plan/PLAN-170-EXPANDED-SURFACE-INTEGRATION-AND-NOTATION-SCOPING.md`

## Dependencies

- ✅ TASK-1744: Hygiene, origin, and scope audit

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Imported/exported notation activation | TASK-1740 | Summary carriers not ready | No | Preserve local-only behavior | Positive direct callable import and negative notation activation tests |
| Macro scope activation | SPEC-095c §6 | Macro system absent | No | Represent boundary only if needed; no activation | Macro invocation must fail before Core |

## Requirements

1. Revisit the active notation table and module-loader import/export behavior from Phase 170.
2. Keep local notation active only in the declaring module unless this task adds a real summary carrier.
3. If any summary carrier is added, require both:
   - positive tests for intended import/export visibility;
   - negative leakage tests for non-selected and private notation.
4. Add or tighten diagnostics for attempted imported notation use when only the callable target is imported.
5. Define macro scope-table placeholders only as fail-closed metadata; do not execute macros.
6. Ensure notation/macro scope metadata does not change callable authority, rows, contracts, failures, or evidence.

## TDD Steps

### Step 1: Write RED scope tests

**Expected file:** `crates/ash-engine/tests/task_1747_notation_macro_scope_boundaries.rs`.

Test cases:
1. Imported callable target remains callable by name.
2. Imported `pub` notation remains inactive in caller scope unless a real carrier is implemented.
3. Re-export does not activate notation transitively.
4. Macro-scope placeholder cannot activate syntax or bypass expansion.

### Step 2: Implement or preserve conservative behavior

**Likely files:**
- `crates/ash-parser/src/surface.rs`
- `crates/ash-engine/src/module_loader.rs`
- notation table helpers identified by TASK-1744

### Step 3: Document the selected scope contract

Patch `docs/design/phase-170-notation-summary-export-semantics.md` or create a Phase 171 design note if behavior changes.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-engine --test task_1747_notation_macro_scope_boundaries -- --nocapture
  - cargo test -p ash-engine --test task_1740_notation_non_propagation -- --nocapture
  - cargo test -p ash-engine
  - cargo check --workspace
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Local-only notation behavior remains explicit unless real carriers exist.
  - [x] Positive callable import and negative notation leakage tests pass.
  - [x] Macro scope placeholders do not execute or activate syntax.
```

## Completion Evidence

Added `crates/ash-engine/tests/task_1747_notation_macro_scope_boundaries.rs` covering the conservative scope contract without adding notation summary carriers: re-exported callable targets remain directly callable, re-exported `pub` notation does not activate transitively in callers, and macro-like placeholder syntax remains fail-closed at the module boundary. No callable authority, rows, contracts, failures, evidence, or macro execution behavior was changed.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 30
toolsets: [terminal, file]
```

## Dependencies for Next Task

Defines the scope behavior that TASK-1748 macro boundary and TASK-1749 validation must preserve.
