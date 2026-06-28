# NOTE-028: Purity, Evaluation Modes, and Contract Timing

**Date:** 2026-06-28
**Status:** Living document — design direction captured; resolves NOTE-014 GAP 4 and
NOTE-025 §7.9
**Purpose:** Define how Ash classifies purity for type-level attributes (`strict`/`lazy`/`memo`
and the handler marker), and define when contracts fire for lazy and memoized computations.
The central decision is denotational: an attribute is purity-preserving if it preserves
referential transparency. Operational mechanisms such as thunk allocation, cache cells, and
handler frames do not by themselves make a term impure; user-visible effects remain governed
by rows.

Companion to NOTE-014 (contract systems unification), NOTE-023 (handler marker), NOTE-025
(effect identity and parked purity question), NOTE-027 (contract blame and subsumption),
SPEC-097b §15 (evaluation modes), and `docs/reference/core-ash-lazy-memo-modes.md`.

## Pre-Spec Delta

This note is pre-spec. When the project moves to spec updates, reconcile:

- **Purity definition (SPEC-027, SPEC-097b):** purity is denotational referential
  transparency, not absence of runtime implementation mechanisms. Empty residual row remains
  the first-order purity test, but type-level attributes are not row items and do not, by
  themselves, make a function impure.
- **Evaluation modes (SPEC-097b §15):** `lazy` and `memo` are purity-preserving mode
  attributes when their latent row is empty. If the latent row is non-empty, forcing is
  effectful because the latent row says so, not because the mode is `lazy` or `memo`.
- **Handler marker (NOTE-023 §7 / SPEC-095b §6.4):** the marker is purity-preserving. A
  handler-marked function can be pure if its residual row is empty and the computation it
  interprets is fully handled to an empty residual row.
- **Contract timing (NOTE-014 / SPEC-098b):** contracts on lazy/memo values fire at the
  observation boundary (force), not at thunk construction. Lazy rechecks on each force; memo
  checks at first force and replays cached terminal outcomes thereafter.

## 0. Motivation

NOTE-014 GAP 4 asked when contracts fire for lazy and memoized computations:

- Is a `requires` on a lazy argument checked at call site or force site?
- Is an `ensures` on a lazy result checked at construction or force?
- Does a memoized computation check once or every access?

NOTE-025 §7.9 raised the parallel purity question: type-level attributes sit beside the
row. Should `lazy`, `memo`, or the handler marker make a function impure simply by being
present?

The answer is one principle:

```text
Purity is denotational: a term is pure when it is referentially transparent.
Type attributes are purity-preserving when they preserve referential transparency.
Rows account for user-visible effects. Attributes do not silently add row effects.
```

This separates language semantics from implementation mechanics. A memoized pure thunk may
allocate or write a cache cell at runtime, but that cache write is not an Ash effect exposed
to the program. Conversely, a lazy thunk with latent row `{Db::read}` is effectful when
forced, because the latent row contains `Db::read`.

## 1. Purity model

### 1.1 Denotational purity is the language-level test

Ash classifies purity by referential transparency: a term is pure if it can be replaced by
its denotation without changing any program observation at the same type and mode boundary.

This definition intentionally ignores implementation-only mechanisms that do not appear in
the Ash effect row:

- allocating a thunk closure;
- writing a memo cache cell;
- replaying a cached terminal outcome;
- installing an internal handler frame that fully discharges the handled row.

Those operations are runtime implementation details. They may be visible to a profiler or
trace tool, but they are not user-visible effects unless the language exposes them through a
row item, trap, diagnostic, or capability.

### 1.2 Rows remain the first-order purity test

A computation with an empty residual row is pure at its observation boundary:

```text
row = {}  ⇒  no user-visible effects remain at that boundary
```

For strict values, the observation boundary is evaluation of the expression. For lazy and
memo values, the observation boundary is force. For handlers, the observation boundary is
handler application: the handler consumes a thunk and returns an answer with a residual row.

This means a mode attribute does not erase the latent row:

```text
lazy A {Db::read}     -- constructing the thunk is pure; forcing it is effectful
memo A {Db::read}     -- constructing the thunk is pure; first force is effectful
lazy A {}             -- constructing and forcing are pure, modulo divergence/bottom
memo A {}             -- constructing and forcing are pure, modulo cached terminal outcome
```

SPEC-097b §15.6 already states the row accounting rule: mode affects **when** effects fire,
not **what** effects are present. NOTE-028 extends that rule to purity and contracts.

### 1.3 Attribute classification

| Attribute | Denotationally purity-preserving? | Operationally effect-free? | Purity decision |
|---|---:|---:|---|
| `strict` | yes | yes | Pure iff residual row is empty. |
| `lazy` | yes | not necessarily | Pure iff latent row is empty at force sites. Delaying bottom/effects is temporal behavior, not impurity by itself. |
| `memo` | yes | no (cache cell) | Pure iff latent row is empty at force sites. Cache mutation is an implementation effect, not an Ash row effect. |
| `handler` marker | yes | not necessarily | Pure iff the handler's residual row is empty after peeling the handled operations. The marker itself is not an effect. |

The key distinction is between **the attribute** and **the computation under the attribute**.
`memo` does not make a pure computation impure. A memoized computation can still be effectful
if its latent row is non-empty.

### 1.4 Divergence and bottom

Divergence and bottom are observations, so explicit mode conversion remains `_unsafe` when it
changes when or how bottom is observed. This does not mean `lazy` or `memo` are impure.

Instead:

- `lazy` preserves the denotation of a delayed computation at the lazy boundary. Forcing is
  the observation that may diverge or trap.
- `memo` preserves the denotation of a delayed computation and shares its first terminal
  outcome. A cached trap is replayed as the same terminal outcome.
- Converting between modes can change temporal behavior. That is why SPEC-097b §15.5 uses
  `force_unsafe`, `memoize_unsafe`, and `strip_cache_unsafe`.

The rule is: **mode conversion can be unsafe without the target mode being impure.**

## 2. Type-level consequences

### 2.1 Mode types carry latent rows

The implemented Core model already represents lazy/memo modes as type wrappers with latent
rows:

```text
(strict T)
(lazy T {row})
(memo T {row})
```

A strict type has no latent row. A lazy or memo type carries the row that will be charged at
force sites.

Purity is classified by the relevant observation row:

```text
pure(strict T)      iff current residual row = {}
pure(lazy T ρ)      iff ρ = {} at force sites
pure(memo T ρ)      iff ρ = {} at force sites
```

This is not a subtyping rule. Modes remain invariant (SPEC-097b §15.4):

```text
strict A  ≮: lazy A
lazy A    ≮: strict A
memo A    ≮: lazy A
lazy A    ≮: memo A
```

### 2.2 Handler-marked function types

A handler-marked function type is a function type with a type-level marker:

```text
handler (Unit -> {op | r} A) -> {r} Ans
```

The marker does not add an effect row item. A handler-marked function is pure when applying
it leaves an empty residual row:

```text
pure(handler (Unit -> {op} A) -> {} Ans)      -- yes, if handler body itself has row {}
pure(handler (Unit -> {op | r} A) -> {r} Ans) -- polymorphic; pure only when r = {}
```

The handler may interpret an effectful computation, but interpretation is a fold over the
free computation generated by the handled operations. If the fold discharges all operations
and performs no residual effects, it is referentially transparent at the handler boundary.

### 2.3 No attribute-to-row coercion

No type attribute is automatically converted into a row item:

```text
lazy  ≠ {Lazy}
memo  ≠ {State::write}
handler marker ≠ {Handler::interpret}
```

Rows describe user-visible effects. Attributes describe evaluation or typing behavior.
Operational runtime structures may support an attribute, but they do not leak into the row
unless Ash exposes them as effects.

## 3. Contract timing principle

Contracts fire at the boundary where the contracted value is observed.

```text
strict value: observe at evaluation / call / return
lazy value:   observe at each force
memo value:   observe at first force; replay cached terminal outcome thereafter
handler:      observe at handler application and at each handled/residual operation boundary
```

This rule gives one answer for all GAP 4 questions:

- a `requires` on a lazy argument is checked when the argument's value is forced, not when
  the thunk is passed;
- an `ensures` on a lazy result is checked when the result is forced, not when the thunk is
  constructed;
- a memoized computation checks on first force and caches the terminal outcome (success,
  failure, or trap) according to the existing memo runtime behavior;
- invariants on lazy structures are checked when the relevant boundary is observed.

## 4. Contract timing by mode

### 4.1 Strict mode

Strict mode uses the current NOTE-014 rules:

| Contract | Check time | Blame on failure |
|---|---|---|
| `requires` | function entry / call boundary | caller |
| `ensures` | function exit / return boundary | callee/impl |
| `invariant` | loop/data boundary | boundary-dependent |

A strict function evaluates its body before returning, so an `ensures` clause can inspect the
result immediately.

### 4.2 Lazy mode

A lazy binding allocates a thunk at construction and evaluates the body each time it is
forced. Construction does not evaluate the body and therefore cannot check predicates that
refer to the body result.

| Contract on lazy value | Check time | Recheck? | Blame on failure |
|---|---|---:|---|
| `requires` on lazy argument's eventual value | each force before body use | yes | original caller/provider of the thunk |
| `ensures` on lazy result | each force after body evaluation | yes | thunk producer / callee |
| `invariant` on lazy data boundary | when boundary is forced/observed | yes | boundary-dependent |

Lazy re-runs the thunk body, so dynamic contracts re-run as well. This is consistent with
`docs/reference/core-ash-lazy-memo-modes.md`: **lazy: every force re-runs the thunk body**.

### 4.3 Memo mode

A memo binding allocates a thunk with a memo cell. The first force evaluates the body and
records the terminal outcome. Later forces replay the cached outcome.

| Contract on memo value | Check time | Recheck? | Blame on failure |
|---|---|---:|---|
| `requires` on memo argument's eventual value | first force | no; replay cached failure | original caller/provider of the thunk |
| `ensures` on memo result | first successful body evaluation, before caching success | no; replay cached terminal outcome | thunk producer / callee |
| `invariant` on memo data boundary | first observed boundary | no; replay cached terminal outcome | boundary-dependent |

A cached success is reused. A cached contract violation or trap is also reused. This follows
the implemented memo behavior: terminal successes and cacheable failures/traps are replayed.

The static force-site row still includes the latent row at every force site. A later
state-sensitive analysis may prove a memo cell is filled on a path, but the ordinary checker
must not erase the latent row from a force site just because the value is memoized.

## 5. Worked examples

### 5.1 Lazy parameter precondition

Proposed surface example:

```ash
fn use_positive(lazy x: Int) -> Int
    requires: x > 0
{
    if need_value() {
        force_unsafe(x) + 1
    } else {
        0
    }
}
```

The `requires: x > 0` cannot be checked at call time without forcing `x`. The check is
therefore attached to the force boundary:

```text
call use_positive(thunk)
  -- no force, no x > 0 check yet

force x
  evaluate thunk body
  check x > 0
  if false: ContractViolation(blame = caller/provider of thunk)
```

If `need_value()` is false, the thunk is never forced and the precondition is never observed.
This is not a missed check; it is the semantics of a lazy parameter. The contract is about
the value when demanded.

### 5.2 Lazy result postcondition

Proposed surface example:

```ash
fn lazy_head(xs: List<Int>) -> lazy Int
    ensures: result >= 0
{
    lazy compute_head(xs)
}
```

The function returns a lazy result, so `result >= 0` is checked when the result is forced:

```text
let h = lazy_head(xs)
  -- constructs thunk; no result yet

force h
  evaluate compute_head(xs)
  check result >= 0
  if false: ContractViolation(blame = lazy_head / callee)
```

A lazy result re-runs on each force, so the `ensures` check re-runs on each force.

### 5.3 Memo result postcondition

Proposed surface example:

```ash
fn memo_score(user: User) -> memo Int
    ensures: result >= 0
{
    memo expensive_score(user)
}
```

The first force evaluates the body and checks the postcondition before caching success:

```text
force score
  evaluate expensive_score(user)
  check result >= 0
  if true: cache success(result)
  if false: cache ContractViolation(diagnostic)

force score again
  replay cached success(result) or cached ContractViolation(diagnostic)
```

The check does not re-run on cache hit. Re-running would make memoization observable through
contract side effects and would contradict the memo rule that terminal outcomes are replayed.

### 5.4 Pure memoization

```ash
let memo fib40 = fib(40);   -- latent row {}
```

`fib40` is pure under the denotational rule. The runtime may allocate a memo cell and fill it
on first force, but no Ash-visible effect occurs. Replacing `force fib40` with its cached
value preserves program observations.

If the body has a latent row, purity changes because the row changes:

```ash
let memo n = {Db::read} read_user_count();
```

The memo attribute is still purity-preserving, but the thunk is not pure at force sites.
Forcing has latent row `{Db::read}`. A cache hit may dynamically perform no `Db::read`, but
the static row remains the force-site obligation.

### 5.5 Pure handler marker

```ash
handler maybe_handler<A>(comp: Unit -> {Maybe::none} A) -> Option<A> {
    on comp() {
        Maybe::none { None }
        done(value) { Some(value) }
    }
}
```

The handler interprets `Maybe::none` into `Option`. If its body has no residual effects, the
handler-marked function is pure: the marker identifies handler intent, not impurity. The
handled operation is consumed by the fold; no residual row remains.

If the handler logs while handling, the logging row makes it effectful:

```ash
handler logging_maybe<A>(comp: Unit -> {Maybe::none} A) -> {Stdout::write} Option<A> {
    on comp() {
        Maybe::none {
            print("none")
            None
        }
        done(value) { Some(value) }
    }
}
```

The impurity comes from `{Stdout::write}`, not from the handler marker.

## 6. Interaction with blame and diagnostics

NOTE-027's blame labels remain stable under lazy and memo timing:

- `requires` failure on a delayed argument blames the original caller/provider of the thunk,
  even if the force occurs later in a different function.
- `ensures` failure on a delayed result blames the thunk producer/callee.
- memo replay preserves the original diagnostic payload and blame label.

This matters for memoized contract failures. A later force does not create a new blame event;
it observes the cached terminal outcome. The audit trail may record a replay event, but the
underlying violation keeps its original `ContractDiagnostic`.

## 7. Open questions

1. **Path-sensitive memo row erasure.** SPEC-097b §15.6 allows a future state-sensitive
   analysis to prove a memo cell is already filled on a path. When that exists, can a force
   site row be narrowed to `{}` on proven cache-hit paths? Deferred.

2. **Contract predicates with time-sensitive observations.** Contract predicates should be
   pure, but a predicate can mention values whose interpretation depends on time or force
   count if the surface allows it. The contract language should reject or quarantine such
   predicates. Deferred to the contract-predicate well-formedness track.

3. **Operational purity as a separate diagnostic lens.** Denotational purity is the language
   rule. Tooling may still expose an operational-purity/debug lens (allocates thunk, writes
   memo cell, installs handler frame). That lens should not affect typing unless promoted to
   a first-class row/capability.

4. **Proc/Workflow temporal contracts.** This note resolves Pure/Act-level lazy/memo timing.
   Proc/Workflow contracts involve liveness and monitoring; those remain NOTE-014 GAP 5.

## 8. Working Principle

```text
Purity is denotational: referential transparency is the language-level test.
Rows account for user-visible effects; type attributes are not row items.
strict/lazy/memo are purity-preserving attributes.
A lazy or memo value is pure at force sites iff its latent row is empty.
Memo cache mutation is an implementation detail, not an Ash-visible effect.
The handler marker is purity-preserving; impurity comes from the handler's residual row.
Contracts fire at observation boundaries.
strict: check at call/return/boundary.
lazy: check on every force.
memo: check on first force, cache terminal outcome, replay success/failure/trap thereafter.
Blame labels attach to the original provider/callee/caller, not the later force site.
Mode conversions can be unsafe without making the target mode impure.
```

## 9. References

Internal references:

- [NOTE-014: Contract Systems Unification](NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md) — GAP 4
  (contracts × evaluation modes), GAP 5 (temporal contracts), GAP 6 (failure observability)
- [NOTE-023: Handler Surface — Dispatch Side](NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md) —
  handler marker as type-level attribute
- [NOTE-025: Effect Identity via Sorts and Impls](NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md)
  — §7.9 parked purity-classification question
- [NOTE-027: Contract Blame and Subsumption](NOTE-027-CONTRACT-BLAME-AND-SUBSUMPTION.md) —
  blame labels and diagnostic payloads
- [SPEC-027: Pure Functions](../spec/SPEC-027-PURE-FUNCTIONS.md)
- [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md) — §15 evaluation
  modes, §15.5 explicit conversions, §15.6 row accounting
- [SPEC-101: Lazy and Memo Computation Modes](../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md)
  — source design for lazy/memo evaluation modes
- [Core Ash Lazy and Memo Modes](../reference/core-ash-lazy-memo-modes.md) — implemented Core
  behavior for lazy/memo thunk typing, lowering, runtime behavior, and traces

External references:

- Launchbury, John. "A Natural Semantics for Lazy Evaluation" (1993). Defines a natural
  semantics for lazy evaluation with sharing. https://doi.org/10.1145/158511.158618
- Ariola, Zena M.; Felleisen, Matthias. "The Call-by-Need Lambda Calculus" (1997). Formalizes
  call-by-need evaluation and sharing. https://doi.org/10.1017/S0956796897002724
- Sabry, Amr; Felleisen, Matthias. "Reasoning about Programs in Continuation-Passing Style"
  (1993). Relevant to contextual equivalence and CPS observations.
  https://doi.org/10.1145/155090.155113
- Findler, Robert Bruce; Felleisen, Matthias. "Contracts for Higher-Order Functions" (2002).
  Blame theory background for delayed contract failures. https://doi.org/10.1145/581478.581484

## 10. Changelog

- 2026-06-28: Initial version. Resolves NOTE-014 GAP 4 and NOTE-025 §7.9. Defines
  denotational purity (referential transparency) as the language-level purity rule for
  type-level attributes. Classifies `strict`/`lazy`/`memo` and the handler marker as
  purity-preserving attributes; impurity comes from residual/latent rows, not attributes.
  Defines contract timing: strict checks at call/return, lazy checks on every force, memo
  checks on first force and replays cached terminal outcomes. Connects timing to NOTE-027
  blame labels and diagnostics.
