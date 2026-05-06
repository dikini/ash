# TASK-822: Open Neutral and Partial Normalization

## Status: ✅ Complete

## Description

Implement canonical neutral/stuck normal forms for open applications and partial prefix normalization.

## Specification Reference

- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [PLAN-108](../PLAN-108-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.4
- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [SPEC-059](../../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)

## Dependencies

- ✅ [TASK-816](TASK-816-spec-d-spec-plan-packet.md)
- ✅ [TASK-821](TASK-821-closed-computation-head-reduction.md) (planned predecessor)

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

Implement canonical neutral/stuck normal forms for open applications and partial prefix normalization.

## Requirements

1. Write tests for Append<Xs,Ys> staying neutral.
2. Write tests for Append<Cons<A,Xs>,Ys> reducing the Cons prefix and preserving a neutral tail.
3. Record neutral reasons such as abstract scrutinee, neutral projection blocker, or rigid projection blocker.
4. Normalize arguments inside neutral apps for structural comparison.
5. Ensure open catch-all semantics are not introduced.

## Files

- Modify: `crates/ash-typeck/src/normalizer.rs`
- Test: `crates/ash-typeck/tests/task_822_open_neutral_normalization.rs`

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
  - cargo test -p ash-typeck --test task_822_open_neutral_normalization
  - cargo test -p ash-typeck --test task_821_closed_computation_head_reduction
  - cargo fmt --check
checklist:
  - [x] Open neutral tests pass
  - [x] Partial prefix tests pass
  - [x] Neutral forms include non-inverting stuck reasons
```

## Completion Notes

- Added focused `task_822_open_neutral_partial_normalization` tests for open `Append<Xs, Ys>` neutrality, nested open tails under `Cons`, normalized argument spines containing rigid projections, open `Append<Nil, Ys>` partial prefix reduction, and rejection of an open catch-all equation.
- Implemented open-aware fixture matching in the normalizer so constructor-pattern equations can reduce known prefixes while abstract variables/projections/neutral blockers preserve canonical neutral computation apps with normalized argument spines.
- Preserved the no-inversion boundary by refusing to bind top-level open neutral arguments through catch-all variable-pattern equations.

## Notes

Task type: Type/Semantic. Estimated effort: 6 hours. Keep the slice compilable and do not widen beyond SPEC-060 scope.
