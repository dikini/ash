# DESIGN-044: Explicit Refutable Matching and Exhaustiveness

**Status:** Promoted to [SPEC-076](../spec/SPEC-076-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md) / [PLAN-126](../plan/PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)
**Date:** 2026-06-02
**Origin:** User report that `let` currently permits non-exhaustive pattern matching on sum types
**Builds on:** [DESIGN-039](DESIGN-039-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md), [SPEC-068](../spec/SPEC-068-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md)
**Related:** [SPEC-003](../spec/SPEC-003-TYPE-SYSTEM.md), [SPEC-020](../spec/SPEC-020-ADT-TYPES.md), [SPEC-050](../spec/SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md)

## Summary

Ash already has a match-exhaustiveness substrate for ordinary ADTs, but some binding positions still treat pattern matching as an implicit runtime operation. That is the wrong default. If a construct must continue after binding a pattern, the type checker must prove that the pattern cannot fail for the scrutinee type.

This design introduces one rule: Ash does not allow implicit refutable matching. A refutable pattern is allowed only in a construct whose syntax and semantics explicitly say where the non-match case goes. `if let ... else` is a total two-branch eliminator over `P | not P`; current selective `receive` remains a refutable filtering form.

## Problem

A pattern in `let Some { value } = maybe` can fail when `maybe : Option<T>`. If the language accepts that binding without an explicit failure path, one of three undesirable semantics appears:

1. the runtime raises a pattern-failure error from a program that looked statically valid;
2. the binder silently skips or discards the non-match case;
3. later code runs with bindings that were never produced.

All three choices conflict with Ash's static-first design. They also make workflow audit and failure reporting less reliable because an ordinary binder can hide a branch point.

## Design model

Pattern uses fall into three categories.

| Category | Examples | Rule |
|----------|----------|------|
| Irrefutable binder | pure block `let`, core `Expr::Let`, workflow `let`, `observe ... as`, `spawn ... as`, `split ... as`, `foreach` element binders | The pattern must be type-aware irrefutable for the scrutinee type. |
| Exhaustive eliminator | `match`, total `with_error` handler, future total protocol dispatch | The arm set must cover the closed constructor universe, or include a well-typed wildcard/default arm that is universal without constructor enumeration. |
| Explicit complement/refutable construct | `if let ... else`, current selective `receive` arms | `if let ... else` is total by implicit complement; selective `receive` remains a filtering form whose non-match behavior is construct-defined. |

The category is semantic, not syntactic. A variant pattern may be irrefutable if the scrutinee type has exactly that one reachable constructor and all nested fields are irrefutable. A tuple or record pattern may be irrefutable for a known matching product type; for an unknown product shape the checker must fail closed, and for an incompatible type it must report a pattern type/impossibility error rather than treating the pattern as merely refutable.

## Core decisions

- Binding positions must use a shared type-aware irrefutability check, not `Pattern::is_refutable()` alone.
- Exhaustive eliminators must use the same canonical constructor universe as pattern typing from SPEC-068.
- Matching diagnostics must name the construct, scrutinee type, missing case or non-match reason, and the preferred rewrite.
- Runtime pattern errors such as expression `LetPatternBindFailed`, workflow `PatternMatchFailed`, and `NonExhaustiveMatch` remain defensive unchecked-IR/runtime errors, not the normal outcome of checked source.
- `if let ... else` must be treated as total by implicit complement, with mandatory `else`, non-fatal unreachable-else diagnostics for irrefutable patterns, hard errors for impossible patterns, original-environment else typing, and no negative type refinement in this phase.
- Selective `receive` remains an explicit refutable filtering form for this phase.
- Selective `receive` keeps its current matching/filtering role until a later spec tightens protocol-totality and timeout semantics.

## Current baseline

Live code already provides useful pieces:

- `crates/ash-typeck/src/exhaustiveness.rs` checks top-level ADT constructor coverage.
- `crates/ash-typeck/src/check_expr.rs::check_match` calls `check_exhaustive_canonical` / `check_exhaustive` for `Expr::Match`.
- `crates/ash-typeck/src/check_expr.rs` block `let` handling currently binds pattern variables after checking the RHS expression, without first requiring the pattern to be irrefutable.
- `crates/ash-typeck/src/check_expr.rs::check_with_error` checks each handler arm against a fresh failure payload variable and unifies handler body types, but does not prove handler coverage.
- `crates/ash-interp/src/pattern.rs::match_pattern` and observe execution can still return runtime pattern failures.

This phase should close the semantic gap without reopening ADT layout, GADT/refinement patterns, or selective receive semantics.

## Non-goals

- No GADT/refinement-pattern semantics.
- No type-level sealed-domain/promoted-constructor runtime matching.
- No implicit conversion from pattern failure into `None`, `Err`, operational `fail`, or workflow rejection.
- No broad redesign of `receive`; selective receive remains allowed refutable matching in this phase.
- No parser syntax migration unless the audit finds an existing surface cannot express the required diagnostics.
- No negative type-refinement system for complement branches in this phase.

## Handoff

The normative contract is [SPEC-076](../spec/SPEC-076-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md). The implementation order is [PLAN-126](../plan/PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md).
