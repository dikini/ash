# TASK-1375c: Circular Proof Detection

## Status: ✅ Complete

## Description

Detect circular/recursive proof dependencies.

## Requirements

1. Build call graph of proof bodies
2. Detect cycles in the graph
3. Report error for circular proofs

## Acceptance Criteria

- [x] Circular proof detected
- [x] Acyclic proof passes
- [x] Test passes

## Implementation Notes

- Added `TypeEnv::check_proof_cycles(&[ProofDef])` as the direct Stage 3 checker API.
- The checker builds a proof-local call graph by traversing proof expression bodies and recording `Expr::Call` references whose callee name matches another proof in the checked slice.
- Program typechecking now runs cycle detection for module-scoped proofs and impl-scoped proofs before per-proof totality traversal.
- This slice intentionally does not implement theorem proving, full proof-term typechecking, `Prop` kind promotion, CLI/cache reporting, or runtime semantics.

## Verification

- RED: `CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-typeck --test task_1375c_circular_proof_detection -- --nocapture` initially failed because `TypeEnv::check_proof_cycles` was missing.
- `CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo fmt --check` — passed.
- `CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-typeck --test task_1375c_circular_proof_detection -- --nocapture` — 11 passed.
- `CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-typeck --test task_1375b_partial_match_detection -- --nocapture` — 15 passed.
- `CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-typeck --test task_1365_proof_name_checking -- --nocapture` — 6 passed.
- `CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo check -p ash-typeck` — passed.
- Codex review found qualified-call and duplicate-proof-name blockers; remediation added regressions for both, reran focused gates, and Codex re-review reported no blocking issues.

## Related

- [TASK-1375](TASK-1375-stage3-totality-checking.md)
