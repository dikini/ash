# TASK-1502: QuickCheck combinators, recursion, and weights

## Status: 🚧 Partial / Runner-Side MVP

## Description

Implement namespaced function combinators for strategy composition, weighted choice, projection-based shrinking helpers, explicit shrink wrappers, and bounded recursive generation.

## Specification Reference

- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [PLAN-151: QuickCheck v1 Ordinary Strategy Semantics](../PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [DESIGN-NOTE: QuickCheck v1 Ordinary Strategy Semantics](../../design/DESIGN-NOTE-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)

## Dependencies

- ✅ TASK-1498: QuickCheck stdlib module split and prelude
- ✅ TASK-1499: GenContext, RNG, and Strategy value core

## Implementation

### Stdlib Module

Created `std/src/test/quickcheck/combinator.ash` with:

- `Weighted<T>` type: `{ weight: Int, strategy: Strategy<T> }`
- `RecursiveConfig` type: `{ max_depth: Int, breadth: Int }`
- Helper functions:
  - `weighted(weight: Int, strategy: Strategy<T>) -> Weighted<T>`
  - `default_recursive_config() -> RecursiveConfig` (max_depth=5, breadth=3)
  - `recursive_config(max_depth: Int, breadth: Int) -> RecursiveConfig`

- Builtin function declarations (awaiting runner implementation):
  - `one_of<T>(choices: List<Strategy<T>>) -> Strategy<T>`
  - `one_of_weighted<T>(choices: List<Weighted<T>>) -> Strategy<T>`
  - `map<A, B>(s: Strategy<A>, f: (A) -> B) -> Strategy<B>`
  - `map_with_shrink<A, B>(s: Strategy<A>, f: (A) -> B, shrink: (B) -> List<B>) -> Strategy<B>`
  - `map2<A, B, C>(sa: Strategy<A>, sb: Strategy<B>, f: (A, B) -> C) -> Strategy<C>`
  - `with_shrink<T>(s: Strategy<T>, shrink: (T) -> List<T>) -> Strategy<T>`
  - `append_shrink<T>(s: Strategy<T>, extra: List<T>) -> Strategy<T>`
  - `prepend_shrink<T>(s: Strategy<T>, extra: List<T>) -> Strategy<T>`
  - `recursive<T>(base: Strategy<T>, rec: (Strategy<T>) -> Strategy<T>, config: RecursiveConfig) -> Strategy<T>`
  - `recursive_with<T>(base: Strategy<T>, rec: (Strategy<T>) -> Strategy<T>, max_depth: Int, breadth: Int) -> Strategy<T>`

### Why Builtins?

The combinators require creating new function values (closures) at runtime:
- `map` needs to wrap `s.gen` with `f` to create `new_gen(ctx) = f(s.gen(ctx))`
- `one_of` needs to select a strategy based on context
- `recursive` needs to manage depth/breadth state

Ash does not support lambda/closures in ordinary source code, so these must be implemented as builtin functions in the Rust runner/interpreter.

### Engine Blockers Fixed

- ✅ Type-import-in-type-definitions: `check_module_file` now processes imports and registers imported types before local types
- ✅ Pub mod resolution: `mod.ash` pattern works correctly
- ✅ Multi-line `pub use` parsing: trailing commas handled
- ✅ Duplicate type semantic summary: skip instead of error

## Verification

```
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-engine --test phase151_quickcheck_stdlib -- --nocapture
  - cargo test -p ash-cli --test stdlib_corpus_check -- --nocapture
  - cargo clippy -p ash-cli --all-targets -- -D warnings
  - git diff --check
checklist:
  - [x] Stdlib module parses and checks individually
  - [x] All QuickCheck modules compile without errors
  - [x] Integration tests pass
  - [x] Stdlib corpus baseline maintained
```

## Test Results

- `cargo test -p ash-engine --test phase151_quickcheck_stdlib` — 3 passed
- `cargo test -p ash-cli --test stdlib_corpus_check` — 2 passed (60 files: 54 passing, 6 failing)

## Dependencies for Next Task

- Runner-side builtin implementation for combinators
- TASK-1501: Parser/typechecker for `by test quickcheck with { ... }`
- TASK-1506: Phase closeout

## Notes

The combinators are declared in the stdlib but require runner-side builtin implementation. The stdlib surface is stable and ready for builtin wiring. Full source-visible implementations would require Ash lambda/closures, which is out of scope for Phase 151.
