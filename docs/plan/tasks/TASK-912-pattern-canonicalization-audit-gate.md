# TASK-912: Audit pattern/exhaustiveness constructor resolution and decide equality API vs pattern-specific API

## Status: 📝 Planned

## Description

Audit pattern/exhaustiveness constructor resolution and decide equality API vs pattern-specific API

## Specification Reference

- [SPEC-068](../../spec/SPEC-068-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md)
- [PLAN-117](../PLAN-117-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md)

## Dependencies

- ✅ SPEC-068: spec packet exists
- ✅ PLAN-117: implementation plan exists

## Requirements

1. Audit pattern/exhaustiveness constructor resolution and decide equality API vs pattern-specific API.
2. Preserve all non-goals and decision gates from the owning SPEC.
3. Add focused non-zero tests or an explicit docs/audit artifact matching the task type.
4. Avoid broadening later-task semantics by convenience or parser-only lowering.
5. Bind every downstream task to exact files, public entrypoints, and non-zero focused verification commands before Rust implementation starts.

## File Targets

- Create: docs/plan/audits/TASK-912-pattern-canonicalization-audit-gate.md
- Modify: docs/plan/tasks/TASK-913-*.md through TASK-916-*.md to replace fail-closed verification guards

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
  - test -f docs/plan/audits/TASK-912-pattern-canonicalization-audit-gate.md
  - git diff --check
checklist:
  - [ ] Audit artifact exists and names exact live callsites
  - [ ] Downstream fail-closed guards are replaced with focused non-zero commands
  - [ ] cargo fmt --check passes
  - [ ] git diff --check passes
  - [ ] Status surfaces and CHANGELOG are reconciled if this task completes
```

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-117](../PLAN-117-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

Pattern canonicalization is audit-first and must not solve under neutral computation heads.
