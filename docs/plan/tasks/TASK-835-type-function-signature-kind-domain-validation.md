# TASK-835: Validate type-function signatures, kinds, domains, arity, and module-local public boundary

## Status: 📋 Planned

## Description

Validate type-function signatures, kinds, domains, arity, and module-local public boundary.

## Specification Reference

- [SPEC-061](../../spec/SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [PLAN-109](../PLAN-109-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.5
- [SPEC-059](../../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)
- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)

## Dependencies

- ✅ Phase 111 / SPEC-059 sealed-domain APIs complete.
- ✅ Phase 112 / SPEC-060 complete.
- Depends on TASK-831 audit gate completion.
- Depends on TASK-834 lowering/registration completion.

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

Validate type-function signatures, kinds, domains, arity, and module-local public boundary.

## Requirements

1. Validate parameter and return type expressions for kind and domain/constraint conformance.
2. Resolve sealed-domain parameter positions and reject definitions with no sealed-domain scrutinee.
3. Build RHS pattern-variable environments and reject unknown RHS variables.
4. Ensure lowercase pattern variables do not lower as nominal type names.
5. Reject wrong arity, unknown heads, unknown constructors, wrong domains, result-kind mismatch, and result-domain mismatch.
6. Reject ambiguous nominal/type-function heads and marker-constructor-vs-nominal/type-function heads.
7. Enforce source-order same-module dependencies: earlier validated type functions are usable, later forward references are rejected.
8. Reject pub/cross-module type-function use before SPEC-F at the typechecker boundary.

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
  - cargo test -p ash-typeck --test task_835_type_function_validation -- --nocapture
  - cargo fmt --check
  - git diff --check
checklist:
  - [ ] Validate parameter and return type expressions for kind and domain/constraint conformance.
  - [ ] Resolve sealed-domain parameter positions and reject definitions with no sealed-domain scrutinee.
  - [ ] Build RHS pattern-variable environments and reject unknown RHS variables.
  - [ ] Ensure lowercase pattern variables do not lower as nominal type names.
  - [ ] Reject wrong arity, unknown heads, unknown constructors, wrong domains, result-kind mismatch, and result-domain mismatch.
  - [ ] Reject ambiguous nominal/type-function heads and marker-constructor-vs-nominal/type-function heads.
  - [ ] Enforce source-order same-module dependencies: earlier validated type functions are usable, later forward references are rejected.
  - [ ] Reject pub/cross-module type-function use before SPEC-F at the typechecker boundary.
  - [ ] focused tests/evidence recorded in this task file
  - [ ] no SPEC-F/G/H scope creep
```


## Notes

Task type: Type/Semantic. Estimated effort: 6 hours. Keep the slice within SPEC-061 / Phase 113 boundaries.
