# Multi-Shot Continuations for Pure Computations

## Status

Design note. Too raw for spec inclusion.

## Summary

Ash's CPS IR (SPEC-098b) currently enforces **affine** (single-use) continuations. This note explores relaxing this restriction to **multi-shot** (multiple-use) continuations for **pure computations** (empty effect row). This enables non-determinism, backtracking, and other control-flow patterns as effects.

## The Current Restriction

From SPEC-098b §5.3:

> "The `resume` parameter has an affine type: it can be used at most once in the handler body."

Enforced by the type checker. Runtime traps on second use (stopgap) or rejects via affine typing (target).

## The Relaxation

A continuation `k : Cont<A, Ans, ρ, m>` is **multi-shot** (`m = MultiShot`) if:

1. `ρ = {}` — the continuation body is pure (no effects)
2. The handler body is pure, or uses only handler-local state

```text
Γ ⊢ k : Cont<A, Ans, {}, MultiShot>
------------------------------------
Γ ⊢ k can be used multiple times

Γ ⊢ k : Cont<A, Ans, ρ, Affine>  where ρ ≠ {}
------------------------------------
Γ ⊢ k can be used at most once
```

## Why Pure Only?

If `k` is pure, then `k(v1)` and `k(v2)` are **independent**:

```text
k(10) = 11  -- deterministic, no side effects
k(20) = 21  -- deterministic, no side effects
k(10) + k(20) = 32  -- always 32
```

If `k` has effects, then `k(v1)` and `k(v2)` may **interfere**:

```text
k(10) -- writes to log
k(20) -- writes to log again (or reads what k(10) wrote)
```

This is non-deterministic and hard to reason about. Restrict multi-shot to pure to avoid this.

## The Runtime Change

### Current (Affine)

```rust
fn invoke_cont(k: Cont, arg: Value) -> Result {
    // Consume k — mark as used, trap on second use
    k.invoke_once(arg)
}
```

### New (MultiShot for Pure)

```rust
fn invoke_cont_multi(k: Cont, arg: Value) -> Result {
    // Clone the continuation environment and handler chain
    let k_copy = k.clone();
    k_copy.invoke(arg)
}
```

The clone is **deep** for mutable state, **shallow** for immutable data.

### What is Cloned?

A `Cont` value in the CPS IR:

```rust
pub struct Cont {
    pub param: Param,
    pub body: Term,
    pub env: Env,           // captured variable bindings
    pub chain: HandlerChain, // captured handler frames
    pub row: EffectRow,
}
```

Cost: **O(size of env + size of chain)** per clone.

## Optimizations

| Optimization | Mechanism | Cost |
|-------------|-----------|------|
| Persistent env | HAMT for `Env` | O(1) sharing |
| Lazy cloning | Clone only on second invocation | O(1) first, O(n) second |
| Static analysis | Prove single-use, skip clone | O(1) |

## Type System Change

Add `Multiplicity` to `Cont`:

```rust
pub enum Multiplicity {
    Affine,     // use at most once (default)
    MultiShot,  // use multiple times (if pure)
}

pub struct Cont {
    pub arg: Type,
    pub answer: Type,
    pub row: EffectRow,
    pub multiplicity: Multiplicity,  // new
}
```

Inferred from the row:
- `row = {}` → `MultiShot`
- `row ≠ {}` → `Affine`

## Example: Non-Determinism (List Monad)

```ash
effect Choice
  fun choice() : Bool

-- List all possible outcomes
fn all_outcomes(action: {Choice} Int) -> List<Int> {
    on action {
        <choice -> k> => {
            -- k is MultiShot because the computation is pure
            let true_branch = all_outcomes(k(true));
            let false_branch = all_outcomes(k(false));
            true_branch ++ false_branch
        },
        return(v) => [v]
    }
}

fn example() -> List<Int> {
    all_outcomes({
        do {
            x <- choice();
            y <- choice();
            return if x then (if y then 1 else 2) else (if y then 3 else 4)
        }
    })
}
-- Result: [1, 2, 3, 4]
```

The handler explores all branches by calling `k` multiple times.

## Example: Backtracking Search

```ash
effect Search<a>
  fun fail() : a
  fun choose(options: List<a>) : a

-- Depth-first search
fn dfs(action: {Search<a>} a) -> Option<a> {
    on action {
        <fail -> k> => None,  -- k is discarded
        <choose options -> k> => {
            find_first(options, fn(opt) => dfs(k(opt)))
        },
        return(v) => Some(v)
    }
}

fn find_first<T>(xs: List<T>, f: T -> Option<U>) -> Option<U> {
    match xs {
        Nil => None,
        Cons(x, rest) => match f(x) {
            Some(v) => Some(v),
            None => find_first(rest, f)
        }
    }
}
```

The `choose` handler calls `k` multiple times (once per option) until one succeeds.

## The Handler-Local State Problem

When the handler maintains state across invocations:

```ash
-- CORRECT: state built from return values
fn count_invocations(action: {Emit String} Unit) -> Int {
    on action {
        <emit msg -> k> => {
            let rest = count_invocations(k(()));
            1 + rest
        },
        return(()) => 0
    }
}

-- WRONG: mutable state leaks between invocations
fn count_with_var(action: {Emit String} Unit) -> Int {
    let var count = 0;  -- mutable variable
    on action {
        <emit msg -> k> => {
            count := count + 1;  -- mutation!
            k(());  -- if k is called again, count is already incremented
            count
        },
        return(()) => count
    }
}
```

**Rule**: The handler must be pure, or the mutable state must be scoped per invocation.

## Comparison with Haskell

| Haskell | Ash (MultiShot) |
|---------|-----------------|
| `Cont r a` monad | `Cont<A, Ans, {}, MultiShot>` effect |
| `callCC` | `shift` handler |
| `reset` | `reset` handler |
| `m >>= k` | Handler reinstalls on `k` |

The key difference: in Haskell, continuations are always multi-shot (lazy evaluation). In Ash, multi-shot is restricted to pure computations.

## Refined Design Decisions

### 1. Mutable State with Multi-Shot: Already Handled by Effect Rows

Mutable state is an effect. If a handler captures mutable state, its effect row is non-empty. The continuation `k` inherits this row. Therefore, `k` is **already `Affine`** — the multi-shot rule never applies.

```ash
-- REJECTED by type checker (but for a different reason than previously stated)
fn bad_handler(action: {Emit String} Unit) -> Int {
    let var count = 0;  -- mutable state: effect row is {var Int}
    on action {
        <emit msg -> k> => {
            count := count + 1;  -- k's row includes {var Int}, so k is Affine
            k(());               -- OK: k is Affine, used once
            k(())                -- ERROR: k used twice, but k is Affine
        },
        return(()) => count
    }
}
```

The type error is: `"Continuation 'k' is Affine (row includes {var Int}) and cannot be used multiple times."`

**The rule is simpler than I stated:**
- `k` is multi-shot iff `k.row = {}`
- Mutable state makes `k.row ≠ {}`, so `k` is automatically `Affine`
- No special "mutable state check" needed beyond the existing row system

**Workaround:** Thread state through return values (state is then in the return type, not the row):

```ash
-- ACCEPTED: state threaded through return values, no mutable state in handler
fn good_handler(action: {Emit String} Unit) -> Int {
    on action {
        <emit msg -> k> => {
            let rest = good_handler(k(()));  -- state from return value
            1 + rest
        },
        return(()) => 0
    }
}
```

Here, `k` has row `{}` (pure), so `k` is `MultiShot`. The state is in the return type `Int`, not in the effect row.

### 2. Interaction with Lazy/Memo Modes

Ash's target type system (SPEC-097b) includes evaluation modes: `strict`, `lazy`, `memo`. How do these interact with multi-shot continuations?

#### Lazy Multi-Shot = Stream

```ash
-- Lazy evaluation: each invocation produces a value on demand
fn lazy_stream(action: {Yield Int} Unit) -> Stream<Int> {
    on action {
        <yield x -> k> => {
            -- k is lazy: each invocation suspends until demanded
            Stream.Cons(x, fn() => lazy_stream(k(())))
        },
        return(()) => Stream.Nil
    }
}
```

The continuation `k` is **suspended** on each invocation. The stream consumer pulls values one at a time. Multi-shot is natural here: each invocation resumes a suspended computation.

#### Memo Multi-Shot = Cached Stream

```ash
-- Memo evaluation: first invocation computes, subsequent invocations return cached result
fn memo_cached(action: {Yield Int} Unit) -> Stream<Int> {
    on action {
        <yield x -> k> => {
            -- k is memo: first call computes, rest are cached
            let cache = memo(k(()));  -- cache the rest of the stream
            Stream.Cons(x, fn() => cache)
        },
        return(()) => Stream.Nil
    }
}
```

The continuation `k` is **memoized**. The first invocation computes the result and caches it. Subsequent invocations return the cached result. This is only valid if `k` is pure (which it is, by the multi-shot rule).

#### Strict Multi-Shot = Eager List

```ash
-- Strict evaluation: all invocations happen immediately
fn strict_list(action: {Yield Int} Unit) -> List<Int> {
    on action {
        <yield x -> k> => {
            -- k is strict: compute immediately
            Cons(x, strict_list(k(())))
        },
        return(()) => Nil
    }
}
```

The continuation `k` is **strict**: each invocation computes the full result immediately. This is the List monad behavior.

#### Summary Table

| Mode | Multi-Shot Semantics | Use Case |
|------|---------------------|----------|
| `strict` | Eager, all invocations now | List monad, exhaustive search |
| `lazy` | Suspended, on-demand | Streams, generators, iterators |
| `memo` | Cached, first computes | Dynamic programming, memoized search |

### 3. Static Purity Proof

Yes, the type checker can prove purity statically. The proof is the **empty effect row**:

```text
Γ ⊢ k : Cont<A, Ans, {}, MultiShot>
------------------------------------
Γ ⊢ k is pure (no effects in its body)
```

The empty row `ρ = {}` means the continuation body has no effects: no capability calls, no resource access, no mutation, no IO. This is a **syntactic** proof, not a semantic one. The type checker doesn't need to analyze the body — the row tells it everything.

**No purity annotation needed.** The row is the proof.

### 4. Opt-In Multi-Shot

All continuations default to `Affine`, regardless of purity. The user must explicitly opt into `MultiShot`:

```ash
-- Default: k is Affine (single-use)
fn default_handler(action: {Choice} Int) -> List<Int> {
    on action {
        <choice -> k> => { ... k(true) ... }  -- k can only be used once
    }
}

-- Explicit opt-in: k is MultiShot (multiple-use)
fn multi_handler(action: {Choice} Int) -> List<Int> {
    on action multi {
        <choice -> k> => {
            let true_branch = multi_handler(k(true));   -- first use
            let false_branch = multi_handler(k(false)); -- second use
            true_branch ++ false_branch
        },
        return(v) => [v]
    }
}
```

**Rationale:**
- Uniform default across all continuations (pure or effectful)
- No surprise: the user knows exactly when multi-shot is available
- Explicit is safer: the user must think about the cost and semantics
- The type checker still enforces: if `multi` is used but the row is non-empty, it's a type error

```text
Γ ⊢ action : {cap fs.read} Int
Γ ⊢ on action multi { ... }        -- ERROR: row is non-empty, multi not allowed
```

**The rule is simple:**
- Default: `Affine` (all continuations)
- Opt-in: `multi` (only if row is empty)
- The type checker verifies the row is empty before allowing `multi`

## Computationally Efficient Backtracking Search

Given the three dimensions (handler style, computation mode, multi-shot), what is the most efficient combination for backtracking search?

### The Dimensions

| Dimension | Options | Impact on Search |
|-----------|---------|------------------|
| Handler style | Koka-style (`resume`) vs Frank-style (`k`) | Frank-style is more natural for recursive search; Koka-style is fine but implicit continuation capture may add overhead |
| Computation mode | `strict`, `lazy`, `memo` | `strict` for all solutions; `lazy` for first solution; `memo` for dynamic programming |
| Multi-shot | `Affine` (default) vs `multi` (opt-in) | `multi` is essential for exploring alternatives; `Affine` requires manual state threading |

### The Search Operations

Backtracking search involves three operations:
1. **Choose**: branch and explore alternatives
2. **Fail**: backtrack to the previous choice point
3. **Success**: return a solution

### Efficient Combinations by Strategy

#### Strategy 1: Find the First Solution (Lazy)

**Best combination:** Frank-style + `lazy` + `multi`

```ash
effect Search<a>
  fun fail() : a
  fun choose(options: List<a>) : a

-- Lazy: find first solution, suspend rest
fn lazy_first(action: {Search<a>} a) -> Option<a> {
    on action lazy multi {  -- lazy suspension + multi-shot
        <fail -> k> => None,
        <choose options -> k> => {
            -- Try each option lazily
            find_first_lazy(options, fn(opt) => lazy_first(k(opt)))
        },
        return(v) => Some(v)
    }
}

fn find_first_lazy<T>(xs: List<T>, f: T -> Option<U>) -> Option<U> {
    match xs {
        Nil => None,
        Cons(x, rest) => match f(x) {
            Some(v) => Some(v),           -- found! don't explore rest
            None => find_first_lazy(rest, f)  -- lazy: only evaluate if needed
        }
    }
}
```

**Why efficient:**
- `lazy` suspends unexplored branches. If the first branch succeeds, no other branches are evaluated.
- `multi` allows `k` to be called multiple times (once per option).
- The first solution is found with minimal work.

**Cost:** O(depth of first solution) time, O(depth) space for suspensions.

---

#### Strategy 2: Find All Solutions (Strict)

**Best combination:** Frank-style + `strict` + `multi`

```ash
-- Strict: find all solutions eagerly
fn strict_all(action: {Search<a>} a) -> List<a> {
    on action strict multi {  -- strict evaluation + multi-shot
        <fail -> k> => Nil,
        <choose options -> k> => {
            -- Explore all options eagerly
            flat_map(options, fn(opt) => strict_all(k(opt)))
        },
        return(v) => [v]
    }
}
```

**Why efficient:**
- `strict` evaluates all branches immediately. No suspension overhead.
- `multi` allows `k` to be called for each option.
- All solutions are collected in a single pass.

**Cost:** O(total search tree size) time, O(depth) space for recursion stack.

---

#### Strategy 3: Memoized Search (Dynamic Programming)

**Best combination:** Frank-style + `memo` + `multi`

```ash
-- Memo: cache subproblem results
fn memo_search(action: {Search<a>} a, cache: Map<State, List<a>>) -> List<a> {
    on action memo multi {  -- memoization + multi-shot
        <fail -> k> => Nil,
        <choose options -> k> => {
            -- Check cache before exploring
            match cache.get(current_state()) {
                Some(result) => result,  -- cache hit!
                None => {
                    let result = flat_map(options, fn(opt) => memo_search(k(opt), cache));
                    cache.insert(current_state(), result);  -- cache miss: store result
                    result
                }
            }
        },
        return(v) => [v]
    }
}
```

**Why efficient:**
- `memo` caches subproblem results. Each subproblem is solved only once.
- `multi` allows `k` to be called for each option.
- Avoids recomputation of identical subproblems.

**Cost:** O(number of unique subproblems × cost per subproblem) time, O(number of unique subproblems) space for cache.

---

### Comparison Table

| Strategy | Handler | Mode | Multi-Shot | Time | Space | Use Case |
|----------|---------|------|------------|------|-------|----------|
| First solution | Frank | `lazy` | `multi` | O(depth of first) | O(depth) | Constraint solving, parsing |
| All solutions | Frank | `strict` | `multi` | O(tree size) | O(depth) | Enumeration, counting |
| Memoized | Frank | `memo` | `multi` | O(unique subproblems) | O(unique subproblems) | DP, knapsack, shortest path |

### Key Insight

> **Frank-style + `multi` is the common foundation.** The handler is a recursive function that reinstalls itself on each branch. The choice of mode (`lazy`/`strict`/`memo`) determines the evaluation strategy, which is orthogonal to the handler style.

> **Koka-style is less efficient for deep backtracking** because the implicit continuation capture on each `resume` adds overhead. Frank-style's explicit `k` is just a function argument — no capture needed.

> **Without `multi`, backtracking is possible but inefficient.** You'd need to manually thread the search state through the computation, which adds allocation and indirection.

### Example: N-Queens (First Solution)

```ash
fn n_queens(n: Int) -> Option<List<(Int, Int)>> {
    lazy_first({
        do {
            place_queens(n, 1, [])
        }
    })
}

fn place_queens(n: Int, row: Int, placed: List<(Int, Int)>) -> {Search} List<(Int, Int)> {
    if row > n then return placed;
    
    let cols = range(1, n + 1);
    let valid = filter(cols, fn(col) => is_safe(placed, row, col));
    
    if empty(valid) then fail();
    
    let col = choose(valid);  -- branch here
    place_queens(n, row + 1, Cons((row, col), placed))
}
```

With `lazy` + `multi`, the first valid placement is found without exploring all alternatives. The `choose` handler tries columns lazily — if the first column leads to a solution, the rest are never evaluated.

### Example: N-Queens (All Solutions)

```ash
fn n_queens_all(n: Int) -> List<List<(Int, Int)>> {
    strict_all({
        do {
            place_queens(n, 1, [])
        }
    })
}
```

With `strict` + `multi`, all valid placements are collected eagerly. The `choose` handler explores all columns, and each branch is fully evaluated.

### Summary

| Goal | Best Combination | Why |
|------|------------------|-----|
| Find first solution | Frank + `lazy` + `multi` | Suspends unexplored branches; minimal work |
| Find all solutions | Frank + `strict` + `multi` | Eager exploration; no suspension overhead |
| Memoized search | Frank + `memo` + `multi` | Caches subproblems; avoids recomputation |

The common thread: **Frank-style + `multi`** is the foundation. The mode (`lazy`/`strict`/`memo`) is chosen based on the search strategy.

## Open Questions

1. ~~Should the compiler warn when a multi-shot continuation is used in a handler with mutable state?~~ → **Type error**
2. ~~How does this interact with lazy/memo modes?~~ → **Explored above**
3. ~~Can we statically prove that a handler is pure, or do we need a purity annotation?~~ → **Empty row is proof**
4. ~~What is the cost model for cloning? Should the user be able to opt out?~~ → **Opt-out via `once` annotation**

## Changelog

- 2026-06-20: Created design note exploring multi-shot continuations for pure computations in Ash's CPS IR.
