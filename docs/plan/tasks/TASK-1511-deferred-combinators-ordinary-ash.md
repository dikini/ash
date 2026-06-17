# TASK-1511: Implement Deferred QuickCheck Combinators in Ordinary Ash

## Status: 📝 Planned / Blocked on Language Features

## Description

Implement the remaining QuickCheck combinators that were deferred from TASK-1502 because they require Ash language features not yet available. This task serves as a complex use case to test and validate the Ash language itself — what works, what is awkward, what breaks.

## Deferred Combinators

The following combinators were removed from `std/src/test/quickcheck/combinator.ash` because they cannot be implemented in ordinary Ash today:

1. **`one_of<T>(choices: List<Strategy<T>>) -> Strategy<T>`**
   - Randomly selects one strategy from a list and uses it for generation
   - **Blocker**: Requires `List` indexing or random selection primitive

2. **`one_of_weighted<T>(choices: List<Weighted<T>>) -> Strategy<T>`**
   - Weighted random selection over strategies
   - **Blocker**: Requires weighted random selection primitive

3. **`recursive<T>(base: Strategy<T>, rec: (Strategy<T>) -> Strategy<T>, config: RecursiveConfig) -> Strategy<T>`**
   - Bounded recursive generation with depth/breadth limits
   - **Blocker**: Requires managing mutable depth/breadth state across calls

4. **`recursive_with<T>(base: Strategy<T>, rec: (Strategy<T>) -> Strategy<T>, max_depth: Int, breadth: Int) -> Strategy<T>`**
   - Convenience wrapper for `recursive` with explicit config
   - **Blocker**: Same as `recursive`

5. **`append_shrink<T>(s: Strategy<T>, extra: List<T>) -> Strategy<T>`**
   - Appends extra shrink candidates to a strategy's shrink list
   - **Blocker**: Requires list concatenation (`List<T> ++ List<T>`)

6. **`prepend_shrink<T>(s: Strategy<T>, extra: List<T>) -> Strategy<T>`**
   - Prepends extra shrink candidates to a strategy's shrink list
   - **Blocker**: Requires list concatenation

## Language Gaps Identified

### 1. No `let` Destructors for Records

**Current state:** `let { gen, shrink } = strategy` is NOT supported by the parser.

**Workaround:** Use field access (`strategy.gen`, `strategy.shrink`).

**Impact:** Combinators that need to extract multiple fields from a strategy are verbose:
```ash
-- What we want:
let { gen, shrink } = s;

-- What we have to write:
let gen = s.gen;
let shrink = s.shrink;
```

**Required work:** Extend `parse_do_stmt` in `crates/ash-parser/src/parse_expr.rs` to support record patterns after `let`.

### 2. Type Annotation Quirks in `fn` Expressions with Imported Types

**Current state:** Explicit type annotations like `fn(_ctx: GenContext) -> Int` fail when `GenContext` is imported from another module.

**Error:** `expected ($ash_dependency$GenContext) -> Var<18>, got (GenContext) -> Int`

**Workaround:** Let the typechecker infer types (`fn(_ctx) { 42 }`).

**Impact:** Type safety is reduced; developers must rely on inference.

**Required work:** Fix type unification for imported types in `fn` expression type annotations.

### 3. No True Closures / Variable Capture

**Current state:** `fn` expressions create anonymous functions but they do NOT capture variables from the enclosing scope. They are not true closures.

**Impact:** Combinators like `recursive` cannot easily manage state:
```ash
-- This does NOT work: depth is not captured
pub fn recursive<T>(base: Strategy<T>, rec: (Strategy<T>) -> Strategy<T>, config: RecursiveConfig) -> Strategy<T> {
    let current_depth = 0;  -- NOT captured by fn below
    Strategy {
        gen: fn(ctx) {
            if current_depth < config.max_depth {  -- ERROR: current_depth not in scope
                ...
            }
        }
    }
}
```

**Required work:** Either:
- Implement true closures with variable capture in the interpreter
- OR provide a state-passing primitive (e.g., `GenContext` carries depth)

### 4. No List Concatenation / Indexing Primitives

**Current state:** No `++` operator for lists, no `list[i]` indexing.

**Impact:** `one_of`, `append_shrink`, `prepend_shrink` cannot be implemented.

**Required work:** Add list primitives to the stdlib or language.

## Proposed Implementation Path

### Option A: Language-First (Recommended)

1. Implement `let` destructors (parser + typechecker + interpreter)
2. Fix imported type unification in `fn` annotations
3. Add list concatenation/indexing primitives
4. Implement true closures OR state-passing in `GenContext`
5. Then implement all deferred combinators in ordinary Ash

### Option B: GenContext-First (Pragmatic)

1. Extend `GenContext` to carry depth/breadth state and a random choice index
2. Add list primitives
3. Implement combinators using `GenContext` state instead of closures
4. Defer true closures to a later language phase

## Verification

When this task is implemented:
- All 6 deferred combinators should be ordinary Ash functions (no builtins)
- `std/src/test/quickcheck/combinator.ash` should contain all combinators
- `mod.ash` should re-export all combinators
- Stdlib corpus check should pass
- A test fixture should demonstrate each combinator working

## Dependencies

- Language feature: `let` destructors for records (TASK-1520-TASK-1522 may enable workarounds)
- Language feature: imported type unification in `fn` annotations (TASK-1540-TASK-1542 will fix type annotation quirks)
- Language feature: list concatenation / indexing (TASK-1530-TASK-1532 will provide `concat`, `index`)
- Language feature: closures with variable capture (TASK-1520-TASK-1524 closure refinement)
- OR: `GenContext` state extension as workaround

## Notes

This task is intentionally broad because it serves as a forcing function for language improvements. The QuickCheck library is a complex real-world use case that tests the Ash language's expressiveness. As the language grows, these combinators should become implementable without builtins.

**Phase 152 Dependency:** This task is blocked by TASK-1520-TASK-1524 (closure refinement). The closure refinement will enable:
- Pure closures with pure captures (e.g., `fn make_adder(n) { fn(x) { n + x } }`)
- Higher-order function patterns in ordinary Ash
- More natural combinator implementations

**Workaround until Phase 152 completes:** Use `GenContext` state passing instead of closure capture for stateful combinators like `recursive`.

The goal is: **no builtins for combinators**. Everything should be ordinary Ash.
