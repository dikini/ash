# TASK-1740: Implement bounded notation import/export propagation or explicit non-propagation

## Status: ✅ Complete

## Summary

Implement the TASK-1739 decision: preserve explicit non-propagation for notation declarations across imports/exports, while keeping local notation behavior and no-leakage invariants covered by tests.

## Specification Reference

- PLAN-170: notation scoping track
- TASK-1739 design note: `docs/design/phase-170-notation-summary-export-semantics.md`
- SPEC-095c §7 and §10: active notation tables
- PLAN-169 TASK-1732: local notation table diagnostics

## Dependencies

- ✅ TASK-1739: Notation summary/export semantics design

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Imported/exported notation propagation | PLAN-169 non-goal | Needed summary carrier design | TASK-1739 concluded carriers are not ready for honest propagation | Preserve explicit non-propagation; add negative tests proving imported/exported notation is not active | Scope matrix tests pass |

## Requirements

1. Follow the TASK-1739 decision exactly; do not silently broaden notation semantics.
2. Add explicit tests proving imported/exported notation is not active and diagnostics remain stable.
3. Preserve local and inline-module no-leakage behavior from Phase 169.
4. Keep local `pub` notation usable only in its declaring module.
5. Prove ordinary callable imports remain usable by direct call syntax even when notation aliases do not propagate.
6. Ensure notation targets remain ordinary callable paths; no authority is granted by notation aliases.
7. Do not add notation metadata to module-summary/export carriers in this task.

## TDD Steps

1. Add scope matrix tests before code changes.
2. Implement the minimal non-propagation enforcement/fixtures if current behavior is not already explicit.
3. Add negative leakage tests for imported/exported notation and parent/inline boundaries.
4. Run parser, typeck, engine, and workspace checks.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser --test task_1732_local_notation_table_resolution
  - cargo test -p ash-parser
  - cargo test -p ash-typeck
  - cargo test -p ash-engine
  - cargo check --workspace
  - cargo clippy -p ash-parser -p ash-typeck -p ash-engine --all-targets --all-features -- -D warnings
  - cargo fmt --check
checklist:
  - [x] Scope matrix positive cases pass.
  - [x] Scope matrix negative leakage cases pass.
  - [x] Imported/exported behavior matches TASK-1739 exactly: notation does not propagate across imports/exports.
```

## Closeout evidence

- Implemented the TASK-1739 non-propagation decision as explicit regression coverage, without adding notation module-summary carriers.
- Added `crates/ash-engine/tests/task_1740_notation_non_propagation.rs`:
  - imported `pub infixl` notation is not active in the caller scope,
  - the exported callable target remains usable through ordinary direct-call syntax.
- Extended `crates/ash-parser/tests/task_1732_local_notation_table_resolution.rs` with a local `pub infixl` case proving `pub` notation remains active locally.
- Existing parent/inline no-leakage tests remain in `task_1732_local_notation_table_resolution.rs`.
- Fresh verification:
  - `cargo test -p ash-parser --test task_1732_local_notation_table_resolution`
  - `cargo test -p ash-parser`
  - `cargo test -p ash-typeck`
  - `cargo test -p ash-engine`
  - `cargo check --workspace`
  - `cargo clippy -p ash-parser -p ash-typeck -p ash-engine --all-targets --all-features -- -D warnings`
  - `cargo fmt --check`
  - `git diff --check`
