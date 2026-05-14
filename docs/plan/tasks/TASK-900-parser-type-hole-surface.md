# TASK-900: Parse `_` holes at audited type-expression positions and preserve spans distinctly from type-function pattern wildcards

## Status: 📝 Planned

## Description

Parse `_` holes at audited type-expression positions and preserve spans distinctly from type-function pattern wildcards

## Specification Reference

- [SPEC-066](../../spec/SPEC-066-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md)
- [PLAN-115](../PLAN-115-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md)

## Dependencies

- ✅ SPEC-066: spec packet exists
- ✅ PLAN-115: implementation plan exists
- Depends on TASK-898 audit gate replacing this task's fail-closed verification guard
- Depends on TASK-899 completion

## Requirements

1. Parse `_` holes at audited type-expression positions and preserve spans distinctly from type-function pattern wildcards.
2. Preserve all non-goals and decision gates from the owning SPEC.
3. Add focused non-zero tests or an explicit docs/audit artifact matching the task type.
4. Avoid broadening later-task semantics by convenience or parser-only lowering.

## File Targets

- Exact files must be confirmed by the audit gate before implementation.

## TDD / Execution Steps

1. Re-read the referenced SPEC and this task file.
2. Add the smallest failing parser/typechecker/core/engine/doc test that proves this task's boundary.
3. Implement only this task's boundary; do not import later-task semantics.
4. Run the focused command set recorded in this task after the audit gate replaces any failing placeholder.
5. Update this task status, the owning PLAN row, PLAN-INDEX, and CHANGELOG only after verification evidence is fresh.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 16
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - false # TASK-900 must replace this guard with exact focused commands after its audit gate
checklist:
  - [ ] Focused tests are non-zero and pass
  - [ ] cargo fmt --check passes
  - [ ] git diff --check passes
  - [ ] Status surfaces and CHANGELOG are reconciled if this task completes
```

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-115](../PLAN-115-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

Explicit `_` holes are not implicit currying and do not solve by inversion.
