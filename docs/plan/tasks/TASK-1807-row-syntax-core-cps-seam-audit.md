# TASK-1807: Audit row syntax, Core row, CPS row, and lowering seams

## Status: ✅ Complete

## Description

Audit the live parser, type-checking, Core, CPS, engine, and lowering seams that will own Phase 177 row syntax and row taxonomy work. This task must produce a current ownership map before implementation changes row carriers.

## Specification Reference

- [PLAN-177: Target Ash Row Syntax and Core/CPS Alignment](../PLAN-177-TARGET-ASH-ROW-SYNTAX-AND-CORE-CPS-ALIGNMENT.md)
- [SPEC-095b: Target Grammar](../../spec/SPEC-095b-TARGET-GRAMMAR.md)
- [SPEC-096b: Target Effect System](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-098c: Surface-to-Core Lowering](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [NOTE-020: Computation Row Taxonomy](../../notes/NOTE-020-COMPUTATION-ROW-TAXONOMY.md)

## Dependencies

- ✅ TASK-1806 planning packet exists.

## Requirements

### Functional Requirements

1. Audit parser/function/type surfaces in `crates/ash-parser/src/surface.rs`, `crates/ash-parser/src/parse_module.rs`, `crates/ash-parser/src/parse_expr.rs`, and `crates/ash-parser/src/lower.rs`.
2. Audit typechecker and engine/module summary surfaces in `crates/ash-typeck/src/lib.rs`, `crates/ash-engine/src/check.rs`, and `crates/ash-engine/src/module_loader.rs`.
3. Audit Core row carriers and text parsing/formatting in `crates/ash-core/src/core_ash.rs`, `crates/ash-core/src/core_ash_text.rs`, `crates/ash-core/src/core_ash_typecheck.rs`, and `crates/ash-core/src/core_ash_lower.rs`.
4. Audit CPS row/effect carriers in `crates/ash-core/src/cps.rs` and interpreter tests that construct rows directly.
5. Create `docs/audit/PHASE-177-row-syntax-core-cps-seams.md` with current owners, lossiness boundaries, task-risk decisions, and recommended focused test locations.
6. Patch TASK-1808 through TASK-1814 if the live substrate differs from PLAN-177 assumptions.

### Property Requirements

- Do not implement row behavior in the audit task.
- Every later task must have an owner file list and at least one focused verification command.
- Any silent row-loss boundary must be named explicitly.

## TDD Steps

### Step 1: Run read-only searches

Use `rg`, rust-analyzer/LSP when available, and targeted file reads for `CoreRow`, `CoreRowItem`, `EffectRow`, `EffectItem`, `EffectOp`, `where`, function type parsing, and lowering helpers.

### Step 2: Record current row flow

Map current flow from source syntax to surface AST, lowered AST/Core, Core rows, CPS rows, and runtime/interpreter consumers.

### Step 3: Classify risks

Mark each seam as preserve, extend, compatibility-convert, fail-closed, or defer.

### Step 4: Patch downstream task scope

If the audit finds a missing prerequisite, patch the relevant task with a split/defer gate instead of leaving impossible implementation instructions.

## Dispatch

```yaml
agent: codex
reasoning: medium
toolsets: [terminal, file, rust-analyzer]
```

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
checklist:
  - [x] Audit artifact exists.
  - [x] Parser, typechecker, engine, Core, CPS, and lowering seams are mapped.
  - [x] Silent row-loss boundaries are named.
  - [x] Downstream task files are patched if assumptions changed.
```

## Dependencies for Next Task

This task feeds TASK-1808 through TASK-1814.

## Completion Evidence

- Added [PHASE-177-row-syntax-core-cps-seams.md](../../audit/PHASE-177-row-syntax-core-cps-seams.md) with the live ownership map, named lossy boundaries, task-risk decisions, and focused verification targets.
- Patched downstream TASK-1814 wording to make the source-row-to-summary/Core preservation gap an explicit test target.
