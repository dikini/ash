# TASK-892: Audit live ADT, sealed-domain, type-function, normalizer, summary, and pattern seams before implementation

## Status: ✅ Complete

## Description

Audit live ADT, sealed-domain, type-function, normalizer, summary, and pattern seams before implementation

## Specification Reference

- [SPEC-065](../../spec/SPEC-065-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md)
- [PLAN-114](../PLAN-114-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md)

## Dependencies

- ✅ SPEC-065: spec packet exists
- ✅ PLAN-114: implementation plan exists

## Requirements

1. Audit live ADT, sealed-domain, type-function, normalizer, summary, and pattern seams before implementation.
2. Preserve all non-goals and decision gates from the owning SPEC.
3. Add focused non-zero tests or an explicit docs/audit artifact matching the task type.
4. Avoid broadening later-task semantics by convenience or parser-only lowering.
5. Bind every downstream task to exact files, public entrypoints, and non-zero focused verification commands before Rust implementation starts.

## File Targets

- Create: docs/plan/audits/TASK-892-promoted-constructor-audit-gate.md
- Modify: docs/plan/tasks/TASK-893-*.md through TASK-896-*.md to replace fail-closed verification guards

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
  - test -f docs/plan/audits/TASK-892-promoted-constructor-audit-gate.md
  - git diff --check
checklist:
  - [x] Audit artifact exists and names exact live callsites
  - [x] Downstream fail-closed guards are replaced with focused non-zero commands
  - [x] cargo fmt --check passes
  - [x] git diff --check passes
  - [x] Status surfaces and CHANGELOG are reconciled if this task completes
```

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-114](../PLAN-114-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

Promoted constructors are opt-in and distinct from sealed-domain markers and runtime constructors.

## Completion Notes

Completed audit gate in `docs/plan/audits/TASK-892-promoted-constructor-audit-gate.md`, chose explicit `data kind <Name> from type <Adt>;` syntax, and replaced TASK-893 through TASK-896 fail-closed verification guards with focused non-zero commands. Baseline `cargo check --workspace` passed before implementation.
