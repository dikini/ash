# NOTE-013: Ambient Monad and Handler Composition Algebra

**Date:** 2026-06-23
**Status:** Living document — exploration in progress
**Purpose:** Capture the theoretical framework connecting the CPS-based IR, the ambient
continuation monad, computation rows, and handler composition as the replacement for monad
transformers. Updated as new insights emerge; restructured for flow and readability later.

## 0. Motivation

The surface-language overhaul aims to collapse `act`/`proc`/`workflow` into a single `fn`,
with the tower levels expressed as computation-row profiles over a unified computation type. A
central question arises: with eager (call-by-value) evaluation, the ambient monad does not
need explicit sequencing for purity — but it is real and useful for library structuring.
Instead of monad transformers (which layer effects by nesting type constructors), Ash uses
**row extensions** (which layer effects by adding items to a flat requirement set).

The wrinkle: if effects are defined as library monads (State, Option, Either, Future, ...),
their `bind` operations define properties of sequencing — hidden state, short-circuit, early
exit, time-travel respectively. With monad transformers, layering composes these into a
derived monad with a specific order of execution. The handler algebra must assume
responsibility for the same: the programmable semicolon. In the non-commutative case, we
need a deterministic algebra for combining handlers.

This note develops that algebra.

## 1. The Ambient Monad, Explicitly

Every CPS term is typed under a fixed answer type `Ans`:

```
Γ ⊢ term ! Ans, local ρ_local, total ρ_total
```

A surface function `fn f(x: A) -> {ρf} B` lowers to:

```
f : ∀Ans ρk. CpsFn {
    params: [A],
    cont: Cont<B, Ans, ρk>,
    answer: Ans,
    body_row: ρf
}
```

The continuation type `Cont<A, Ans, ρ>` is `A -> {ρ} Ans` in direct-style notation. The
ambient computation type is:

```
Comp<ρ, A>  ≅  (A -> {ρk} Ans) -> {ρ ∪ ρk} Ans
```

For each fixed `ρ`, this is the **continuation monad** specialized to answer type `Ans`.
The row `ρ` is an *index* — a type-level row of computation facts. In this note's handler
examples, the relevant facts are mostly algebraic operations the computation may raise.

## 2. Bind and Return Are Row-Polymorphic, Defined Once

The monad operations are defined polymorphically over the row. They do not change when you
extend the row — what changes is which operations are available.

```
unit : A -> Comp<{}, A>
unit(a) = λk. k(a)
```

The empty row is crucial: `unit` performs no operations and works in any context because
`{} ⊆ ρ` for all `ρ`.

```
bind : Comp<ρ₁, A> -> (A -> Comp<ρ₂, B>) -> Comp<ρ₁ ∪ ρ₂, B>
bind(m, f) = λk. m(λa. f(a)(k))
```

The output row is `ρ₁ ∪ ρ₂`. Bind *accumulates* requirements. This is not a single monad —
it is a **row-indexed family** of monad instances, with `bind` as a single row-polymorphic
operation.

The monad laws hold because they are the continuation monad laws, reducible by
β-reduction:

```
Left identity:   bind(unit(a), f)  =  f(a)
Right identity:  bind(m, unit)     =  m
Associativity:   bind(bind(m, f), g) = bind(m, λx. bind(f(x), g))
```

## 3. The Tower Is Not a Different Monad — It Is a Row Profile

This is what collapses `act`/`proc`/`workflow` into `fn`:

```
Pure      = Comp<{}, A>                                (empty row)
Act       = Comp<{operation ..., resource ...}, A>      (operation/resource rows)
Proc      = Comp<{... channel ..., proc ...}, A>        (Act + process effects)
Workflow  = Comp<{... role ..., policy ...}, A>         (Proc + governance effects)
```

Each tower level is a constraint on which rows are admissible. `bind`/`return` are the same
operation at every level. The tower does not introduce a new monad — it **restricts the row
space**. A Pure function is `Comp<{}, A>`: same `bind`/`return`, but its row is empty, so it
can raise no operations.

## 4. Row Extension Replaces Monad Transformers

In Haskell:

```
StateT s (ReaderT r IO)
```

Each transformer wraps the inner monad, adding a specific effect layer. Composition is
nested — the type encodes the stack.

In Ash with computation rows:

```
Comp<{resource state_ref read/write, resource env_ref read, io}, A>
```

The row is **flat**. No nesting. Extending a computation's possible operations is *row extension*,
not wrapping.

Formally: row inclusion `ρ ⊆ ρ'` induces a monad morphism:

```
lift : Comp<ρ, _> -> Comp<ρ', _>
lift(m) = m   (identity at the term level — the computation doesn't change,
               it just has permission to be placed in a context requiring more)
```

This trivially commutes with `bind`/`return`:

```
lift(unit(a))      = unit(a)                         — unit has the empty row
lift(bind(m, f))   = bind(lift(m), λx. lift(f(x)))   — lift is identity on terms
```

The morphism is the identity on terms because the row is a *permission*, not a *wrapper*.
The computation `m` doesn't change when you allow it to run in a context with more effects
available.

This is the precise sense in which row extension replaces transformers: transformers build
a *different type* for each stack; row extension gives you the *same type indexed by a
larger row*.

## 5. The Wrinkle: Algebraic Operations Are Row-Specific

`bind` and `return` are generic (structural). But effect operations are not:

```
fs.read   : {fs.read} String        — only available when ρ contains fs.read
spawn     : {proc spawn} P<A>       — only available when ρ contains proc spawn
```

These operations lower to `Raise` nodes in the CPS IR. They are the **generators** of the
algebraic theory denoted by the row.

This is the Plotkin-Power insight (see §10 References): the operation family inside a
computation row denotes an algebraic theory (a signature of operations). The computation type
`Comp<ρ, A>` is the **free monad** over the operation theory denoted by `ρ`.
`bind`/`return` are the free monad's structural operations. The effect operations are the
theory's generators.

The separation:

```
STRUCTURAL (defined once, row-polymorphic):
  unit, bind, fmap, apply, sequence, traverse

ALGEBRAIC (row-specific, one generator per effect item):
  fs.read, log.write, spawn, send, receive, etc.
```

Structural operations manipulate the continuation chain. Algebraic operations inject `Raise`
nodes. They live at different levels.

## 6. Handlers Discharge Row Items — and Their Resume Strategy Is the Semicolon

A `Handle` frame in the CPS IR is a fold over the free algebraic theory. Installing a
handler for operation `op`:

1. Removes `op` from the body's local row (it is discharged).
2. Adds the handler's own effects to the residual row.
3. May use the resume continuation to continue the computation.

This is the row transformation in SPEC-098b §5.5:

```
handled_segment.local (delimited, pre-resume): {op, ... | r}
handler_clause.local: {handler_effects}
captured_resume.local: effects reachable after resume (may include same op)
Handle { op, ... } local row: (handled_segment.local - handled_op) ∪ captured_resume.local ∪ handler_clause.local
```

A handler is an **interpreter** for a sub-theory. If the handler satisfies the equational
theory of its operations (e.g., state get/put laws), the interpretation is *sound* — it
preserves all provable equalities.

The **resume strategy** — how the handler continues after intercepting an operation — is
the programmable semicolon. This is the key correspondence, now concretely typed by SPEC-102:

| Monad       | Handler resume strategy | SPEC-102 multiplicity    | What the semicolon does |
|-------------|------------------------|--------------------------|------------------------|
| State       | Deep resume, threads cell | `Affine`              | Each step sees the updated cell |
| Option      | No resume on None | `Affine` (discarded)         | Short-circuits; remaining steps skipped |
| Either      | No resume on Left | `Affine` (discarded)         | Early exit; remaining steps skipped |
| List/Nondet | Multi-shot resume | `MultiShotPure` (`row = {}`)| Branches; explores all paths |
| Future      | Delayed resume    | `Affine` (not yet implemented) | Schedules continuation for later |

The deep resume in the state handler *is* StateT's `bind`. The absent resume in the option
handler *is* Option's `bind`. Multi-shot resume *is* List's `bind`. This is not an analogy —
it is the same operation, implemented differently. Filinski (1994) proved that any monad can
be decomposed into a continuation monad plus a handler; monad transformers are the special
case where handlers are statically determined and composed by nesting.

**SPEC-102 makes the multiplicity explicit and typed.** The resume continuation carries a
`ContMultiplicity` (`Affine` or `MultiShotPure`) declared on the `HandlerClause` via
`resume_multiplicity`. The crucial legality condition is that `MultiShotPure` requires the
resume continuation's row to normalize to `{}` — the post-operation continuation must
be *pure* to be legally reusable. This is not an inference: SPEC-102 §8 states "Core producers
must explicitly choose the multiplicity." The empty row is a legality gate, not a trigger.

This has a deep consequence for handler composition: a `MultiShotPure` resume can only
backtrack through *effect-free* continuations. If the nondeterministic computation interleaves
state effects, the captured resume continuation has a non-empty row, and multi-shot
resumption is ill-typed. The List handler then degrades to affine — it can only explore one
branch. Exploring multiple branches over stateful computation requires a different strategy
(rollback, explicit state threading per branch, or a future temporal/delayed multiplicity).
This is precisely where NOTE-013 §8.5's resume-strategy interaction meets SPEC-102's row
legality: the composition algebra is no longer purely behavioral — it is typed by the row.

## 7. Handler Composition Is Handler Nesting — and Order Matters

Given handlers H₁ (operations O₁) and H₂ (operations O₂), the composition:

```
handle
  handle
    computation
  with H₂          -- innermost
with H₁            -- outermost
```

produces behavior derivable from the individual handlers.

### 7.1 The Derivation Rule

For any handler stack H₁ ∘ H₂ ∘ ... ∘ Hₙ (Hₙ innermost), the behavior of a computation is
derivable by this procedure:

```
evaluate(computation, handler_stack [H₁, ..., Hₙ]):
  when computation does Return(a):
    → produce a as the value for the nearest enclosing context

  when computation raises op:
    → search handler_stack from innermost (Hₙ) to outermost (H₁)
    → first Hᵢ whose clauses contain op catches it
    → execute Hᵢ.clause_for(op, args, resume)
    where resume(r) = evaluate(k(r), handler_stack minus Hᵢ if shallow,
                                              full stack if deep)
    → if Hᵢ does not resume, the computation's continuation is discarded
       up to the next outer handler that can observe the non-resumption
```

This is a fully deterministic operational semantics. Given the handler clauses and resume
strategies, the behavior is mechanically derivable. No ambiguity.

### 7.2 Concrete Example: State ∘ Either (Non-Commutativity)

**ORDER A: Either inside State** (analogous to `StateT s (EitherT e m)`)

```
handle                              -- outer: State handler
  handle                            -- inner: Either handler
    computation
  with { Left(e) => Left(e),        -- short-circuit, NO resume
         return(x) => x }
with { get(k) => k(cell),            -- deep resume, threads cell
       put(s,k) => k(()),
       return(x) => (cell, x) }
```

If computation does `put(s1); Left(err)`:

1. `put(s1)` raises to State (outer), updates cell to `s1`, resumes.
2. `Left(err)` raises to Either (inner), catches it, produces `Left(err)`, does NOT resume.
3. Either's result `Left(err)` flows to State's return clause.
4. **Final: `(s1, Left(err))`** — state IS preserved.

**ORDER B: State inside Either** (analogous to `EitherT e (StateT s m)`)

If computation does `put(s1); Left(err)`:

1. `put(s1)` raises to State (inner), updates cell, resumes.
2. `Left(err)` propagates through State (no Left clause) to Either (outer).
3. Either catches `Left(err)`, does NOT resume.
4. State's return clause is never reached.
5. **Final: `Left(err)`** — state is LOST.

This is exactly `StateT(EitherT)` vs `EitherT(StateT)`. The handler nesting order
determines which you get. This is deterministic and derivable.

## 8. The Algebra: Composition Laws

### 8.1 Associativity

```
(H₁ ∘ H₂) ∘ H₃ = H₁ ∘ (H₂ ∘ H₃)
```

Holds because `Handle` frame nesting is associative in the CPS IR. Three nested frames can
be regrouped without changing dispatch behavior.

### 8.2 Identity

The trivial handler (return clause only, no operation clauses) is the identity:

```
H ∘ Id = Id ∘ H = H
```

Holds because the trivial handler re-raises all operations and passes through returns
unchanged.

### 8.3 Disjoint Commutativity

If `O₁ ∩ O₂ = ∅` and both handlers are **deep**: `H₁ ∘ H₂ ≅ H₂ ∘ H₁`.

This is the Lüth-Ghani (2002) coproduct result: disjoint algebraic theories compose
commutatively. Each handler only intercepts its own operations, and deep resume means
neither interferes with the other's operations during resume.

### 8.4 Non-Commutativity When Operations Overlap

If `O₁ ∩ O₂ ≠ ∅`: `H₁ ∘ H₂ ≠ H₂ ∘ H₁` in general.

The innermost handler catches shared operations first. Swapping order changes which handler
interprets those operations.

### 8.5 Non-Commutativity When Resume Strategies Interact

Even when `O₁ ∩ O₂ = ∅`, if one handler is shallow (no resume) and the other is deep,
composition may not commute.

Example: A shallow exception handler inside a deep state handler preserves state on error
(ORDER A above). A shallow exception handler outside a deep state handler discards state on
error (ORDER B above). Operation sets are disjoint, but resume-strategy interaction makes
order matter.

This is the subtlest law. It says: **non-commutativity is not just about which operations
overlap — it is about how resume strategies interact across the handler boundary.**

### 8.6 Multiplicity Constrains Which Composition Laws Are Typeable

SPEC-102 sharpens §8.3 and §8.5 from behavioral claims to *typeability* claims. The
multiplicity of the resume continuation — `Affine` or `MultiShotPure` — is typed, and the
`MultiShotPure` variant carries a row-legality condition (`row = {}`). This means:

- **§8.3 (disjoint commutativity) now has a precondition beyond `O₁ ∩ O₂ = ∅`.** The
  Lüth-Ghani coproduct result assumes both handlers are deep and their resume continuations
  are *unrestricted*. In SPEC-102, a `MultiShotPure` resume requires a pure continuation row.
  If handler H₂ captures a `MultiShotPure` resume whose continuation would, after H₁'s
  operations flow through it, have a non-empty row, the multi-shot resumption is *ill-typed*.
  So disjoint commutativity holds only when the multi-shot resume's empty-row legality is
  preserved under composition — i.e., when the outer handler does not inject effects into the
  resumed continuation's row.

- **§8.5 (resume-strategy interaction) is now partially recoverable by typing.** The
  dangerous case — a shallow handler discarding a continuation that a deep handler expected
  to resume — is caught by the affine-use checker. An `Affine` resume consumed by the shallow
  handler's clause is a well-typed *single* use. The deep handler's expectation of seeing the
  continuation return is not a type obligation; it is an operational consequence. The row
  system does not (and should not) track "will this handler resume?" — it tracks only "which
  operations *can* this continuation raise." The shallow/deep distinction lives at the
  operational layer (§7.1), not the row layer.

The net effect: SPEC-102 does not remove the non-commutativity of §8.5, but it *constrains*
the search space. The type system rejects compositions where `MultiShotPure` would backfire
(resuming a non-pure continuation). For the affine case — the site of all shallow/deep
interaction — the type system is silent, and operational order (§7) is the sole determinant.
This is a clean separation: **the row types the *what* (which effects), the multiplicity
types the *how-many* (how many times), and the nesting order determines the *when* (which
handler sees the operation first).**

## 9. Provability: When Can We Prove Composition Equalities?

### 9.1 Level 1 — Free Theories (no equations on operations)

Every handler is automatically sound. Any handler for a free theory (Choice, Trace, bare
operation requests with no equational constraints) produces a valid interpretation. No
proof obligations.

### 9.2 Level 2 — Theories with Equations (State, Reader, Writer)

The handler must satisfy the theory's equations. For State:

```
get(λs. put(s, k)) ≡ k         -- get-after-put is identity
get(λs. k) ≡ get(λs. k)        -- get is deterministic
```

A state handler must satisfy these. This becomes a proof obligation checked by the type
system / law prover. If the handler's clause bodies satisfy these equations for all possible
continuations `k`, the handler is sound.

### 9.3 Level 3 — Cross-Handler Equations (the hard case)

When two handlers interact, new equations may hold or fail. `State ∘ Either` satisfies:

```
put(s1); Left(e) ≡ (s1, Left(e))    -- in ORDER A
put(s1); Left(e) ≡ Left(e)           -- in ORDER B
```

These cross-handler equations are derivable from the handler definitions and nesting order,
but they must be stated and checked.

### 9.4 Level 4 — Temporal Equations (Future, async, concurrency)

For delayed/multi-shot resume, equational reasoning is harder because it involves time. Two
computations may be equivalent in *what* they compute but differ in *when* they compute it.

For Future/async handlers, relevant equivalences include:

```
par(a, b) ≡ par(b, a)          -- only if a, b don't interact
cancel(fork(a)) ≡ unit(())     -- only if a has no observable side effects
bind(a, λx. bind(b, λy. c)) ≡ bind(bind(a, λx. b), λy. c)
                                -- only if a doesn't depend on y
```

These require a notion of observational equivalence that accounts for timing, ordering of
effects, and resource usage. The CPS trace preserves ordering, but equational reasoning
needs to decide which orderings are observationally equivalent.

This is where the user's "temporal variance" concern lives.

## 10. Applicatives: A Subtle Point

The applicative structure is derivable from monad:

```
pure  = unit
apply : Comp<ρ₁, (A -> B)> -> Comp<ρ₂, A> -> Comp<ρ₁ ∪ ρ₂, B>
apply(mf, ma) = bind(mf, λf. bind(ma, λa. pure(f(a))))
```

There is a genuine subtlety: applicative semantics traditionally does not guarantee
evaluation order — `apply(mf, ma)` could evaluate in either order or in parallel. In CPS
with eager evaluation, the continuation chain imposes a specific order (left-to-right in
the definition above).

The **row** (`ρ₁ ∪ ρ₂`) is the same regardless of order — row union is commutative. But the
**operational trace** differs. This means:

- At the type level (row accounting): applicative is order-independent.
- At the operational level (CPS trace): applicative imposes an order.

This is fine for eager evaluation: the row system only tracks *what* effects occur, not
*when*. The row system deliberately does not encode temporal ordering — that's the CPS
continuation chain's job.

But if we want **parallel** applicatives (where `mf` and `ma` genuinely run concurrently),
that needs a different primitive than sequential `bind`. That is the `par` combinator
(see NOTE-005 §11.2), and it needs its own row accounting.

## 11. What the CPS IR Already Gives Us

The IR already implements the deterministic derivation:

- `Handle { clause, body, cont, row }` — handler frame with clause and continuation.
- `Raise { op, args, resume, row }` — operation dispatch with resume continuation.
- `HandlerClause.resume` — resume parameter, now typed by SPEC-102 with
  `resume_multiplicity` (`Affine` / `MultiShotPure`) and `resume_row`
  (`Known(EffectRow)` / `LegacyInheritFromTarget`). The runtime constructs the dynamic
  `Value::Cont` for the resume, copying the clause's multiplicity and resolved row.
- `LetCont` / `LetContCall` — continuation binding and answer-binding invocation (SPEC-102 §5),
  the latter enabling handler bodies that observe the resume's answer before continuing.
- Row transformation in §5.5 of SPEC-098b — removes handled operations from the body row,
  adds handler effects.

The nesting of `Handle` terms in the IR *is* the ordered composition. The row
transformation *is* the type-level accounting of which operations are handled at which
level. SPEC-102 adds the multiplicity dimension: the *same* nesting can now express both
single-shot (affine) and multi-shot-pure resume, with the row-legality gate ensuring that
multi-shot resume is only constructed over pure continuations.

What is missing is the **equational layer**: the laws that let us prove two handler stacks
are equivalent, or that a particular handler satisfies its theory's equations. This is where
the law/proof system connects to the effect system.

### 11.1 Effect Operations and Host Externs

This reframes the current Ash capability system as target effects:

```
current capability operation  = effect operation generator
current capability provider   = handler/interpreter for that effect
current capability binding    = admitted handler/environment binding
current capability authority  = row discharge evidence
current capability audit      = evidence/provenance effect emitted by the handler
```

The important distinction is that a computation row can record that a computation may
*request* an operation. It does not itself grant authority. In the target language, the former
capability concept is an effect operation whose handler may only be installed by an admission
fact with authority provenance.

This matters for runtime/host/FFI integrations. Operations such as reading a file, opening a
network socket, polling a timer, or calling a host LLM provider need an implementation
boundary that the host can actually satisfy. The host boundary should be small and
ABI-shaped; the Ash boundary should be semantic and effect-shaped.

**Host/FFI and extern placement have been consolidated in [NOTE-024](NOTE-024-HOST-FFI-AND-EXTERN.md).**

The current target position (per NOTE-024): `extern` is a reserved keyword with no grammar
production; `builtin(...)` is the only host-reaching mechanism, callable only inside trusted
stdlib handler/provider method bodies. The static invariant is:

```
ordinary Ash code calls typed operations (interface methods);
trusted stdlib handler bodies call builtin(...);
nothing else reaches the host.
```

The prior extern placement proposals (Placement A: interface-attached, Placement B:
handler-local) and the safety obligation layers are archived in NOTE-024 §3 as the design
space for a future host/FFI spec. NOTE-022 invalidated Placement A (externs do not attach to
interfaces). Placement B remains a candidate shape for future handler-local FFI hooks.

Laws can state the semantic theory of the effect and the handler. They cannot, by
themselves, prove arbitrary host ABI safety unless the ABI layer exposes enough structure to
reason about it.

## 12. Key Design Decisions Surfaced

1. **Should the surface syntax make handler order explicit and visible?** Nesting in
   `handle ... with` blocks does this naturally. Frank-style function handlers hide it
   inside function application order.

2. **Should computation rows be ordered or unordered?** Recommendation: keep rows unordered
   (they are requirement sets). Let handler nesting order be the sole determinant of
   priority. This separates:
   - WHAT is required (row, unordered)
   - HOW it is discharged (handler stack, ordered)

3. **Should resume strategy (deep/shallow/multi-shot) be a property of the handler
   declaration or of each clause?** Recommendation: per-handler, declared at the handler
   definition site, so the "programmable semicolon" is visible at declaration time.

4. **How do proof obligations attach?** When a handler claims to interpret a theory with
   equations (State, Reader), the type system should require evidence that the handler
   satisfies those equations. This is a natural extension of the existing law/proof system.

5. **Where can host externs be declared?** **Consolidated in NOTE-024.** `extern` is a reserved
   keyword with no grammar production in the current target language. `builtin(...)` is the
   only host-reaching mechanism, callable inside trusted stdlib handler bodies. The prior
   placement proposals are archived as the design space for a future host/FFI spec.

## 13. Open Questions

1. What is the minimal surface for declaring effect operations and their signatures? **Resolved
   by NOTE-022:** operation signatures are declared as interface methods using the existing
   interface/impl machinery. No separate `effect` keyword. The interface is the type contract;
   dispatch is Handle frame nesting; authority is admission.

2. What surface form do handlers take? **Partially resolved by NOTE-023:** handlers are
   functions consuming computation thunks, using the `on` eliminator. Named handler sugar
   (`handler Name for Interface`) and `handle...with` scoped installation are optional sugar.
   Remaining open: answer-type transformation in sugar form, extern placement syntax.

3. How do the existing Ash algebraic classes (Functor/Applicative/Monad in
   `std/src/algebra/`) connect to the row-polymorphic structural operations? Do they become
   instances over `Comp<ρ, _>` for each `ρ`, or is there a single generic instance?

4. How do contracts/proofs attach to handlers? (Handler-soundness-as-proof-obligation from
   §9 — where the law system meets the effect system.)

5. What is the concrete surface syntax for the four resume strategies (deep, shallow,
   multi-shot, delayed)? **Partially resolved by NOTE-023:** the continuation is an ordinary
   typed function parameter. Deep = call it; shallow = don't call it; multi-shot = call it
   multiple times (legal only when the continuation's row is pure `{}`). SPEC-102 implements
   the Core/CPS substrate for affine and multi-shot-pure. The surface makes no syntactic
   distinction — the function type carries multiplicity. Delayed resume remains an
   operational strategy without dedicated Core/CPS multiplicity.

6. Which extern placement should be the default surface? **Consolidated in NOTE-024.** `extern`
   is reserved but has no grammar production in the current target language. The prior
   two-placement model (interface-level vs handler-level) is archived in NOTE-024 §3 as the
   design space for a future host/FFI spec. NOTE-022 invalidated Placement A (interface-level).

## 14. References

Foundational and directly relevant work. Where an official DOI or canonical version exists,
it is linked.

### Algebraic effects and handlers

- **Plotkin & Power, "Computational Effects as Operations"** (2002).
  Introduces the view of effects as operations of an algebraic theory.
  https://www.sciencedirect.com/science/article/pii/S0304397502004449

- **Plotkin & Pretnar, "Handlers of Algebraic Effects"** (2009).
  Generalizes exception handling to arbitrary algebraic effects; establishes that algebraic
  effects include exceptions, state, nondeterminism, I/O, time.
  https://link.springer.com/chapter/10.1007/978-3-642-02273-9_7

- **Pretnar, "An Introduction to Algebraic Effects and Handlers"** (2015).
  Tutorial survey connecting algebraic effects, handlers, and induced monads.
  https://doi.org/10.2168/LMCS-11(1:23)2015
  Open access: https://lmcs.episciences.org/2053

- **Bauer & Pretnar, "Programming with Algebraic Effects and Handlers"** (2012).
  The Eff language: first-class algebraic effects and handlers as homomorphisms from free
  algebras.
  https://arxiv.org/abs/1203.1539

### Monads and continuations

- **Moggi, "Notions of Computation and Monads"** (1991).
  Foundational work on monads for modeling computational effects.
  https://doi.org/10.1016/0890-5401(91)90052-4

- **Filinski, "Representing Monads"** (1994).
  Proves that any monad can be decomposed into a continuation monad plus a handler. The
  theoretical basis for the handler-as-semicolon correspondence.
  https://doi.org/10.1145/174675.178047

### Handler composition and coproducts of theories

- **Lüth & Ghani, "Composing Monads Using Coproducts"** (2002).
  Coproduct construction for combining computational monads from algebraic theories. The
  formal basis for disjoint commutativity (§8.3).
  https://doi.org/10.1016/S0304-3975(02)00018-9

- **Hyland, Plotkin & Power, "Combining Effects: Sum and Tensor"** (2006).
  Sum (coproduct) and tensor products of computational effects, covering when combination is
  commutative and when it is not.
  https://doi.org/10.1016/j.tcs.2006.06.014

### Row-based effect systems

- **Leijen, "Koka: Programming with Row-Polymorphic Effects"** (2014).
  Koka's effect-row system, `handler` syntax, and row-based effect inference. The
  direct inspiration for row-polymorphic operation requirements as flat requirement sets.
  https://www.microsoft.com/en-us/research/wp-content/uploads/2016/08/koka-technical.pdf

- **Lindley, McBride & McLaughlin, "Do Be Do Be Do"** (2017).
  Frank language: handlers as functions, making the row transformation explicit in the type.
  Direct inspiration for the Frank-style handler surface.
  https://doi.org/10.1145/3064898

### Efficient encoding: free monads vs. CPS

- **Kammar, Lindley & Oury, "Handlers in Action"** (2013).
  Efficient implementation of effect handlers, including the CPS encoding and the
  relationship between the free-monad semantics and the continuation-based implementation.
  https://doi.org/10.1145/2500365.2500590
  Preprint: https://homepages.inf.ed.ac.uk/slindley/papers/handlers-in-action.pdf

### Type-and-effect systems

- **Lucassen & Gifford, "Polymorphic Effect Systems"** (1988).
  Original type-and-effect system with effect rows.
  https://doi.org/10.1145/62084.62094

- **Marino & Millstein, "A Generic Type-and-Effect System"** (2009).
  Generic framework for type-and-effect systems, relevant to row-based effect tracking.
  https://doi.org/10.1145/1706296.1706330

### Internal references

- **SPEC-098b** — Target CPS IR with unified computation rows, three-layer grammar,
  operation-typed raise/handle.
  `docs/spec/SPEC-098b-TARGET-IR.md`
- **SPEC-096b** — Target effect system: computation rows as requirement sets, kind-specific
  discharge, "rows are requirements, not grants."
  `docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md`
- **SPEC-097b** — Target type system: row syntax, row variables, constraint kinds,
  function subtyping.
  `docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md`
- **SPEC-047** — Act monad: first-class effectful computation, the precursor design.
  `docs/spec/SPEC-047-ACT-MONAD.md`
- **NOTE-001** — Workflow computation type (`comp T`): the earlier proposal for an explicit
  computation type, deferred. This note supersedes its motivation for the unified approach.
  `docs/notes/NOTE-001-WORKFLOW-COMPUTATION-TYPE.md`
- **NOTE-005** — The Act monad unifying pure and effectful computation: the earlier design
  that this note generalizes to the row-polymorphic continuation monad.
  `docs/notes/NOTE-005-ACT-MONAD-UNIFYING-PURE-AND-EFFECTFUL.md`
- **NOTE-012** — Mutual recursion and CPS translation design.
  `docs/notes/NOTE-012-MUTUAL-RECURSION-CPS-ASPECTS-DESIGN.md`
- **SPEC-102** — CPS continuation multiplicity (affine vs. multi-shot-pure resume).
  `docs/spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md`
- **docs/design/effect-handling-styles.md** — Koka vs Frank handler syntax comparison.
  `docs/design/effect-handling-styles.md`

## 15. Changelog

| Date       | Change |
|------------|--------|
| 2026-06-23 | Initial version. Ambient monad, row extension vs. transformers, handler composition algebra, composition laws, provability levels. |
| 2026-06-24 | Integrated SPEC-102 (now implemented): sharpened §6 resume-strategy table with concrete `Affine`/`MultiShotPure` multiplicities and the `row = {}` legality gate; added §6 consequence on multi-shot over stateful computation. Added §8.6 — multiplicity constrains which composition laws are typeable (row types the *what*, multiplicity the *how-many*, nesting the *when*). Updated §11 to reference the SPEC-102 `HandlerClause` fields (`resume_multiplicity`, `resume_row`) and `LetCont`/`LetContCall`. Updated Open Question 5 with SPEC-102 status. |
| 2026-06-24 | Added §11.1 on current capabilities as target effect operations plus handlers, with host/FFI externs as effect-local unsafe implementation hooks. Captured the invariant that externs are not ordinary Ash functions: the public authority surface remains the typed effect operation and its row-discharge/admission path. |
| 2026-06-24 | Expanded §11.1 with two effect-local extern placement alternatives: externs in the effect declaration for canonical host ABIs, and externs in trusted handlers for backend-specific adapters. Updated Open Question 6 to treat placement as a surface-syntax choice over the same semantics. |
| 2026-06-27 | Normalized target-row wording to use computation rows for the type-level row concept while preserving effect operations as algebraic generators and external effect-row literature references. |
| 2026-06-27 | Applied NOTE-022 decision: replaced all `effect Foo { ... }` declaration examples with `interface Foo { ... }` in §11.1. Externs now shown as dispatch-side constructs attached to the effect's interface (Placement A) or to trusted handlers (Placement B). Marked Open Question 1 as resolved by NOTE-022. Open Questions 2, 5, and 6 remain open as dispatch-side concerns. |
| 2026-06-27 | Applied NOTE-023 decision: marked Open Questions 2 and 5 as partially resolved — handler surface form and resume strategy syntax are captured in NOTE-023. |
| 2026-06-27 | Consolidated host/FFI and extern placement into NOTE-024. Replaced the detailed §11.1 extern placement content (Placement A/B, safety obligation layers) with a pointer to NOTE-024. Updated Open Questions 5 and 6 and Key Decision 5 to reference NOTE-024. The current target position: `extern` is reserved with no grammar production; `builtin(...)` is the only host-reaching mechanism. |
