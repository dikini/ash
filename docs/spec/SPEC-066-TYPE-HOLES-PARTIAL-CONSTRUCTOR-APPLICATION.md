# SPEC-066: Type Holes and Partial Type-Constructor Application

**Status:** Implemented MVP
**Date:** 2026-05-14
**Promotes:** [DESIGN-037](../design/DESIGN-037-TYPE-HOLES-PARTIAL-TYPE-CONSTRUCTOR-APPLICATION.md)
**Origin:** [TASK-888](../plan/tasks/TASK-888-type-holes-and-partial-type-constructor-application-packet.md)
**Builds on:** [SPEC-058](SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md), [SPEC-060](SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md), [SPEC-064](SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)
**Related:** [SPEC-054](SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md), [SPEC-055](SPEC-055-MONAD-COMPREHENSION-SYNTAX.md), [SPEC-067](SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md)
**Plan:** [PLAN-115](../plan/PLAN-115-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md)
**Implementation Tasks:** [TASK-898](../plan/tasks/TASK-898-type-hole-audit-gate.md) through [TASK-903](../plan/tasks/TASK-903-type-hole-closeout.md)

## 1. Summary

SPEC-066 defines explicit source type holes and partial type-constructor applications. Its primary motivating example is `do:Result<_, E>`, which must elaborate to a unary computation constructor without implicit currying or type-function inversion.

## 2. Scope

In scope:

1. source `_` holes in audited type-expression positions;
2. canonical representation for holes and partial constructor applications;
3. kind/arity validation for saturated and partially applied constructors;
4. do-target elaboration for exactly one value-position hole;
5. diagnostics for unsupported, ambiguous, or wrong-kind hole usage;
6. non-interference with type-function pattern wildcards.

Out of scope:

- implicit currying of bare higher-arity constructors;
- arbitrary type lambdas in user syntax;
- solving holes by output inversion under type functions or associated families;
- do-target inference without an explicit `do:K` target;
- higher-kinded interface binders, which belong to SPEC-067.

## 3. Source Rules

The MVP recognizes `_` as a type hole only in positions enabled by the audit gate. The first required position is an explicit do target:

```ash
do:Result<_, ParseError> {
    return value
}
```

Rules:

1. exactly one value-position hole is accepted in an MVP do target;
2. the elaborated target must have effective kind `* -> *`;
3. bare `Result` is rejected with a hint to write `Result<_, E>`;
4. multiple holes are rejected until a later spec defines type lambdas/defaulting;
5. `_` in type-function patterns remains a pattern wildcard and is not this source hole.

## 4. Canonical Model

Implementations must not represent partial applications as saturated ordinary constructors with fake arguments. Required logical forms:

```rust
pub enum TypeConstructorExpr {
    ProperType(CanonicalTypeExpr),
    ConstructorHead(CanonicalTypeExpr),
    PartialApplication { head: ConstructorHeadId, args: Vec<PartialArg>, result_kind: Kind },
}

pub enum PartialArg {
    Applied(CanonicalTypeExpr),
    Hole(TypeHoleId),
}
```

The exact carrier names may differ, but the implementation must preserve hole identity, source span, expected kind, and ambiguity state.

## 5. Kinding

Kinding proceeds left to right over the constructor's declared kind.

- Applying a proper type argument consumes one `*` input.
- A hole consumes one input and records the open value parameter.
- The final result for a do target must be `* -> *` after abstracting exactly one hole.
- If the constructor's remaining kind is not unary after hole abstraction, reject.

## 6. Do-Notation Integration

SPEC-054 historically used MVP hidden dictionaries for `Act`, `Proc`, and `Workflow`, and rejected explicit type arguments. SPEC-066 changes only the target-shape elaboration boundary. Dictionary/evidence resolution for arbitrary `Monad<K>` remains SPEC-067, while Phase 133 supersedes the unqualified hidden-dictionary wording with public `std::algebra::Monad` evidence where available.

Required behavior:

- `do:Result<_, E>` may elaborate to a unary target term;
- it is still rejected if no `Monad<Result<_, E>>` evidence exists;
- diagnostics must distinguish wrong target shape from missing dictionary evidence.

## 7. Diagnostics

Required diagnostics:

- unsupported hole position;
- multiple holes in MVP position;
- ambiguous hole with no expected value slot;
- wrong constructor arity after hole elaboration;
- bare higher-arity constructor where unary target was expected;
- attempt to solve a hole by inverting a type function or associated family.

## 8. Acceptance Matrix

| ID | Case | Expected result |
|----|------|-----------------|
| H-1 | parse `Result<_, E>` in enabled do-target position | hole preserved with span |
| H-2 | bare `Result` as do target | wrong-kind diagnostic with hole hint |
| H-3 | `Result<_, E>` without Monad evidence | missing evidence diagnostic after target elaboration |
| H-4 | `Foo<_, _, E>` in MVP do target | multiple-hole diagnostic |
| H-5 | `_` in type-function pattern | remains pattern wildcard, not source hole |
| H-6 | hole under neutral type-function output | deferred/rejected without inversion |

## 9. Implementation Tasks

See [PLAN-115](../plan/PLAN-115-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md).
