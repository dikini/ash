# TASK-823: Neutral/Rigid Projection and Alias Normalization

## Status: ✅ Complete

## Description

Normalize transparent aliases plus neutral and rigid projection argument spines without adding associated-family computation.

## Specification Reference

- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [PLAN-108](../PLAN-108-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.4
- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [SPEC-059](../../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)

## Dependencies

- ✅ [TASK-816](TASK-816-spec-d-spec-plan-packet.md)
- 📝 [TASK-822](TASK-822-open-neutral-and-partial-normalization.md) (planned predecessor)

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

Normalize transparent aliases plus neutral and rigid projection argument spines without adding associated-family computation.

## Requirements

1. Write tests for transparent alias expansion before normal-form comparison.
2. Write tests for both `ProjectionRigidity::Neutral` and `ProjectionRigidity::Rigid` projection argument normalization.
3. Preserve ProjectionRigidity semantics from Phase 110.
4. Do not invoke recursive impl/family search for associated outputs.
5. Ensure neutral projections are treated as blockers/structural forms rather than converted into rigid projections.
6. Keep diagnostics preserving readable source spelling where possible.

## Files

- Modify: `crates/ash-typeck/src/normalizer.rs`
- Modify: `crates/ash-typeck/src/type_env.rs` only as needed for alias/projection helpers
- Test: `crates/ash-typeck/tests/task_823_rigid_projection_alias_normalization.rs`

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
  - cargo test -p ash-typeck --test task_823_rigid_projection_alias_normalization
  - cargo test -p ash-typeck --test task_800_associated_projection_canonicalization_red
  - cargo fmt --check
checklist:
  - [ ] Alias normalization tests pass
  - [ ] Neutral and rigid projection structural tests pass
  - [ ] No associated-family computation path is added
```

## Notes

Task type: Type/Semantic. Estimated effort: 5 hours. Keep the slice compilable and do not widen beyond SPEC-060 scope.

## Completion Notes

Implemented in the Phase 112 normalizer with focused tests in `crates/ash-typeck/tests/task_823_rigid_projection_alias_normalization.rs`. The normalizer now peels registered transparent aliases only at normalizer inputs via a narrow `TypeEnv` helper, recursively normalizes rigid and neutral projection argument spines (including nested reducible fixture computations), preserves `ProjectionRigidity` and blocker reasons, and intentionally leaves associated projections as structural normal forms without recursive associated-family reduction or new equality forcing-point adoption.
