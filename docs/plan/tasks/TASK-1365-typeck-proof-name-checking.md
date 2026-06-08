# TASK-1365: Typechecker — verify proof names match declared laws

## Status: ✅ Complete

## Description

Compiler rejects `proof unknown_law(...) { ... }` if no matching law exists.

## Requirements

1. Add `register_module_proofs` to `TypeEnv`
2. Add `register_impl_proofs` to `TypeEnv`
3. Verify proof name matches a declared law in scope
4. Error if no matching law found

## Acceptance Criteria

- [x] Proof for unknown law produces error
- [x] Proof for known law passes
- [x] Typechecker test passes
- [x] No regressions

## Verification

- `cargo test -p ash-typeck --test task_1365_proof_name_checking -- --nocapture` — 6 passed
- `cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings` — passed
- `cargo check --workspace` — passed
- `git diff --check` — passed

## Completion Notes

- Added `TypeEnv::register_module_proofs` and `TypeEnv::register_impl_proofs`.
- Module-scope proofs must match a module-scope `law` with the same name.
- Impl-scoped proofs must match a law declared by the implemented interface.
- This task only verifies proof/law name matching; proof parameter compatibility, proof body typechecking/totality, Pure-only law restrictions, runner integration, and synthetic tests remain later Phase 136 tasks.

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1362](TASK-1362-parser-proof-in-impls.md)
- [TASK-1363](TASK-1363-parser-proof-module-scope.md)
- [TASK-1364](TASK-1364-typeck-law-name-checking.md)
