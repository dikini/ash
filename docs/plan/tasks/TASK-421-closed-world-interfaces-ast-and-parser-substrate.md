# TASK-421: Closed-World Interfaces AST and Parser Substrate

## Status: ✅ Complete

## Description

Implement the first code follow-on after TASK-415 by adding the parser and AST substrate for the
closed-world interfaces MVP.

This task should make the MVP surface parseable and representable, but should stop short of full
impl resolution and method-call typechecking.

Frozen MVP surface from TASK-415:
- canonical bound form: `T: Interface`
- canonical method-call form: `Interface::method(value)`
- explicit interface declarations
- explicit impl declarations

## Specification Reference

- [TASK-415: Closed-World Interfaces MVP Spec Cut](TASK-415-closed-world-interfaces-mvp-spec-cut.md)
- [TYPES-002 V2 MVP Cut](../../ideas/type-system/TYPES-002-v2-mvp-cut.md)
- [SPEC-003: Type System](../../spec/SPEC-003-TYPE-SYSTEM.md)

## Dependencies

- ✅ TASK-415 complete

## Requirements

### Functional Requirements

1. Extend the parser/module AST to represent:
   - interface declarations
   - impl declarations
   - constrained generic parameters using `T: Interface`
   - explicit namespaced method calls `Interface::method(value)`
2. Add parser tests covering the frozen MVP syntax.
3. Reject obviously malformed interface/impl syntax.
4. Preserve existing language features.

### Non-Functional Requirements

1. Do not implement coherence or type resolution in this task.
2. Do not add associated types/effects.
3. Do not add dynamic dispatch or trait objects.
4. Update `CHANGELOG.md`.

## Files

- Modify: `crates/ash-core/src/ast.rs`
- Modify: `crates/ash-parser/src/surface.rs`
- Modify parser files needed for module/type/expression parsing
- Add/modify parser tests under `crates/ash-parser/tests/`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write failing parser/AST tests

Add tests for the frozen MVP interface/impl/bound/call forms.

### Step 2: Implement AST + parser substrate

Add the minimum syntax/AST support needed to preserve the frozen MVP source contract.

### Step 3: Verify parser crate quality

Run at least:
- `cargo test -p ash-parser`
- `cargo clippy -p ash-parser --all-targets -- -D warnings`
- `cargo fmt --check`

## Completion Checklist

- [x] interface/impl AST substrate added
- [x] parser accepts frozen MVP forms
- [x] parser tests added/updated
- [x] `CHANGELOG.md` updated

## Notes

This task intentionally stops at syntax/AST support. Coherence, impl resolution, and method-call
typechecking belong to later tasks.
