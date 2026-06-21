# TASK-1661: Add Core mode text format support

**Status:** Done
**Phase:** [PLAN-163](../PLAN-163-CORE-LAZY-MEMO-MODES.md)
**Owner:** Phase 163

## Description

Extend the `.core` debug/fixture text format for SPEC-101 mode types and expressions.

## Specification Reference

- [SPEC-101 §5](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md#5-core-values-and-expressions)
- [SPEC-101 §15](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md#15-open-questions)

## Dependencies

- [TASK-1660](TASK-1660-core-mode-ast-carriers.md)

## Requirements

1. Parse and serialize mode types using exactly:
   - `(strict Type)`
   - `(lazy Type Row)`
   - `(memo Type Row)`
2. Parse and serialize thunk values, `LetMode`, and `Force`.
3. Use exactly `(thunk lazy Type Row Expr)` and `(thunk memo Type Row Expr)` for thunk values.
4. Use exactly `(let-mode Name strict : Type Expr Expr)`,
   `(let-mode Name lazy : Type Expr Expr)`, and
   `(let-mode Name memo : Type Expr Expr)` for `LetMode`.
5. Use exactly `(force Name Atom Expr)` for `Force`.
6. Add fixture examples for strict, lazy, memo, and invalid mode mismatch shapes.
7. Preserve parser/serializer round-trips for existing Phase 161/162 fixtures.
8. Parser sets every parsed thunk's `captures` to `CoreCaptureSet { values: vec![] }`.
9. Serializer always omits `captures`; non-empty captures are never emitted in `.core` fixture
   text.

## Golden Syntax Examples

These examples must parse and serialize canonically:

```text
(strict Int)
(lazy Int {cap fs.read})
(memo (record (a Int) (b String)) {})
(thunk lazy Int {cap fs.read} (raise (cap fs read () Int)))
(let-mode x lazy : (lazy Int {cap fs.read}) (call read-int) (force y x y))
```

This malformed Core fixture must parse but be rejected by validation/type checking:

```text
(let-mode x lazy : (memo Int {}) 1 x)
```

## TDD Steps

1. Add failing tests in `crates/ash-core/tests/task_1661_core_mode_text.rs`.
2. Add `.core` fixtures under `crates/ash-core/tests/fixtures/core/`.
3. Run `cargo test -p ash-core --test task_1661_core_mode_text`; expect parse failures.
4. Implement parser and serializer cases in `core_ash_text.rs`.
5. Re-run `task_1661`, `task_1622`, `task_1623`, and `task_1624`.

## Completion Checklist

- [x] Mode type text round-trips.
- [x] Thunk values round-trip.
- [x] Parsed thunk capture metadata defaults to an empty `CoreCaptureSet`.
- [x] Serializer omits capture metadata even when the AST contains non-empty capture metadata.
- [x] `LetMode` and `Force` expressions round-trip.
- [x] Existing Core text tests remain green.
