# PLAN-153: Move List Builtins to Standard Library

**Status:** 📝 Planned
**Spec:** [SPEC-089: List Builtin to Stdlib](../spec/SPEC-089-LIST-BUILTIN-TO-STDLIB.md)
**Amends:** [PLAN-151](PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md) (TASK-1511), [PLAN-152](PLAN-152-CLOSURE-REFINEMENT-AND-TOWER-DOCUMENTATION.md) (TASK-1524)
**Builds on:** [ASSESSMENT-001](../assessments/ASSESSMENT-001-NATIVE-LIST-IMPLEMENTATION.md)
**Task range:** TASK-1530 through TASK-1539

## Goal

Replace Rust-implemented list builtins with pure Ash implementations in `std/src/list.ash`. Lists become ordinary algebraic data types (`Cons`/`Nil`) rather than opaque runtime primitives. This unblocks Phase 151's deferred QuickCheck combinators and aligns with Ash's principle of minimizing builtins.

## Core Design

```ash
pub type List<T> = Nil | Cons { head: T, tail: List<T> };

pub fn len<T>(list: List<T>) -> Int { ... }
pub fn head<T>(list: List<T>) -> T { ... }
pub fn tail<T>(list: List<T>) -> List<T> { ... }
pub fn append<T>(list: List<T>, item: T) -> List<T> { ... }
pub fn concat<T>(a: List<T>, b: List<T>) -> List<T> { ... }
pub fn map<T, U>(list: List<T>, f: (T) -> U) -> List<U> { ... }
pub fn filter<T>(list: List<T>, predicate: (T) -> Bool) -> List<T> { ... }
```

## Non-Goals

- No tail-call optimization (deferred to future phase)
- No lazy evaluation (deferred to future phase)
- No persistent vector/HAMT (deferred to future phase)
- No parallel list operations (deferred to future phase)

## Decision Gates

| Gate | Decision | Owner task |
|---|---|---|
| D1 | Verify `List<T>` type definition parses and typechecks correctly | TASK-1530 |
| D2 | Implement all 7 core list operations in pure Ash | TASK-1531 |
| D3 | Add new operations (index, take, drop, reverse, prepend) | TASK-1532 |
| D4 | Implement algebraic structures (Applicative, Monad, Foldable, Traversable) | TASK-1533 |
| D5 | Update parser to desugar `[...]` to Cons/Nil | TASK-1534 |
| D6 | Update type checker to handle `List<T>` as constructor | TASK-1535 |
| D7 | Remove `Value::List` from runtime and update evaluation | TASK-1536 |
| D8 | Verify all tests pass and benchmark performance | TASK-1537 |
| D9 | Update dependent tasks (TASK-1511, TASK-1524) | TASK-1538 |
| D10 | Close out with documentation and changelog | TASK-1539 |

## Task Table

| Task | Description | Status |
|---|---|---|
| [TASK-1530](tasks/TASK-1530-list-type-definition-and-parsing.md) | Add `List<T>` type definition to stdlib, verify parsing and typechecking | 📝 Planned |
| [TASK-1531](tasks/TASK-1531-core-list-operations.md) | Implement `len`, `head`, `tail`, `append`, `concat`, `map`, `filter` in pure Ash | 📝 Planned |
| [TASK-1532](tasks/TASK-1532-extended-list-operations.md) | Implement `index`, `take`, `drop`, `reverse`, `prepend` for QuickCheck combinators | 📝 Planned |
| [TASK-1533](tasks/TASK-1533-list-algebraic-structures.md) | Implement Applicative, Monad, Foldable, Traversable instances for List | 📝 Planned |
| [TASK-1534](tasks/TASK-1534-parser-list-literal-desugaring.md) | Update parser to desugar `[...]` syntax to Cons/Nil variants | 📝 Planned |
| [TASK-1535](tasks/TASK-1535-typechecker-list-constructor.md) | Update type checker to handle `List<T>` as ordinary type constructor | 📝 Planned |
| [TASK-1536](tasks/TASK-1536-runtime-remove-list-primitive.md) | Remove `Value::List` from runtime, update evaluation and pattern matching | 📝 Planned |
| [TASK-1537](tasks/TASK-1537-verification-and-benchmarking.md) | Verify all tests pass, run property tests, benchmark performance | 📝 Planned |
| [TASK-1538](tasks/TASK-1538-update-dependent-tasks.md) | Update TASK-1511, TASK-1524, and other dependent tasks with new list primitives | 📝 Planned |
| [TASK-1539](tasks/TASK-1539-phase-153-closeout.md) | Close out Phase 153 with documentation, changelog, and status reconciliation | 📝 Planned |

## Implementation Order

1. TASK-1530: Define the type (foundation)
2. TASK-1531 + TASK-1532: Implement operations (parallel)
3. TASK-1533: Add algebraic structures (depends on operations)
4. TASK-1534: Update parser (depends on type definition)
5. TASK-1535: Update type checker (depends on parser)
6. TASK-1536: Update runtime (depends on type checker)
7. TASK-1537: Verify and benchmark (depends on runtime)
8. TASK-1538: Update dependent tasks (depends on verification)
9. TASK-1539: Close out

## Verification Strategy

Every implementation task must include:
- Focused Rust tests for the changed component
- Property tests for list operations (proptest)
- Algebraic law tests (Functor, Monoid, Applicative, Monad)
- Negative tests for panics (head of empty list, etc.)
- `cargo fmt --check`, `cargo test`, `cargo clippy` gates
- `git diff --check`
- Performance benchmarks

## Closeout Criteria

- All TASK-1530 through TASK-1538 tasks are complete or explicitly marked with honest partial status
- SPEC-089, PLAN-153, and PLAN-INDEX agree on scope/status
- No `Value::List` references remain in runtime
- All list operations are pure Ash functions
- Algebraic structures verified with property tests
- Performance benchmarks show acceptable behavior
- CHANGELOG.md records the migration
- Phase 151/152 tasks updated with new dependencies

## Notes

This phase unblocks Phase 151's TASK-1511 (deferred combinators) by providing:
- `concat` for `append_shrink` and `prepend_shrink`
- `index` for `one_of`
- `take`/`drop` for `recursive` state management

The risk is in TASK-1536 (runtime changes). The `eval.rs` and `small_step.rs` files have extensive list handling. Changing `Value::List` to `Value::Variant` is a breaking change that affects pattern matching, foreach loops, and builtin dispatch.

Performance will degrade from O(1) to O(n) for `len` and `index`, but this is acceptable for the purity gain. Future phases can add TCO and persistent data structures to recover performance.
