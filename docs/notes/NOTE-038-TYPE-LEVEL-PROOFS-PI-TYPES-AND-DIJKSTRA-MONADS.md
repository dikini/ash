# NOTE-038: From Ash Type-Level Proofs to Π-Types and Dijkstra Monads

**Date:** 2026-07-07
**Status:** Living document — design direction and research roadmap
**Purpose:** Synthesize the current Ash type system's proof capabilities, the
existing contract lowering and monadic Hoare composition, the path to
Ash-native weakest-precondition inference over the Ash monad, the optional
extension to dependent function types (Π-types), and the resulting connection to
Hoare triples, laws, contracts, and Dijkstra monads. Estimate implementation
effort and identify research questions.

**Companion to:** NOTE-030 (monadic Hoare logic), NOTE-031 (predicate
well-formedness), NOTE-033 (surface-to-Core lowering), NOTE-034
(contract-capability boundary), NOTE-035 (temporal contracts), NOTE-036
(gradual verification), NOTE-037 (symbolic-connectionist duality), SPEC-064
(constraint/proposition layer), SPEC-096b (target effect system), PLAN-194
(contract and evidence system), and the literature survey [Verification,
Prover Integration, and LLM-Driven Proving: A Literature
Survey](../reference/verification-and-prover-integration-survey.md).

---

## 1. Summary

Ash already has:

1. A conservative type-level proposition layer (SPEC-064) that proves structural
   facts: normalized type equality, sealed-domain constructor disjointness,
   interface bounds, and row subsumption.
2. A contract lowering pipeline (NOTE-033) that turns surface `requires`/`ensures`
   into Core `LoweredPredicate` artifacts, classifies them as `Static` or
   `Dynamic`, and emits either `PredicateProofObligation`s or `RuntimeCheckPlan`s.
3. A monadic Hoare composition rule (NOTE-030) that treats `bind` as a
   predicate-transformer: continuation preconditions are discharged by producer
   postconditions.

What is missing is a **general weakest-precondition (WP) inference** layer that
derives these predicate transformers directly from the **Ash monad**. The Ash
monad — the substrate of `do`/`[]` notation and effect rows — already provides
`return`, `bind`, and primitive effect operations. By assigning a WP transformer
to each primitive and using the monad laws for composition, the compiler can
infer WPs generically. This is the **Dijkstra monads for free** construction
applied to Ash.

This note argues that Ash should pursue **Ash-native Dijkstra monads first**,
keeping WPs as an internal compiler feature and external solvers/testers as
optional evidence providers. Π-types become relevant only if/when Ash wants to
expose WPs as user-written types or check user-supplied proof terms.

The note also refines the relation between **effect rows** and **weakest
preconditions**: rows are not merely syntax for WPs, nor WPs merely refinements
of rows. They are related by a **body-guided abstraction/refinement pair** that
must be parameterized by **evaluation mode** (eager, lazy, memo) and
**per-argument strictness**.

---

## 2. What Ash can prove today

The current Ash type system, via SPEC-064 and the built-in checks in
`ash-core` / `ash-typeck`, can discharge the following fact classes without any
external prover.

### 2.1 Normalized type equality

Given a type equality proposition `T == U`, the normalizer reduces both sides
and checks definitional equality. Examples:

```text
Append<Nil, Ys> == Ys
<Iterator<List<A>>>::Item == A
```

Satisfied when both sides reduce to the same canonical type expression.

### 2.2 Conservative disequality

Sealed-domain constructors with different heads are provably disjoint even when
their arguments contain open variables:

```text
Cons<A, T> != Nil
```

### 2.3 Interface bounds

`T : Iterator` is satisfied by concrete impl evidence, an in-scope where-bound,
or trusted imported summary evidence. The solver does not perform new impl
search or output-driven unification.

### 2.4 Effect-row containment and subsumption

Facts such as "this computation's row is a subrow of `{...}`" or "this
computation does not require operation `O`" are built-in decidable checks.

### 2.5 Simple arithmetic and constructor disjointness

Lightweight built-in fragments: SPEC-064 normalization, simple arithmetic, and
obvious constructor-head disjointness.

### 2.6 Compiler-known named predicates

Named predicates such as `prop Sorted<Xs>` can be recorded and checked if
registered as builtins; arbitrary named predicates are deferred.

### 2.7 What is deliberately out of scope

SPEC-064 excludes:

- solving under neutral computation heads,
- type-function or associated-family inversion,
- output-driven unification from proposition goals,
- unrestricted SMT, proof search, proof terms, tactics,
- higher-kinded logic, holes, implicit currying,
- value-level runtime predicates and capability-provider evaluation.

Consequently, **value-level laws** such as associativity, commutativity, or
identity cannot be proved by the type system alone.

### 2.8 Existing contract lowering and predicate transforms

Ash already implements much of the predicate infrastructure that a WP calculus
needs. The implementation in `crates/ash-core/src/core_ash_contract.rs` and
`crates/ash-interp/src/predicate_evaluator.rs` includes:

- `LoweredPredicate` and `PredicateNode` — the Core predicate AST from
  NOTE-033.
- `PredicateClassification` (`Static`, `Dynamic`) — decides whether a predicate
  is discharged statically or evaluated at runtime.
- `RuntimeCheckPlan` and `DynamicPredicatePlan` — runtime check artifacts for
  dynamic predicates.
- `ContractDischargeStatus` (`StaticProven`, `StaticModelChecked`, `StaticProved`,
  `Dynamic`) — evidence outcomes.
- `PredicateProofObligation` — proof obligations emitted for static predicates.
- `ComposedContract` — metadata connecting producer postconditions to
  continuation preconditions, implementing NOTE-030's monadic Hoare composition.
- `PredicateEntailment` — subsumption obligations for interface/impl contracts.

So the static-vs-dynamic transform, the bind-composition rule, and the runtime
check plan already exist. The missing layer is the **general WP inference** that
produces these obligations from the Ash monad structure rather than from
hand-crafted contract-composition rules.

---

## 3. Extending Ash with Π-types (deferred)

Π-types are **not required** for Ash-native Dijkstra monads. They become
relevant only if Ash decides to expose WPs as user-written types or to check
user-supplied proof terms. This section records what Π-types would add and what
they would cost.

### 3.1 What becomes possible

With dependent function types, value-level laws become types and proofs become
functions. For example, associativity of a semigroup operation becomes:

```text
Associative (<> : T -> T -> T) : Type =
  (a : T) -> (b : T) -> (c : T) -> Eq ((a <> b) <> c) (a <> (b <> c))
```

A value of that type is a proof, typically constructed by induction or by
cases.

This enables:

- interface laws checked instance-by-instance,
- data-structure invariants (balanced trees, sorted lists, length bounds),
- preconditions and postconditions as types,
- effect-sandboxing proofs beyond row subsumption.

### 3.2 Required ingredients

To add Π-types to Ash without breaking the existing type/effect infrastructure,
the following are necessary:

1. **A proof/proposition universe** `Prop : Type`, proof-irrelevant and erasable.
2. **Π-types** `(x : A) -> B(x)` where the codomain depends on the argument.
3. **Higher-order quantification** so weakest preconditions can quantify over
   postconditions.
4. **A propositional equality type** `Eq : (A : Type) -> A -> A -> Prop`.
5. **Erasure** so that proof terms disappear at runtime and do not appear in
   effect rows.
6. **Termination checking** if the language remains total (or a partiality
   monad if not).

### 3.3 Staging options

Ash need not jump to full dependent types at once. Possible staged paths:

1. **Type-level Π only** — indexed families like `Vector<A, n>` where `n` is a
   type-level natural. This is close to the existing type-function substrate.
2. **Proof-irrelevant Π** — Π-types for propositions only, erased at runtime,
   no dependent return types for ordinary values.
3. **Full predicative Π** — the Idris/Lean model: value-dependent types,
   termination checking, proof erasure.
4. **Refinement + dependent effects** — the F\* model: keep effects first-class
   and use Dijkstra monads, which may fit Ash's workflow/effect model better
   than pure Π-types.

---

## 4. Relation to Hoare triples, laws, contracts, and the Ash monad

### 4.1 The Ash monad as the substrate

Ash's `do` notation and `[]` comprehensions desugar into the Ash monad:

```text
do {
    x <- op1
    y <- op2 x
    return y
}
```

is structurally:

```text
bind(op1, λx. bind(op2 x, λy. return y))
```

Effect rows replace monad transformers: instead of `StateT s (ErrorT e IO) a`,
a computation carries a row `{state, error, io}`. The row is the static
footprint; the monad is the dynamic interpreter.

### 4.2 Contracts as weakest preconditions over the Ash monad

An Ash contract such as:

```ash
fn sqrt(x: Int) requires { x >= 0 } ensures { result * result <= x } -> Int { ... }
```

is already a weakest-precondition specification. It can be read as:

```text
sqrt : (x : Int) -> M Int (λpost. x >= 0 ∧ ∀(r : Int). r*r <= x → post r)
```

where `M A wp` is a Dijkstra-monad computation type over the Ash monad `M`.

### 4.3 Laws as proof obligations

Interface laws are proof obligations attached to instances. Today, Ash supports
empirical evidence via `by test` (SPEC-081). With a WP calculus, laws can also
be discharged by built-in simplification, SMT providers, or (much later)
user-supplied proof terms under Π-types.

For example, a semigroup associativity law becomes a `PredicateProofObligation`
over the operation:

```text
∀(a, b, c : T). (a <> b) <> c == a <> (b <> c)
```

The obligation is recorded; discharge can be `verified` (solver/proof),
`tested` (property tests), or `deferred` (dynamic check).

### 4.4 Triples as computation summaries

A Hoare triple `{P} c {Q}` is already represented in Ash's computation summary:

```text
CompSummary<A> = {
  row: ρ,
  requires: Predicate Γ,
  ensures: Predicate (Γ, result: A),
  discharge: Vec<ContractDischarge>
}
```

This is exactly the information a WP needs. The next step is to generate these
summaries by inference from the monad structure rather than by hand-crafted
rules for each sequencing form.

### 4.5 Evidence rows as outcomes

NOTE-036's evidence-outcome lattice (`verified`, `tested`, `monitored`,
`deferred`, `refuted`, `untested`) classifies how a proof obligation is
discharged. A WP inference layer produces obligations; the discharge layer
classifies them. With Π-types, a `verified` outcome could optionally carry an
actual proof term; without Π-types, `verified` means discharged by a trusted
provider or built-in simplifier.

---

## 5. Inferring weakest preconditions from the Ash monad

### 5.1 The missing layer

Ash already has:

- the Ash monad (`return`, `bind`, effect operations),
- contract lowering (NOTE-033),
- monadic Hoare composition (NOTE-030).

The missing layer is a **WP inference pass** that walks the Core IR and derives
a weakest precondition from the monadic structure. For each primitive effect
operation, Ash registers a WP transformer. `return` and `bind` are derived once
and for all from the monad laws.

### 5.2 Primitive WP transformers

Each builtin effect operation gets a WP transformer. For example:

```text
wp(return a)        = λpost. post a
wp(bind(m, f))      = λpost. wp(m)(λx. wp(f x)(post))
wp(read_ref r)      = λpost. ∀v. post(v)
wp(write_ref r v)   = λpost. post()
wp(throw e)         = λpost. true   -- or exceptional postcondition
```

User contracts refine these inferred WPs. A `requires` clause strengthens the
precondition; an `ensures` clause strengthens the postcondition.

### 5.3 Dijkstra monads for free

F\*'s "Dijkstra monads for free" result shows that any monad `M` can be turned
into a Dijkstra monad `DM M` by a predicate-transformer CPS translation:

```text
DM M A wp = ∀post : A -> Prop. wp post -> M (post-result A post)
```

Applied to Ash, this means the compiler does not need a hand-crafted Hoare
logic for every effect. Instead, it derives the WP semantics from the existing
monadic structure of workflows. The survey §3.1 discusses this in detail.

The derivation is internal: users still write `requires`/`ensures`, and the
compiler uses the monad structure to compose and simplify the resulting WPs.

### 5.4 Relation to NOTE-030 and NOTE-033

NOTE-030's bind rule:

```text
bind(m, k) requires P ∧ ∀a. Q(a) ⇒ R(a) ensures ∃a. Q(a) ∧ S(a, b)
```

is exactly the WP of `bind` when `m` has WP `(P, Q)` and `k(a)` has WP `(R(a),
S(a, -))`. So NOTE-030 is already a special case of WP inference for the Ash
monad.

NOTE-033's static/dynamic classification is the discharge step: after WP
inference, the compiler asks whether each obligation is built-in dischargeable,
provider-dischargeable, or must become a `RuntimeCheckPlan`.

### 5.5 Ash-native discharge, solvers optional

WP inference and simplification are Ash-internal. They do not depend on Z3,
CVC5, Lean, or any external tool. The compiler discharges what it can using:

- SPEC-064 normalization and constructor disjointness,
- simple arithmetic,
- row subsumption,
- structural WP simplification (e.g., `wp(return x)(Q) = Q(x)`).

Residual obligations can be left `deferred`, turned into dynamic checks, or
submitted to optional evidence providers:

- `by solver` — SMT/Why3 for first-order fragments.
- `by lean` — proof assistant for quantified/inductive goals.
- `by test` — property/small-world tests for empirical evidence.
- `by llm` — suggestion only, must pass a trusted checker.

This keeps Ash self-contained. External provers extend the set of dischargeable
obligations but are not required for the mechanism to function.

### 5.6 Relation to effect rows

Effect rows and WPs are two projections of the same monadic computation:

```text
Ash monad computation M
    ├── row projection:   which effects may occur
    └── wp projection:    precise pre/post conditions
```

Rows are a **sound abstraction** of WPs:

```text
abst  : WP -> Row        -- extract the effect footprint
concr : Row -> WP        -- weakest WP consistent with the row
```

These form a Galois connection:

```text
abst(wp) ⊑ row   iff   wp ⊑ concr(row)
```

However, `concr` is refined by the **function body**, because the body imposes
an execution order (at minimum a pre-order) on operations. The same row
`{fs, http}` can correspond to different WPs depending on whether `fs` happens
before `http` or vice versa.

So the relation is body-guided:

```text
wp_of_body     : Body × EvalMode -> WP
row_of_body    : Body -> Row
refine         : Row × Body × EvalMode -> WP
```

### 5.7 Evaluation modes and strictness

Ash currently restricts lazy and memo modes to **pure functions**. This keeps
effect rows simple: rows describe eager effects only. But lazy/memo still matter
for WPs because they change **when bottom appears** and **which arguments must
be total**.

- An **eager** function is strict in all arguments: each argument must be
  defined for the call to be defined.
- A **lazy** function may accept a bottom argument and still produce a
  non-bottom result if the argument is never forced.
- A **memo** function forces a lazy argument at most once.

This means the WP of a function depends on the evaluation mode of each
parameter. The precondition is demand-driven:

```text
lazy_if(c, t, e) : c↓ ∧ (c=true → t↓) ∧ (c=false → e↓)
```

The row system is mode-independent for effects, but the WP generator must thread
forcing order through pure lazy/memo subterms when they are consumed by eager
effectful operations.

---

## 6. Required development effort

The Ash-native Dijkstra-monad path is much smaller than full Π-types because it
builds on existing infrastructure: the Ash monad, NOTE-033 lowering, NOTE-030
composition, and NOTE-036 discharge.

### 6.1 WP representation (ash-core / ash-typeck)

- Add an internal `Wp` datatype / AST. It need not be a user-facing type.
- Define WP transformers for builtin effect operations.
- Derive `return` and `bind` combinators from the Ash monad laws.
- Connect the `Wp` type to the existing `Predicate` AST for first-order
  assertions.

### 6.2 WP inference pass

- Walk Core IR and infer WPs for each computation.
- Compose WPs at `bind`/`LetVal`/`LetCont` boundaries.
- Refine inferred WPs with user `requires`/`ensures` contracts.
- Reuse NOTE-033's classification to decide static vs. dynamic discharge.

### 6.3 WP simplifier and built-in discharge

- Structural simplification: `wp(return x)(Q) -> Q(x)`.
- Integration with SPEC-064 for type-level and simple value-level facts.
- Emit `PredicateProofObligation` for residual obligations, reusing existing
  evidence carriers.

### 6.4 Evaluation-mode sensitivity

- Parameterize WP inference by eager/lazy/memo modes.
- Track per-parameter strictness.
- Thread forcing order for pure lazy subterms consumed by eager effectful
  operations.

### 6.5 Row/WP coherence

- Formalize `abst` and `concr` between rows and WPs.
- Ensure row inference and WP inference agree on the effect footprint.
- Add diagnostics when a row and WP are inconsistent.

### 6.6 Optional provider integration

- Extend NOTE-036's provider interface to accept `Wp`-shaped obligations.
- `by solver`, `by lean`, `by test`, and `by llm` remain optional.

### 6.7 Π-types (deferred)

Only if/when Ash wants user-written WPs or proof terms:

- Add Π-types, `Prop`, and `Eq` to the type system.
- Update semantic summaries to transport dependent signatures.
- Add erasure so proof terms do not appear in runtime rows.

### 6.8 Rough magnitude

- **Ash-native Dijkstra monads:** a single focused phase, comparable in scope to
  the contract-system work in NOTE-033 / NOTE-030. It touches `ash-core` and
  `ash-typeck` but does not require a new surface syntax or a full dependent
  type system.
- **Π-types:** a Phase 200+ sized initiative, only needed if Ash decides to
  expose WPs as user-written types.

---

## 7. Research directions

### 7.1 Ash-native WP inference

Design the internal `Wp` AST and the inference pass that derives WPs from the
Ash monad. Determine which Core IR forms need WP transformers beyond `return`,
`bind`, and primitive operations.

### 7.2 Mode-sensitive WP calculus

Formalize how eager, lazy, and memo evaluation modes change weakest
preconditions, contract timing, and the appearance of bottom. Prove that the
WP of a pure lazy subterm is demand-driven when consumed by eager effectful
operations.

### 7.3 Row/WP adjunction

Develop the Galois connection between effect rows and weakest preconditions,
including the body-guided refinement `Row × Body × EvalMode -> WP`. Prove
soundness: if the row is empty, the WP is pure; if the row is a subrow, the WP
is no stronger than the superset's WP.

### 7.4 Dijkstra monads for Ash effects

Apply the DM4Free construction to Ash's existing workflow monad. Determine
whether the derived WP semantics matches the hand-designed contract semantics
in NOTE-030. Identify any gaps for async, nondeterminism, or resource effects.

### 7.5 Optional provider integration

Extend NOTE-036's provider model so that `by solver`, `by lean`, `by test`, and
`by llm` can discharge or provide evidence for WP-shaped obligations. The
evidence row must distinguish built-in discharge, solver discharge, empirical
evidence, and deferred/dynamic outcomes.

### 7.6 Π-types as a future surface

Investigate whether, when, and how to expose WPs as user-written Π-types. This
is a much larger step and should be deferred until the internal WP calculus is
stable.

---

## 8. Alignment with Ash design philosophy

This proposal aligns with Ash's symbolic-connectionist duality (NOTE-037):

- **Symbolic side:** the Ash monad, WP inference, built-in discharge, and
  optional SMT/Lean proof providers.
- **Connectionist side:** LLMs suggest WP refinements, loop invariants, or
  predicate-function summaries, which the compiler checks symbolically.
- **Compiler as orchestrator:** validates every discharge and records evidence.

It also aligns with gradual verification (NOTE-036):

- Programs can live at different points on the precision lattice:
  row-only → row + contracts → inferred WP with built-in discharge →
  provider-dischargeable WP → deferred/dynamic WP.
- `deferred` remains a valid outcome for obligations Ash cannot discharge.

Finally, it keeps Ash's effect-first design: rows remain the runtime-relevant
abstraction, while WPs provide compile-time precision. The Ash monad is the
shared substrate.

---

## 9. References

### Internal references

- [NOTE-030: Monadic Hoare Logic for Ash Computations](NOTE-030-MONADIC-HOARE-LOGIC-FOR-ASH-COMPUTATIONS.md)
- [NOTE-031: Contract Predicate Well-Formedness and Snapshots](NOTE-031-CONTRACT-PREDICATE-WELL-FORMEDNESS-AND-SNAPSHOTS.md)
- [NOTE-033: Surface-to-Core Contract Lowering](NOTE-033-SURFACE-TO-CORE-CONTRACT-LOWERING.md)
- Implementation pointers (not specs): `crates/ash-core/src/core_ash_contract.rs`
  (`LoweredPredicate`, `PredicateClassification`, `PredicateProofObligation`,
  `ComposedContract`, `ContractDischargeStatus`) and
  `crates/ash-interp/src/predicate_evaluator.rs`.
- [NOTE-034: Contract Capability Boundary](NOTE-034-CONTRACT-CAPABILITY-BOUNDARY.md)
- [NOTE-035: Temporal and Concurrent Contracts](NOTE-035-TEMPORAL-AND-CONCURRENT-CONTRACTS.md)
- [NOTE-036: Gradual Verification and Proof Provider Architecture](NOTE-036-GRADUAL-VERIFICATION-AND-PROOF-PROVIDERS.md)
- [NOTE-037: Ash as a Symbolic-Connectionist Duality](NOTE-037-SYMBOLIC-CONNECTIONIST-DUALITY.md)
- [SPEC-064: Constraint and Proposition Layer](../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)
- [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [PLAN-194: Contract and Evidence System](../plan/PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md)
- [Verification, Prover Integration, and LLM-Driven Proving: A Literature Survey](../reference/verification-and-prover-integration-survey.md)

### External references

- C. A. R. Hoare, "An Axiomatic Basis for Computer Programming," *Communications of the ACM*, 1969.
  <https://doi.org/10.1145/363235.363259>
- N. Swamy et al., "Dependent Types and Multi-monadic Effects in F\*." POPL 2016.
  <https://www.fstar-lang.org/papers/mumon/>
- D. Ahman et al., "Dijkstra Monads for Free." POPL 2017.
  <https://fstar-lang.org/papers/dm4free/>
- N. Vazou et al., "Refinement Types for Haskell." ICFP 2014.
  <https://goto.ucsd.edu/~nvazou/icfp14/haskell-refinements-techrep.pdf>
- L. de Moura and S. Ullrich, "The Lean 4 Theorem Prover and Programming Language," CADE-28, 2021.
  <https://lean-lang.org/papers/lean4.pdf>
- E. Brady, "Idris 2: Quantitative Type Theory in Practice."
  <https://idris2.readthedocs.io/>
- U. Norell, "Dependently Typed Programming in Agda."
  <https://agda.readthedocs.io/>

---

## 10. Changelog

| Date | Change |
|---|---|
| 2026-07-07 | Initial note. Synthesizes current Ash type-level proofs, Π-type extensions, Dijkstra-monad connection, row/WP adjunction, evaluation-mode considerations, effort estimate, and research directions. |
| 2026-07-07 | Revised to center on Ash-native WP inference from the existing Ash monad, acknowledge implemented predicate lowering (NOTE-033) and monadic Hoare composition (NOTE-030), and clarify that external solvers/testers are optional evidence providers.
