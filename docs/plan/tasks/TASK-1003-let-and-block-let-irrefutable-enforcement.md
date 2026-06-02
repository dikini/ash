# TASK-1003: Pure let irrefutability enforcement

## Status: 📝 Planned

## Description

Wire the irrefutability API into pure surface block `let` and lowered/core `Expr::Let` paths so checked pure code cannot bind a refutable pattern implicitly.

## Specification Reference

- [SPEC-076](../../spec/SPEC-076-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)
- [PLAN-126](../PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)

## Dependencies

- ✅ TASK-1000 packet exists
- ✅ TASK-1001 audit gate completed and patched this task with focused commands
- 📝 TASK-1002 shared irrefutability API must be implemented before this task wires pure/core let binders

## Requirements

1. Preserve SPEC-076 non-goals and decision gates.
2. Add RED tests for non-exhaustive `let Some` over `Option`, nested refutable fields inside one-variant ADTs, blocked product/variant cases, duplicate binders, and accepted variable/wildcard/single-variant cases.
3. Bind variables only after irrefutability succeeds.
4. Report construct-specific diagnostics with pattern span and rewrite hint.
5. Verify lowered core `Expr::Let` cannot bypass the source check.

## File Targets

- Modify candidates: `crates/ash-typeck/src/check_expr.rs`, `crates/ash-typeck/src/names.rs`, `crates/ash-typeck/src/error.rs`
- Test: `crates/ash-typeck/tests/task_1003_let_irrefutability.rs`

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
  - bash -lc "cargo test -p ash-typeck --test task_1003_let_irrefutability -- --list | rg 'pure_block_let_rejects_some_over_option|pure_block_let_rejects_nested_refutable_binders|pure_block_let_rejects_list_patterns|core_expr_let_rejects_refutable_host_ir|pure_block_let_accepts_variable_wildcard_and_single_variant|pure_block_let_duplicate_binders_rejected' && cargo test -p ash-typeck --test task_1003_let_irrefutability"
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
checklist:
  - [x] TASK-1001 replaced the fail-closed guard
  - [ ] RED tests fail before implementation and pass after implementation
  - [ ] Scope did not expand beyond SPEC-076
  - [ ] Diagnostics are asserted where required
```

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-126](../PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

Pure expression/block-let semantics
