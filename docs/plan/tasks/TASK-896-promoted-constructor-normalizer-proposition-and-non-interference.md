# TASK-896: Integrate promoted apps with type functions/propositions and prove runtime ADT/sealed-domain non-interference

## Status: 📝 Planned

## Description

Integrate promoted apps with type functions/propositions and prove runtime ADT/sealed-domain non-interference

## Specification Reference

- [SPEC-065](../../spec/SPEC-065-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md)
- [PLAN-114](../PLAN-114-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md)

## Dependencies

- ✅ SPEC-065: spec packet exists
- ✅ PLAN-114: implementation plan exists
- Depends on TASK-892 audit gate replacing this task's fail-closed verification guard
- Depends on TASK-895 completion

## Requirements

1. Integrate promoted apps with type functions/propositions and prove runtime ADT/sealed-domain non-interference.
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
  - false # TASK-896 must replace this guard with exact focused commands after its audit gate
checklist:
  - [ ] Focused tests are non-zero and pass
  - [ ] cargo fmt --check passes
  - [ ] git diff --check passes
  - [ ] Status surfaces and CHANGELOG are reconciled if this task completes
```

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-114](../PLAN-114-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

Promoted constructors are opt-in and distinct from sealed-domain markers and runtime constructors.
