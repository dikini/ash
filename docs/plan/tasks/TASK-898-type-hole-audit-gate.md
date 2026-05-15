# TASK-898: Audit parser/core/typeck/do-target/type-function wildcard seams and freeze enabled hole positions

## Status: ✅ Complete

## Description

Audit parser/core/typeck/do-target/type-function wildcard seams and freeze enabled hole positions

## Specification Reference

- [SPEC-066](../../spec/SPEC-066-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md)
- [PLAN-115](../PLAN-115-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md)

## Dependencies

- ✅ SPEC-066: spec packet exists
- ✅ PLAN-115: implementation plan exists

## Requirements

1. Audit parser/core/typeck/do-target/type-function wildcard seams and freeze enabled hole positions.
2. Preserve all non-goals and decision gates from the owning SPEC.
3. Add focused non-zero tests or an explicit docs/audit artifact matching the task type.
4. Avoid broadening later-task semantics by convenience or parser-only lowering.
5. Bind every downstream task to exact files, public entrypoints, and non-zero focused verification commands before Rust implementation starts.

## File Targets

- Create: docs/plan/audits/TASK-898-type-hole-audit-gate.md
- Modify: docs/plan/tasks/TASK-899-*.md through TASK-902-*.md to replace fail-closed verification guards

## TDD / Execution Steps

1. Re-read the referenced SPEC, PLAN, and downstream task files.
2. Audit live parser/core/typeck/engine/test callsites named by the SPEC.
3. Write the audit artifact named in File Targets.
4. Patch downstream implementation tasks to replace fail-closed guards with exact focused non-zero commands.
5. Update this task status, the owning PLAN row, PLAN-INDEX, and CHANGELOG only after verification evidence is fresh.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - test -f docs/plan/audits/TASK-898-type-hole-audit-gate.md
  - git diff --check
checklist:
  - [x] Audit artifact exists and names exact live callsites
  - [x] Downstream fail-closed guards are replaced with focused non-zero commands
  - [x] cargo fmt --check passes
  - [x] git diff --check passes
  - [x] Status surfaces and CHANGELOG are reconciled if this task completes
```

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-115](../PLAN-115-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

Explicit `_` holes are not implicit currying and do not solve by inversion.
