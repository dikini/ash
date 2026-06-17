# TASK-1511: Implement Deferred QuickCheck Combinators in Ordinary Ash

## Status: ✅ Complete

## Description

Implement the remaining QuickCheck combinators that were deferred from TASK-1502 because they require Ash language features not yet available. This task serves as a complex use case to test and validate the Ash language itself — what works, what is awkward, what breaks.

## Implemented Combinators

The following combinators have been implemented in ordinary Ash:

1. ✅ **`one_of<T>(strategies: List<Strategy<T>>) -> Strategy<T>`**
   - Randomly selects one strategy from a list and uses it for generation
   - Implemented using `choose_int` and `index_strategies` helper

2. ✅ **`one_of_weighted<T>(choices: List<Weighted<T>>) -> Strategy<T>`**
   - Weighted random selection over strategies
   - Implemented using `sum_weights` and `pick_weighted` helpers

3. ✅ **`append_shrink<T>(s: Strategy<T>, extra: List<T>) -> Strategy<T>`**
   - Appends extra shrink candidates to a strategy's shrink list
   - Uses `concat` from list module

4. ✅ **`prepend_shrink<T>(s: Strategy<T>, extra: List<T>) -> Strategy<T>`**
   - Prepends extra shrink candidates to a strategy's shrink list
   - Uses `concat` from list module

## Deferred Combinators

The following combinators remain deferred due to language limitations:

5. 📝 **`recursive<T>(base: Strategy<T>, rec: (Strategy<T>) -> Strategy<T>, config: RecursiveConfig) -> Strategy<T>`**
   - **Blocker**: Requires self-referential values (recursive value binding)
   - The language cannot express `let self_ref = Strategy { gen: fn(ctx) { rec(self_ref).gen(ctx) } }`

6. 📝 **`recursive_with<T>(base: Strategy<T>, rec: (Strategy<T>) -> Strategy<T>, max_depth: Int, breadth: Int) -> Strategy<T>`**
   - **Blocker**: Same as `recursive`

## Language Limitations Encountered

### 1. No `if` inside `fn` expressions

**Current state:** `if` expressions cannot be parsed inside `fn` bodies.

**Workaround:** Use `match` on boolean expressions:
```ash
-- Instead of:
if condition then a else b

-- Use:
match condition {
    true => a,
    false => b
}
```

### 2. No Self-Referential Values

**Current state:** Cannot create recursive values like `let self_ref = Strategy { gen: fn(ctx) { rec(self_ref).gen(ctx) } }`.

**Impact:** The `recursive` combinator cannot be implemented.

**Workaround:** Use `GenContext` size to limit recursion depth, but this requires passing the recursive strategy as an argument, which changes the API.

## Verification

- ✅ All 6 deferred combinators from original list have been addressed
- ✅ 4 implemented in ordinary Ash
- ✅ 2 documented as deferred with specific blockers
- ✅ `std/src/test/quickcheck/combinator.ash` parses and typechecks
- ✅ `mod.ash` exports all new combinators
- ✅ All ash-engine tests pass (except pre-existing task_870 failure)

## Files Modified

- `std/src/test/quickcheck/combinator.ash` - Added new combinators
- `std/src/test/quickcheck/mod.ash` - Updated exports

## Dependencies Resolved

- ✅ Language feature: `let` destructors for records (Phase 155 complete)
- ✅ Language feature: list concatenation via `concat` (Phase 153 complete)
- ✅ Language feature: type annotation quirks fixed (Phase 154 complete)
- ✅ Language feature: `fn` expressions in struct literals (TASK-1510 complete)
- 📝 Language feature: closures with variable capture (TASK-1580 deferred)
- 📝 Language feature: self-referential values (not yet planned)

## Notes

The goal of **no builtins for combinators** is partially achieved. All implemented combinators are ordinary Ash functions. The `recursive` combinator requires either:
1. Self-referential value support in the language, or
2. A different API design that passes the recursive strategy explicitly

Both options are documented for future phases.
