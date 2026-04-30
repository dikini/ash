# TASK-769: Workflow Form, Projection, Obligation, and Adapter Semantics

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [SPEC-051](../../spec/SPEC-051-WORKFLOW-SEMANTICS.md)
- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md)
- [PLAN-104](../PLAN-104-FIRST-CLASS-WORKFLOW-CARRIER.md)

## Objective

Validate and freeze SPEC-056's first-class workflow model as an implementation-grade semantic gate before Rust carrier work. This task does not re-create the already-landed draft packet; it confirms that the downstream implementation may rely on stable definitions for `WorkflowForm`, projection events, source-order legacy header events, staged contract plans, obligations, WorkflowForm-preserving typed-do artifacts, and the legacy-body adapter contract.

## Requirements

1. Define the first-slice `WorkflowForm` grammar: `Unit`, `Bind`, `FromProc`, `FromAct`, `Requires`, `Ensures`, and `Scope`. `Then` remains derived from `Bind`.
2. Define concrete schema requirements for `WorkflowNodeId`, `ProjectionKind`, `ProjectionEvent`, `AlignmentKey`, `SourceOrigin`, `ContractPlan`, `WorkflowObligation`, `CoverageEvidence`, and `WorkflowContractSummary`.
3. Define `WorkflowHeaderEvent` as the source-ordered compatibility carrier for deprecated workflow declarations. It must preserve spans and source order for `plays role(...)`, capability headers, `owns`, `uses`, `requires:`, and `ensures:` clauses while legacy convenience fields may remain derived.
4. Define `Requirement` coverage semantics, including implemented OR-role semantics for `any_role([...])`; do not defer accepted legacy-compatible role policy forms.
5. Define `OpenPostcondition` as a delayed postcondition with optional `result` binder. Unresolved/open `Ensures` nodes at workflow finalization must reject with a target diagnostic.
6. Define the non-denotable contract-argument model: `Requirement` and `OpenPostcondition` are semantic classifier products / intrinsic parameter classes, not ordinary Ash-denotable types.
7. Define the WorkflowForm-preserving typed-do artifact required by TASK-772. Workflow elaboration may expose a `CoreExpr`/Proc projection, but the `WorkflowForm` remains source of truth.
8. Define `legacy_body_as_proc_summary` adapter contract, including input, output summary fields, rejection cases, and lower Proc coverage obligations. The adapter must match legacy semantics and must not create a separate legacy runtime/typechecking path.
9. Define obligation generation and conservative Phase 108 discharge: unproven/opaque obligations are rejected, not silently accepted; full proof search/dynamic residualization remains later.
10. Define equality/normalization strata: structural/source-preserving form equality before coverage, projection equality only after alignment is preserved, and evidence-preserving optimization only after evidence exists.
11. Record all decisions in SPEC-056 and realign NOTE-010 so remaining questions are follow-up refinements, not implementation blockers.

## Implementation Notes / Target Files

This is a docs/spec gate. It should cite the live implementation surfaces that later tasks must touch:

- `crates/ash-parser/src/surface.rs`: `WorkflowDef`, `Contract`, `Requirement`, `EnsuresClause`, `DoStmt`.
- `crates/ash-parser/src/parse_workflow.rs`: `workflow_def`, `parse_plays_roles`, `parse_workflow_header_clauses`, `parse_opt_contract`.
- `crates/ash-typeck/src/check_expr.rs`: typed-do result and elaboration path.
- `crates/ash-core/src/workflow_contract.rs`: current `Contract`, `Requirement`, `PostPredicate`, `ArithConstraint`.

## TDD / Documentation Steps

1. Re-read SPEC-056 schema sections for all carriers named above and patch any remaining blocker before code tasks begin.
2. Confirm the classifier mapping table and adapter contract are implementation-grade and cite live substrate names where possible.
3. Confirm PLAN-104 task ordering still makes code tasks depend on this gate.
4. Confirm TASK-770 through TASK-779 references to this gate remain accurate after any task renumbering or split.
5. Run markdown link and task-structure verification.

## Verification

- [ ] SPEC-056 defines `WorkflowForm` and all carrier schemas required by downstream tasks.
- [ ] SPEC-056 defines source-ordered `WorkflowHeaderEvent` and states legacy aggregate fields are derived/compatibility views.
- [ ] SPEC-056 defines non-denotable intrinsic contract argument classes.
- [ ] SPEC-056 defines `any_role` OR semantics as implemented first-slice behavior.
- [ ] SPEC-056 defines WorkflowForm-preserving typed-do artifact requirements.
- [ ] SPEC-056 defines `legacy_body_as_proc_summary` adapter contract.
- [ ] PLAN-104 marks this task as a blocking semantic gate before implementation.
- [ ] `git diff --check` passes.
