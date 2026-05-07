# TASK-838: Register checked source equations with the SPEC-060 normalizer and definitional equality API

## Status: 📋 Planned

## Description

Register checked source equations with the SPEC-060 normalizer and definitional equality API.

## Specification Reference

- [SPEC-061](../../spec/SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [PLAN-109](../PLAN-109-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.5
- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)

## Dependencies

- ✅ Phase 112 / SPEC-060 complete.
- Depends on TASK-831 audit gate completion.
- Depends on TASK-837 structural recursion validation completion.

## Dispatch

```
agent: hermes
provider: openai-codex
model: gpt-5.5
profile: default
reasoning: medium
max_turns: 20
toolsets: [terminal, file]
```

## Objective

Register checked source equations with the SPEC-060 normalizer and definitional equality API.

## Requirements

1. Convert validated equations into source-backed normalizer tables.
2. Substitute bound pattern variables such as `h`, `t`, and `ys` into RHS/result expressions during reduction.
3. Preserve SPEC-060 fixture semantics for known-scrutinee/open/partial reduction.
4. Test Append known-scrutinee reduction and abstract neutrality from source declarations.
5. Keep fixtures as test/internal setup, not production source representation.

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
  - cargo test -p ash-typeck --test task_838_type_function_normalizer -- --nocapture
  - cargo fmt --check
  - git diff --check
checklist:
  - [ ] Convert validated equations into source-backed normalizer tables.
  - [ ] Substitute bound pattern variables such as `h`, `t`, and `ys` into RHS/result expressions during reduction.
  - [ ] Preserve SPEC-060 fixture semantics for known-scrutinee/open/partial reduction.
  - [ ] Test Append known-scrutinee reduction and abstract neutrality from source declarations.
  - [ ] Keep fixtures as test/internal setup, not production source representation.
  - [ ] focused tests/evidence recorded in this task file
  - [ ] no SPEC-F/G/H scope creep
```


## Notes

Task type: Type/Integration. Estimated effort: 7 hours. Keep the slice within SPEC-061 / Phase 113 boundaries.
