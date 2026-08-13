---
id: docs.notes.043
title: Notional Existential Types, Generative Witnesses, and Schema Boundaries
kind: design-note
status: exploratory
authority: non-normative
date: 2026-08-13
tags:
  - type-system
  - semantics
  - syntax
  - effect-system
  - evidence
  - authority
  - deferred
  - orientation
---

# NOTE-043: Notional Existential Types, Generative Witnesses, and Schema Boundaries

## Status, authority, and non-interference

[SPEC-104](../spec/SPEC-104-LANGUAGE-SCOPE-FREEZE.md) controls whether a feature belongs to Ash and
in which phase. Existential types and generative type identities are absent from its P1, P2, and
P3+ sets. This note is therefore a non-normative exploration only. It does not reserve `exists`,
`pack`, `unpack`, `compose`, or any other syntax; create representation compatibility; amend a
semantic rule; or authorize implementation. Promotion first requires an explicit SPEC-104
amendment, followed by a target specification, task packet, semantic-rule coverage where
applicable, and ordinary implementation evidence.

This exploration is an add-on to, not a dependency of, the current simplification work. In
particular, it does not ask current Ash interfaces, parametric polymorphism, Ash effect
declarations, structural Ash modules, providers, Core/CPS, or the runnable Engine path to
accommodate future existential representation now.

The current boundaries remain:

- Ash interfaces and Ash effect declarations are distinct constructs. Their superficially similar
  operation signatures do not share selection, dispatch, row, handler, continuation, admission,
  or runtime semantics.
- Current Ash interfaces may contain associated type declarations and methods. Current Ash effect
  declarations contain bodyless operation signatures only; they do not contain associated type
  members.
- [SPEC-103](../spec/SPEC-103-MODULE-REALIZATION-AND-OPERATIONAL-SEMANTICS.md) gives ordinary Ash
  modules stable canonical identities and conventional structural name, import, and visibility
  behavior. Ash modules are not user-visible runtime module values, dynamic imports, or a
  first-class module-computation language.
- Provider declarations, manifest provider recipe selections, and Engine-authorized provider
  bindings are distinct. Under SPEC-104 none is a first-class Ash value.

Every Ash-like example below is explicitly **non-normative pseudo-Ash**. It illustrates questions
and possible decompositions, not accepted grammar or behavior.

The examples deliberately retain the discussion's compact `type Name = ...` notation. They do
not decide whether an outer declaration would be Ash's current transparent `alias`, a nominal
`type`, or another future form. Likewise, declaration-like members such as `fn zero() -> S` and
record-like members such as `zero: () -> S` are alternative surface sketches, not two accepted
existential member forms. Outer nominality, equation transparency, and existential witness hiding
remain separate design choices throughout this note.

## 1. Motivation and vocabulary

The motivating question is whether selected capabilities commonly supplied by SML modules,
SML signatures, and SML applicative or generative functors could eventually be composed from
smaller Ash features rather than added as a second module language. The relevant pieces are:

- rank-1 parametric polymorphism and Ash interface constraints for caller-supplied types and
  evidence;
- ordinary Ash module privacy and stable nominal identities;
- existential packages for producer-selected hidden types and first-class values that use them;
- a separately designed source of genuinely fresh nominal identity, if scoped existential
  abstraction is insufficient;
- independent control over whether a type equation is visible.

Terms in this note are deliberately qualified:

- **Ash interface declaration** means the language construct named `interface`, not a checked Ash
  module interface exported by SPEC-103.
- **Ash effect declaration** means the nominal declaration of operations, not an effect row or a
  handler.
- **SML functor** means the Standard ML module-level construct, not Haskell's `Functor` typeclass.
- **opaque nominal type** and **opaque associated-type equation** are different possible
  abstraction boundaries.
- **existential package** means a first-class package with a hidden type witness, not merely the
  logical proposition that a witness exists.
- **abstract type member** is used for a type member without a known equation. It becomes an
  associated type only where a declaration supplies a selector/projection relationship, such as
  an Ash interface implementation selected for an interface application.

## 2. A notional existential type expression

The smallest proposal treats `exists` as an ordinary type-expression constructor. It could
therefore appear wherever a type expression is permitted, including as the right-hand side of a
transparent type alias or, independently, an Ash interface associated-type binding if a future
scope decision permits it.

```ash
// Non-normative pseudo-Ash.
type SomeCounter =
    exists type S {
        fn zero() -> S;
        fn increment(S) -> S;
        fn render(S) -> String;
    };
```

Read this as: a producer chooses some type `S` and packages operations that agree on that choice.
The type does not say which representation the producer chose.

A type parameter on the left-hand side has a different direction:

```ash
// Non-normative pseudo-Ash.
type CounterFor<S> = {
    zero: () -> S,
    increment: (S) -> S,
    render: (S) -> String,
};
```

At a use of `CounterFor<S>`, `S` is supplied from outside the alias; it is not literally a
top-level `forall` binder in this syntax. In `SomeCounter`, the producer supplies the hidden
witness. This directionality is the useful universal/existential distinction:

```text
caller/external context supplies S  : parameterized type or generic callable
package producer supplies S         : existential package
```

That generality would permit an independent composition with Ash interface associated types:

```ash
// Non-normative pseudo-Ash; no current existential syntax is implied.
interface CounterFactory<F> {
    type Counter;
    make(F) -> F::Counter;
}

impl CounterFactory<IntFactory> {
    type Counter =
        exists type S {
            fn zero() -> S;
            fn increment(S) -> S;
            fn render(S) -> String;
        };

    make(factory) = ...
}
```

The associated projection `IntFactory::Counter` would be stable because it is selected by the
same coherent Ash interface application. The nested `S` would still be hidden by each package.
Whether clients can reduce the associated projection to the existential package shape is the
separate equation-transparency decision; it is not decided by `exists` itself.

### 2.1 `pack` supplies a witness

```ash
// Non-normative pseudo-Ash.
fn integer_counter() -> SomeCounter {
    pack SomeCounter [
        type S = Int,
        zero = || -> 0,
        increment = |n| -> n + 1,
        render = int::to_string,
    ]
}
```

This `pack` would establish that all package members use one witness. It does not make `Int`
visible through the package type, and it does not grant authority or install any runtime frame.

### 2.2 Block-scoped `unpack`

```ash
// Non-normative pseudo-Ash.
fn render_zero(counter: SomeCounter) -> String {
    unpack counter as [type S, zero, increment, render] {
        let value: S = increment(zero());
        render(value)
    }
}
```

The explicit `type S` binder and braces make the escape boundary visible. `unpack` introduces a
fresh, rigid, local abstract name for the package witness. It does **not** unseal an equation or
reveal that the producer used `Int`.

```ash
// Non-normative pseudo-Ash; intended to reject.
fn leak(counter: SomeCounter) -> ??? {
    unpack counter as [type S, zero, increment, render] {
        zero()
        // Reject: the result mentions S outside the scope that introduced S.
    }
}
```

A result independent of `S` may leave the block. A value mentioning `S` may leave only after an
operation hides it again, for example by repacking it into another existential package.

Two independent openings introduce distinct local abstract names even when both producers used
the same representation:

```ash
// Non-normative pseudo-Ash; intended to reject.
unpack first as [type A, a_zero, a_increment, a_render] {
    unpack second as [type B, b_zero, b_increment, b_render] {
        let value: A = a_zero();
        b_increment(value)
        // Reject: A and B are unrelated rigid local types.
    }
}
```

This is **skolem freshness at existential elimination**, not proof that each `pack` minted a new
persistent nominal identity. That distinction matters for session brands in §8.

## 3. Nested existential binders

Two already-visible schemas can be written independently:

```ash
// Non-normative pseudo-Ash.
type SomeS =
    exists type S {
        fn zero() -> S;
        fn increment(S) -> S;
        fn render(S) -> String;
    };

type SomeZ =
    exists type Z {
        fn zero() -> Z;
        fn decrement(Z) -> Z;
        fn render(Z) -> String;
    };
```

Their explicit combined result might be:

Here `Incrementing<S>` and `Decrementing<Z>` are merely shorthand for the corresponding operation
schemas already shown in `SomeS` and `SomeZ`; they are not additional current Ash declarations.

```ash
// Non-normative pseudo-Ash.
type SomeSZ =
    exists type S {
        exists type Z {
            source: Incrementing<S>;
            target: Decrementing<Z>;
            convert: (S) -> Z;
        }
    };
```

The nesting describes binder introduction and scope: `S` and `Z` are both available in the inner
schema. Although an existential is sometimes called a dependent sum in type theory, that use of
"sum" does not mean an Ash variant type. Nor does `exists S. exists Z. ...` itself mean function
implication; an actual transformation is the explicit `(S) -> Z` member.

Quantifier order and arrows express different ownership patterns:

```text
exists S. exists Z. R<S, Z>       producer hides both witnesses
for each S, exists Z. R<S, Z>     caller supplies S; producer chooses Z
exists S. (S -> exists Z. Q<S,Z>) package hides S and carries a producer of hidden Z values
```

These are not interchangeable simply because their binders look lambda-like.

## 4. Composing existing existential schemas

Repeating every member of `SomeS` and `SomeZ` loses the benefit of defining their shapes once. A
possible future operation would need to be **binder-aware**, not merely a record intersection.

```ash
// Non-normative pseudo-Ash; no syntax is reserved.
type SomeSZ =
    compose
        SomeS as [type S, source],
        SomeZ as [type Z, target]
    {
        convert: (S) -> Z;
    };
```

Such an elaboration would have to:

1. expose each existential binder only to the new type expression;
2. alpha-rename hidden binders hygienically to avoid capture;
3. rebind them under the combined existential package;
4. preserve `source` and `target` member namespaces, including their separate `zero` and `render`
   names;
5. retain every original schema constraint and opacity boundary; and
6. require the producer to supply the explicit bridge.

The compiler cannot derive `S -> Z` from `SomeS` and `SomeZ`. Composition may state that bridge as
a required member or require separately available interface evidence, but it cannot invent a
conversion. Binder-aware composition must also respect opacity: it cannot inspect or copy a
schema whose equation is deliberately hidden from the composing context.

An ordinary product remains a useful, distinct construction:

```ash
// Non-normative pseudo-Ash.
type Independent = {
    left: SomeS,
    right: SomeZ,
};
```

It retains two separately created and separately opened packages. A combined existential opens
one package and can state relationships between both hidden types. Neither construction should be
silently rewritten into the other without an explicit equivalence rule and representation plan.

## 5. Independent design dimensions

The main design lesson is to avoid making one feature imply unrelated side effects.

| Dimension | Possible choices | Must not imply |
|---|---|---|
| Quantification | externally supplied/generic; producer-supplied existential | equation opacity or generativity |
| Schema shape | named interface schema; anonymous existential schema; ordinary record | common dispatch or runtime semantics |
| Abstract type members | explicit parameters; local existential witnesses; interface associated projections | automatic visibility or fresh nominal identity |
| Equation visibility | transparent; opaque to a boundary | freshness, packaging, or authority |
| Type identity | stable/applicative; fresh/generative | representation hiding |
| Nominal/phantom marker | represented nominal; zero-runtime-data marker | runtime authorization or liveness |
| Ash module visibility | private; `pub`; explicitly re-exported | type freshness or first-class module values |

For example, a visible associated-type equation could reduce to an existential type expression,
letting clients know the package shape while the witness remains hidden. An opaque associated-type
equation would hide even that shape. Conversely, an opaque equation may hide an ordinary concrete
type without any existential package or generativity.

Likewise, resolving or importing an Ash module path remains stable and non-generative under
SPEC-103. Any future source of fresh type identity should be explicit and separate; importing an
Ash module must not acquire the generativity of applying an SML generative functor.

## 6. Interfaces, effects, and existential schemas

There is a useful visual resemblance among three kinds of declarations:

```ash
// Current-shape illustration of an Ash interface declaration.
interface StoreOps<S> {
    type Key;
    type Value;

    get(S, StoreOps<S>::Key) -> Option<StoreOps<S>::Value>;
    put(S, StoreOps<S>::Key, StoreOps<S>::Value) -> Unit;
}
```

```ash
// Current-shape illustration of an Ash effect declaration.
// Current effects have parameters and bodyless operations, not associated type members.
effect Store<Route, Key, Value> {
    fn get(key: Key) -> Option<Value>;
    fn put(key: Key, value: Value) -> Unit;
}
```

```ash
// Non-normative pseudo-Ash existential schema.
type SomeStore =
    exists type State {
        type Key;
        type Value;

        get(State, Key) -> Option<Value>;
        put(State, Key, Value) -> Unit;
    };
```

In the third illustration, `Key` and `Value` are additional hidden witnesses, approximately
`exists State. exists Key. exists Value. ...`; they are better called abstract type members unless
the future design gives them a selector and projection relationship. A conforming `pack` would
have to supply all three witnesses consistently. This is a possible anonymous existential schema,
not a current extension of Ash interface-associated-type syntax.

Each visually groups type information with typed operation signatures. That resemblance might
justify reusing parser, kind-checking, diagnostic, or IR concepts in a future design. It does not
establish a current shared schema substrate or make the constructs semantically interchangeable.

| Construct | Type choice/equation owner | Meaning of an operation |
|---|---|---|
| Ash interface declaration | coherent selected Ash `impl` | ordinary method available through checked interface evidence |
| Ash effect declaration | static nominal effect application and its explicit parameters | operation request present in a computation row and interpreted by handler/provider rules |
| Notional existential schema | package producer at `pack` | first-class package member/evidence used only after scoped `unpack` |

The first example uses the current associated-type/projection idea illustratively. [SPEC-035](../spec/SPEC-035-ASSOCIATED-TYPES.md)
owns the current interface-associated surface and simple selected-implementation compatibility
rule; [SPEC-058](../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
owns canonical projection identity and rigid IR plumbing only. Neither spec authorizes existential
syntax or general normalization.

The implementation-backed
[interface reference](../reference/language/types/generics-kinds-interfaces-and-impls.md) reports
the current interface path as partial, tested, and below specification. It does not establish a
general runtime dictionary or arbitrary method-dispatch model. Therefore an example that packages
"interface evidence" is conceptual, not a claim that Ash can currently store such a dictionary as
a value.

If future existential syntax reused a named Ash interface declaration as its schema, the concise
shape might be:

```ash
// Non-normative pseudo-Ash.
type SomeStore = exists type S where StoreOps<S>;
```

Packing would have to preserve coherent associated projections chosen by the selected
implementation. An explicit `using`-like evidence choice could select among permitted evidence;
it could not override `StoreOps<S>::Key` or `StoreOps<S>::Value` independently and thereby break
the implementation's equations.

### 6.1 Why future effect type members are a separate question

A future proposal might ask whether Ash effect declarations should admit abstract type members
because the declaration shape resembles an Ash interface. That is not current Ash. It would be
sound only if each member equation were fixed by the **static nominal effect application**, never
by whichever handler happens to receive an operation at runtime. Otherwise one operation
occurrence could change input or result type when dynamic handler selection changes.

Current explicit effect parameters already provide the straightforward safe analogue:

```ash
// Current-shape illustration, not a proposal for associated effect members.
effect Store<Route, Key, Value> {
    fn get(key: Key) -> Option<Value>;
}
```

Every handler or provider for `Store<Memory, String, Bytes>` must respect those statically visible
arguments. A future `Store<Memory>::Key` projection would need an equally stable equation, but no
such effect-member feature is selected here.

## 7. Phantom markers: currency and typestate

Existentials are not the best tool when clients must retain and compare a known type identity.
Ordinary phantom parameters illustrate this distinction.

### 7.1 Currency-safe arithmetic

```ash
// Non-normative pseudo-Ash; current phantom/newtype support is not claimed.
type USD;
type EUR;

type Money<Currency> = private Money(Int);

fn add<C>(left: Money<C>, right: Money<C>) -> Money<C> { ... }
fn subtract<C>(left: Money<C>, right: Money<C>) -> Money<C> { ... }
```

`Money<USD>` and `Money<EUR>` are distinct, while ordinary arithmetic preserves the externally
supplied marker `C`. Minor integer units avoid making floating-point rounding part of this example.

Conversion is a separate, explicit concern:

```ash
// Non-normative pseudo-Ash.
type ExchangeRate<From, To> = private ExchangeRate({
    numerator: Int,
    denominator: Int,
});

fn convert<From, To>(
    amount: Money<From>,
    rate: ExchangeRate<From, To>,
) -> Money<To> { ... }
```

An Ash interface associated type could separately express a functional relationship such as
`Account<A>::Currency`, while an Ash interface constraint could supply display metadata. Neither
is required for the central same-marker arithmetic invariant.

An existential package such as `exists type C { amount: Money<C>; ... }` becomes useful only when
the currency is discovered dynamically and should be hidden. Two independently unpacked amounts
cannot be added merely because runtime strings both say `USD`; recovering static equality would
require a checked type-equality witness or an explicit repack/conversion design.

### 7.2 Construction history and typestate

```ash
// Non-normative pseudo-Ash.
type Raw;
type Validated;
type Sent;

type Request<Session, State> = private Request(RequestData);

fn validate<S>(request: Request<S, Raw>)
    -> Result<Request<S, Validated>, ValidationError> { ... }

fn send<S>(handle: SessionHandle<S>, request: Request<S, Validated>)
    -> Request<S, Sent> { ... }
```

The state marker records which typed constructor path was taken. The `Session` marker records an
origin. Private constructors are essential: if arbitrary code can construct
`Request<S, Validated>`, the marker proves no construction history.

These markers are compile-time evidence only. They do not prove a remote session is still live,
that credentials remain valid, or that Engine admission granted network authority. Runtime checks,
Ash effect rows, providers, admission, and provenance remain independent.

[NOTE-026](NOTE-026-NEWTYPE-AND-PHANTOM-TYPES.md) is useful pre-freeze comparative context for
newtypes and phantom parameters. SPEC-104 currently leaves newtypes outside P1 and P2, so these
examples do not claim present surface support.

## 8. Session-local brands and genuine generativity

The stronger requirement is that values produced during one particular runtime session cannot be
used with another. A globally named marker distinguishes a category, not individual sessions.

An existential session package can ensure consistency inside one opening:

```ash
// Non-normative pseudo-Ash.
type OpenSession =
    exists type Session {
        handle: SessionHandle<Session>;
        request: (SessionHandle<Session>, Url) -> Request<Session, Raw>;
        send: (SessionHandle<Session>, Request<Session, Validated>) -> Response;
    };

unpack open_session(config) as [type S, handle, request, send] {
    // All values using S remain under one visible scope.
}
```

This guarantees only what existential elimination guarantees: the hidden witness is treated as a
rigid abstract type within that `unpack` scope. If the producer simply wrote `type Session = Int`
in every ordinary `pack`, the language has not thereby created a new runtime-indexed static
nominal stamp. Re-opening the same stored package also creates a fresh local skolem unless a rule
preserves and exposes the package's path-dependent identity; independent skolems express lack of
known equality, not necessarily distinct underlying nominal witnesses.

True "one fresh brand per session creation" needs an additional sound mechanism, for example:

- a statically delimited generative binder whose identity cannot depend on unrestricted runtime
  control flow;
- a rank-2/scoped generator that supplies a fresh brand to a callback whose result cannot mention
  it; or
- an existential API discipline that keeps all same-session operations and branded values under
  one `unpack` and never claims a persistent per-call nominal identity outside it.

Choosing among these requires explicit rules for evaluation, identity equality, storage,
serialization, separate compilation, and escape. It cannot be obtained by treating ordinary
`pack` syntax as magic generativity.

## 9. Inspiration from SML modules and compilation analogies

SML generative functors inspire the desired capability: applying an SML functor that produces an
abstract type can yield identities that values from separate applications cannot mix. SML
applicative functors demonstrate stable result identities determined by module arguments. These
capabilities arise within the SML module calculus of SML structures, signatures, ascription, and
functors.

Ash need not import that calculus. A possible decomposition is:

```text
SML signature-like shape       -> Ash interface or a notional existential schema, by intent
SML applicative computation    -> ordinary Ash polymorphism and stable type constructors
SML opaque ascription          -> an independent equation-visibility mechanism
SML structure packaging        -> notional existential pack/unpack
SML generative type component  -> a separately specified fresh-identity mechanism
Ash module naming              -> SPEC-103 stable structural modules, unchanged
```

This mapping is motivational, not an equivalence proof. In particular, SML functor application is
a module-elaboration event, while an ordinary Ash function call is runtime computation. Confusing
the two is exactly the static-freshness problem in §8.

Existential packages also resemble closure conversion:

```text
exists Environment. { environment: Environment, call: (Environment, A) -> B }
```

The hidden environment and its call operation form the familiar closure representation. If the
complete set of closures is closed, Reynolds-style defunctionalization may replace stored code
with an algebraic tag and a global `apply` dispatcher. Ash interface evidence and Ash handler
clauses can have related dictionary/closure representations, and CPS continuations can be
defunctionalized into frames. These are compiler analogies only: they neither define existential
source semantics nor collapse interface dispatch, effect handling, and package elimination.

## 10. Evidence, effects, providers, and authority

No existential operation should manufacture evidence or authority:

- `pack` can store values and, in a future design, checked interface evidence. It cannot create an
  implementation that violates coherence or independently override associated projections.
- Packaging an object that resembles a handler does not install a handler frame, discharge an
  effect row, change continuation multiplicity, or grant a capability.
- A provider declaration is trusted metadata, a provider recipe selection is manifest metadata,
  and a provider binding is created by the Engine after admission. None can be inserted as an
  existential "provider value" under the current model.
- Phantom and generative markers can record static admissibility or origin only to the extent
  enforced by inaccessible constructors and scoped rules. They do not authorize host operations.
- Ash module interfaces and imports preserve identities and visibility but contain no implicit
  runtime authority.

This separation is part of the design, not an implementation detail.

## 11. Soundness obligations and open questions

Any proposal promoted from this note would need to answer at least the following:

1. **Formation and kinds:** Is `exists type S { ... }` limited to witnesses of kind `Type`, and
   which type/member schemas are well formed?
2. **Escape:** What exact type-and-effect judgment prevents a local rigid witness from escaping
   `unpack`, including through closures, effects, mutable/runtime storage, processes, and public
   module summaries?
3. **Value restriction:** How do strict evaluation, generalization, and existential introduction
   interact so that packages cannot smuggle unsound polymorphic state?
4. **Freshness:** Is skolem freshness sufficient, or is genuine generative nominal identity
   required? If the latter, is generation static, rank-2/scoped, path-dependent, or something
   else?
5. **Reopening:** When should two openings of the same package be known to share a witness rather
   than merely receive unrelated local abstract names?
6. **Opacity:** Can binder-aware composition see a package schema without revealing its witness or
   crossing an opaque associated-type equation?
7. **Evidence:** Can Ash interface evidence be first-class at all, and how are coherence,
   associated projections, specialization exclusions, and separate compilation preserved?
8. **Effects:** If effect type members are ever proposed, how are their equations fixed by static
   nominal effect application and prevented from varying with handler/provider choice?
9. **Runtime representation:** Are packages dictionaries, closures, defunctionalized tags,
   specialized code, or an abstract representation with multiple lowering strategies?
10. **Complexity:** Do anonymous existential schemas and binder-aware `compose` earn their parser,
    kinding, diagnostics, public-summary, Core/CPS, and runtime costs, or should named Ash
    interface schemas and ordinary records carry most use cases?
11. **Boundaries:** Can existential packages cross application JSON, process, serialization, or
    persistence boundaries, and if so how are hidden identities represented without forging them?

Until a SPEC-104 amendment and a complete target rule answer these questions, the conservative
interpretation is simple: notional existentials preserve a future direction, while current Ash
interfaces, Ash effects, providers, and Ash modules keep their existing distinct boundaries.

## 12. Orientation and related material

Use these sources in authority order when revisiting the exploration:

1. [SPEC-104](../spec/SPEC-104-LANGUAGE-SCOPE-FREEZE.md) for scope inclusion and phase.
2. [SPEC-103](../spec/SPEC-103-MODULE-REALIZATION-AND-OPERATIONAL-SEMANTICS.md) for stable Ash
   module identity, visibility, and the no-first-class-module boundary.
3. [SPEC-035](../spec/SPEC-035-ASSOCIATED-TYPES.md) and
   [SPEC-058](../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
   for current Ash interface associated-type syntax/compatibility and canonical projection IR.
4. [NOTE-026](NOTE-026-NEWTYPE-AND-PHANTOM-TYPES.md) for exploratory pre-freeze phantom/newtype
   context, subject to SPEC-104.
5. The implementation-backed
   [generics, kinds, interfaces, and implementations reference](../reference/language/types/generics-kinds-interfaces-and-impls.md)
   for current partial, tested, below-spec implementation evidence.

Historical [NOTE-022](NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md),
[NOTE-023](NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md), and
[NOTE-025](NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md) may explain earlier design pressure,
but they do not override SPEC-104's current separation of Ash interfaces and Ash effects.
