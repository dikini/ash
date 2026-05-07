# TASK-831: Audit live parser/core/typeck/normalizer/engine seams before implementation begins

## Status: ✅ Complete

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
  - [x] Produce an audit artifact under `docs/plan/audits/`.
  - [x] Name exact parser dispatch functions and AST carriers to change.
  - [x] Name exact core/type_ir and semantic_summary carriers to extend.
  - [x] Name exact TypeEnv and normalizer integration seams.
  - [x] Name public/import boundary seams in ash-engine, including public ordinary export leakage before SPEC-F.
  - [x] Name source type-expression resolution seams and ambiguity checks.
  - [x] No Rust implementation changes.
  - [x] focused tests/evidence recorded in this task file
  - [x] no SPEC-F/G/H scope creep
```

## Evidence

- Audit artifact: [`docs/plan/audits/TASK-831-type-function-audit.md`](../audits/TASK-831-type-function-audit.md).
- Live code inspected and mapped: `ash-parser` surface/module/type parser seams, `ash-core` canonical/normal/semantic-summary carriers, `ash-typeck` TypeEnv/normalizer/error seams, and `ash-engine` module summary/import/export seams.
- Verification commands:
  - `test -f docs/plan/audits/TASK-831-type-function-audit.md`
  - `git diff --check`
- Scope evidence: no Rust implementation changes; audit explicitly keeps public type-function summary export/import, associated recursive type-family computation, and SPEC-G/H behavior out of scope.


## Notes

Task type: Docs/Substrate. Estimated effort: 5 hours. Keep the slice within SPEC-061 / Phase 113 boundaries.
