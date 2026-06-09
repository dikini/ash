# TASK-1376c: Runtime Escape Prevention

## Status: ✅ Complete

## Description

Prevent `Prop`-typed values from escaping into runtime code.

## Requirements

1. Reject functions returning `Prop`
2. Reject `Prop` in struct fields
3. Reject `Prop` in enum variants

## Acceptance Criteria

- [x] `fn foo() -> Prop` rejected
- [x] `fn foo() -> Proof` rejected when `Proof` is a transparent alias to `Prop`
- [x] `builtin fn foo() -> Prop` rejected
- [x] `builtin fn foo() -> Proof` rejected when `Proof` is a transparent alias to `Prop`
- [x] `Prop` in struct field rejected
- [x] transparent alias to `Prop` in struct field rejected
- [x] generic transparent alias to `Prop` in struct field rejected
- [x] `Prop` in enum variant rejected
- [x] transparent alias to `Prop` in enum variant rejected
- [x] Test passes

## Implementation Notes

- Added local/static runtime escape checks in `ash-typeck` only.
- `Prop` is lowered as a `Kind::Prop` constructor so diagnostics can identify Prop-kind values explicitly.
- Runtime escape rejection covers ordinary/builtin function returns and ordinary ADT runtime representations (struct fields and enum variant payload fields), including transparent aliases to `Prop` at those seams.
- This task does not implement full theorem proving or codegen/runtime proof erasure.

## Verification

- RED (Codex remediation): `cargo test -p ash-typeck --test task_1376c_runtime_escape_prevention` failed with 4 passed / 2 failed; the new alias-to-`Prop` struct-field and enum-variant regressions were accepted before the fix.
- GREEN (focused): `cargo test -p ash-typeck --test task_1376c_runtime_escape_prevention` — 9 passed.
- Package regression: `cargo test -p ash-typeck` — passed, including doc tests (38 passed, 1 ignored).

## Related

- [TASK-1376](TASK-1376-stage3-prop-kind.md)
