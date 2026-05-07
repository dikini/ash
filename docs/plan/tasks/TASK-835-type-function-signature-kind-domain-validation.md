# TASK-835: Validate type-function signatures, kinds, domains, arity, and module-local public boundary

## Status: ✅ Complete

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
  - [x] Validate parameter and return type expressions for kind and domain/constraint conformance.
  - [x] Resolve sealed-domain parameter positions and reject definitions with no sealed-domain scrutinee.
  - [x] Build RHS pattern-variable environments and reject unknown RHS variables.
  - [x] Ensure lowercase pattern variables do not lower as nominal type names.
  - [x] Reject wrong arity, unknown heads, unknown constructors, wrong domains, result-kind mismatch, and result-domain mismatch.
  - [x] Reject ambiguous nominal/type-function heads and marker-constructor-vs-nominal/type-function heads.
  - [x] Enforce source-order same-module dependencies: earlier validated type functions are usable, later forward references are rejected.
  - [x] Reject pub/cross-module type-function use before SPEC-F at the typechecker boundary.
  - [x] focused tests/evidence recorded in this task file
  - [x] no SPEC-F/G/H scope creep
```

## Completion Evidence

- Added focused TDD tests in `crates/ash-typeck/tests/task_835_type_function_validation.rs`; initial implementation attempt exposed missing validation coverage in `TypeEnv::register_local_type_functions`.
- Implemented TASK-835 validation in `crates/ash-typeck/src/type_env.rs`: signature type/domain resolution, no-sealed-scrutinee rejection, pattern variable environments, lowercase pattern-variable precedence over nominal types, wrong arity/unknown head/unknown constructor/wrong domain/result-kind/result-domain rejection, ambiguous type-function-vs-nominal and marker-vs-type-head rejection, source-order dependency enforcement, and `pub type fn` rejection before SPEC-F.
- Focused pass after implementation and targeted review remediation: `cargo test -p ash-typeck --test task_835_type_function_validation -- --nocapture` — 19 passed, 0 failed.
- TASK-834 non-regression: `cargo test -p ash-typeck --test task_834_type_function_lowering -- --nocapture` — 6 passed, 0 failed.
- Workspace compile after validation additions: `cargo check --workspace` — passed.
- Formatting: `cargo fmt --check` — passed.
- Whitespace: `git diff --check` — passed.
- Targeted review remediation added regression coverage for marker-constructor ambiguity in pattern position, wrong-domain marker constructors in RHS position, and current provisional type-function head vs marker-constructor ambiguity.
- Scope note: TASK-835 did not implement residual coverage/overlap, structural recursion, normalizer reduction, engine export/import, or SPEC-F/G/H public/cross-module semantics.

## Notes

Task type: Type/Semantic. Estimated effort: 6 hours. Keep the slice within SPEC-061 / Phase 113 boundaries.
