# TASK-825: Non-Inverting Unification Boundary

## Status: ✅ Complete

## Description

Prove and enforce that equality/unification does not solve underneath neutral computation heads.

## Specification Reference

- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [PLAN-108](../PLAN-108-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.4
- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [SPEC-059](../../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)

## Dependencies

- ✅ [TASK-816](TASK-816-spec-d-spec-plan-packet.md)
- ✅ [TASK-824](TASK-824-definitional-equality-api.md)

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

Prove and enforce that equality/unification does not solve underneath neutral computation heads.

## Requirements

1. Add tests showing F<X> == F<Y> does not infer X = Y under neutral heads.
2. Add tests showing Append<Xs,Ys> == Cons<A,Nil> reports BlockedByNeutrality instead of solving inputs.
3. Preserve ordinary same-headed nominal constructor decomposition.
4. Explicitly distinguish canonical abstract variables (`CanonicalTypeExpr::Var(String)`) from current inference metas (`Type::Var(TypeVar)`) in tests/comments.
5. Ensure top-level meta behavior remains limited to existing supported unifier capabilities unless a concrete bridge is implemented in this task.
6. Document the exact boundary in code comments/tests.

## Files

- Modify: `crates/ash-typeck/src/normalizer.rs`
- Modify: `crates/ash-typeck/src/types.rs` only if unifier boundary helpers are needed
- Test: `crates/ash-typeck/tests/task_825_non_inverting_unification_boundary.rs`

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
  - cargo test -p ash-typeck --test task_825_non_inverting_unification_boundary
  - cargo test -p ash-typeck --test task_802_canonicalization_boundary_adoption_red
  - cargo fmt --check
checklist:
  - [ ] No-solving-under-neutral tests pass
  - [ ] Ordinary nominal unification regression tests pass
  - [ ] Neutral-blocked diagnostics mention no inversion
  - [ ] Canonical-var versus inference-meta behavior is explicitly covered
```

## Notes

Task type: Type/Semantic. Estimated effort: 5 hours. Keep the slice compilable and do not widen beyond SPEC-060 scope.
