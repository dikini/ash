# TASK-1375b: Partial Match Detection

## Status: ✅ Complete

## Description

Detect non-exhaustive pattern matches in proof bodies.

## Requirements

1. Analyze `match` expressions in proof bodies
2. Report error if patterns are not exhaustive
3. Require `_` catch-all or complete coverage

## Acceptance Criteria

- [x] Non-exhaustive match detected
- [x] Exhaustive match passes
- [x] Test passes

## Implementation Notes

- Extended the Stage 3 proof-totality traversal to inspect `Expr::Match` nodes in proof bodies.
- Proof parameters are now bound into a proof-local `TypeEnv` before expression traversal so match scrutinee variables can be typed.
- Match arm patterns are lowered to core patterns and checked with the existing conservative `check_match_exhaustive` coverage engine.
- Match arm, block `let`, and function-parameter bindings are threaded into nested traversal, so nested proof matches can inspect variables introduced by enclosing proof expressions.
- Proof matches with top-level `_`/variable catch-all arms are treated as exhaustive without requiring scrutinee type resolution, preserving the Stage 3 boundary that TASK-1375b does not fully typecheck arbitrary proof terms.
- Generic or otherwise unresolved proof parameter annotations and opaque simple `let` initializers are handled conservatively with fresh proof-local type variables, avoiding regressions for proof terms outside this partial-match slice.
- Generic ADT proof parameters preserve their outer constructor identity, allowing complete constructor-name coverage such as `Some`/`None` over `Option<T>` even when type arguments remain proof-local variables.
- The generic-ADT constructor-name fallback only accepts untyped-irrefutable constructor payload patterns, so nested refutable payloads such as `Some(Some(_))` remain rejected rather than masking missing cases.
- Match scrutinee traversal runs before non-wildcard scrutinee type resolution, preserving TASK-1375a fuel semantics so low-fuel traversals return `Untested(FuelExhausted)` rather than resolution errors.
- Missing coverage returns a `TypeEnvError::InvalidDefinition` diagnostic that names the missing witness and tells authors to add `_` or cover every constructor.
- Fuel semantics from TASK-1375a are preserved; match analysis still consumes traversal fuel and fuel exhaustion remains an untested result rather than an error.

## Boundary

- This task validates AST-level `Expr::Match` proof bodies. It does not introduce source `match` parsing syntax; source-level match syntax remains a parser task if needed.
- This task does not implement circular proof dependency detection, full proof-term typechecking, theorem proving, or `Prop` kind semantics.
- `if let` and `with_error` handler coverage are not expanded in this slice.

## Verification

- RED: `cargo test -p ash-typeck --test task_1375b_partial_match_detection -- --nocapture` initially failed because a proof match missing `None` returned `ProofTotalityResult { status: Checked, ... }`.
- `cargo fmt --check` — passed.
- `cargo test -p ash-typeck --test task_1375a_proof_fuel -- --nocapture` — 2 passed.
- `cargo test -p ash-typeck --test task_1367_proof_totality_stub -- --nocapture` — 3 passed.
- `cargo test -p ash-typeck --test task_1375b_partial_match_detection -- --nocapture` — 15 passed.
- `cargo check -p ash-typeck` — passed.
- `cargo clippy -p ash-typeck --lib --all-features -- -D warnings` — passed.
- Codex review found a catch-all fuel-ordering blocker; remediation added `low_fuel_catchall_match_returns_untested_before_pattern_type_error`, reran focused gates, and Codex re-review reported no blocking issues. Full all-target clippy is deferred until TASK-1375c's intentionally red follow-on test is implemented.

## Related

- [TASK-1375](TASK-1375-stage3-totality-checking.md)
