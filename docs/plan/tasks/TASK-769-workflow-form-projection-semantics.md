# TASK-769: Workflow Form, Projection, and Obligation Semantics

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [DESIGN-033](../../design/DESIGN-033-WORKFLOW-CONTRACT-OPERATOR-LIFTING.md)
- [NOTE-010](../../notes/NOTE-010-WORKFLOW-FORM-PRECHECK-QUESTIONS.md)
- [SPEC-050](../../spec/SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md)
- [SPEC-051](../../spec/SPEC-051-WORKFLOW-SEMANTICS.md)
- [SPEC-053](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md)
- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md)
- [SPEC-055](../../spec/SPEC-055-MONAD-COMPREHENSION-SYNTAX.md)

## Objective

Add the semantic gate that must precede first-class workflow implementation: a structure-preserving `WorkflowForm` grammar, stable node/alignment identities, projection-event model, staged `ContractPlan` algebra, obligation vocabulary, and typed-do/comprehension lowering rules. This task prevents Phase 108 from implementing `Workflow<A>` as either dead Proc metadata or a second hidden workflow calculus.

## Requirements

1. Define the first-slice `WorkflowForm<A>` grammar as the source of truth for workflow semantics:
   - `Unit(expr)`
   - `Bind(form, binder, form)`
   - `FromProc(proc_term)`
   - `FromAct(act_term)`
   - `Requires(requirement)`
   - `Ensures(open_postcondition)`
   - `Scope(scope, form)`
2. Treat `Then(w1, w2)` as derived syntax for `Bind(w1, _, w2)`, not as a required primitive form.
3. Do not add first-slice primitive `Fail` or `WithError` workflow nodes unless implementation proves workflow-specific failure routing cannot be represented through lower Proc/failure projections.
4. Define stable `WorkflowNodeId`, `ProjectionKind`, `ProjectionEvent`, and `AlignmentKey` carriers. Projection events must carry source-origin metadata and must support imported-summary origins.
5. Define first-slice projections for Proc, contract, checks, authority/resources, failure, reporting, and provenance as aligned events over `WorkflowNodeId`s.
6. Define `ContractPlan<A>` as a staged tree over the same workflow form, including dependent `BindContract(C1, binder, C2)` frames.
7. Define `requires` classification and refinement semantics:
   - role, capability, resource, precondition, and policy requirements;
   - requirements may refine the continuation checking environment;
   - requirements must never manufacture authority;
   - final coverage/admission must prove every refined assumption.
8. Define `ensures` target semantics: `Bind(Ensures(Q), _, rest : Workflow<A>)` attaches `Q` to the successful result boundary of `rest`, typechecking `Q` under `result : A`.
9. Define `workflow::from_proc` and `workflow::from_act` as obligation-producing nodes. They must preserve lower Proc/Act contract summaries and emit coverage obligations instead of requiring immediate empty-header coverage at the local expression site.
10. Define the type/constraint handoff judgment:

    ```text
    Γ ⊢ᴡ form : Workflow<A> ▷ C, Ω
    ```

    where `C` is a staged `ContractPlan<A>` and `Ω` is an obligation set to be discharged later into `CoverageEvidence` or diagnostics.
11. Define first-slice obligation classes, including requirement coverage, open postcondition target resolution, lower Proc coverage, required capability/resource coverage, failure route definition, provenance recordability, and opaque imported-summary rejection.
12. Define equality strata:
    - `WorkflowForm` equality is source/projection-preserving and does not erase contract-injection nodes;
    - Proc-projection equality may treat `Requires`/`Ensures` as neutral in the Proc dimension;
    - runtime optimization equality may erase discharged neutral executable nodes only while preserving evidence.
13. Define lowering from `do:Workflow` statements and `[...]: Workflow` comprehensions into `WorkflowForm`; comprehensions must first normalize through SPEC-055's do path and then use the same workflow-form builder.
14. Record the decisions in SPEC-056 and realign NOTE-010 so the remaining questions are follow-up refinements, not blockers for implementation.

## TDD / Documentation Steps

1. Patch SPEC-056 with normative sections for `WorkflowForm`, projection events, staged contract plans, obligations, `requires`/`ensures`, delayed `from_proc` coverage, and equality strata.
2. Patch NOTE-010 with a decision summary for questions 1-8 and 17-20, plus any remaining open follow-ups.
3. Patch PLAN-104 and downstream tasks so implementation begins only after this semantic gate.
4. Add or update task requirements for TASK-770 through TASK-774 to depend on the workflow-form/projection model.
5. Run documentation checks (`git diff --check`, link/path existence checks, and task structural checks).
6. Request independent review before TASK-770 begins implementation.

## Verification

- [ ] SPEC-056 contains a closed first-slice `WorkflowForm` grammar.
- [ ] SPEC-056 defines projection event identity and source-origin metadata.
- [ ] SPEC-056 defines staged `ContractPlan` / obligation handoff instead of local eager coverage solving.
- [ ] SPEC-056 states `requires` may refine checking context but cannot manufacture authority.
- [ ] SPEC-056 states `ensures` targets the successful result boundary of the suffix workflow.
- [ ] SPEC-056 states `from_proc` / `from_act` emit coverage obligations and do not require immediate `EmptyHeader` coverage at the expression site.
- [ ] PLAN-104 marks TASK-769 as a blocking semantic gate before carrier/type/library implementation.
- [ ] TASK-770 through TASK-774 reference the semantic gate where relevant.
- [ ] `git diff --check` passes.
- [ ] Markdown links to renamed TASK-769 path resolve.
- [ ] CHANGELOG.md updated.
