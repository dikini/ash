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
  type is the operation identity — it can be bodyless (nominal, no constructors) or carry data. This is new
  surface syntax.
- **Handler derivation:** `derive handler <name>` is a new declaration inside impl bodies.
- **Handler installation:** `handle expr with <name>` always resolves to function
  application. `<name>` is always a value-namespace function. No type-namespace resolution.
- **SPEC-095b/097b/098b:** the `OperationEffect` identity changes from `{interface, operation}`
  to `{impl_type, operation}` — the concrete impl type replaces the interface as the
  identity qualifier after specialization.
- **Bodyless type declarations (grammar delta):** `type PosixFs;` — a type declaration with
  no `=` and no body — introduces a nominal type with no constructors. It has identity but
  cannot be constructed. This is distinct from a transparent alias `type PosixFs = Unit;`,
  which canonicalizes to `Unit` at definitional equality (SPEC-058/SPEC-100) and would
  collapse all identity-only types into one identity. The current grammar requires a body
  (`type_definition = "type" identifier [type_params] "=" type_body ";"`); the delta makes
  the `= type_body` optional. See §7 Q1. Phantom types and newtype-like nominal forms
  carrying type parameters are a related but separate deferred type-system enhancement —
  see §7 Q1 "Deferred follow-up."
- **Handler marker (type-level attribute):** `handler` is no longer a pure keyword alias
  for `fn`. It produces a function whose type carries a `handler` marker — a type-level
  attribute identifying handler intent, analogous to comp mode (eager/lazy/memo). Derive
  uses it to filter operations from handlers; `handle expr with name` validates it. The
  underlying function type is structurally identical; the marker is erased at runtime. See
  NOTE-023 §7 for the full grammar, typing, subtyping, and worked examples.

## 0. Motivation

NOTE-022 settled that operations are declared as interface methods. The row item was `Fs.read`
— the interface name qualifying the operation. This worked for type-checking but had a
semantic problem: the interface is a grouping (namespace + laws + shared generics), not a
dispatch mechanism. Multiple handlers for `Fs.read` cannot coexist because they share one
identity.

The sort/impl model separates the abstraction layers:

```
Interface = sort (abstract effect family + laws)
Impl type = identity carrier (the impl type — bodyless nominal or data-carrying)
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

    -- Example law: reading immediately after writing returns the written value.
    -- Law syntax is not yet finalized; shown here as an illustrative constraint.
    law read_after_write
}
```

### 1.2 Impl type + impl as identity carrier

The impl type is the operation identity. It can be bodyless (a nominal type with no
constructors, used purely for compile-time identity) or it can carry data (configuration,
state, connection parameters via a record body). The type system does not restrict which.
Every effect operation that appears in a row must have a concrete impl behind it — for
method resolution and type-checking. The impl type parameter is the operation identity.

**Important — not a transparent alias.** A bodyless type declaration `type PosixFs;` is a
nominal type: it declares a new type with no constructors that cannot equal any other type.
This is distinct from a transparent alias `type PosixFs = Unit;`, which canonicalizes to
`Unit` at definitional equality (per SPEC-058/SPEC-100) and would collapse all identity-only
types into one identity. See §7 Q1 for the full grammar delta.

```ash
// Bodyless nominal type — unconstructable, identity-only. Not a transparent alias.
type PosixFs;

impl Fs for PosixFs
where requires host posix_fs
{
    fn read(path: Path) -> String { builtin(fs_read, path) }
    fn write(path: Path, contents: String) -> Unit { builtin(fs_write, path, contents) }
}

// Data-carrying type — nominal record, carries runtime configuration.
// Identity + config in one type. A record body is nominal (not a transparent alias).
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

type PosixFs;
type MemoryFs;

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

#### Identity types and impls

```ash
type Panic;
type CatchIO;

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

#### Identity types and impls

```ash
type FirstChoice;
type AllSolutions;

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
| Identity qualifier | Interface name | Impl type (the `for` parameter — bodyless nominal or data-carrying) |
| Multiple simultaneous handlers | ❌ Same identity | ✅ Distinct identities per impl |
| Generic code | Row polymorphism only | Type parameter `F: Fs` + row polymorphism |
| Monomorphization | Not needed | Required — generates concrete identities |
| IR `EffectOp` | `{ interface: "Fs", operation: "read" }` | `{ impl_type: "PosixFs", operation: "read" }` |
| Interface role | Declaration site + identity qualifier | Sort (abstract family + laws) only |
| Impl role | Not part of the effect model | Identity carrier + default behavior |

## 7. Resolved Decisions

All eight open questions from the initial draft are now resolved. Each decision records the
grammar, type, semantic, and worked-example consequences.

### 7.1 Q1 — Impl type declaration: bodyless nominal type

**Decision:** Empty identity-only types use `type PosixFs;` — a bodyless type declaration
(no `=`, no body). It declares a new nominal type with no constructors: it has identity but
cannot be constructed. This is the minimal form sufficient for the effect model.

**Grammar delta.** The current grammar requires a body:

```ebnf
type_definition = "type" identifier [ type_params ] "=" type_body ";" ;
```

The delta makes `= type_body` optional:

```ebnf
type_definition = "type" identifier [ type_params ] [ "=" type_body ] ";" ;
```

Without `=`, the type is an opaque nominal unit — no representation, no constructors,
identity-only. With `=` and a `type_body` (enum, struct, record, tuple), the existing
nominal forms are unchanged. With `=` and an `alias_body` (a bare type), the existing
transparent alias is unchanged.

**Critical: not a transparent alias.** `type PosixFs = Unit;` is a transparent alias.
Per SPEC-058/SPEC-100, transparent aliases canonicalize to their origin head at
definitional equality. So `type PosixFs = Unit` makes `PosixFs ≡ Unit` — the type checker
canonicalizes one to the other. This silently breaks the identity model: `PosixFs::read`
and `MemoryFs::read` both canonicalize to `Unit::read` → identities collide. The bodyless
form avoids this because a nominal type with no body is never equated to any other type.

**No sort annotation at the type site.** `type PosixFs: Fs;` is rejected. The interface
relationship lives in the `impl Fs for PosixFs` block, not the type declaration. Coupling
type declaration to interface membership would create redundancy (the relationship is
already in `impl`) and coherence coupling (must the type and interface be co-located?).

**Deferred follow-up — phantom types and newtype.** A transparent alias
`type F<A> = Unit` does not give distinct phantom identities either: the phantom parameter
`<A>` is erased at definitional equality (`PosixFs<Int> ≡ PosixFs<String> ≡ Unit`). True
phantom types and newtype-like nominal forms that carry type parameters without equating
to a representation are one deferred type-system enhancement — not needed for the current
effect model, where bodyless types suffice for identity.

**Worked example:**

```ash
type PosixFs;        // bodyless nominal — identity-only, unconstructable
type ConfiguredFs = { root: Path, readonly: Bool };  // data-carrying, nominal record
```

### 7.2 Q2 — Derive naming: always explicit

**Decision:** `derive handler <name>;` — the name is always explicit. The compiler never
infers the handler name from the impl type name.

**Rationale.** A derived handler is a named function in the value namespace. It participates
in normal name resolution, shadowing, and import/export. It needs an explicit name because:

1. Value-namespace functions must have explicit names — there is no implicit transformation
   rule from type-name conventions (PascalCase `PosixFs`) to value-name conventions
   (snake_case `posix_fs`).
2. One impl may derive multiple handlers (§7.4), so a single inferred name is insufficient.
3. Explicit over implicit: the declaration site visually states what enters the value
   namespace.

Rust's `#[derive(Debug)]` does not name the result because the derived impl is in the type
namespace (accessed via `<T as Debug>`). Ash's derived handler is in the value namespace —
it is a callable function — so it must have an explicit name.

**Worked example:**

```ash
impl Fs for PosixFs
where requires host posix_fs
{
    fn read(path: Path) -> String { builtin(fs_read, path) }
    fn write(path: Path, contents: String) -> Unit { builtin(fs_write, path, contents) }

    derive handler posix_fs;    // explicit name — enters value namespace as `posix_fs`
}
```

### 7.3 Q3 — Derive scope: total fold over all operations

**Decision:** `derive handler` always generates clauses for ALL interface methods. It is the
total, mechanical deep handler — the identity interpretation (semantic unit of the handler
algebra). There is no subset derive (`derive handler { read };` is not supported).

**Rationale — three angles.**

1. **Semantic.** Derive is the total fold over the free monad generated by the effect
   signature. Every operation gets `resume(ImplType::op(args))`. There are no choices to
   make — which is exactly what makes it synthesizable. A partial derive would require the
   compiler to make a semantic decision ("intercept these, let those escape"), which belongs
   in explicit code.

2. **Type-theoretic.** A total derive produces a handler whose residual row is just `r`:

   ```
   handler posix_fs<A, r: Row>(
       comp: Unit -> {PosixFs::read, PosixFs::write | r} A
   ) -> {r} A        // ← all Fs operations peeled
   ```

   A partial derive (only `read`) would have a structurally different type — `PosixFs::write`
   survives in the residual row. This is a different position in the handler stack, not "a
   derive that handles less." Collapsing both under `derive` muddies what the keyword
   promises about the residual row.

3. **Practical.** Partial behavior is already covered by explicit handlers (Form B/C). The
   non-overridden clauses are trivial one-liners delegating to impl method bodies. No
   feature gap.

**Feature space coverage:**

| Need | Mechanism |
|---|---|
| All default, no customization | `derive handler name;` (total fold) |
| Override some, delegate rest | Explicit handler in impl (Form B) |
| Override some, let rest escape | Explicit handler, omit clauses for escaping ops |
| Fully custom (escape, multi-shot) | Explicit standalone handler (Form C) |

### 7.4 Q4 — Multiple handlers per impl: yes, unbounded

**Decision:** An impl block may define as many named handlers as needed, plus at most one
`derive handler` (the canonical total fold). There is no "one handler per impl" limit.

**Rationale.** Handlers are named functions in the value namespace, distinguishable by the
**handler marker** — a type-level attribute on their function type (see NOTE-023 §7). An
impl block can declare multiple `handler` blocks because they produce separate value-namespace
bindings with distinct names. No coherence conflict arises: they don't share an identity.

**How derive filters.** The derive mechanism must fold over operations only, not handlers.
Because the handler marker is carried in the type system, derive filters by checking the
marker: members without the marker are operation candidates; members with the marker are
skipped. This works even across module boundaries because the marker survives into module
summaries.

**Constraint:** ordinary name uniqueness within a scope. Two handlers in the same impl block
cannot share a name — the standard function-shadowing rule, nothing effect-specific.

**Worked example — one impl, two handlers:**

```ash
impl Choice for AllSolutions {
    fn choose<A>(opts: List<A>) -> A { opts.head() }

    derive handler all_solutions_deep;   // total fold, returns A

    handler all_solutions<A>(             // explicit multi-shot, returns List<A>
        comp: Unit -> {AllSolutions::choose} A
    ) -> List<A> {
        on comp() {
            AllSolutions::choose(xs, resume) => xs.flat_map(fn x -> resume(x))
            done(value) => [value]
        }
    }
}
```

Both `all_solutions_deep` and `all_solutions` are in the value namespace. The caller
chooses which to install:

```ash
handle search<AllSolutions>(tree) with all_solutions_deep   // returns A
handle search<AllSolutions>(tree) with all_solutions        // returns List<A>
```

### 7.5 Q5 — Sort constraints in rows: `{F::read | r}` always sufficient

**Decision:** `{F::read | r}` is always sufficient. The fully-qualified `<F as Fs>::read`
syntax is not needed and not introduced.

**Rationale.** This is a consequence of the coherence decision (§7.7). Under strong coherence,
identity is `{impl_type, operation}` with no interface field. Coherence ensures that for any
type `T`, each operation name may be defined by at most one interface. Therefore `T::op`
resolves to exactly one method — there is never ambiguity to disambiguate.

If `PosixFs` implements both `Fs` (with `read`) and `Other` (also with `read`), coherence
forbids this (§7.7, rule 2). The collision is prevented at the impl site, not papered over
with verbose row syntax.

The `<F as Fs>` fully-qualified syntax (Rust-style) is powerful but verbose, and only needed
in rare disambiguation cases that strong coherence prevents entirely. If a future use case
genuinely requires same-named operations across interfaces for one type, the fully-qualified
syntax can be added without breaking existing code.

### 7.6 Q6 — Impl-less types: hard error

**Decision:** Referencing `TypeName::op` without an impl is a hard type error. There is no
diagnostic pathway (soft warning, deferred resolution).

**Rationale.** Without an impl, there is:
- No method body — nothing to call or inline during monomorphization.
- No operation identity — the impl type IS the identity, but the operation doesn't exist for
  that type without an impl binding it to an interface.
- No handler clause to synthesize — `derive handler` has nothing to derive from.

The downstream monomorphization pass cannot proceed without a concrete method body. The
error message should name both the type and the missing interface:

```
Error: Type `PosixFs` has no implementation of interface `Fs`.
       Cannot reference operation `PosixFs::read`.
       Hint: add `impl Fs for PosixFs { ... }`.
```

This is the same class of error as Rust's "the trait bound `T: Fs` is not satisfied."

### 7.7 Q7 — Coherence: global uniqueness, stricter than Rust

**Decision:** Global uniqueness, strictly stronger than Rust's trait coherence.

Under the identity model `{impl_type, operation}` (no interface field, per §7.5), coherence
must ensure:

1. **Per (type, interface) pair:** at most one impl globally — same as Rust/Haskell.
2. **Per (type, operation-name) pair:** at most one interface may declare that operation name
   for that type — *stronger* than Rust. Rust allows `Display::fmt` and `Debug::fmt` to
   coexist because the identity includes the trait. Ash's identity drops the interface, so
   the collision must be prevented structurally.

Rule 2 is the consequence of dropping the interface from the identity. It is unusual but
simple to state: *"For any type T, each operation name may be defined by at most one
interface."* After monomorphization, `T::op` must resolve to exactly one method body. Two
impls would produce two bodies for the same identity — unsound.

**Orphan rule:** follow Rust — an `impl Fs for T` must be in the same crate/module as either
`Fs` or `T`. This prevents two independently developed modules from each defining
`impl Fs for PosixFs` and producing a coherence conflict at link time.

**Worked example — forbidden:**

```ash
interface Fs       { fn read(path: Path) -> String; }
interface Cache    { fn read(key: Key) -> Bytes; }  // same operation name!

type PosixFs;

impl Fs    for PosixFs { fn read(path: Path) -> String { ... } }
impl Cache for PosixFs { fn read(key: Key) -> Bytes { ... } }
// ERROR: operation name `read` is already defined for type `PosixFs` by interface `Fs`.
// Coherence rule 2: at most one interface may declare `read` for a given type.
```

This is conservative but natural for a statically monomorphic identity model.

### 7.8 Q8 — Dynamic dispatch: deferred, bridge via vtable-in-method-body

**Decision:** Defer dynamic dispatch. Monomorphization is the sole path. The bridge to
runtime dynamism is a data-carrying impl type whose method bodies call through a vtable —
no new language feature needed.

**Rationale.** The current sort/impl model is statically monomorphic: the identity is a
concrete type name known at compile time, the method body is inlined, and the handler clause
matches on a concrete identity string. Dynamic dispatch breaks all three: the impl type isn't
known at the call site (loaded at runtime), the method body can't be inlined (behind a
vtable), and the handler clause must match at runtime (dynamic identity comparison).

This is a fundamentally different dispatch model requiring existentials/erased types,
runtime identity scheme, and row-system interaction (rows are compile-time). It is out of
scope for the current design.

**Bridge pattern — no new feature needed.** A plugin adapter is a concrete identity carrier
whose method bodies delegate to a runtime vtable:

```ash
type PluginFs = { vtable: FsVtable };   // data-carrying, nominal record

impl Fs for PluginFs {
    fn read(path: Path) -> String { self.vtable.read(path) }
    fn write(path: Path, contents: String) -> Unit { self.vtable.write(path, contents) }
}
```

The identity `PluginFs::read` is a concrete, compile-time-known identity. The dynamism lives
entirely inside the method body (the vtable call), not in the dispatch model. This keeps the
identity model intact while pushing dynamism into the data layer — the same pattern Rust
uses internally with trait objects.

## 8. Working Principle

```text
An interface is an effect sort: it declares signatures, generics, and laws.
An impl type is the identity carrier: the impl type parameter is the operation identity.
The impl type can be bodyless (nominal, no constructors) or carry data — the type system does not restrict which.
A bodyless type is `type T;` (nominal). It is NOT `type T = Unit;` (transparent alias — collapses identity).
The impl method body is the default deep-handler behavior.
A handler is a named function: (Unit -> {op | r} A) -> {r} Ans.
A handler function's type carries a handler marker — a type-level attribute (like comp mode).
Three ways to produce a handler: derive (total fold), in-impl (co-located), standalone.
derive handler <name> folds over ALL operations — filtered by absence of the handler marker.
Multiple handlers per impl are allowed — distinguished by the handler marker.
handle expr with name is function application — name must carry the handler marker.
Coherence is global and stricter than Rust: per (type, op-name) uniqueness, not just per (type, interface).
Dynamic dispatch is deferred — bridge via data-carrying vtable impl type, no new feature needed.
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
- 2026-06-27: Corrected — the impl type is not restricted to bodyless types. It can be empty
  (identity-only) or carry data (configuration, state, connection parameters). Added a
  data-carrying `ConfiguredFs` example. The identity comes from the type itself, not from
  emptiness.
- 2026-06-28: Resolved all eight §7 open questions. Key decisions: (Q1) bodyless nominal
  type `type PosixFs;` replaces transparent alias `type PosixFs = Unit;` which collapses
  identity; no sort annotation at type site; phantom types/newtype deferred. (Q2) derive
  naming always explicit. (Q3) derive is the total fold over all operations. (Q4) multiple
  handlers per impl allowed — distinguished by the handler marker (see NOTE-023 §7). (Q5)
  `{F::read | r}` always sufficient — strong coherence eliminates ambiguity. (Q6) impl-less
  reference is a hard error. (Q7) global coherence stricter than Rust — per (type, op-name)
  uniqueness. (Q8) dynamic dispatch deferred — bridge via data-carrying vtable impl type.
  Updated Pre-Spec Delta with grammar delta for bodyless types and handler marker reference.
  Swept all type declarations to bodyless form throughout.
