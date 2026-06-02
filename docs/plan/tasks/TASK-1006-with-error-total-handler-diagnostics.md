# TASK-1006: with_error total handler diagnostics

## Status: 📝 Planned

## Description

Decide and implement the first totality rule for `with_error` handlers over known closed payload types, or explicitly record why payload typing blocks enforcement in this phase.

## Specification Reference

- [SPEC-076](../../spec/SPEC-076-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)
- [PLAN-126](../PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)

## Dependencies

- ✅ TASK-1000 packet exists
- ✅ TASK-1001 audit gate completed and patched this task with focused commands

## Requirements

1. Preserve SPEC-076 non-goals and decision gates.
2. Audit failure payload typing and handler arm typing.
3. If closed payload types are available, add non-exhaustive handler RED tests and enforce coverage using the same wildcard/default and blocked-universe rules as `match`.
4. If unavailable, add a precise blocked/deferred diagnostic path and task note rather than silent best-effort; update SPEC-076 evidence to say coverage is deferred for unavailable payload universes.
5. Preserve handler body type unification.

## File Targets

- Modify candidates: `crates/ash-typeck/src/check_expr.rs`, `crates/ash-typeck/src/exhaustiveness.rs`, `crates/ash-typeck/src/error.rs`
- Test: `crates/ash-typeck/tests/task_1006_with_error_total_handlers.rs`

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
  - bash -lc "cargo test -p ash-typeck --test task_1006_with_error_total_handlers -- --list | rg 'with_error_total_handler_reports_or_defers_closed_payload_missing_case|with_error_handler_pattern_type_error_is_structured|with_error_wildcard_accepts_open_payload|with_error_branch_type_mismatch_reports_handler_context' && cargo test -p ash-typeck --test task_1006_with_error_total_handlers"
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

Operational-bottom handler semantics
