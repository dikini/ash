# TASK-837: Implement declared decreasing-parameter and structural recursion validation

## Status: ✅ Complete

## Description

Implement declared decreasing-parameter and structural recursion validation.

## Specification Reference

- [SPEC-061](../../spec/SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [PLAN-109](../PLAN-109-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.5
- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)

## Dependencies

- ✅ Phase 112 / SPEC-060 complete.
- Depends on TASK-831 audit gate completion.
- Depends on TASK-836 pattern coverage/overlap completion.

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

Implement declared decreasing-parameter and structural recursion validation.

## Requirements

1. Require `decreases` for recursive definitions.
2. Require the decreasing parameter to be a sealed-domain parameter with structural subcomponent metadata.
3. Consume provisional self-head resolution from TASK-834 without publishing invalid recursive heads.
4. Detect recursive calls by recursively walking all canonical RHS children, including nominal apps, domain constructors, projections, and nested computation-head apps.
5. Accept direct structural subcomponent recursion.
6. Reject same, rebuilt, alias/computed, and mutually recursive arguments.

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
  - cargo test -p ash-typeck --test task_837_type_function_recursion -- --nocapture
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Require `decreases` for recursive definitions.
  - [x] Require the decreasing parameter to be a sealed-domain parameter with structural subcomponent metadata.
  - [x] Consume provisional self-head resolution from TASK-834 without publishing invalid recursive heads.
  - [x] Detect recursive calls by recursively walking all canonical RHS children, including nominal apps, domain constructors, projections, and nested computation-head apps.
  - [x] Accept direct structural subcomponent recursion.
  - [x] Reject same, rebuilt, alias/computed, and mutually recursive arguments.
  - [x] focused tests/evidence recorded in this task file
  - [x] no SPEC-F/G/H scope creep
```

## Evidence

- Added focused TDD coverage in `crates/ash-typeck/tests/task_837_type_function_recursion.rs` for missing/invalid `decreases`, non-structural domains, accepted Append-style tail recursion, same/rebuilt/computed recursive argument rejection, nested self-call detection, source-order mutual-recursion rejection, and invalid recursive head non-publication.
- Verified expected pre-implementation failures with `cargo test -p ash-typeck --test task_837_type_function_recursion -- --nocapture` (9 failed / 2 passed before implementation).
- Implemented structural recursion validation in `crates/ash-typeck/src/type_env.rs` after TASK-836 pattern coverage, using checked result-expression carriers and existing staged publication to avoid publishing invalid recursive heads.
- Verification commands run:
  - `cargo test -p ash-typeck --test task_837_type_function_recursion -- --nocapture` — 11 passed.
  - `cargo test -p ash-typeck --test task_836_type_function_patterns -- --nocapture` — 10 passed.

## Notes

Task type: Type/Semantic. Estimated effort: 6 hours. Keep the slice within SPEC-061 / Phase 113 boundaries.
