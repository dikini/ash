# TASK-834: Lower source declarations and register module-local type-function heads in TypeEnv

## Status: 📋 Planned

## Description

Lower source declarations and register module-local type-function heads in TypeEnv.

## Specification Reference

- [SPEC-061](../../spec/SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [PLAN-109](../PLAN-109-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.5
- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)

## Dependencies

- ✅ Phase 112 / SPEC-060 complete.
- Depends on TASK-831 audit gate completion.
- Depends on TASK-832 parser surface completion.
- Depends on TASK-833 core carrier completion.

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

Lower source declarations and register module-local type-function heads in TypeEnv.

## Requirements

1. Lower raw surface type functions into core carriers.
2. Predeclare/provisionally allocate the current local `TypeComputationHeadId` so recursive self-reference can resolve while invalid heads remain unpublished.
3. Publish validated heads only in source order; reject later same-module forward references in SPEC-E.
4. Preserve equation order, RHS pattern-variable substitution metadata, marker-constructor RHS carriers, and source anchors.
5. Ensure type-function applications lower to computation-head carriers and marker-constructor RHS applications lower to domain-constructor result carriers rather than nominal constructors.
6. Do not export source equations or public summaries before SPEC-F.
7. Add registration tests proving self-reference, earlier validated dependencies, duplicate-name rejection, invalid-publication rejection, and later-forward-reference rejection.

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
  - cargo test -p ash-typeck --test task_834_type_function_lowering -- --nocapture
  - cargo fmt --check
  - git diff --check
checklist:
  - [ ] Lower raw surface type functions into core carriers.
  - [ ] Predeclare/provisionally allocate the current local `TypeComputationHeadId` so recursive self-reference can resolve while invalid heads remain unpublished.
  - [ ] Publish validated heads only in source order; reject later same-module forward references in SPEC-E.
  - [ ] Preserve equation order, RHS pattern-variable substitution metadata, marker-constructor RHS carriers, and source anchors.
  - [ ] Ensure type-function applications lower to computation-head carriers and marker-constructor RHS applications lower to domain-constructor result carriers rather than nominal constructors.
  - [ ] Do not export source equations or public summaries before SPEC-F.
  - [ ] Add registration tests proving self-reference, earlier validated dependencies, duplicate-name rejection, invalid-publication rejection, and later-forward-reference rejection.
  - [ ] focused tests/evidence recorded in this task file
  - [ ] no SPEC-F/G/H scope creep
```


## Notes

Task type: Type/Substrate. Estimated effort: 6 hours. Keep the slice within SPEC-061 / Phase 113 boundaries.
