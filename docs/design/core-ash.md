# Core Ash: Lambda Calculus with Types and Effects

## Status

Design note. Defines the minimal core language of Ash — a lambda calculus with types and effects — without syntactic sugar. All sugar desugars to this core. The core lowers mechanically to the CPS IR (SPEC-098b).

## Summary

Core Ash is a **typed lambda calculus with effect rows**. It is the intermediate layer between surface syntax and CPS IR. Every surface construct desugars to core; core lowers to CPS IR without surprises.

The design goals:
1. **Simplicity**: Core should be small enough to reason about formally.
2. **Mechanical lowering**: Core to CPS IR should be a straightforward translation.
3. **Optimization-friendly**: ANF-like structure, explicit control flow, no hidden allocations.
4. **Expressiveness**: Full enough for the type and effect system to work.

## The Core Grammar

### Types

```text
Type ::= BaseType                    -- Int, String, Bool, Unit
       | Type -> {Row} Type          -- function with effect row
       | (Type, Type)                -- tuple
       | {Label: Type}               -- record
       | Row                         -- effect row

Row ::= {}                           -- empty
      | {EffectItem}                 -- closed
      | {EffectItem | Var}           -- open

EffectItem ::= cap Path.Op
             | resource Path Mode
             | role Path
             | policy Path
             | contract ...
             | channel Path Mode Type
             | proc Op
             | fail
             | evidence Path
```

### Expressions (ANF)

```text
Expr ::= Atom
       | let x = Value in Expr       -- let-binding
       | let rec x = Value in Expr   -- recursive binding
       | if Atom then Expr else Expr -- conditional
       | Call Atom Atom*              -- function call
       | Jump Atom Atom               -- jump to continuation
       | Raise EffectOp Atom*         -- raise effect
       | Handle HandlerClause Expr    -- handle effect

Value ::= Atom
        | fn(x1: T1, ..., xn: Tn) -> {Row} Expr  -- lambda
        | (Atom, ..., Atom)                      -- tuple
        | {Label: Atom, ...}                     -- record

Atom ::= Var | Lit | PrimOp

HandlerClause ::= { op: EffectOp, params: [Param], resume: Param, body: Expr }
```

### Key Invariants

1. **Every intermediate result is named**: `let x = ... in ...`
2. **No nested calls**: `Call` takes atoms, not arbitrary expressions
3. **Explicit control flow**: `Call`, `Jump`, `Raise`, `Handle` — no hidden returns
4. **Effect rows on every function type**: `A -> {Row} B`

## Desugaring from Surface to Core

### Function Definition

```ash
-- Surface:
fn add(a: Int, b: Int) -> Int { a + b }

-- Core:
let add = fn(a: Int, b: Int) -> {} Int { a + b }
```

### Do Block (Monadic Sequencing)

```ash
-- Surface:
fn read_config(path: String) -> {cap fs.read} String {
    do {
        contents <- fs.read(path);
        return contents
    }
}

-- Core:
fn read_config(path: String) -> {cap fs.read} String {
    let k = fn(contents: String) -> {cap fs.read} String {
        contents
    };
    Raise { op: fs.read, args: [path], resume: k, row: {cap fs.read} }
}
```

The `do` block desugars to explicit continuation-passing. Each `<-` binds a continuation; `return` is the identity continuation.

### Handle Block (Effect Handler)

```ash
-- Surface:
fn with_stdout(action: {Console} Unit) -> Unit {
    handle action with {
        print(msg) => { stdout.write(msg); resume(()) }
    }
}

-- Core:
fn with_stdout(action: {Console} Unit) -> Unit {
    Handle {
        clause: {
            op: Console.print,
            params: [msg: String],
            resume: k,
            body: { stdout.write(msg); Jump { cont: k, arg: () } }
        },
        body: action,
        cont: identity_continuation
    }
}
```

### Observe Block (Comonadic Context)

```ash
-- Surface:
fn moving_average(s: Stream<Float>) -> Stream<Float> {
    observe s {
        let x = head;
        let y = tail.head;
        (x + y) / 2.0
    }
}

-- Core:
fn moving_average(s: Stream<Float>) -> Stream<Float> {
    extend(fn(ctx: Stream<Float>) -> Float {
        let x = ctx.head;
        let y = ctx.tail.head;
        (x + y) / 2.0
    }, s)
}
```

The `observe` block desugars to `extend` — the comonadic operation.

### List Comprehension

```ash
-- Surface:
[ x + y | x <- xs, y <- ys ]

-- Core:
flat_map(fn(x: Int) -> List<Int> {
    map(fn(y: Int) -> Int { x + y }, ys)
}, xs)
```

### Co-Comprehension

```ash
-- Surface:
{ f(x, y) | x = head, y = tail.head } from stream

-- Core:
zip_with(fn(x: A, y: B) -> C { f(x, y) }, stream, stream.tail)
```

## Lowering from Core to CPS IR

The lowering is mechanical and mostly identity-preserving:

| Core | CPS IR |
|------|--------|
| `let x = v in e` | `LetVal { name: x, value: [v], body: [e] }` |
| `let rec x = v in e` | `LetRec { name: x, value: [v], body: [e] }` |
| `if a then e1 else e2` | `If { cond: a, then_branch: [e1], else_branch: [e2] }` |
| `Call f args` | `Call { func: f, args: args, cont: k, row: ρ }` |
| `Jump k arg` | `Jump { cont: k, arg: arg }` |
| `Raise op args` | `Raise { op: op, args: args, resume: k, row: ρ }` |
| `Handle clause body` | `Handle { clause: [clause], body: [body], cont: k }` |
| `fn(x1, ..., xn) -> {ρ} e` | `Lam { params: [x1, ..., xn], cont_param: k, body: [e], row: ρ }` |

The only non-trivial part is adding the continuation parameter `k` to every function and call.

## Why This is Good for Optimization

### 1. ANF Structure

Every intermediate result is named. The optimizer can:
- Reorder let-bindings freely (if no dependencies)
- Inline small values
- Eliminate dead bindings

### 2. Explicit Control Flow

No hidden returns. Every path is visible:
- `Call` goes to a function
- `Jump` goes to a continuation
- `Raise` goes to a handler
- `If` branches

The optimizer can:
- Build control-flow graphs
- Identify tail calls
- Merge basic blocks

### 3. Explicit Effect Rows

Every function carries its effect row. The optimizer can:
- Reorder pure functions freely
- Hoist effectful operations out of loops
- Parallelize independent effectful operations

### 4. No Sugar Surprises

`do` notation desugars to explicit binds. No hidden allocations from monadic combinators. The optimizer sees the real code.

## Comparison with Other Core Languages

| Language | Core | Features |
|----------|------|----------|
| **GHC Core** | System Fc + coercions | Type classes, GADTs, roles |
| **OCaml Lambda** | Lambda calculus + records | Mutable state, exceptions, objects |
| **Rust MIR** | SSA + control flow | Ownership, borrows, lifetimes |
| **Ash Core** | Lambda calculus + effect rows | Effect rows, handlers, comonads |

## Advanced Type System Features

### Parametric Polymorphism (Core)

Already in Core Ash:

```text
fn map<A, B, r: EffectRow>(xs: List<A>, f: A -> {r} B) -> {r} List<B> {
    ...
}
```

The type checker instantiates `A`, `B`, `r` at call sites. No surprises.

### Row Polymorphism (Core)

Already in Core Ash:

```text
fn log_and_return<A, r>(x: A) -> {cap log.write | r} A {
    ...
}
```

The row variable `r` is unified with the ambient row. This is the heart of the effect system.

### Ad-Hoc Polymorphism (Sugar)

**Not in core initially.** Two approaches:

**Approach 1: Explicit Functions (Core)**

```text
fn sort_by<A>(xs: List<A>, compare: (A, A) -> {} Ordering) -> List<A> {
    ...
}

-- Usage: explicit comparison function
sort_by<Int>(numbers, Int.compare)
```

**Approach 2: Dictionary Passing (Sugar)**

```text
-- Surface:
fn sort<A: Ord>(xs: List<A>) -> List<A> { ... }

-- Core: desugars to explicit dictionary
fn sort<A>(xs: List<A>, ord_dict: OrdDict<A>) -> List<A> {
    ...
}

-- Usage: dictionary inferred
sort<Int>(numbers)  -- desugars to sort<Int>(numbers, Int.ord_dict)
```

The dictionary is a record of methods:

```text
type OrdDict<A> = {
    compare: (A, A) -> {} Ordering,
    less_than: (A, A) -> {} Bool,
    ...
}
```

**Recommendation**: Start with Approach 1 (explicit). Add Approach 2 (dictionaries) as sugar later.

### Higher-Kinded Types (HKT) (Future)

HKT means type constructors that take type constructors:

```haskell
-- Haskell: Functor is a type constructor that takes a type constructor
class Functor (f :: * -> *) where
    fmap :: (a -> b) -> f a -> f b
```

**Not in core initially.** Core Ash has no `Type -> Type` kind.

**Workaround: Explicit Instances**

```text
-- No Functor type class. Just explicit functions.
fn map_list<A, B>(f: A -> {} B, xs: List<A>) -> List<B> { ... }
fn map_option<A, B>(f: A -> {} B, xs: Option<A>) -> Option<B> { ... }
fn map_stream<A, B>(f: A -> {} B, xs: Stream<A>) -> Stream<B> { ... }
```

**Why this is acceptable**: Effect rows replace many HKT use cases. You don't need `Monad` as a type class if you have effect rows. You don't need monad transformers if you have row union.

**Future**: Add HKT if needed for generic programming (e.g., `traverse` over any container).

### GADTs (Future)

GADTs allow types to vary by constructor:

```haskell
-- Haskell GADT
data Expr a where
    Lit :: Int -> Expr Int
    Add :: Expr Int -> Expr Int -> Expr Int
    Eq  :: Expr a -> Expr a -> Expr Bool
```

**Not in core initially.** Use plain ADTs with runtime checks:

```text
type Expr = Lit(Int) | Add(Expr, Expr) | Eq(Expr, Expr);

fn eval(e: Expr) -> Int {
    match e {
        Lit(n) => n,
        Add(a, b) => eval(a) + eval(b),
        Eq(_, _) => panic!("Eq returns Bool, not Int")
    }
}
```

**Future**: Add GADTs if needed for embedded DSLs or typed ASTs.

### Summary Table

| Feature | In Core Ash? | How | Priority |
|---------|-----------|-----|----------|
| **Parametric polymorphism** | Yes | `forall A, B` | Core |
| **Row polymorphism** | Yes | `r: EffectRow` | Core |
| **Ad-hoc polymorphism (explicit)** | Yes | Explicit function params | Core |
| **Ad-hoc polymorphism (type classes)** | No | Dictionary passing | Sugar |
| **HKT** | No | `Type -> Kind` | Future |
| **GADTs** | No | Type equality constraints | Future |
| **Dependent types** | No | Types depend on values | Future |

## Relationship to Other Ash Design Notes

## Relationship to Other Ash Design Notes

| Design Note | Builds On Core Ash |
|-------------|-------------------|
| [effect-handling-styles.md](effect-handling-styles.md) | `Raise` and `Handle` in core |
| [multi-shot-continuations.md](multi-shot-continuations.md) | `Cont` type in core |
| [DESIGN-NOTE-COMONADIC-COMPUTATION.md](DESIGN-NOTE-COMONADIC-COMPUTATION.md) | `extend` and `extract` in core |
| [effectful-stream-sinks.md](effectful-stream-sinks.md) | `Yield`/`Await` effects in core |
| [process-model.md](process-model.md) | `proc spawn` effect in core |

## Open Questions

1. Should core have explicit `letcont` (label bindings) or is that only in CPS IR?
2. Should core have `trap` or is that only in CPS IR?
3. How do type holes (`_`) desugar to core?
4. How do pattern matches desugar to core? (case expressions?)
5. Should core have explicit `fix` for recursion, or is `let rec` sufficient?

## Changelog

- 2026-06-20: Created design note defining Core Ash — the minimal lambda calculus with types and effects that underlies all surface syntax.
