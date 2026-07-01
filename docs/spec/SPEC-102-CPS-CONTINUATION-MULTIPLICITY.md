---
id: spec.ash.cps-continuation-multiplicity
title: CPS Continuation Multiplicity
kind: spec
audience: [human, agent]
authority: design
status: implemented-mvp
stability: alpha
owner: language
last_verified: 2026-06-22
verified_against:
  specs:
    - docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md
    - docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md
    - docs/spec/SPEC-098b-TARGET-IR.md
    - docs/spec/SPEC-099-CORE-LANGUAGE.md
    - docs/spec/SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md
    - docs/spec/SPEC-100-CORE-TYPE-CHECKING.md
    - docs/spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md
---

# SPEC-102: CPS Continuation Multiplicity

**Status:** Implemented MVP (Phase 164)
**Scope:** Core Ash continuation types, CPS IR continuation values, handler resume semantics, Core type checking, and Core-to-CPS lowering.
**Depends on:** SPEC-096b, SPEC-097b, SPEC-098b, SPEC-099, SPEC-099c, SPEC-100, SPEC-101.
**Amends:** SPEC-098b, SPEC-099, SPEC-099c, and SPEC-100.

## 1. Summary

Ash currently treats CPS continuations as affine: a continuation value can be invoked at most once.
SPEC-102 adds explicit continuation multiplicity so pure continuations can opt into multi-shot
invocation while effectful continuations remain affine.

The normative rule is:

```text
Cont<A, Ans, row, affine>          may be invoked at most once.
Cont<A, Ans, {}, multi-shot-pure>  may be invoked zero or more times.
Cont<A, Ans, row, multi-shot-pure> where row != {} is invalid Core.
```

The empty row is a legality condition for `multi-shot-pure`. It does not, by itself, imply
multi-shot behavior. A pure continuation remains affine unless Core explicitly marks it
`multi-shot-pure`.

## 2. Scope

This spec is about Ash Core and CPS IR only.

In scope:

1. Core continuation multiplicity semantics.
2. CPS continuation value multiplicity and runtime invocation behavior.
3. Core text parsing/serialization for continuation multiplicity using current `.core` spelling.
4. Core and CPS answer-binding continuation invocation for handlers that inspect resumed answers.
5. Core validation and type checking for legal multiplicity shapes.
6. Affine use checking and multi-shot use acceptance in handler bodies.
7. Core-to-CPS lowering that preserves multiplicity.
8. Motivational examples represented as executable Core/CPS fixtures.

Out of scope:

| Item | Reason |
|------|--------|
| Surface Ash syntax | Surface syntax in design notes is informational only and may change. |
| Surface-to-Core lowering | Correct upper-layer lowering is outside this phase. Correct Core programs are assumed if they type check. |
| New standard-library search or choice APIs | This phase proves the Core/CPS substrate only. |
| General algebraic effect surface design | User-facing effect declarations and handler syntax are not specified here. |
| Lazy/memo continuation modes | SPEC-101 remains a separate thunk mode feature; this spec only references interactions. |
| Optimizations such as persistent continuation environments or clone elision | These require later runtime/optimizer work. |

## 3. Terminology

`Affine` means at most once. An affine continuation may be discarded or invoked once. A second
invocation is a type error for well-typed Core and a runtime trap for unchecked CPS input.

`Multi-shot-pure` means zero or more invocations. The continuation body must have a closed empty
effect row. Re-entering the continuation with different arguments is semantically equivalent to
running the same pure continuation body independently for each argument.

`Pure continuation row` means the continuation row is exactly the closed empty row `{}` after Core
row normalization. Open rows, effect-group references, or rows that normalize ambiguously are not
pure enough for `multi-shot-pure`.

## 4. Core Type Amendment

SPEC-099 already carries continuation multiplicity in Core:

```text
CoreType::Cont {
  input: A,
  answer: Ans,
  row: Row,
  multiplicity: CoreMultiplicity,
}
```

For this spec, `CoreMultiplicity::Affine` is operational. The existing
`CoreMultiplicity::MultiShotPure` hook becomes an operational multiplicity with this contract:

```text
well-formed Cont<A, Ans, row, Affine>

row normalizes to {}
-------------------------------------
well-formed Cont<A, Ans, row, MultiShotPure>

row does not normalize to {}
-------------------------------------
ill-formed Cont<A, Ans, row, MultiShotPure>
```

The current Core text spelling is:

```text
(cont A Ans Row affine)
(cont A Ans {} multi-shot-pure)
```

Tasks may add a clearer internal enum name such as `MultiShot`, but they must preserve existing
`.core` text compatibility for `multi-shot-pure` unless a later spec explicitly migrates fixtures.

## 5. CPS IR Amendment

CPS continuation values must carry multiplicity:

```rust
Value::Cont {
    param: Name,
    body: Box<Term>,
    captured_env: Env,
    captured_chain: HandlerChain,
    consumed: ConsumedFlag,
    row: EffectRow,
    multiplicity: ContMultiplicity,
}
```

`ContMultiplicity` has two semantic cases:

```rust
Affine
MultiShotPure
```

`Affine` is the default for existing CPS fixtures and serde input that predates this field.

`consumed` remains meaningful only for affine continuations. Implementations may keep the field on
all `Value::Cont` values for compatibility, but invoking a `MultiShotPure` continuation must not
set or inspect `consumed` for one-shot rejection.

CPS handler clauses must also carry the multiplicity used for the dynamic resume continuation:

```rust
enum ResumeRowMetadata {
    Known(EffectRow),
    LegacyInheritFromTarget,
}

HandlerClause {
    op: EffectOp,
    params: Vec<Name>,
    resume: Name,
    resume_row: ResumeRowMetadata,
    resume_multiplicity: ContMultiplicity,
    body: Box<Term>,
    row: EffectRow,
}
```

These fields are required because the runtime, not Core lowering, constructs the dynamic
`Value::Cont` for a handled operation. Lowering must therefore write row and multiplicity onto
`HandlerClause`; the interpreter must resolve the row metadata and copy multiplicity into the
`Value::Cont` it binds as the handler resume.

`resume_row` is also required for new checked lowering. It is the declared static row of the
dynamic resume continuation, not a runtime fact that unchecked CPS may use without corroboration.
Checked Core lowering writes `ResumeRowMetadata::Known(row)` from the Core resume parameter type.

Legacy serialized handler clauses that omit `resume_row` must not deserialize as a real known
empty row. They must deserialize to an explicit compatibility state such as
`ResumeRowMetadata::LegacyInheritFromTarget`. This state is valid only with
`resume_multiplicity = Affine`; multi-shot-pure resumes require a known row. At handler dispatch,
`LegacyInheritFromTarget` derives the dynamic resume row from the resolved `Raise.resume` target
row instead of comparing against `{}`. If the target row cannot be resolved, dispatch must trap or
otherwise fail closed.

For unchecked CPS input with `ResumeRowMetadata::Known(row)`, CPS validation must ensure the known
row matches the row required by the `Raise.resume` target at the handler boundary when that target
is statically resolvable. Runtime handler dispatch must also compare the known `clause.resume_row`
with the row of the resolved `Raise.resume` target before constructing the dynamic resume. If the
target row cannot be resolved, or if it differs from the known `clause.resume_row`, dispatch must
trap or otherwise fail closed. Only after this comparison may the interpreter construct the dynamic
resume as:

```rust
Value::Cont {
    row: resolved_resume_row.clone(),
    multiplicity: clause.resume_multiplicity,
    ...
}
```

It must not use `EffectRow::default()` as the resume row except when the resolved or known row is
already the closed empty row.

To support handlers that invoke a continuation, observe its answer, and continue, CPS IR also gains
an answer-binding continuation invocation:

```rust
Term::LetCont {
    name: Name,
    param: Name,
    cont_body: Box<Term>,
    row: EffectRow,
    multiplicity: ContMultiplicity,
    body: Box<Term>,
}

Term::LetContCall {
    name: Name,
    cont: ContRef,
    arg: Atom,
    row: EffectRow,
    body: Box<Term>,
}
```

`Jump` remains the terminal continuation transfer. `LetContCall` is the non-tail form used when a
handler needs the continuation answer as a value before evaluating more handler code. Its `row`
field is the same row-accounting carrier as `Jump.row`: it records the requirements of invoking
the target continuation before evaluating `body`.

`Term::LetCont.row` and `Term::LetCont.multiplicity` are the declared source for the
`Value::Cont.row` and `Value::Cont.multiplicity` created when the runtime evaluates `LetCont`.
For checked Core lowering, these fields are already type-derived facts. For unchecked CPS input,
CPS validation must verify that the effective row of `cont_body` matches `Term::LetCont.row`
before treating a `MultiShotPure` `LetCont` as reusable. Existing serialized CPS terms that omit
those fields default to `row = {}` and `multiplicity = Affine` for backward compatibility.

## 6. Runtime Semantics

### 6.1 Affine Jump

When a CPS `Jump` targets an affine continuation:

```text
resolve cont to Value::Cont { multiplicity = Affine, consumed, ... }
consumed == false
-------------------------------------
set consumed = true; evaluate continuation body

resolve cont to Value::Cont { multiplicity = Affine, consumed, ... }
consumed == true
-------------------------------------
Trap("resume already consumed")
```

The exact trap string may remain the current one for compatibility, but new structured tests should
match a structured reason when available rather than depending only on text.

When evaluating `LetCont`, the runtime constructs:

```rust
Value::Cont {
    row: term.row.clone(),
    multiplicity: term.multiplicity,
    ...
}
```

It must not infer multiplicity from `term.row`.

### 6.2 Multi-Shot Jump

When a CPS `Jump` targets a multi-shot-pure continuation:

```text
resolve cont to Value::Cont { multiplicity = MultiShotPure, row = {}, ... }
-------------------------------------
evaluate continuation body under its captured env and captured handler chain, with param bound
```

The invocation must not mutate the continuation into a consumed state. Repeated jumps to the same
continuation value are valid.

The continuation environment and handler chain are logically captured at continuation creation.
Each invocation must observe that captured environment and chain. The initial implementation may
use existing persistent/clone semantics for `Env` and `HandlerChain`; it must not introduce shared
mutable user-visible state between invocations.

### 6.3 Answer-Binding Continuation Invocation

`LetContCall` invokes a continuation and binds the terminal answer before continuing:

```text
resolve cont to Value::Cont { input = A, answer = Ans, row, multiplicity, ... }
arg : A
term.row includes row
invoke continuation according to multiplicity rules
continuation invocation returns ans : Ans
-------------------------------------
evaluate body with name bound to ans
```

For affine continuations, `LetContCall` consumes the continuation exactly like `Jump`. For
multi-shot-pure continuations, `LetContCall` may be evaluated repeatedly against the same
continuation value.

`LetContCall` exists for Core/CPS handler bodies. It is not a surface syntax commitment.

<a id="63-runtime-validation-boundary"></a>

### 6.4 Runtime Validation Boundary

Unchecked CPS input can still build inconsistent values. Runtime must fail closed:

1. A `MultiShotPure` continuation with a non-empty declared row, or with a body whose effective
   row does not match the declared empty row, must be rejected by CPS validation or trap before
   reusable invocation.
2. A `HandlerClause` with `resume_multiplicity = MultiShotPure` and non-empty or legacy/unknown
   `resume_row` must be rejected by CPS validation or trap before the dynamic resume is
   constructed.
3. A `LetContCall` whose `row` does not include the resolved continuation row must be rejected by
   CPS validation.
4. An affine continuation invoked twice traps.
5. A multi-shot-pure continuation invoked repeatedly does not trap only because of repetition.

## 7. Handler Resume Semantics

Handler clauses bind a resume parameter. The resume parameter type decides whether the captured
resume continuation is affine or multi-shot-pure.

```text
(resume k : (cont A Ans row affine))
(resume k : (cont A Ans {} multi-shot-pure))
```

For `affine`, existing handler behavior is preserved.

For `multi-shot-pure`, the captured resume continuation:

1. captures the post-operation continuation environment;
2. captures the handler chain after the matched shallow handler is removed, matching existing
   resume-chain semantics;
3. may be jumped to multiple times by the handler body;
4. requires the resume row to normalize to `{}`.

At handler dispatch, the runtime must resolve the `Raise.resume` continuation target row before
constructing the dynamic resume. If `clause.resume_row` is known, dispatch must compare it with the
resolved target row. If `clause.resume_row` is the legacy compatibility state, dispatch must derive
the dynamic affine resume row from the resolved target row. If the target row cannot be resolved,
or if a known row differs from the target row, dispatch must trap or otherwise fail closed. This
runtime check is required even when CPS validation already checked statically resolvable cases.

A handler body may discard any resume continuation. Discarding is valid for both affine and
multi-shot-pure continuations.

## 8. Core Type Checking

SPEC-100 is amended with these checks:

1. `CoreType::Cont` well-formedness validates multiplicity/row legality.
2. `CoreMultiplicity::MultiShotPure` requires a normalized closed empty row.
3. Handler resume checking accepts both `Affine` and legal `MultiShotPure` continuation types.
4. Handler resume checking rejects `MultiShotPure` with non-empty, open, or ambiguous rows.
5. The affine-use checker continues to reject more than one invocation of an affine resume.
6. The affine-use checker does not count repeated invocations of a multi-shot-pure resume as an
   affine violation.
7. `CoreExpr::LetContCall` requires a continuation of type `Cont<A, Ans, row, multiplicity>`,
   checks the argument against `A`, binds the result name as `Ans`, and contributes `row` plus the
   body row.
8. `CoreExpr::LetContCall` consumes affine continuations for use-discipline purposes and leaves
   multi-shot-pure continuations reusable.
9. Clause result and residual-row checks are unchanged except for using the resume row carried by
   the continuation type.

This spec does not infer multi-shot-pure from row emptiness. Core producers must explicitly choose
the multiplicity.

## 9. Core-to-CPS Lowering

Core-to-CPS lowering must preserve continuation multiplicity in all generated CPS continuation
values:

1. `LetCont` lowering uses `Affine` unless the source Core continuation type or checked lowering
   fact says `MultiShotPure`.
2. `LetCont` lowering writes the checked continuation row and multiplicity into CPS
   `Term::LetCont.row` and `Term::LetCont.multiplicity`.
3. Handler resume lowering maps `CoreMultiplicity::Affine` to CPS `Affine`.
4. Handler resume lowering maps legal `CoreMultiplicity::MultiShotPure` to CPS `MultiShotPure`.
5. Handler lowering stores resume multiplicity on CPS `HandlerClause` because runtime constructs
   the dynamic resume `Value::Cont`.
6. Handler lowering stores the Core resume parameter row on CPS `HandlerClause.resume_row` as a
   known row; checked lowering must not emit the legacy compatibility state.
7. Core `LetContCall` lowers to CPS `Term::LetContCall` with the checked continuation row in
   `Term::LetContCall.row`.
8. Lowering must not infer multi-shot-pure from an empty row.
9. Lowering must not silently downgrade explicit multi-shot-pure to affine.

If lowering receives untyped Core where a handler resume type is unavailable or unsupported, it
must preserve the existing conservative affine behavior.

## 10. Motivational Examples

The examples below are requirements for test coverage, not surface syntax commitments. Test agents
must encode them as Core `.core` fixtures using the Phase 164 Core continuation-invocation form
and may add direct CPS IR tests for lower-level runtime coverage.

### 10.1 Choice: All Outcomes

A pure `choice : Unit -> Bool` operation can be handled by invoking the resume continuation twice:
once with `true` and once with `false`. A Core/CPS fixture should prove the handler can combine both
branches without a second-resume trap when the resume type is:

```text
(cont Bool Ans {} multi-shot-pure)
```

The same fixture with `affine` must still reject or trap on the second invocation.

### 10.2 Backtracking: Find First

A pure choose-like operation can try candidates in order. The test fixture should model a handler
that invokes a multi-shot resume for at least two candidate values and returns the first branch that
meets a pure predicate. This proves that multi-shot resume is not only "invoke twice and collect",
but also supports discard of later branches after a successful result.

### 10.3 Nested Choice

Nested pure choices should produce independent continuation invocations. A fixture should model two
captured multi-shot resumes and verify four logical paths or the Core equivalent available in the
current syntax.

### 10.4 Discarded Resume

A handler may discard a multi-shot-pure continuation and return directly. This must remain valid and
must not be reported as an affine-use error.

## 11. Lazy/Memo Interaction Commentary

SPEC-101 lazy and memo thunks remain separate from continuation multiplicity. A future phase may
combine the features to build streams, cached searches, or demand-driven nondeterminism. This spec
does not require such integration.

The important compatibility point is row-based: if a thunk force or mutable state effect appears in
the continuation row, that row is not `{}` and the continuation is not legal as multi-shot-pure.

## 12. Mutual Recursion Commentary

Multi-shot continuation examples often pair naturally with recursive handlers. SPEC-099c already
documents tuple-of-lambdas mutual recursion and scoped `LetRec` behavior. Implementers should use
that existing substrate for recursive Core/CPS fixtures and must not add a new recursion mechanism
as part of this spec.

## 13. Conformance

An implementation conforms to SPEC-102 when:

1. Core accepts legal `multi-shot-pure` continuation types with empty rows.
2. Core rejects `multi-shot-pure` continuation types with non-empty, open, or ambiguous rows.
3. Core handler type checking permits repeated use of legal multi-shot resumes.
4. Core handler type checking continues to reject repeated use of affine resumes.
5. CPS runtime permits repeated jumps to multi-shot-pure continuations.
6. CPS runtime continues to trap repeated jumps to affine continuations.
7. Core-to-CPS lowering preserves multiplicity.
8. Motivational examples are represented by executable Core/CPS tests using current syntax.
