# TASK-961: Callable syntax reference docs

## Status: 📝 Planned

## Description

Update reference pages, agent card, and amended legacy specs so daily readers learn the new syntax and do not copy stale examples. This task establishes the documentation guidance; TASK-963 performs the repository-wide `std/` and current `reference/` syntax migration after implementation support lands.

## Specification Reference

- SPEC-072 §10
- SPEC-072 C72-8
- SPEC-071

## Dependencies

- ✅ TASK-955: Tower callable syntax packet
- 📝 TASK-957 through TASK-960 completion for final implemented-behavior docs, or explicit draft labels if run earlier.

## Requirements

### Functional Requirements

1. Update `reference/language/functions.md` and sub-pages to prefer `(A, B) -> C` and `|args| -> body` in prose and canonical teaching examples.
2. Update `reference/agents/cards/functions.md`.
3. Patch SPEC-027/SPEC-031 notes or sections to point to SPEC-072 as the current callable syntax owner.
4. Hand off broad `std/` and top-level `reference/` syntax scans/migration to TASK-963 rather than closing the phase with stale examples.

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
reasoning: medium
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - false # Replace with TASK-specific focused docs/audit/closeout verification command before marking complete.
checklist:
  - [ ] Required docs/audit artifacts updated.
  - [ ] Status surfaces reconciled.
  - [ ] Independent review completed where required.
```

## Dependencies for Next Task

This task contributes to PLAN-121 and SPEC-072 completion.

## Notes

Area: reference documentation. Keep the callable-stratum axis separate from return type. Pure smart constructors returning `Act<T>`, `Proc<T>`, or `Workflow<T>` remain pure callables. Broad current-corpus migration for `std/` and `reference/` is owned by TASK-963.
