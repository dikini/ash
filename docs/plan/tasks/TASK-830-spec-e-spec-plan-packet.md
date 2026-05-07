# TASK-830: Promote DESIGN-034 SPEC-E into a tracked normative specification and implementation plan for direct structural type functions

## Status: ✅ Complete

## Description

Promote DESIGN-034 SPEC-E into a tracked normative specification and implementation plan for direct structural type functions.

## Specification Reference

- [SPEC-061](../../spec/SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [PLAN-109](../PLAN-109-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.5
- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)

## Dependencies

- N/A for planning packet.

## Dispatch

```
agent: hermes
provider: openai-codex
model: gpt-5.5
profile: default
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Objective

Promote DESIGN-034 SPEC-E into a tracked normative specification and implementation plan for direct structural type functions.

## Requirements

1. Create SPEC-061 as the normative SPEC-E owner.
2. Create PLAN-109 as the Phase 113 implementation plan.
3. Create TASK-830 through TASK-842 task files.
4. Register SPEC-061 in docs/spec/README.md.
5. Register Phase 113 in docs/plan/PLAN-INDEX.md.
6. Update CHANGELOG.md.
7. Keep Rust implementation tasks planned.

## Files

- Modify/create exact files identified by [PLAN-109](../PLAN-109-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md) and the TASK-831 audit gate.
- Update `CHANGELOG.md` for completed implementation/tooling/docs-policy changes.

## TDD Steps

1. Write focused failing tests or docs/audit checks appropriate to task type.
2. Run the focused target and verify the expected failure or missing evidence.
3. Implement the minimal change for this task only.
4. Re-run the focused target and relevant non-regression tests.
5. Update docs/status evidence only after verification.

## Verification

```
strictness: clean
commands:
  - git diff --check
  - test -f docs/spec/SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md
  - test -f docs/plan/PLAN-109-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md
  - python - <<'PY'
    from pathlib import Path
    required = [Path('docs/spec/SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md'), Path('docs/plan/PLAN-109-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md')] + sorted(Path('docs/plan/tasks').glob('TASK-83*.md')) + sorted(Path('docs/plan/tasks').glob('TASK-84*.md'))
    assert all(p.exists() for p in required), required
    assert 'SPEC-061' in Path('docs/spec/README.md').read_text()
    assert 'PLAN-109' in Path('docs/plan/PLAN-INDEX.md').read_text()
    PY
checklist:
  - [x] Create SPEC-061 as the normative SPEC-E owner.
  - [x] Create PLAN-109 as the Phase 113 implementation plan.
  - [x] Create TASK-830 through TASK-842 task files.
  - [x] Register SPEC-061 in docs/spec/README.md.
  - [x] Register Phase 113 in docs/plan/PLAN-INDEX.md.
  - [x] Update CHANGELOG.md.
  - [x] Keep Rust implementation tasks planned.
  - [x] focused tests/evidence recorded in this task file
  - [x] packet file-existence and registration evidence recorded
  - [x] no SPEC-F/G/H scope creep
```


## Notes

Task type: Docs/Planning. Estimated effort: 4 hours. Keep the slice within SPEC-061 / Phase 113 boundaries.
