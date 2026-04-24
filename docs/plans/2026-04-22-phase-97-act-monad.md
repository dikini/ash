# PLAN-097: Act Monad Implementation

**Date:** 2026-04-22
**Status:** Complete
**Depends on:** SPEC-047, Phase 96 (complete)
**Related:** NOTE-005, SPEC-001/002/003/004/027/031

## 1. Goal

Implement expression-level `Act<A>` as an additive capability in Ash: a first-class effectful computation model that interoperates with the existing workflow runtime without requiring immediate replacement of workflow execution or the current `Type::Fun(...)` substrate.

## 2. Phase-97 Architectural Decisions

These decisions are frozen for this plan and MUST be reflected consistently across tasks:

1. **Surface-only act block.** `act { ... }` is introduced in the surface AST as `Expr::ActBlock` with `ActStmt` items. It does not survive into the canonical core IR in Phase 97.
2. **Lower-away strategy.** `Expr::ActBlock` lowers into existing core expression forms using nested `bind` / `unit` structure plus existing `Expr::Call`, `Expr::FnDef`, and `Expr::FnApply`.
3. **`invoke` is a runtime primitive callable.** It travels through the existing `Expr::Call` path. It is not a dedicated `Expr::Invoke` variant and it is not a pure `builtin fn` under the current SPEC-BUILTIN-FN contract.
4. **Library-vs-runtime split.** `unit`, `bind`, `then`, and `guard` remain ordinary library functions in `std/src/act.ash`. Only `invoke` requires runtime support.
5. **Additive typing model.** `Act<A>` is introduced via existing `Type::Constructor`. Phase 97 does not retire or redefine `Type::Fun(...)`; it coexists with it.
6. **No SPEC-025 scope expansion.** Expression-level micro-stepping remains out of scope in Phase 97.

## 3. Track Structure

### Track A: Preflight + Surface + Lowering Foundation
Prerequisite: nothing.

| Task | Description | Est. | Dependencies |
|------|-------------|------|--------------|
| TASK-672 | Preflight doc cleanup: normalize `Act<T>` syntax, remove `Expr::Invoke`, align builtin/library split in SPEC-047 and this plan | 2h | — |
| TASK-673 | Add surface `ActStmt` type + `Expr::ActBlock` to `surface.rs` | 2h | TASK-672 |
| TASK-674 | Parse `act { ... }` in expression context in `parse_expr.rs` | 3h | TASK-673 |
| TASK-675 | Lower `SurfaceExpr::ActBlock` into existing core expressions using nested `bind` / `unit` calls in `lower.rs` | 5h | TASK-673, TASK-674 |
| TASK-676 | Property/integration tests for act-block parsing and lowering | 3h | TASK-674, TASK-675 |

**Track A gate:** `act { ret 42; }` parses and lowers without any new core IR variants.

### Track B: Type System
Prerequisite: Track A.

| Task | Description | Est. | Dependencies |
|------|-------------|------|--------------|
| TASK-677 | Register `Act` type constructor with kind `* -> *` in type environment | 1h | TASK-672 |
| TASK-678 | Type-check `Expr::ActBlock`: bind, pure-bind, and return rules | 4h | TASK-675, TASK-677 |
| TASK-679 | Type-check `invoke(provider, action, args)` as `Act<Value>` | 2h | TASK-677 |
| TASK-680 | Purity enforcement: reject `act {}` and `invoke(...)` in pure `fn` bodies | 3h | TASK-678, TASK-679 |
| TASK-681 | Record and test the additive coexistence rule with existing `Type::Fun(...)` behavior | 2h | TASK-678 |
| TASK-682 | Type-system tests: purity rejection, inference, and `Act<T>` constructor behavior | 4h | TASK-678-TASK-681 |

**Track B gate:** The type checker accepts `fn ... -> Act<T>` and rejects `act {}` or `invoke(...)` inside pure `fn ... -> T` bodies.

### Track C: Runtime
Prerequisite: Track A. Can proceed in parallel with Track B after surface/lowering stabilizes.

| Task | Description | Est. | Dependencies |
|------|-------------|------|--------------|
| TASK-683 | Define `ActEnv` runtime struct and construction boundary | 2h | TASK-672 |
| TASK-684 | Add `invoke` runtime primitive dispatch through the existing `Expr::Call` path | 4h | TASK-683 |
| TASK-685 | Implement closure-backed execution path for desugared `Act<T>` values | 4h | TASK-675, TASK-683, TASK-684 |
| TASK-686 | Workflow bridge: construct/apply `ActEnv` from workflow execution context when needed | 3h | TASK-683, TASK-685 |
| TASK-687 | Runtime integration tests: effectful fn composition, nested act blocks, workflow + act interop | 4h | TASK-685, TASK-686 |

**Track C gate:** `fn read(p: String) -> Act<String> { act { x = invoke("Fs", "read", [p]); ret x; } }` executes against a real provider path.

### Track D: Spec + Library + Cross-Layer Validation
Prerequisite: Tracks A-C complete.

| Task | Description | Est. | Dependencies |
|------|-------------|------|--------------|
| TASK-688 | Finalize SPEC-047 amendments and targeted updates to SPEC-002/003/004/027/031 | 2h | Tracks A-C |
| TASK-689A | Establish honest `std::act` substrate for ordinary library helpers | 3h | TASK-688, TASK-687 |
| TASK-689B | Preserve imported ordinary `pub fn` signatures for `std::act` | 3h | TASK-689A |
| TASK-689C | Establish policy/environment substrate for ordinary `std::act` `guard` | 3h | TASK-689B |
| TASK-689E | Refine library type-export semantics for opaque `Act` | 3h | TASK-689C |
| TASK-689D | Establish honest opaque `Act` library boundary for ordinary `std::act` helpers | 3h | TASK-689E |
| TASK-689 | Replace placeholder `std::act` stubs with ordinary library implementations | 2h | TASK-689D |
| TASK-690 | Cross-layer validation: parse -> type -> execute end-to-end examples | 3h | TASK-688, TASK-689 |
| TASK-691 | Performance baseline for desugared act-block execution | 1h | TASK-690 |

**Track D gate:** docs, the `std::act` substrate, library definitions, end-to-end tests, and the approximate benchmark smoke baseline all reflect the same additive architecture. This gate is now complete in the Phase 97 worktree.

## 4. Decision Gates

**D1: Act block representation.** Resolved: surface-only `Expr::ActBlock` with `Vec<ActStmt>`. It lowers away before core IR.

**D2: Invoke representation.** Resolved: `invoke` is a runtime primitive callable routed through the existing `Expr::Call` path. No `Expr::Invoke` variant.

**D3: Runtime boundary.** Resolved: `ActEnv` is runtime-only Rust state, not an Ash value.

**D4: Workflow compatibility.** Resolved: `Workflow::Act` remains unchanged in Phase 97. Expression-level `act {}` is additive.

**D5: Builtin/library split.** Resolved at the contract level: `unit`, `bind`, `then`, and `guard` are library functions; `invoke` is the only runtime primitive introduced by this phase. Track D closeout is now landed in the Phase 97 worktree: `std/src/act.ash` uses the ordinary-library helper surface, `guard` executes through the deferred internal `act::__guard` bridge so policy is checked at Act-force time, focused engine/interpreter validation covers import + type + execute behavior, and a standalone `ash-bench` baseline exists for representative desugared Act execution paths.

**D6: Typing coexistence.** Resolved: `Act<A>` is additive and does not retire existing `Type::Fun(...)` behavior in this phase.

**D7: Builtin substrate decision protocol.** Resolved as an implementation rule-set rather than a precommitted branch:
- prefer `builtin type ActEnv` plus ordinary `type Act<A> = ActEnv -> (ActEnv, A)`;
- if real implementation pressure makes that materially riskier or more complex, escalate to builtin `Act` with the same explicit equation;
- only if that also becomes materially riskier or more complex should Phase 97 fall back to a more opaque builtin `Act` form.
- prefer treating the explicit RHS as definitional equality; downgrade to checked correspondence only if real engine/runtime complexity justifies it.
- builtin artifacts use internal flat identities, not source module paths, so reexports/aliases do not change builtin meaning.

## 5. Spec Amendment Inventory

| Spec | Change | Severity |
|------|--------|----------|
| SPEC-047 | Normalize architecture around surface-only act blocks and runtime-primitive `invoke` | Major cleanup |
| SPEC-002 | Add expression-level `act {}` grammar and dual-context dispatch | Minor extension |
| SPEC-003 | Add `Act<A>` typing, purity rules, and coexistence note with `Type::Fun(...)` | Moderate extension |
| SPEC-004 | Add `ActEnv` and expression-level effectful computation semantics | Moderate extension |
| SPEC-027 | Add effectful `fn ... -> Act<T>` form and purity-boundary amendment | Moderate amendment |
| SPEC-031 | Clarify relationship between closure values, `Act<T>`, and existing workflow-closure typing | Minor amendment |

No Phase-97 SPEC-025 amendment is planned.
No Phase-97 SPEC-BUILTIN-FN amendment is planned.

## 6. Risk Assessment

| Risk | Mitigation |
|------|------------|
| Lowering complexity for pure-vs-monadic bind forms | Keep Phase-97 lowering simple; optimize later if needed |
| Confusion between `Act<T>` and `Type::Fun(...)` | Keep additive coexistence explicit in docs and tests |
| Runtime dispatch ambiguity for `invoke` | Route through a clearly distinguished runtime primitive path keyed off `Expr::Call { func: "invoke", .. }` |
| Drift against existing workflow semantics | Keep `Workflow::Act` unchanged in this phase |
| Overreaching into small-step semantics | Explicitly keep SPEC-025 untouched in Phase 97 |

## 7. Estimated Total

- Track A: 15 hours
- Track B: 16 hours
- Track C: 17 hours
- Track D: 23 hours
- **Total: 71 hours**

Tracks B and C can partially overlap once Track A is stable. Realistic calendar time: 1-2 focused implementation weeks.
