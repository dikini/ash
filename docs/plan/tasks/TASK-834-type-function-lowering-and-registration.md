# TASK-834: Lower source declarations and register module-local type-function heads in TypeEnv

## Status: ✅ Complete

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
  - [x] Lower raw surface type functions into core carriers.
  - [x] Predeclare/provisionally allocate the current local `TypeComputationHeadId` so recursive self-reference can resolve while invalid heads remain unpublished.
  - [x] Publish validated heads only in source order; reject later same-module forward references in SPEC-E.
  - [x] Preserve equation order, RHS pattern-variable substitution metadata, marker-constructor RHS carriers, and source anchors.
  - [x] Ensure type-function applications lower to computation-head carriers and marker-constructor RHS applications lower to domain-constructor result carriers rather than nominal constructors.
  - [x] Do not export source equations or public summaries before SPEC-F.
  - [x] Add registration tests proving self-reference, earlier validated dependencies, duplicate-name rejection, invalid-publication rejection, and later-forward-reference rejection.
  - [x] focused tests/evidence recorded in this task file
  - [x] no SPEC-F/G/H scope creep
```

## Verification Evidence

- Added focused TDD tests in `crates/ash-typeck/tests/task_834_type_function_lowering.rs`; initial focused run failed because `TypeEnv::register_local_type_functions` and lookup APIs were absent.
- Implemented module-local lowering/registration in `crates/ash-typeck/src/type_env.rs` with staged all-or-nothing batch registration, provisional current self-head resolution, source-order publication, duplicate and later-forward-reference rejection, checked carrier storage, marker-constructor RHS/domain-constructor carriers, computation-head RHS carriers, pattern-variable metadata, and source anchors.
- Verified focused target:
- Focused pass after implementation: `cargo test -p ash-typeck --test task_834_type_function_lowering -- --nocapture` — 6 passed, 0 failed.
- Workspace compile after TypeEnv registration API additions: `cargo check --workspace` — passed.
- Formatting: `cargo fmt --check` — passed.


## Notes

Task type: Type/Substrate. Estimated effort: 6 hours. Keep the slice within SPEC-061 / Phase 113 boundaries.
