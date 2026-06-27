# NOTE-025: Effect Identity via Sorts and Impls

**Date:** 2026-06-27
**Status:** Living document — design direction captured; revises the identity model from
NOTE-022; open questions tracked
**Purpose:** Establish the effect identity model: interfaces are sorts (abstract effect
groupings), impl types are identity carriers, and handler functions are always
named values installed via function application. Record the derivation mechanism and provide
worked examples for deep, escape, and multi-shot handlers.

Companion to NOTE-022 (declaration side: interfaces), NOTE-023 (dispatch side: handler
surface), NOTE-013 (handler composition algebra), and NOTE-024 (host/FFI).

## Pre-Spec Delta

This note is pre-spec and revises the identity model established in NOTE-022 and reconciled
into SPEC-095b/096b/097b. When the project moves to spec updates:

- **Row item identity:** `{F::read}` (abstract, sort-parameterized) replaces `{Fs.read}`
  (concrete interface-qualified). After monomorphization, `{PosixFs::read}`.
- **Impl types and impls:** every effect operation identity requires a concrete impl. The impl
  type is the operation identity — it can be empty (a phantom type) or carry data. This is new
  surface syntax.
- **Handler derivation:** `derive handler <name>` is a new declaration inside impl bodies.
- **Handler installation:** `handle expr with <name>` always resolves to function
  application. `<name>` is always a value-namespace function. No type-namespace resolution.
- **SPEC-095b/097b/098b:** the `OperationEffect` identity changes from `{interface, operation}`
  to `{impl_type, operation}` — the concrete impl type replaces the interface as the
  identity qualifier after specialization.

## 0. Motivation

NOTE-022 settled that operations are declared as interface methods. The row item was `Fs.read`
— the interface name qualifying the operation. This worked for type-checking but had a
semantic problem: the interface is a grouping (namespace + laws + shared generics), not a
dispatch mechanism. Multiple handlers for `Fs.read` cannot coexist because they share one
identity.

The sort/impl model separates the abstraction layers:

```
Interface = sort (abstract effect family + laws)
Impl type = identity carrier (the impl type — can be empty or carry data)
Handler   = behavior provider (named function, installed at runtime)
```

The row item is parameterized by the impl type: `F::read` where `F: Fs`. After
monomorphization, the identity is concrete: `PosixFs::read`. Different impls produce
different identities. Multiple handlers can coexist.

## 1. The Identity Model

### 1.1 Interface as sort

An interface declares operation signatures, generics, associated types, laws, and contracts.
It is the abstract effect family — the *sort*. It does not dispatch, does not provide
identity, and does not define handler behavior.

```ash
interface Fs {
    fn read(path: Path) -> String;
    fn write(path: Path, contents: String) -> Unit;

    law read_after_write {
        forall p, c. write(p, c); read(p) == c
    }
}
```

### 1.2 Impl type + impl as identity carrier

The impl type is the operation identity. It can be empty (a phantom type used purely for
compile-time identity) or it can carry data (configuration, state, connection parameters).
The type system does not restrict which. Every effect operation that appears in a row must
have a concrete impl behind it — for method resolution and type-checking. The impl type
parameter is the operation identity.

```ash
// Phantom type — empty, identity-only. Common for pure algebraic effects.
type PosixFs = Unit;

impl Fs for PosixFs
where requires host posix_fs
{
    fn read(path: Path) -> String { builtin(fs_read, path) }
    fn write(path: Path, contents: String) -> Unit { builtin(fs_write, path, contents) }
}

// Data-carrying type — carries runtime configuration. Identity + config in one type.
type ConfiguredFs = { root: Path, readonly: Bool };

impl Fs for ConfiguredFs {
    fn read(path: Path) -> String { builtin(fs_read, self.root.join(path)) }
    fn write(path: Path, contents: String) -> Unit {
        if self.readonly { panic("readonly") } else { builtin(fs_write, path, contents) }
    }
}
```

After monomorphization, `PosixFs::read` and `ConfiguredFs::read` are distinct concrete
operation identities. A different impl type produces a different identity, regardless of
whether the type carries data. The Core/IR sees `EffectOp { impl_type: "PosixFs", operation: "read" }` or `EffectOp { impl_type: "ConfiguredFs", operation: "read" }`.

### 1.3 The impl method body is the default deep-handler behavior

The impl method body provides what `derive handler` uses: the computation that produces the
operation result. For a derived deep handler, the compiler wraps it: `resume(impl_method(args))`.

When an explicit handler overrides the behavior (multi-shot, escape), the impl method body is
not called — but it must still exist for name resolution and type-checking.

### 1.4 Row item spelling

**Abstract (generic code):**

```ash
fn load_config<F: Fs, r: Row>(path: Path) -> {F::read | r} String {
    F.read(path)
}
```

The row item `F::read` is abstract — parameterized by the sort constraint `F: Fs`. The
caller specializes `F` to a concrete impl type.

**Concrete (after monomorphization):**

```
load_config<PosixFs> → row item: {PosixFs::read}
load_config<MemoryFs> → row item: {MemoryFs::read}
```

These are distinct operation identities. The Core/IR sees `EffectOp { impl_type: "PosixFs", operation: "read" }`.

## 2. Handler Functions

### 2.1 The handler type

A handler is a function. Its type is always:

```text
forall A, r: Row. (Unit -> {op | r} A) -> {r} Ans
```

Where `op` is the concrete operation identity (after specialization), `A` is the computation's
value type, and `Ans` is the handler's answer type (which may differ from `A`).

### 2.2 `handler` as keyword alias for `fn`

`handler` is a pure keyword alias for `fn` (per NOTE-023). It carries no semantic difference.
It signals intent to humans and LLMs: "this function is meant to be used as a handler." The
compiler treats it identically to `fn`.

### 2.3 Three ways to produce a handler function

There are three ways to get a named handler function in the value namespace:

1. **Derive** — `derive handler <name>` inside an impl body. The compiler synthesizes a
   deep handler from the impl's method bodies. Mechanical, total for deep handlers.

2. **Handler defined in impl** — an explicit `handler <name>(...) { ... }` block inside the
   impl body. Same semantics as a standalone handler, but co-located with the impl for DX.
   Can express deep, escape, or multi-shot behavior. Has access to impl method bodies.

3. **Standalone handler function** — a `handler <name>(...) { ... }` at module level,
   independent of any impl. The canonical explicit form. Everything desugars to this.

All three produce a named function in the value namespace. All three are usable with
`handle expr with <name>`.

## 3. Handler Installation

### 3.1 `handle ... with` is always function application

```ebnf
handle_with_expr = "handle" expr "with" identifier ;
```

`identifier` resolves through normal value-name resolution. It must be a function whose first
parameter accepts the thunk type `Unit -> {op | r} A`. The type checker validates the
application. No type-namespace lookup, no impl-synthesis branch at the installation site.

```ash
handle expr with my_handler
```

desugars to:

```ash
my_handler(fn () -> expr)
```

### 3.2 Installation type-checking

The handler function's input row must subsume the thunk's row. If the thunk requires
`{PosixFs::read}` and the handler accepts `{PosixFs::read | r}`, then `r` unifies with `{}`
and the installation type-checks.

## 4. Worked Examples

### 4.1 Deep handler (Fs, PosixFs) — three forms

#### Declaration

```ash
interface Fs {
    fn read(path: Path) -> String;
    fn write(path: Path, contents: String) -> Unit;
}

type PosixFs = Unit;
type MemoryFs = Unit;

impl Fs for PosixFs
where requires host posix_fs
{
    fn read(path: Path) -> String { builtin(fs_read, path) }
    fn write(path: Path, contents: String) -> Unit { builtin(fs_write, path, contents) }
}

impl Fs for MemoryFs {
    fn read(path: Path) -> String { memory_map[path] }
    fn write(path: Path, contents: String) -> Unit { memory_map[path] = contents }
}
```

#### Form A: derive (compiler synthesizes)

```ash
impl Fs for PosixFs
where requires host posix_fs
{
    fn read(path: Path) -> String { builtin(fs_read, path) }
    fn write(path: Path, contents: String) -> Unit { builtin(fs_write, path, contents) }

    derive handler posix_fs;
}
```

Compiler generates:

```ash
handler posix_fs<A, r: Row>(
    comp: Unit -> {PosixFs::read, PosixFs::write | r} A
) -> {r} A
where requires host posix_fs
{
    on comp() {
        PosixFs::read(path, resume) => resume(PosixFs::read(path))
        PosixFs::write(path, contents, resume) => resume(PosixFs::write(path, contents))
        done(value) => value
    }
}
```

Installation:

```ash
handle load_config<PosixFs>("/etc/config") with posix_fs
// Desugars to: posix_fs(fn () -> load_config<PosixFs>("/etc/config"))
```

#### Form B: handler defined in impl (explicit, co-located)

```ash
impl Fs for PosixFs
where requires host posix_fs
{
    fn read(path: Path) -> String { builtin(fs_read, path) }
    fn write(path: Path, contents: String) -> Unit { builtin(fs_write, path, contents) }

    handler posix_fs<A, r: Row>(
        comp: Unit -> {PosixFs::read, PosixFs::write | r} A
    ) -> {r} A {
        on comp() {
            PosixFs::read(path, resume) => {
                log("reading {}", path);            // ← custom behavior in the clause
                resume(PosixFs::read(path))
            }
            PosixFs::write(path, contents, resume) => resume(PosixFs::write(path, contents))
            done(value) => value
        }
    }
}
```

Same type as the derived handler. Same installation. But the author controls the clause bodies
explicitly — useful when the handler needs logging, tracing, or custom logic alongside the
impl method call. The impl method bodies are still available inside the handler as
`PosixFs::read(path)` etc.

Installation is identical:

```ash
handle load_config<PosixFs>("/etc/config") with posix_fs
```

#### Form C: standalone handler function

```ash
handler posix_fs<A, r: Row>(
    comp: Unit -> {PosixFs::read, PosixFs::write | r} A
) -> {r} A
where requires host posix_fs
{
    on comp() {
        PosixFs::read(path, resume) => resume(PosixFs::read(path))
        PosixFs::write(path, contents, resume) => resume(PosixFs::write(path, contents))
        done(value) => value
    }
}
```

Identical semantics. The standalone form is what Forms A and B desugar to. It lives at module
level and references the impl's methods through normal name resolution.

#### What the IR sees (all three forms)

```
Raise  { op: EffectOp { impl_type: "PosixFs", operation: "read" }, args: [path], resume: k, ... }
Handle { clause: HandlerClause { op: EffectOp { "PosixFs", "read" }, params: [path], resume: k', body: ..., ... }, ... }
```

#### Multiple simultaneous handlers (the payoff)

```ash
handle
    handle
        load_config<MemoryFs>("test.toml")     // raises MemoryFs::read
        load_config<PosixFs>("/etc/config")    // raises PosixFs::read
    with memory_fs                              // catches MemoryFs::read only
with posix_fs                                    // catches PosixFs::read only
```

Both handlers active. Both calls dispatched correctly. No interference — distinct identities.

### 4.2 Escape handler (Exception, no resume) — answer type ≠ value type

#### Declaration

```ash
interface Exception {
    fn throw<E>(err: E) -> Unit;
}
```

#### Phantom types and impls

```ash
type Panic: Exception;
type CatchIO: Exception;

impl Exception for Panic {
    fn throw<E>(err: E) -> Unit { builtin(panic, err) }
}

impl Exception for CatchIO {
    fn throw<E>(err: E) -> Unit { builtin(panic, err) }   // default — never called by the handler
}
```

#### Standalone handler (escape — discards resume, transforms answer type)

```ash
handler catch_io<A, E>(
    comp: Unit -> {CatchIO::throw<E>} A
) -> Result<A, E> {
    on comp() {
        CatchIO::throw<E>(err, _resume) => Err(err)
        //                    ^^^^^^
        // Discarded — the computation short-circuits. Answer type is Result<A, E>, not A.
        done(value) => Ok(value)
    }
}
```

The answer type `Result<A, E>` differs from the computation's value type `A`. The `done` clause
wraps in `Ok`, the `throw` clause produces `Err`. The continuation is available but unused.

#### Handler defined in impl (co-located escape handler)

```ash
impl Exception for CatchIO {
    fn throw<E>(err: E) -> Unit { builtin(panic, err) }

    handler catch_io<A, E>(
        comp: Unit -> {CatchIO::throw<E>} A
    ) -> Result<A, E> {
        on comp() {
            CatchIO::throw<E>(err, _resume) => Err(err)
            done(value) => Ok(value)
        }
    }
}
```

Same semantics, co-located with the impl. The impl method body exists for identity and
type-checking; the handler clause overrides behavior.

#### Installation

```ash
handler risky_parse(text: String) -> {CatchIO::throw<ParseError>} Ast { ... }

handle risky_parse(input) with catch_io
// Desugars to: catch_io(fn () -> risky_parse(input))
// Returns Result<Ast, ParseError>
```

#### What the IR sees

```
Raise  { op: EffectOp { impl_type: "CatchIO", operation: "throw" }, args: [err], resume: k, ... }
Handle { clause: HandlerClause { op: EffectOp { "CatchIO", "throw" }, ..., body: <jump to outer cont with Err(err)> }, ... }
```

The resume continuation `k` is never jumped to. The handler clause jumps to the outer
continuation with `Err(err)`, short-circuiting the computation.

### 4.3 Multi-shot handler (Choice, all solutions) — pure continuation

#### Declaration

```ash
interface Choice {
    fn choose<A>(opts: List<A>) -> A;
}
```

#### Phantom types and impls

```ash
type FirstChoice: Choice;
type AllSolutions: Choice;

impl Choice for FirstChoice {
    fn choose<A>(opts: List<A>) -> A { opts.head() }
}

impl Choice for AllSolutions {
    fn choose<A>(opts: List<A>) -> A { opts.head() }   // default — overridden by handler
}
```

#### Standalone handler (multi-shot — calls resume multiple times)

```ash
handler all_solutions<A>(
    comp: Unit -> {AllSolutions::choose} A
) -> List<A> {
    on comp() {
        AllSolutions::choose(xs, resume) =>
            xs.flat_map(fn x -> resume(x))
        //          ^^^^^^^^^^^^^^^^^^
        // resume called once per element. Legal because the continuation's row is {}
        // (pure). The type checker enforces: resume : A -> {} List<A>.
        done(value) => [value]
    }
}
```

The continuation's row is empty (the computation requires only `{AllSolutions::choose}`, which
the handler peels). A pure continuation is copyable — multi-shot is legal. The answer type is
`List<A>` — results from all branches are collected.

#### Handler defined in impl (co-located multi-shot handler)

```ash
impl Choice for AllSolutions {
    fn choose<A>(opts: List<A>) -> A { opts.head() }

    handler all_solutions<A>(
        comp: Unit -> {AllSolutions::choose} A
    ) -> List<A> {
        on comp() {
            AllSolutions::choose(xs, resume) =>
                xs.flat_map(fn x -> resume(x))
            done(value) => [value]
        }
    }
}
```

#### Derive for the deep variant of the same impl

```ash
impl Choice for FirstChoice {
    fn choose<A>(opts: List<A>) -> A { opts.head() }
    derive handler first_choice;
}
// Compiler synthesizes:
// handler first_choice<A, r: Row>(comp: Unit -> {FirstChoice::choose | r} A) -> {r} A {
//     on comp() {
//         FirstChoice::choose(xs, resume) => resume(FirstChoice::choose(xs))
//         done(value) => value
//     }
// }
```

#### Installation (both handlers, same interface, different impls)

```ash
handle search<FirstChoice>(tree) with first_choice    // returns Maybe<A> (first solution)
handle search<AllSolutions>(tree) with all_solutions   // returns List<A> (all solutions)
```

#### What the IR sees

```
// search<AllSolutions> monomorphized:
Raise  { op: EffectOp { impl_type: "AllSolutions", operation: "choose" }, args: [xs], resume: k, ... }
Handle { clause: HandlerClause { op: EffectOp { "AllSolutions", "choose" }, ..., body: <flat_map resume over xs> }, ... }
// k is MultiShotPure — row normalizes to {} — may be invoked multiple times.
```

## 5. Lowering

### 5.1 Monomorphization

Generic code with sort constraints (`F: Fs`) is monomorphized at each call site where `F` is
concrete. The monomorphization pass:

1. Resolves `F` to a concrete impl type (e.g., `PosixFs`).
2. Rewrites row items: `F::read` → `PosixFs::read`.
3. Rewrites operation calls: `F.read(path)` → `PosixFs::read(path)` (raises
   `EffectOp { PosixFs, read }`).
4. Inlines the impl method body at the call site (or keeps it as a function reference).

After monomorphization, the Core/IR sees only concrete operation identities. No abstract
sorts, no type parameters in row items.

### 5.2 The IR is unchanged

The CPS IR (`Raise`, `Handle`, `HandlerClause`, `Value::Cont`) is unchanged. The only
difference from the previous model is the *value* in `EffectOp.impl_type`: it's the impl
type name (`PosixFs`) instead of the interface name (`Fs`).

### 5.3 Row accounting

The row transformation rules (SPEC-098b §5.5) are unchanged. Installing a handler for
`PosixFs::read` removes `PosixFs::read` from the body's row and adds the handler's own
effects. The residual row propagates to the handler's caller.

## 6. What Changes from NOTE-022

| Aspect | NOTE-022 (concrete name) | NOTE-025 (sort/impl) |
|---|---|---|
| Row item | `Fs.read` | `F::read` (abstract), `PosixFs::read` (concrete) |
| Identity qualifier | Interface name | Impl type (the `for` parameter — can be empty or data-carrying) |
| Multiple simultaneous handlers | ❌ Same identity | ✅ Distinct identities per impl |
| Generic code | Row polymorphism only | Type parameter `F: Fs` + row polymorphism |
| Monomorphization | Not needed | Required — generates concrete identities |
| IR `EffectOp` | `{ interface: "Fs", operation: "read" }` | `{ impl_type: "PosixFs", operation: "read" }` |
| Interface role | Declaration site + identity qualifier | Sort (abstract family + laws) only |
| Impl role | Not part of the effect model | Identity carrier + default behavior |

## 7. Open Questions

1. **Impl type declaration.** The impl type can be empty (`type PosixFs = Unit;`) or
   data-carrying (`type ConfiguredFs = { root: Path, readonly: Bool };`). Is there a dedicated
   syntax for empty identity-only types, or is `type Name = Unit;` sufficient? Should sort
   annotation be available: `type PosixFs: Fs;`?

2. **Derive naming convention.** `derive handler posix_fs;` — is the name always explicit,
   or can it be inferred from the impl type name? If inferred, what's the convention?

3. **Derive scope.** Does `derive handler` always generate clauses for ALL interface methods,
   or can it target a subset? `derive handler { read };` for partial handlers?

4. **Multiple handlers per impl.** Can an impl define multiple named handlers (e.g., both a
   deep `first_choice` and a multi-shot `all_solutions` for the same `AllSolutions` type)?
   Or one handler per impl?

5. **Sort constraints in rows.** Is `{F::read | r}` the right spelling, or should it be
   `{<F as Fs>::read | r}` for disambiguation when `F` implements multiple interfaces with
   same-named methods?

6. **Impl-less types.** Is it a hard error to reference `TypeName::op` without an
   impl, or is there a diagnostic pathway?

7. **Coherence.** Can two impls of the same interface for the same impl type exist in
   different modules? Or is there a global uniqueness constraint (like Rust trait coherence)?

8. **Dynamically dispatched handlers.** For effects where monomorphization is undesirable
   (plugin systems, dynamic loading), is there a non-monomorphic path? Boxed trait objects?
   Or is monomorphization always required?

## 8. Working Principle

```text
An interface is an effect sort: it declares signatures, generics, and laws.
An impl type is the identity carrier: the impl type parameter is the operation identity.
The impl type can be empty (phantom) or carry data — the type system does not restrict which.
The impl method body is the default deep-handler behavior.
A handler is a named function: (Unit -> {op | r} A) -> {r} Ans.
Three ways to produce a handler: derive (compiler-synthesized), in-impl (co-located), standalone.
handle expr with name is always function application — name is a function value.
Monomorphization produces concrete identities from abstract sort constraints.
The CPS IR is unchanged — it sees concrete EffectOp identities.
```

## 9. References

Internal references:

- [NOTE-022: Effects as Interfaces — Declaration Side](NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md)
- [NOTE-023: Handler Surface — Dispatch Side](NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md)
- [NOTE-013: Ambient Monad and Handler Composition Algebra](NOTE-013-AMBIENT-MONAD-AND-HANDLER-COMPOSITION-ALGEBRA.md)
- [NOTE-024: Host/FFI and Extern](NOTE-024-HOST-FFI-AND-EXTERN.md)
- [SPEC-095b: Target Grammar](../spec/SPEC-095b-TARGET-GRAMMAR.md)
- [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-098b: Target CPS IR](../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-102: CPS Continuation Multiplicity](../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md)

External references:

- Leijen, "Koka: Programming with Row-Polymorphic Effects" (2014).
  https://www.microsoft.com/en-us/research/wp-content/uploads/2016/08/koka-technical.pdf
- Lindley, McBride & McLaughlin, "Do Be Do Be Do" (2017).
  https://doi.org/10.1145/3064898
- Plotkin & Power, "Computational Effects as Operations" (2002).
  https://www.sciencedirect.com/science/article/pii/S0304397502004449

## 10. Changelog

- 2026-06-27: Initial version. Establishes the sort/impl identity model: interfaces are sorts,
  impl types are identity carriers, handler functions are named values. Records the derive
  mechanism, handler-in-impl option, and three handler installation examples (deep, escape,
  multi-shot). Revises the NOTE-022 concrete-name model to the sort/impl model.
- 2026-06-27: Corrected — the impl type is not restricted to phantom types. It can be empty
  (identity-only) or carry data (configuration, state, connection parameters). Added a
  data-carrying `ConfiguredFs` example. The identity comes from the type itself, not from
  emptiness.
