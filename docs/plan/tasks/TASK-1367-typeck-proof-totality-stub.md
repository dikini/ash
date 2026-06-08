# TASK-1367: Typechecker — proof body totality check (Stage 3 prep)

## Status: ✅ Complete

## Description

Add infrastructure for totality checking. Stage 3 will fill in actual checks.

## Requirements

1. Add `check_proof_totality` stub to `TypeEnv`
2. Stub accepts all proof bodies (no actual checking yet)
3. Leave TODO comment for Stage 3 implementation

## Acceptance Criteria

- [x] Stub exists and compiles
- [x] All proof bodies accepted by stub
- [x] Typechecker test passes
- [x] No regressions

## Verification

- `cargo test -p ash-typeck --test task_1367_proof_totality_stub -- --nocapture` — 3 passed
- `cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings` — passed
- `cargo check --workspace` — passed
- `git diff --check` — passed

## Completion Notes

- Added public `TypeEnv::check_proof_totality` Stage 3 preparation hook.
- The hook intentionally accepts all proof bodies for Phase 136 and carries a TODO for real proof totality/termination checking.
- Module-scoped and impl-scoped proof registration now calls the hook after proof-name matching.
- This task does not implement actual totality, termination, partial-match, circular-proof, runner, CLI/cache, or `Kind::Prop` semantics.

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1365](TASK-1365-typeck-proof-name-checking.md)
