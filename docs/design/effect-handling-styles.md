# Effect Handling in Ash: Design Choices

## Summary

This document captures the design discussion for effect handling in Ash, comparing Koka-style explicit handlers with Frank-style implicit handlers. Both approaches lower to the same CPS IR (`Handle`/`Raise` nodes). The choice is surface syntax and readability.

## The Core Mechanism (CPS IR)

Both approaches lower to the same CPS IR:

```text
Handle {
    clause: HandlerClause { op: Console.print, params: [msg], resume: k, body: ... },
    body: action,
    cont: k_outer,
    row: ...
}
```

The runtime:
1. Runs `action`
2. When `action` raises `Console.print`, suspends it
3. Dispatches to the matching `HandlerClause`
4. The clause body runs, optionally calling `resume` (or `k`) to continue

This is the same for both styles. The difference is only in the surface syntax.

---

## Approach 1: Koka-Style (Explicit Handlers)

### Syntax

```ash
-- Effect declaration
 effect Console
   fun print(msg: String)
   fun println(msg: String)

-- Computation that uses the effect
fn greet() -> {Console} Unit {
    do {
        print("Hello, ");
        println("World!")
    }
}

-- Handler: explicit `handle ... with` block
fn with_stdout(action: {Console} Unit) -> Unit {
    handle action with {
        print(msg) => {
            stdout.write(msg);
            resume(())
        },
        println(msg) => {
            stdout.write(msg ++ "\n");
            resume(())
        }
    }
}

-- Usage: explicit handler installation
fn main() -> Unit {
    with_stdout(greet)
}
```

### Key Features

| Feature | How it works |
|---------|-------------|
| **Handler installation** | `handle action with { ... }` — explicit syntax |
| **Continuation** | `resume(())` — primitive that resumes the suspended computation |
| **Handler recursion** | Implicit — the `handle` block stays active for all commands |
| **Pattern matching** | By operation name: `print(msg)`, `println(msg)` |
| **Return** | Implicit — when `action` returns normally, the `handle` block returns that value |

### Readability

- **Pros**: Clear boundary between computation and handler. `resume` is explicit. Familiar to users of try/catch or Koka/Eff.
- **Cons**: `handle ... with` is verbose. `resume` is a magic primitive. Nested handlers create indentation.

---

## Approach 2: Frank-Style (Implicit Handlers)

### Syntax

```ash
-- Effect declaration (same as Koka)
effect Console
  fun print(msg: String)
  fun println(msg: String)

-- Computation that uses the effect (same as Koka)
fn greet() -> {Console} Unit {
    do {
        print("Hello, ");
        println("World!")
    }
}

-- Handler: ordinary function with command patterns
fn with_stdout(action: {Console} Unit) -> Unit {
    on action {
        <print msg -> k> => {
            stdout.write(msg);
            with_stdout(k(()))
        },
        <println msg -> k> => {
            stdout.write(msg ++ "\n");
            with_stdout(k(()))
        },
        return(()) => ()
    }
}

-- Usage: ordinary function application
fn main() -> Unit {
    with_stdout(greet)
}
```

### Key Features

| Feature | How it works |
|---------|-------------|
| **Handler installation** | Ordinary function application: `with_stdout(greet)` |
| **Continuation** | `k` — explicit function argument, not a primitive |
| **Handler recursion** | Explicit — the handler calls itself on `k(())` |
| **Pattern matching** | By command pattern: `<print msg -> k>`, `<println msg -> k>` |
| **Return** | Explicit — `return(()) => ()` matches normal completion |

### Readability

- **Pros**: No special syntax. Handler is just a function. `k` is a real value you can inspect, pass around, or ignore. Composable by ordinary function composition.
- **Cons**: `k` is verbose. Recursive handler calls can look like infinite loops. Less familiar to imperative programmers.

---

## Side-by-Side Comparison: Same Task

### Task: Collect emitted strings into a list

**Koka-style:**
```ash
fn emit_to_list(action: {Emit String} Unit) -> List<String> {
    handle action with {
        emit(msg) => {
            let rest = resume(());
            Cons(msg, rest)
        }
    }
}
```

**Frank-style:**
```ash
fn emit_to_list(action: {Emit String} Unit) -> List<String> {
    on action {
        <emit msg -> k> => {
            let rest = emit_to_list(k(()));
            Cons(msg, rest)
        },
        return(()) => Nil
    }
}
```

### Task: Retry on failure (resume with default)

**Koka-style:**
```ash
fn with_retry(action: {Fail} Int) -> Int {
    handle action with {
        fail(msg) => {
            log("Retry: " ++ msg);
            resume(0)  -- resume with default
        }
    }
}
```

**Frank-style:**
```ash
fn with_retry(action: {Fail} Int) -> Int {
    on action {
        <fail msg -> k> => {
            log("Retry: " ++ msg);
            with_retry(k(0))  -- resume with default, reinstall handler
        },
        return(v) => v
    }
}
```

### Task: Count emits before collecting

**Koka-style:**
```ash
fn count_and_collect(action: {Emit String} Unit) -> (Int, List<String>) {
    handle action with {
        emit(msg) => {
            let (count, rest) = resume(());
            (count + 1, Cons(msg, rest))
        }
    }
}
```

**Frank-style:**
```ash
fn count_and_collect(action: {Emit String} Unit) -> (Int, List<String>) {
    on action {
        <emit msg -> k> => {
            let (count, rest) = count_and_collect(k(()));
            (count + 1, Cons(msg, rest))
        },
        return(()) => (0, Nil)
    }
}
```

---

## Design Tradeoffs

| Aspect | Koka-style | Frank-style |
|--------|-----------|-------------|
| **Syntax noise** | `handle ... with`, `resume` | `on`, `k`, recursive calls |
| **Continuation visibility** | Hidden (magic `resume`) | Explicit (`k` is a value) |
| **Handler composition** | Nested `handle` blocks | Ordinary function composition |
| **Reinstalling handler** | Automatic (implicit) | Explicit (recursive call) |
| **Learning curve** | Familiar to try/catch users | Familiar to functional programmers |
| **Debugging** | Harder (magic resume) | Easier (k is inspectable) |
| **Nested handlers** | Indentation pyramid | Flat function calls |
| **Multiple resumes** | `resume` multiple times | Call `k` multiple times |
| **Ignoring continuation** | `resume` is required | Can ignore `k` (non-resumptive) |

---

## The Core Difference

**Koka**: The handler is a **delimited scope**. The `handle` block captures the continuation implicitly. `resume` is a primitive that resumes the captured continuation.

**Frank**: The handler is a **recursive function**. The continuation `k` is an explicit argument. Resuming is just calling `k`. Reinstalling the handler is just recursing.

Both are valid. Both are expressive. The choice is **readability and familiarity**.

---

## Recommendation for Ash

**Support both, with Koka-style as default and Frank-style as advanced.**

Rationale:
- Ash targets workflow developers who may have imperative backgrounds. Koka-style `handle ... with` is more familiar.
- Frank-style is valuable for advanced users who need explicit control over continuations (e.g., for backtracking, cooperative threading, or custom resume logic).
- Both lower to the same CPS IR. The compiler doesn't care.
- The `on` keyword for Frank-style is already reserved in the grammar (for observations). Could be reused or a new keyword chosen.

### Default (Koka-style)
```ash
fn with_stdout(action: {Console} Unit) -> Unit {
    handle action with {
        print(msg) => { stdout.write(msg); resume(()) }
    }
}
```

### Advanced (Frank-style)
```ash
fn with_stdout(action: {Console} Unit) -> Unit {
    on action {
        <print msg -> k> => { stdout.write(msg); with_stdout(k(())) }
    }
}
```

Both compile to the same CPS `Handle` node. The user chooses the style that fits their algorithm and mental model.

---

## Open Questions

1. Should `on` be the keyword for Frank-style, or something else (`match`, `run`, `handle`)?
2. Should Frank-style be allowed to mix with Koka-style in the same project?
3. Should the compiler warn when `k` is unused (non-resumptive handler)?
4. How do we teach the difference without confusing users?

---

## Example: STM (Software Transactional Memory)

STM provides atomic transactions on shared memory. In Ash, STM is an effect discharged by a handler, not a monad.

### Effect Declaration

```ash
effect STM<a>
  fun readTVar(tvar: TVar<a>) : a
  fun writeTVar(tvar: TVar<a>, value: a) : Unit
  fun retry() : a
  fun orElse(left: {STM} a, right: {STM} a) : a
```

### Using STM

```ash
fn transfer(from: TVar<Int>, to: TVar<Int>, amount: Int) : {STM} Unit {
    do {
        balance <- readTVar(from);
        if balance < amount then retry();
        writeTVar(from, balance - amount);
        writeTVar(to, readTVar(to) + amount);
        return ()
    }
}
```

### Koka-Style Handler

```ash
fn atomically(action: {STM} a) : a {
    handle action with {
        readTVar(tvar) => {
            let value = readFromLog(tvar);
            resume(value)
        },
        writeTVar(tvar, value) => {
            recordWrite(tvar, value);
            resume(())
        },
        retry() => {
            let watched = getReadSet();
            blockUntilChange(watched);
            atomically(action)  -- restart from scratch, discarding continuation
        },
        orElse(left, right) => {
            let result = tryAtomically(left);
            match result {
                Success(v) => resume(v),
                Retry => resume(right())
            }
        }
    }
}
```

### Frank-Style Handler

```ash
fn atomically(action: {STM} a, log: TransactionLog) : (a, TransactionLog) {
    on action {
        <readTVar tvar -> k> => {
            let (value, newLog) = readFromLog(tvar, log);
            atomically(k(value), newLog)
        },
        <writeTVar tvar value -> k> => {
            let newLog = recordWrite(tvar, value, log);
            atomically(k(()), newLog)
        },
        <retry -> k> => {
            -- k is discarded: non-resumptive
            let watched = getReadSet(log);
            blockUntilChange(watched);
            atomically(action, emptyLog)  -- restart from scratch
        },
        <orElse left right -> k> => {
            let result = tryAtomically(left, log);
            match result {
                Success(v, newLog) => {
                    atomically(k(v), mergeLog(log, newLog))
                },
                Retry => {
                    atomically(k(right()), log)
                }
            }
        },
        return(v) => (v, log)
    }
}
```

### Key Insights from STM

| Aspect | How it works |
|--------|-------------|
| **Handler-local state** | The transaction log is threaded through recursive calls (Frank) or captured in the handler closure (Koka) |
| **Resumptive operations** | `readTVar`, `writeTVar` — resume with value |
| **Non-resumptive operations** | `retry` — discards continuation, restarts transaction |
| **Nested transactions** | `orElse` — runs sub-transaction with its own log, merges on success |
| **Composability** | STM composes with other effects (e.g., `{STM, Console}`) |

### Comparison with Haskell STM

| Haskell | Ash (Koka-style) | Ash (Frank-style) |
|---------|------------------|-------------------|
| `atomically :: STM a -> IO a` | `atomically : ({STM} a) -> a` | `atomically : ({STM} a, TransactionLog) -> (a, TransactionLog)` |
| `readTVar :: TVar a -> STM a` | `readTVar : TVar<a> -> {STM} a` | `readTVar : TVar<a> -> {STM} a` |
| `retry :: STM a` | `retry : {STM} a` | `retry : {STM} a` |
| `orElse :: STM a -> STM a -> STM a` | `orElse : {STM} a -> {STM} a -> {STM} a` | `orElse : {STM} a -> {STM} a -> {STM} a` |

The key difference: in Haskell, STM is a **monad**. In Ash, STM is an **effect discharged by a handler**. The handler maintains the transaction log and controls commit/abort/retry semantics.

---

## Changelog

- 2026-06-20: Created design document comparing Koka-style and Frank-style effect handling in Ash.
- 2026-06-20: Added STM example illustrating resumptive vs non-resumptive handlers, nested transactions, and handler-local state.
