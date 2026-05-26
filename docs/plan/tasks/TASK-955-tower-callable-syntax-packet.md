# TASK-955: Tower callable syntax packet

## Status: ✅ Complete

## Description

Create the SPEC-072/PLAN-121/task packet for tower callable type and closure syntax and register it in the project status surfaces.

## Specification Reference

- SPEC-072 §1-§13
- PLAN-121

## Dependencies

- ✅ TASK-954: Functions reference chapter expansion

## Requirements

### Functional Requirements

1. Create `docs/spec/SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md`.
2. Create `docs/plan/PLAN-121-TOWER-CALLABLE-SYNTAX.md`.
3. Create TASK-955 through TASK-963 task files, with TASK-962 as the final closeout gate.
4. Update `docs/spec/README.md`, `docs/plan/PLAN-INDEX.md`, `CHANGELOG.md`, and amended legacy spec notes.

### Non-goals

- Do not implement Act/Proc/Workflow callable runtime semantics unless this task explicitly says so.
- Do not introduce partial application or currying.
- Do not silently reinterpret higher-stratum arrows as pure functions returning computation values.

## Work Steps

1. Inspect the exact live files named by TASK-956 or this task.
2. Write focused RED tests or docs assertions before changing behavior.
3. Implement or document the minimal target behavior.
4. Run focused verification.
5. Update status surfaces and CHANGELOG.md if files beyond tests are changed.
6. Request independent review before marking complete.

## Dispatch

```yaml
agent: codex
reasoning: high
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 - <<'PY'
from pathlib import Path
required = [
 'docs/spec/SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md',
 'docs/plan/PLAN-121-TOWER-CALLABLE-SYNTAX.md',
 'docs/plan/tasks/TASK-955-tower-callable-syntax-packet.md',
 'docs/plan/tasks/TASK-956-callable-syntax-audit-gate.md',
 'docs/plan/tasks/TASK-957-pure-callable-type-parser.md',
 'docs/plan/tasks/TASK-958-callable-type-typeck-rendering.md',
 'docs/plan/tasks/TASK-959-pure-closure-arrow-syntax.md',
 'docs/plan/tasks/TASK-960-reserved-tower-callable-arrows.md',
 'docs/plan/tasks/TASK-961-callable-syntax-reference-docs.md',
 'docs/plan/tasks/TASK-962-tower-callable-syntax-closeout.md',
 'docs/plan/tasks/TASK-963-stdlib-and-reference-callable-syntax-migration.md',
]
for rel in required:
    assert Path(rel).exists(), rel
PY
checklist:
  - [x] SPEC-072 drafted.
  - [x] PLAN-121 drafted.
  - [x] TASK-955 through TASK-963 created, with TASK-962 as the final closeout gate.
  - [x] PLAN-INDEX, spec index, amended spec notes, and CHANGELOG updated.
```

## Dependencies for Next Task

This task contributes to PLAN-121 and SPEC-072 completion.

## Notes

Area: docs/planning. Keep the callable-stratum axis separate from return type. Pure smart constructors returning `Act<T>`, `Proc<T>`, or `Workflow<T>` remain pure callables.
