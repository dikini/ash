# TASK-839: Enforce module-local engine/import boundary and non-interference with existing semantic summaries

## Status: 📋 Planned

## Description

Enforce module-local engine/import boundary and non-interference with existing semantic summaries.

## Specification Reference

- [SPEC-061](../../spec/SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [PLAN-109](../PLAN-109-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.5
- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)

## Dependencies

- ✅ Phase 112 / SPEC-060 complete.
- Depends on TASK-831 audit gate completion.
- Depends on TASK-838 source-equation normalizer integration completion.

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

Enforce module-local engine/import boundary and non-interference with existing semantic summaries.

## Requirements

1. Verify ModuleFile/engine integration preserves local type-function definitions for same-module checking.
2. Reject or fence imported/public type-function normalization before SPEC-F.
3. Reject public ordinary aliases/signatures/interface surfaces that leak local computation heads before SPEC-F.
4. Prove ordinary type, sealed-domain, workflow, and normalizer fixture summaries remain non-regressed.

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
  - cargo test -p ash-engine --test task_839_type_function_module_boundary -- --nocapture
  - cargo fmt --check
  - git diff --check
checklist:
  - [ ] Verify ModuleFile/engine integration preserves local type-function definitions for same-module checking.
  - [ ] Reject or fence imported/public type-function normalization before SPEC-F.
  - [ ] Reject public ordinary aliases/signatures/interface surfaces that leak local computation heads before SPEC-F.
  - [ ] Prove ordinary type, sealed-domain, workflow, and normalizer fixture summaries remain non-regressed.
  - [ ] focused tests/evidence recorded in this task file
  - [ ] no SPEC-F/G/H scope creep
```


## Notes

Task type: Engine/Integration. Estimated effort: 5 hours. Keep the slice within SPEC-061 / Phase 113 boundaries.
