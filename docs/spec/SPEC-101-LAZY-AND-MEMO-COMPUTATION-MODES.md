---
id: spec.ash.lazy-memo-computation-modes
title: Lazy and Memo Computation Modes
kind: spec
audience: [human, agent]
authority: design
status: implemented-mvp
stability: alpha
last_verified: 2026-06-20
verified_against:
  specs:
    - docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md
    - docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md
    - docs/spec/SPEC-098b-TARGET-IR.md
    - docs/spec/SPEC-099-CORE-LANGUAGE.md
    - docs/spec/SPEC-100-CORE-TYPE-CHECKING.md
---

# SPEC-101: Lazy and Memo Computation Modes

**Status:** Implemented MVP (Phase 163)
**Scope:** Core Ash representation, typing, operational behavior, and Core-to-CPS lowering for `lazy` and `memo` computation modes.
**Depends on:** SPEC-096b, SPEC-097b, SPEC-098b, SPEC-099, SPEC-100.
**Amends:** SPEC-097b, SPEC-098b, SPEC-099, and SPEC-100. This spec requires a value-level thunk carrier with captured handler/provider chain, but does not require new SPEC-098b CPS tail-term variants.

## 1. Summary

Ash supports three evaluation modes:

| Mode | Evaluation timing | Sharing |
|------|-------------------|---------|
| `strict` | evaluate before binding or call entry | none |
| `lazy` | evaluate when forced | none; each force may re-run |
| `memo` | evaluate when first forced | shared result or failure is cached |

SPEC-097b already defines the type-level contract for these modes. This spec defines the Core Ash representation and lowering contract needed to make that contract executable and auditable.

The key design rule is conservative:

> Add mode structure to Core Ash because mode affects observable timing, sharing, bottom, and effect execution. Do not add new CPS IR nodes unless existing CPS closures, values, records, continuations, and runtime memo cells cannot preserve the required identity and forcing behavior.

## 2. Motivation

Lazy and memo computation are not optimization hints. They change:

1. when effects fire;
2. whether divergence or failure is observed at binding time or force time;
3. whether repeated demand re-runs a computation;
4. which dynamic path owns provider/handler operation authority and resource access at force time;
5. whether shared subcomputations preserve asymptotic behavior.

Representing modes only as comments or backend conventions would make Core type checking, row diagnostics, contract discharge, runtime traces, and Core-to-CPS lowering disagree. Core therefore needs explicit mode carriers even though CPS IR can lower them into ordinary closure and value machinery.

## 3. Non-Goals

This spec does not add:

- implicit subtyping or coercion between modes;
- call-by-need as the language default;
- lazy pattern matching;
- lazy records, lazy tuples, or per-field laziness;
- parallel forcing or speculative evaluation;
- persistent cross-run memo caches;
- provider-level caching semantics;
- broad optimizer rewrites that move effectful forces across observable boundaries.

## 4. Core Type Shape

SPEC-097b defines mode invariance. Core Ash materializes that rule with an explicit mode type wrapper:

```text
Type ::= ...
       | ModeType

ModeType ::= Strict Type
           | Lazy Type
           | Memo Type
```

`Strict T` is the default source spelling and may be rendered as `T` in user-facing diagnostics. Core should preserve it explicitly where the distinction matters for round-trips, summaries, or mode mismatch diagnostics.

Mode types are invariant:

```text
Strict A != Lazy A
Strict A != Memo A
Lazy A   != Memo A
```

No mode conversion is implicit. All conversion uses explicit operations listed in Section 8.

## 5. Core Values and Expressions

SPEC-099 is amended with mode-aware computation carriers:

```text
Value ::= ...
        | Thunk { mode: ThunkMode, body: Expr, row: Row, captures: CaptureSet }

ThunkMode ::= Lazy | Memo

Expr ::= ...
       | LetMode { name: Name, ty: ModeType, mode: EvalMode, expr: Expr, body: Expr }
       | Force { name: Name, thunk: Atom, body: Expr }

EvalMode ::= Strict | Lazy | Memo
```

`LetMode` is the Core form for a mode-sensitive binding when the surface or elaboration phase must preserve mode semantics. Strict `LetMode` may normalize to existing `LetCall`, `LetPrim`, or `LetVal` forms once the checker has proven that no mode boundary remains.

`LetMode.mode` and `LetMode.ty` must agree exactly. The checker must reject malformed Core
where `mode: Lazy` is paired with `ty: Memo A`, where `mode: Memo` is paired with
`ty: Lazy A`, or where `mode: Strict` is paired with a non-`Strict` mode type. Neither field
wins by precedence; disagreement is an invalid Core term.

`Thunk` is a value because constructing a thunk captures an expression and environment without evaluating the expression body. `Thunk.body` is not user data and must not be inspected by ordinary programs.

`Force` is an expression because forcing may run the thunk body and therefore may perform the thunk's latent row, diverge, raise, or trap.

## 6. Operational Semantics

### 6.1 Strict

Strict bindings evaluate their initializer before binding the name:

```text
let x = expr; body
```

Core evaluates `expr`, binds its result to `x`, then evaluates `body`.

### 6.2 Lazy

A lazy binding constructs a fresh non-sharing thunk:

```text
let lazy x = expr; body
```

Forcing `x` evaluates `expr` in the captured lexical environment and active runtime/provider chain captured by the thunk. Each force evaluates the body again. A successful value, failure, trap, or divergence is not cached.

### 6.3 Memo

A memo binding constructs a fresh sharing thunk with stable memo identity:

```text
let memo x = expr; body
```

The first force evaluates `expr` in the captured lexical environment and active runtime/provider chain captured by the thunk, then records the terminal outcome in the thunk's memo cell. Later forces return or re-raise that same cached terminal outcome without re-running the body.

The memo cell is process-local runtime state. It is not a language-visible value, not serializable program data, and not a persistent cache.

### 6.4 Cached Failures and Traps

Memoization caches terminal outcomes, not only successful values:

- a successful value is returned on later forces;
- a recoverable `fail` outcome is re-raised according to the lowered failure representation;
- an unrecoverable `Trap` is repeated as the same trap reason;
- divergence never fills the memo cell.

Caching failure is required so repeated force does not duplicate effects before the failing point.
If the trap reason is `ContractViolation(ContractDiagnostic)`, the memo cell preserves the
original diagnostic and blame label. Later forces replay that terminal diagnostic; they may
record a replay event, but they must not create a new blame event.

### 6.5 Re-Entrant Forcing

If a memo thunk is forced while its own first evaluation is still in progress, the runtime must reject the re-entrant force with a structured runtime diagnostic unless a later spec defines black-hole semantics.

The initial required behavior is:

```text
MemoState ::= Empty | Evaluating | Filled(TerminalOutcome)

force memo:
  Empty      -> Evaluating, run body, then Filled(outcome)
  Evaluating -> Trap(Panic("re-entrant memo force")) or equivalent structured runtime error
  Filled(o)  -> replay o
```

## 7. Row Accounting

Mode affects when effects fire, not which requirements exist.

For a thunk with latent row `rho`:

```text
construct lazy/memo thunk:
  local row = {}
  total row = continuation row

force thunk:
  local row = rho
  total row = rho union continuation row
```

A function that accepts or returns `Lazy A` or `Memo A` must export the latent row of the thunk body where the row is part of the mode type, function summary, or associated obligation metadata. The row must not be erased just because construction is pure.

Memo thunks still expose the latent row at every force site for static checking. Although later runtime forces may hit the cache and perform no effects dynamically, a static checker must assume that a given force may be the first force unless it has a local proof of filled state. The initial Core checker should not attempt such state-sensitive refinement.

### 7.1 Purity and contract timing

Purity is denotational, following SPEC-097b §15.7. Constructing a `lazy` or `memo` thunk is
pure when construction itself has row `{}`. Forcing the thunk is pure exactly when the latent
row is `{}` at that force site.

Contract checks happen at the observation boundary:

- `lazy`: checks run on every force because the body re-runs on every force;
- `memo`: checks run on first force, then the terminal outcome is cached and replayed;
- `strict`: checks happen at the ordinary call, return, or data boundary.

The memo cache cell is process-local runtime state and not an Ash-visible row effect. It does
not make an otherwise empty-row memoized computation impure.

## 8. Explicit Conversion Operations

The standard mode conversion surface from SPEC-097b lowers through these Core operations:

| Operation | Type | Core behavior |
|-----------|------|---------------|
| `delay` | `Strict A -> Lazy A` | construct lazy thunk returning the already-computed value |
| `delay_memo` | `Strict A -> Memo A` | construct memo thunk returning the already-computed value |
| `force_unsafe` | `Lazy A -> Strict A` or `Memo A -> Strict A` | force the thunk |
| `memoize_unsafe` | `Lazy A -> Memo A` | construct a memo thunk whose first force forces the lazy thunk |
| `strip_cache_unsafe` | `Memo A -> Lazy A` | construct a lazy thunk that forces the memo thunk |

The `_unsafe` suffix is semantic: the operation changes when bottom, failure, or effects are observed. It does not imply memory unsafety.

## 9. Capture and Authority

A thunk captures:

- lexical values needed by its body;
- the active handler/provider chain needed to interpret raised operations;
- source/provenance metadata for diagnostics and traces;
- for `memo`, a fresh memo identity and cell.

Captured authority must follow existing row and discharge rules. Constructing a thunk does not grant authority. Forcing a thunk requires the same residual row discharge that evaluating the original body would require.

The target semantics for this spec is creation-time capture, matching SPEC-098b continuation closure capture of handler/provider chains. A conforming implementation must therefore preserve the authority boundary that was active at thunk construction. Force-time provider resolution is not the public semantics of this spec.

## 10. Core Type Checking

SPEC-100 is amended with these checks:

1. Mode types are well-formed only when their inner type is well-formed.
2. Mode equality is invariant.
3. `Thunk { mode, body, row }` checks the body against the thunk result type and verifies `row` is the body local row.
4. `LetMode.mode` and `LetMode.ty` must agree exactly; mismatches are invalid Core and must be rejected before binding the name.
5. `LetMode Strict` checks like the existing strict binding forms.
6. `LetMode Lazy` binds `name: Lazy A` and gives construction row `{}` while recording the initializer row as the thunk latent row.
7. `LetMode Memo` binds `name: Memo A` and gives construction row `{}` while recording the initializer row as the thunk latent row.
8. `Force` requires an atom of type `Lazy A` or `Memo A`, binds the forced result as `Strict A`, and contributes the thunk latent row.
9. Public summaries must preserve mode and latent-row facts for exported parameters, returns, and bindings.
10. Diagnostics must distinguish mode mismatch from ordinary type mismatch.

The initial checker may require explicit Core annotations for thunk result types and latent rows. It does not need to infer latent rows from arbitrary unannotated source expressions.

## 11. Core-to-CPS Lowering

No new SPEC-098b tail-term variant is required for the initial design. A value-level thunk carrier is required because ordinary `Lam` values do not carry a captured handler/provider chain in SPEC-098b.

Core lowers modes to existing CPS tail terms plus the value-level thunk carrier:

| Core concept | CPS representation |
|--------------|--------------------|
| lazy thunk | `ThunkClosure { mode: Lazy, body: zero-arg Lam, env, chain }` |
| memo thunk | `ThunkClosure { mode: Memo, body: zero-arg Lam, env, chain, cell }` |
| force lazy | runtime force operation evaluates `body` under the thunk's captured `env` and `chain`, using the force-site continuation as the return continuation |
| force memo | runtime force operation checks/updates `cell`, then either replays the cached terminal outcome or evaluates `body` under the thunk's captured `env` and `chain` |
| `delay(value)` | lazy thunk whose body immediately jumps `value` |
| `delay_memo(value)` | memo thunk whose first body immediately jumps `value` |

The thunk's `body` may be represented by an ordinary zero-argument `Lam`, but the thunk as a whole is not just that `Lam`. The thunk carrier owns the captured lexical environment and captured handler/provider chain. Forcing the thunk must restore that captured chain for the dynamic extent of thunk-body evaluation, while the force-site continuation remains the continuation that receives the produced value or replayed terminal outcome.

The required value-level CPS/runtime shape is:

```text
Value ::= ...
        | ThunkClosure {
            mode: ThunkMode,
            body: Atom,          -- zero-argument CPS Lam
            env: Env,
            chain: HandlerChain,
            cell: Option<MemoCellId>
          }
```

`Force` must not lower to a plain `Call { func: body, args: [], cont: current_cont }` for an effectful thunk, because that would dispatch raises through the force-time handler/provider chain. It must lower to the runtime force operation for `ThunkClosure`, which evaluates the body under `ThunkClosure.chain`.

## 12. Tracing and Observability

Runtime traces must distinguish:

- thunk construction;
- lazy force start/end;
- memo force start/end;
- memo cache hit;
- memo cache fill;
- memo re-entrant-force rejection;
- terminal outcome replay for cached failures or traps.

Trace output must not expose internal memo-cell storage addresses. It may expose stable per-run thunk ids for correlation.

## 13. Acceptance Criteria

An implementation conforms to this spec when:

1. `Lazy A`, `Memo A`, and `Strict A` are represented distinctly in Core type checking.
2. Mode mismatch is rejected without implicit conversion.
3. Lazy force re-runs the thunk body on repeated force.
4. Memo force runs the thunk body at most once after successful or failing completion.
5. Memo force caches successful values, recoverable failures, and traps.
6. Re-entrant memo force is rejected deterministically.
7. Construction of lazy and memo thunks has row `{}`.
8. Forcing a thunk contributes the thunk latent row.
9. Forcing an effectful thunk dispatches raised operations through the handler/provider chain captured at thunk construction.
10. Core-to-CPS lowering uses a value-level thunk carrier and existing CPS tail-term forms.
11. Runtime traces distinguish construction, force, cache fill, cache hit, and replay.

## 14. Relationship to Other Specs

| Spec | Relationship |
|------|--------------|
| SPEC-096b | Defines effect rows and discharge; this spec preserves row identity across delayed execution. |
| SPEC-097b | Defines evaluation modes at the type-system level; this spec materializes them in Core. |
| SPEC-098b | Provides the CPS closure, continuation, handler-chain, call, jump, and trap machinery used for lowering; this spec adds the required value-level thunk carrier for creation-time chain capture. |
| SPEC-099 | Owns Core Ash syntax; this spec amends it with mode-aware Core forms. |
| SPEC-100 | Owns Core type checking; this spec amends it with mode typing and force row rules. |

## 15. Open Questions

1. Should the textual `.core` fixture format print `Strict T` explicitly, or keep strict mode implicit outside mismatch diagnostics?
2. Should memo thunk cached recoverable failure replay preserve the original trace id or emit a new replay event linked to the original?
3. Should black-hole semantics eventually replace deterministic re-entrant-force rejection?
4. Should optimizer passes be allowed to convert `lazy` to `memo` when the thunk body is proven pure and terminating?

## 16. Implementation Tasks

No implementation task is assigned by this spec. A future plan should create task files under `docs/plan/tasks/` before changing Rust code.

Likely task slices:

- Core AST and `.core` text-format carriers for `ModeType`, `Thunk`, `LetMode`, and `Force`.
- Core validator/type-checker rules for mode invariance, latent rows, and force row accounting.
- Core-to-CPS lowering for lazy thunks through `ThunkClosure` carriers whose body is a zero-argument CPS lambda and whose value stores the captured handler/provider chain.
- Runtime memo value/cell support and trace events.
- Focused property tests for lazy re-run, memo single-run, cached failure, row accounting, and mode mismatch.

## 17. Changelog

- 2026-06-20: Initial draft. Defines Core-level lazy and memo computation semantics, typing rules, row accounting, CPS lowering through existing IR forms, and the decision not to add new CPS IR term variants for the initial design.
- 2026-06-21: Corrected CPS lowering to require a value-level thunk carrier with captured handler/provider chain, preserving creation-time authority semantics while still avoiding new CPS tail-term variants.
- 2026-06-21: Required `LetMode.mode` and `LetMode.ty` to agree exactly, and rejected malformed Core mode/type mismatches.
- 2026-06-28: Reconciled with NOTE-028 and NOTE-029. Added §7.1 denotational purity and contract timing for lazy/memo modes, and clarified memo replay of `ContractViolation(ContractDiagnostic)` without creating a new blame event.
