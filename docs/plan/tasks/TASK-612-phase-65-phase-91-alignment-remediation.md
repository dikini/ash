# TASK-612: Phase 65 ↔ Phase 91 Alignment Remediation

## Status: ✅ Complete

## Description

Close the remaining alignment gaps between the accepted Phase 65 contracts and the delivered Phase 91 substrate. This is a narrow remediation task, not a feature expansion: fix two malformed parser acceptances, restore the documented TASK-423 contract for `Propose.binding`, and reconcile stale `RuntimeError` task/docs surfaces so Phase 91's production-quality claims remain honest.

## Specification Reference

- [PLAN-091: Small-Step and Statement-Lifting Productionization](../PLAN-091-SMALL-STEP-LIFTING-PRODUCTIONIZATION.md)
- [TASK-418: Tuple Variant Runtime Support and RuntimeError Reconciliation](TASK-418-tuple-variant-runtime-and-entry-contract-reconciliation.md)
- [TASK-419: Effect Inference and Runtime Verification Alignment](TASK-419-effect-inference-and-runtime-verification-alignment.md)
- [TASK-420: Pure Bottom-Effect Follow-On](TASK-420-pure-bottom-effect-follow-on-decision.md)
- [TASK-421: Closed-World Interfaces AST and Parser Substrate](TASK-421-closed-world-interfaces-ast-and-parser-substrate.md)
- [TASK-422: Closed-World Interfaces Coherence and Method Resolution](TASK-422-closed-world-interfaces-coherence-and-method-resolution.md)
- [TASK-423: Workflow Binding Propagation and Honest Unsupported Bindings](TASK-423-workflow-binding-propagation-and-honest-unsupported-bindings.md)
- `docs/spec/SPEC-002-SURFACE.md`
- `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- `docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md`

## Dependencies

- ✅ TASK-418 complete
- ✅ TASK-421 complete
- ✅ TASK-422 complete
- ✅ TASK-423 complete (but requires remediation to match its documented contract)
- ✅ TASK-608 complete
- ✅ TASK-609 complete
- ✅ TASK-610 complete

## Requirements

### Functional Requirements

1. Reject bare qualified method syntax (`Interface::method`) when it is not followed by call parentheses.
2. Reject lowercase tuple/record pseudo-constructors as variant patterns (for example `foo(bar)` and `foo { x: y }`).
3. Restore TASK-423’s documented MVP contract by explicitly rejecting surfaced `Propose.binding` until real result semantics exist.
4. Ensure workflow-side interface-call validation and declared return checking agree on `Propose.binding` being unsupported in the MVP.
5. Reconcile active task/docs surfaces so `RuntimeError` no longer appears as a record-shaped canonical current contract where TASK-418 says tuple-variant reconciliation is complete.
6. Preserve all existing valid uppercase tuple/unit/record variant behavior and valid qualified call parsing.

### Non-Functional Requirements

1. Do not widen interface syntax beyond the frozen Phase 65 MVP.
2. Do not implement real `Propose` result semantics in this task.
3. Do not add new effect lattice elements or reopen Phase 65/91 contracts.
4. Treat doc/task-state drift as a blocking bug and resolve it explicitly.
5. Prefer explicit rejection over fabricated typing or silent parser acceptance.

## Files

### Parser acceptance fixes
- Modify: `crates/ash-parser/src/parse_expr.rs`
- Modify: `crates/ash-parser/src/parse_pattern.rs`
- Modify/add parser regression tests near existing parser test locations

### TASK-423 contract remediation
- Modify: `crates/ash-typeck/src/lib.rs`
- Modify any nearby workflow validation helpers if needed
- Modify: `crates/ash-typeck/tests/workflow_binding_paths_task_423.rs`

### RuntimeError corpus reconciliation
- Modify: `docs/plan/tasks/TASK-359-stdlib-initialization.md`
- Modify: `docs/plan/tasks/TASK-360-runtime-error-type.md`
- Modify: `docs/plan/tasks/TASK-362-system-supervisor.md`
- Modify: `docs/plan/tasks/TASK-369-phase-57-closeout.md`
- Inspect/update any still-active related task/docs surfaces that present record-shaped `RuntimeError` as current truth

### Planning/index tracking
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md` if project policy requires documenting this alignment remediation

## TDD Steps

### Step 1: Parser regression tests (Red)

Add tests proving:
- `Interface::method` is rejected when no `(` follows
- `Interface::method(x)` still parses correctly
- lowercase tuple pseudo-variant patterns are rejected
- lowercase record pseudo-variant patterns are rejected
- uppercase tuple/record/unit variant patterns still parse correctly

### Step 2: Parser fixes (Green)

Implement the smallest parser changes needed so invalid qualified names and lowercase pseudo-variants are no longer silently accepted.

### Step 3: TASK-423 contract tests (Red)

Replace the current acceptance test for `Propose.binding` with a rejection test that asserts an explicit unsupported MVP diagnostic.

### Step 4: Typechecker remediation (Green)

Remove the fresh-type-variable acceptance path for `Propose.binding`, explicitly reject it in MVP workflow validation/typechecking, and ensure declared return inference does not silently diverge from validation behavior.

### Step 5: Corpus reconciliation (Green)

Update stale task/docs surfaces so no still-active/still-complete document presents `RuntimeError { exit_code, message }` as the canonical current contract. Where historical context is needed, mark it as superseded by TASK-413/TASK-418.

### Step 6: Final verification

Run targeted and workspace verification and confirm the new task/docs state is reflected honestly in PLAN-INDEX.

## Verification Steps

### Targeted parser/typechecker/runtime checks
- [x] `cargo test -p ash-parser`
- [x] `cargo test -p ash-typeck --test workflow_binding_paths_task_423`
- [x] `cargo test -p ash-typeck --test closed_world_interfaces_task_422`
- [x] `cargo test -p ash-typeck`
- [x] `cargo test -p ash-interp --test adt_contracts`

### Workspace quality gates
- [x] `cargo test --workspace`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo fmt --check`

### Corpus verification
- [x] Search confirms no still-active/current task doc presents record-shaped `RuntimeError` as canonical current syntax
- [x] PLAN-INDEX reflects this follow-on remediation honestly

## Completion Checklist

- [x] Bare `Interface::method` is rejected without call syntax
- [x] Lowercase tuple pseudo-variant patterns are rejected
- [x] Lowercase record pseudo-variant patterns are rejected
- [x] `Propose.binding` is explicitly rejected in MVP typechecking
- [x] TASK-423 regression tests assert rejection rather than fresh-var acceptance
- [x] `RuntimeError` task/docs corpus is reconciled to the tuple-variant contract or marked superseded explicitly
- [x] Targeted tests pass
- [x] Workspace verification passes
- [x] PLAN-INDEX updated

## Dependencies for Next Task

This task outputs:
- honest parser acceptance boundaries for the closed-world interface and tuple-variant surfaces
- restored TASK-423 MVP honesty for `Propose.binding`
- reconciled task/docs corpus for `RuntimeError`

Required by:
- any future production-readiness or phase-close claims that depend on Phase 91 being fully aligned with prior frozen contracts

## Notes

This task is intentionally remediation-only. It should not introduce new language features, broaden closed-world interfaces, or implement real `Propose` result semantics. The correct outcome is the smallest set of parser/typechecker/doc fixes needed to make the repo’s behavior and corpus match the accepted contract surfaces.