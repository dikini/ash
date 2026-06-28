# NOTE-030: Monadic Hoare Logic for Ash Computations

**Date:** 2026-06-28
**Status:** Living document — design direction captured; resolves NOTE-014 GAP 2
**Purpose:** Define how Hoare contracts compose through Ash computation sequencing. Rows compose
by union, but contracts compose through predicate transformers: the postcondition of the first
computation must establish the precondition of the continuation, and the final postcondition is
threaded through the intermediate value.

Companion to NOTE-013 (ambient monad and handler composition), NOTE-014 (contract systems
unification), NOTE-027 (blame and subsumption), NOTE-028 (evaluation-mode contract timing),
NOTE-029 (structured bottom), SPEC-097b (target type system), SPEC-098b (target IR), and
SPEC-099 (Core language).

## Pre-Spec Delta

This note is pre-spec and resolves NOTE-014 §12 GAP 2. When promoted into target specs,
reconcile:

- **SPEC-097b Target Type System:** add a contract-composition rule for `bind` / computation
  sequencing. The row of `bind(m, k)` is `ρm ∪ ρk`; the contract summary is computed by a
  predicate-transformer rule, not by row union.
- **SPEC-098b Target IR:** preserve enough `ContractDischarge` metadata to connect a
  continuation precondition to the producer postcondition that discharged it.
- **SPEC-099 Core language:** specify that Core sequencing and `LetCont`/`Jump` composition
  may record composed contract summaries as sidecar metadata without adding new term forms.
- **SPEC-100 Core type checking:** add proof obligations for continuation precondition
  discharge: if `m` ensures `Q(a)` and `k(a)` requires `R(a)`, the checker/prover must
  establish `∀a. Q(a) ⇒ R(a)` or leave a residual dynamic check.

## 0. Motivation

NOTE-013 made sequencing explicit:

```text
bind : Comp<ρ₁, A> -> (A -> Comp<ρ₂, B>) -> Comp<ρ₁ ∪ ρ₂, B>
```

Rows are easy here. The combined computation requires everything required by the producer and
everything required by the continuation.

Contracts are harder. A continuation can require facts about the value produced by the first
computation:

```text
m    : Comp<ρ₁, A>              -- produces an intermediate a
k    : A -> Comp<ρ₂, B>         -- may require facts about that a
bind : Comp<ρ₁ ∪ ρ₂, B>
```

The type checker cannot compose these contracts by simply unioning predicates. It must prove
that the values `m` can produce are acceptable inputs to `k`. Without this rule, modular
verification collapses into inlining: every composed computation must be re-proved from
scratch.

## 1. Core decision

Ash treats contract composition through `bind` as a weakest-precondition / predicate-transformer
rule over the ambient computation monad.

```text
Rows compose by union.
Contracts compose by continuation precondition discharge.
```

In data-dependent form:

```text
m : Comp<ρm, A>
  requires P
  ensures  Q(a)

k : A -> Comp<ρk, B>
  requires R(a)
  ensures  S(a, b)

bind(m, k) : Comp<ρm ∪ ρk, B>
  requires P ∧ ∀a. Q(a) ⇒ R(a)
  ensures  ∃a. Q(a) ∧ S(a, b)
```

The middle obligation is the important part:

```text
∀a. Q(a) ⇒ R(a)
```

It says: for every value the first computation may deliver, the continuation's precondition
must hold. If the checker can prove this statically, the continuation precondition is discharged
by the producer's postcondition. If it cannot, the boundary remains dynamic and lowers to a
runtime check at the point where `k` is invoked.

This is the contract analogue of row union. Row union says which effects may happen. The
predicate transformer says which facts must be true when each step starts.

## 2. Grammar impact

No new Ash surface syntax is introduced by this note.

The rule applies to existing sequencing forms after lowering:

```ash
let a = m();
k(a)
```

Core/Ash explanatory notation:

```text
bind(m, k)
```

The `bind` notation is meta-notation from NOTE-013, not proposed surface syntax. Existing
`requires` and `ensures` clauses are enough:

```ash
fn parse(s: String) -> Parsed
    requires: len(s) > 0
    ensures: result.source_len == len(s)
{ ... }

fn compile(p: Parsed) -> Module
    requires: p.source_len > 0
    ensures: result.item_count >= 0
{ ... }
```

The composition rule is a type-checker/prover rule over the lowered Core shape. It does not
require programmers to write an explicit `bind` form.

## 3. Types and contract summaries

### 3.1 Computation summaries

A typed computation carries a row summary and a contract summary:

```text
CompSummary<A> = {
  row: ρ,
  requires: Predicate Γ,
  ensures: Predicate (Γ, result: A),
  discharge: Vec<ContractDischarge>
}
```

This is explanatory notation. The implementation may store summaries in typed Core nodes,
module summaries, or sidecar metadata.

For a continuation, the summary is a family indexed by the intermediate value:

```text
KSummary<A, B> = ∀a: A. CompSummary<B> under Γ, a: A
```

The type checker treats continuation contracts as value-dependent. A continuation can require
a fact about the `a` it receives.

### 3.2 Bind summary rule

Given:

```text
Γ ⊢ m : CompSummary<A> = { row = ρm, requires = P, ensures = Q(a) }
Γ, a: A ⊢ k(a) : CompSummary<B> = { row = ρk(a), requires = R(a), ensures = S(a, b) }
```

Then:

```text
Γ ⊢ bind(m, k) : CompSummary<B> = {
  row      = ρm ∪ ⋃a.ρk(a),
  requires = P ∧ ∀a. Q(a) ⇒ R(a),
  ensures  = ∃a. Q(a) ∧ S(a, b)
}
```

For ordinary non-row-polymorphic Core, `ρk(a)` is a static row expression and the row summary
is just `ρm ∪ ρk`. The `⋃a` notation only says that the continuation summary is a family; it
is not a value-level row computation.

### 3.3 User-chosen final postconditions

The existential postcondition is the strongest generic summary the checker can infer from the
two pieces. A function signature usually declares a simpler postcondition `T(b)`. The checker
therefore proves:

```text
∀b. (∃a. Q(a) ∧ S(a, b)) ⇒ T(b)
```

If the declared final postcondition is stronger than the inferred composed postcondition, the
function needs additional proof, a stronger continuation postcondition, or a dynamic check.

## 4. Semantics

### 4.1 Static discharge

When the implication is provable:

```text
∀a. Q(a) ⇒ R(a)
```

the continuation precondition is discharged statically. The IR records a `ContractDischarge`
entry showing that `R(a)` was discharged by the producer postcondition `Q(a)`.

This matters for audit. The residual row should not contain a runtime `requires R(a)` item if
the type checker has already proved it, but the proof boundary must remain visible in metadata.

### 4.2 Dynamic demotion

When the implication is not statically provable, Ash may keep a dynamic check at the bind
boundary:

```text
let a = m();
-- dynamic requires check for k(a)
k(a)
```

If the dynamic check fails, NOTE-029 applies:

```text
Trap { reason: ContractViolation(ContractDiagnostic) }
```

The blame label follows NOTE-027. Because `R(a)` is a precondition of `k`, failure blames the
caller of `k` at that boundary. In a desugared bind, the caller is the composed computation that
fed `a` into `k`; the diagnostic should also point to the producer postcondition that failed to
establish the required fact.

### 4.3 Recoverable contract behavior

Recoverable behavior is not special to monadic composition. If a surface construct chooses a
recoverable contract boundary, the failed continuation precondition lowers to explicit `fail`
and the row exposes that failure item:

```text
row(bind(m, k)) = ρm ∪ ρk ∪ {fail ContractError}
```

`ContractViolation` itself remains structured bottom metadata unless explicitly mapped into
`fail`, as in NOTE-029.

### 4.4 Handler interaction

Handlers do not change the proof rule. They may change the row by discharging effect operations,
but the contract summary still composes through the value passed to the continuation.

If a handler transforms the intermediate value, the postcondition used for `Q(a)` is the
postcondition after the handler's transformation. If a handler catches a recoverable `fail` and
resumes, the diagnostic history records that decision; it does not rewrite the original blame
label.

## 5. Worked examples

### 5.1 Successful static composition

Surface Ash example:

```ash
fn parse_nonempty(s: String) -> Parsed
    requires: len(s) > 0
    ensures: result.source_len == len(s)
{ ... }

fn compile_nonempty(p: Parsed) -> Module
    requires: p.source_len > 0
    ensures: result.item_count >= 0
{ ... }

fn compile_source(s: String) -> Module
    requires: len(s) > 0
    ensures: result.item_count >= 0
{
    let p = parse_nonempty(s);
    compile_nonempty(p)
}
```

Composition facts:

```text
P      = len(s) > 0
Q(p)   = p.source_len == len(s)
R(p)   = p.source_len > 0
S(p,m) = m.item_count >= 0
```

The checker proves:

```text
∀p. p.source_len == len(s) ∧ len(s) > 0 ⇒ p.source_len > 0
```

So the continuation precondition is discharged statically. The final postcondition follows
from `S`.

### 5.2 Failed composition requires dynamic check or rejection

```ash
fn parse_maybe_empty(s: String) -> Parsed
    ensures: result.source_len >= 0
{ ... }

fn compile_nonempty(p: Parsed) -> Module
    requires: p.source_len > 0
{ ... }

fn compile_source(s: String) -> Module {
    let p = parse_maybe_empty(s);
    compile_nonempty(p)
}
```

Here:

```text
Q(p) = p.source_len >= 0
R(p) = p.source_len > 0
```

The implication fails:

```text
∀p. p.source_len >= 0 ⇒ p.source_len > 0    -- false
```

The checker has three choices, depending on the compilation profile:

1. reject the composition statically;
2. require the caller to strengthen the outer precondition and prove it flows into `Q`;
3. leave a dynamic precondition check at the call to `compile_nonempty`.

If the dynamic check fails, the diagnostic blames the composed caller boundary and points to
`compile_nonempty`'s precondition.

### 5.3 Final postcondition threading

```ash
fn normalize(s: String) -> Normalized
    ensures: result.len <= len(s)
{ ... }

fn hash(n: Normalized) -> Hash
    ensures: result.input_len == n.len
{ ... }

fn hash_normalized(s: String) -> Hash
    ensures: result.input_len <= len(s)
{
    let n = normalize(s);
    hash(n)
}
```

The composed strongest postcondition is:

```text
∃n. n.len <= len(s) ∧ result.input_len == n.len
```

The declared postcondition follows:

```text
∃n. n.len <= len(s) ∧ result.input_len == n.len
  ⇒ result.input_len <= len(s)
```

This is the common case: the inferred composed postcondition is more detailed than the public
signature should expose.

## 6. `old(x)` and snapshots across bind

`old(x)` is a boundary-local snapshot. In a composed computation, each contract boundary owns
its own snapshot environment:

```text
m requires/ensures boundary: old_m(...)
k requires/ensures boundary: old_k(...)
outer function boundary:     old_outer(...)
```

The bind rule must not conflate these snapshots. When a continuation postcondition mentions
`old(a)`, it means the value of `a` at entry to `k`, not the producer's internal pre-state.

A composed final postcondition may relate the outer old state to the final result only through
facts exported by `m` and `k`:

```text
Q(a, old_outer) ∧ S(a, b, old_k) ⇒ T(b, old_outer)
```

For most direct `let` sequencing, `old_k(a) = a` at the continuation boundary. The important
rule is provenance: diagnostics must say which boundary a snapshot came from.

## 7. Lowering and metadata

No new Core or CPS term form is required. The rule is metadata and proof structure over existing
sequencing.

A direct-style Core shape:

```text
LetVal a = m in
  k(a)
```

or CPS shape:

```text
m(λa. k(a)(k_final))
```

gets a composed contract summary:

```text
ComposedContract {
  producer: DischargeRef(Q),
  continuation_requires: DischargeRef(R),
  proof_obligation: ForAll(a, Implies(Q(a), R(a))),
  mode: Static | Dynamic | Evidence,
}
```

This sidecar is enough for diagnostics, evidence caching, and optimizer safety. Optimizations
may reassociate binds only if they preserve the composed contract evidence and snapshot
boundaries.

## 8. Design decisions

1. **No new surface syntax.** Monadic Hoare composition is a type-checker/prover rule over
   existing sequencing.
2. **Rows still compose by union.** Contract predicates do not; they compose through weakest
   precondition / predicate-transformer reasoning.
3. **Continuation preconditions are obligations at the bind boundary.** They are discharged by
   producer postconditions when the implication is provable.
4. **The generic composed postcondition is existential over the intermediate value.** Public
   signatures may expose a simpler consequence.
5. **Dynamic fallback is allowed only as an explicit discharge strategy.** Default failure is
   structured bottom; recoverability must use `fail`.
6. **Snapshots are boundary-local.** `old(x)` must preserve which contract boundary captured it.
7. **Handler discharge does not rewrite the proof rule.** Handlers may change rows and values,
   but the composed contract uses the post-handler intermediate value.

## 9. Open questions

1. **Quantifier profile.** How much first-order quantification does the initial SMT profile
   permit for `∀a. Q(a) ⇒ R(a)`? A conservative implementation may require annotations or
   dynamic fallback for hard quantified obligations.
2. **Evidence granularity.** Should `ComposedContract` store the full proof object, a compact
   evidence reference, or only a source-span link to the producer/continuation contracts?
3. **Predicate language for existentials.** The generic postcondition uses `∃a`. If the public
   predicate language avoids existential syntax, this remains internal proof metadata rather
   than source syntax.
4. **Associativity of evidence.** Bind reassociation is semantically valid, but evidence trees
   may differ. We need a canonical evidence normalization if optimizers reassociate large
   composed computations.

## 10. References

### Internal references

- [NOTE-013: Ambient Monad and Handler Composition Algebra](NOTE-013-AMBIENT-MONAD-AND-HANDLER-COMPOSITION-ALGEBRA.md)
  — defines Ash's row-indexed ambient continuation monad and row-polymorphic `bind`.
- [NOTE-014: Contract Systems Unification](NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md)
  — original GAP 2 statement and contract-system context.
- [NOTE-027: Contract Blame and Subsumption](NOTE-027-CONTRACT-BLAME-AND-SUBSUMPTION.md)
  — blame polarity and Hoare subsumption for interface/impl contracts.
- [NOTE-028: Purity, Evaluation Modes, and Contract Timing](NOTE-028-PURITY-EVALUATION-MODES-AND-CONTRACT-TIMING.md)
  — contract timing across strict/lazy/memo observation boundaries.
- [NOTE-029: Structured Bottom and Contract Diagnostics](NOTE-029-STRUCTURED-BOTTOM-AND-CONTRACT-DIAGNOSTICS.md)
  — structured bottom, diagnostics, and explicit `fail` boundary.
- [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
  — target row typing, contract subsumption, and evaluation modes.
- [SPEC-098b: Target IR](../spec/SPEC-098b-TARGET-IR.md)
  — CPS IR, `ContractDischarge`, `BlameLabel`, and `ContractDiagnostic` metadata.
- [SPEC-099: Core Language](../spec/SPEC-099-CORE-LANGUAGE.md)
  — Core sequencing, dynamic contract checks, and structured contract traps.

### External references

- C. A. R. Hoare. "An Axiomatic Basis for Computer Programming." Communications of the ACM,
  1969. DOI: <https://doi.org/10.1145/363235.363259>. Original Hoare triples.
- Robert Atkey. "Parameterised Notions of Computation." Journal of Functional Programming,
  2009. URL: <https://bentnib.org/param-notions.pdf>. Indexed/parameterized monads; useful
  for row-indexed computation families.
- N. Swamy, D. Hriţcu, C. Keller, A. Rastogi, A. Delignat-Lavaud, S. Forest, K. Bhargavan,
  C. Fournet, P.-Y. Strub, M. Kohlweiss, J. K. Zinzindohoué, and S. Zanella-Béguelin.
  "Dependent Types and Multi-Monadic Effects in F*." POPL 2016. URL:
  <https://www.fstar-lang.org/papers/mumon/>. Dijkstra-style weakest-precondition monads in a
  practical effectful language.
- N. Swamy, C. Hritcu, C. Keller, A. Rastogi, A. Delignat-Lavaud, S. Forest, K. Bhargavan,
  C. Fournet, P.-Y. Strub, M. Kohlweiss, J. K. Zinzindohoué, and S. Zanella-Béguelin.
  "Dijkstra Monads for All." 2013/2016 lineage. URL:
  <https://www.fstar-lang.org/papers/dm4free/>. Weakest-precondition calculus for monadic
  effects.

## 11. Changelog

| Date | Change |
|------|--------|
| 2026-06-28 | Initial note. Resolves NOTE-014 GAP 2 by defining contract composition through `bind` as predicate-transformer reasoning: rows union, continuation preconditions are discharged by producer postconditions, and final postconditions existentially thread the intermediate value. |
