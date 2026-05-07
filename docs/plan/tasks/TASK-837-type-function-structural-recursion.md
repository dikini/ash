# TASK-837: Implement declared decreasing-parameter and structural recursion validation

## Status: 📋 Planned

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
  - [ ] Require `decreases` for recursive definitions.
  - [ ] Require the decreasing parameter to be a sealed-domain parameter with structural subcomponent metadata.
  - [ ] Consume provisional self-head resolution from TASK-834 without publishing invalid recursive heads.
  - [ ] Detect recursive calls by recursively walking all canonical RHS children, including nominal apps, domain constructors, projections, and nested computation-head apps.
  - [ ] Accept direct structural subcomponent recursion.
  - [ ] Reject same, rebuilt, alias/computed, and mutually recursive arguments.
  - [ ] focused tests/evidence recorded in this task file
  - [ ] no SPEC-F/G/H scope creep
```


## Notes

Task type: Type/Semantic. Estimated effort: 6 hours. Keep the slice within SPEC-061 / Phase 113 boundaries.
