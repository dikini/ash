# TASK-423: Workflow Binding Propagation and Honest Unsupported Bindings

## Status: ✅ Complete

## Description

Close the remaining workflow-binding gaps uncovered during the focused post-TASK-422 review sweep. This task tightens the typechecker’s treatment of `Observe`, `For`, and `Propose` bindings so that workflow-scoped names are handled honestly during interface-call validation and declared return-type checking.

The narrow goal is to eliminate the current split where `For`-bound values lose their collection-derived type and `Propose.binding` is surfaced syntactically but ignored semantically. For `Observe`, this task should avoid silently fabricating evidence from fresh type variables when no honest result type is available from the current MVP machinery.

## Specification Reference

- [TASK-415: Closed-World Interfaces MVP Spec Cut](TASK-415-closed-world-interfaces-mvp-spec-cut.md)
- [TASK-421: Closed-World Interfaces AST and Parser Substrate](TASK-421-closed-world-interfaces-ast-and-parser-substrate.md)
- [TASK-422: Closed-World Interfaces Coherence and Method Resolution](TASK-422-closed-world-interfaces-coherence-and-method-resolution.md)
- [TYPES-002 V2 MVP Cut](../../ideas/type-system/TYPES-002-v2-mvp-cut.md)
- [SPEC-003: Type System](../../spec/SPEC-003-TYPE-SYSTEM.md)

## Dependencies

- ✅ TASK-421 complete
- ✅ TASK-422 complete

## Requirements

### Functional Requirements

1. `For`-bound pattern variables must inherit an honest element type derived from the loop collection during workflow-side interface-call validation.
2. `For`-bound pattern variables must also be visible to declared workflow return-type inference so workflows like `for item in items { ret item }` typecheck when the collection element type is known.
3. `Observe` bindings must not silently satisfy typed workflows by leaking unconstrained fresh variables into declared return-type checking when no honest observed-value type is available in the current MVP path.
4. `Propose.binding` must be handled honestly in the MVP:
   - either fully propagated end-to-end with real semantics, or
   - explicitly rejected as unsupported.
   This task adopts the conservative MVP choice: explicitly reject surfaced `Propose.binding` until result-type semantics exist.
5. Workflow-side interface-call validation and declared return-type checking must agree on binding visibility and scope for the covered forms.
6. Add focused regression tests for all of the above.

### Non-Functional Requirements

1. Do not widen the closed-world interface MVP beyond the frozen TASK-415/TASK-422 contract.
2. Do not add dynamic dispatch, associated items, or capability/interface unification.
3. Do not silently infer `Observe`/`Propose` result types from fabricated fresh variables when no authoritative source exists.
4. Keep diagnostics explicit when a binding form is unsupported or lacks enough type information.

## Files

- Modify: `crates/ash-typeck/src/lib.rs`
- Modify: `crates/ash-typeck/src/names.rs`
- Modify: `crates/ash-typeck/src/check_expr.rs` if needed for parity with workflow binding semantics
- Add/modify tests under `crates/ash-typeck/tests/`
- Modify task/docs status when complete

## TDD Steps

### Step 1: Write failing focused tests

Add task-specific tests covering:
- `For`-bound canonical interface calls that should succeed when the collection element type is known
- `For`-bound declared return inference that should succeed when the loop body returns the bound item
- `Observe` bindings that currently appear typeable only because of fabricated fresh variables and should instead fail honestly until result typing exists
- `Propose.binding` being explicitly rejected rather than silently ignored

### Step 2: Implement honest binding propagation

Teach workflow-side validation / return inference to derive and thread `For` element types correctly, and remove accidental fresh-variable leakage from `Observe`-driven declared return checking.

### Step 3: Implement conservative `Propose.binding` handling

Reject surfaced `Propose.binding` explicitly in the MVP unless full result-type semantics are introduced within this task.

### Step 4: Verify affected crate quality

Run at least:
- `cargo test -p ash-typeck --test workflow_binding_paths_task_423`
- `cargo test -p ash-typeck`
- `cargo clippy -p ash-typeck --all-targets -- -D warnings`
- `cargo fmt --check`

## Completion Checklist

- [x] `For` binding types propagate into interface-call validation
- [x] `For` binding types propagate into declared return inference
- [x] `Observe` typed uses fail honestly when result typing is unavailable
- [x] `Propose.binding` is handled honestly (explicit rejection in MVP)
- [x] focused regression tests added/updated
- [x] full `ash-typeck` verification passes

## Notes

This is intentionally a narrow follow-up rather than a reopening of the interface MVP. If later work wants typed `Observe` / `Propose` results, that should come from explicit capability/action result typing rather than fresh-variable fabrication.