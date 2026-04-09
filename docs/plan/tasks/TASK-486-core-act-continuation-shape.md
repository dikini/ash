# TASK-486: Update Core `Workflow::Act` with Result Binding and Continuation

## Status: Planned

## Description

Extend the core `Workflow::Act` variant in `ash-core/src/ast.rs` with `result_name: Option<Name>`
and `continuation: Box<Workflow>`. Migrate every `Workflow::Act` construction site across the
workspace so that bare `act` nodes use `result_name: None, continuation: Done` (semantically
identical to the current terminal behavior).

## Specification Reference

- [DESIGN-019](../../design/DESIGN-019-ACTION-RESULT-BINDING.md)
- [PLAN-019](../PLAN-019-ACTION-RESULT-BINDING.md)
- [SPEC-001](../../spec/SPEC-001-IR.md) — core Workflow contract

## Dependencies

- None (this is the foundational change)

## Requirements

1. Add `result_name: Option<Name>` field to `Workflow::Act` in `crates/ash-core/src/ast.rs`.
2. Add `continuation: Box<Workflow>` field to `Workflow::Act`.
3. Audit and migrate every `Workflow::Act { ... }` construction site in:
   - `ash-core` (tests, visualize, provenance, test_helpers, etc.)
   - `ash-parser` (lowering)
   - `ash-interp` (execute, tests)
   - `ash-engine` (any test or harness code)
   - `ash-cli` (any test or example code)
4. Every migrated site uses `result_name: None, continuation: Box::new(Workflow::Done)` unless
   it already has continuation semantics.
5. Update `PartialEq`, `Clone`, `Debug` derived behavior if field additions affect it.
6. `cargo check --workspace` passes after migration.
7. `cargo test -p ash-core -p ash-parser -p ash-interp -p ash-cli` passes.
   Note: 5 pre-existing ash-engine failures are known residuals unrelated to this change and
   are not in scope. Verify `cargo test -p ash-engine` does not introduce new failures.

## TDD Steps

### Red

- Add the two new fields to `Workflow::Act` but do not migrate construction sites. Observe compile
  errors at every site (confirms full audit).

### Green

- Migrate every construction site to include the new fields with defaults.
- Verify `cargo check --workspace` and `cargo test -p ash-core -p ash-parser -p ash-interp -p ash-cli` pass.

### Refactor

- Consider adding a helper `Workflow::act_terminal(provider, action, args, guard, provenance)`
  constructor to reduce boilerplate at sites that don't need continuation.

## Completion Checklist

- [ ] `Workflow::Act` has `result_name` and `continuation` fields in `ast.rs`
- [ ] All construction sites migrated (verified by `rg 'Workflow::Act'`)
- [ ] `cargo check --workspace` passes
- [ ] `cargo test -p ash-core -p ash-parser -p ash-interp -p ash-cli` passes
- [ ] `cargo test -p ash-engine` introduces no new failures (5 pre-existing residuals allowed)
- [ ] `cargo fmt --check` clean
- [ ] `cargo clippy --workspace --all-targets` clean
- [ ] CHANGELOG.md entry added
