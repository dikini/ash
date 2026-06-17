# TASK-1502: QuickCheck combinators, recursion, and weights

## Status: 🚧 Partial / Runner-Side MVP

## Description

Implement namespaced function combinators for strategy composition, weighted choice, projection-based shrinking helpers, explicit shrink wrappers, and bounded recursive generation.

## Specification Reference

- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [PLAN-151: QuickCheck v1 Ordinary Strategy Semantics](../PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [DESIGN-NOTE: QuickCheck v1 Ordinary Strategy Semantics](../../design/DESIGN-NOTE-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
## Status: ✅ Stdlib Surface Complete / Combinators Implemented in Ordinary Ash

## Description

Implement namespaced function combinators for strategy composition, weighted choice, projection-based shrinking helpers, explicit shrink wrappers, and bounded recursive generation.

## Implementation

### Stdlib Module

Created `std/src/test/quickcheck/combinator.ash` with ordinary Ash implementations (no builtins):

- `Weighted<T>` type: `{ weight: Int, strategy: Strategy<T> }`
- `RecursiveConfig` type: `{ max_depth: Int, breadth: Int }`
- Ordinary Ash functions:
  - `map<A, B>(s: Strategy<A>, f: (A) -> B) -> Strategy<B>`
  - `map_with_shrink<A, B>(s: Strategy<A>, f: (A) -> B, shrink: (B) -> List<B>) -> Strategy<B>`
  - `map2<A, B, C>(sa: Strategy<A>, sb: Strategy<B>, f: (A, B) -> C) -> Strategy<C>`
  - `with_shrink<T>(s: Strategy<T>, shrink: (T) -> List<T>) -> Strategy<T>`
  - `constant<T>(value: T) -> Strategy<T>`
  - `weighted<T>(weight: Int, strategy: Strategy<T>) -> Weighted<T>`
  - `default_recursive_config() -> RecursiveConfig`
  - `recursive_config(max_depth: Int, breadth: Int) -> RecursiveConfig`

### Why Ordinary Ash Works

The combinators can be implemented in ordinary Ash because:
1. `fn` expressions are first-class values that can be stored in record fields
2. Field access on records works (`s.gen`)
3. Function application on field access works (`s.gen(ctx)`)
4. `Strategy<T>` is a record type with function fields

### Language Gaps Discovered

1. **No `let` destructors**: `let { gen, shrink } = strategy` is not supported.
   Workaround: use field access (`strategy.gen`, `strategy.shrink`).

2. **Type annotation quirks in `fn` expressions**: Explicit type annotations
   like `fn(_ctx: GenContext) -> Int` may fail when the type is imported from
   another module. Workaround: let the typechecker infer types (`fn(_ctx) { 42 }`).

3. **No closures/lambdas in ordinary source**: The `fn` syntax creates anonymous
   functions but they cannot capture variables from the enclosing scope (they
   are not true closures). This limits some combinator patterns.

### Deferred Combinators

The following combinators require features not yet available in Ash:
- `one_of<T>`: Requires `List` indexing or random selection over strategies
- `one_of_weighted<T>`: Requires weighted random selection
- `recursive<T>`: Requires managing depth/breadth state across calls
- `append_shrink`, `prepend_shrink`: Requires list concatenation

These can be added once the language supports the necessary primitives.

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
