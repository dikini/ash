# TASK-1008: Runtime defensive pattern error boundary and closeout

## Status: ✅ Complete

## Description

Verify runtime pattern-match failure remains defensive only, reconcile status surfaces, run broad gates, and close SPEC-076 acceptance evidence.

## Specification Reference

- [SPEC-076](../../spec/SPEC-076-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)
- [PLAN-126](../PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)

## Dependencies

- ✅ TASK-1000 packet exists
- ✅ TASK-1001 audit gate completed and patched this task with focused commands
- ✅ TASK-1002 through TASK-1007 complete before closeout evidence is recorded

## Requirements

1. Preserve SPEC-076 non-goals and decision gates.
2. Add or update interpreter tests showing unchecked/host-created mismatches still produce structured runtime errors, using exact live variants such as expression `LetPatternBindFailed`, workflow `PatternMatchFailed`, and `NonExhaustiveMatch` or their TASK-1001-refreshed names.
3. Add integration tests proving checked source rejects binder failures before runtime.
4. Update SPEC-076 acceptance matrix with evidence.
5. Run broad gates and independent review before promoting status.

## File Targets

- Modify candidates: `crates/ash-interp/src/eval.rs`, `crates/ash-interp/src/execute.rs`, `crates/ash-interp/src/execute_observe.rs`, `crates/ash-interp/src/small_step.rs`, `crates/ash-cli/src/commands/check.rs`, `crates/ash-lsp-core/src/diagnostics.rs`
- Test: `crates/ash-interp/tests/task_1008_runtime_defensive_pattern_errors.rs`, `crates/ash-cli/tests/task_1008_matching_diagnostics_surface.rs`, `crates/ash-lsp-core/tests/task_1008_matching_diagnostics_lsp.rs`

## TDD / Execution Steps

1. Stop if this file still contains the fail-closed TASK-1001 verification guard.
2. Write RED tests named by TASK-1001.
3. Implement the smallest semantic change for this task only.
4. Run focused tests and required workspace checks.
5. Request independent review before marking complete.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 20
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - bash -lc "cargo test -p ash-interp --test task_1008_runtime_defensive_pattern_errors -- --list | rg 'runtime_defensive_expr_let_still_yields_LetPatternBindFailed_for_unchecked_ir|runtime_defensive_workflow_binder_still_yields_PatternMatchFailed_for_unchecked_ir|runtime_defensive_match_still_yields_NonExhaustiveMatch_for_unchecked_ir|checked_source_refutable_binders_fail_in_typeck_not_runtime' && cargo test -p ash-interp --test task_1008_runtime_defensive_pattern_errors"
  - bash -lc "cargo test -p ash-cli --test task_1008_matching_diagnostics_surface -- --list | rg 'cli_and_lsp_surface_matching_diagnostics_from_typeck_when_available' && cargo test -p ash-cli --test task_1008_matching_diagnostics_surface"
  - bash -lc "cargo test -p ash-lsp-core --test task_1008_matching_diagnostics_lsp -- --list | rg 'cli_and_lsp_surface_matching_diagnostics_from_typeck_when_available' && cargo test -p ash-lsp-core --test task_1008_matching_diagnostics_lsp"
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - cargo test --workspace
  - bash -lc "cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-plan-126-doc.log && ! grep -i '^warning:' /tmp/ash-plan-126-doc.log"
checklist:
  - [x] TASK-1001 replaced the fail-closed guard
  - [x] Focused tests were added first; they passed immediately as evidence of existing defensive behavior rather than a semantic runtime change
  - [x] Scope did not expand beyond SPEC-076
  - [x] Diagnostics are asserted where required
  - [x] Required focused closeout gates pass on the final diff
  - [x] SPEC-076/PLAN-126/PLAN-INDEX/CHANGELOG status and evidence are reconciled
```

## Dependencies for Next Task

This is the closeout task for [PLAN-126](../PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md). Later phases may cite this task's evidence but must not silently expand its scope.

## Notes

Runtime/closeout evidence:

- `crates/ash-interp/tests/task_1008_runtime_defensive_pattern_errors.rs`
  asserts unchecked core expression `let`, workflow `let`, and ordinary match
  fallback keep the live defensive variants `EvalError::LetPatternBindFailed`,
  `ExecError::PatternMatchFailed`, and `EvalError::NonExhaustiveMatch`.
- The same interpreter test proves checked source refutable workflow binders
  fail in type checking before runtime.
- `crates/ash-cli/tests/task_1008_matching_diagnostics_surface.rs` proves the
  CLI JSON check surface includes the typechecker matching diagnostic.
- `crates/ash-lsp-core/tests/task_1008_matching_diagnostics_lsp.rs` records the
  honest current LSP-core limitation: typecheck diagnostics are still deferred,
  so LSP does not invent a matching diagnostic before that path is wired.
- Broad closeout gates were run after focused remediation:
  `cargo test --workspace` passed, and `cargo doc --workspace --no-deps`
  completed without rustdoc warnings. Focused TASK-1008 tests, `fmt`, diff
  whitespace, workspace check, and workspace clippy also passed.
