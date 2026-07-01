# TASK-1511: Implement Deferred QuickCheck Combinators in Ordinary Ash

## Status: ✅ Complete / Phase 176 Reconciled

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

## Phase 176 recursive-combinator reconciliation

Phase 176 re-opened this deferred item under TASK-1799/TASK-1800. The stale blocker is no longer just closure visibility: TASK-1798 proves module-level helper visibility inside closures. The current disposition is:

5. ✅/📝 **`recursive<T>(base: Strategy<T>, rec: (Strategy<T>) -> Strategy<T>) -> Strategy<T>`**
   - Public SPEC-087 name is present and importable.
   - Execution is explicitly fail-closed through private `recursive_deferred` until parser/type-metadata support can accept the required ordinary-Ash size-descending helper implementation.

6. ✅/📝 **`recursive_with<T>(base: Strategy<T>, rec: (Strategy<T>) -> Strategy<T>, config: RecursiveConfig) -> Strategy<T>`**
   - Public SPEC-087 name and `RecursiveConfig { base_weight, expand_weight, size_step }` are present and importable.
   - Real bounded recursive generation remains deferred; no hidden Rust fallback or self-referential value binding was introduced.

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

**Impact:** Self-referential value implementations of `recursive` cannot be implemented. Phase 176 preserved the SPEC-087 public API and config shape, but routes execution through a fail-closed guard.

**Workaround:** Use the Phase 176 public names for import/type surface coverage only. Real bounded generation should use an ordinary-Ash size-descending helper once parser/type-metadata support accepts the required fn-body helper shapes; do not add a hidden Rust fallback.

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
- ✅ Language feature: closures with variable capture/module-helper visibility for this use case (TASK-1798)
- 📝 Language/parser substrate: fn-body helper shapes needed for bounded recursive generation still need follow-up parser/type-metadata support
- 📝 Language feature: self-referential values (not required for the Phase 176 chosen API, still not planned)

## Notes

The goal of **no builtins for combinators** is partially achieved. All implemented combinators are ordinary Ash functions. The recursive combinators now have their final public names and config shape, but execution remains fail-closed. The next implementation path is parser/type-metadata support for the ordinary-Ash size-descending helper design documented in `../../audit/PHASE-176-quickcheck-recursive-combinator-audit.md`, not a hidden Rust fallback.
