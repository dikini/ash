# TASK-821: Closed Computation-Head Reduction

## Status: ✅ Complete

## Description

Implement closed fixture reduction from computation-head applications to domain-constructor normal forms.

## Specification Reference

- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [PLAN-108](../PLAN-108-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.4
- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [SPEC-059](../../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)

## Dependencies

- ✅ [TASK-816](TASK-816-spec-d-spec-plan-packet.md)
- ✅ [TASK-820](TASK-820-internal-fixture-equation-registry.md) (completed predecessor)

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

Implement closed fixture reduction from computation-head applications to domain-constructor normal forms.

## Requirements

1. Write RED tests for Append<Nil, Ys> and Append<Cons<A, Nil>, Cons<B, Nil>>.
2. Implement weak-head reduction for a selected fixture equation.
3. Implement full recursive normalization for closed fixture applications.
4. Keep reductions keyed by TypeComputationHeadId and DomainConstructorId.
5. Classify malformed fixture shapes as implementation errors, not user syntax errors.

## Files

- Modify: `crates/ash-typeck/src/normalizer.rs`
- Test: `crates/ash-typeck/tests/task_821_closed_computation_head_reduction.rs`

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
  - cargo test -p ash-typeck --test task_821_closed_computation_head_reduction
  - cargo test -p ash-typeck task_802
  - cargo fmt --check
checklist:
  - [x] Closed Append reductions pass (`cargo test -p ash-typeck --test task_821_closed_computation_head_reduction`)
  - [x] Reduction output uses domain constructor normal forms (`cargo test -p ash-typeck --test task_821_closed_computation_head_reduction`)
  - [x] Existing TypeEnv equality tests still pass (`cargo test -p ash-typeck --test task_826_typeenv_forcing_point_rollout`)
```

## Notes

Task type: Type/Semantic. Estimated effort: 6 hours. Keep the slice compilable and do not widen beyond SPEC-060 scope.
