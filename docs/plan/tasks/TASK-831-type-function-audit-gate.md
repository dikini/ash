# TASK-831: Audit live parser/core/typeck/normalizer/engine seams before implementation begins

## Status: 📋 Planned

## Description

Audit live parser/core/typeck/normalizer/engine seams before implementation begins.

## Specification Reference

- [SPEC-061](../../spec/SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [PLAN-109](../PLAN-109-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.5
- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)

## Dependencies

- ✅ Phase 112 / SPEC-060 complete.

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

Audit live parser/core/typeck/normalizer/engine seams before implementation begins.

## Requirements

1. Produce an audit artifact under `docs/plan/audits/`.
2. Name exact parser dispatch functions and AST carriers to change.
3. Name exact core/type_ir and semantic_summary carriers to extend.
4. Name exact TypeEnv and normalizer integration seams.
5. Name public/import boundary seams in ash-engine, including public ordinary export leakage before SPEC-F.
6. Name source type-expression resolution seams and ambiguity checks.
7. No Rust implementation changes.

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
  - test -f docs/plan/audits/TASK-831-type-function-audit.md
checklist:
  - [ ] Produce an audit artifact under `docs/plan/audits/`.
  - [ ] Name exact parser dispatch functions and AST carriers to change.
  - [ ] Name exact core/type_ir and semantic_summary carriers to extend.
  - [ ] Name exact TypeEnv and normalizer integration seams.
  - [ ] Name public/import boundary seams in ash-engine, including public ordinary export leakage before SPEC-F.
  - [ ] Name source type-expression resolution seams and ambiguity checks.
  - [ ] No Rust implementation changes.
  - [ ] focused tests/evidence recorded in this task file
  - [ ] no SPEC-F/G/H scope creep
```


## Notes

Task type: Docs/Substrate. Estimated effort: 5 hours. Keep the slice within SPEC-061 / Phase 113 boundaries.
