# NOTE-038: From Ash Type-Level Proofs to Π-Types and Dijkstra Monads

**Date:** 2026-07-07
**Status:** Living document — design direction and research roadmap
**Purpose:** Synthesize the current Ash type system's proof capabilities, the
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

Ash already has a conservative type-level proposition layer (SPEC-064) that can
prove structural facts: normalized type equality, sealed-domain constructor
disjointness, interface bounds, and row subsumption. It cannot prove value-level
laws such as associativity because it lacks dependent types, quantification over
values, proof terms, and induction.

This note argues that adding **dependent function types (Π-types)** would move
Ash from a Dafny/Liquid-Haskell-style verifier toward an Idris/Agda/Lean/F\*
-style dependently typed language. The payoff is that Ash's existing
`requires`/`ensures` contracts and effect rows can be understood as
**Dijkstra-monad weakest preconditions** in disguise. With Π-types, those
contracts become first-class types, laws become proof obligations, and the
compiler can generate verification conditions generically via the **Dijkstra
monads for free** construction.

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

---

## 3. Extending Ash with Π-types

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

## 4. Relation to Hoare triples, laws, and contracts

### 4.1 Contracts as weakest preconditions

An Ash contract such as:

```ash
fn sqrt(x: Int) requires { x >= 0 } ensures { result * result <= x } -> Int { ... }
```

is already a weakest-precondition specification. It can be read as:

```text
sqrt : (x : Int) -> M Int (λpost. x >= 0 ∧ ∀(r : Int). r*r <= x → post r)
```

where `M A wp` is a Dijkstra-monad computation type.

### 4.2 Laws as Π-types

Interface laws become proof obligations attached to instances. For example:

```text
SemigroupLaw : Type =
  (T : Type) -> (op : T -> T -> T) -> Associative op -> Semigroup T op
```

The `Associative op` argument is the proof. Each instance supplies its own
proof, either by hand, by `by test`, or by `by solver`/`by lean`.

### 4.3 Triples as function types

A Hoare triple `{P} c {Q}` becomes a Π-type:

```text
(c : M A (λpost. P ∧ ∀(r : A). Q r → post r))
```

Sequential composition is function composition in the Dijkstra monad.

### 4.4 Evidence rows as outcomes

NOTE-036's evidence-outcome lattice (`verified`, `tested`, `monitored`,
`deferred`, `refuted`, `untested`) classifies how a proof obligation is
discharged. With Π-types, a `verified` outcome can carry an actual proof term;
`deferred` means the proof term is not available or not required; `tested`
means the obligation is covered by empirical evidence rather than a proof.

---

## 5. Dijkstra monads and Dijkstra monads for free

### 5.1 Dijkstra monads

A Dijkstra monad is a computation type indexed by a weakest precondition:

```text
M (A : Type) (wp : (A -> Prop) -> Prop) : Type
```

- `return a` has WP `λpost. post a`.
- `bind m f` has WP `λpost. wp_m (λa. wp_{f a} post)`.
- Subtyping on WPs gives an effect ordering.

### 5.2 Connection to Ash

Ash's existing contracts, effect rows, and evidence rows map naturally onto
Dijkstra monads:

- `requires`/`ensures` → explicit WP.
- `M A wp` → computation type in the Core IR.
- `bind` → sequential composition of workflows.
- `return` → pure value introduction.
- Row subsumption → WP implication.

### 5.3 Dijkstra monads for free

F\*'s "Dijkstra monads for free" result shows that any monad `M` can be turned
into a Dijkstra monad `DM M` by a predicate-transformer CPS translation:

```text
DM M A wp = ∀post : A -> Prop. wp post -> M (post-result A post)
```

This is directly relevant to Ash: it means Ash does not need a hand-crafted
Hoare logic for every effect. Instead, the compiler can derive a WP semantics
from the existing monadic structure of workflows. The survey §3.1 discusses this
in detail.

### 5.4 Relation to effect rows

Effect rows and WPs are not isomorphic. Rows are a **sound abstraction** of WPs:

```text
abst  : WP -> Row        -- extract the effect footprint
concr : Row -> WP        -- weakest WP consistent with the row
```

These form a Galois connection:

```text
abst(wp) ⊑ row   iff   wp ⊑ concr(row)
```

However, `concr` can be refined by the **function body**, because the body
imposes an execution order (at minimum a pre-order) on operations. The same row
`{fs, http}` can correspond to different WPs depending on whether `fs` happens
before `http` or vice versa.

So the relation is body-guided:

```text
wp_of_body     : Body × EvalMode -> WP
row_of_body    : Body -> Row
refine         : Row × Body × EvalMode -> WP
```

### 5.5 Evaluation modes and strictness

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

Adding Π-types and Dijkstra-monad support to Ash is a foundational, multi-phase
initiative. It touches every layer of the compiler.

### 6.1 Core IR

- Extend `CanonicalTypeExpr` with Π-types, dependent application, and
  type-level lambdas.
- Add a proof/proposition carrier (`Prop` universe).
- Unify term and type namespaces where values appear in types.
- Add dependent substitution and splicing.

### 6.2 Type checker

- Move from unification to conversion checking under binders.
- Add bidirectional checking for Π-introduction and elimination.
- Handle implicit arguments and dependent pattern matching.
- Integrate with existing inference metas without breaking SPEC-064's
  non-inversion rule.

### 6.3 Normalizer

- Reduce under binders and handle open neutral terms.
- Extend `DefinitionalEqualityResult` with function-extensionality and
  proof-irrelevant equality cases.

### 6.4 Surface syntax

- Syntax for dependent functions, implicit arguments, and dependent pattern
  matching.
- Distinguish type-level and value-level binders.

### 6.5 Semantic summaries

- A new summary version (V6 or later) to transport dependent signatures,
  proof evidence, and erased terms across crate boundaries.
- Revalidation rules for imported Π-types.

### 6.6 Effect system

- Reconcile Π-types with effect rows.
- Decide whether rows abstract WPs or WPs refine rows, or both via an
  adjunction.
- Add mode-sensitive WP generation.
- Handle per-parameter strictness/laziness in function types.

### 6.7 Erasure and runtime

- Separate computationally relevant terms from proof-only terms.
- Ensure proof terms do not appear in runtime effect rows or module summaries.

### 6.8 Tooling

- LSP support for holes, tactic suggestions, and proof-state display.
- Extend the MCP/prover provider architecture to translate dependent proof
  obligations.

### 6.9 Rough magnitude

This is comparable to moving from a non-dependent language to a dependently
typed one. It is a **Phase 200+ sized initiative**, likely spanning multiple
plan phases and requiring its own spec packet.

---

## 7. Research directions

### 7.1 Mode-sensitive WP calculus

Formalize how eager, lazy, and memo evaluation modes change weakest
preconditions, contract timing, and the appearance of bottom. This is a
prerequisite for sound dependent effects in Ash.

### 7.2 Row/WP adjunction

Develop the Galois connection between effect rows and weakest preconditions,
including the body-guided refinement `Row × Body × EvalMode -> WP`. Prove
soundness: if the row is empty, the WP is pure; if the row is a subrow, the WP
is no stronger than the superset's WP.

### 7.3 Dijkstra monads for Ash effects

Apply the DM4Free construction to Ash's existing workflow monad. Determine
whether the derived WP semantics matches the hand-designed contract semantics
in NOTE-030. Identify any gaps for async, nondeterminism, or resource effects.

### 7.4 Automation providers for dependent proofs

Extend NOTE-036's provider model so that `by solver`, `by lean`, and `by llm`
can synthesize or suggest Π-type proof terms, not just discharge first-order
predicates. The evidence row must record whether a proof term is kernel-checked,
SMT-dischargeable, or empirically tested.

### 7.5 Staged adoption

Investigate whether a **proof-irrelevant Π** fragment is sufficient for Ash's
near-term needs (laws, contracts, pre/post conditions) without requiring full
value-dependent types. This could give much of the benefit at lower cost.

---

## 8. Alignment with Ash design philosophy

This proposal aligns with Ash's symbolic-connectionist duality (NOTE-037):

- **Symbolic side:** Π-types, Dijkstra monads, SMT/Lean proof providers.
- **Connectionist side:** LLMs suggest proof terms, lemmas, or WP refinements.
- **Compiler as orchestrator:** validates every proof term and records evidence.

It also aligns with gradual verification (NOTE-036):

- Programs can live at different points on the precision lattice:
  row-only → row + contracts → full WP → erased WP with evidence.
- `deferred` remains a valid outcome for obligations that are not yet proved.

Finally, it keeps Ash's effect-first design: rows remain the runtime-relevant
abstraction, while WPs provide compile-time precision.

---

## 9. References

### Internal references

- [NOTE-030: Monadic Hoare Logic for Ash Computations](NOTE-030-MONADIC-HOARE-LOGIC-FOR-ASH-COMPUTATIONS.md)
- [NOTE-031: Contract Predicate Well-Formedness and Snapshots](NOTE-031-CONTRACT-PREDICATE-WELL-FORMEDNESS-AND-SNAPSHOTS.md)
- [NOTE-033: Surface-to-Core Contract Lowering](NOTE-033-SURFACE-TO-CORE-CONTRACT-LOWERING.md)
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
