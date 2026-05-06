# TASK-824: Definitional Equality API

## Status: ✅ Complete

## Description

Add a structured normalize-and-compare definitional equality API over normal forms.

## Specification Reference

- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [PLAN-108](../PLAN-108-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.4
- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [SPEC-059](../../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)

## Dependencies

- ✅ [TASK-816](TASK-816-spec-d-spec-plan-packet.md)
- ✅ [TASK-823](TASK-823-rigid-projection-and-alias-normalization.md)

## Dispatch

```
agent: hermes
provider: openai-codex
model: gpt-5.5
profile: default
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Objective

Add a structured normalize-and-compare definitional equality API over normal forms.

## Requirements

1. Define equality result variants: Equal, NotEqual, BlockedByNeutrality.
2. Compare normal forms structurally by canonical identity and normalized argument spine.
3. Add boolean convenience only on top of structured evidence.
4. Add tests for closed equality after reduction, neutral computation structural equality, and neutral/rigid projection structural equality.
5. Add tests for normalized mismatch evidence.

## Files

- Modify: `crates/ash-typeck/src/normalizer.rs`
- Test: `crates/ash-typeck/tests/task_824_definitional_equality.rs`

## TDD Steps

1. Write focused failing tests for the task-owned behavior.
2. Run the focused test and confirm it fails for the expected reason.
3. Implement the smallest compiling change that passes the focused test.
4. Re-run focused tests and nearby regression suites.
5. Run formatting and the verification commands below.

## Verification

```
strictness: clean
commands:
  - cargo test -p ash-typeck --test task_824_definitional_equality
  - cargo test -p ash-typeck --test task_822_open_neutral_partial_normalization
  - cargo fmt --check
checklist:
  - [x] Structured equality tests pass
  - [x] Boolean wrapper is derived from structured result
  - [x] Mismatch evidence contains normalized slices
  - [x] Neutral/rigid projections compare by identity, rigidity, and normalized arguments
```

## Notes

Task type: Type/Semantic. Estimated effort: 6 hours. Keep the slice compilable and do not widen beyond SPEC-060 scope.

## Completion Notes

- Implemented `DefinitionalEqualityResult::{Equal, NotEqual, BlockedByNeutrality}` and `Normalizer::definitional_equality(...)` as normalize-and-compare over canonical normal forms.
- Added `Normalizer::definitionally_equal(...)` as a boolean convenience wrapper derived from the structured result.
- Equality compares normal-form heads by canonical identities, kind/rigidity where applicable, and recursively normalized argument spines. It does not perform proof search, inversion, associated-family computation, or TypeEnv forcing-point rollout.
- Added focused `task_824_*` tests covering closed reduction equality, open neutral structural equality, neutral/rigid projection structural equality, normalized mismatch slices, blocker evidence, and fuel/error propagation.
