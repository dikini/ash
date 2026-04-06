# TASK-422: Closed-World Interfaces Coherence and Method Resolution

## Status: 🟡 Ready

## Description

Implement the second code follow-on after TASK-421 by teaching the typechecker to enforce the
closed-world interfaces MVP.

This task should make the frozen MVP surface semantically meaningful:
- constrained generic parameters are checked
- impls are registered and resolved
- duplicate/conflicting/overlapping impls are rejected per the strict coherence rule
- canonical method calls `Interface::method(value)` are typechecked

## Specification Reference

- [TASK-415: Closed-World Interfaces MVP Spec Cut](TASK-415-closed-world-interfaces-mvp-spec-cut.md)
- [TASK-421: Closed-World Interfaces AST and Parser Substrate](TASK-421-closed-world-interfaces-ast-and-parser-substrate.md)
- [TYPES-002 V2 MVP Cut](../../ideas/type-system/TYPES-002-v2-mvp-cut.md)
- [SPEC-003: Type System](../../spec/SPEC-003-TYPE-SYSTEM.md)

## Dependencies

- ✅ TASK-415 complete
- 🟡 TASK-421 ready/completes parser substrate

## Requirements

### Functional Requirements

1. Register interface declarations and impl declarations in typechecking environments.
2. Enforce the strict coherence rule:
   - at most one impl per `(Interface, ConcreteNominalType)`
   - reject duplicate/conflicting/overlapping impls
3. Typecheck constrained generic parameters using the canonical `T: Interface` form.
4. Typecheck canonical method calls `Interface::method(value)` via impl resolution.
5. Emit good failure cases for:
   - missing impl
   - duplicate/conflicting impl
   - invalid bound
   - invalid interface method call
6. Preserve capability/interface separation and effect-conservative MVP scope.
7. Add tests covering the above.

### Non-Functional Requirements

1. Do not add associated types/effects.
2. Do not add dynamic dispatch/trait objects.
3. Do not add capability/interface unification.
4. Update `CHANGELOG.md`.

## Files

- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `crates/ash-typeck/src/types.rs`
- Modify: `crates/ash-typeck/src/check_expr.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Modify any related name/instantiation/solver files as needed
- Add/modify tests under `crates/ash-typeck/tests/`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write failing typechecker tests

Add tests for interface registration, impl coherence, constrained bounds, and canonical method-call resolution.

### Step 2: Implement environment/coherence support

Add the minimum semantic machinery needed for the closed-world interface MVP.

### Step 3: Implement bound/call checking

Teach the typechecker to validate `T: Interface` and `Interface::method(value)`.

### Step 4: Verify affected crate quality

Run at least:
- `cargo test -p ash-typeck`
- `cargo clippy -p ash-typeck --all-targets -- -D warnings`
- `cargo fmt --check`

## Completion Checklist

- [ ] interface/impl environments added
- [ ] strict coherence enforced
- [ ] constrained bounds typecheck
- [ ] canonical method calls resolve/typecheck
- [ ] tests added/updated
- [ ] `CHANGELOG.md` updated

## Notes

This task should stay inside the frozen MVP. Later expansion to associated items/effects or dynamic
dispatch must be separate work.
