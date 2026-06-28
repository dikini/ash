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

- **Handler grammar:** NOTE-023 introduces `on`, `handle...with`, `handler` (with the handler
  marker), and `derive handler` surface forms. SPEC-095b does not yet describe them.
- **Handler marker:** `handler` is NOT a pure alias for `fn`. It produces a handler-marked
  function type — a type-level attribute (like comp mode) identifying handler intent. This is
  required for derive filtering (§7.4) and `handle expr with` validation (§6). The underlying
  function type is structurally identical; the marker is erased at runtime.
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

**Handler marker validation.** In Form 2 (`handle...with`), name resolution checks that the
resolved identifier carries the handler marker (§7). A plain `fn` with a compatible
signature is rejected — see §7.3 (Subtyping). Form 1 (explicit application) does not check
the marker: it is ordinary function application, and the handler marker coerces to plain `fn`
(`handler fn <: fn`).

## 7. Handler Declaration: `handler` and the Handler Marker

`handler` is **not** a pure keyword alias for `fn`. It is a declaration-site keyword that
produces a function whose type carries a **handler marker** — a type-level attribute
identifying handler intent. The underlying function type is structurally identical to the
equivalent `fn` type, but the marker distinguishes handlers from plain functions in the type
system. This is required so that `derive` can filter operations from handlers (§7.4) and
`handle expr with name` can validate handler intent (§6).

### 7.1 The marker as a type-level attribute

The handler marker is analogous to how comp mode (eager/lazy/memo) annotates a computation
without changing its structural type. A function type can carry a `handler` marker:

```text
-- plain function type
(Unit -> {PosixFs::read | r} A) -> {r} A

-- handler-marked function type (same structure, carries handler intent)
handler (Unit -> {PosixFs::read | r} A) -> {r} A
```

The marker says: *"this function is intended for handler installation."* Nothing more. The
underlying function type is structurally identical — same parameters, same row, same return
type. The marker is erased at runtime; it carries no data.

| Aspect | Comp mode (eager/lazy/memo) | Handler marker |
|---|---|---|
| Location | Type-level attribute | Type-level attribute |
| Changes structure? | No | No |
| Runtime effect | Yes (evaluation strategy) | No (erased) |
| Purpose | Controls evaluation | Identifies intent |
| Set at | Declaration / annotation | Declaration (`handler` keyword) |

The analogy is structural — both are type-level modes that don't change the underlying type
constructor. The divergence is that comp mode has runtime consequences while the handler
marker is purely static. For now. If future uses (handler registries, dynamic dispatch)
require runtime presence, the marker can gain a runtime representation later without breaking
existing code — the type-level attribute is already there.

### 7.2 Declaration site

The `handler` keyword at declaration produces the marker:

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

The type of `posix_fs` is `handler (Unit -> {PosixFs::read | r} A) -> {r} A` — a
handler-marked function. A plain `fn` with the same parameters and return type has the
unmarked type `(Unit -> {PosixFs::read | r} A) -> {r} A`.

`derive handler <name>;` also produces a handler-marked value — the sugar expands to an
implicit `handler <name>(...)` declaration (§7.3).

### 7.3 Subtyping

`handler fn <: fn` — a handler-marked function is usable anywhere a plain function is
expected. The marker is an additive refinement: it says MORE about the function, not
something different. So:

```ash
fn apply_thunk<A, B>(f: (Unit -> A) -> B, thunk: Unit -> A) -> B { f(thunk) }

-- this works: handler-marked posix_fs coerces to plain fn
apply_thunk(posix_fs, fn () -> load_config<PosixFs>("/etc/config"))
```

The reverse — using a plain `fn` where a handler is expected — fails at `handle expr with`:

```ash
fn utility<A, r>(comp: Unit -> {PosixFs::read | r} A) -> {r} A { ... }

handle expr with utility
-- ERROR: `utility` is not a handler. Expected a handler-marked function.
```

This catches accidental misuse: a utility function with a compatible signature cannot be
silently installed as a handler just because the types line up.

### 7.4 Why the marker is required — the derive filter

The handler marker solves a concrete problem discovered when resolving NOTE-025 §7 Q3/Q4.
Inside an impl block, operations and handlers are co-located:

```ash
impl Fs for PosixFs {
    fn read(path: Path) -> String { ... }               // operation
    fn write(path: Path, contents: String) -> Unit { ... } // operation
    handler posix_fs<A, r: Row>(...) { ... }             // handler
    handler logging_fs<A, r: Row>(...) { ... }           // handler
}
```

`derive handler` must fold over **operations only** (§7.3 of NOTE-025: derive is the total
fold). But handlers are syntactically just functions — without the marker, the type system
cannot distinguish them from operations. The marker makes derive's filter a type-level check:
members without the handler marker are operation candidates; members with the marker are
skipped. This works even across module boundaries because the marker survives into module
summaries.

### 7.5 Grammar

The `handler` keyword gets its own production in impl blocks, distinct from `impl_method`:

```ebnf
impl_member    = impl_method | handler_decl | derive_decl ;

impl_method    = "fn" identifier type_params? "(" parameter_list? ")" [ "->" type ] fn_body ;

handler_decl   = "handler" identifier type_params? "(" parameter_list ")"
                 "->" type handler_body ;

derive_decl    = "derive" "handler" identifier ";" ;

handler_body   = "{" "on" scrutinee_call "{" handler_clause+ "}" "}" ;

handler_clause = "done" "(" identifier ")" "=>" expr
               | impl_qualified_op "(" pattern_list ")" "=>" expr ;
```

At module level, `handler` is a top-level definition producing a handler-marked function:

```ebnf
top_level_def  = ... | handler_decl | ... ;
```

The `handler` keyword is NOT folded into `impl_method` or `fn_definition`. It is a distinct
AST node (`HandlerDecl` vs `FnDecl`) and records the handler marker in the type.

### 7.6 Answer-type transformation

The answer type is always the function's return type — no hidden `A`/`Ans` conflation, no
annotation needed. Answer-type transformation is expressed in the return type:

```ash
handler catch_throw<E, A, r>(comp: Unit -> {CatchIO::throw<E> | r} A) -> {r} Result<A, E> {
    on comp() {
        done(value) => Ok(value)
        CatchIO::throw<E>(err, _resume) => Err(err)
    }
}
```

The type of `catch_throw` is `handler (Unit -> {CatchIO::throw<E> | r} A) -> {r} Result<A, E>`.
The answer type `Result<A, E>` differs from the computation's value type `A` — expressed in
the return type, as always.

### 7.7. Derive: the total fold

`derive handler <name>;` synthesizes the **deep handler** — the identity interpretation where
each operation runs its impl method body and resumes with the result. It is the total,
mechanical fold over ALL interface operations (NOTE-025 §7.3). The marker is what lets derive
filter correctly: it folds over members without the handler marker (operations), skipping
those with it.

```ash
impl Fs for PosixFs
where requires host posix_fs
{
    fn read(path: Path) -> String { builtin(fs_read, path) }
    fn write(path: Path, contents: String) -> Unit { builtin(fs_write, path, contents) }

    derive handler posix_fs;
}
```

The compiler synthesizes:

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

There is no subset derive — derive always generates clauses for all operations. Partial
behavior requires an explicit handler (§7.8). See NOTE-025 §7.3 for the full rationale.

### 7.8. Handler defined in impl (co-located)

An explicit handler can be defined inside the impl body for DX — same semantics as a
standalone handler, but co-located with the impl it references. Multiple handlers per impl
are allowed (NOTE-025 §7.4), distinguished by name and by the handler marker:

```ash
impl Fs for PosixFs
where requires host posix_fs
{
    fn read(path: Path) -> String { builtin(fs_read, path) }
    fn write(path: Path, contents: String) -> Unit { builtin(fs_write, path, contents) }

    derive handler posix_fs;       // total fold — all default

    handler logging_fs<A, r: Row>(comp: Unit -> {PosixFs::read, PosixFs::write | r} A) -> {r} A {
        on comp() {
            PosixFs::read(path, resume) => {
                log("reading {}", path);          // custom behavior in the clause
                resume(PosixFs::read(path))
            }
            PosixFs::write(path, contents, resume) => resume(PosixFs::write(path, contents))
            done(value) => value
        }
    }
}
```

Both `posix_fs` and `logging_fs` are handler-marked values in the value namespace. The
non-overridden clause (`PosixFs::write`) delegates to the impl method body — exactly what
derive would generate. The caller chooses which to install:

```ash
handle load_config<PosixFs>("/etc/config") with posix_fs      // total fold
handle load_config<PosixFs>("/etc/config") with logging_fs    // with logging
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
- ~~~~`handler` as pure alias for `fn`~~~~ — superseded. `handler` now carries a handler
  marker (type-level attribute) so derive can filter and `handle expr with` can validate.

## 12. Open Questions

1. **Extern placement.** **Consolidated in NOTE-024.** `extern` is a reserved keyword with no
   grammar production in the current target language; `builtin(...)` is the only host-reaching
   mechanism. The prior two-placement proposals are archived in NOTE-024 §3 as future-FFI
   design space.
2. **Answer type parameter.** **Resolved.** `handler` produces a handler-marked function —
   the answer type is always the function's return type. No hidden `A`/`Ans` conflation.
   Answer-type transformation (`A` → `Result<A, E>`, `A` → `List<A>`) is expressed in the
   return type.
3. **Pattern syntax for operations with many parameters.** `PosixFs::read(path, resume)`
   works for single-parameter operations. For `fn transfer(from: Account, to: Account,
   amount: Int) -> Receipt`, the clause is `Bank::transfer(from, to, amount, resume)`. Is
   this readable enough, or does the continuation need syntactic separation from operation
   arguments?
4. **`on` scrutinee shape.** Currently `on comp()` — always a thunk call. Should `on` accept
   arbitrary expressions that the compiler wraps in a thunk? Or should the thunk be
   explicit?
5. **Default `done` clause.** **Resolved.** With `handler` carrying the handler marker (no
   sugar that hides the signature), the `done` clause is always explicit inside `on`.
   Derive-generated handlers always include `done(value) => value`.

## 13. Working Principle

```text
A handler is a function that consumes a computation thunk.
`handler` carries a type-level handler marker — it is NOT a pure alias for `fn`.
The marker is a type-level attribute (like comp mode): identifies intent, erased at runtime.
`handler fn <: fn` — handler-marked functions coerce to plain functions, not vice versa.
`handle expr with name` requires name to carry the handler marker.
The continuation is an ordinary function-typed parameter.
Multiplicity is in the function type: affine if the row is non-empty, multi-shot if pure.
The answer type is the function's return type — no annotation needed.
One clause shape: ImplType::method(args, continuation) => body.
Row items are impl-type-qualified (e.g., PosixFs::read), not interface-qualified (NOTE-025).
Derive is the total fold over all operations — filters by absence of the handler marker.
Multiple handlers per impl are allowed — distinguished by the handler marker.
Three handler production forms: derive (total fold), in-impl (co-located), standalone.
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
- 2026-06-28: Handler marker introduced. `handler` is no longer a pure alias for `fn` — it
  produces a handler-marked function type (type-level attribute, like comp mode). Required
  so derive can filter operations from handlers (the Q3/Q4 interaction from NOTE-025 §7) and
  `handle expr with name` can validate handler intent. §7 fully rewritten with grammar
  (handler_decl production), types (handler marker attribute), subtyping (`handler fn <: fn`),
  worked examples (derive filtering, co-located multiple handlers), and the comp-mode analogy.
  Pre-Spec Delta, §11, §12, §13 updated to reflect the marker.
