# Effectful Stream Sinks and Comonad-Monad Bridges

## Status

Design note. Explores how effectful stream sinks (e.g., network send) interact with comonadic stream sources in Ash. Collects examples and external references for future documentation.

## Summary

A stream in Ash is a **comonad** — it produces values with context (history, neighbors, position). A network sink is a **monad/effect** — it consumes values and performs side effects. This note explores how to bridge comonadic sources to monadic sinks, with examples framed in Ash terms and references to prior art.

## References

- [DESIGN-NOTE-COMONADIC-COMPUTATION.md](DESIGN-NOTE-COMONADIC-COMPUTATION.md) — comonadic `observe` blocks and co-comprehensions in Ash
- [effect-handling-styles.md](effect-handling-styles.md) — Koka-style vs Frank-style effect handlers
- [multi-shot-continuations.md](multi-shot-continuations.md) — multi-shot continuations for pure computations
- Haskell `pipes` library (Gabriel Gonzalez) — composable stream processing with producer/consumer/transducer abstraction
- Haskell `conduit` library (Michael Snoyman) — streaming data with left/right fusion
- Haskell `machines` library (Edward Kmett, Rúnar Bjarnason) — stream processing with automata
- "Comonads and OOP" (Edward Kmett) — comonads as objects with a context
- "The Essence of the Iterator Pattern" (Jeremy Gibbons, Bruno Oliveira) — applicative traversal as the bridge between comonad and monad
- "Comonadic Scanning" (Conal Elliott) — comonadic pattern for accumulation over streams

## The Core Problem

A stream is a **comonad** (context-aware values). A sink is a **monad** (effectful consumer). How do they compose?

```ash
-- Stream (comonad): produces values with context
let sensor_readings: Stream<Float> = ...;

-- Network sink (monad/effect): consumes values, performs effects
fn send_readings(readings: Stream<Float>) -> {cap network.send} Unit {
    -- ???
}
```

The comonad is pure. The monad is effectful. The bridge is **iteration**.

## Approach 1: Pure Stream, Then Effectful Iteration

Build a pure stream, then iterate with effects.

```ash
-- Build a pure stream from a sensor
fn sensor_stream() -> Stream<Float> {
    -- Pure construction of stream
    {
        head: 23.5,
        tail: {
            head: 24.1,
            tail: {
                head: 22.8,
                tail: Nil
            }
        }
    }
}

-- Effectful iteration over stream
fn send_readings(readings: Stream<Float>) -> {cap network.send} Unit {
    do {
        for value in readings {
            network.send(value.to_string());
        }
    }
}

-- Usage
fn main() -> {cap network.send} Unit {
    let readings = sensor_stream();
    send_readings(readings);
}
```

### External Reference: Haskell `pipes`

Haskell's `pipes` library uses a `Producer`/`Consumer`/`Pipe` abstraction:

```haskell
-- Producer: yields values (comonad-like)
producer :: Producer Float IO ()
producer = yield 23.5 >> yield 24.1 >> yield 22.8

-- Consumer: awaits values (monad-like)
consumer :: Consumer Float IO ()
consumer = do
    x <- await
    liftIO $ sendOverNetwork (show x)
    consumer

-- Composition: connect producer to consumer
pipeline :: Effect IO ()
pipeline = producer >-> consumer
```

In Ash, the `for` loop is the equivalent of `>->` (pipe composition).

## Approach 2: Effectful Stream Production (Yield Effect)

Produce the stream via an effect, then consume.

```ash
-- Yield effect: produces values on demand
effect Yield<A>
  fun yield(value: A) : Unit

-- Producer: effectful stream generation
fn sensor_producer() -> {Yield<Float>} Unit {
    do {
        yield(23.5);
        yield(24.1);
        yield(22.8);
    }
}

-- Consumer: intercepts yields, sends to network
fn send_over_network(producer: {Yield<Float>} Unit) -> {cap network.send} Unit {
    on producer {
        <yield value -> k> => {
            network.send(value.to_string());
            send_over_network(k(()))
        },
        return(()) => ()
    }
}

-- Usage
fn main() -> {cap network.send} Unit {
    send_over_network(sensor_producer);
}
```

### External Reference: Haskell `conduit`

Haskell's `conduit` uses `yield` and `await` as primitives:

```haskell
-- Producer: yields values
source :: ConduitT () Float IO ()
source = yield 23.5 >> yield 24.1 >> yield 22.8

-- Consumer: awaits and processes
sink :: ConduitT Float Void IO ()
sink = do
    mx <- await
    case mx of
        Nothing -> return ()
        Just x  -> liftIO (sendOverNetwork (show x)) >> sink

-- Connect: source $$ sink
```

In Ash, the Frank-style handler is the equivalent of `ConduitT`.

## Approach 3: Comonadic Observation, Then Effectful Send

Observe the stream's context (comonad), then send (monad).

```ash
-- Observe stream context (comonad)
fn observe_and_send(readings: Stream<Float>) -> {cap network.send} Unit {
    do {
        -- Observe context: current and previous values
        let context = observe readings {
            let current = this.head;
            let previous = this.tail.head;
            (current, previous)
        };

        -- Effectful send of context
        network.send("Current: " ++ context.0.to_string());
        network.send("Previous: " ++ context.1.to_string());

        -- Continue with tail
        observe_and_send(readings.tail);
    }
}
```

### External Reference: Comonadic Scanning (Conal Elliott)

Conal Elliott's "Comonadic Scanning" shows how comonads capture accumulation patterns:

```haskell
-- Comonadic scan: accumulate over a stream
scan :: Comonad w => (w a -> b) -> w a -> w b
scan f = extend f . duplicate

-- In Ash: observe block with context
```

The `observe` block in Ash is the equivalent of `extend` — it spreads a context-aware function over the stream.

## Approach 4: Bidirectional Pipe (Producer + Consumer)

A pipe that both produces and consumes, with effects in the middle.

```ash
-- Pipe: transforms a stream with effects
fn alert_pipe(input: Stream<Float>) -> {cap network.send} Stream<Alert> {
    do {
        -- Observe context (comonad)
        let alert = observe input {
            let current = this.head;
            let previous = this.tail.head;
            if abs(current - previous) > threshold {
                Alert(current)
            } else {
                NoAlert
            }
        };

        -- Effectful send (monad)
        if alert != NoAlert {
            network.send("ALERT: " ++ alert.to_string());
        }

        -- Continue with transformed stream
        Cons(alert, alert_pipe(input.tail))
    }
}
```

### External Reference: Haskell `pipes` (Bidirectional)

Haskell's `pipes` supports bidirectional flow:

```haskell
-- Pipe: transforms upstream to downstream, with effects
pipe :: Pipe Float Alert IO ()
pipe = do
    x <- await          -- from upstream
    y <- await          -- from upstream
    let alert = if abs (x - y) > threshold then Alert x else NoAlert
    when (alert /= NoAlert) $ liftIO $ sendOverNetwork (show alert)
    yield alert         -- to downstream
    pipe
```

In Ash, the combination of `observe` (comonad) and `do` (monad) achieves the same.

## The Comonad-Monad Bridge: General Pattern

The general pattern for connecting comonad to monad:

```ash
-- Comonad provides structure (stream, grid, tree)
-- Monad provides effects (send, log, write)
-- Bridge: iterate with effects

fn comonad_to_monad<W, A, B>(
    w: W<A>,                    -- comonad
    extract: W<A> -> A,        -- get current value
    step: W<A> -> W<A>,         -- move to next
    action: A -> {E} B           -- effectful action
) -> {E} Unit {
    do {
        let value = extract(w);
        action(value);
        comonad_to_monad(step(w), extract, step, action)
    }
}
```

This is **monadic fold on a comonadic structure**.

## External References: The Iterator Pattern

Jeremy Gibbons and Bruno Oliveira's "The Essence of the Iterator Pattern" shows that **applicative functors** are the bridge between comonads and monads:

```haskell
-- Applicative traversal: the bridge
traverse :: Applicative f => (a -> f b) -> [a] -> f [b]
traverse f []     = pure []
traverse f (x:xs) = pure (:) <*> f x <*> traverse f xs

-- In Ash: for-loop with effects is the equivalent
```

The `for` loop in Ash is the applicative traversal — it bridges the pure stream (comonad) to the effectful sink (monad).

## Summary Table

| Approach | Comonad Role | Monad Role | Bridge | External Reference |
|----------|-------------|-----------|--------|-------------------|
| Pure stream, then iterate | Stream construction | `for` loop | `for` | Haskell `pipes` |
| Yield effect, handler | `Yield` effect | Frank-style handler | `on` | Haskell `conduit` |
| Observe then send | `observe` block | `do` block | Sequential | Comonadic scanning |
| Bidirectional pipe | `observe` + `Cons` | `do` + `network.send` | Recursion | Haskell `pipes` bidirectional |

## Expressive Power: Can Ash Implement `pipes`/`conduit`/`machines`?

**Yes.** Ash's effect system + comonads provide sufficient expressive power.

### The Core Abstraction

Streaming libraries (`pipes`, `conduit`, `machines`) share a core abstraction:

| Role | Operation | Type |
|------|-----------|------|
| **Producer** | `yield` | `Producer<a> = {Yield<a>} Unit` |
| **Consumer** | `await` | `Consumer<a> = {Await<a>} Unit` |
| **Pipe** | `yield` + `await` | `Pipe<a, b> = {Await<a>, Yield<b>} Unit` |
| **Connect** | interleave | `connect : Producer<a> -> Consumer<a> -> Unit` |
| **Compose** | pipe fusion | `compose : Pipe<a, b> -> Pipe<b, c> -> Pipe<a, c>` |

### Ash Implementation Sketch

```ash
-- User-defined effect (or built-in)
effect Yield<A>
  fun yield(value: A) : Unit

effect Await<A>
  fun await() : A

-- Producer: yields values
fn producer() -> {Yield<Int>} Unit {
    do {
        yield(1);
        yield(2);
        yield(3);
    }
}

-- Consumer: awaits values
fn consumer() -> {Await<Int>} Unit {
    do {
        let x = await();
        log(x);
        let y = await();
        log(y);
    }
}

-- Connect: interleave producer and consumer via handler
fn connect<A>(
    producer: {Yield<A>} Unit,
    consumer: {Await<A>} Unit
) -> Unit {
    -- Handler routes yield to await
    -- CPS IR Handle/Raise supports this interleaving
    ...
}

-- Pipe: both awaits and yields
fn pipe() -> {Await<Int>, Yield<Int>} Unit {
    do {
        let x = await();
        yield(x * 2);
    }
}

-- Compose: pipe fusion
fn compose<A, B, C>(
    left: {Await<A>, Yield<B>} Unit,
    right: {Await<B>, Yield<C>} Unit
) -> {Await<A>, Yield<C>} Unit {
    -- Interleave left and right
    -- left's yield feeds right's await
    ...
}
```

### Why Ash is Sufficient

| Feature | `pipes`/`conduit` | Ash Equivalent |
|---------|-------------------|----------------|
| `yield` | `yield :: a -> Producer a m ()` | `Yield` effect + handler |
| `await` | `await :: Consumer a m a` | `Await` effect + handler |
| `>->` | pipe composition | `compose` function (handler combinator) |
| `>->` | producer-consumer connect | `connect` function (handler) |
| `lift` | lift underlying monad | Effect row union (`{Yield, Console}`) |
| `runEffect` | run the pipeline | Handler discharge |

The CPS IR's `Handle`/`Raise` nodes support **coroutine-style interleaving** — exactly what streaming libraries need.

### The Comonad Connection

`machines` uses the **Store comonad** for stateful transducers:

```haskell
-- Machine as Store comonad
data Machine k o = Machine { runMachine :: k -> (o, Machine k o) }
```

In Ash, this is the `Stream` comonad with a cursor:

```ash
-- Machine: comonad over input/output
type Machine<k, o> = {
    run: k -> (o, Machine<k, o>)
}

-- extract: current output
fn extract(m: Machine<k, o>) -> o {
    let (_, output) = m.run(default_k);
    output
}

-- extend: apply context-aware function
fn extend(f: Machine<k, o> -> p, m: Machine<k, o>) -> Machine<k, p> {
    {
        run: fn(k) => {
            let (o, next) = m.run(k);
            (f(next), extend(f, next))
        }
    }
}
```

This is **expressible** in Ash's comonad framework (see [DESIGN-NOTE-COMONADIC-COMPUTATION.md](DESIGN-NOTE-COMONADIC-COMPUTATION.md)).

### The Library Path

Ash should **not** bake `pipes`/`conduit` into the language. Instead:

1. **Provide `Yield`/`Await` as built-in effects** (or user-definable)
2. **Provide `connect`/`compose` as library functions** (handlers)
3. **Provide `Stream` as a comonad** (for context-aware sources)
4. **Let users build `pipes`-like libraries** on top

This matches Ash's philosophy: minimal core, expressive power for libraries.

## Open Questions

1. Should Ash provide a `for` loop sugar over streams, or rely on explicit recursion?
2. Should the `Yield` effect be built-in, or user-defined?
3. How does lazy stream iteration interact with effectful sinks? (Lazy send = deferred effects)
4. How does memo stream iteration interact with effectful sinks? (Memo send = cached effects, only first send fires)
5. Should Ash provide a `Pipe` abstraction (like `pipes` or `conduit`) as a built-in type?
6. Should `connect` be a primitive operator or a library function?
7. How does bidirectional flow (upstream/downstream in `pipes`) map to Ash's effect rows?

## Changelog

- 2026-06-20: Created design note exploring effectful stream sinks and comonad-monad bridges in Ash, with examples and external references.
- 2026-06-20: Added expressive power analysis: Ash can implement `pipes`/`conduit`/`machines` as libraries via user-defined effects + handlers + comonads.
