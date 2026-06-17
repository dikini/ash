# Assessment: Native Ash List Implementation vs. Rust Builtin Primitives

## Date: 2026-06-17
## Context: Phase 151 blocker analysis for TASK-1511
## Scope: List concatenation, indexing, and other list primitives

---

## Current State

Lists in Ash are **Rust primitives** implemented as `Vec<Value>` in the runtime:

```rust
// crates/ash-core/src/value.rs:93
pub enum Value {
    // ...
    List(Box<Vec<Value>>),  // Rust Vec primitive
    // ...
}
```

### Current List Builtins (Rust-implemented)

| Builtin | Arity | Status | Location |
|---------|-------|--------|----------|
| `list::len` | 1 | ✅ Implemented | `ash-interp/src/eval/builtins.rs` |
| `list::head` | 1 | ✅ Implemented | `ash-interp/src/eval/builtins.rs` |
| `list::tail` | 1 | ✅ Implemented | `ash-interp/src/eval/builtins.rs` |
| `list::append` | 2 | ✅ Implemented | `ash-interp/src/eval/builtins.rs` |
| `list::concat` | 2 | ✅ Implemented | `ash-interp/src/eval/builtins.rs` |
| `list::filter` | 2 | ✅ Implemented | `ash-interp/src/eval/builtins.rs` |
| `list::map` | 2 | ✅ Implemented | `ash-interp/src/eval/builtins.rs` |

### Stdlib Surface

```ash
// std/src/list.ash
pub builtin fn len<a>(list: List<a>) -> Int;
pub builtin fn head<a>(list: List<a>) -> a;
pub builtin fn tail<a>(list: List<a>) -> List<a>;
pub builtin fn append<a>(list: List<a>, item: a) -> List<a>;
pub builtin fn concat<a>(a: List<a>, b: List<a>) -> List<a>;
pub builtin fn filter<a>(list: List<a>, predicate: (a) -> Bool) -> List<a>;
pub builtin fn map<a, b>(list: List<a>, f: (a) -> b) -> List<b>;
```

All are `pub builtin` — declared in Ash, implemented in Rust.

---

## What "Native Ash Implementation" Would Mean

A native Ash list implementation would define lists and their operations **in Ash itself**, not as Rust builtins. For example:

```ash
// Hypothetical native list implementation

type List<T> = Nil | Cons { head: T, tail: List<T> };

pub fn len<T>(list: List<T>) -> Int {
    match list {
        Nil => 0,
        Cons { head: _, tail: rest } => 1 + len(rest)
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
```

---

## Effort Assessment: Native Ash Implementation

### What Would Need to Change

| Layer | Current | Native Ash | Effort |
|-------|---------|-----------|--------|
| **Value representation** | `Value::List(Box<Vec<Value>>)` | `Value::Variant` (Cons/Nil) | Medium — change core enum |
| **Literal syntax** | `[1, 2, 3]` desugars to `Vec` | `[1, 2, 3]` desugars to `Cons` chain | Medium — parser change |
| **Pattern matching** | `match` on variants | `match` on Cons/Nil | Low — already works |
| **Type checker** | `Type::List(Box<Type>)` | `Type::Constructor("List", [T])` | Medium — type representation |
| **Builtin dispatch** | 7 list builtins in Rust | Remove all, use Ash functions | Medium — remove builtins |
| **Performance** | O(1) `len`, O(1) indexing | O(n) `len`, O(n) indexing | N/A — semantic change |
| **Memory** | Contiguous Vec | Linked list (Cons cells) | N/A — semantic change |

### Detailed Effort Breakdown

#### 1. Value Representation (2-3 days)

**Option A: Full replacement**
- Remove `Value::List(Box<Vec<Value>>)`
- Use `Value::Variant` with `Cons`/`Nil` constructors
- Update all pattern matching on lists
- Update serialization/deserialization

**Option B: Dual representation (hybrid)**
- Keep `Value::List` for runtime efficiency
- Add `List<T>` as a type alias or wrapper type
- Implement operations in Ash, but use Rust primitives under the hood
- **This is the pragmatic approach**

#### 2. Parser Changes (1-2 days)

- `[1, 2, 3]` currently desugars to `Value::List(Box::new(vec![...]))`
- Would need to desugar to `Cons(1, Cons(2, Cons(3, Nil)))`
- Or: keep `Value::List` syntax but allow Ash functions to operate on it

#### 3. Type Checker Changes (2-3 days)

- `Type::List(Box<Type>)` is a primitive type
- Would need to either:
  - Keep `Type::List` but allow Ash functions to have `List<T>` parameters
  - Or replace with `Type::Constructor("List", [T])`

#### 4. Stdlib Rewrite (1 day)

- Replace `pub builtin fn ...` with `pub fn ...` in `std/src/list.ash`
- Implement all operations in Ash

#### 5. Runtime Builtin Removal (1-2 days)

- Remove 7 list builtins from `builtin_dispatch_table()`
- Remove list builtin implementations from `eval/builtins.rs`
- Ensure Ash functions are called instead

#### 6. Testing (2-3 days)

- Update all tests that use `Value::List` directly
- Update property tests for lists
- Verify performance is acceptable

**Total effort: 10-15 days** for a full native implementation.

---

## Pros and Cons

### Pros of Native Ash Implementation

| Pro | Explanation |
|-----|-------------|
| **Language purity** | Lists are "just" ADTs, not magic primitives |
| **User-defined variants** | Users can define their own list-like types with same syntax |
| **Pattern matching** | `match` works naturally on Cons/Nil |
| **No builtin magic** | Fewer builtins = simpler runtime |
| **Extensibility** | Users can write their own list functions |
| **Formal semantics** | Easier to specify formally (algebraic data type) |

### Cons of Native Ash Implementation

| Con | Explanation |
|-----|-------------|
| **Performance** | O(n) `len`, O(n) indexing vs O(1) for Vec |
| **Memory overhead** | Cons cells have pointer overhead vs contiguous Vec |
| **Stack depth** | Recursive list functions risk stack overflow |
| **Breaking change** | All existing code using `Value::List` breaks |
| **Effort** | 10-15 days of implementation work |
| **Runtime complexity** | Need tail-call optimization or iteration primitives |
| **Indexing** | `list[i]` becomes O(n) instead of O(1) |

---

## Pragmatic Alternative: Hybrid Approach

Instead of full native implementation, a **hybrid approach** keeps Rust primitives for performance but exposes them through Ash-native interfaces:

```ash
// std/src/list.ash — hybrid approach

// Keep Rust primitives for performance
pub builtin fn len<a>(list: List<a>) -> Int;
pub builtin fn head<a>(list: List<a>) -> a;
pub builtin fn tail<a>(list: List<a>) -> List<a>;
pub builtin fn append<a>(list: List<a>, item: a) -> List<a>;
pub builtin fn concat<a>(a: List<a>, b: List<a>) -> List<a>;

// But add native Ash functions that compose them
pub fn prepend<a>(item: a, list: List<a>) -> List<a> {
    concat([item], list)
}

pub fn reverse<a>(list: List<a>) -> List<a> {
    reverse_acc(list, [])
}

fn reverse_acc<a>(list: List<a>, acc: List<a>) -> List<a> {
    match list {
        [] => acc,
        [head, ..tail] => reverse_acc(tail, append(acc, head))
    }
}

pub fn take<a>(n: Int, list: List<a>) -> List<a> {
    if n <= 0 {
        []
    } else {
        match list {
            [] => [],
            [head, ..tail] => concat([head], take(n - 1, tail))
        }
    }
}

pub fn drop<a>(n: Int, list: List<a>) -> List<a> {
    if n <= 0 {
        list
    } else {
        match list {
            [] => [],
            [_, ..tail] => drop(n - 1, tail)
        }
    }
}

pub fn index<a>(list: List<a>, n: Int) -> a {
    match list {
        [] => panic("index out of bounds"),
        [head, ..tail] => {
            if n == 0 {
                head
            } else {
                index(tail, n - 1)
            }
        }
    }
}
```

### Hybrid Approach Effort: 2-3 days

| Task | Effort |
|------|--------|
| Add native list functions to stdlib | 1 day |
| Add pattern matching support for list literals | 1 day |
| Add `[]` and `[head, ..tail]` syntax | 1 day |
| Testing | 1 day |

**Total: 2-3 days** (vs 10-15 for full native)

---

## Recommendation

### Short-term (Phase 151/152): Hybrid Approach

**Keep Rust primitives** for `len`, `head`, `tail`, `append`, `concat`, `filter`, `map` but **add native Ash functions** that compose them.

This unblocks TASK-1511 immediately:
- `append_shrink` can use `concat` builtin
- `prepend_shrink` can use `concat` builtin
- `one_of` can use `index` (native Ash function using `head`/`tail`)
- `recursive` can use `GenContext` state

### Medium-term (Future Phase): Full Native Implementation

**Revisit** when:
- Tail-call optimization is implemented
- Performance benchmarking shows Vec is a bottleneck
- Formal semantics work requires algebraic list types
- User-defined collection types need first-class status

### Immediate Action for TASK-1511

Add these **native Ash list functions** to `std/src/list.ash`:

```ash
pub fn prepend<a>(item: a, list: List<a>) -> List<a> {
    concat([item], list)
}

pub fn index<a>(list: List<a>, n: Int) -> a {
    // Using head/tail recursion
    if n == 0 {
        head(list)
    } else {
        index(tail(list), n - 1)
    }
}

pub fn take<a>(n: Int, list: List<a>) -> List<a> {
    // ...
}

pub fn drop<a>(n: Int, list: List<a>) -> List<a> {
    // ...
}
```

**Effort: 1-2 days**
**Unblocks: `append_shrink`, `prepend_shrink`, `one_of`, `recursive`**

---

## Conclusion

| Approach | Effort | Performance | Purity | Recommendation |
|----------|--------|-------------|--------|----------------|
| **Keep Rust builtins** | 0 days | O(1) ops | Low | Current state |
| **Hybrid (native Ash + builtins)** | 2-3 days | O(1) core, O(n) derived | Medium | **Recommended now** |
| **Full native (Cons/Nil)** | 10-15 days | O(n) all ops | High | Future phase |

The **hybrid approach** gives us the best of both worlds: native Ash expressiveness for combinator authors, while keeping Rust performance for core operations. It unblocks TASK-1511 with minimal effort and risk.
