---
id: docs.notes.042
title: Comprehensions, Reflection, Reification, and Multi-Shot Handlers
kind: design-note
status: exploratory
authority: non-normative
date: 2026-08-07
tags:
  - grammar
  - syntax
  - semantics
  - type-system
  - effect-system
  - core-ir
  - runtime
  - testing
  - target-state
  - deferred
---

# NOTE-042: Comprehensions, Reflection, Reification, and Multi-Shot Handlers

## Status and purpose

This is a comprehensive pre-spec design note. It does not define current Ash syntax or authorize
implementation. Its purpose is to connect the theory of monads, algebraic effects, handlers,
delimited continuations, and comprehensions to Ash's evolving direct-style language and its
production path:

```text
Surface Ash
→ checked expanded surface
→ checked Core
→ checked CPS
→ Engine admission
→ Engine execution
→ CLI/daemon terminal result
```

The note answers four questions:

1. Which comprehension semantics can be represented using effects and handlers?
2. What additional structure is required for arbitrary monads, alternatives, nondeterminism,
   nesting, applicative composition, positional zip, and reflection/reification?
3. Which parts are already resolved, deficient, contradictory, or absent in Ash specifications?
4. Which implementation gaps and mitigations separate the current compiler from that model?

The recommended direction is:

> Keep ordinary Ash sequencing direct-style; treat comprehensions as a distinct checked construct;
> elaborate them through scoped reflection/reification and explicitly selected composition
> algebras; preserve continuation multiplicity, handler nesting, rows, admission, and client parity
> through the canonical route.

This note is companion to:

- [NOTE-013](NOTE-013-AMBIENT-MONAD-AND-HANDLER-COMPOSITION-ALGEBRA.md), which develops the ambient
  continuation monad and handler-composition theory;
- [NOTE-015](NOTE-015-CURRENT-TO-TARGET-LANGUAGE-FORMS.md), which records current-to-target surface
  convergence;
- [SPEC-055](../spec/SPEC-055-MONAD-COMPREHENSION-SYNTAX.md), the bounded historical monadic
  comprehension specification;
- [SPEC-067](../spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md), the constructor-kinded/HKT
  substrate;
- [SPEC-095b](../spec/SPEC-095b-TARGET-GRAMMAR.md), the target grammar;
- [SPEC-097b](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md), the target type system;
- [SPEC-098b](../spec/SPEC-098b-TARGET-IR.md), [SPEC-099b](../spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md),
  and [SPEC-100](../spec/SPEC-100-CORE-TYPE-CHECKING.md), which own the target Core/CPS/runtime
  contract;
- [SPEC-102](../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md), which defines the bounded
  multi-shot-pure Core/CPS substrate;
- the [2026-08-07 specification-to-execution audit](../audit/2026-08-07-spec-to-execution-gap-audit.md),
  which records the present production-route gaps.

Dynamic module loading is outside this note. Dedicated role and policy language has been removed
and is not a prerequisite for comprehension semantics.

## 1. Executive conclusions

### 1.1 Ordinary `do` is not the semantic problem

Ash already sequences effectful direct-style calls:

```ash
let source = read(path)
let parsed = parse(source)
write(render(parsed))
```

The callable row records the operations that may be requested. Providers and handlers interpret
those requests. A target `do { ... }` block therefore adds little beyond block layout and explicit
final-result spelling. It should not be the semantic foundation of comprehensions.

The older carrier-selecting form is different:

```ash
do:Option {
    x <- mx
    return f(x)
}
```

That form chooses `Monad<Option>` and returns `Option<A>`. Current target Ash removed named
`do:K` towers and made `do { ... }` direct-style. Specifications that still define comprehensions
by translation to `do:K` are consequently not aligned with the target language.

### 1.2 Comprehensions are semantically substantial

A comprehension determines more than evaluation order. Depending on its qualifiers and selected
interpretation, it may:

- transform zero, one, or many values;
- stop early;
- branch and resume a continuation repeatedly;
- combine independent sources applicatively;
- zip sources positionally;
- delimit nested collections;
- construct eager collections, lazy streams, search trees, probabilities, or other carriers.

Those choices cannot all be recovered from ordinary direct syntax and cannot all be derived from a
single `Monad` interface.

### 1.3 Multi-shot continuations are necessary but not sufficient

Pure list-style nondeterminism requires a handler to invoke the same continuation more than once.
SPEC-102 provides a useful Core/CPS representation for that case. Complete source comprehensions
also require:

- a checked surface form and qualifier semantics;
- a way to choose the result algebra;
- answer-type transformation from an element computation to a carrier value;
- safe capture rules for duplicated continuations;
- lexical reification scopes and nested identity;
- lowering and admission of handlers on the canonical module route;
- distinct rules for bind, alternatives, applicative grouping, and zip;
- explicit interaction with other effects and handler order.

### 1.4 The primary prerequisite is specification reconciliation

Ash has relevant implemented MVPs, but the current documents do not form one coherent target rule.
Some behaviors are explicitly deferred; others are absent; several source, type, IR, and runtime
clauses contradict one another. Implementing broad comprehensions before reconciling those clauses
would force the compiler to invent language semantics.

## 2. Three possible design approaches

### 2.1 Approach A: retain carrier-selecting `do:K`

Under this approach, comprehensions remain a compact spelling of typed do-notation:

```ash
[f(x) | x <- xs]: List
```

becomes:

```text
bind_List(xs, x => unit_List(f(x)))
```

Advantages:

- the traditional translation is familiar;
- SPEC-054, SPEC-055, SPEC-067, and SPEC-078 already contain bounded pieces;
- pure carriers do not need Engine handler frames merely to sequence `bind`.

Disadvantages:

- it revives a target form that direct-style Ash has deliberately removed;
- it makes comprehensions depend on a second sequencing language;
- it does not explain effects-first nondeterminism or handler composition;
- guards, alternatives, zip, fairness, and scoped interpretation still require extra algebras;
- arbitrary user `Monad` impl bodies still need executable dictionary/method lowering.

This is the least disruptive historical path, but it does not fit the current language direction.

### 2.2 Approach B: define each collection comprehension directly

The compiler could give each result family a dedicated lowering:

```text
List comprehension   → nested flat_map/filter/map
Option comprehension → nested and_then/map
Zip comprehension    → zip_with
Set comprehension    → insertion loop
```

Advantages:

- concrete implementations can be efficient;
- list support can be added without generic HKT or handlers;
- diagnostics can be specific to each collection.

Disadvantages:

- every carrier becomes compiler semantics;
- the language acquires multiple unrelated lowering paths;
- user-defined interpretations are excluded or require compiler extensions;
- nesting and interaction rules are repeated;
- the approach does not exploit Ash's effect/handler model.

Concrete lowering is useful as a later optimization, but it is a poor canonical semantics.

### 2.3 Approach C: scoped reflection/reification with explicit algebras

The recommended approach gives comprehensions one checked semantic framework:

```text
comprehension qualifiers
→ scoped reflection of source carriers
→ handler-directed reification into the result carrier
```

Sequential generators consume `Bind`; guards consume `Empty`; alternatives consume `Plus`;
independent groups consume `Apply`; positional groups consume `Zip`. A list collector is one
handler/evidence selection, not the meaning of all comprehensions.

Advantages:

- aligns with Ash's ambient direct-style computation model;
- explains list bind as multi-shot continuation interpretation;
- supports user-defined carriers without making each one compiler syntax;
- makes nesting and handler order explicit;
- preserves separate algebraic requirements rather than pretending all composition is monadic;
- permits specialized direct lowering after semantic equivalence is established.

Costs:

- needs a normative reflection/reification contract;
- requires the multi-shot and deep-handler specs to agree;
- needs constructor-kinded evidence and executable interface methods end to end;
- must prevent opaque carriers from hiding Ash operation requirements;
- needs a clear answer-type and prompt/instance model.

The remainder of this note develops Approach C.

## 3. Theory: monadic comprehensions

### 3.1 The basic translation

For a monad `M`, a sequential comprehension:

```text
[f(x, y) | x <- mx, y <- next(x)] : M
```

has the standard translation:

```text
bind_M(mx, λx.
    bind_M(next(x), λy.
        pure_M(f(x, y))))
```

The qualifier order is significant. The second source may depend on the first value. This is the
essential monadic case.

The laws normally expected of the evidence are:

```text
bind(pure(a), f)          = f(a)
bind(m, pure)             = m
bind(bind(m, f), g)       = bind(m, λx. bind(f(x), g))
```

The compiler may call `bind` without proving these laws. It may not reassociate, fuse, parallelize,
or otherwise use the laws as optimization authorization unless the language defines acceptable law
evidence.

### 3.2 Ambient sequencing and carrier sequencing differ

Ash's ambient computation has the CPS shape:

```text
Comp<row, A> ≅ (A -> Ans) -> Ans
```

with rows describing possible requirements. Ordinary direct-style `let` composes this ambient
computation. It does not construct `List<A>`, `Option<A>`, or an arbitrary `M<A>`.

Carrier sequencing instead operates on first-class values:

```text
bind_M : M<A> -> (A -> M<B>) -> M<B>
```

Reflection/reification bridges those levels: an `M<A>` value is temporarily viewed as a source of
an `A` inside a delimited ambient computation, then the ambient computation is folded back into
`M<B>`.

## 4. Theory: reflection and reification

### 4.1 Core equations

For a selected monad `M`, introduce a scoped operation conceptually equivalent to:

```text
reflect[p, M, A] : M<A> -> {Reflect<p, M>} A
```

and a delimiter:

```text
reify[p, M, A] : (() -> {Reflect<p, M>} A) -> M<A>
```

`p` denotes a fresh lexical prompt or handler-instance identity. The semantic equations are:

```text
reify_M { yield a }
    = pure_M(a)

reify_M {
    let x = reflect_M(ma)
    rest(x)
}
    = bind_M(ma, λx. reify_M { rest(x) })
```

In handler notation:

```text
done(a)
    → pure_M(a)

reflect(ma, resume)
    → bind_M(ma, λa. resume(a))
```

For `List`, `bind` calls the callback for every element. The handler therefore resumes the captured
continuation repeatedly. For `Option`, `bind(None, ...)` does not resume. For `Result`, `Err` does
not resume. The handler's resume strategy realizes the carrier's sequencing behavior.

### 4.2 Relation to delimited continuations

An effect operation captures the continuation up to its handler delimiter. The handler receives
that continuation as `resume`. Reification is therefore an instance of delimited control:

```text
reflect  ≈ capture the current delimited continuation
reify    ≈ establish the delimiter and interpret captures
```

Filinski's monadic reflection result explains why a continuation substrate plus suitable
reflection/reification can represent arbitrary monads. Translating that result into a typed
language requires more than raw control operators: kinding, polymorphic operations, answer types,
multiplicity, lexical prompts, and effect transparency must all be explicit.

### 4.3 Algebraic and non-algebraic behavior

Ordinary first-order algebraic operations are not the whole story. Operations such as:

```text
local(environment, computation)
catch(computation, handler)
bracket(acquire, use, release)
transaction(computation)
timeout(duration, computation)
```

take computations as arguments or delimit their behavior. They are scoped or higher-order
operations. Reflection/reification can model their monadic carriers in a sufficiently expressive
delimited-control system, but a first-order `Raise` signature alone does not specify their surface
typing, resource discipline, or operational laws.

Therefore “any monad can be represented” is a theoretical expressiveness statement, not proof that
Ash's current first-order effect surface already supports every useful monadic interface.

## 5. Comprehension algebras

### 5.1 Sequential dependent bind

```text
x <- mx
y <- next(x)
yield f(x, y)
```

requires:

```text
Pure<M>
Bind<M>
```

or a conventional `Monad<M>` containing both.

The second generator may depend on the first. Evaluation is ordered and cannot generally be
parallelized.

### 5.2 Pure lexical binding

```text
let y = f(x)
```

is an ordinary lexical binding. It does not consume `Bind` evidence and should remain distinct in
the checked qualifier plan.

### 5.3 Guard and empty

```text
guard predicate
```

requires an empty computation:

```text
Empty<M>:
    empty<A>() -> M<A>
```

Its translation is:

```text
if predicate then pure(()) else empty()
```

`Monad` alone does not provide `empty`. A bare Boolean qualifier must therefore either request
`Empty<M>` or be rejected. It must not silently turn operational `fail` into a domain-level empty
result.

### 5.4 Alternative choice

A source-level alternative requires:

```text
Plus<M>:
    plus<A>(M<A>, M<A>) -> M<A>
```

often paired with `Empty<M>` as `Alternative<M>` or `MonadPlus<M>`. The specification must choose
evaluation order, strictness, fairness, and whether the right branch is evaluated eagerly.

### 5.5 Nondeterminism

A direct nondeterminism theory may expose:

```text
choose<A>(Collection<A>) -> A
empty<A>() -> A
```

An eager ordered-list handler has equations:

```text
done(a)                 → [a]
empty(_, resume)        → []
choose(values, resume)  → concat_map(values, λx. resume(x))
```

The same theory may instead be handled as first result, a set, a search tree, a lazy stream, fair
interleaving, or weighted choice. These are different interpretations. `MonadPlus` laws do not
select one automatically.

### 5.6 Applicative-independent groups

Independent sources can use:

```text
Apply<M>:
    map2<A, B, C>(M<A>, M<B>, (A, B) -> C) -> M<C>
```

An applicative group records that later source expressions do not depend on earlier bound values.
For an eager sequential applicative, evaluation order must still be specified. A truly parallel
interpretation requires process/scheduling semantics, not merely commutative row union.

### 5.7 Positional zip

Positional zip is a separate algebra:

```text
Zip<M>:
    zip<A, B>(M<A>, M<B>) -> M<(A, B)>
```

For lists:

```text
bind/List product: [1, 2] × [10, 20] = [(1,10), (1,20), (2,10), (2,20)]
positional zip:     [1, 2] ⋈ [10, 20] = [(1,10), (2,20)]
```

The conventional list `Applicative` is commonly Cartesian, whereas a zip-list applicative is
positional. Ash must not infer positional zip from `Monad<List>` or a generic `Applicative<List>`.

The target must also decide whether zip truncates, diagnoses unequal finite lengths, pads, or
depends on carrier-specific evidence.

### 5.8 Nested comprehensions

Nested comprehension results are delimited independently:

```text
reify_list@outer {
    x = reflect@outer(xs)
    yield reify_list@inner {
        y = reflect@inner(children(x))
        yield y
    }
}
```

This produces `List<List<A>>`, not one flattened list. The inner and outer reifiers need distinct
lexical identities even when they use the same carrier and operation family.

### 5.9 Fair, lazy, and infinite comprehensions

Basic multi-shot execution naturally supports finite depth-first enumeration. Fair search over an
infinite branch also requires:

- branch suspension;
- a queue or interleaving scheduler;
- lazy/productive result construction;
- cancellation and cleanup;
- a rule for traps and branch-local failures.

These requirements should not be hidden inside the initial eager-list semantics. A future fair
stream handler can implement them over the same checked choice theory.

## 6. Proposed Ash semantic model

This section proposes a target direction. It is not frozen syntax.

### 6.1 Preserve a dedicated checked comprehension form

The expanded surface AST should preserve the distinctions the type checker needs:

```text
CheckedComprehension {
    result_strategy,
    qualifiers: [
        Bind(pattern, expression),
        Let(pattern, expression),
        Guard(predicate),
        Alternative(branches),
        ApplyGroup(bindings),
        ZipGroup(bindings),
        Nested(comprehension),
    ],
    yield_expression,
    selected_evidence,
    source_origin,
}
```

The parser need not use these exact names. The invariant is that sequential bind, independent
application, and positional zip must not collapse into one undifferentiated qualifier list.

### 6.2 Request only the algebra actually used

The checker should derive evidence requirements compositionally:

| Checked form | Evidence requirement |
|---|---|
| `yield e` | `Pure<M>` |
| sequential generator | `Bind<M>` |
| pure `let` | none beyond expression typing |
| guard | `Empty<M>` |
| explicit alternatives | `Plus<M>` |
| independent group | `Apply<M>` |
| positional group | `Zip<M>` |
| repeated handler resume | legal multi-shot continuation |

A comprehension without guards should not require `MonadPlus`. A positional zip should not select
ordinary monadic bind. Missing or ambiguous evidence must reject before Core lowering.

### 6.3 Make the reification boundary explicit in checked artifacts

Surface shorthand may use brackets, `collect`, or another spelling, but checked lowering should
know:

- the carrier constructor or concrete collector;
- the selected evidence identities;
- the lexical prompt/handler identity;
- the result element type and final carrier type;
- the qualifier strategy for every group;
- the computation row before and after local handling;
- the required continuation multiplicity;
- whether any operation remains for Engine admission.

Expected-result inference may select a carrier only when evidence is unique and the rule is
specified. Otherwise an explicit collector/reifier is preferable to heuristic inference from the
first generator.

### 6.4 Deep scoped reification

The recommended semantic default is a deep handler: resumed computation remains under the same
reifier. That is necessary for later generators in the same comprehension to be interpreted by the
same collector.

The reifier's operation identity should be lexically scoped. Nearest matching reifier wins. A fresh
instance identity prevents an inner collection from being intercepted by the outer collection.

### 6.5 Multi-shot legality

The current pure-row rule is necessary but incomplete. A safe judgment should check both the
continuation and its captured environment:

```text
resume multiplicity = MultiShotPure
closed normalized resume row = {}
every captured value is unrestricted/duplicable
every captured continuation is compatible with repeated invocation
handler/reifier is deep and reinstallable
---------------------------------------------------------------
resume may be invoked zero or more times
```

An affine resource, unique value, mutable cell, process handle, or affine continuation must not be
duplicated merely because no operation is syntactically raised after capture.

The empty row should be interpreted after local handling where appropriate. In a nested list
comprehension, the tail contains further `Reflect<List>` operations, but a deep list reifier handles
those operations. The target spec must define whether the checked residual resume row removes that
locally handled operation before the multi-shot legality test. Without that rule, nested choice is
either ill-typed or accidentally accepted by an implementation-specific row shortcut.

### 6.6 Effect transparency and admission

An opaque `M<A>` must not hide operational authority from Ash. At least one of these must be true:

1. the carrier is pure and its evidence is checked as pure;
2. the carrier type includes a latent Ash row, such as a conceptual `M<Row, A>`;
3. selected `bind`/`pure`/collector evidence carries checked callable rows that flow into the
   reified computation;
4. the carrier's runtime operations remain explicit `Raise` terms discharged by Engine admission.

Reflection must not become an escape hatch that turns an imported value into an unvalidated host
operation or provider frame. Rows remain requirements, not grants; a reifier interprets only the
operations it owns, while residual requirements continue to admission.

### 6.7 Handler order determines interaction

Rows are unordered requirement collections. Handler nesting determines operational priority.

For example:

```text
State outside Nondeterminism
```

may share or preserve state across branches, whereas:

```text
Nondeterminism outside State
```

may give each branch independent state. Similar differences arise with failure, trace, memoization,
resources, and cancellation.

Ash should not invent a universal commutation rule. Source/checked artifacts should preserve
lexical handler order. Optimizers may reorder only with applicable algebraic and observational
equivalence evidence.

### 6.8 Canonical lowering path

The intended route is:

```text
comprehension surface syntax
→ expanded, source-faithful qualifier AST
→ checked qualifier plan and selected algebra evidence
→ checked Core values, closures, Reflect operations, and reification Handle
→ checked CPS with explicit answer type, resume row, and multiplicity
→ linked closure and kind-specific admission
→ Engine handler/provider frame installation
→ shared CLI/daemon execution
```

No direct list evaluator, raw source recognizer, or CLI-specific comprehension interpreter should
be introduced. A concrete flat-map loop may replace generic handler execution only as a proven
lowering/optimization of the same checked semantics.

## 7. Specification audit

The following classification distinguishes resolved substrate from target completion.

### 7.1 Resolved or substantially specified pieces

| Area | Existing specification | What it establishes |
|---|---|---|
| Ambient row-indexed computation | NOTE-013, SPEC-096b/097b | Direct computations carry requirement rows; row extension is not a monad-transformer stack. |
| Constructor-kinded interface shape | SPEC-067 | Bounded `M : * -> *`, `M<A>`, interface/impl evidence, and coherence substrate. |
| Standard algebra names | SPEC-078 | Source-visible `Functor`, `Applicative`, and `Monad` MVP surfaces and selected evidence. |
| Monad-only historical comprehension | SPEC-055 | Explicit-target bind/let qualifiers and nested `bind`/`unit` elaboration for its bounded domain. |
| Handler answer type at surface/type level | SPEC-097b | A handler may consume a computation returning `A` and produce shared branch answer `Ans`. |
| Explicit Core/CPS multiplicity | SPEC-102 | `Affine` and legal `MultiShotPure`, empty-row validation, reusable CPS continuation values, and answer-binding calls. |
| Operation/frame dispatch | SPEC-099b | Innermost matching handler/provider lookup and deep source-handler behavior for its affine target. |

These pieces reduce the design work, but they do not compose into a current comprehension rule.

### 7.2 Explicitly deferred areas

SPEC-054, SPEC-055, SPEC-067, SPEC-078, and SPEC-102 explicitly defer some or all of:

- full List Monad/comprehension semantics;
- guards and filtering;
- `Empty`, `Alternative`, `MonadFail`, and `MonadPlus` semantics;
- applicative, zip, and parallel comprehensions;
- target inference;
- pattern binders;
- arbitrary user-defined Monad execution;
- higher-rank polymorphism and unrestricted type lambdas;
- automatic algebra-law proof/testing;
- source syntax and source-to-Core lowering for multi-shot choice;
- lazy/memo integration and fair streams.

These are not implementation bugs. They are deliberately absent from the current bounded specs and
need target decisions before implementation.

### 7.3 Contradiction: current `do` versus historical typed `do`

[SPEC-095b](../spec/SPEC-095b-TARGET-GRAMMAR.md) and
[SPEC-098c](../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md) define target `do { ... }` as direct-style
sequencing and remove named target forms. SPEC-054, SPEC-055, SPEC-067, SPEC-069, and parts of
SPEC-078 continue to define or consume `do:K` as carrier-selecting `Monad<K>` elaboration.

SPEC-095b is also internally stale: its `do_block_expr` production still admits an optional
`do_profile` even though the surrounding target prose says named profiles were removed. This is a
grammar contradiction within the target document, not only a disagreement with historical specs.

Because SPEC-055 defines comprehensions through `do:K`, its semantic owner has been removed from
the target without a replacement. The index still labels the older specs “implemented” rather than
clearly historical relative to the target read path.

Mitigation:

- mark `do:K` specifications as implemented historical/current-state substrate;
- preserve `Monad` interfaces as library abstractions;
- rewrite the target comprehension rule independently of `do:K`;
- keep target `do { ... }` only if its minor layout value justifies the syntax.

### 7.4 Missing target comprehension grammar

SPEC-095b includes `comprehension_expr` in `primary_expr` but never defines the production. The only
complete grammar is SPEC-055's older explicit-target form. SPEC-098c likewise contains no target
comprehension lowering rule.

The target grammar must define:

- result/yield placement;
- sequential bind and pure-let qualifiers;
- guard and alternative forms;
- applicative and zip grouping delimiters;
- nested comprehension boundaries;
- explicit collector/reifier selection and any inference rule;
- pattern binders and refutable-pattern failure semantics.

### 7.5 Contradiction: target kind grammar versus HKT

SPEC-067 uses constructor kinds such as `M : * -> *`. SPEC-095b currently defines only `Type`,
`Row`, and `Resource` kind atoms and no kind-arrow production. SPEC-097b does not integrate the
complete SPEC-067 surface into its target rules.

Mitigation:

- adopt one target spelling, preferably `Type -> Type` rather than maintaining both `*` and `Type`;
- define kind arrows and parenthesization in SPEC-095b;
- define constructor-variable application and evidence bounds in SPEC-097b;
- preserve constructor-kinded identities in public module summaries and checked artifacts.

### 7.6 Contradiction: multi-shot surface/type versus affine IR/runtime

SPEC-095b and SPEC-097b state that a continuation with empty residual row is multi-shot. SPEC-102
instead says that an empty row is only a legality condition: a checked Core producer must explicitly
select `MultiShotPure`, and an empty-row continuation remains affine by default. In addition:

- SPEC-098b still describes handler resume as one-shot and affine;
- SPEC-100 still types handler resume as affine and lists operational `MultiShotPure` as out of
  scope;
- SPEC-099b selects deep **affine** source handlers;
- SPEC-102's handler-chain description inherits the earlier shallow-handler behavior.

The target must reconcile these into one rule. Pure-row legality should not imply multiplicity;
checked lowering should explicitly choose `Affine` or `MultiShotPure`. Both choices need deep target
operational semantics if comprehensions are handled deeply.

### 7.7 Missing reflection/reification contract

No target spec defines `reflect`, `reify`, their equations, evidence selection, prompt identity,
answer-type behavior, or lowering. NOTE-013 explains the theory but is non-normative.

Mitigation:

- specify reflection/reification as a compiler-known checked elaboration over ordinary algebra
  evidence and handler Core, rather than immediately adding privileged user syntax;
- require a fresh lexical prompt/instance per reifier;
- define deep resume and result-carrier construction;
- preserve residual rows and selected callable evidence;
- expose surface syntax only after the checked rule is stable.

### 7.8 Answer-type transformation is partial

SPEC-097b allows handler result `Ans` to differ from the handled computation's value type. SPEC-098b
uses one fixed answer type per CPS region and says answer-type polymorphism requires explicit region
support. There is no generic rule for:

```text
reify<M, A> : computation A -> M<A>
```

A monomorphized reifier can choose `Ans = M<A>`, but the specs do not decide whether generic
reification is monomorphized before CPS, represented by a polymorphic region, or handled by a
dedicated Core construct.

Recommended mitigation: require static evidence selection and specialization before CPS in the
first generic implementation. Defer existential/runtime-selected monad dictionaries.

### 7.9 Multi-shot capture safety is incomplete

SPEC-102 checks that the resume row is a normalized closed `{}`. It does not fully connect this to
ownership and closure capture. An apparently pure continuation may capture an affine resource,
unique value, mutable reference, process handle, or affine continuation.

The target type system must add a capture-duplicability premise. Until then, surface lowering must
not produce multi-shot resumptions that capture values whose multiplicity is unknown.

### 7.10 Nested choice row semantics is missing

At the first choice in a multi-generator comprehension, the captured tail contains later choices.
If the resume row is measured before local deep handling, it contains `Choose` and fails the
`row = {}` rule. If measured after the reifier removes its own operation, it may be pure.

The specs do not state which row is written to the dynamic resume for this case. The answer must be
consistent across surface typing, Core `Handle`, CPS `HandlerClause.resume_row`, runtime validation,
and nested handler reinstatement.

### 7.11 Handler composition laws are non-normative

NOTE-013 discusses ordering, commutativity, and state/failure/nondeterminism interaction. The target
specs define frame lookup but not the algebraic equivalences under which handlers or comprehension
groups may be reordered.

Mitigation:

- make lexical handler nesting the default and preserve it exactly;
- perform no commuting optimization without explicit law evidence;
- specify a small observational equivalence for the first pure eager-list domain;
- defer effectful nondeterministic reordering and transformer-like distributive laws.

### 7.12 Zip, fairness, and laws are unresolved

No target specification chooses:

- Cartesian versus positional list application;
- zip length behavior;
- depth-first versus fair nondeterminism;
- eager versus lazy result construction;
- `Empty`/`Plus` laws;
- which monad/applicative/zip laws are checked or trusted;
- which laws authorize compiler transformations.

These need explicit decisions. They cannot be delegated to whichever stdlib method happens to be
found first.

## 8. Implementation audit

The specification gaps above are separate from the production gaps. Even the currently bounded
historical comprehension model does not survive the canonical execution route.

### 8.1 Parser and expanded surface

Current strengths:

- bracket comprehension carriers exist;
- typed-do and comprehension source origins exist in historical paths;
- parser lowering fails closed instead of inventing untyped `bind` calls.

Current deficiencies:

- the canonical legacy expression lowerer explicitly rejects comprehensions pending typed
  elaboration;
- there is no target qualifier AST for guards, alternatives, applicative groups, or zip;
- pattern binders are not part of the bounded comprehension carrier;
- current target grammar and parser behavior are not aligned around one comprehension syntax.

Mitigation: first freeze the target expanded AST independently of final surface punctuation. Parser
work should preserve every semantic distinction and leave evidence selection to type checking.

### 8.2 Type checking and evidence

Current strengths:

- constructor-kinded and partial-constructor evidence has bounded implementation;
- selected stdlib/prelude `Monad` evidence exists for historical targets;
- missing and ambiguous evidence generally fails closed;
- computation rows and handler continuation types have substantial checked carriers.

Current deficiencies:

- arbitrary user-defined Monad method bodies are explicitly outside the bounded executable path;
- parent-scoped interface and impl methods are checked but skipped by complete module callable
  lowering;
- target kind syntax and module summaries do not present one coherent HKT contract;
- no evidence families exist for target `Empty`, `Plus`, or `Zip` comprehension semantics;
- no multi-shot captured-environment duplicability judgment exists;
- target inference, patterns, and guard failure typing are unresolved.

Mitigation: introduce a checked `ComprehensionPlan` that records canonical evidence identities and
rows. Do not emit Core until every required operation has unique usable evidence and every
multi-shot capture is proved duplicable.

### 8.3 Values, patterns, and closures

The current canonical module projector is too narrow for ordinary list-like comprehensions:

- list literals lower through `Cons`/`Nil` constructors that the canonical module projector cannot
  execute;
- ADT constructor expressions do not generally reach checked module Core;
- generator pattern destructuring and general matches do not lower;
- anonymous functions and closure captures do not lower;
- higher-order calls are not generally executable;
- recursive callable linking is rejected.

Generic monadic elaboration needs a continuation closure equivalent to `A -> M<B>`. List collection
needs executable constructors or another canonical collection value representation. These are
prerequisites even if handler semantics are otherwise complete.

Mitigation: complete first-order ADT/list/pattern lowering, then closure conversion and executable
interface-method dispatch. Do not build a comprehension-only value evaluator.

### 8.4 Core and CPS

Current strengths:

- checked Core/CPS contain continuations, calls, raises, handles, rows, answer types, and explicit
  continuation multiplicity;
- SPEC-102 has direct Core/CPS tests for reusable pure continuations;
- the private evaluator can execute multi-shot continuation values in its bounded domain.

Current deficiencies:

- canonical surface lowering does not produce comprehension Core;
- no checked `Reflect`/`Reify` lowering exists;
- source handler lowering accepts only a narrow one-clause/body-shape subset;
- the target specs disagree on deep versus shallow and affine versus multi-shot behavior;
- open row tails reject in Core-to-CPS lowering;
- answer-type-polymorphic generic reification is not represented as a complete source-to-CPS rule.

Mitigation: reconcile the IR contracts first, then add hand-checked Core fixtures for one eager pure
collector before adding surface lowering. The fixtures must use the same deep frame and
multiplicity semantics intended for source.

### 8.5 Engine admission and runtime

Current strengths:

- the private CPS evaluator has handler frames and continuation invocation;
- Engine has separate row-admission machinery for a legacy application request;
- structured admission and terminal results already exist.

Current deficiencies:

- canonical linked module admission rejects every CPS closure containing `Handle` or `Raise`;
- linked module admission does not invoke the general row-discharge API;
- provider/handler installation is available only through bounded compatibility routes;
- no admitted collector/reifier frame is constructed from checked comprehension evidence;
- no fairness scheduler or lazy search result integration exists.

Mitigation: extend the one canonical linked admission seam to validate locally discharged
comprehension operations, preserve residual rows, and install only the checked reifier frame. Do not
add a source spelling recognizer for list comprehensions.

### 8.6 CLI and daemon

CLI and daemon share Engine execution for admitted programs, but canonical handler-bearing modules
do not reach that point. Comprehension parity therefore needs evidence that both clients submit the
same checked and admitted linked closure, not two independently elaborated source programs.

Mitigation: add client parity only after source, Core, CPS, and admission products are stable. A
CLI-only successful list evaluator would be a semantic fork and must be rejected.

## 9. Mitigation and staged delivery

### 9.1 Immediate documentation mitigation

Before implementation:

1. classify SPEC-054 and the `do:K` portions of SPEC-055/067/069/078 as historical bounded
   substrate relative to the direct-style target;
2. define `comprehension_expr` in SPEC-095b or explicitly remove it until its target packet lands;
3. add constructor-kind arrows to the target grammar/type system or state that generic carrier
   comprehensions are deferred;
4. reconcile SPEC-098b, SPEC-099b, SPEC-100, and SPEC-102 around deep
   `Affine`/`MultiShotPure` handlers;
5. make all remaining deferred cases explicit in the target read path.

This prevents historical “Implemented MVP” labels from being mistaken for complete target
semantics.

### 9.2 Specification packet A: minimal eager pure comprehensions

Freeze the smallest useful target:

- eager finite `List` collection;
- sequential dependent generators;
- pure `let` qualifiers;
- `yield` result;
- optional guard through explicit `Empty<List>`;
- nested comprehensions with fresh lexical reifiers;
- ordered depth-first enumeration;
- closed pure continuations only;
- no zip, fair search, external effects, or arbitrary runtime-selected monads.

This packet must still specify the full source → Core → CPS → admission → client rule. “List only”
narrows the carrier domain; it does not authorize a shortcut around checked semantics.

### 9.3 Specification packet B: generic static reification

After executable HKT/interface methods exist:

- define `Pure<M>`/`Bind<M>` or `Monad<M>` evidence selection;
- define generic `reflect`/`reify` and static specialization before CPS;
- support user-defined pure carriers whose evidence bodies lower canonically;
- preserve latent rows of evidence callables;
- add `Empty` and `Plus` independently;
- require law metadata without yet treating unproved laws as optimizer authority.

### 9.4 Specification packet C: applicative and zip groups

Add distinct syntax and checked plans for:

- independent applicative groups;
- positional zip groups;
- evaluation order and strictness;
- finite length mismatch behavior;
- effect restrictions and optional parallel interpretations.

Do not desugar zip to bind unless the selected `Zip` evidence explicitly defines that equivalence.

### 9.5 Specification packet D: fair/lazy and effectful search

Only after lazy/memo and scheduler semantics reach the canonical route:

- define fair interleaving;
- define suspended branch ownership;
- define cancellation and cleanup;
- define branch traps and failure aggregation;
- define interaction with trace, state, resources, and external operations;
- decide whether effectful repeated branches require explicit rollback/replay evidence or remain
  rejected.

### 9.6 Implementation order

The shortest implementation path consistent with the theory is:

1. reconcile target specs and semantic-rule coverage;
2. complete list/ADT values and general pattern lowering;
3. complete closures and first-class callable invocation;
4. make interface/impl method bodies executable through module lowering;
5. reconcile and verify deep multi-shot-pure Core/CPS behavior;
6. add the checked comprehension plan and eager-list lowering;
7. integrate local handler discharge with linked Engine admission;
8. add nested and guarded list cases;
9. generalize to statically selected user carrier evidence;
10. add Alternative, applicative, and zip as separate capabilities;
11. add lazy/fair interpretations later;
12. establish generated negative, mutation, and CLI/daemon parity evidence.

## 10. Failure containment while the design is incomplete

The current fail-closed behavior should be preserved:

- parsed but untyped comprehensions reject before Core;
- missing or ambiguous evidence rejects before lowering;
- illegal multi-shot captures reject statically;
- unknown or mismatched resume rows reject in CPS validation or admission;
- a linked module containing unsupported handler authority rejects rather than falling back;
- a bodyless or unavailable algebra method never receives an invented implementation;
- neither CLI nor daemon selects a direct collection evaluator.

Diagnostics should identify the actual missing layer:

```text
ComprehensionTargetMissing
ComprehensionAlgebraMissing(Bind | Empty | Plus | Apply | Zip)
ComprehensionStrategyAmbiguous
ComprehensionPatternRefutableWithoutEmpty
MultiShotContinuationNotPure
MultiShotCaptureNotDuplicable
ReificationAnswerTypeUnsupported
ReificationPromptEscapes
LinkedComprehensionHandlerNotAdmitted
```

These are proposed diagnostic concepts, not frozen names.

## 11. Conformance evidence

Every promoted comprehension rule should report separate implementation, evidence, and parity
axes. The conformance matrix should include at least:

| Family | Positive evidence | Negative/mutation evidence |
|---|---|---|
| sequential list | dependent second generator and stable order | wrong generator carrier |
| empty/guard | true and false guard | missing `Empty` evidence |
| nondeterminism | multiple and nested choices | affine or effectful resume duplicated |
| nesting | `List<List<A>>` with separate inner scope | inner operation captured by outer prompt |
| arbitrary static monad | user-defined pure carrier and impl | missing/ambiguous/bodyless evidence |
| alternatives | left/right/empty laws as declared | implicit operational `fail` conversion |
| applicative | independent group | dependent expression placed in group |
| zip | positional finite result | unspecified length mismatch |
| effect transparency | residual row reaches admission | opaque carrier hides external operation |
| clients | same admitted artifact through CLI and daemon | one client takes compatibility fallback |

For multi-shot behavior, tests must distinguish:

- one continuation invoked repeatedly;
- two independently nested captured continuations;
- captured environment isolation;
- affine second-use rejection;
- non-empty row rejection;
- non-duplicable captured value rejection;
- deep reinstallation of the same reifier;
- nearest-handler behavior under nested reifiers.

Property and law tests are useful evidence but do not replace the semantic rule. Relevant laws
include Monad identity/associativity, Empty/Plus identities, collector order, zip shape, and handler
nesting invariants. Optimizations need explicit authorization from the subset of laws they use.

## 12. Recommended decisions

The following decisions best match Ash's current evolution:

1. **Do not revive `do:K` as target syntax.** Keep direct-style sequencing and treat carrier
   comprehensions separately.
2. **Keep bracket comprehension syntax only as surface sugar.** Its checked meaning comes from an
   explicit collector/reifier strategy, not from brackets themselves.
3. **Use scoped reflection/reification as canonical semantics.** Permit specialized direct lowering
   only as a semantics-preserving implementation choice.
4. **Make algebra requirements granular.** Bind, empty, plus, apply, and zip are separate evidence
   obligations.
5. **Use deep handlers for comprehension reification.** Later generators remain under the same
   collector.
6. **Make multi-shot explicit.** Empty rows permit but do not infer it; checked producers select it.
7. **Require captured-value duplicability.** Pure rows alone are insufficient.
8. **Give each nested reifier a fresh lexical identity.** Nearest matching scope handles reflection.
9. **Freeze eager ordered depth-first List first.** Fair/lazy search is a later handler, not an
   accidental property of the first implementation.
10. **Keep positional zip distinct from monadic/applicative list product.** Syntax and evidence must
    expose the distinction.
11. **Specialize generic evidence before CPS initially.** Runtime-selected arbitrary monad
    dictionaries and answer-type polymorphism can remain deferred.
12. **Do not hide effects inside carriers.** Evidence rows and residual requirements must reach
    admission.
13. **Preserve one production route.** Source comprehensions must reach checked Core/CPS and Engine
    admission before either client executes them.

## 13. Open research and design questions

1. What surface spelling selects a collector without reintroducing typed `do:K`?
2. Can expected-result inference select `M` without making evidence resolution unstable or
   non-local?
3. Should `Reflect<M>` be a compiler-internal operation, a source-visible interface, or a dedicated
   checked Core construct that erases to ordinary `Handle`/`Raise`?
4. Is lexical nearest-handler lookup sufficient for nested reification, or must prompt identity be
   represented explicitly in rows and module summaries?
5. How is the resume row calculated after deep local handling when later generators raise the same
   reflection operation?
6. Which Ash value types satisfy unrestricted capture, and how does this interact with closure
   refinement and ownership?
7. Does generic static specialization suffice for all desired user monads, or is first-class
   existential monad evidence eventually required?
8. How are scoped/higher-order operations such as `local`, `catch`, `bracket`, and `transaction`
   represented without expanding first-order operation syntax incorrectly?
9. Which List applicative is canonical, if any: Cartesian, positional, or neither without an
   explicit wrapper?
10. What observation model distinguishes lawful handler reorderings when traces, time, resources,
    or concurrency are present?
11. Which law evidence is trusted, tested, proved, or merely declared, and which compiler
    transformations may consume each grade?
12. Can fair search be expressed through existing lazy/memo Core plus a library handler, or does it
    require an explicit scheduler/search-tree carrier?
13. How should branch-local traps, recoverable failures, cancellation, and resource cleanup compose
    in multi-shot or fair comprehensions?

## 14. Non-goals

This note does not propose:

- restoring `Act`, `Proc`, or `Workflow` carrier towers;
- restoring dedicated role or policy language;
- dynamic module/package loading;
- a direct evaluator for comprehension syntax;
- treating computation rows as runtime authority;
- inferring zip, parallelism, or fairness from row union;
- assuming algebra laws merely because an impl has the right method signatures;
- supporting effectful continuation duplication without an explicit safe semantics;
- making all theoretical monads first-class runtime values in the first implementation.

## 15. Literature and conceptual lineage

The direction builds on established relationships:

- monad comprehensions and `bind`/`pure` elaboration;
- Moggi's monadic account of computational effects;
- Filinski's representation of monads through delimited continuations and reflection/reification;
- Plotkin and Power's algebraic operations;
- Plotkin and Pretnar's handlers as interpretations of algebraic effects;
- row-polymorphic effect systems such as Koka;
- Frank-style handlers and explicit computation adjustment;
- applicative structure as distinct information about independent computation;
- algebraic sums/tensors and the nontriviality of effect composition.

NOTE-013 contains links to the principal effects, handlers, continuation, and composition
references. This note applies those ideas specifically to Ash comprehensions and records where the
current target contract is not yet sufficient.

## 16. Promotion criteria

This note is ready to be promoted into normative specifications only when the following decisions
are explicit and mutually consistent:

1. one target comprehension grammar and checked qualifier AST;
2. the status of `do:K` and historical SPEC-055 semantics;
3. constructor-kind syntax in the target grammar/type system;
4. reflection/reification types and equations;
5. evidence requirements for every qualifier family;
6. deep affine and deep multi-shot-pure handler semantics across Core, CPS, and runtime;
7. resume-row calculation after local handling;
8. captured-value duplicability rules;
9. answer-type/specialization strategy;
10. lexical prompt/instance identity;
11. initial List order, strictness, and finite-search semantics;
12. distinct applicative and positional-zip rules;
13. effect transparency and Engine admission behavior;
14. one source-to-client conformance matrix.

Until those criteria are met, implementation should remain fail-closed outside already specified
bounded compatibility behavior.
