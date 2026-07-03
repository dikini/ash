# TASK-1851: Add impl/type-qualified operation authority fixtures

## Description

Add fixtures proving impl/type-qualified operation identities reach admission diagnostics and discharge checks.

## Requirements

- Cover `PosixFs::read` as a concrete impl/type-qualified operation row.
- Preserve existing parser/typechecker validation for `F::read` where a type parameter is bounded by an interface.
- Assert admission diagnostics name the operation identity rather than treating the row as a grant.

## Completion criteria

- [x] Concrete impl/type-qualified row fixtures pass.
- [x] Diagnostics preserve operation identity.
- [x] Generic identity validation remains covered by existing typechecker tests.

## Evidence

- Added `task_1851_operation_authority_diagnostic_preserves_impl_qualified_identity` for `PosixFs::read`; existing `crates/ash-typeck/tests/task_1810_impl_qualified_operation_row_identity.rs` continues to cover generic and concrete identity validation.

## Depends on

- TASK-1850.
