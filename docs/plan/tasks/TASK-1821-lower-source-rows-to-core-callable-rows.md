# TASK-1821: Lower source rows into Core callable rows

## Status: ✅ Complete

## Description

Lower explicit source callable row summaries into `CoreRow` and `CoreType::Function { row, .. }` for the supported Phase 177 row families. This task owns the first real source-to-Core row bridge.

## Specification Reference

- [PLAN-178](../PLAN-178-SOURCE-TO-CORE-ROW-LOWERING-BRIDGE.md)
- [SPEC-098c](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [SPEC-099](../../spec/SPEC-099-CORE-LANGUAGE.md)
- [SPEC-100](../../spec/SPEC-100-CORE-TYPE-CHECKING.md)

## Dependencies

- TASK-1820 parsed row summary threading complete.
- TASK-1818 audit identifies Core construction owner files.

## Requirements

### Functional Requirements

1. Add conversion from explicit source row summary items to `CoreRowItem` for operation, resource, role, policy, channel, process, failure, evidence, and group rows supported by Phase 177.
2. Preserve whole-row variables and open-row tails where Core row variables can represent them.
3. Fail closed with precise diagnostics for unsupported row forms that cannot lower to Core yet.
4. Populate `CoreType::Function { row, .. }` or equivalent Core callable row carrier for explicit source rows.
5. Keep rowless source functions on existing empty/default row behavior.
6. Add focused Core lowering tests for inline rows, `where row` rows, open rows, evidence rows, and unresolved source-path metadata behavior.
7. Add tests proving Core typecheck sees the explicit row as a requirement row.

### Property Requirements

- Lowering records row requirements; it does not grant authority.
- Unsupported row forms must not be erased or converted to empty rows.
- Core row conversion should reuse Phase 177 Core helpers and CPS taxonomy where possible.

## TDD Steps

### Step 1: Write failing source-to-Core row tests

Add tests that inspect actual Core rows produced from functions with explicit source rows.

### Step 2: Verify RED

Run focused tests and confirm rows are currently empty, missing, or inaccessible.

### Step 3: Implement lowering

Patch the source-to-Core conversion path identified by TASK-1818.

### Step 4: Verify GREEN

Run focused tests plus `cargo test -p ash-core`, `cargo test -p ash-engine`, and `cargo test -p ash-typeck`.

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
  - cargo test -p ash-core
  - cargo test -p ash-engine
  - cargo test -p ash-typeck
  - git diff --check
checklist:
  - [x] Explicit source rows lower to Core rows.
  - [x] Inline and expanded row forms lower equivalently.
  - [x] Unsupported rows fail closed.
  - [x] Rowless functions preserve existing behavior.
```

## Completion Evidence

- Added `Workflow::core_callable_types` as the engine-owned Core Ash metadata bridge for imported and local callables.
- Added conversion from source computation rows to `CoreRow`, covering operation, resource, role, policy, channel, process, failure, evidence, group, whole-row variable, and open-tail rows.
- Added `crates/ash-engine/tests/task_1821_core_callable_row_lowering.rs` covering inline rows, expanded `where row` rows, open tails, rowless defaults, and supported target row families including evidence and group rows.
- Verification: `cargo fmt --check`; `cargo test -p ash-engine --test task_1821_core_callable_row_lowering -- --nocapture`; `cargo test -p ash-core`; `cargo test -p ash-engine`; `cargo test -p ash-typeck`; `git diff --check`; `python3 tools/docs/validate_orientation_indexes.py --self-test`.

## Dependencies for Next Task

This task feeds TASK-1822 and TASK-1823.
