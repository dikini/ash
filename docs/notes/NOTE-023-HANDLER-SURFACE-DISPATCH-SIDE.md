# NOTE-023: Handler Surface — Dispatch Side Design

**Date:** 2026-06-27
**Status:** Living document — dispatch-side direction captured; open questions tracked
**Purpose:** Record the surface grammar for handler dispatch: how handlers consume
computations, how the continuation is threaded, how multiplicity is expressed, and how
installation works. This note completes the declaration/dispatch separation begun in
NOTE-022 by specifying the dispatch side.

Companion to NOTE-022 (declaration side: interfaces), NOTE-013 (handler composition
algebra), NOTE-018 (boundary discipline), and NOTE-019 (convergence plan).

## Pre-Spec Delta

This note is pre-spec. When the project moves to spec updates, reconcile:

- **Handler grammar:** NOTE-023 introduces `on`, `handle...with`, and `handler Name for
  Interface` surface forms. SPEC-095b does not yet describe them.
- **Continuation parameter convention:** NOTE-023 establishes the continuation as an
  ordinary function-typed parameter, not a magic keyword. SPEC-095b has no handler clause
  grammar to reconcile yet.
- **`done` clause:** NOTE-023 introduces `done(value)` as the computation-completion
  pattern. The specs do not yet describe handler clauses.
- **Multiplicity via function type:** NOTE-023 derives multiplicity from the continuation's
  function row rather than a dedicated annotation. This is consistent with SPEC-102's
  Core/CPS encoding but the surface spelling is new.

## 0. Motivation

NOTE-022 settled the declaration side: operations are interface methods, dispatch is Handle
frame nesting, authority is admission. What remained open was the surface grammar for
writing and installing handlers — the dispatch side.

The design constraints are:

1. **No magic syntax for the continuation.** The captured continuation is an ordinary
   function-typed parameter, named by the author, typed by the type system. No `resume`
   keyword, no implicit binding, no wrapper type.
2. **One clause shape.** There is no Koka-clause vs Frank-clause distinction. Both
   installation forms desugar to the same clause structure.
3. **LLM-legibility.** Keywords should pattern-match against established concepts in the
   algebraic effects literature. An LLM (or human) seeing handler code should immediately
   classify the construct.
4. **Sugar is optional and layered.** The explicit form (bare function + `on`) always works.
   Sugar forms (`handle...with`, named handlers) desugar to it.

## 1. The Handler Function

A handler is an ordinary function whose parameter is a computation thunk:

```ash
handler posix_fs<A, r>(comp: Unit -> {PosixFs::read | r} A) -> {r} A {
    on comp() {
        done(value) => value

        PosixFs::read(path, resume) => {
            resume(PosixFs::read(path))
        }
    }
}
```

Type breakdown:

- `comp : Unit -> {PosixFs::read | r} A` — the computation to be interpreted. Its row includes
  the operation this handler peels (`PosixFs::read`) plus a tail `r` for effects the handler
  does NOT peel.
- `A` — the computation's value type AND the handler's answer type.
- `{r}` — the residual row: the handler removes `PosixFs::read` from the row and contributes
  only what its own clauses raise (nothing here, so the output row is `r`).

The handler function's signature IS the row-peeling contract: input row
`{PosixFs::read | r}`, output row `{r}`.

**NOTE-025 revision:** Row items are impl-type-qualified (`PosixFs::read`), not
interface-qualified (`Fs.read`). See NOTE-025 for the full identity model.

## 2. The `on` Eliminator

`on` is the computation eliminator — the dual of `do`. Where `do` produces computations
(sequences operations via `bind`), `on` consumes them (interprets operations via handler
clauses).

```ash
on comp() {
    done(value) => result_expr

    ImplType::method(args, resume) => result_expr
    ImplType::other_method(args, resume) => result_expr
    ...
}
```

**NOTE-025 revision:** Clause identities are impl-type-qualified (e.g., `PosixFs::read`), not
interface-qualified. In generic code before monomorphization, this may be abstract:
`F::read` where `F: Fs`. After monomorphization, the concrete impl type qualifies the
operation.

Lowering: `on comp()` installs a `Handle` frame in the CPS IR, then invokes `comp(())`.
When `comp` raises an operation, the runtime searches the Handle frame's clauses for a
matching operation identity. When `comp` completes normally (reaches its tail), the `done`
clause fires.

**The `done` clause is required.** Every handler must specify what happens when the
computation finishes normally. The name `done` is chosen to avoid collision with `return`
(which belongs to `do` notation as the ambient monad's unit). `done(value)` receives the
computation's final value (type `A`).

## 3. The Continuation Is an Ordinary Parameter

In each operation clause, the continuation is a parameter — not a keyword, not a magic
binding. The author names it:

```ash
PosixFs::read(path, resume) => resume(decode_utf8(bytes))
PosixFs::read(path, k)      => k(decode_utf8(bytes))
PosixFs::read(path, cont)   => cont(decode_utf8(bytes))
```

All three are identical. The parameter's type is derived from:

- **Input type:** the operation's result type from the interface (`String` for
  `fn read(path: Path) -> String`).
- **Row:** the remaining computation's effects — the original row minus the handled
  operation (SPEC-098b §5.5: `captured_resume.row = handled_segment.local - handled_op`).
- **Output type:** the handler's answer type (`A`).

So for `posix_fs` above, `resume : String -> {r} A`. Calling `resume(x)` is ordinary
function application. The runtime provides the function value (a `Value::Cont`) when it
dispatches to the clause.

### Naming convention

- **`resume`** in surface handler examples and documentation — self-documenting, maps to
  the algebraic effects literature, strongest LLM pattern-match signal.
- **`k`** in formal CPS discussion, type proofs, and IR-level documentation — continuation
  monad convention.

The docs should state this convention once. Both refer to the same captured continuation.

## 4. Multiplicity via Function Type

Multiplicity is not a separate annotation. It falls out of the continuation's function type:

**Affine (default):** the continuation's row is non-empty (`{r}` where `r ≠ {}`). An
effectful function is affine — consumed on first use. Calling it twice is a type error.
This covers State, Reader, Writer, Fs, most handlers.

```ash
PosixFs::read(path, resume) => {
    resume(read_from_disk(path))    // resume : String -> {log.write, fail Error} A
    // resume(x) again here would be a type error — affine, already consumed
}
```

**Multi-shot (pure):** the continuation's row is `{}`. A pure function is copyable — it can
be stored, called multiple times, passed to higher-order functions. This covers
nondeterminism, backtracking, all-solutions search.

```ash
AllSolutions::choose(xs, resume) => {
    // resume : A -> {} Ans — pure, copyable
    xs.flat_map(fn x -> resume(x))  // called once per element — legal
}
```

SPEC-102's `ContMultiplicity` (`Affine` / `MultiShotPure`) is the Core/CPS-level encoding of
this ordinary type-system distinction. The `MultiShotPure` legality gate (row must normalize
to `{}`) is enforced by the type checker as a standard purity check — no surface annotation
needed.

This eliminates: ~~`resume: multi`~~, ~~`Resume<T>`~~, ~~`Multi<T>`~~ wrapper types,
~~multiplicity annotations on clauses~~.

## 5. One Clause Shape

There is no Koka-clause vs Frank-clause distinction. Every operation clause has the same
shape:

```ash
ImplType::method(op_args, continuation) => body
```

- `ImplType::method` — the impl-type-qualified operation identity, resolved through normal
  module/name resolution and monomorphization (NOTE-025).
- `op_args` — the operation's parameters, taken from the interface method signature.
- `continuation` — the captured continuation, an ordinary function-typed parameter.
- `body` — handler clause body, type-checks against the handler's answer type.

The Koka/Frank distinction lives entirely at the surrounding form — how the handler is
installed around the computation. The clauses inside are identical regardless.

## 6. Installation Forms

### Form 1: Explicit function application (Frank-style)

The handler is a function. Installation is ordinary function application:

```ash
posix_fs(fn () -> read_config("app.toml"))
```

The thunk `fn () -> read_config("app.toml")` captures the computation to be handled. The
handler function receives it as `comp` and uses `on comp() { ... }` to install the Handle
frame.

### Form 2: `handle...with` sugar (Koka-recognizable)

```ash
handle read_config("app.toml") with posix_fs
```

desugars to:

```ash
posix_fs(fn () -> read_config("app.toml"))
```

This sugar is purely syntactic — it wraps the expression in a thunk and applies the handler
function. It exists for readability and LLM-legibility: `handle...with` is immediately
recognizable as scoped effect handling.

Both forms are always available. Neither introduces a different clause shape.

## 7. Handler Declaration: `handler` as Alias for `fn`

`handler` is a pure keyword alias for `fn` (NOTE-023 §6 design discussion, NOTE-025 §2.2).
It carries no semantic difference — the compiler treats it identically to `fn`. It signals
intent to humans and LLMs: "this function is meant to be used as a handler."

```ash
handler posix_fs<A, r>(comp: Unit -> {PosixFs::read | r} A) -> {r} A
where requires host posix_fs
{
    on comp() {
        done(value) => value
        PosixFs::read(path, resume) => resume(PosixFs::read(path))
    }
}
```

This IS a function. Type: `(Unit -> {PosixFs::read | r} A) -> {r} A`. Nothing special except
the keyword. The answer type is the return type — no hidden `A`/`Ans` conflation, no
annotation needed.

**Answer-type transformation** is trivial — the return type is the answer type:

```ash
handler catch_throw<E, A, r>(comp: Unit -> {CatchIO::throw<E> | r} A) -> {r} Result<A, E> {
    on comp() {
        done(value) => Ok(value)
        CatchIO::throw<E>(err, _resume) => Err(err)
    }
}
```

### 7.1 Derive: compiler-synthesized deep handlers

When the handler's behavior is "compute result from impl, resume with it" (the deep handler
case), the impl can declare a derivation so the compiler synthesizes the handler function:

```ash
impl Fs for PosixFs
where requires host posix_fs
{
    fn read(path: Path) -> String { builtin(fs_read, path) }
    fn write(path: Path, contents: String) -> Unit { builtin(fs_write, path, contents) }

    derive handler posix_fs;
}
```

The compiler synthesizes the deep handler (the explicit form above) from the impl's method
bodies. See NOTE-025 §3-4 for the full derivation mechanics and worked examples for deep,
escape, and multi-shot handlers.

### 7.2 Handler defined in impl (co-located)

An explicit handler can be defined inside the impl body for DX — same semantics as a
standalone handler, but co-located with the impl it references:

```ash
impl Fs for PosixFs
where requires host posix_fs
{
    fn read(path: Path) -> String { builtin(fs_read, path) }

    handler posix_fs<A, r>(comp: Unit -> {PosixFs::read | r} A) -> {r} A {
        on comp() {
            PosixFs::read(path, resume) => {
                log("reading {}", path);          // custom behavior in the clause
                resume(PosixFs::read(path))
            }
            done(value) => value
        }
    }
}
```

See NOTE-025 §4 for all three handler production forms (derive, in-impl, standalone).

## 8. Admission

Authority is checked before the handler function executes — before the Handle frame is
installed. If admission fails, the handler never enters the dispatch stack.

Admission uses the existing `where` clause machinery:

```ash
handler posix_fs<A, r>(comp: Unit -> {PosixFs::read | r} A) -> {r} A
where requires host posix_fs
{
    on comp() { ... }
}
```

The admission predicate (`host posix_fs`) is checked at the point where the handler
function would be called. If it fails, the call is rejected. The Handle frame is never
installed, so the handler cannot intercept any operations.

This is the gate-before-dispatch model: authority is a precondition on handler
installation, not a runtime check during dispatch.

## 9. The Three Handler Shapes

### Deep handler (State, Reader, Fs, most effects)

Resume continues the computation. The handler threads state or resources through:

```ash
handler with_first<A, r>(comp: Unit -> {FirstChoice::choose | r} A) -> {r} A {
    on comp() {
        done(value) => value
        FirstChoice::choose(xs, resume) => resume(xs.head())
    }
}
```

### No-resume handler (Option, Either, Exception)

The handler does not call the continuation on certain branches — it short-circuits:

```ash
handler catch_throw<E, A, r>(comp: Unit -> {CatchIO::throw<E> | r} A) -> {r} Result<A, E> {
    on comp() {
        done(value) => Ok(value)
        CatchIO::throw<E>(err, _resume) => Err(err)
    }
}
```

The continuation is available but unused (`_resume` signals intentional discard). The
affine-use checker validates that discarding is legal. The answer type differs from the
computation's value type: `Result<A, E>` instead of `A`.

### Multi-shot handler (Nondet, all-solutions search)

Resume is called multiple times. The continuation must be pure (empty row):

```ash
handler all_choices<A>(comp: Unit -> {AllSolutions::choose} A) -> List<A> {
    on comp() {
        done(value) => [value]
        AllSolutions::choose(xs, resume) =>
            xs.flat_map(fn x -> resume(x))
    }
}
```

The type checker enforces that `resume : A -> {} Ans` — the continuation's row is empty.
If the computation interleaves effects after the raise point, the continuation's row is
non-empty, and the `flat_map` storing `resume` multiple times is ill-typed.

## 10. Row Accounting Summary

| Handler aspect | Row contribution |
|---|---|
| Handler function input row | `{op | r}` — operations to peel plus tail |
| Handler function output row | `{r}` — tail only (peeled operations removed) |
| Operation clause body row | `{r}` — may add handler's own effects |
| `resume` continuation type | `B_op -> {r} Ans` — remaining effects after peeling |
| `done` clause | contributes no row — it's pure handler return |
| Residual row | `r` plus any effects raised by clause bodies |

This matches SPEC-098b §5.5's Handle frame row transformation exactly. The surface grammar
elaborates into the same row facts that Core/CPS already tracks.

## 11. What This Eliminates

- ~~`effect` keyword~~ — resolved by NOTE-022 (interfaces)
- ~~`resume` as a magic keyword~~ — it's an ordinary named parameter
- ~~`resume: multi` annotation~~ — multiplicity is in the function type
- ~~`Resume<T>` / `Multi<T>` wrapper types~~ — ordinary function types
- ~~Separate Koka vs Frank clause shapes~~ — one clause shape, two installation forms
- ~~Separate handler/provider distinction at the grammar level~~ — a handler is a function;
  a provider is a handler with admission (a `where` clause)

## 12. Open Questions

1. **Extern placement.** **Consolidated in NOTE-024.** `extern` is a reserved keyword with no
   grammar production in the current target language; `builtin(...)` is the only host-reaching
   mechanism. The prior two-placement proposals are archived in NOTE-024 §3 as future-FFI
   design space.
2. **Answer type parameter.** **Resolved.** `handler` is an alias for `fn` — the answer type
   is always the function's return type. No hidden `A`/`Ans` conflation. Answer-type
   transformation (`A` → `Result<A, E>`, `A` → `List<A>`) is expressed in the return type.
3. **Pattern syntax for operations with many parameters.** `PosixFs::read(path, resume)`
   works for single-parameter operations. For `fn transfer(from: Account, to: Account,
   amount: Int) -> Receipt`, the clause is `Bank::transfer(from, to, amount, resume)`. Is
   this readable enough, or does the continuation need syntactic separation from operation
   arguments?
4. **`on` scrutinee shape.** Currently `on comp()` — always a thunk call. Should `on` accept
   arbitrary expressions that the compiler wraps in a thunk? Or should the thunk be
   explicit?
5. **Default `done` clause.** **Resolved.** With `handler` as alias for `fn` (no sugar that
   hides the signature), the `done` clause is always explicit inside `on`. Derive-generated
   handlers always include `done(value) => value`.

## 13. Working Principle

```text
A handler is a function that consumes a computation thunk.
`handler` is a keyword alias for `fn` — no semantic difference.
The continuation is an ordinary function-typed parameter.
Multiplicity is in the function type: affine if the row is non-empty, multi-shot if pure.
The answer type is the function's return type — no annotation needed.
One clause shape: ImplType::method(args, continuation) => body.
Row items are impl-type-qualified (e.g., PosixFs::read), not interface-qualified (NOTE-025).
Three handler production forms: derive (compiler-synthesized), in-impl (co-located), standalone.
Two installation forms: explicit application and handle...with sugar.
Authority is a where-clause gate before installation, not a runtime check during dispatch.
```

## 14. References

Internal references:

- [NOTE-022: Effects as Interfaces — Declaration Side](NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md)
- [NOTE-025: Effect Identity via Sorts and Impls](NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md) — revises the identity model
- [NOTE-013: Ambient Monad and Handler Composition Algebra](NOTE-013-AMBIENT-MONAD-AND-HANDLER-COMPOSITION-ALGEBRA.md)
- [NOTE-018: Boundary Discipline for Target Ash](NOTE-018-BOUNDARY-DISCIPLINE.md)
- [NOTE-019: Target Ash Convergence Plan](NOTE-019-TARGET-ASH-CONVERGENCE-PLAN.md)
- [SPEC-098b: Target CPS IR](../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-102: CPS Continuation Multiplicity](../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md)

External references:

- Leijen, "Koka: Programming with Row-Polymorphic Effects" (2014).
  https://www.microsoft.com/en-us/research/wp-content/uploads/2016/08/koka-technical.pdf
- Lindley, McBride & McLaughlin, "Do Be Do Be Do" (2017).
  https://doi.org/10.1145/3064898
- Plotkin & Pretnar, "Handlers of Algebraic Effects" (2009).
  https://link.springer.com/chapter/10.1007/978-3-642-02273-9_7
- Pretnar, "An Introduction to Algebraic Effects and Handlers" (2015).
  https://doi.org/10.2168/LMCS-11(1:23)2015

## 15. Changelog

- 2026-06-27: Initial version. Captures the dispatch-side design: handlers as functions,
  `on` eliminator, continuation as ordinary typed parameter, multiplicity via function type,
  one clause shape with two installation forms, named handler sugar, admission via where
  clauses. Completes the declaration/dispatch separation from NOTE-022.
- 2026-06-27: Revised by NOTE-025. All handler examples updated to impl-type-qualified row
  items (`PosixFs::read` instead of `Fs.read`). Named handler sugar (§7) replaced by
  `handler`-as-alias-for-`fn` with derive and handler-in-impl forms. Open questions #2
  (answer type) and #5 (default done) resolved. Working principle updated.
