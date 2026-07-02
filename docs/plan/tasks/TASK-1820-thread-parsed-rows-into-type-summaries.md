# TASK-1820: Thread parsed rows into function/type summaries

## Status: ✅ Complete

## Description

Thread explicit Phase 177 callable rows into function/type summary paths used by engine/module validation and typechecker-facing callable signatures. This task closes the summary transport part of the rowless `Type::Fn` boundary without adding row inference.

## Specification Reference

- [PLAN-178](../PLAN-178-SOURCE-TO-CORE-ROW-LOWERING-BRIDGE.md)
- [SPEC-097b](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-098c](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)

## Dependencies

- TASK-1819 row-bearing callable summary carriers complete.

## Requirements

### Functional Requirements

1. Thread explicit row summaries through local function summaries.
2. Thread explicit row summaries through imported/exported public callable summaries.
3. Preserve inline rows and `where row` rows equivalently.
4. Keep rowless `Type::Fn` behavior for callables without explicit row syntax.
5. Preserve unresolved source-path operation rows as requirement metadata, not as resolved impl-qualified identity.
6. Add tests for local and imported public functions with explicit rows.
7. Add tests proving row summary transport does not activate callable execution or provider authority.

### Property Requirements

- This task must not infer rows from function bodies.
- This task must not discharge rows.
- This task must not install runtime handlers, providers, roles, or resources.

## TDD Steps

### Step 1: Write failing summary transport tests

Add focused engine/typechecker tests for local and imported function summaries carrying explicit rows.

### Step 2: Verify RED

Run focused tests and confirm explicit rows disappear at the summary/type boundary.

### Step 3: Implement threading

Patch the audited summary conversion paths so explicit rows travel with callable signatures.

### Step 4: Verify GREEN

Run focused tests plus `cargo test -p ash-engine` and `cargo test -p ash-typeck`.

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
  - cargo fmt --check
  - cargo test -p ash-engine
  - cargo test -p ash-typeck
  - git diff --check
checklist:
  - [x] Local callable summaries preserve explicit rows.
  - [x] Imported/exported callable summaries preserve explicit rows.
  - [x] Rowless callables remain compatible.
  - [x] No provider/admission/handler authority is installed.
```

## Completion Evidence

- Added `Workflow::callable_row_requirements` and public callable row requirement summary exports so explicit inline rows and `where row` rows are visible at the engine workflow summary boundary without changing rowless callable behavior.
- Added `crates/ash-engine/tests/task_1820_row_summary_transport.rs` covering local inline-row functions, imported `where row` functions, and rowless imported functions.
- Verification: `cargo fmt --check`; `cargo test -p ash-engine`; `cargo test -p ash-typeck`; `git diff --check`; `python3 tools/docs/validate_orientation_indexes.py --self-test`.

## Dependencies for Next Task

This task feeds TASK-1821 and TASK-1823.
