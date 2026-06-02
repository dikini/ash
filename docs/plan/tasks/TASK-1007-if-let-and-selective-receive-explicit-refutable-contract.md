# TASK-1007: If-let implicit-complement and selective receive contract

## Status: 📝 Planned

## Description

Treat `if let ... else` as a total implicit-complement eliminator while preserving current selective `receive` as an explicit refutable filtering form. Ensure diagnostics, branch scopes, and branch result typing remain correct.

## Specification Reference

- [SPEC-076](../../spec/SPEC-076-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)
- [PLAN-126](../PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)

## Dependencies

- ✅ TASK-1000 packet exists
- 📝 TASK-1001 audit gate must complete and patch this task before implementation starts
- 📝 TASK-1002 through TASK-1006 must complete before this task finalizes explicit-complement/refutable-form behavior

## Requirements

1. Preserve SPEC-076 non-goals and decision gates.
2. Add tests proving `if let ... else` accepts refutable patterns as total `P | not P` eliminators, binds names only in the then branch, typechecks `else` under the original environment, and unifies branch result types.
3. Add parser-entrypoint tests proving the live syntax works in each source context identified by TASK-1001, and proving `if let` without `else` is rejected or remains unsupported if the parser admits the spelling.
4. Add tests proving pattern type/impossibility errors from `check_pattern` are propagated rather than silently ignored.
5. Add tests/diagnostics for degenerate complements: irrefutable `P` leaves the expression accepted with a non-fatal unreachable-else diagnostic/warning; impossible `P` is a hard error.
6. Add tests for then-only binding scope, no binding escape after the expression, branch shadowing of outer names, branch result type mismatch, and absence of negative type refinement in `else`.
7. Add tests proving current selective receive arms remain non-exhaustive/selective under existing semantics, including guard/order/no-match behavior where TASK-1001 confirms the live surface.
8. Document any ambiguity between selective receive and future total protocol receive.
9. Prevent shared irrefutability enforcement from accidentally rejecting these explicit forms.

## File Targets

- Modify: exact files to be patched by TASK-1001 audit
- Test: exact focused test target to be patched by TASK-1001 audit

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
  - false # TASK-1001 must replace this guard with exact focused non-zero commands before TASK-1007 implementation starts
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
checklist:
  - [ ] TASK-1001 replaced the fail-closed guard
  - [ ] RED tests fail before implementation and pass after implementation
  - [ ] Scope did not expand beyond SPEC-076
  - [ ] Diagnostics are asserted where required
```

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-126](../PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

`if let ... else` is total by implicit complement (`P | not P`), not by explicit constructor enumeration. Its complement branch is checked under the original environment; no negative type refinement is required or exposed in this phase. No `match let` or `let match` syntax is in scope; use ordinary `let x = match ...` if common binding after a match is needed.
