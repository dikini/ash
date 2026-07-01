# SPEC-089: Move List Builtins to Standard Library

**Status:** Implemented MVP (Phase 153; Phase 176 runtime cleanup removed legacy `Value::List`)
**Date:** 2026-06-17
**Amends:** [SPEC-031](SPEC-031-FIRST-CLASS-FUNCTIONS.md), [SPEC-072](SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md)
**Builds on:** [SPEC-088](SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md), [ASSESSMENT-001](../assessments/ASSESSMENT-001-NATIVE-LIST-IMPLEMENTATION.md)
**Plan:** [PLAN-153](../plan/PLAN-153-LIST-BUILTIN-TO-STDLIB.md)

## 1. Summary

Replace Rust-implemented list builtins with pure Ash implementations in `std/src/list.ash`. Lists become ordinary algebraic data types (`Cons`/`Nil`) rather than opaque runtime primitives. This aligns with Ash's principle of minimizing builtins and maximizing expressiveness in the language itself.

## 2. Motivation

Currently, lists are magic primitives:

```rust
// crates/ash-core/src/value.rs
pub enum Value {
    List(Box<Vec<Value>>),  // Opaque Rust primitive
}
```

This creates a two-tier system where lists have special status. Moving lists to ordinary ADTs:
- Eliminates builtin magic (fewer builtins = simpler runtime)
- Enables user-defined list-like types with identical syntax
- Makes list operations first-class Ash code (inspectable, extensible)
- Aligns with the tower principle: Pure operations should be expressible in Pure Ash

## 3. Core Design

### 3.1 List as Algebraic Data Type

```ash
// std/src/list.ash — new type definition
pub type List<T> = Nil | Cons { head: T, tail: List<T> };
```

The `[1, 2, 3]` literal syntax desugars to:
```ash
Cons { head: 1, tail: Cons { head: 2, tail: Cons { head: 3, tail: Nil } } }
```

### 3.2 Pattern Matching on Lists

```ash
match list {
    Nil => ...,
    Cons { head: h, tail: rest } => ...,
}
```

Or with shorthand syntax:
```ash
match list {
    [] => ...,
    [head, ..tail] => ...,
}
```

### 3.3 Native List Operations (Pure Ash)

```ash
pub fn len<T>(list: List<T>) -> Int {
    match list {
        Nil => 0,
        Cons { head: _, tail: rest } => 1 + len(rest)
    }
}

pub fn head<T>(list: List<T>) -> T {
    match list {
        Cons { head: h, tail: _ } => h,
        Nil => panic("head of empty list")
    }
}

pub fn tail<T>(list: List<T>) -> List<T> {
    match list {
        Cons { head: _, tail: t } => t,
        Nil => panic("tail of empty list")
    }
}

pub fn append<T>(list: List<T>, item: T) -> List<T> {
    match list {
        Nil => Cons { head: item, tail: Nil },
        Cons { head: h, tail: rest } => Cons { head: h, tail: append(rest, item) }
    }
}

pub fn concat<T>(a: List<T>, b: List<T>) -> List<T> {
    match a {
        Nil => b,
        Cons { head: h, tail: rest } => Cons { head: h, tail: concat(rest, b) }
    }
}

pub fn map<T, U>(list: List<T>, f: (T) -> U) -> List<U> {
    match list {
        Nil => Nil,
        Cons { head: h, tail: rest } => Cons { head: f(h), tail: map(rest, f) }
    }
}

pub fn filter<T>(list: List<T>, predicate: (T) -> Bool) -> List<T> {
    match list {
        Nil => Nil,
        Cons { head: h, tail: rest } => {
            if predicate(h) {
                Cons { head: h, tail: filter(rest, predicate) }
            } else {
                filter(rest, predicate)
            }
        }
    }
}

// New operations needed for QuickCheck combinators
pub fn index<T>(list: List<T>, n: Int) -> T {
    match list {
        Nil => panic("index out of bounds"),
        Cons { head: h, tail: rest } => {
            if n == 0 { h } else { index(rest, n - 1) }
        }
    }
}

pub fn take<T>(n: Int, list: List<T>) -> List<T> {
    if n <= 0 { Nil }
    else {
        match list {
            Nil => Nil,
            Cons { head: h, tail: rest } => Cons { head: h, tail: take(n - 1, rest) }
        }
    }
}

pub fn drop<T>(n: Int, list: List<T>) -> List<T> {
    if n <= 0 { list }
    else {
        match list {
            Nil => Nil,
            Cons { head: _, tail: rest } => drop(n - 1, rest)
        }
    }
}

pub fn reverse<T>(list: List<T>) -> List<T> {
    reverse_acc(list, Nil)
}

fn reverse_acc<T>(list: List<T>, acc: List<T>) -> List<T> {
    match list {
        Nil => acc,
        Cons { head: h, tail: rest } => reverse_acc(rest, Cons { head: h, tail: acc })
    }
}

pub fn prepend<T>(item: T, list: List<T>) -> List<T> {
    Cons { head: item, tail: list }
}
```

## 4. Runtime Changes

### 4.1 Remove `Value::List` Primitive

```rust
// Remove from crates/ash-core/src/value.rs
pub enum Value {
    // ... remove this:
    // List(Box<Vec<Value>>),
    // ... keep everything else
}
```

### 4.2 List Literals Desugar to Variants

The parser changes `[1, 2, 3]` from:
```rust
Expr::Literal(Value::List(Box::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)])))
```

To:
```rust
Expr::Variant {
    name: "Cons",
    fields: vec![
        ("head", Expr::Literal(Value::Int(1))),
        ("tail", Expr::Variant {
            name: "Cons",
            fields: vec![
                ("head", Expr::Literal(Value::Int(2))),
                ("tail", Expr::Variant {
                    name: "Cons",
                    fields: vec![
                        ("head", Expr::Literal(Value::Int(3))),
                        ("tail", Expr::Variant { name: "Nil", fields: vec![] })
                    ]
                })
            ]
        })
    ]
}
```

### 4.3 Remove List Builtins from Dispatch Table

Remove from `crates/ash-interp/src/eval/builtins.rs`:
- `list::len`
- `list::head`
- `list::tail`
- `list::append`
- `list::concat`
- `list::filter`
- `list::map`

### 4.4 Type Representation

Replace `Type::List(Box<Type>)` with `Type::Constructor("List", [T])` or keep as alias.

## 5. Algebraic Structure Implementations

Lists implement standard algebraic structures. These must be verified after the migration:

### 5.1 Functor (already exists)

```ash
pub impl Functor<List> {
    map(value, f) = list::map(value, f)
}
```

### 5.2 Semigroup (already exists)

```ash
pub impl <A : *> Semigroup<List<A>> {
    append(left, right) = list::concat(left, right)
}
```

### 5.3 Monoid (already exists)

```ash
pub impl <A : *> Monoid<List<A>> {
    empty() = Nil
    append(left, right) = list::concat(left, right)
}
```

### 5.4 Applicative (to implement)

```ash
pub impl Applicative<List> {
    pure(x) = Cons { head: x, tail: Nil }
    
    apply(f_list, x_list) = {
        match f_list {
            Nil => Nil,
            Cons { head: f, tail: rest_f } => {
                concat(map(x_list, f), apply(rest_f, x_list))
            }
        }
    }
}
```

### 5.5 Monad (to implement)

```ash
pub impl Monad<List> {
    unit(x) = Cons { head: x, tail: Nil }
    
    bind(list, f) = {
        match list {
            Nil => Nil,
            Cons { head: h, tail: rest } => {
                concat(f(h), bind(rest, f))
            }
        }
    }
}
```

### 5.6 Foldable (to implement)

```ash
pub interface Foldable<F : * -> *> {
    foldl(F<A>, B, (B, A) -> B) -> B
    foldr(F<A>, B, (A, B) -> B) -> B
}

pub impl Foldable<List> {
    foldl(list, init, f) = {
        match list {
            Nil => init,
            Cons { head: h, tail: rest } => foldl(rest, f(init, h), f)
        }
    }
    
    foldr(list, init, f) = {
        match list {
            Nil => init,
            Cons { head: h, tail: rest } => f(h, foldr(rest, init, f))
        }
    }
}
```

### 5.7 Traversable (to implement)

```ash
pub interface Traversable<T : * -> *> where T: Foldable, T: Functor {
    traverse(T<A>, A -> F<B>) -> F<T<B>>
    sequence(T<F<A>>) -> F<T<A>>
}

pub impl Traversable<List> {
    sequence(list) = {
        match list {
            Nil => pure(Nil),
            Cons { head: h, tail: rest } => {
                apply(map(h, |x| -> |xs| -> Cons { head: x, tail: xs }), sequence(rest))
            }
        }
    }
    
    traverse(list, f) = sequence(map(list, f))
}
```

## 6. Performance Considerations

| Operation | Vec (current) | Cons list (new) | Impact |
|-----------|---------------|-----------------|--------|
| `len` | O(1) | O(n) | Significant for large lists |
| `head` | O(1) | O(1) | No change |
| `tail` | O(1) | O(1) | No change |
| `append` | O(1) amortized | O(n) | Significant |
| `concat` | O(n) | O(n) | Same |
| `map` | O(n) | O(n) | Same |
| `filter` | O(n) | O(n) | Same |
| `index` | O(1) | O(n) | Significant |
| `reverse` | O(n) | O(n) | Same |
| Memory | Contiguous | Linked cells | ~2x overhead per element |

### Mitigation Strategies

1. **Tail-call optimization**: Essential for recursive list operations
2. **Lazy evaluation**: For `map`/`filter` chains (future work)
3. **Benchmarking**: Establish performance baselines before migration
4. **Profile-guided optimization**: Identify hot paths

## 7. Risk Assessment

### High Risk: Runtime Evaluation Changes

The `eval.rs` and `small_step.rs` files have extensive list handling. Changing `Value::List` to `Value::Variant` affects:
- Pattern matching exhaustiveness
- `foreach` loop execution
- List literal evaluation
- Builtin dispatch

### Medium Risk: Type Checker Changes

`Type::List(Box<Type>)` is used throughout `ash-typeck`. Migration requires:
- Type unification rules
- Pattern type inference
- Generic instantiation

### Low Risk: Parser Changes

List literal parsing is localized. The change is mechanical.

## 8. Verification Strategy

### 8.1 Property Tests

```rust
proptest! {
    fn list_cons_len_inverse(list in arb_list::<i64>()) {
        // len(Cons(x, list)) == 1 + len(list)
    }
    
    fn concat_associative(a in arb_list(), b in arb_list(), c in arb_list()) {
        // concat(concat(a, b), c) == concat(a, concat(b, c))
    }
    
    fn map_identity(list in arb_list::<i64>()) {
        // map(list, |x| -> x) == list
    }
    
    fn filter_idempotent(list in arb_list::<i64>()) {
        // filter(filter(list, p), p) == filter(list, p)
    }
}
```

### 8.2 Algebraic Law Tests

```rust
proptest! {
    fn monoid_left_identity(list in arb_list::<i64>()) {
        // concat(Nil, list) == list
    }
    
    fn monoid_right_identity(list in arb_list::<i64>()) {
        // concat(list, Nil) == list
    }
    
    fn functor_composition(list in arb_list::<i64>()) {
        // map(map(list, f), g) == map(list, |x| -> g(f(x)))
    }
}
```

### 8.3 Negative Tests

- `head(Nil)` must panic
- `tail(Nil)` must panic
- `index(list, len(list))` must panic
- Type mismatch in list operations

## 9. Acceptance Criteria

### C89-1: List type is ordinary ADT

```ash
type List<T> = Nil | Cons { head: T, tail: List<T> };
let xs = Cons { head: 1, tail: Cons { head: 2, tail: Nil } };
assert len(xs) == 2;
```

### C89-2: List literal syntax works

```ash
let xs = [1, 2, 3];
assert head(xs) == 1;
assert len(xs) == 3;
```

### C89-3: All list operations are pure Ash

No `builtin` declarations in `std/src/list.ash` for list operations.

### C89-4: Algebraic structures verified

```ash
// Functor laws
assert map([1, 2, 3], |x| -> x) == [1, 2, 3];

// Monoid laws
assert concat([], [1, 2]) == [1, 2];
assert concat([1, 2], []) == [1, 2];
```

### C89-5: No `Value::List` in runtime

`grep -r "Value::List" crates/ash-core/src/` returns no matches.

### C89-6: Performance baseline established

Benchmarks show acceptable performance for typical use cases (≤1000 elements).

## 10. Deferred Items

| Item | Reason | Future Work |
|------|--------|-------------|
| Lazy lists | Requires lazy evaluation infrastructure | Phase for streams/iterators |
| Tail-call optimization | Required for large list performance | Phase for TCO |
| Persistent vector (HAMT) | Alternative to Cons list for O(1) indexing | Phase for advanced collections |
| Parallel list operations | Requires `Par`/`scatter`/`gather` | Phase for parallel stdlib |

## 11. Relationship to Other Specs

| Spec | Relationship |
|------|-------------|
| SPEC-031 | Amends: closures work with Cons lists |
| SPEC-072 | Consistent: list operations are pure |
| SPEC-088 | Enables: closure capture in list operations |
| ASSESSMENT-001 | Builds on: pure approach selected |

## 12. Implementation Notes

### Order of Changes

1. Add `List<T>` type definition to `std/src/list.ash`
2. Implement all list operations in pure Ash
3. Add algebraic structure implementations
4. Update parser to desugar `[...]` to `Cons`/`Nil`
5. Update type checker to handle `List<T>` as constructor
6. Update runtime to remove `Value::List`
7. Update pattern matching for list patterns
8. Verify all tests pass
9. Benchmark and optimize

### Files to Modify

| File | Change |
|------|--------|
| `crates/ash-core/src/value.rs` | Remove `Value::List`, add `List` type registration |
| `crates/ash-core/src/types.rs` | Update `Type::List` handling |
| `crates/ash-parser/src/surface.rs` | Update list literal parsing |
| `crates/ash-parser/src/parse_expr.rs` | Desugar `[...]` to Cons/Nil |
| `crates/ash-typeck/src/types.rs` | Handle `List<T>` as type constructor |
| `crates/ash-interp/src/eval.rs` | Remove list builtin handling |
| `crates/ash-interp/src/eval/builtins.rs` | Remove list builtins from dispatch table |
| `crates/ash-interp/src/small_step.rs` | Update list iteration |
| `std/src/list.ash` | Implement all operations in pure Ash |
| `std/src/algebra/*.ash` | Add Applicative, Monad, Foldable, Traversable instances |

## 13. Closeout Criteria

- [ ] C89-1 through C89-6 all pass
- [ ] No `Value::List` references remain in runtime
- [ ] All list operations are pure Ash functions
- [ ] Algebraic structures (Functor, Monoid, Applicative, Monad, Foldable, Traversable) verified
- [ ] Performance benchmarks show acceptable behavior
- [ ] PLAN-153 and PLAN-INDEX updated
- [ ] CHANGELOG.md records the migration
- [ ] Phase 151/152 tasks updated with new dependencies
