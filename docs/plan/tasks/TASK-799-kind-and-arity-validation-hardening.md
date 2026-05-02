# TASK-799: Kind and Arity Validation Hardening

## Status: 📝 Planned

## Description

Harden kind/arity validation so nominal constructors, canonical projections, and future computation-head placeholders are checked explicitly and early.

## Specification Reference

- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [PLAN-106](../PLAN-106-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)

## Dependencies

- ✅ [TASK-798](TASK-798-canonical-type-ir-lowering-from-surface-and-core.md)

## Objective

Turn kind/arity checking into a first-class gate for the new canonical lowering path.

## Requirements

1. Reuse the existing `Kind` model instead of inventing a second kind system.
2. Validate nominal constructor arity/kind through the canonical path.
3. Validate canonical projection argument spines against the registered interface/member identities and the shared core-owned `Kind`.
4. Reject wrong kind/arity before later packets could consume the type expression.
5. Do not add public kind binder syntax, holes, or partial applications.

## Files

- Modify: `crates/ash-typeck/src/kind.rs`
- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Add focused `ash-typeck` tests for kind/arity diagnostics

## TDD Steps

1. Write failing tests for wrong nominal arity, wrong imported arity, and projection argument shape failures.
2. Implement the minimal validation changes to pass those tests.
3. Re-run focused kind/arity tests.
4. Confirm current accepted ordinary type cases still pass.

## Verification Steps

- [ ] `cargo test -p ash-typeck` for the new kind/arity tests
- [ ] `cargo fmt --check`
- [ ] `git diff --check`

## Notes

Single-crate task. No parser or engine work belongs here. This task closes the projection-spine validation precondition that TASK-800 relies on.
