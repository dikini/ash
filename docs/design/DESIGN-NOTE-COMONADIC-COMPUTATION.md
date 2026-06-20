# Comonadic Computation and Co-Comprehensions in Ash

## Status

Design note. Explores the comonadic dual of monadic `do` notation and list comprehensions for stream-based, context-dependent computation in Ash.

## Summary

Ash has `do` notation for monadic sequencing and `[]` comprehensions for applicative collection-building. The missing piece is **comonadic computation** for streams, signals, and data flow — where the natural operation is **contextual observation**, not sequencing or combination.

This note proposes:
- `observe` blocks as the dual of `do` (comonadic context binding)
- `{}` co-comprehensions as the dual of `[]` (context decomposition)
- The categorical foundation: comonads, co-applicatives, and cokleisli composition

## References

- [SPEC-054: Generalized Typed Do-Notation](../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md) — monadic `do:K` syntax
- [SPEC-055: Monad Comprehension Syntax](../spec/SPEC-055-MONAD-COMPREHENSION-SYNTAX.md) — `[]` comprehensions for applicative functors
- [SPEC-079: Standard Algebra — Comonad and Kleisli Helpers](../spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md) — comonad interface and laws
- [DESIGN-032: Monad Comprehension Syntax](DESIGN-032-MONAD-COMPREHENSION-SYNTAX.md) — design rationale for comprehensions
- [DESIGN-031: Generalized Do-Notation](DESIGN-031-GENERALIZED-DO-NOTATION.md) — design rationale for `do:K`
- [multi-shot-continuations.md](multi-shot-continuations.md) — multi-shot continuations for pure computations
- [effect-handling-styles.md](effect-handling-styles.md) — Koka-style vs Frank-style effect handlers
- [effectful-stream-sinks.md](effectful-stream-sinks.md) — effectful stream sinks and comonad-monad bridges

## The Categorical Four-Square

| Structure | Operation | Identity | Composition | Syntax |
|-----------|-----------|----------|-------------|--------|
| **Monad** | `bind` (>>=) | `return` | Kleisli (>=>) | `do { x <- action; ... }` |
| **Applicative** | `apply` (<*>) | `pure` | `liftA2` | `[ f(x,y) | x <- xs, y <- ys ]` |
| **Comonad** | `extend` (=>) | `extract` | Cokleisli (<=>) | `observe s { let x = head; ... }` |
| **Co-applicative** | `zip` | `extract` | `zipWith` | `{ f(x,y) | x = head, y = tail.head }` |

## The Duality

### Monadic: "What happens next?"

```ash
-- do: sequence effects, thread state forward
do {
    x <- read_sensor();      -- effectful action
    y <- read_sensor();      -- next effect
    return f(x, y)            -- combine results
}
```

The bind operator `>>=` threads the **result** of one action into the next.

### Comonadic: "What is the context?"

```ash
-- observe: spread context, observe surroundings
observe stream {
    let x = head;             -- current value
    let y = tail.head;        -- next value
    let z = tail.tail.head;   -- future value
    return f(x, y, z)         -- context-aware result
}
```

The extend operator `=>` spreads a **context-aware function** over the entire stream.

### Applicative: "What are the combinations?"

```ash
-- []: build combinations from independent sources
[ f(x, y) | x <- xs, y <- ys, x > 1 ]
```

The applicative functor combines **independent** values.

### Co-applicative: "What are the shared observations?"

```ash
-- {}: decompose context from shared observations
{ f(x, y) | x = head, y = tail.head, x > 1 } from stream
```

The co-applicative functor **zips** contexts together.

## The `observe` Block

### Syntax

```ash
observe expr {
    let x = head;              -- extract current value
    let y = tail.head;         -- extract next value
    let z = tail.tail.head;    -- extract future value
    return f(x, y, z)          -- context-aware result
}
```

### Desugaring

```ash
-- Surface:
observe stream {
    let x = head;
    let y = tail.head;
    return f(x, y)
}

-- Desugars to:
extend(fn(ctx) => {
    let x = ctx.head;
    let y = ctx.tail.head;
    f(x, y)
}, stream)
```

The `observe` block binds the **entire context** to an implicit variable (here `ctx`), then extracts values from it.

### Comparison with `do`

| Aspect | `do` (Monad) | `observe` (Comonad) |
|--------|------------|-------------------|
| Binding | `x <- action` | `let x = head` |
| Direction | Forward (next) | Spread (context) |
| Operation | `bind` (>>=) | `extend` (=>) |
| Identity | `return` | `extract` |
| Use case | Effects, state, IO | Streams, signals, data flow |

## The `{}` Co-Comprehension

### Syntax

```ash
-- Co-comprehension: decompose stream into observations
{ f(x, y) | x = head, y = tail.head, x > 1 } from stream
```

### Desugaring

```ash
-- Surface:
{ f(x, y) | x = head, y = tail.head, x > 1 } from stream

-- Desugars to:
extend(fn(ctx) => {
    let x = ctx.head;
    let y = ctx.tail.head;
    if x > 1 then f(x, y) else extract(ctx)
}, stream)
```

The `|` separates **result** from **observations** (decomposition), not **generators** (construction).

### Comparison with `[]`

| Aspect | `[]` (Applicative) | `{}` (Co-applicative) |
|--------|-------------------|----------------------|
| Binding | `x <- xs` | `x = head` |
| Direction | Build up | Decompose down |
| Operation | `apply` (<*>) | `zip` |
| Filter | Exclude values | Exclude contexts |
| Result | New collection | New stream |

## Examples

### Stream Transform: Moving Average

```ash
-- observe: context-aware stream transform
fn moving_average(window: Int, s: Stream<Float>) -> Stream<Float> {
    observe s {
        let values = take(window, this);  -- 'this' is the context
        sum(values) / window as Float
    }
}
```

### Data Flow Network

```ash
-- Co-comprehension: combine streams
fn alert_system(temps: Stream<Float>, pressures: Stream<Float>) -> Stream<Alert> {
    { classify(t, p) | t = head from temps, p = head from pressures } 
}
```

### Mailbox Processing

```ash
-- observe: context includes history
fn process_mailbox(inbox: Mailbox<Email>) -> Mailbox<Action> {
    observe inbox {
        let current = head;
        let history = tail;  -- previous messages
        
        if is_from_boss(current) {
            ImmediateReply(current)
        } else if count_from_sender(history, current.from) > 10 {
            BatchWithPrevious(current, history)
        } else {
            Queue(current)
        }
    }
}
```

### Cellular Automaton (Conway's Game of Life)

```ash
-- observe: neighborhood context
fn life(grid: Stream<Stream<Cell>>) -> Stream<Stream<Cell>> {
    observe grid {
        let cell = head.head;
        let neighbors = count_live(this);  -- count live neighbors in context
        
        match (cell, neighbors) {
            (Alive, 2) => Alive,
            (Alive, 3) => Alive,
            (Dead, 3) => Alive,
            _ => Dead
        }
    }
}
```

## The Categorical Interface

### Comonad

```ash
interface Comonad<W> {
    fun extract(wa: W<a>) : a;
    fun extend(f: W<a> -> b, wa: W<a>) : W<b>;
    
    -- Laws:
    -- extract . extend f = f           (left identity)
    -- extend extract = id              (right identity)
    -- extend f . extend g = extend (f . extend g)  (associativity)
}
```

### Co-applicative

```ash
interface Coapplicative<W> {
    fun extract(wa: W<a>) : a;
    fun zip(wa: W<a>, wb: W<b>) : W<(a, b)>;
    
    -- Laws:
    -- extract (zip wa wb) = (extract wa, extract wb)  (naturality)
    -- zip (extract wa) (extract wb) = extract (zip wa wb)  (identity)
}
```

### Cokleisli Composition

```ash
-- Data flow arrows: compose context-aware functions
type Cokleisli<W, a, b> = W<a> -> b;

fun compose(f: Cokleisli<W, a, b>, g: Cokleisli<W, b, c>) : Cokleisli<W, a, c> {
    fn(wa) => g(extend(f, wa))
}

-- Operator: <=>
let pipeline = extract <=> moving_average(10) <=> threshold_alert(0.5);
```

## The Stream Type

```ash
-- Stream as cofree comonad over Identity
type Stream<a> = Cofree<Identity, a>;

-- Equivalent to:
type Stream<a> = { head: a, tail: Stream<a> };

-- Comonad instance:
impl Comonad<Stream> {
    fun extract(s) = s.head;
    fun extend(f, s) = {
        head: f(s),
        tail: extend(f, s.tail)
    };
}

-- Co-applicative instance:
impl Coapplicative<Stream> {
    fun extract(s) = s.head;
    fun zip(s1, s2) = {
        head: (s1.head, s2.head),
        tail: zip(s1.tail, s2.tail)
    };
}
```

## Design Decisions

### 1. `observe` vs `with`

Alternative keyword: `with` instead of `observe`:

```ash
with stream { let x = head; ... }
```

**Decision:** Use `observe`. Rationale:
- `with` is already used for handler installation in Koka-style
- `observe` is unambiguous and reflects the semantic operation
- `observe` is the dual of `do` (both are verbs)

### 2. Implicit Context Variable

The `observe` block binds the context to an implicit variable. Options:
- `this` (like object-oriented languages)
- `ctx` (explicit)
- `self` (like Rust)

**Decision:** Use `this`. Rationale:
- Familiar to programmers from OOP
- Short, doesn't clutter the syntax
- The context is the "current object" being observed

### 3. `from` in Co-comprehensions

The `{}` syntax requires `from` to specify the source stream:

```ash
{ f(x) | x = head } from stream
```

Without `from`, the syntax is ambiguous: is this a set comprehension or a co-comprehension?

**Decision:** Require `from`. Rationale:
- Unambiguous: `{} ... from` is always co-comprehension
- Mirrors SQL syntax (`SELECT ... FROM`)
- The `from` keyword is already reserved in Ash

## Relationship to Existing Ash Features

### Do-notation (`do:K`)

```ash
-- Monadic: sequence effectful computations
do:Workflow {
    x <- read_sensor();
    y <- read_sensor();
    return f(x, y)
}
```

`observe` is the dual for **pure, context-dependent** computation.

### List Comprehensions (`[]`)

```ash
-- Applicative: build combinations
[ f(x, y) | x <- xs, y <- ys ]
```

`{}` is the dual for **stream decomposition**.

### Effect Handlers (`handle` / `on`)

```ash
-- Effect handlers: intercept and transform effects
handle action with { ... }
```

`observe` is not an effect handler. It is a **context binder**, not an effect interceptor. However, streams can be produced by effect handlers (e.g., a mailbox handler that yields messages).

## Open Questions

1. Should `observe` support pattern matching on the context? (`observe { Cons(x, xs) } { ... }`)
2. How does `observe` interact with lazy evaluation? (Lazy streams = suspended computation)
3. Can `observe` be nested? (Observe a stream of streams)
4. Should `zip` support more than two streams? (`zip3`, `zip4`, etc.)
5. How does `observe` interact with multi-shot continuations? (Pure streams are naturally multi-shot)

## Changelog

- 2026-06-20: Created design note exploring comonadic computation and co-comprehensions as the dual of monadic `do` and list comprehensions in Ash.
