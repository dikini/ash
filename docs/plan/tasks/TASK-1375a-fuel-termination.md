# TASK-1375a: Fuel-Based Termination Analysis

## Status: ✅ Complete

## Description

Implement fuel-based proof-body traversal for Stage 3 proof totality checking.

## Requirements

1. Add `fuel` parameter to proof checking (default: 1000 steps)
2. Count reduction steps during proof normalization/traversal
3. Exceeding fuel = `untested` result (not error)

## Acceptance Criteria

- [x] Fuel counter tracks reduction steps
- [x] Fuel exceeded returns `untested`
- [x] Configurable via CLI flag
- [x] Test passes

## Implementation Notes

- Added `DEFAULT_PROOF_FUEL = 1000` and typed proof-totality result carriers in `ash-typeck`:
  - `ProofTotalityResult`
  - `ProofTotalityStatus::{Checked, Untested(...)}`
  - `ProofTotalityUntestedReason::FuelExhausted`
- Added `TypeEnv::check_proof_totality_with_fuel(proof, fuel)` and kept `check_proof_totality(proof)` as the default-fuel wrapper.
- Added a conservative AST traversal over proof expression bodies that consumes one fuel step per visited surface expression node.
- Fuel exhaustion returns `Ok(ProofTotalityResult { status: Untested(FuelExhausted), ... })` from the direct checker and is not converted to a type error during program registration. The result is not persisted or reported through CLI output in this slice.
- Added `TypeCheckConfig { proof_fuel }` plus configured typecheck/engine entrypoints so `ash check --proof-fuel <N>` reaches program typechecking.

## Boundary

This task does not implement theorem proving, full proof-term normalization, non-exhaustive match rejection, or circular proof dependency detection. Those remain owned by TASK-1375b and TASK-1375c.

## Verification

- RED: `cargo test -p ash-typeck --test task_1375a_proof_fuel -- --nocapture` initially failed because `DEFAULT_PROOF_FUEL`, `ProofTotalityStatus`, `ProofTotalityUntestedReason`, and `check_proof_totality_with_fuel` did not exist.
- RED: `cargo test -p ash-cli --test test_command check_help_exposes_proof_fuel_flag -- --nocapture` initially failed because `ash check --help` did not expose `--proof-fuel`.
- `cargo fmt --check` — passed.
- `cargo test -p ash-typeck --test task_1375a_proof_fuel -- --nocapture` — 2 passed.
- `cargo test -p ash-typeck --test task_1367_proof_totality_stub -- --nocapture` — 3 passed.
- `cargo test -p ash-cli --test test_command check_help_exposes_proof_fuel_flag -- --nocapture` — 1 passed.
- `cargo test -p ash-cli --test test_command check_proof_fuel_flag_accepts_explicit_value -- --nocapture` — 1 passed.
- `cargo check --workspace` — passed.
- `cargo clippy -p ash-typeck -p ash-engine -p ash-cli --all-targets --all-features -- -D warnings` — passed.

## Related

- [TASK-1375](TASK-1375-stage3-totality-checking.md)
