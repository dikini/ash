# SPEC-076: Explicit Refutable Matching and Exhaustiveness

**Status:** Draft
**Date:** 2026-06-02
**Promotes:** [DESIGN-044](../design/DESIGN-044-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)
**Builds on:** [SPEC-068](SPEC-068-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md)
**Related:** [SPEC-003](SPEC-003-TYPE-SYSTEM.md), [SPEC-020](SPEC-020-ADT-TYPES.md), [SPEC-050](SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md)
**Plan:** [PLAN-126](../plan/PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)
**Implementation Tasks:** [TASK-1000](../plan/tasks/TASK-1000-explicit-refutable-matching-packet.md) through [TASK-1008](../plan/tasks/TASK-1008-runtime-defensive-pattern-error-cleanup-closeout.md)

## 1. Summary

Ash source must not contain implicit refutable matching. A pattern that can fail is valid only when the enclosing construct makes the failure path explicit.

This spec defines the type-checking contract for three pattern-use classes:

1. irrefutable binders;
2. exhaustive eliminators;
3. explicit complement/refutable constructs.

It treats `if let ... else` as a total two-branch eliminator over `P | not P`, where the `else` branch is the explicit complement path. Current selective `receive` remains an allowed refutable filtering form for now.

## 2. Motivation

Pattern matching is a control-flow boundary. If a binder accepts a pattern that does not cover all possible values of the scrutinee type, the program has hidden control flow. In Ash, hidden control flow is especially harmful because workflows rely on explicit contracts, evidence, and failure reporting.

For example, this binding is not total when `maybe : Option<Int>`:

```ash
let Some { value } = maybe;
value
```

The author must instead write an explicit eliminator:

```ash
match maybe {
  Some { value } => value,
  None => 0,
}
```

or an explicit complement eliminator:

```ash
if let Some { value } = maybe then {
  value
} else {
  0
}
```

The exact surface spelling must be verified against the live parser by TASK-1001 before implementation tests are frozen. As of this packet, the expected parser shape is `if let pattern = expr then expr else expr`.

## 3. Terms

### 3.1 Refutable pattern

A pattern is refutable for a scrutinee type when some well-typed value of that type does not match the pattern.

### 3.2 Irrefutable pattern

A pattern is irrefutable for a scrutinee type when every well-typed value of that type matches the pattern and all nested subpatterns are irrefutable for their corresponding field types.

### 3.3 Impossible pattern

A pattern is impossible for a scrutinee type when no well-typed value of that scrutinee type can match the pattern. This includes outer-shape/type mismatches such as a variant pattern against a non-ADT scrutinee, a constructor from the wrong ADT universe, and any future proven-empty refined case. Impossible patterns are hard type-checking errors. They are not merely refutable patterns.

Pattern type mismatch is the common diagnostic route for impossible patterns caused by incompatible outer type shape. If a future refinement system proves a same-universe pattern impossible, the diagnostic may use an unreachable-pattern family, but it remains fatal in this phase.

### 3.4 Constructor universe

The constructor universe is the closed set of constructors that may produce values of a canonical ADT scrutinee type. SPEC-068 owns canonicalization into this universe for ordinary runtime ADTs. Wildcard and variable patterns are universal patterns and do not require enumerating a constructor universe by themselves; constructor-specific coverage does.

### 3.5 Explicit complement eliminator

An explicit complement eliminator is a two-branch construct whose syntax and semantics define both the values that match a pattern `P` and the implicit complement `not P`. In this phase, `if let ... else` is the explicit complement eliminator. It is total because the `else` branch covers every well-typed scrutinee value not matched by `P`.

### 3.6 Explicit refutable filtering construct

An explicit refutable filtering construct is a construct whose syntax and semantics define a non-match/filtering path without promising global constructor coverage. In this phase, current selective `receive` arms remain in this category.

## 4. Pattern-use classes

### 4.1 Irrefutable binders

The type checker must reject refutable patterns in binder positions that must continue after binding.

This phase covers at least these binders after the audit resolves exact live callsites:

- surface block `let` and lowered/core `Expr::Let`;
- workflow `let`;
- `observe` result binders;
- `spawn` result binders;
- `split` result binders;
- `foreach` / loop element binders if present in the live surface;
- future function-parameter patterns if such syntax is introduced later.

Required diagnostic shape:

```text
non-irrefutable pattern in <construct>: pattern <P> does not cover <Type>; missing <Witness>; use match or if let ... else
```

The checker must include a source span for the pattern when available.

### 4.2 Exhaustive eliminators

The type checker must reject non-exhaustive arm sets in eliminators that promise to return a value or continue after selecting one branch.

This phase covers:

- `match` expressions;
- `with_error` handlers when the failure payload type has a known closed constructor universe;
- any existing total handler/dispatch form identified by the audit.

`match` already has a partial implementation through SPEC-068. This phase must preserve that implementation and improve gaps discovered by the audit, especially diagnostics and nested/product-pattern limitations that can make coverage look broader than it is.

Required diagnostic shape:

```text
non-exhaustive match on <Type>: missing <Witnesses>
```

When a `match` contains only a well-typed wildcard/variable/default arm, that arm is universally exhaustive and does not require a constructor universe. When non-wildcard arms require constructor-specific coverage and the constructor universe is unavailable because the type is neutral, rigid, unknown, or non-ADT, the checker must produce an explicit blocked/unsupported or pattern-type diagnostic instead of guessing.

### 4.3 Explicit complement/refutable constructs

The type checker may allow refutable patterns only when the construct exposes the non-match path in the source-level semantics.

`if let ... else` is a total two-branch eliminator over `P | not P`. The `else` branch is mandatory for this phase: `if let` without `else` must be rejected or remain unsupported by the parser. The checker must typecheck the pattern against the scrutinee type, bind pattern variables only in the then branch, typecheck the `else` branch under the original environment without those bindings, and unify the then/else result types. The complement is a control-flow coverage fact only: this phase does not require negative type refinement in the `else` branch.

The checker must add reachability diagnostics for degenerate complements:

- if `P` is irrefutable for the scrutinee type, the `else` branch is unreachable; the expression remains accepted, but the checker must emit a structured non-fatal unreachable-else diagnostic/warning;
- if `P` is impossible for the scrutinee type, the then branch is unreachable because the pattern is not a valid refutable pattern for the scrutinee; the checker must reject the expression with a hard pattern type/impossibility error.

Pattern bindings introduced by `P` are scoped only to the then branch. They must not be visible in the `else` branch or after the `if let` expression. If an outer variable has the same name as a pattern binding, the then branch sees the inner pattern binding and the `else` branch sees the outer binding. Duplicate binders inside a single pattern, such as `(x, x)`, are rejected in this phase rather than treated as equality constraints or shadowing.

Current selective `receive` is allowed because its arms act as filters over incoming messages. This spec does not finalize protocol-total receive semantics. The audit must document the current behavior and preserve it unless a later spec tightens the receive model.

## 5. Irrefutability algorithm

The implementation must add or select a shared type-aware irrefutability API in `ash-typeck`.

Required behavior:

1. variable and wildcard patterns are irrefutable for every type that is otherwise well-typed;
2. tuple and record patterns are irrefutable only for matching product shapes and irrefutable nested fields;
3. list fixed-prefix patterns without a rest binder are refutable for variable-length lists;
4. list prefix patterns with a rest binder are irrefutable for lists if every fixed element pattern is irrefutable for the element type and the list shape guarantees at least the fixed prefix, if Ash has such a shape; otherwise they remain refutable for ordinary variable-length lists;
5. literal patterns are refutable unless the scrutinee type is a singleton type proven by the current type system;
6. variant patterns are irrefutable only when the canonical constructor universe contains exactly that constructor and nested payload patterns are irrefutable;
7. blocked canonicalization is an error for irrefutable binder positions when a variant pattern requires a closed constructor universe; tuple and record patterns instead require a known product shape and field types.

The first slice may conservatively reject patterns that might be provably irrefutable only with future refinement types.

## 6. Error model

This phase must make matching errors structured and user-facing.

At minimum, `ash-typeck` must distinguish:

- non-irrefutable binder pattern;
- non-exhaustive eliminator;
- pattern type mismatch;
- impossible pattern;
- pattern canonicalization blocked;
- unreachable `if let` else branch from an irrefutable pattern;
- unsupported selective/total receive distinction if the audit finds ambiguous receive forms.

Errors must include:

- construct kind (`let`, `workflow let`, `observe`, `match`, `with_error`, `if let`, `receive`);
- scrutinee type, when known;
- rendered pattern or missing witness;
- source span, when available;
- one likely rewrite.

Runtime pattern errors must remain possible only for unchecked IR, host-created values, or defensive interpreter boundaries. A checked Ash source program must not normally reach expression-level let binding failures such as `EvalError::LetPatternBindFailed`, workflow execution failures such as `ExecError::PatternMatchFailed`, or match fallback failures such as `EvalError::NonExhaustiveMatch` for cases that the source type checker is responsible for rejecting or proving exhaustive. TASK-1001 must refresh these exact variant names against live code before TASK-1008 records evidence.

## 7. Cross-crate ownership

| Crate | Ownership |
|-------|-----------|
| `ash-parser` | Preserve raw pattern syntax and source spans; do not classify totality. |
| `ash-core` | Continue to carry pattern and expression/workflow nodes; add shared metadata only if the audit proves typeck diagnostics need it across crates. |
| `ash-typeck` | Own type-aware irrefutability, exhaustiveness, canonicalization-blocked diagnostics, and branch environment rules. |
| `ash-engine` | Surface type-check diagnostics through existing check APIs without reinterpreting matching semantics. |
| `ash-interp` | Keep defensive runtime pattern errors, but remove reliance on them for checked-source binder failure. |
| `ash-cli` / `ash-lsp` | Render structured diagnostics with useful spans and fixes when the shared diagnostic path exposes them. |

## 8. Acceptance matrix

| ID | Case | Expected result | Owner |
|----|------|-----------------|-------|
| A76-1 | `let` binds a variable or wildcard from any well-typed scrutinee | accepted | TASK-1003 |
| A76-2 | `let Some { value } = maybe` where `maybe : Option<T>` | rejected as non-irrefutable, missing `None` | TASK-1003 |
| A76-3 | `let Only { value } = one_variant` where the ADT has one variant and nested fields are irrefutable | accepted | TASK-1002/TASK-1003 |
| A76-3a | `let Only { field: Some { value } } = one_variant` where `field : Option<T>` | rejected because a nested field pattern is refutable | TASK-1002/TASK-1003 |
| A76-4 | workflow `let`, `observe`, `spawn`, `split`, and loop binders reject refutable sum/list/literal patterns | rejected before runtime execution | TASK-1004 |
| A76-5 | `match` over an ADT missing a constructor | rejected with missing witness and span | TASK-1005 |
| A76-6 | `match` over blocked constructor-specific canonicalization without a universal wildcard/default | explicit blocked diagnostic, no guessed universe | TASK-1005 |
| A76-6a | binder over blocked variant canonicalization or unknown product shape | explicit blocked diagnostic before binding variables | TASK-1002/TASK-1003/TASK-1004 |
| A76-6b | `match x { _ => body }` where `x` has a non-ADT or open type but the wildcard is well-typed | accepted as universally exhaustive without constructor enumeration | TASK-1005 |
| A76-7 | `with_error` handler over a known closed payload omits a case | rejected as non-exhaustive or explicitly documented as deferred if payload typing is not yet available | TASK-1006 |
| A76-8 | `if let ... else` with a refutable pattern | accepted as a total `P | not P` eliminator, with bindings only in the then branch and branch result types unified | TASK-1007 |
| A76-8a | `if let` without `else` if the parser admits the spelling | rejected or unsupported; no implicit skipped/non-match path | TASK-1007 |
| A76-8b | `if let ... else` with an irrefutable pattern | accepted with a non-fatal unreachable-else diagnostic/warning | TASK-1007 |
| A76-8c | `if let ... else` with an impossible pattern or outer pattern type mismatch | rejected as a hard pattern type/impossibility error | TASK-1007 |
| A76-8d | `else` branch or following expression uses a name bound only by the `if let` pattern | rejected as out-of-scope; outer shadowed names remain visible in `else` | TASK-1007 |
| A76-8e | `if let ... else` then/else branch result types do not unify | rejected with branch type mismatch diagnostic | TASK-1007 |
| A76-8f | duplicate binders inside one pattern, such as `(x, x)` | rejected; no implicit equality constraint or same-pattern shadowing | TASK-1002/TASK-1007 |
| A76-8g | `if let ... else` complement branch relies on negative type refinement | no negative refinement is required or exposed in this phase; else checks under original environment | TASK-1007 |
| A76-9 | selective `receive` with non-covering arms | accepted under current selective semantics, with an audit note and tests proving no accidental total-receive tightening | TASK-1007 |
| A76-10 | checked source no longer reaches runtime binder failures such as `EvalError::LetPatternBindFailed` or workflow `ExecError::PatternMatchFailed` for covered binder cases | verified by defensive runtime tests and typecheck rejections | TASK-1008 |

## 9. Implementation tasks

See [PLAN-126](../plan/PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md).
