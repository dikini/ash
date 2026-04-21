# PLAN-097: Act Monad Implementation

**Date:** 2026-04-22
**Status:** Draft
**Depends on:** SPEC-047, Phase 96 (complete)
**Related:** NOTE-005, SPEC-001/002/003/004/025/027/031/BUILTIN-FN

## 1. Goal

Implement the Act monad as specified in SPEC-047, unifying pure expression evaluation and effectful workflow execution into a single composable framework.

## 2. Track Structure

### Track A: Surface + Core Foundation
Prerequisite: nothing (can start immediately after Phase 96 merge).

| Task | Description | Est. | Dependencies |
|------|-------------|------|--------------|
| TASK-672 | Add `ActStmt` type + `Expr::ActBlock` to surface AST (`surface.rs`) | 2h | — |
| TASK-673 | Parse `act { ... }` in expression context (`parse_expr.rs`) | 3h | TASK-672 |
| TASK-674 | Add `Expr::ActBlock` to core AST (`ast.rs`) | 1h | — |
| TASK-675 | Lower `SurfaceExpr::ActBlock` to `CoreExpr::ActBlock` with bind/unit desugaring (`lower.rs`) | 4h | TASK-672, TASK-674 |
| TASK-676 | Desugarer pass-through for `ActBlock` in all three desugar passes (`desugar.rs`) | 2h | TASK-672 |
| TASK-677 | Add `invoke`, `unit`, `bind` to builtin dispatch table (`eval.rs`) | 3h | — |
| TASK-678 | Register `invoke`, `unit`, `bind` in engine builtin fn registry | 2h | TASK-677 |
| TASK-679 | Property tests for act block parsing and lowering | 3h | TASK-673, TASK-675 |

**Track A gate:** `act { ret 42 }` parses, lowers, and evaluates to a closure value.

### Track B: Type System
Prerequisite: Track A (parsing + lowering).

| Task | Description | Est. | Dependencies |
|------|-------------|------|--------------|
| TASK-680 | Register `Act` type constructor with kind `* -> *` in type env | 1h | — |
| TASK-681 | Type-check `Expr::ActBlock`: bind rule, pure bind rule, return rule | 4h | TASK-680, TASK-675 |
| TASK-682 | Type-check `invoke`: `String → String → List → Act Value` | 2h | TASK-680 |
| TASK-683 | Purity enforcement: reject `ActBlock`/`invoke` in pure fn bodies | 3h | TASK-681 |
| TASK-684 | Type-check `unit`, `bind`, `then`, `guard` builtin signatures | 2h | TASK-680 |
| TASK-685 | Property tests for type system: monad laws, purity rejection, act block inference | 4h | TASK-681–684 |

**Track B gate:** Type checker accepts effectful fn declarations and rejects pure fn bodies with act blocks.

### Track C: Runtime
Prerequisite: Track A. Can proceed in parallel with Track B (but needs Track B for full testing).

| Task | Description | Est. | Dependencies |
|------|-------------|------|--------------|
| TASK-686 | Define `ActEnv` struct in interpreter | 2h | — |
| TASK-687 | Implement `invoke` builtin: policy check → provider dispatch → effect log append | 4h | TASK-677, TASK-686 |
| TASK-688 | Implement `unit` builtin: lift value into Act closure | 2h | TASK-677, TASK-686 |
| TASK-689 | Implement `bind` builtin: thread ActEnv through closure chain | 4h | TASK-677, TASK-686 |
| TASK-690 | Implement `then` and `guard` builtins | 2h | TASK-689 |
| TASK-691 | Evaluate `Expr::ActBlock`: produce closure capturing ActEnv threading | 3h | TASK-688, TASK-689 |
| TASK-692 | Workflow bridge: construct ActEnv from workflow execution context | 3h | TASK-686 |
| TASK-693 | Integration tests: effectful fn composition, nested act blocks, workflow+act interop | 4h | TASK-691, TASK-692 |

**Track C gate:** `fn read(p) -> Act String { act { x = invoke("Fs","read",[p]); ret x } }` executes with real provider.

### Track D: Specs + Amendment + Cross-Layer Tests
Prerequisite: All tracks complete.

| Task | Description | Est. | Dependencies |
|------|-------------|------|--------------|
| TASK-694 | Amend SPEC-001: add ActBlock, Invoke, ActStmt to core forms | 1h | Track A |
| TASK-695 | Amend SPEC-002: document expression-level act syntax | 1h | Track A |
| TASK-696 | Amend SPEC-003: Act type constructor, purity rules, typing rules | 2h | Track B |
| TASK-697 | Amend SPEC-004: semantic rules for bind, invoke, act block | 2h | Track B, C |
| TASK-698 | Amend SPEC-025: small-step rules for expression-level act | 1h | Track C |
| TASK-699 | Amend SPEC-027: effectful fn declaration, purity boundary | 1h | Track B |
| TASK-700 | Amend SPEC-031: note on ActEnv-capturing closures | 0.5h | Track C |
| TASK-701 | Amend SPEC-BUILTIN-FN: add invoke, unit, bind | 1h | Track A |
| TASK-702 | Create `std/src/act.ash` with unit, bind, then, guard library functions | 2h | Track C |
| TASK-703 | Cross-layer validation: end-to-end from parse → type → execute | 3h | All tracks |
| TASK-704 | Performance baseline for act block execution | 1h | Track C |

**Track D gate:** All specs amended, library module created, cross-layer tests pass.

## 3. Decision Gates

**D1: Act block representation.** Resolved: `Expr::ActBlock` with `Vec<ActStmt>`. Alternative (desugar directly to nested `bind`/`unit` calls in the lowerer) was rejected because it loses source structure needed for error messages and debugging.

**D2: Invoke as Expr variant vs builtin fn.** Resolved: `invoke` is a `builtin fn` dispatched through the existing `Expr::Call` path. No new `Expr::Invoke` variant needed. The lowerer maps `invoke(a,b,c)` to `Expr::Call { func: "invoke", .. }`.

**D3: ActEnv as Value vs runtime-only.** Resolved: runtime-only. `ActEnv` is a Rust struct passed through closures. It is not an Ash value (cannot be constructed or inspected by user code).

**D4: Workflow backward compatibility.** Resolved: `Workflow::Act` continues unchanged. Expression-level `act {}` is a new, separate construct. No migration of existing workflow syntax.

## 4. Spec Amendment Inventory

| Spec | Change | Severity |
|------|--------|----------|
| SPEC-001 | Add `Expr::ActBlock`, `ActStmt` to core forms | Minor extension |
| SPEC-002 | Add expression-level `act {}` grammar, dual-context dispatch | Minor extension |
| SPEC-003 | Add `Act<A>` typing, purity rules, act block rules | Moderate extension |
| SPEC-004 | Add `ActEnv` domain, `ACT-BIND`, `ACT-INVOKE` rules | Moderate extension |
| SPEC-025 | Add small-step for act block reduction | Minor addition |
| SPEC-027 | Add effectful fn form, amend purity definition | Moderate amendment |
| SPEC-031 | Note on ActEnv-capturing closures | Minor note |
| SPEC-BUILTIN-FN | Add `invoke`, `unit`, `bind`, `then`, `guard` | Minor extension |

## 5. Risk Assessment

| Risk | Mitigation |
|------|------------|
| Circular dependency: bind needs closures, closures need ActEnv | ActEnv is a plain struct, not dependent on closures. Bind operates on Value::Closure. |
| Type system complexity: Act<A> unification | Act<A> is a Type::Constructor, unification already handles constructors. |
| Performance: nested bind chains create deep closures | Optimization pass in future phase. Correctness first. |
| Breaking existing stdlib .ash files | No changes to Workflow::Act. Existing files untouched. Migration is a separate phase. |
| Cross-crate data flow: ActStmt must survive lowering | ActStmt is lowered to nested CoreExpr calls. Only CoreExpr crosses crate boundaries. |

## 6. Estimated Total

- Track A: 20 hours
- Track B: 16 hours
- Track C: 24 hours
- Track D: 15.5 hours
- **Total: 75.5 hours**

Tracks A, B, C have some parallelism (A must finish first, then B and C can proceed concurrently). Realistic calendar time: 2-3 weeks with sequential subagent execution.
