# TASK-826: TypeEnv Forcing-Point Rollout

## Status: ✅ Complete

## Description

Adopt the definitional equality API at the named TypeEnv forcing points only.

## Specification Reference

- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [PLAN-108](../PLAN-108-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.4
- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [SPEC-059](../../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)

## Dependencies

- ✅ [TASK-816](TASK-816-spec-d-spec-plan-packet.md)
- ✅ [TASK-825](TASK-825-non-inverting-unification-boundary.md)
- ✅ [TASK-817](TASK-817-normalizer-defeq-audit-gate.md) forcing-point matrix

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

Adopt the definitional equality API at the named TypeEnv forcing points only.

## Requirements

Completion evidence: consumed TASK-817 FP-1/FP-2/FP-6/FP-7/FP-17 and left FP-10/FP-11/FP-12/FP-13/FP-15/FP-16 deferred/fallback.

1. Consume the exact forcing-point matrix from TASK-817 and list each touched function/callsite in this task before implementation.
2. Route TypeEnv equality wrappers through definitional equality when both sides canonicalize safely.
3. Adopt expected-vs-actual expression/return comparison only for callsites explicitly marked owned in the TASK-817 matrix; constructor-field/pattern/exhaustiveness callsites are deferred unless the matrix says otherwise.
4. Adopt impl-overlap/coherence normalization only for compatible canonical heads.
5. Normalize associated projection argument spines while preserving their Phase 110 `ProjectionRigidity` (`Neutral` or `Rigid`).
6. Use final inferred-type rendering only for exact callsites named by TASK-817, starting with `TypeEnv::render_type_for_diagnostics` if selected; direct `to_string()` diagnostics remain deferred unless named.
7. Preserve fallback behavior for legacy shapes outside canonical IR support and document non-owned callsites as deferred/fallback.

## Files

- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `crates/ash-typeck/src/check_expr.rs` only for exact callsites owned by the TASK-817 matrix
- Test: `crates/ash-typeck/tests/task_826_typeenv_forcing_point_rollout.rs`

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
  - cargo test -p ash-typeck --test task_826_typeenv_forcing_point_rollout
  - cargo test -p ash-typeck
  - cargo fmt --check
checklist:
  - [x] Named forcing-point tests pass and cite the TASK-817 matrix
  - [x] Ordinary constructor unification remains stable
  - [x] No new pattern/exhaustiveness/constructor-field forcing-point adoption occurs unless explicitly authorized by the matrix
```

## Notes

Task type: Type/Integration. Estimated effort: 7 hours. Keep the slice compilable and do not widen beyond SPEC-060 scope.


## Completion Notes

Completed in Phase 112 implementation. `TypeEnv::unify_types` and `types_equivalent_for_equality` now use guarded normalizer/definitional equality only when both `Type` values safely lower to canonical IR. The rollout covers associated projection canonical identity aliases, transparent alias-compatible impl overlap, impl method return checking, and projection-spine normalization while preserving legacy fallback for inference metas and deferred legacy shapes.
