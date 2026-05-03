# TASK-796: Core Canonical Type-Expression IR and Neutral Carriers

## Status: ✅ Complete

## Description

Add the shared canonical type-expression IR in `ash-core`, including distinct computation-head application and rigid/neutral carrier shapes, without implementing normalization.

## Specification Reference

- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [PLAN-106](../PLAN-106-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)

## Dependencies

- ✅ [TASK-795](TASK-795-core-type-computation-identity-carriers.md)

## Objective

Define the shared data model that later packets will consume.

## Requirements

1. Add a canonical IR that distinguishes nominal application, projection, and computation-head application, using the core-owned shared `Kind` from TASK-795.
2. Add rigid/neutral carriers and a normal-form view shape suitable for later packets.
3. Do not add normalization or equality semantics yet.
4. Keep the IR sharable across crates through `ash-core`.

## Files

- Create or modify: `crates/ash-core/src/type_ir.rs`
- Modify: `crates/ash-core/src/lib.rs`
- Add focused `ash-core` tests for the new IR carriers

## TDD Steps

1. Write failing tests proving computation-head apps are distinct from nominal constructors.
2. Write failing tests for rigid/neutral carrier construction and debug/serde behavior.
3. Implement the minimal IR to pass those tests.
4. Re-run focused `ash-core` tests.

## Verification Steps

- [x] `cargo test -p ash-core` for the new type IR tests
- [x] `cargo fmt --check`
- [x] `git diff --check`

## Notes

Single-crate task. Do not change parser or typechecker behavior yet. This task assumes TASK-795 already resolved shared `Kind` ownership.
