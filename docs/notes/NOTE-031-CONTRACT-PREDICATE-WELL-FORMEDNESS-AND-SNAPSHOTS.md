# NOTE-031: Contract Predicate Well-Formedness and Snapshot Semantics

**Date:** 2026-06-29
**Status:** Living document — design direction captured; resolves NOTE-014 §13 open question 1 and NOTE-030 §9 open question 3
**Purpose:** Define the predicate language boundary for Ash contracts. Contract predicates are ordinary-looking expressions at the surface, but the compiler must classify them before proof, dynamic checking, diagnostics, and optimization can trust them. This note defines the well-formed predicate fragment, boundary-local `old(...)` snapshots, predicate-fault behavior, and the split between SMT-safe, dynamic-only, and rejected predicates.

Companion to NOTE-014 (contract systems unification), NOTE-027 (blame and subsumption), NOTE-028 (evaluation-mode timing), NOTE-029 (structured bottom), NOTE-030 (monadic Hoare composition), SPEC-096 (unified effect system), SPEC-097b (target type system), SPEC-098b (target IR), SPEC-099 (Core language), and SPEC-100 (Core type checking).

## Pre-Spec Delta

This note is pre-spec. When promoted into target specs, reconcile:

- **SPEC-096 Unified Effect System:** replace `predicate = expr` with a typed predicate grammar boundary. Preserve the convenience of expression-like syntax, but require predicate classification before lowering.
- **SPEC-097b Target Type System:** add a predicate well-formedness judgment, snapshot environment typing, and the three-way classification: `StaticPredicate`, `DynamicPredicate`, and `RejectedPredicate`.
- **SPEC-098b Target IR:** add or refine metadata for `SnapshotRef`, `PredicateFault`, predicate classification, and diagnostic redaction policy.
- **SPEC-099 Core language:** specify that dynamic predicate evaluation is pure observer code over captured boundary environments and must not introduce capability, process, workflow, or handler effects.
- **SPEC-100 Core type checking:** emit well-formedness obligations before SMT proof obligations. A predicate that is not well formed must be rejected before the prover or runtime checker sees it.

## 0. Motivation

NOTE-030 made Hoare composition depend on predicates such as:

```text
∀a. Q(a) ⇒ R(a)
∃a. Q(a) ∧ S(a, b)
old(x)
```

That shape is not enough. Ash also needs to know which predicates are legal, what they may observe, whether they may force delayed computations, and what happens when predicate evaluation itself traps.

Without this boundary, contract checking can accidentally change program behavior. A predicate that reads time, calls a capability, forces a lazy value, or observes handler dispatch is not just a logical assertion. It is a computation. Ash therefore treats contract predicates as a restricted observer language, not as arbitrary effectful Ash code.

## 1. Core decision

Contract predicates must be denotationally pure, well scoped, and classified before discharge.

```text
A contract predicate may observe the contract boundary.
It must not perform authority-requiring computation.
It must not create new program observations.
```

The checker classifies each predicate into one of three classes:

```text
StaticPredicate   -- accepted by the SMT/proof fragment
DynamicPredicate  -- pure, well formed, but checked at runtime
RejectedPredicate -- not a valid contract predicate
```

The important distinction is between **dynamic** and **rejected**. Dynamic predicates are still pure observers. They may be too rich for the initial SMT profile, but the runtime can evaluate them without changing the program's meaning. Rejected predicates would perform effects, observe unstable operational state, or require authority that a contract checker must not acquire implicitly.

## 2. Grammar impact

SPEC-096 currently says:

```ebnf
-- SPEC-096 (current)
predicate = expr ;
```

This note refines that boundary. The surface still uses expression syntax, but the parser/type checker reclassifies the expression into a contract predicate AST:

```ebnf
-- SPEC-096b / SPEC-100 target delta
predicate          = predicate_or ;
predicate_or       = predicate_and { "||" predicate_and } ;
predicate_and      = predicate_not { "&&" predicate_not } ;
predicate_not      = [ "!" ] predicate_cmp ;
predicate_cmp      = predicate_add [ cmp_op predicate_add ] ;
predicate_add      = predicate_mul { ("+" | "-") predicate_mul } ;
predicate_mul      = predicate_unary { ("*" | "/" | "%") predicate_unary } ;
predicate_unary    = literal
                   | identifier
                   | "result"
                   | "message"
                   | "old" "(" snapshot_expr ")"
                   | predicate_call
                   | field_projection
                   | tuple_projection
                   | "(" predicate ")"
                   ;

predicate_call     = predicate_function "(" [ predicate_args ] ")" ;
predicate_function = identifier | qualified_identifier ;
predicate_args     = predicate { "," predicate } ;
snapshot_expr      = identifier { "." identifier } ;
cmp_op             = "==" | "!=" | "<" | "<=" | ">" | ">=" ;
```

This grammar is intentionally conservative. It does not introduce source-level `forall` or `exists` syntax. Quantifiers used by NOTE-030 are internal proof metadata unless a later surface syntax explicitly admits them.

Rejected forms include:

```ash
requires: clock.now() > deadline          -- time observation
requires: fs.exists(path)                 -- capability call
requires: force_unsafe(x) > 0             -- explicit force in predicate
requires: handle check() with h           -- handler dispatch
requires: spawn worker() == ok            -- Proc/Workflow operation
```

The exact parser may initially parse these as expressions. The predicate well-formedness pass rejects them when they are used in contract position.

## 3. Types and judgments

### 3.1 Predicate environment

A predicate is checked under a boundary-specific environment:

```text
PredicateEnv = {
  lexical: Γ,
  boundary: BoundaryKind,
  result: Option<Type>,
  message: Option<Type>,
  snapshots: SnapshotEnv,
  allowed_predicate_fns: Set<Name>,
  redaction_policy: RedactionPolicy,
}
```

`result` is available only in `ensures` and post-boundary invariant positions. `message` is available only in channel guard positions that bind a message. `old(...)` is available only where the boundary captures a pre-state snapshot.

### 3.2 Well-formedness judgment

The core judgment is:

```text
Γp ⊢ pred ⇓ PredicateSummary
```

where:

```text
PredicateSummary = {
  ty: Bool,
  free_vars: Set<Name>,
  snapshot_refs: Set<SnapshotRef>,
  class: StaticPredicate | DynamicPredicate,
  proof_fragment: Option<ProofFragment>,
  diagnostic_shape: DiagnosticShape,
}
```

A predicate is well formed only if it has type `Bool`. Non-boolean expressions do not coerce into predicates.

```text
Γp ⊢ e : Bool
row(e) = {}
no_forces(e)
no_authority(e)
no_handler_dispatch(e)
stable_observer(e)
---------------------------------
Γp ⊢ e ⇓ PredicateSummary
```

The `row(e) = {}` condition is necessary but not sufficient. Some operations may be operationally row-empty but still unstable as predicates, such as time, randomness, pointer identity, or force-count observation. The predicate checker therefore also requires a stable-observer classification.

### 3.3 Predicate functions

A function called from a predicate must be explicitly admitted as a predicate function:

```text
pred fn sorted(xs: List<Int>) -> Bool { ... }
```

The `pred fn` marker is proposed target syntax. It means:

- the function's latent row is empty;
- the function does not force delayed values except through arguments that are already strict at the boundary;
- the function does not observe time, randomness, pointer identity, force counts, handler state, process state, or authority state;
- the function is total or has an explicit predicate-fault classification.

If Ash chooses not to add `pred fn` syntax, the same classification can be represented as compiler metadata on ordinary pure functions. The design requirement is the classification, not the spelling.

## 4. Snapshot semantics

### 4.1 Boundary-local snapshots

`old(x)` refers to the value of `x` captured at the entry of the contract boundary that owns the postcondition.

```text
old_boundary(x) = snapshot(boundary_id, x)
```

Different boundaries have different snapshot environments:

```text
outer function boundary: old_outer(x)
producer boundary:       old_m(x)
continuation boundary:   old_k(x)
```

These are not interchangeable. NOTE-030's bind rule may relate them only through exported postconditions.

### 4.2 Snapshot capture timing

The runtime or lowered Core captures snapshots before executing the body governed by the contract:

```text
requires boundary:
  evaluate/check precondition over current arguments
  no old(...) needed unless an invariant boundary defines it

ensures boundary:
  capture SnapshotEnv before body starts
  run body
  check postcondition over SnapshotEnv + result
```

For an ordinary function:

```ash
fn push(s: Stack<A>, a: A) -> Stack<A>
    ensures: result.len == old(s.len) + 1
{ ... }
```

`old(s.len)` is captured at function entry. It is not re-read after the body mutates or replaces `s`.

### 4.3 Snapshot expressions

The initial snapshot form is deliberately narrow:

```text
old(x)
old(x.field)
old(x.field.subfield)
```

Snapshot expressions are paths through values available at the boundary. They are not arbitrary computations.

Rejected:

```ash
ensures: result == old(expensive(x))       -- call inside old(...)
ensures: result == old(force_unsafe(x))    -- force inside old(...)
ensures: result == old(clock.now())        -- time inside old(...)
```

If a richer snapshot is needed, the programmer must bind it before the boundary or expose it through a pure, admitted predicate function whose evaluation timing is explicit.

### 4.4 Snapshot ownership across bind

For NOTE-030 composition:

```text
m : requires P(old_m) ensures Q(a, old_m)
k : requires R(a)      ensures S(a, b, old_k)
```

The composed postcondition can use:

```text
∃a. Q(a, old_outer) ∧ S(a, b, old_k)
```

only when the metadata records which snapshot environment each predicate refers to. Optimizers may reassociate binds only if they preserve this boundary identity.

## 5. Lazy and memo values inside predicates

Predicates must not force delayed computations implicitly.

```text
A predicate may inspect a strict value already present at the boundary.
A predicate must not force a lazy or memo value merely to decide the predicate.
```

This preserves NOTE-028's rule that contract timing follows observation boundaries. If a contract boundary explicitly owns a force, the force happens in the program, and the predicate may inspect the produced strict result after that force.

Example:

```ash
fn consume(lazy x: Int) -> Int
    requires: x > 0
{ ... }
```

The predicate `x > 0` is not checked at call time by forcing `x`. It is attached to the force boundary that observes `x`, as described in NOTE-028. At that later boundary, the predicate sees a strict `Int` result.

For memo values, a predicate check over the forced result participates in memo replay. The predicate itself must not become a second hidden force or a second hidden cache observer.

## 6. Predicate failure is not predicate false

Predicate evaluation can fail for reasons other than returning `false`:

- arithmetic trap inside a dynamic predicate;
- partial predicate function despite admission rules;
- malformed dynamic value crossing an FFI or builtin boundary;
- resource exhaustion while evaluating a dynamic predicate;
- redaction policy refusing to expose a value required for a diagnostic.

Ash distinguishes:

```text
predicate returns false  => ContractViolation(predicate_false)
predicate traps/faults   => Trap { reason: ContractPredicateFault(...) }
```

A predicate fault is not evidence that the contract condition is false. It is evidence that the checker could not evaluate the predicate safely at that boundary.

The diagnostic shape should be:

```rust
pub enum ContractCheckFailure {
    PredicateFalse(ContractDiagnostic),
    PredicateFault(PredicateFaultDiagnostic),
}
```

`PredicateFalse` follows NOTE-027 blame polarity: failed `requires` blames the caller; failed `ensures` blames the callee. `PredicateFault` normally blames the contract author or the admitted predicate function provider, because the fault is in the checking machinery or predicate definition rather than in the caller/callee relation.

## 7. Proof fragment vs dynamic fragment

### 7.1 Static fragment

The initial SMT-safe fragment should include:

- booleans and boolean connectives;
- integer and bounded numeric comparisons;
- equality over SMT-supported scalar values;
- field projection over transparent records with known encodings;
- uninterpreted predicates for admitted predicate functions when an axiom/evidence summary exists;
- boundary-local `old(path)` values as named constants.

### 7.2 Dynamic-only fragment

A predicate may be dynamic-only when it is pure and stable but not in the initial proof fragment:

- list traversal such as `sorted(xs)`;
- finite map membership checks;
- string normalization checks;
- opaque-domain equality through an admitted equality relation;
- structural predicates over values whose SMT encoding is deferred.

Dynamic-only predicates are checked at runtime. They are not rejected merely because SMT cannot prove them.

### 7.3 Rejected fragment

A predicate is rejected when it would create or observe computation outside the boundary:

- capability calls;
- process/workflow operations;
- handler installation or dispatch;
- time, randomness, environment reads, global mutable state;
- pointer identity or allocation identity unless explicitly part of an exposed value type;
- forcing lazy/memo values outside the contract-owned observation boundary;
- nontermination without an explicit dynamic predicate-fault policy.

## 8. Diagnostics and redaction

A contract diagnostic may record observed values, but observed values are policy-governed evidence.

```rust
pub enum ObservedValue {
    Full(ValueRef),
    Summary(ValueSummary),
    Redacted(RedactionReason),
    Unavailable(UnavailabilityReason),
}
```

The redaction decision is part of the diagnostic, not a reason to erase the violation. For example:

```text
requires: user.role == Admin
observed user.role = Redacted(policy = pii_role_policy)
```

The runtime can still report which predicate failed, where it failed, who is blamed, and which values were unavailable. This keeps diagnostics useful without requiring contracts to bypass secrecy, capability, or provenance policy.

## 9. Worked examples

### 9.1 Static predicate with `old(...)`

```ash
fn push(s: Stack<Int>, x: Int) -> Stack<Int>
    ensures: result.len == old(s.len) + 1
{ ... }
```

Classification:

```text
old(s.len)          => SnapshotRef(boundary = push.entry, path = s.len)
result.len          => post-state field projection
==, +               => SMT-supported integer fragment
class               => StaticPredicate
```

The checker may prove or dynamically check this predicate depending on the available model for `Stack.len`, but the predicate itself is well formed.

### 9.2 Dynamic-only pure predicate

```ash
pred fn sorted(xs: List<Int>) -> Bool { ... }

fn binary_search(xs: List<Int>, target: Int) -> Option<Int>
    requires: sorted(xs)
{ ... }
```

`sorted(xs)` may be outside the first SMT profile because it traverses a list. It remains a valid dynamic predicate if `sorted` is admitted as pure, stable, and total-or-fault-classified.

### 9.3 Rejected effectful predicate

```ash
fn read_if_exists(path: Path) -> String
    requires: fs.exists(path)
{ ... }
```

`fs.exists(path)` is a capability call. The contract checker must not acquire filesystem authority just to decide a precondition. The programmer must make the observation explicit in the program:

```ash
fn read_if_exists(path: Path, exists: Bool) -> String
    requires: exists
{ ... }
```

or return an `Act`/effectful computation that performs the check under explicit authority before calling the pure contract boundary.

### 9.4 Lazy value boundary

```ash
fn use_positive(lazy x: Int) -> Int
    requires: x > 0
{ ... }
```

At call time, `x > 0` is not a valid immediate check because evaluating it would force `x`. The contract is attached to the force boundary. Once the program explicitly forces `x`, the produced strict `Int` can be checked with the predicate `value > 0`.

### 9.5 Predicate fault

```ash
pred fn valid_ratio(n: Int, d: Int) -> Bool {
    n / d >= 0
}

fn f(n: Int, d: Int) -> Int
    requires: valid_ratio(n, d)
{ ... }
```

If `d == 0`, the dynamic predicate evaluation traps. Ash reports `ContractPredicateFault`, not `PredicateFalse`. The caller did not necessarily violate `valid_ratio`; the predicate function attempted a partial operation without guarding its domain.

## 10. Design decisions

1. **Surface predicates remain expression-like, but not arbitrary expressions.** Contract position triggers a predicate well-formedness pass.
2. **Predicates are pure observers.** They must not call capabilities, observe time/randomness, dispatch handlers, or perform Proc/Workflow operations.
3. **`old(...)` is boundary-local.** Snapshot identity is part of contract metadata and must survive bind composition and optimization.
4. **Predicates must not force delayed values implicitly.** Lazy/memo contract timing follows NOTE-028 observation boundaries.
5. **Predicate false and predicate fault are distinct.** False is a contract violation; fault is a checker/predicate-definition failure.
6. **Static and dynamic are both valid discharge modes.** Rejected predicates are neither static nor dynamic; they are invalid contract syntax after classification.
7. **Diagnostics are policy-governed.** Redaction affects observed-value payloads, not whether a violation exists.

## 11. Open questions

1. **Exact `pred fn` spelling.** Should Ash add an explicit `pred fn` marker, infer predicate eligibility for pure functions, or use an attribute on ordinary `fn` declarations?
2. **Totality threshold.** Should admitted predicate functions require a static totality proof, or may they carry a dynamic `PredicateFault` fallback?
3. **Richer snapshots.** Do we need an explicit `snapshot name = expr` boundary form for expensive or computed snapshots, instead of widening `old(...)`?
4. **Existential syntax.** NOTE-030 uses existential predicates internally. Should source Ash eventually expose existential contract syntax, or keep it as proof metadata only?
5. **Redaction proof obligations.** If a diagnostic redacts an observed value, should the runtime record evidence that the redaction policy itself was authorized?

## 12. References

### Internal references

- [NOTE-014: Contract Systems Unification](NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md)
  — source contract-system note; this note resolves its open `old(x)` snapshot question.
- [NOTE-027: Contract Blame and Subsumption](NOTE-027-CONTRACT-BLAME-AND-SUBSUMPTION.md)
  — defines blame polarity and `old(x)` behavior for interface/impl subsumption.
- [NOTE-028: Purity, Evaluation Modes, and Contract Timing](NOTE-028-PURITY-EVALUATION-MODES-AND-CONTRACT-TIMING.md)
  — defines lazy/memo contract timing and memo replay.
- [NOTE-029: Structured Bottom and Contract Diagnostics](NOTE-029-STRUCTURED-BOTTOM-AND-CONTRACT-DIAGNOSTICS.md)
  — defines `ContractDiagnostic`, observed values, structured bottom, and explicit `fail` recovery.
- [NOTE-030: Monadic Hoare Logic for Ash Computations](NOTE-030-MONADIC-HOARE-LOGIC-FOR-ASH-COMPUTATIONS.md)
  — defines bind-level contract composition and boundary-local snapshots across sequencing.
- [SPEC-096: Unified Effect System](../spec/SPEC-096-UNIFIED-EFFECT-SYSTEM.md)
  — current `predicate = expr` placeholder and contract-effect grammar.
- [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
  — target contract effects, subsumption, sequencing composition, and evaluation modes.
- [SPEC-098b: Target IR](../spec/SPEC-098b-TARGET-IR.md)
  — target `ContractDischarge`, `ContractDiagnostic`, `ComposedContract`, and evidence metadata.
- [SPEC-099: Core Language](../spec/SPEC-099-CORE-LANGUAGE.md)
  — Core dynamic contract checks and structured contract traps.
- [SPEC-100: Core Type Checking](../spec/SPEC-100-CORE-TYPE-CHECKING.md)
  — Core proof obligations and dynamic fallback behavior.

### External references

- Racket documentation, ["Contracts" in The Racket Guide](https://docs.racket-lang.org/guide/contracts.html).
  Practical contract boundaries, higher-order contracts, dependent contracts, and blame-oriented runtime errors.
- Eiffel documentation, ["Design by Contract and Assertions"](https://www.eiffel.org/doc/solutions/Design_by_Contract_and_Assertions).
  Classic precondition/postcondition/invariant framing, including contract inheritance and assertion methodology.
- LiquidHaskell documentation, ["LiquidHaskell Docs"](https://ucsd-progsys.github.io/liquidhaskell/).
  Refinement typing with logical predicates, totality checking, and law proofs through ordinary code.
- Clark Barrett, Pascal Fontaine, and Cesare Tinelli, ["The SMT-LIB Standard: Version 2.7"](https://smt-lib.org/language.shtml).
  Current SMT-LIB language reference used as the external boundary for SMT-safe predicate fragments.

## 13. Changelog

| Date | Change |
|------|--------|
| 2026-06-29 | Initial note. Defines contract predicate well-formedness, boundary-local `old(...)` snapshots, lazy/memo force boundaries, predicate-fault behavior, SMT-vs-dynamic classification, and diagnostic redaction. |
