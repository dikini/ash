# TASK-1844: Add canonical cross-boundary target fixture

## Description

Add one canonical target fixture proving `fn` + explicit row + target `do { ... }` reaches Core metadata.

## Requirements

- Fixture should be small and ordinary: one row-bearing `fn`, one target `do { ... }` body, one inert `workflow main` only if the engine still needs an entrypoint.
- Assert parser row, engine callable row summary, and Core callable row.
- Assert no authority is installed by the row.

## Completion criteria

- [x] Fixture passes.
- [x] Fixture documents the intended target path.

## Evidence

- Added `crates/ash-engine/tests/task_1844_core_computation_conformance.rs`.
- The fixture checks a row-bearing `fn read(...) -> String where row { PosixFs.read } { do { out <- path; return out } }`, engine row summary source `WhereRow`, and Core row operation metadata.
- Verification: `cargo test -p ash-engine --test task_1844_core_computation_conformance` passed.

## Depends on

- TASK-1841; TASK-1842.
