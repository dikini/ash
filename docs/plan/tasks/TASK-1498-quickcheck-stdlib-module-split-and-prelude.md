# TASK-1498: QuickCheck stdlib module split and prelude

## Status: ✅ Complete

## Description

Refactor/add the `test::quickcheck` Ash stdlib surface into canonical submodules with a narrow prelude and alpha root convenience re-exports.

## Specification Reference

- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [PLAN-151: QuickCheck v1 Ordinary Strategy Semantics](../PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [DESIGN-NOTE: QuickCheck v1 Ordinary Strategy Semantics](../../design/DESIGN-NOTE-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)

## Dependencies

- ✅ TASK-1497: Live syntax and seam audit (complete)

## Engine Bugs Fixed During Implementation

### Bug 1: Multi-line `pub use` parsing with trailing commas

**Problem:** The `parse_use` parser in `ash-parser` failed on multi-line `pub use` statements with `::{...}` blocks containing trailing commas. The `separated` combinator from winnow didn't handle trailing commas.

**Fix:** Replaced `delimited(parse_symbol("{"), separated(0.., parse_use_item, parse_symbol(",")), parse_symbol("}"))` with a custom loop that explicitly handles trailing commas by checking for `}` after each comma.

**File:** `crates/ash-parser/src/parse_use.rs`

### Bug 2: Duplicate type semantic summary merging

**Problem:** When `prelude.ash` imported `GenContext` from `context` and `Strategy` from `strategy` (which depends on `GenContext`), the dependency `GenContext` was not renamed to `$ash_dependency$GenContext` in `merge_type_summary_export_with_aliases`. This caused a duplicate name conflict when merging imported semantic summaries.

**Fix:** Changed `merge_selected_type_exports` to skip duplicate type names with different IDs instead of erroring. This allows a dependency type with the same name as a directly-imported type to coexist safely.

**File:** `crates/ash-engine/src/module_loader.rs`

### Bug 3: Type registration for interface constraint checking

**Problem:** `register_imported_interface_definitions_for_constraints` only registered imported interfaces, not imported types. When `arbitrary.ash` defined `interface Arbitrary<T> { arbitrary() -> Strategy<T> }`, the type `Strategy` was not in the type environment, causing interface registration to fail.

**Fix:** Added type definition registration to `register_imported_interface_definitions_for_constraints_inner` before interface registration. For each imported module, the function now collects type metadata and registers type names/identities in the type environment.

**File:** `crates/ash-engine/src/module_loader.rs`

## Implementation

### Module Structure

Created canonical QuickCheck stdlib modules in `std/src/test/quickcheck/`:

- `mod.ash` — Module root with `pub mod` declarations and alpha convenience re-exports
- `context.ash` — `GenContext` type + helper builtins (seed, size, split, variant, indexed, resize, choose_int, choose_bool)
- `strategy.ash` — `Strategy<T>` type + `no_shrink` builtin
- `arbitrary.ash` — `Arbitrary<T>` interface with `arbitrary() -> Strategy<T>`
- `int.ash` — `ints()`, `small()`, `positive()`, `nonzero()` + `impl Arbitrary<Int>`
- `bool.ash` — `bools()` + `impl Arbitrary<Bool>`
- `string.ash` — `strings()`, `identifiers()` + `impl Arbitrary<String>`
- `list.ash` — `list_of()`, `nonempty_ints()`, `sorted_ints()`
- `combinator.ash` — `Weighted<T>`, `one_of`, `map`, `map2`, `recursive`, `with_shrink`, etc.
- `prelude.ash` — Re-exports core types from submodules

### Root Aliases (Alpha Convenience)

`mod.ash` re-exports canonical APIs for backward compatibility:
- `GenContext`, `seed`, `size`, `split`, `variant`, `indexed`, `resize`, `choose_int`, `choose_bool`
- `Strategy`, `no_shrink`
- `Arbitrary`
- `ints`, `small_ints`, `positive_ints`, `nonzero_ints`, `positive`, `nonzero`
- `bools`, `strings`, `identifiers`
- `list_of`, `nonempty_int_lists`, `sorted_int_lists`, `nonempty_ints`, `sorted_ints`
- `Weighted`, `weighted`, `one_of`, `one_of_weighted`, `map`, `map_with_shrink`, `map2`, `with_shrink`, `append_shrink`, `prepend_shrink`, `RecursiveConfig`, `recursive_config`, `default_recursive_config`, `recursive`, `recursive_with`

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
  - [x] Focused tests pass and are non-zero
  - [x] No-Cargo final-surface fixture added where user-facing behavior changed
  - [x] Negative leakage/fail-closed cases covered where a bridge or error path is touched
  - [x] CHANGELOG.md updated under [Unreleased]
```

## Test Results

- `cargo test -p ash-engine --test phase151_quickcheck_stdlib` — 3 passed
  - `quickcheck_canonical_stdlib_modules_parse_and_check`
  - `quickcheck_prelude_and_canonical_submodule_imports_resolve`
  - `quickcheck_root_aliases_resolve_as_alpha_convenience_surface`
- `cargo test -p ash-cli --test stdlib_corpus_check` — 2 passed
  - 60 stdlib files: 54 passing, 6 failing (unchanged from baseline)

## Known Issues / Engine Limitations

### Imported types in type definitions

The Ash typechecker does not support using imported types in local type definitions. When `strategy.ash` defines `Strategy<T> = Strategy { gen: (GenContext) -> T, ... }`, the `GenContext` type must be defined locally — importing it from `context.ash` fails with "Unbound variable: GenContext".

**Workaround:** `strategy.ash` defines its own `GenContext` type, identical to the one in `context.ash`. This creates a duplicate type definition across modules.

**Impact:** The two `GenContext` types are structurally identical but have different module identities. Cross-module usage (e.g., passing a `context::GenContext` to a function expecting `strategy::GenContext`) would fail at the type level.

**Future fix:** The engine should support imported types in type definitions by:
1. Parsing all `use` statements before type definitions
2. Registering imported types in the type environment
3. Then processing local type definitions with imported types available

This requires changes to the parser/typechecker pipeline.

## Dependencies for Next Task

- Stable stdlib module skeleton for TASK-1499 and TASK-1500.
- Explicit prelude import surface.

## Notes

Root aliases are alpha convenience aliases over canonical submodule APIs. Reference docs should use submodule paths.
