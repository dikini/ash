# TASK-1376a: Add `Kind::Prop` Variant

## Status: ✅ Complete

## Description

Add `Prop` as a distinct kind in the type system.

## Requirements

1. Add `Kind::Prop` variant to `ash_core::Kind`
2. Update kind checking to handle `Prop`
3. Ensure `Prop` is not compatible with `Type`
4. Parse `Prop` as a source kind atom where kind annotations are accepted

## Acceptance Criteria

- [x] `Kind::Prop` exists
- [x] `Prop` incompatible with `Type`
- [x] `Prop` displays as `Prop` and has arity 0
- [x] Parser accepts `Prop` as a kind atom
- [x] Test passes

## Implementation Notes

- Added `Kind::Prop` to `ash_core::Kind` without implementing proof irrelevance or runtime escape prevention; those remain TASK-1376b/c.
- `Kind::Prop.is_type()` is false, `Kind::Prop != Kind::Type`, and `Kind::arrow(Kind::Prop, Kind::Type)` displays as `Prop -> *`.
- The parser now accepts `Prop` in explicit kind annotations and preserves it in the surface AST.
- Parser boundary coverage proves longer identifiers such as `Property` are not consumed as a `Prop` kind prefix.

## Verification

- RED: `CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-core --test task_1376a_prop_kind -- --nocapture` initially failed because `Kind::Prop` did not exist.
- `CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo fmt --check` — passed.
- `CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-core --test task_1376a_prop_kind -- --nocapture` — 2 passed.
- `CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-parser --test task_1376a_prop_kind_parser -- --nocapture` — 3 passed.
- `CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo check --workspace` — passed.
- Codex review reported no blocking or non-blocking issues after the parser boundary regression and doc-comment patch.

## Related

- [TASK-1376](TASK-1376-stage3-prop-kind.md)
