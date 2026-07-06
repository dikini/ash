# TASK-1911: Process Concurrency Cross-Boundary Fixtures

**Status:** ✅ Complete
**Phase:** [PLAN-195: Process And Concurrency Model](../PLAN-195-PROCESS-AND-CONCURRENCY-MODEL.md)

## Description

Add parser, typecheck, Core/CPS, runtime, and CLI fixtures for process/concurrency behavior.

## Requirements

- Cover successful process/channel paths and fail-closed invalid boundary crossings.
- Include imports where process rows and sendability summaries cross module boundaries.
- Ensure CLI behavior reports structured diagnostics.

## TDD Steps

1. Add failing cross-boundary fixtures.
2. Wire fixture execution into focused tests.
3. Verify diagnostics and row preservation.

## Completion Checklist

- [x] Parser/typechecker/Core/CPS fixtures cover process carriers.
- [x] Runtime/CLI fixtures cover process and channel execution.
- [x] Invalid boundary crossings fail closed.

## Evidence

- Parser fixture: `crates/ash-parser/tests/task_1911_process_concurrency_rows.rs`
- Engine/typecheck/import/Core fixture:
  `crates/ash-engine/tests/task_1911_process_concurrency_cross_boundary.rs`
- Core/CPS fixture: `crates/ash-core/tests/task_1911_process_concurrency_core_cps.rs`
- Runtime sendability/channel fixture:
  `crates/ash-interp/tests/task_1911_process_concurrency_runtime_boundaries.rs`
- CLI JSON diagnostic fixture:
  `crates/ash-cli/tests/task_1911_process_concurrency_json_diagnostics.rs`

Focused verification:

```bash
cargo test -p ash-parser --test task_1911_process_concurrency_rows
cargo test -p ash-engine --test task_1911_process_concurrency_cross_boundary
cargo test -p ash-core --test task_1911_process_concurrency_core_cps
cargo test -p ash-interp --test task_1911_process_concurrency_runtime_boundaries
cargo test -p ash-cli --test task_1911_process_concurrency_json_diagnostics
```
