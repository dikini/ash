# TASK-1000: Explicit refutable matching design/spec/plan packet

## Status: ✅ Complete

## Description

Create the design, specification, plan, task files, indexes, and changelog entry for banning implicit refutable matching while preserving explicit refutable forms.

## Specification Reference

- [SPEC-076](../../spec/SPEC-076-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)
- [PLAN-126](../PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)
- [DESIGN-044](../../design/DESIGN-044-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)

## Dependencies

- ✅ User requirement captured in session
- ✅ SPEC-068 pattern canonicalization implemented MVP exists

## Requirements

1. Create DESIGN-044, SPEC-076, PLAN-126, and TASK-1000 through TASK-1008.
2. Register SPEC-076 in the spec index.
3. Register Phase 131 in PLAN-INDEX.
4. Add a CHANGELOG entry under [Unreleased].
5. Verify new files and links without changing Rust implementation code.

## File Targets

- Create: docs/design/DESIGN-044-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md
- Create: docs/spec/SPEC-076-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md
- Create: docs/plan/PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md
- Create: docs/plan/tasks/TASK-1000-*.md through TASK-1008-*.md
- Modify: docs/spec/README.md
- Modify: docs/plan/PLAN-INDEX.md
- Modify: CHANGELOG.md

## TDD / Execution Steps

1. Inspect live pattern/exhaustiveness code surfaces and existing SPEC-068 packet.
2. Write DESIGN-044 with the three pattern-use categories.
3. Write SPEC-076 with normative rules, error model, and acceptance matrix.
4. Write PLAN-126 and task files with fail-closed downstream verification guards.
5. Patch indexes and CHANGELOG.
6. Run docs verification commands and record results in final response.

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
  - test -f docs/design/DESIGN-044-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md
  - test -f docs/spec/SPEC-076-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md
  - test -f docs/plan/PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md
  - python3 -c 'from pathlib import Path; missing=[n for n in range(1000,1009) if not list(Path("docs/plan/tasks").glob(f"TASK-{n}-*.md"))]; assert not missing, missing'
  - git diff --check
checklist:
  - [x] Packet files exist
  - [x] Indexes patched
  - [x] Changelog patched
  - [x] No Rust implementation started
```

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-126](../PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

Docs-only packet task. Implementation begins at TASK-1001 after the audit gate.

## Completion Evidence

- Created the initial docs packet on 2026-06-02.
- Verification is recorded in the controller response for this session.
