# TASK-1842: Preserve row requirements through direct-style Core metadata

## Description

Prove that target `fn` bodies using `do { ... }` still preserve explicit row requirements through summaries and Core callable metadata.

## Requirements

- Add engine/Core preservation tests first.
- Ensure rows remain requirements and do not install authority.
- Keep row-polymorphic inference out of scope.

## Completion criteria

- [x] Cross-boundary test covers parser -> engine/check -> Core callable row metadata.
- [x] Test proves target `do { ... }` does not affect row metadata.

## Evidence

- Added target ambient `do` raw lowering to ordinary Core `Let` sequencing in `crates/ash-parser/src/lower.rs` so row-bearing local `fn` bodies register through the existing engine path.
- Added `crates/ash-engine/tests/task_1844_core_computation_conformance.rs`, asserting parser body retention, engine callable row summary, and Core callable row metadata for a `where row` function using target `do { ... }`.
- Verification: `cargo test -p ash-engine --test task_1844_core_computation_conformance` passed.

## Depends on

- TASK-1841.
