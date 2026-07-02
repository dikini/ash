# TASK-1818: Audit source-to-typechecker/Core row-loss boundaries

## Status: ✅ Complete

## Description

Audit where Phase 177 parsed callable rows are retained, transformed, or lost before reaching typechecker-facing summaries and Core callable rows. This task must produce the ownership map before implementation changes carriers.

## Specification Reference

- [PLAN-178](../PLAN-178-SOURCE-TO-CORE-ROW-LOWERING-BRIDGE.md)
- [PLAN-177](../PLAN-177-TARGET-ASH-ROW-SYNTAX-AND-CORE-CPS-ALIGNMENT.md)
- [SPEC-098c](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [SPEC-100](../../spec/SPEC-100-CORE-TYPE-CHECKING.md)

## Dependencies

- ✅ TASK-1817 planning packet exists.

## Requirements

### Functional Requirements

1. Audit parsed row carriers in `crates/ash-parser/src/surface.rs`.
2. Audit parser lowering and type conversion paths in `crates/ash-parser/src/lower.rs` and relevant parse/type files.
3. Audit module validation and imported/exported summary transport in `crates/ash-engine/src/module_loader.rs` and `crates/ash-engine/src/check.rs`.
4. Audit typechecker function/type summary conversion in `crates/ash-typeck/src/lib.rs`.
5. Audit Core row construction and public summaries in `crates/ash-core/src/core_ash.rs`, `crates/ash-core/src/core_ash_typecheck.rs`, and `crates/ash-core/src/semantic_summary.rs`.
6. Create `docs/audit/PHASE-178-source-to-core-row-loss.md` with current row flow, row-loss points, owner files, recommended tests, and downstream task adjustments.
7. Patch TASK-1819 through TASK-1823 if live code differs from PLAN-178 assumptions.

### Property Requirements

- Do not implement row lowering in the audit task.
- Every named row-loss point must be assigned to a later Phase 178 task or explicitly deferred.
- The audit must preserve the distinction between requirement rows and authority/admission.

## TDD Steps

### Step 1: Run read-only searches

Use `rg` and rust-analyzer/LSP when available for `CallableRow`, `SourceRow`, `Type::Fn`, `FunctionSignature`, `CoreType::Function`, `CoreRow`, and Phase 177 task/test names.

### Step 2: Map current row flow

Record where inline rows and `where row` rows are parsed, validated, summarized, imported/exported, converted to types, and lost.

### Step 3: Classify implementation seams

Mark each seam as preserve, extend, compatibility-convert, fail-closed, or defer.

### Step 4: Patch downstream tasks

Patch TASK-1819 through TASK-1823 if the audit identifies different owner files or prerequisite blockers.

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
  - [ ] Audit artifact exists.
  - [ ] Parser, engine, typechecker, summary, and Core row-loss boundaries are mapped.
  - [ ] Every downstream task has an audit disposition.
```

## Dependencies for Next Task

This task feeds TASK-1819 through TASK-1823.

## Completion Evidence

- Created [PHASE-178 source-to-Core row-loss audit](../../audit/PHASE-178-source-to-core-row-loss.md).
- Mapped parser, engine, typechecker, semantic-summary, and Core row retention/loss boundaries.
- Confirmed no PLAN-178 scope change is needed: rows are retained in parser AST and validation, then lost at rowless function/type conversion boundaries.
