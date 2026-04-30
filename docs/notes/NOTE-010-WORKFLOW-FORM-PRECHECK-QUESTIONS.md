# NOTE-010: Workflow Form Pre-Typecheck Question Backlog

**Status:** Exploratory / Q&A backlog with initial decisions
**Date:** 2026-04-29
**Related:** [DESIGN-033](../design/DESIGN-033-WORKFLOW-CONTRACT-OPERATOR-LIFTING.md), [SPEC-056](../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md), [PLAN-104](../plan/PLAN-104-FIRST-CLASS-WORKFLOW-CARRIER.md)

## Purpose

This note preserves the open questions that must be answered before the first-class workflow carrier design can be hardened into implementation-grade pre-typecheck, constraint-checking, and verification work.

Use this note as a future guided Q&A backlog: ask one question at a time, discuss the answer, record the decision, then move to the next question. The order below is intentionally dependency-aware; earlier answers constrain later ones.

## Current Working Model

The current first-principles model is:

```text
Workflow<A> = Proc<A> × Contract<A> × AlignmentEvidence
```

or, in current spec terminology:

```text
Workflow<A> = Proc<A> × WorkflowContract<A> × AlignmentEvidence
WorkflowContract<A> = AdmissionEnvelope + ContractPlan<A>
```

The product is synchronized, not a pair of unrelated lowered values. A workflow expression lowers to one preserved compositional form:

```text
Surface workflow syntax -> WorkflowForm<A>
```

and the Proc, Contract, check, resource, failure, reporting, and provenance dimensions are projections over that same form. The reference interpreter is a zipper over aligned projected steps/events.

Static discharge, dynamic residualization, simplification, no-op erasure, and proof search are later typechecking/verification/lowering concerns. Pre-typecheck lowering must preserve structure.

## Initial Decision Pass: 2026-04-30

The following decisions are now promoted into SPEC-056 and should be treated as the current baseline for Phase 108 planning:

1. `WorkflowForm` is primary. Carrier records such as `{ body, contract, evidence }` are implementation views derived from the preserved form, not independent artifacts.
2. First-slice `WorkflowForm` grammar is `Unit`, `Bind`, `FromProc`, `FromAct`, `Requires`, `Ensures`, and `Scope`. `Then` is derived from `Bind`; `Fail` and `WithError` remain inherited lower failure behavior for now.
3. Stable alignment uses `WorkflowNodeId`, `ProjectionKind`, `ProjectionEvent`, and `AlignmentKey = WorkflowNodeId × ProjectionKind`. Events carry source-origin metadata for source spans, synthetic desugarings, and imported summaries.
4. `ContractPlan<A>` is staged and aligned with `WorkflowForm`, including dependent `BindContract` frames and lower Proc/Act summary nodes.
5. `requires` is classified into role, capability, resource, precondition, and policy requirements. It may refine continuation checking context, but it never manufactures authority; final coverage/admission must prove the refined assumption.
6. `ensures` targets the successful result boundary of the suffix workflow. In `Bind(Ensures(Q), _, rest : Workflow<A>)`, `Q` is checked under `result : A`; if `rest` fails before producing `A`, failure projection handles that path instead.
7. `workflow::from_proc` and `workflow::from_act` preserve lower summaries and emit delayed coverage obligations. They do not require immediate empty-header coverage at the local expression site when an enclosing/composed workflow contract can cover the obligation.
8. Typechecking hands off staged contract and obligations using `Γ ⊢ᴡ form : Workflow<A> ▷ C, Ω`; coverage/verification later discharges `Ω` into `CoverageEvidence` or diagnostics.
9. `do:Workflow` and `[...]: Workflow` lower through the same workflow-form builder. Comprehensions first normalize through the SPEC-055 do path.
10. Equality has strata: `WorkflowForm` equality preserves governance nodes; Proc-projection equality may see them as neutral; optimized runtime equality may erase only after evidence is preserved.
11. First-slice workflow contract syntax uses legacy-compatible colon forms inside `do:Workflow`: `requires: expr;` and `ensures: expr;`. Direct calls to `workflow::requires(expr)` and `workflow::ensures(expr)` are compiler-known intrinsic elaborations, not evidence that `Requirement` or `OpenPostcondition` are ordinary first-class value types.
12. The current workflow declaration surface remains accepted but becomes deprecated. It must warn, then translate its role/capability/resource/`requires:`/`ensures:` clauses and body into the same `WorkflowForm` path used by first-class workflow expressions.

Remaining questions below are retained as refinement prompts. Items already answered by this decision pass should be read as historical context unless they expose additional implementation details not yet covered by SPEC-056.

## Q&A Method

For each question:

1. Restate the question in concrete Ash examples if possible.
2. Decide the semantic answer, not the implementation shortcut.
3. Record whether the answer belongs in:
   - workflow algebra / pre-typecheck lowering;
   - type/constraint checking;
   - verification;
   - runtime lowering;
   - modular export/import summaries.
4. Patch DESIGN-033 / SPEC-056 / PLAN-104 / task files only after the local decision is stable.

## Ordered Question Backlog

### 1. What is the core `WorkflowForm` grammar?

We need a closed first-slice grammar for the structure-preserving workflow form.

Candidate forms:

```text
WorkflowForm<A> ::=
    Unit(a)
  | Bind(w, binder, k)
  | Then(w1, w2)
  | FromProc(p)
  | FromAct(a)
  | Requires(R)
  | Ensures(Q)
  | Scope(name, w)
  | Fail(e)
  | WithError(w, handlers)
```

Questions:

- Which forms are in the first slice?
- Are `Then` and expression statements primitive forms or syntax for `Bind(_, _)`?
- Are `Fail` and `WithError` workflow forms or inherited lower forms projected from Proc/Act?
- Is `Scope` a workflow form or only declaration/export metadata?

Decision target:

```text
WorkflowForm first-slice grammar = ...
```

### 2. What is a projection event?

We need event vocabularies for each projection.

Questions:

- Does each workflow-form node emit exactly one event per projection, or zero-or-more?
- Are events a linear stream, tree-shaped stream, or graph over `WorkflowNodeId`s?
- What are the minimum event kinds for:
  - Proc projection;
  - Contract projection;
  - check projection;
  - authority/resource projection;
  - failure projection;
  - reporting/provenance projection?

Candidate shape:

```text
ProjectionEvent = {
  node      : WorkflowNodeId,
  projection: ProjectionKind,
  kind      : EventKind,
  span      : SourceSpan,
}
```

Decision target:

```text
Projection events are represented as ...
```

### 3. What is the alignment identity model?

The zipper depends on stable position identity.

Questions:

- What is the stable identity of a workflow-form position?
- Is position source-span-based, AST-node-based, lowered-node-based, or synthesized?
- How are positions preserved through desugaring?
- How do positions survive imported workflow summaries without exposing private source internals?
- How do workflow-form positions relate to runtime identities such as `WorkflowId`, `ProcessId`, `EffectScopeId`, and provenance event ids?

Candidate terms:

```text
WorkflowNodeId
ProjectionEventId
AlignmentKey = WorkflowNodeId + ProjectionKind
```

Decision target:

```text
Workflow position/alignment identity = ...
```

### 4. What is `Contract<A>` / `WorkflowContract<A>` as a staged algebra?

We currently use `WorkflowContract<A> = AdmissionEnvelope + ContractPlan<A>`, but `ContractPlan<A>` needs constructors.

Questions:

- Is `ContractPlan<A>` a free algebra over contract events?
- Is it a product of component plans?
- Is it a staged/dependent tree mirroring `WorkflowForm`?
- Which constructors exist in the first slice?

Candidate operations:

```text
contract::unit      : A -> Contract<A>
contract::bind      : Contract<A> -> (A -> Contract<B>) -> Contract<B>
contract::then      : Contract<A> -> Contract<B> -> Contract<B>
contract::requires  : Requirement -> Contract<Unit>
contract::ensures   : OpenPostcondition -> Contract<Unit>
contract::from_proc : ProcSummary<A> -> Contract<A>
contract::from_act  : ActSummary<A> -> Contract<A>
contract::scope     : Name -> Contract<A> -> Contract<A>
```

Decision target:

```text
ContractPlan first-slice constructors and algebra = ...
```

### 5. What exactly is `requires`?

`requires` may cover several semantic categories.

Examples:

```ash
requires: role(analyst);
requires: input_size > 0;
```

Questions:

- Is there one `Requirement` type or several typed requirement classes?
- Does `requires` only check an existing fact, or can it refine/introduce availability in the following environment?
- If `requires: role(admin);` appears before `workflow::from_proc(admin_proc)`, does the requirement make admin availability provisional for checking the continuation, or merely assert it should already be available?
- Which projections does each requirement kind produce?

Candidate classification:

```text
Requirement ::=
    RoleRequirement(RoleRef)
  | CapabilityRequirement(CapabilityRef, Mode)
  | ResourceRequirement(ResourceRef, AccessMode)
  | Precondition(CheckExpr)
  | PolicyRequirement(PolicyRef)
```

Decision target:

```text
requires projection and environment-refinement semantics = ...
```

### 6. What exactly is `ensures` and what does it target?

`workflow::ensures : OpenPostcondition -> Workflow<Unit>` preserves an open postcondition event, but its target must be resolved by contract bind / zipper semantics.

Example:

```ash
ensures: result.valid;
x <- w;
f(x)
```

Questions:

- Does `result` refer to the whole continuation after the `ensures` event?
- Does it refer to the immediately following workflow segment?
- Does it refer to the nearest enclosing workflow declaration result?
- Is target resolution specified by `contract::bind(Ensures(Q), λ_. C_rest)`?
- What happens with nested `ensures`?
- What happens if the body fails before producing a result?

Candidate rule:

```text
contract::bind(contract::ensures(Q), λ_. C_rest : Contract<A>)
  resolves Q under result : A
  and attaches Q to the successful result boundary of C_rest.
```

Decision target:

```text
ensures target and result-binder semantics = ...
```

### 7. How does `contract::bind` represent dependent/staged contracts?

Workflow bind can depend on runtime values:

```ash
x <- fetch_user(id);
requires: x.active;
process(x)
```

Questions:

- Does `Contract<A>` store symbolic bind frames?
- Are bound values represented as symbolic variables in contract plans?
- How are value-dependent requirements summarized for admission?
- When is a static upper bound required?
- What is allowed before dynamic admission exists?

Candidate shape:

```text
BindContract(C1, binder x, C2(x))
```

Decision target:

```text
contract::bind dependent representation = ...
```

### 8. What is the abstract zipper transition state?

The zipper aligns events, but we need a symbolic transition model.

Questions:

- What state does the pre-typecheck zipper carry?
- Does the zipper generate obligations, mutate an environment, build evidence, or all of these?
- Which obligations are emitted by `requires` / `ensures`?

Candidate state:

```text
ZipState = {
  lexical_env,
  workflow_env,
  obligation_set,
  projected_contract,
  alignment_evidence,
  diagnostics,
}
```

Candidate transition:

```text
zip_step(Requires(R)):
  add Obligation::Precondition(position, R)
  add FailureRoute::PreconditionFailure(position, R)
  add ProvenanceObligation::Requirement(position, R)
```

Decision target:

```text
zipper transition model = ...
```

### 9. What is the workflow-block grammar before typechecking?

We need exact grammar/lowering boundaries for blocks.

Questions:

- Are `requires` / `ensures` valid in any `do:Workflow`, or only workflow declarations?
- Can users write `_ <- workflow::requires(R);` explicitly?
- Is `requires: R;` always sugar for `_ <- workflow::requires(R);`?
- Are bare workflow expression statements allowed, or only explicit `_ <- action;`?
- Should `workflow_1(input);` require `Workflow<Unit>` or allow discarding `Workflow<A>`?

Decision target:

```text
Workflow block statement forms and lowering = ...
```

### 10. How are contract names resolved?

Examples:

```ash
requires: role(analyst);
requires: input_size > 0;
ensures: result.valid;
```

Questions:

- Are `role(name)` and legacy capability/resource contract forms ordinary lexical/module values, helper syntax, or symbolic references?
- Are they special contract namespace paths?
- Are roles/capabilities/resources first-class values or symbolic references?
- Is `result` a special open binder or an ordinary unresolved name?
- Which lexical names are visible in `requires` and `ensures` expressions?

Decision target:

```text
contract-expression name resolution = ...
```

### 11. How should existing workflow header syntax be represented?

Even if compatibility is not the design driver, the parser must preserve source order.

Questions:

- Are old header clauses parsed into the same workflow statement list as computation statements?
- Are `requires` / `ensures` allowed after computation statements?
- Does source order matter for all contract injection forms?
- Is there any remaining semantic distinction between declaration metadata and contract events?

Decision target:

```text
workflow declaration block representation = ...
```

### 12. What is `OpenPostcondition`?

`workflow::ensures : OpenPostcondition -> Workflow<Unit>` requires a non-ordinary expression object.

Questions:

- Is `OpenPostcondition` an AST with a distinguished `result` binder?
- Is it typechecked only after a continuation target type is known?
- Can it mention workflow parameters and local variables?
- Can it call pure functions?
- Can it mention effects/capabilities/resources?

Candidate shape:

```text
OpenPostcondition = {
  binder: result,
  expr: SurfaceExpr,
  captured_lexical_scope: LexicalScopeId,
}
```

Decision target:

```text
OpenPostcondition representation and typing boundary = ...
```

### 13. What is exported for imported workflows?

Downstream workflow composition needs contract information, but private bodies should not necessarily be exported.

Questions:

- Does an exported workflow expose a full `WorkflowForm`?
- Does it expose a staged public contract summary?
- How much position/alignment information must survive module boundaries?
- How are private internal node ids hidden while preserving useful diagnostics?

Candidate answer:

```text
Export a public `WorkflowContractSummary<A>` that preserves staged public contract structure and alignment boundaries, but not private body internals.
```

Decision target:

```text
workflow export/import summary format = ...
```

### 14. Is `Scope(name, w)` a workflow form?

Declaration scope likely affects more than contract metadata.

Questions:

- Does `Scope` create a `WorkflowId` / boundary identity?
- Does it affect failure routing?
- Does it affect report/provenance projections?
- How are nested subworkflow scopes represented?
- Is `Scope` present in expression-built workflows, declarations, or both?

Decision target:

```text
Scope form/projection semantics = ...
```

### 15. What is the minimum failure event vocabulary?

We mention failure projection, but not the first-slice event kinds.

Questions:

- What failure kind does a failed `requires` produce?
- What failure kind does a failed `ensures` produce?
- How do lower Proc/Act failures project into workflow failure events?
- Which failure events are introduced before verification versus after residualization?

Candidate kinds:

```text
FailureEvent ::=
    None
  | PreconditionFailure(position, requirement)
  | PostconditionFailure(position, postcondition)
  | LowerProcFailure(position, lower_failure)
  | CoverageFailure(position, evidence_gap)
  | AdmissionFailure(position, reason)
```

Decision target:

```text
first-slice failure event vocabulary = ...
```

### 16. How do workflow requirements project into Act/Proc environments?

Workflow governance must constrain lower environments without manufacturing authority.

Questions:

- How does a workflow requirement constrain `EffEnv` / `ProcEnv`?
- What information must `from_act` / `from_proc` preserve about lower authority/resource needs?
- How is no-authority-widening represented before verification?
- Are requirements local gates, environment refinements, or both?

Decision target:

```text
workflow-to-proc/effect environment projection model = ...
```

### 17. What obligation/constraint classes does pre-typecheck lowering emit?

The workflow algebra should generate obligations but not solve them.

Candidate obligation classes:

```text
PreconditionMustHold(position, R)
OpenPostconditionTarget(position, Q, target_type_hole)
ContractGovernsProc(position, contract_event, proc_event)
RequiredCapabilityCovered(position, capability)
ResourceAvailable(position, resource, mode)
FailureRouteDefined(position, failure_kind)
ProvenanceRecordable(position, event)
```

Questions:

- Which obligations are emitted during workflow-form lowering?
- Which obligations are emitted during typechecking?
- Which are verification-only?
- Which must survive into runtime lowering if not discharged?

Decision target:

```text
pre-typecheck obligation vocabulary = ...
```

### 18. How are requirement/postcondition projections computed?

`requires(R)` and `ensures(Q)` are polymorphic over sub-projections.

Questions:

- Is there a `projectRequirement` function?
- Is there a `projectPostcondition` function?
- Does classification require type information, or can it be syntactic first?
- How are ambiguous requirements represented before typechecking?

Candidate functions:

```text
projectRequirement : Requirement -> ProjectionEvents
projectPostcondition : OpenPostcondition -> ProjectionEvents
```

Decision target:

```text
requirement/postcondition projection classification = ...
```

### 19. What equivalences are valid before optimization?

We must avoid erasing nodes with neutral Proc projections too early.

Questions:

- What is definitional equality for `WorkflowForm`?
- What is equality for the Proc projection only?
- What is equality for Contract projection?
- Are Monad laws valid at the workflow-form level or only up to projection equivalence?
- Is `bind(requires(R), λ_. w)` ever definitionally equal to `w`? It should not be at `WorkflowForm` level.

Decision target:

```text
WorkflowForm/projection/optimized equality strata = ...
```

### 20. What source-origin metadata must every projected event carry?

Later diagnostics need precise origins.

Questions:

- Do projected events carry source span, workflow node id, and projection kind?
- How are spans attached to synthetic desugarings?
- How are imported summary events diagnosed?
- How are source positions preserved through normalization?

Decision target:

```text
Projection event source-origin metadata = ...
```

### 21. How does the first-slice zipper avoid blocking future parallel forms?

The first slice is sequential, but the model should not assume a forever-linear stream.

Questions:

- Is the zipper over a linear stream, tree, or general structured form?
- How should the first slice phrase “stream” to avoid excluding future `par` / `scatter` forms?
- What minimal branch markers should exist now, if any?

Decision target:

```text
zipper structure model for sequential-first but parallel-compatible forms = ...
```

### 22. Which existing PLAN/TASK docs need realignment?

SPEC-056 has moved toward `WorkflowForm`, projection events, and contract-injection forms.

Likely updates:

- TASK-769 owns `WorkflowForm`, node ids, projections, staged `ContractPlan`, obligation handoff, non-denotable contract argument semantics, WorkflowForm-preserving typed-do artifacts, the legacy-body adapter contract, first-slice contract syntax decisions, and equality strata.
- TASK-770 owns `requires:` / `ensures:` do-statement parsing, source-ordered `WorkflowHeaderEvent`s, conservative contract-expression name resolution, the classifier mapping table, and implemented legacy-compatible role semantics such as `any_role` OR.
- TASK-771 owns public `Workflow<A>`, shared `ash-core` carriers derived from `WorkflowForm`, qualified compiler-known workflow operations, and non-denotable intrinsic parameter classes for contract arguments.
- TASK-772 consumes TASK-770's contract-injection statement syntax through a WorkflowForm-preserving typed-do artifact.
- TASK-773 owns WorkflowForm-aware ordinary expression elaboration for all compiler-known first-slice `workflow::...` algebra/contract calls after TASK-771/TASK-772 exist.
- TASK-774 owns executable Workflow lowering/runtime projection through existing Proc/workflow boundaries and proves contract-injection metadata is not dead.
- TASK-775 owns deprecated legacy declaration warning plumbing, source-ordered header translation, and `legacy_body_as_proc_summary` compatibility lowering.
- TASK-778 includes preservation/no-erasure diagnostics/tests plus legacy-declaration deprecation warnings.

Decision target:

```text
PLAN-104 / TASK-769..779 realignment patch scope = split/reorder implementation tasks around parser/classifier/header events, workflow type/intrinsic parameters, WorkflowForm-preserving do, intrinsic call elaboration, executable runtime projection, legacy translation, modules, diagnostics, and closeout
```

### 23. Which reusable skills/memories need drift correction?

The `ash-semantic-tower-environment-modeling` skill now includes the zipper/projection model, but still contains older header/body reconciliation wording as a primary-looking rule.

Questions:

- Should the skill distinguish primary model vs legacy/transitional model?
- Should a new focused skill be created for workflow-form zipper design, or should the existing semantic-tower skill absorb it?
- Which memory entries should be updated after the Q&A decisions stabilize?

Decision target:

```text
Reusable knowledge update scope = ...
```

## Suggested Discussion Order

Use this order for a future session:

```text
1. WorkflowForm grammar
2. Projection event vocabulary
3. Position/alignment identity
4. ContractPlan staged algebra
5. requires semantics
6. ensures semantics
7. contract::bind dependency/staging
8. zipper transition state
9. workflow contract syntax and block lowering
10. contract name resolution
11. deprecated workflow declaration/header translation
12. Workflow runtime projection boundary
13. OpenPostcondition representation
14. module export/import summaries
15. Scope form semantics
16. failure event vocabulary
17. environment projection to Proc/Act
18. obligation/constraint vocabulary
19. requirement/postcondition projection classification
20. equality/normalization strata
21. source-origin metadata
22. parallel-compatible zipper structure
23. PLAN/TASK realignment
24. skill/memory drift correction
```

## Stop Conditions for the Q&A Pass

The Q&A pass is complete when we can write, without open semantic holes:

```text
WorkflowForm grammar
ProjectionEvent model
Alignment identity model
ContractPlan first-slice algebra
requires / ensures projection rules
contract::bind staging rules
zipper transition rules
workflow contract syntax / block lowering / legacy declaration translation rules
obligation vocabulary handoff to typechecking/verification
```

At that point, the next artifact should be either:

1. a patch to SPEC-056 turning these decisions into normative sections, or
2. a narrower follow-on spec for workflow-form/projection/zipper pre-typecheck semantics.
