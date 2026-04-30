# TASK-772: WorkflowForm-Preserving Workflow Do Target

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md)
- [TASK-770](TASK-770-workflow-contract-surface-classifier-and-header-events.md)
- [TASK-771](TASK-771-workflow-type-stdlib-and-intrinsic-parameters.md)

## Objective

Add `Workflow` as a typed-do target while preserving a `WorkflowForm` artifact as the semantic source of truth instead of lowering workflow do blocks directly to CoreExpr-only dictionary calls.

## Dependencies

- 📝 TASK-770: Workflow contract surface, classifier, and header events.
- 📝 TASK-771: Workflow type, qualified builtins, shared carriers, and intrinsic parameters.

## Requirements

1. Extend `crates/ash-typeck/src/do_target.rs` with `DoTowerLevel::Workflow` and `Workflow` target resolution.
2. Add a workflow `DoDictionary` using `workflow::unit` and `workflow::bind` without implicitly importing ordinary workflow operations into scope.
3. Replace or extend the current `DoElaborationResult { expr: CoreExpr, ty: Type }` shape in `crates/ash-typeck/src/check_expr.rs` with a representation capable of carrying `WorkflowTypedArtifact` for Workflow target.
4. Required artifact shape must include at least: `WorkflowForm`, projection events, `ContractPlan`, obligations, and source-origin/alignment metadata. A CoreExpr/Proc projection may be derived, but it is not the semantic source of truth for Workflow.
5. Check `let`, `<-`, `_ <-`, and final `return` with the same SPEC-054 rules. `<-` in `do:Workflow` requires `Workflow<A>` RHS whose value carries or references a live `WorkflowTypedArtifact` or public workflow summary; otherwise reject as opaque.
6. Lower `requires: R;` and `ensures: Q;` by invoking the contract classifier from TASK-770 and adding preserved `Requires` / `Ensures` WorkflowForm nodes.
7. Contract statement variants bypass ordinary value name resolution for role-list symbols and `result` postcondition binding; other lexical names in predicates resolve in the surrounding environment.
8. Reject contract statement variants in `do:Act` and `do:Proc` with workflow-only diagnostics.
9. Add diagnostics suggesting `workflow::from_proc` / `workflow::from_act` for wrong RHS tower.
10. Do not change existing `do:Act`, `do:Proc`, new-form `act { ... }`, or Act/Proc comprehension behavior.

## TDD Steps

1. Write failing tests for resolving `do:Workflow` target.
2. Write failing tests for `do:Workflow { return x }` producing `Workflow<A>` and a `Unit` WorkflowForm node.
3. Write failing tests for workflow `<-` producing nested binder-scoped `Bind` WorkflowForm nodes.
4. Write failing tests for `requires:` / `ensures:` nodes preserved in the WorkflowForm with projection events.
5. Write negative tests for `Proc<A>`/`Act<A>` RHS without explicit lifts.
6. Write negative tests for local/imported `Workflow<A>` RHS values that lack a live artifact or public summary.
7. Write regression tests proving `do:Act` and `do:Proc` still elaborate through their existing paths.
8. Implement target, artifact, checking, and elaboration changes.
9. Run focused `ash-typeck` tests.

## Verification

- [ ] `Workflow` resolves as a typed-do target.
- [ ] `do:Workflow` synthesizes `Workflow<A>` from final `return`.
- [ ] Workflow do elaboration preserves `WorkflowForm` / projection-event alignment.
- [ ] Workflow bind statements reject opaque Workflow values lacking a live artifact or public summary.
- [ ] `requires:` and `ensures:` statements produce preserved WorkflowForm nodes.
- [ ] Wrong tower RHS values produce explicit lift hints.
- [ ] Existing Act/Proc do behavior and tests do not regress.
- [ ] CHANGELOG.md updated.
