# SPEC-056: First-Class Workflow Carrier

**Status:** Draft
**Date:** 2026-04-29
**Promotes:** [DESIGN-033](../design/DESIGN-033-WORKFLOW-CONTRACT-OPERATOR-LIFTING.md)
**Builds on:** [SPEC-048](SPEC-048-PROC-LIBRARY.md), [SPEC-049](SPEC-049-PROCESS-RUNTIME-SEMANTICS.md), [SPEC-050](SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md), [SPEC-051](SPEC-051-WORKFLOW-SEMANTICS.md), [SPEC-054](SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md), [SPEC-055](SPEC-055-MONAD-COMPREHENSION-SYNTAX.md)
**Related:** [SPEC-003](SPEC-003-TYPE-SYSTEM.md), [SPEC-004](SPEC-004-SEMANTICS.md), [SPEC-047](SPEC-047-ACT-MONAD.md), [SPEC-052](SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md), [SPEC-053](SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md)
**Plan:** [PLAN-104](../plan/PLAN-104-FIRST-CLASS-WORKFLOW-CARRIER.md)
**Implementation Tasks:** [TASK-768](../plan/tasks/TASK-768-first-class-workflow-spec-plan-packet.md) through [TASK-775](../plan/tasks/TASK-775-first-class-workflow-closeout.md)

## 1. Summary

Ash adds `Workflow<A>` as a first-class computation constructor and monadic target.

The semantic model is:

```text
Workflow<A> = WorkflowContract<A> + Proc<A>
WorkflowContract<A> = AdmissionEnvelope + ContractPlan<A>
```

`Proc<A>` owns the process computation. `Workflow<A>` owns the governed version of that computation: admission envelope, staged contract plan, coverage evidence, failure/reporting boundary, and provenance obligations.

This spec defines the first implementation slice for first-class workflows:

1. `Workflow<A>` as a public type constructor.
2. A `workflow` library namespace with Monad-shaped operations analogous to `proc`.
3. Compiler-known `do:Workflow` target resolution through the existing SPEC-054 typed-do machinery.
4. `[...]: Workflow` comprehensions through the existing SPEC-055 comprehension machinery.
5. A blocking workflow-form/projection semantic gate: `WorkflowForm`, stable node/alignment identities, projection events, staged `ContractPlan`, obligation vocabulary, and equality strata.
6. Internal coverage/evidence carriers sufficient to prove that a workflow contract governs its body.
7. Structure-preserving workflow forms whose Proc and Contract projections are interpreted by a zipper over aligned events.
8. Sequential workflow composition first: `unit`, `bind`, `then`, explicit lifts from `Proc`/`Act`, and initial contract-injection forms such as `requires` and `ensures`.

This spec intentionally does not make parallel workflow handles, dynamic admission, or user-defined contract components part of the first implementation slice.

## 2. Motivation

SPEC-054 generalized typed do-notation over computation constructors, but deliberately limited MVP targets to `Act` and `Proc`. SPEC-055 then reused that substrate for explicit-target bracket comprehensions, again deferring `Workflow` as a target.

DESIGN-033 establishes the missing semantic model: a workflow is not a separate execution calculus above Proc. It is a contract-indexed process carrier.

Making `Workflow<A>` first-class gives Ash a uniform way to construct workflows as expressions of the workflow algebra:

```ash
do:Workflow {
    x <- prepare(input);
    y <- analyze(x);
    return y
}
```

and, by reuse of SPEC-055:

```ash
[finalize(y) | x <- prepare(input), y <- analyze(x)]: Workflow
```

Both forms are ordinary typed elaborations through the same Monad-shaped dictionary path. No parser-only workflow semantics are introduced.

## 3. Implementation Baseline

This spec assumes the following already exist:

1. `Proc<A>` and `P<A>` as public type constructors and `proc::unit`, `proc::bind`, `proc::then`, `proc::from_act`, `proc::par`, `proc::await`, `proc::join`, and `proc::gather` as library/builtin surfaces from SPEC-048/049.
2. Operational bottom and scoped handling from SPEC-050, including the rule that `fail` remains operational bottom rather than domain failure.
3. Workflow admission/reporting substrate from SPEC-051, including workflow/run identities, boundary outcomes, report metadata, and compatibility with existing workflow execution.
4. Capability/resource admission and provenance substrate from SPEC-052/053.
5. SPEC-054 typed do infrastructure: `DoTarget`, `DoStmt`, target resolution, builtin Act/Proc dictionaries, typed elaboration, and diagnostics.
6. SPEC-055 comprehension infrastructure: source-fidelity comprehension surface node, explicit postfix target annotation, and typed-do-based elaboration.

This spec adds `Workflow` to the computation-constructor target set. It must not fork the do/comprehension elaboration path.

## 4. Scope

In scope for the first implementation slice:

- Public type constructor `Workflow<A>`.
- Internal semantic carriers for `WorkflowContract<A>`, `AdmissionEnvelope`, `ContractPlan<A>`, and `CoverageEvidence` sufficient for sequential workflow composition.
- Structure-preserving `WorkflowForm` lowering with Proc/Contract/check/resource/failure/provenance projections.
- Public `workflow` namespace operations:
  - `workflow::unit`
  - `workflow::bind`
  - `workflow::then`
  - `workflow::from_proc`
  - `workflow::from_act`
  - `workflow::requires`
  - `workflow::ensures`
- Closed first-slice `WorkflowForm` grammar, projection-event vocabulary, alignment identity model, staged `ContractPlan`, obligation vocabulary, and equality strata before Rust carrier implementation begins.
- Compiler-known `Workflow` typed-do dictionary.
- `do:Workflow { ... }` typed elaboration.
- `[...]: Workflow` comprehension typed elaboration.
- Header/body/total contract reconciliation model for existing workflow declarations.
- Export/import of enough workflow type/contract summaries for modular checking.
- Diagnostics for unsupported workflow targets, wrong bind RHS constructor, missing coverage, and explicit-lift requirements.

Out of scope for the first implementation slice:

- User-defined `Monad<M>` implementations.
- Public type parameterization such as `Workflow<C, A>`.
- Public construction of arbitrary contract values by user syntax.
- Dynamic admission.
- Workflow parallel/applicative syntax beyond existing sequential do/comprehension bind.
- `WorkflowHandle<A>` as a public surface type.
- Workflow-level `par`, `spawn`, `scatter`, `cancel`, mailboxes/channels, monitors, or subscriptions.
- Pattern binders, guards, target inference, one-hole constructor targets, or pure `List`/`Option`/`Result` dictionaries.
- Aggressive contract normalization or scope erasure.

## 5. Public Type and Library Surface

### 5.1 Public type constructor

The public type form is:

```text
Workflow<A>
```

`Workflow<A>` is a unary computation constructor of kind `* -> *` for the purposes of SPEC-054 target resolution.

This spec does not expose:

```text
Workflow<C, A>
Workflow<Contract, A>
Workflow<Env, A>
```

Contract/evidence details remain internal semantic and implementation carriers.

### 5.2 Workflow namespace

The workflow library namespace is:

```text
workflow
```

Required first-slice operations:

```text
workflow::unit      : A -> Workflow<A>
workflow::bind      : Workflow<A> -> (A -> Workflow<B>) -> Workflow<B>
workflow::then      : Workflow<A> -> Workflow<B> -> Workflow<B>
workflow::from_proc : Proc<A> -> Workflow<A>
workflow::from_act  : Act<A> -> Workflow<A>
workflow::requires  : Requirement -> Workflow<Unit>
workflow::ensures   : OpenPostcondition -> Workflow<Unit>
```

Notation note: the `Fn(A) -> Workflow<B>` shape is specification notation. The implementation may reuse the existing typed-do continuation representation used for `Act`/`Proc` elaboration.

### 5.3 Algebraic intent

`Workflow` is Monad-shaped:

```text
unit  : A -> Workflow<A>
bind  : Workflow<A> -> (A -> Workflow<B>) -> Workflow<B>
then  : Workflow<A> -> Workflow<B> -> Workflow<B>
```

Expected laws hold up to contract-plan equivalence:

```text
workflow::bind(workflow::unit(a), f) ≈ f(a)
workflow::bind(w, workflow::unit)    ≈ w
workflow::bind(workflow::bind(w, f), g)
  ≈ workflow::bind(w, λx. workflow::bind(f(x), g))
```

The implementation is not required to prove these laws in the typechecker. Tests should check representative dictionary elaboration equivalence and contract-shape preservation.

## 6. Semantic Model

### 6.1 Workflow carrier

Internally, a workflow value is modeled as a synchronized product of a process projection and a contract projection:

```text
Workflow<A> = {
  contract : WorkflowContract<A>,
  body     : Proc<A>,
  evidence : CoverageEvidence,
}
```

The public type remains only `Workflow<A>`.

This record shape is a carrier view. The reference semantics preserves the core workflow form from which projections are derived; it must not treat `body` and `contract` as unrelated lowered artifacts.

### 6.2 Workflow forms, projections, and zipper semantics

Lowering produces a structure-preserving core workflow form:

```text
Surface workflow syntax -> WorkflowForm<A>
```

Each `WorkflowForm` has multiple projections:

```text
proc(form)       : Proc<A>
contract(form)   : WorkflowContract<A>
checks(form)     : CheckProjection<A>
resources(form)  : ResourceProjection<A>
failure(form)    : FailureProjection<A>
provenance(form) : ProvenanceProjection<A>
```

The reference workflow interpretation is a zipper over aligned projected events in the shared form:

```text
position p in WorkflowForm:
  E_proc(p)
  E_contract(p)
  E_check(p)
  E_resource(p)
  E_failure(p)
  E_provenance(p)

zip(E_proc(p), E_contract(p), E_check(p), E_resource(p), ...)
  = meaning of workflow step p
```

A form may be neutral in one projection and non-neutral in another. For example, a `requires` form may have an identity/unit `Proc` projection while still contributing a precondition event, failure behavior, and provenance obligation in other projections.

Workflow-algebra lowering must preserve nodes whose non-`Proc` projections are non-neutral. Static discharge, dynamic residualization, and no-op elimination belong to later type/constraint checking, verification, and runtime-lowering phases.

### 6.2.1 First-slice `WorkflowForm` grammar

The first implementation slice uses a closed core grammar:

```text
WorkflowForm<A> ::=
    Unit(expr : A)
  | Bind(form : WorkflowForm<A>, binder, cont : WorkflowForm<B>)
  | FromProc(proc : Proc<A>)
  | FromAct(act : Act<A>)
  | Requires(requirement : Requirement)
  | Ensures(postcondition : OpenPostcondition)
  | Scope(scope : WorkflowScope, form : WorkflowForm<A>)
```

`Then(w1, w2)` is derived syntax:

```text
Then(w1, w2) = Bind(w1, _, w2)
```

`Fail` and `WithError` are not first-slice primitive workflow-form nodes. They remain operational-bottom and scoped-failure behavior inherited from SPEC-050/SPEC-051 through the Proc/failure projections. A future spec may add explicit workflow failure forms only if workflow-specific routing cannot be represented by projected events.

`WorkflowForm` is the semantic source of truth. Carrier records such as `{ contract, body, evidence }` are implementation views derived from the preserved form, not independently maintained artifacts.

### 6.2.2 Projection events and alignment identity

Every workflow-form node has a stable synthetic identity:

```text
WorkflowNodeId
```

Every projected event carries at least:

```text
ProjectionEvent = {
  node       : WorkflowNodeId,
  projection : ProjectionKind,
  kind       : EventKind,
  origin     : SourceOrigin,
}

ProjectionKind ::= Proc | Contract | Check | AuthorityResource | Failure | Reporting | Provenance
AlignmentKey   ::= WorkflowNodeId × ProjectionKind
```

`SourceOrigin` must distinguish:

```text
SourceOrigin ::= SourceSpan(span)
               | Synthetic(parent_span, reason)
               | ImportedSummary(module, public_anchor)
```

The zipper is over the structured `WorkflowForm`, not over a permanently linear stream. Sequential forms may be traversed linearly in the first slice, but the representation must not exclude future branch forms such as workflow-level `par` or `scatter`.

A node may emit zero, one, or multiple events per projection. Neutral events are explicit when required for alignment, diagnostics, or evidence preservation.

### 6.2.3 Staged `ContractPlan` and obligation handoff

`WorkflowContract<A>` contains a staged `ContractPlan<A>` aligned with `WorkflowForm`:

```text
ContractPlan<A> ::=
    EmptyContract(A)
  | BindContract(C1 : ContractPlan<A>, binder, C2 : ContractPlan<B>)
  | RequirementContract(node, Requirement)
  | EnsuresContract(node, OpenPostcondition, target)
  | LowerProcContract(node, ProcContractSummary<A>)
  | LowerActContract(node, ActContractSummary<A>)
  | ScopeContract(scope, ContractPlan<A>)
```

The type/constraint handoff judgment is:

```text
Γ ⊢ᴡ form : Workflow<A> ▷ C, Ω
```

where `C` is a staged `ContractPlan<A>` and `Ω` is an obligation set. Typechecking constructs and classifies obligations; coverage/verification discharges them later into `CoverageEvidence` or diagnostics.

First-slice obligation classes include:

```text
RequirementMustHold(node, Requirement)
RequirementRefinementCovered(node, Requirement)
OpenPostconditionTarget(node, OpenPostcondition, target_type)
LowerProcCovered(node, ProcContractSummary<A>)
LowerActCovered(node, ActContractSummary<A>)
RequiredCapabilityCovered(node, CapabilityRef, Mode)
ResourceAvailable(node, ResourceRef, AccessMode)
FailureRouteDefined(node, FailureEventKind)
ProvenanceRecordable(node, ProvenanceEventKind)
OpaqueSummaryRejected(node, imported_name)
```

`CoverageEvidence` is produced by solving or residualizing these obligations. A first-slice implementation may residualize checks into runtime gates where existing workflow boundary semantics permit it, but it must not silently drop obligations.

### 6.2.4 Equality and normalization strata

Workflow equality has three distinct strata:

1. `WorkflowForm` equality is source/projection-preserving. `Requires` and `Ensures` nodes are not erased, even when their Proc projection is neutral.
2. Proc-projection equality may identify forms whose only differences are neutral governance nodes:

   ```text
   proc(Bind(Requires(R), _, w)) ≈ proc(w)
   ```

3. Optimized runtime equality may erase already-discharged neutral executable checks only when the pass preserves or commits the corresponding coverage/provenance/report evidence.

Therefore:

```text
Bind(Requires(R), _, w) != w
```

at the `WorkflowForm` level. Monad laws for `Workflow` hold only up to contract-plan/projection equivalence, not by early erasure of governance nodes.

### 6.3 Contract layers

A workflow declaration or expression may have:

```text
C_header   = HeaderContract       // declared workflow clauses, if any
C_body     = BodyContract         // inferred from the Proc body / lifted operands
E_coverage = CoverageEvidence     // proof/reconciliation witness
C_total    = TotalContract        // executable/reported workflow contract
```

The core coverage/reconciliation rule is:

```text
C_header ⊒cov C_body ⇓ E_coverage
C_total = Scope(name_or_expr, Reconcile(C_header, C_body, E_coverage))
```

For expression-built workflows, contract-injection forms such as `workflow::requires` and `workflow::ensures` contribute to `C_body` / the workflow expression's own `ContractPlan`. Declaration-level headers, where retained for compatibility, are best understood as leading contract-injection forms rather than a separate semantic island.

### 6.4 Coverage relation

Coverage is written:

```text
C_decl ⊒cov C_body ⇓ E_coverage
```

Coverage is componentwise and variance-aware. The declared/header contract must be sufficient to admit, govern, observe, verify, and report the inferred body behavior.

The minimum evidence components are:

```text
CoverageEvidence = {
  authority,
  resources,
  roles,
  checks,
  obligations,
  failure,
  reporting,
  provenance,
}
```

First-slice implementations may keep some components as empty/default evidence if the corresponding behavior is not yet expressible. Authority, resource, failure, and provenance evidence must not be silently discarded when already available from existing runtime substrates.

### 6.5 Governs relation

A workflow contract governs a proc when proc contract inference plus coverage succeeds:

```text
Γ ⊢ᴾ p : Proc<A> ▷ C_body
C ⊒cov C_body ⇓ E_coverage
---------------------------
Governs(C, p, E_coverage)
```

Every constructed `Workflow<A>` must satisfy `Governs(contract, body, evidence)`.

## 7. Operation Semantics

### 7.1 `workflow::unit`

```text
workflow::unit : A -> Workflow<A>
```

Semantics:

```text
body(workflow::unit(a))     = proc::unit(a)
contract(workflow::unit(a)) = EmptyWorkflowContract
```

Evidence:

```text
CoverageEvidence::empty()
```

No authority, resource, check, obligation, failure, reporting, or provenance requirements are introduced.

### 7.2 `workflow::bind`

```text
workflow::bind : Workflow<A> -> (A -> Workflow<B>) -> Workflow<B>
```

Semantics:

```text
body(bindW(w, f)) =
  proc::bind(body(w), λa. body(f(a)))

contract(bindW(w, f)) =
  bindC(contract(w), λa. contract(f(a)))
```

The continuation contract is staged after the first result. The implementation must preserve this staging and must not flatten value-dependent checks into unconditional static preconditions unless proven equivalent.

Admission envelope:

```text
envelope(bindW(w, f)) = envelope(w) ∪ staticUpperBound(λa. envelope(f(a)))
```

First-slice rule:

```text
If a static upper bound cannot be computed from available summaries, the composed workflow is rejected.
Dynamic admission is forbidden in this spec.
```

### 7.3 `workflow::then`

```text
workflow::then : Workflow<A> -> Workflow<B> -> Workflow<B>
```

`then` is sequencing that ignores the first normal result:

```text
workflow::then(w1, w2) = workflow::bind(w1, λ_. w2)
```

Its contract behavior is the corresponding non-dependent sequential contract composition.

### 7.4 `workflow::from_proc`

```text
workflow::from_proc : Proc<A> -> Workflow<A>
```

`from_proc` explicitly embeds process computation into governed workflow computation.

Semantics:

```text
body(workflow::from_proc(p)) = p
```

The contract contribution is inferred from `p` and preserved as a delayed coverage obligation:

```text
Γ ⊢ᴾ p : Proc<A> ▷ C_p
Γ ⊢ᴡ FromProc(p) : Workflow<A> ▷ LowerProcContract(node, C_p), { LowerProcCovered(node, C_p) }
```

First-slice rule:

- `from_proc(p)` does not require immediate `EmptyHeader ⊒cov C_p` coverage at the local expression site.
- Enclosing/composed workflow contracts, including preceding `requires` nodes in the same `WorkflowForm`, may cover the lower Proc requirements when final coverage is checked.
- Dynamic admission remains forbidden: `C_p` and its admission envelope must be statically available as a lower Proc summary, or the lift is rejected as opaque.
- Source-visible pure/proc computations with no extra authority requirements may discharge their obligations immediately.
- Proc values imported without summaries require exported contract summaries before they can be lifted in a checked way.

This rule prevents `from_proc` from becoming an authority-smuggling operation while preserving useful staged composition such as `requires capability.store.read; x <- workflow::from_proc(store_proc);`.

### 7.5 `workflow::from_act`

```text
workflow::from_act : Act<A> -> Workflow<A>
```

`from_act` is equivalent to explicit Act-to-Proc embedding followed by workflow lift:

```text
workflow::from_act(a) = workflow::from_proc(proc::from_act(a))
```

No implicit lift from `Act<A>` to `Workflow<A>` exists in `do:Workflow` or `[...]: Workflow`.

### 7.6 Contract-injection forms

The first workflow algebra includes contract-injection forms as ordinary workflow forms with possibly neutral `Proc` projections and non-neutral contract/check/failure/provenance projections.

Required initial forms:

```text
workflow::requires : Requirement -> Workflow<Unit>
workflow::ensures  : OpenPostcondition -> Workflow<Unit>
```

`requires` projection sketch:

```text
Requirement ::=
    RoleRequirement(RoleRef)
  | CapabilityRequirement(CapabilityRef, Mode)
  | ResourceRequirement(ResourceRef, AccessMode)
  | Precondition(CheckExpr)
  | PolicyRequirement(PolicyRef)

proc(workflow::requires(R))       = proc::unit(())
contract(workflow::requires(R))   = RequirementContract(node, R)
checks(workflow::requires(R))     = establish/check R at this workflow position
failure(workflow::requires(R))    = precondition/admission/coverage failure if R is not discharged
provenance(workflow::requires(R)) = record R and its discharge method/result
```

A requirement may refine the continuation checking environment. For example, `requires capability.store.read;` may allow a following `workflow::from_act(store.get(k))` or `workflow::from_proc(store_proc)` to be checked under a provisional capability assumption.

This refinement is not authority creation. The final coverage/admission pass must prove that the refined role/capability/resource/policy/precondition is actually admitted or derivable. If the assumption cannot be proven, checking fails with a component-specific coverage diagnostic.

`ensures` projection sketch:

```text
proc(workflow::ensures(Q))       = proc::unit(())
contract(workflow::ensures(Q))   = EnsuresContract(node, Q, unresolved_target)
checks(workflow::ensures(Q))     = postcondition obligation whose target is resolved by contract bind/zipper semantics
failure(workflow::ensures(Q))    = postcondition failure if the obligation is not satisfied on successful completion
provenance(workflow::ensures(Q)) = record Q, target, and discharge method/result
```

Target rule:

```text
Bind(Ensures(Q), _, rest : Workflow<A>)
```

resolves `Q` under a distinguished binder:

```text
result : A
```

and attaches `Q` to the successful result boundary of `rest`. If `rest` fails before producing a normal `A`, the postcondition is not checked as a successful-result postcondition; failure projection handles that path. Nested `ensures` forms stack in source order over their respective suffix workflow forms.

This spec does not require `requires` or `ensures` to lower immediately to executable checks. They are preserved as workflow-form events. Type/constraint checking and verification later determine whether each event is statically discharged, rejected, or residualized into a dynamic runtime check/gate.

### 7.7 Operational `fail` and `with_error`

Inside `do:Workflow`, `fail` remains operational bottom as defined by SPEC-050.

It does not become MonadFail, `Result.Err`, `Option.None`, or a domain-level workflow value.

`with_error` inside workflow expressions routes operational failures through the scoped failure-boundary behavior defined by SPEC-050 and the workflow boundary hooks defined by SPEC-051. First-slice `Workflow` dictionary support must not redefine failure semantics.

## 8. Typed Do-Notation Integration

### 8.1 Target resolution

`Workflow` becomes a compiler-known do target alongside `Act` and `Proc`.

```ash
do:Workflow {
    x <- wf1;
    y <- wf2(x);
    return y
}
```

Target resolution requirements:

- `Workflow` must resolve as a unary computation constructor of kind `* -> *`.
- The target dictionary uses `workflow::unit` and `workflow::bind`.
- `DoTowerLevel` or equivalent internal tower classification must distinguish Workflow from Proc.
- Diagnostics must mention `Act`, `Proc`, and `Workflow` as registered computation constructors when appropriate.

### 8.2 Statement checking

SPEC-054 statement rules apply unchanged for ordinary monadic statements:

```text
let x = pure_expr;      // pure lexical binding
x <- workflow_expr;     // RHS must synthesize Workflow<A>
_ <- workflow_expr;     // explicit ignored workflow action
return result_expr      // final result wrapped by workflow::unit
```

Workflow blocks additionally admit contract-injection statements as syntax for ordinary `Workflow<Unit>` forms:

```text
requires requirement;       // lowers as _ <- workflow::requires(requirement);
ensures postcondition;      // lowers as _ <- workflow::ensures(open_postcondition);
```

These statements preserve their workflow-form nodes during algebraic lowering. Their `Proc` projection may be neutral, but their contract/check/failure/provenance projections are not erased at this stage.

No implicit tower lifts are introduced.

Invalid in `do:Workflow`:

```ash
x <- some_proc();       // Proc<A>, not Workflow<A>
y <- some_act();        // Act<A>, not Workflow<A>
```

Valid with explicit lifts:

```ash
x <- workflow::from_proc(some_proc());
y <- workflow::from_act(some_act());
```

### 8.3 Typed elaboration

Typed elaboration must reuse SPEC-054's nested bind/return checking path, but the target artifact for `Workflow` is a preserved `WorkflowForm` before any evidence-preserving optimization.

Workflow block statements lower as follows:

```text
return e
  => Unit(e)

x <- e; rest
  => Bind(elaborateWorkflow(e), x, elaborateWorkflow(rest))

_ <- e; rest
  => Bind(elaborateWorkflow(e), _, elaborateWorkflow(rest))

let x = e; rest
  => pure lexical binding of x while elaborating rest

requires R; rest
  => Bind(Requires(R), _, elaborateWorkflow(rest))

ensures Q; rest
  => Bind(Ensures(Q), _, elaborateWorkflow(rest))
```

Conceptual source:

```ash
do:Workflow {
    requires role.analyst;
    ensures result.valid;
    x <- a;
    y <- b(x);
    return f(x, y)
}
```

elaborates to the same workflow-form skeleton as:

```text
Bind(Requires(role.analyst), _,
  Bind(Ensures(open(result.valid)), _,
    Bind(a, x,
      Bind(b(x), y,
        Unit(f(x, y))))))
```

and has the same Proc-projection shape as the dictionary expression:

```text
workflow::bind(workflow::requires(role.analyst), λ_.
  workflow::bind(workflow::ensures(open(result.valid)), λ_.
    workflow::bind(a, λx.
      workflow::bind(b(x), λy.
        workflow::unit(f(x, y))))))
```

Modulo spans/origin metadata, ordinary bind/return checking is the same path used for Act/Proc targets. The difference is that workflow contract-injection forms contribute non-Proc projected events that remain present for zipper interpretation and later verification/lowering.

## 9. Comprehension Integration

SPEC-055 bracket comprehensions become valid for explicit `Workflow` targets:

```ash
[result | x <- wf1, y <- wf2(x)]: Workflow
```

They normalize through the existing comprehension-to-do path and then through the same workflow-form builder used by `do:Workflow`:

```ash
do:Workflow {
    x <- wf1;
    y <- wf2(x);
    return result
}
```

All SPEC-055 MVP restrictions still apply:

- explicit target required unless target inference is implemented later;
- simple identifier binders only;
- no guards;
- no pattern binders;
- no applicative/parallel comprehension semantics;
- no implicit imports or implicit tower lifts.

## 10. Workflow Declarations and First-Class Values

Existing workflow declarations remain valid.

This spec adds a semantic interpretation:

```text
workflow declaration = named/scoped/exported Workflow<A> expression plus host/runtime entrypoint metadata
```

A declaration body is target-typed as a workflow block and lowers to a `WorkflowForm<A>` / `Workflow<A>` value. Header-like clauses such as `requires` and `ensures` are contract-injection workflow forms in that block, not a separate semantic island.

Conceptual declaration rule for new-style bodies:

```text
Γ, params ⊢ block ⇝ form : WorkflowForm<A>
proc(form)     = p : Proc<A>
contract(form) = C_expr : WorkflowContract<A>
Governs(C_expr, p, E_expr)
C_total = Scope(name, C_expr)
```

The declaration exports a workflow summary sufficient for imported callers to type-check uses of the named workflow as a `Workflow<A>` value.

Where compatibility requires parsing a legacy proc-only body, the implementation may elaborate it through an explicit synthetic workflow form whose contract projection is inferred from the proc body. That compatibility path must still preserve the resulting workflow-form/projection alignment before later checking or optimization.

## 11. Module Export and Import Requirements

First-class workflows require module metadata to preserve more than a parameter list.

For every exported workflow-producing definition, module metadata must preserve:

```text
name
parameter names and types
return type A
public type Workflow<A>
public HeaderContract / TotalContract summary sufficient for coverage checking
public staged WorkflowContractSummary<A> sufficient for bind/then composition
admission envelope summary
exported failure/report/provenance summary
public source-origin / alignment anchors without exposing private body node ids
```

Private internal body details need not be exported, but the exported summary must be strong enough for downstream `do:Workflow` and `[...]: Workflow` checking.

Imported workflow values without a contract summary must be rejected for first-class composition or treated as opaque values that cannot be safely bound under `Workflow` until a summary exists.

## 12. Diagnostics

The first implementation slice must add or adapt diagnostics for:

1. Unknown do/comprehension target: mention `Workflow` as a registered target once implemented.
2. Wrong target kind: `Workflow` must be unary `* -> *`.
3. Missing `Workflow` dictionary: internal compiler error or unsupported-target diagnostic until implemented.
4. Wrong `<-` RHS in `do:Workflow`: expected `Workflow<A>`, found `Act<A>`/`Proc<A>`/pure `A`.
5. Explicit lift hint: suggest `workflow::from_proc(...)` or `workflow::from_act(...)` where applicable.
6. Coverage/obligation failure: declared/enclosing contract does not cover inferred body/lower Proc/lower Act contract obligation.
7. `requires` refinement failure: a requirement refined the continuation checking environment but final coverage/admission cannot prove it.
8. `ensures` target/type failure: an open postcondition cannot be resolved against the successful result type of its suffix workflow.
9. Opaque imported workflow/proc/act lacks required contract summary.
10. Dynamic admission required but forbidden by this spec.
11. Parser-only lowering attempted for `do:Workflow` or `[...]: Workflow`.
12. Evidence-preserving optimization violation: a neutral Proc-projection governance node was erased before its coverage/provenance/report evidence was preserved.

Diagnostics should state expected shape, found type, relevant contract/evidence component, and one likely fix.

## 13. Runtime and Lowering Boundary

This spec is primarily a type/elaboration and semantic-carrier spec.

Runtime/lowering requirements for the first slice:

- `Workflow<A>` values may be represented internally by an existing or new runtime carrier that contains a `Proc<A>` body plus contract/evidence metadata.
- Running a workflow still occurs through the workflow boundary semantics of SPEC-051.
- `workflow::bind` must sequence the underlying Proc bodies via `proc::bind` and preserve contract/evidence staging.
- `workflow::unit` may reuse `proc::unit` for the body.
- `workflow::requires` and `workflow::ensures` must preserve their workflow-form nodes and non-Proc projected events through workflow-algebra lowering.
- `workflow::from_proc` and `workflow::from_act` must not bypass workflow admission/coverage. They preserve lower summaries and emit delayed coverage obligations rather than requiring local empty-header discharge when the enclosing/composed workflow may cover them.

Static discharge of requirements, dynamic residualization of checks, and erasure of neutral `Proc`-projection nodes are type/constraint-checking and runtime-lowering concerns. They must not happen before the workflow-form/projection alignment has been constructed.

This spec does not require a new scheduler, new process runtime, new workflow terminal state, or new CLI command.

## 14. Verification Requirements

Implementation must include tests for:

1. `WorkflowForm` grammar preserves `Unit`, `Bind`, `FromProc`, `FromAct`, `Requires`, `Ensures`, and `Scope` nodes with stable `WorkflowNodeId`s.
2. Projection events carry node id, projection kind, event kind, and source-origin metadata, including synthetic and imported-summary origins.
3. TypeEnv registers `Workflow` as a builtin unary constructor.
4. `Workflow` do target resolves with a workflow dictionary.
5. `do:Workflow { return x }` synthesizes `Workflow<A>`.
6. `do:Workflow` binds only `Workflow<A>` RHS values.
7. `do:Workflow` rejects `Proc<A>` and `Act<A>` RHS values without explicit lifts.
8. `workflow::from_proc` and `workflow::from_act` allow explicit lifts where lower summaries exist and emit delayed coverage obligations.
9. `requires` and `ensures` in a workflow block lower to preserved `workflow::requires` / `workflow::ensures` workflow forms.
10. `requires` may refine continuation checking context but produces coverage obligations that must be proven later.
11. `ensures` targets the successful result boundary of the suffix workflow and typechecks under `result : A`.
12. Typed elaboration of `do:Workflow` produces nested workflow-form `Bind` / `Unit` shape without erasing neutral-Proc contract-injection nodes.
13. `[result | x <- wf]: Workflow` elaborates equivalently to `do:Workflow` and the same `WorkflowForm` / projection alignment.
14. Coverage/obligation failures produce component-specific diagnostics.
15. Imported workflow summaries are preserved across module boundaries or rejected if absent.
16. Existing `do:Act`, `do:Proc`, Act blocks, and Act/Proc comprehensions remain unchanged.

## 15. Non-Interference

This spec must not redefine:

- Proc runtime identity, process handles, or resource split/join semantics from SPEC-049.
- Operational bottom/failure handling from SPEC-050.
- Workflow boundary outcomes from SPEC-051.
- Capability/resource authority provenance from SPEC-052/053.
- Generic typed-do syntax from SPEC-054.
- Generic comprehension syntax from SPEC-055.

Workflow target support is an additional dictionary target over existing typed-do/comprehension infrastructure.

## 16. Future Work

Deferred follow-on specs should address:

- Additional workflow contract forms for admission, reporting, provenance policies, and failure policies beyond the initial `requires` / `ensures` forms.
- `WorkflowHandle<A>` and handle-latent obligation lifecycle.
- Workflow-level `par`, `spawn`, `scatter`, `cancel`, `await`, `join`, and `gather`.
- Dynamic admission as an explicit audited capability.
- Public contract/value reflection APIs for tools.
- Workflow target inference for comprehensions.
- User-defined Monad dictionaries.
- Formatter and LSP enhancements for `do:Workflow` and `[...]: Workflow` examples.

## 17. Changelog

### 2026-04-30

- Hardened the draft around a blocking workflow-form/projection semantic gate before implementation: closed first-slice `WorkflowForm` grammar, node/alignment identity, projection events, staged `ContractPlan`, obligation handoff, `requires` refinement, `ensures` suffix-result targeting, delayed `from_proc`/`from_act` coverage obligations, and equality strata.

### 2026-04-29

- Initial draft promoted from DESIGN-033 into a normative first-class `Workflow<A>` carrier, Monad target, typed-do, and comprehension integration spec.
