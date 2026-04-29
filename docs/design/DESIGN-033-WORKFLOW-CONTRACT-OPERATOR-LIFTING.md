# DESIGN-033: Workflow Contract Operator Lifting

**Status:** Draft
**Date:** 2026-04-29
**Related:** DESIGN-030, DESIGN-031, SPEC-047, SPEC-048, SPEC-049, SPEC-050, SPEC-054, SPEC-056, PLAN-104, NOTE-007

## 1. Summary

Workflow should be modeled as a contract-indexed process carrier:

```text
Workflow<A> = WorkflowContract<A> + Proc<A>
WorkflowContract<A> = AdmissionEnvelope + ContractPlan<A>
```

`Proc<A>` owns the computational/process behavior. `Workflow<A>` owns the governed version of that behavior: admission, authority envelope, staged contract plan, obligations, reporting, failure-boundary behavior, and coverage evidence.

The central design rule is:

```text
A Proc operator is workflow-liftable only when its induced WorkflowContract behavior is defined.
```

For every Proc operator `opᴾ`, future specs should define:

```text
opᶜ : contract-level behavior
opᵂ : workflow-level behavior
```

such that:

```text
body(opᵂ(...))     = opᴾ(body(...))
contract(opᵂ(...)) = opᶜ(contract(...))
```

This design note is not yet a normative workflow spec. It is a reusable ruleset for writing future workflow specs and tasks.

## 2. Motivation

The lower Ash tower levels already have compositional semantics:

```text
Pure  = value computation
Act   = sequential effect computation
Proc  = process/concurrency computation
```

Workflow has historically been weaker because it mixes several responsibilities:

- entrypoint/declaration shape;
- role/capability/resource admission;
- `requires` / `ensures` contracts;
- reporting/provenance summarization;
- operational failure reinterpretation at the workflow boundary;
- process orchestration through the Proc substrate.

The normalization in this note separates those responsibilities without making workflow non-compositional.

`Workflow<A>` should be first understood as:

```text
a Proc<A> plus a compositional contract algebra governing that Proc.
```

Then workflow composition becomes:

```text
Workflow composition = Proc composition + WorkflowContract composition + coverage evidence.
```

The hard part is therefore not inventing a new execution calculus. The hard part is defining how every relevant Proc operator acts on workflow contracts.

## 3. Core Definitions

### 3.1 Workflow carrier

Conceptually:

```text
Workflow<A> = {
  contract : WorkflowContract<A>,
  body     : Proc<A>,
}
```

A future implementation should make coverage evidence explicit internally:

```text
Workflow<A> = {
  contract : WorkflowContract<A>,
  body     : Proc<A>,
  evidence : CoverageEvidence(contract, body),
}
```

The evidence may be compiler/typechecker-produced rather than a public runtime value, but it is not merely documentary. It is important at compile/verification time and may also guide runtime projection, enforcement, provenance construction, and audit reporting.

A more formal reading:

```text
Workflow<A> = Σ c : WorkflowContract<A>. Proc_c<A>
```

where `Proc_c<A>` means a proc whose authority/resource/effect/failure behavior is covered by contract `c`.

### 3.2 Workflow form, projections, and zipper interpretation

The carrier/product notation must not be read as two unrelated lowered artifacts. A workflow expression has one preserved compositional form, and `Proc` and `WorkflowContract` are projections of that form:

```text
WorkflowForm<A>
  ├─ proc      : ProcProjection(form)      = Proc<A>
  ├─ contract  : ContractProjection(form)  = WorkflowContract<A>
  └─ evidence  : Alignment/CoverageEvidence(form)
```

Equivalently, the workflow carrier is a synchronized product:

```text
Workflow<A> = Proc<A> × WorkflowContract<A> × AlignmentEvidence
```

The `AlignmentEvidence` records that contract/check/resource/failure/provenance obligations are located at the same workflow-form positions as the process behavior they constrain or annotate. This is the communication relation between projections.

Reference interpretation is a zipper over the aligned projected steps/events:

```text
position p in WorkflowForm:
  proc event        E_proc(p)
  contract event    E_contract(p)
  check event       E_check(p)
  resource event    E_resource(p)
  failure event     E_failure(p)
  provenance event  E_provenance(p)
  ...

zip(E_proc(p), E_contract(p), E_check(p), E_resource(p), ...)
  = workflow step meaning at p
```

Different projections may interpret the same form differently. A form whose `Proc` projection is neutral can still be semantically non-neutral in contract/check/failure/provenance projections.

For example, a future contract-injection form such as `requires R` may have:

```text
proc(requires R)       = proc::unit(())       // neutral process projection
contract(requires R)   = Precondition(R)      // non-neutral contract event
check(requires R)      = obligation to establish R at this position
failure(requires R)    = precondition/admission failure if R is not discharged
provenance(requires R) = record R and discharge evidence
```

The workflow algebra spec should define the per-form projected events and zipper meaning. It should not prematurely decide whether a projected event becomes a runtime check, is statically discharged, or is optimized away.

#### Structure preservation rule

Lowering from surface workflow syntax into core workflow algebra must preserve the compositional structure of workflow forms across all projections:

```text
lower(surface_workflow) = WorkflowForm
proc(lower(...))        // projected from the same form
contract(lower(...))    // projected from the same form
```

A node whose `Proc` projection is neutral must not be erased during workflow-algebra lowering if any non-`Proc` projection is non-neutral. Erasure, simplification, static discharge, and dynamic residualization are later typechecking/verification/lowering transformations, not part of the initial workflow-form lowering.

### 3.3 Workflow contract

A workflow contract has two conceptual layers:

```text
WorkflowContract<A> = AdmissionEnvelope + ContractPlan<A>
```

#### AdmissionEnvelope

The admission envelope is the static or conservative authority/resource envelope that must be admitted before a closed workflow run starts.

It answers:

- which capabilities may be invoked;
- which resources may be used;
- which roles/authority projections are permitted;
- what dynamic admission, if any, is explicitly allowed.

#### ContractPlan

The contract plan is the staged, scope-preserving contract structure aligned with workflow/proc execution.

It records:

- staged `requires` checks;
- `ensures` and other obligations;
- subworkflow scopes;
- branch and handle-latent obligations;
- failure-boundary behavior;
- report/provenance plans;
- authority/resource use topology.

The plan must preserve sequencing, dependency, branch identity, and scopes. It should not be flattened into a set of pre/post conditions too early.

### 3.4 Contract summaries

A future checker or runtime may derive summaries from plans:

```text
normalize : ContractPlan<A> -> ContractSummary
```

A summary may contain:

```text
ContractSummary = {
  admissionEnvelope : AdmissionEnvelope,
  authorityUseGraph : AuthorityUseGraph,
  resourceUseGraph  : ResourceUseGraph,
  stagedChecks      : Vec<StagedCheck>,
  obligations       : ObligationGraph,
  failureBoundaries : FailureBoundaryTree,
  reportPlan        : ReportPlan,
  provenancePlan    : ProvenancePlan,
}
```

But summaries are derived artifacts. The source `ContractPlan` remains the structure-preserving object used to reason about composition.

### 3.5 Coverage, entailment, and evidence

`Covers` is not intended as a first public Ash type. In this design note it names a family of checker judgments and evidence values that connect declared contracts, inferred body behavior, and executable runtime projections.

The primary contract-level relation should be written:

```text
C_decl ⊒cov C_body
```

Read:

```text
C_decl covers C_body.
```

where:

```text
C_decl : WorkflowContract<A>       // declared/header/public contract
C_body : WorkflowContract<A>       // contract behavior inferred from the Proc body
```

Coverage is not equality. The declared contract may be coarser and more public-facing than the inferred body contract, but it must be sufficient to admit, govern, observe, verify, and report the body behavior.

A related proc-level relation is:

```text
Governs(C, p)
```

which is derived from contract inference plus coverage:

```text
Γ ⊢ᴾ p : Proc<A> ▷ C_body
C ⊒cov C_body
-------------------------
Governs(C, p)
```

The implementation should produce an internal witness:

```text
CoverageEvidence(C_decl, C_body)
```

or, when attached to a body:

```text
CoverageEvidence(C_decl, p)
```

This witness is operationally significant. It records how the declared contract covers the inferred body behavior, and it can be used by later compile-time verification and runtime enforcement.

A possible evidence shape is:

```text
CoverageEvidence = {
  authority  : AuthorityCoverageEvidence,
  resources  : ResourceCoverageEvidence,
  roles      : RoleCoverageEvidence,
  checks     : CheckCoverageEvidence,
  obligations: ObligationCoverageEvidence,
  failure    : FailureCoverageEvidence,
  reporting  : ReportingCoverageEvidence,
  provenance : ProvenanceCoverageEvidence,
}
```

Component meanings:

- **Authority evidence** maps inferred capability/authority uses to admitted bindings or authority projections.
- **Resource evidence** maps inferred resource uses to resource admissions, lifetime/split/share/move decisions, and child projections.
- **Role evidence** maps body role assumptions to admitted role contexts and child role projections.
- **Check evidence** proves that required staged checks dominate the body points that need them.
- **Obligation evidence** proves that inferred `ensures` and obligations are preserved, strengthened, discharged, or reported at the right scope.
- **Failure evidence** proves that all possible operational failures can be routed/mapped while preserving identity and cause provenance.
- **Reporting evidence** proves that required report events are retained or summarized according to policy.
- **Provenance evidence** proves that required audit evidence is constructed and not erased before the relevant boundary.

A future implementation entry point may look like:

```rust
fn check_covers(
    declared: &WorkflowContract,
    inferred: &WorkflowContract,
) -> Result<CoverageEvidence, CoverageError>
```

The evidence is useful in at least three phases:

1. **Compile/type-check time.** Reject workflows whose declared/header contract cannot cover the inferred body contract. Produce diagnostics using the failed component evidence.
2. **Verification time.** Prove or test that lifted operators preserve governance invariants such as no authority widening, resource linearity, failure identity preservation, and provenance monotonicity.
3. **Runtime/lowering time.** Project admitted authority/resources into `EffEnv`/`ProcEnv`, derive child environments for `par`, route failures through the correct boundaries, and construct report/provenance records with explicit evidence links.

Coverage should be componentwise but not a naive subset test. Examples:

```text
Authority/capabilities:
  body-required authority must be admitted by the declared envelope.

Resources:
  the declared envelope must support the inferred use topology;
  sequential exclusive reuse may be valid while parallel exclusive reuse may fail.

Requires/checks:
  checks must occur at or before the staged body point that needs them.

Ensures/obligations:
  inferred obligations must be preserved, strengthened, explicitly discharged, or reported;
  they must not be silently erased.

Failure:
  body failures must be routable through declared boundaries while preserving cause identity and provenance.

Reporting/provenance:
  declared policy must retain or validly summarize all body-required audit evidence.
```

This note may continue to use `covers(...)` informally, but future specs should prefer the explicit relations:

```text
C_decl ⊒cov C_body
Governs(C_decl, p)
CoverageEvidence(C_decl, C_body)
```

### 3.6 Header, body, total contract, and reconciliation

Future specs should use stable names for the three contract layers involved in a workflow declaration:

```text
C_header   = HeaderContract       // elaborated from workflow declaration clauses
C_body     = BodyContract         // inferred from the Proc body
E_coverage = CoverageEvidence     // witness that C_header covers C_body
C_total    = TotalContract        // executable/reported workflow contract
```

The earlier shorthand:

```text
Scope(name, C_header ⋄ C_body)
```

should be understood as a placeholder for reconciliation, not as a naive merge.

Preferred design rule:

```text
C_header ⊒cov C_body ⇓ E_coverage
C_total = Scope(name, Reconcile(C_header, C_body, E_coverage))
```

`Reconcile` defines which parts of the header and body contracts become part of the executable workflow contract.

It must preserve:

- public/header requirements, admissions, and final obligations;
- inferred internal scopes needed for obligations, failure routing, provenance, and reporting;
- evidence mappings needed for runtime authority/resource projection;
- subworkflow or branch structure that remains observable in reports, failures, or verification.

It may hide or summarize:

- internal body details not needed for public reporting, runtime enforcement, or audit;
- proof-only details that do not need to appear in the stable workflow report;
- redundant inferred requirements already dominated by header checks.

This means `C_total` is not necessarily equal to either `C_header`, `C_body`, or a raw product of both. It is the reconciled contract produced by checking coverage.

Future specs should distinguish at least three evidence surfaces:

```text
FullCoverageEvidence      // internal compiler/runtime witness
DiagnosticEvidence        // spans and explanations for users
ProvenanceEvidenceProjection // stable report/audit-facing subset
```

This avoids making every internal proof detail public while still preserving correctness and provenance.

### 3.7 Coverage component variance

`C_decl ⊒cov C_body` is a structured coverage preorder, not one simple lattice order. Each component has its own coverage criterion and variance.

```text
Component      Coverage criterion                         Variance intuition
---------------------------------------------------------------------------------------------
Authority      body uses authorized by envelope            permissive enough at use sites, no widening
Capabilities   invoked operations admitted/bound           required operation surface covered
Resources      use graph supported by envelope             topology/lifetime/split/share/move check
Roles          role assumptions admitted/projected         role context covers body assumptions
Requires       declared checks dominate body needs         staged implication/dominance
Ensures        obligations preserved or strengthened       preservation/strengthening
Obligations    obligations discharged/reported correctly   no silent drop
Failure        failures routable with identity/cause       compatibility/refinement
Reporting      required events retained/summarized         evidence-preserving summary
Provenance     audit chain constructed and retained        monotone evidence preservation
```

Important consequence:

```text
coverage is componentwise, but each component has different polarity.
```

For example, a broader authority envelope can cover a narrower body authority use, even though broader authority is less restrictive. By contrast, a stronger final `ensures` clause can cover a weaker body guarantee because it preserves and strengthens the obligation. Resource coverage is neither ordinary subset nor ordinary logical implication; it is a check that the declared resource envelope supports the inferred use topology.

Future specs should therefore avoid defining `⊒cov` as a single set inclusion. They should define per-component coverage rules and then combine them into the structured relation.

## 4. Contract Composition Model

Workflow contracts compose componentwise, but not by naive record merge.

A contract is a product of component algebras:

```text
WorkflowContract =
  Admission
  × Authority
  × Roles
  × Capabilities
  × Resources
  × Requires
  × Ensures
  × Obligations
  × FailureBoundary
  × Reporting
  × Provenance
```

For each workflow composition operator, there must be a corresponding operation for each component.

Therefore contract composition is:

```text
componentwise + scope-preserving + operator-indexed
```

Meaning:

- **componentwise:** each contract component has its own composition operation;
- **scope-preserving:** subworkflow/process/branch/failure/report scopes remain visible unless a verified normalization erases them;
- **operator-indexed:** sequential bind, parallel start, observation, failure handling, scope, choice, and cancellation induce different component operations.

Example component behavior:

```text
Component          Sequential composition              Parallel composition
--------------------------------------------------------------------------------
Capabilities       union + constraint meet/refine       split/share check + union
Roles              ordered/stacked role contexts        branch role projection
Resources          lifetime-aware sequence              split/share/exclusive check
Requires           check first, then staged second      conjunction before branches
Ensures            first as intermediate obligation     branch-indexed obligations
Obligations        ordered concatenation                branch-indexed merge
Provenance         sequential append                    DAG/branch merge
Reporting          append sections                      branch-indexed merge
Failure boundary   nested/ordered handlers              aggregate-preserving handlers
Authority          no widening; staged refinement       no widening; child projection
```

This table is intentionally design-level. Future specs should refine each row into normative rules.

## 5. Operator Lifting Rule

For each Proc operator:

```text
opᴾ : Proc operands -> Proc result
```

future specs must define:

```text
opᶜ : Contract operands -> Contract result
opᵂ : Workflow operands -> Workflow result
```

with the homomorphism obligations:

```text
body(opᵂ(args)) = opᴾ(map body args)
```

and:

```text
contract(opᵂ(args)) = opᶜ(map contract args)
```

For dependent bind:

```text
body(bindᵂ(w, f)) =
  bindᴾ(body(w), λa. body(f(a)))

contract(bindᵂ(w, f)) =
  bindᶜ(contract(w), λa. contract(f(a)))
```

This rule makes Workflow a principled lift of Proc rather than a separate process calculus.

## 6. Structural Contract Judgment

A useful future formalization is an inferred contract judgment over Proc:

```text
Γ ⊢ᴾ p : Proc<A> ▷ C
```

Read:

```text
Under Γ, proc p has type Proc<A> and inferred workflow-contract behavior C.
```

Example rules:

```text
Γ ⊢ v : A
-------------------------------
Γ ⊢ᴾ pure(v) : Proc<A> ▷ Empty
```

```text
Γ ⊢ᴾ p : Proc<A> ▷ C1
Γ, x:A ⊢ᴾ k(x) : Proc<B> ▷ C2(x)
----------------------------------------------------------
Γ ⊢ᴾ bind(p, k) : Proc<B> ▷ BindC(C1, λx. C2(x))
```

```text
Γ ⊢ᴾ p1 : Proc<A> ▷ C1
Γ ⊢ᴾ p2 : Proc<B> ▷ C2
split_ok(C1, C2)
------------------------------------------------------------------
Γ ⊢ᴾ par(p1, p2) : Proc<(Handle<A>, Handle<B>)> ▷ ParC(C1, C2)
```

A workflow declaration then combines declared/header contract information with the inferred body contract:

```text
Γ ⊢ᴾ body : Proc<A> ▷ C_body
Γ ⊢ headers ⇝ C_header
C_header ⊒cov C_body ⇓ E_coverage
C_total = Scope(name, Reconcile(C_header, C_body, E_coverage))
-------------------------------------------------------------------------
Γ ⊢ workflow name(...) -> A { headers; body }
    : Workflow<A>
    with contract C_total
    and evidence E_coverage
```

The declared/header contract is the public/admission contract. The inferred body contract preserves internal scopes and obligations. The total contract is a reconciliation of both. The coverage evidence records the reconciliation and should remain available to later verification/lowering/runtime phases.

## 7. Global Properties for Lifted Operators

Future operator specs should reference this shared property set.

### P0. Body homomorphism

For every lifted operator:

```text
body(opᵂ(args)) = opᴾ(map body args)
```

The workflow operator must not secretly change Proc behavior except through explicit runtime enforcement terms specified by contract interpretation.

### P1. Contract construction homomorphism

For every lifted operator:

```text
contract(opᵂ(args)) = opᶜ(map contract args)
```

For dependent operators:

```text
contract(bindᵂ(w, f)) = bindᶜ(contract(w), λa. contract(f(a)))
```

### P2. Contract soundness / coverage evidence

The produced contract must cover the produced body, and the checker should retain evidence of that fact:

```text
Γ ⊢ᴾ body(opᵂ(args)) : Proc<A> ▷ C_body
contract(opᵂ(args)) ⊒cov C_body ⇓ E_coverage
```

Equivalently:

```text
Governs(contract(opᵂ(args)), body(opᵂ(args)))
```

where `Governs` is backed by `E_coverage`.

For declarations:

```text
Γ ⊢ᴾ body : Proc<A> ▷ C_body
Γ ⊢ headers ⇝ C_header
C_header ⊒cov C_body ⇓ E_coverage
```

Coverage evidence is part of the design invariant, not optional commentary. Future specs should state how each lifted operator composes or produces the evidence needed to prove the resulting workflow remains governed.

### P3. No authority widening

An operator may preserve, narrow, split, move, sequence, or consume authority.

It may not create authority outside the admitted envelope.

```text
AuthorityUsed(opᶜ(...)) ⊆ AuthorityEnvelope(opᶜ(...))
```

and child/projected authority must satisfy:

```text
ChildAuthority ⊆ ParentAuthority
```

### P4. Resource linearity and honest sharing

An operator may not duplicate linear/exclusive resources.

Parallel composition requires an explicit split/share/move rule for every non-copyable resource.

Sequential composition may reuse exclusive resources only if lifetimes do not overlap.

### P5. Scope preservation

Operator contract construction must preserve workflow/process/branch/failure/report/obligation scopes unless a normalization proves erasure is behavior-preserving.

```text
eraseScope(C) allowed only with evidence PreservesObservableBehavior(C, eraseScope(C))
```

### P6. Staged dependency preservation

If a later contract depends on an earlier runtime value, the `ContractPlan` must preserve that staging.

It must not flatten value-dependent checks into unconditional static checks unless proven equivalent.

### P7. Handle obligation preservation

Every handle-producing operator must attach latent contract obligations to each returned handle.

```text
parᵂ(w1, w2)
  returns handles h1, h2
  where h1 carries contract(w1)
    and h2 carries contract(w2)
```

### P8. Observation discharge

Every handle-observing operator must consume or otherwise account for the handle's latent contract obligations.

```text
awaitᵂ(h)
  consumes h
  observes child terminal state
  discharges/imports/surfaces h.contract
```

### P9. Failure identity preservation

Failure mapping may reclassify or wrap failures, but must preserve original cause identity and provenance.

For aggregate observation:

```text
join/gather must preserve every failed child identity.
```

### P10. Provenance monotonicity

Operator composition may add, structure, or summarize provenance.

It must not erase required audit evidence before the boundary where that evidence is no longer required.

### P11. Contract refinement monotonicity

If `C1` is at least as strong/restrictive as `C2`, then operator composition should preserve that refinement order.

For unary operators:

```text
C1 ⊑ C2  =>  opᶜ(C1) ⊑ opᶜ(C2)
```

For binary operators:

```text
C1 ⊑ C1' and C2 ⊑ C2'
  => opᶜ(C1, C2) ⊑ opᶜ(C1', C2')
```

The exact direction of `⊑` must be fixed by the contract lattice convention. The design intent is:

```text
refining a subworkflow contract must not make the composed workflow less governed.
```

### P12. Monad/applicative law compatibility

For sequential workflow composition:

```text
pureW(a) >>= f       ≈ f(a)
w >>= pureW          ≈ w
(w >>= f) >>= g      ≈ w >>= (λx. f(x) >>= g)
```

where `≈` is contract-plan equivalence preserving observable admission/check/failure/report behavior and required scopes.

Parallel/applicative laws should be added only after `parᶜ` is stable.

## 8. Operator-Lift Specification Template

Every future operator-lift spec should include the following sections.

```text
1. Proc signature
2. Workflow signature
3. Contract signature
4. Informal behavior
5. Contract-plan construction
6. Admission-envelope behavior
7. Authority behavior
8. Resource behavior
9. Check timing
10. Ensures/obligation behavior
11. Failure behavior
12. Provenance/report behavior
13. Handle behavior, if any
14. Scope behavior
15. Static coverage rule
16. Runtime enforcement rule
17. Algebraic laws/properties
18. Diagnostics/invalid cases
19. Examples
20. Open questions / deferred cases
```

### 8.1 Proc signature

State the existing Proc-level operator and classify it as one or more of:

```text
immediate | sequential/dependent | handle-producing | handle-observing | boundary-forming | resource/communication
```

Also state what identities it creates or preserves:

```text
WorkflowId | ProcessId | BranchId | EffectScopeId | LexicalFrameId
```

### 8.2 Workflow signature

State the lifted workflow-level operator.

If handles are involved, state whether the surface exposes workflow-aware handles or whether the contract annotation is internal.

### 8.3 Contract signature

State the induced contract operator. This is the central object of the lift spec.

Examples:

```text
bindᶜ : WorkflowContract<A>
      -> (A -> WorkflowContract<B>)
      -> WorkflowContract<B>
```

```text
parᶜ : WorkflowContract<A>
     -> WorkflowContract<B>
     -> WorkflowContract<(WorkflowHandle<A>, WorkflowHandle<B>)>
```

```text
awaitᶜ : HandleContract<A> -> WorkflowContract<A>
```

### 8.4 Contract-plan construction

Define the exact `ContractPlan` constructor or rewrite produced by the operator.

Rules:

- preserve operator structure;
- preserve value dependency;
- preserve branch identity;
- preserve subworkflow scopes;
- do not flatten too early.

### 8.5 Admission-envelope behavior

Define how the static or conservative admission envelope changes.

For dependent bind:

```text
envelope(bindᵂ(w, f)) =
  envelope(w) ∪ staticUpperBound(λa. envelope(f(a)))
```

If no static upper bound exists, the spec must either reject the closed workflow or require explicit audited dynamic admission.

### 8.6 Authority behavior

Classify authority behavior as:

```text
Preserve | Narrow | Split | Move | Consume | ObserveOnly | Require | Reject
```

State how the rule satisfies no-authority-widening.

### 8.7 Resource behavior

Classify resource behavior as:

```text
CopyReadOnly | SequentialReuse | Split | Move | SharedConcurrent | Consume | Forbidden
```

State how the rule respects linearity and exclusive-resource constraints.

### 8.8 Check timing

State when `requires` and related checks fire.

Possible phases:

```text
BeforeOuterStart
BeforeOperator
BeforeChildStart
AfterPriorResult
AtObservation
AtSuccess
AtFailure
AtFinalBoundary
```

### 8.9 Ensures and obligation behavior

State whether obligations are:

```text
final | intermediate | branch-local | handle-latent | failure-triggered | report-only
```

and what event discharges or reports them.

### 8.10 Failure behavior

State how operational failure propagates or is transformed.

The rule must distinguish operational failure from domain `Result.Err` values.

### 8.11 Provenance and report behavior

State the trace/report shape:

```text
sequential append | branch/DAG merge | observation merge | scope frame | failure mapping event
```

### 8.12 Handle behavior

Required for handle-producing and handle-observing operators.

Internal model:

```text
WorkflowHandle<A> = {
  processHandle : P<A>,
  contract      : HandleContract<A>,
}
```

or notation:

```text
P<A @ C>
```

Any operator returning a process handle must preserve latent child contract obligations on that handle.

### 8.13 Handle-latent obligation lifecycle

Handle-producing workflow operators introduce obligations that are not discharged when the handle is created. They are latent until the handle is observed, transferred, cancelled, or rejected by the enclosing workflow boundary.

MVP rule:

```text
Every workflow handle must be observed, transferred, or explicitly cancelled before the enclosing workflow boundary.
Silent drop of a workflow handle with latent obligations is invalid.
```

This rule is intentionally conservative. It prevents branch obligations, child failures, resource cleanup requirements, and provenance evidence from disappearing between `par` and the workflow boundary.

Allowed lifecycle outcomes:

```text
Observed:
  await/join/gather consumes the handle and discharges/imports/surfaces its HandleContract.

Transferred:
  a future operator explicitly moves the handle and its HandleContract into another governing scope.

Cancelled:
  a future cancellation operator consumes the handle, records cancellation evidence, and discharges or reports remaining obligations.

Rejected:
  static checking rejects a workflow that would let a handle escape or be dropped silently.
```

Deferred lifecycle outcomes:

```text
shared observation handles
monitor/subscription handles
detached supervisor-owned children
implicit cancellation on drop
handle escape outside the workflow scope
```

Future specs for `par`, `spawn`, `scatter`, `await`, `join`, and `gather` must state how they preserve or discharge handle-latent obligations.

### 8.14 Static coverage rule

Define the static acceptance condition and the evidence it produces, usually a refinement of:

```text
C_decl ⊒cov C_inferred ⇓ E_coverage
```

The rule should identify which component evidence is produced and how that evidence is preserved for verification/lowering/runtime use.

### 8.15 Runtime enforcement rule

State the checks that remain runtime responsibilities, such as provider availability, resource liveness, cancellation, terminal-state aggregation, failure routing, and report sink behavior.

### 8.16 Diagnostics and examples

Every operator-lift spec should include at least:

- one valid minimal example;
- one valid example with non-trivial contract behavior;
- one invalid example with expected diagnostic;
- one failure/obligation/provenance example if relevant.

Examples must use live Ash syntax or be explicitly marked as design pseudocode.

## 9. Heuristic Checklist for Designing a Lifted Operator

Use this checklist before adding or specifying any workflow-level operator derived from Proc.

```text
1. Identify the Proc operator class.
2. Write the Proc signature and behavior.
3. Write the Workflow signature.
4. Write the induced Contract signature.
5. Identify the ContractPlan constructor needed.
6. Define admission-envelope composition.
7. Define authority behavior and prove no widening.
8. Define resource behavior and check linear/exclusive cases.
9. Define check timing without flattening staged checks.
10. Define obligation behavior, including intermediate/branch/handle-latent obligations.
11. Define failure behavior and identity preservation.
12. Define provenance/report behavior.
13. Define handle behavior if handles are created or consumed.
14. Define static coverage checks and runtime enforcement checks.
15. State algebraic laws and equivalence assumptions.
16. Provide valid and invalid examples.
17. Add diagnostics for invalid authority/resource/failure/handle cases.
18. Mark deferred cases explicitly.
```

Heuristic consequences:

```text
If an operator cannot satisfy the template, do not lift it to Workflow yet.
If a contract component lacks operator-specific behavior, mark that operator incomplete.
If a behavior violates a global property, redesign the operator or narrow its scope.
If the only way to define the lift requires dynamic admission, make that an explicit capability and audit event.
If handle obligations cannot be preserved, defer the handle-producing operator.
```

## 10. Initial Operator Matrix

Future specs should elaborate this matrix into operator-specific rules.

```text
Proc operator   Proc behavior                      Contract behavior
--------------------------------------------------------------------------------
pure            immediate value                     empty contract
bind            sequential dependency               staged sequential contract bind
from_act        embed Act in Proc                   infer/require effect authority
yield           cooperative scheduler point         usually no contract change; cancellation checkpoint
fail            operational bottom                  route through current failure boundary
with_error      scoped failure handler              push/pop failure-boundary component
scope           process/dynamic scope               push labeled contract scope
par             start child processes               split/project authority/resources; attach obligations to handles
scatter         start N child processes             indexed par; attach obligations to each handle
spawn           create child process                child contract projection/admission; latent handle contract
await           observe one handle                  consume/discharge latent handle contract
join            observe two handles                 aggregate obligations/failures preserving identities
gather          observe N handles                   indexed aggregate obligations/failures
cancel          terminate child                     require cancellation authority; map cancellation failure/report
send            mailbox/channel send                require endpoint/resource authority; append communication trace
receive         mailbox/channel receive             require endpoint/resource authority; possible blocking/cancel behavior
```

The first implementation-grade slice should probably focus on:

```text
pure
bind
from_act
fail
with_error
par
await
join
gather
```

and defer:

```text
spawn
scatter
cancel
mailboxes/channels
monitor/subscriptions
parallel workflow applicative syntax
explicit dynamic admission
```

## 11. Filled Sketch: bind

### Proc signature

```text
bindᴾ : Proc<A> -> (A -> Proc<B>) -> Proc<B>
```

### Workflow signature

```text
bindᵂ : Workflow<A> -> (A -> Workflow<B>) -> Workflow<B>
```

### Contract signature

```text
bindᶜ : WorkflowContract<A>
      -> (A -> WorkflowContract<B>)
      -> WorkflowContract<B>
```

### Informal behavior

Run the first workflow. If it succeeds, instantiate and run the continuation workflow using the first result. Preserve the first workflow's scoped obligations and stage the continuation contract after the first result.

### Contract-plan construction

```text
contractPlan(bindᵂ(w, f)) =
  BindPlan(contractPlan(w), λa. contractPlan(f(a)))
```

### Admission-envelope behavior

```text
envelope(bindᵂ(w, f)) =
  envelope(w) ∪ staticUpperBound(λa. envelope(f(a)))
```

If no static upper bound exists, reject as a closed workflow unless explicit audited dynamic admission is present.

### Authority behavior

Sequential threading. The continuation may refine or narrow authority based on the first result, but not widen beyond the admitted envelope.

### Resource behavior

Sequential lifetimes. Exclusive resource reuse is valid only if first-segment lifetime ends before continuation use begins.

### Check timing

First requirements fire before the first segment. Continuation requirements fire after the first result and before the continuation body.

### Ensures and obligations

First ensures become intermediate scoped obligations. Continuation ensures apply to the continuation boundary. Outer workflow ensures apply to the final result.

### Failure behavior

Failure in the first segment skips the continuation. Failure in the continuation occurs in the continuation scope and propagates through outer boundaries.

### Provenance and reporting

Sequential trace append, preserving subworkflow scopes.

### Laws

Monad left identity, right identity, and associativity should hold up to scope-preserving contract equivalence.

## 12. Filled Sketch: par

### Proc signature

```text
parᴾ : Proc<A> -> Proc<B> -> Proc<(P<A>, P<B>)>
```

### Workflow signature

```text
parᵂ : Workflow<A> -> Workflow<B>
    -> Workflow<(WorkflowHandle<A>, WorkflowHandle<B>)>
```

### Contract signature

```text
parᶜ : WorkflowContract<A>
     -> WorkflowContract<B>
     -> WorkflowContract<(WorkflowHandle<A>, WorkflowHandle<B>)>
```

### Informal behavior

Start two workflow bodies as child processes. Project or split authority and resources into child scopes. Return handles carrying latent child contract obligations.

### Contract-plan construction

```text
contractPlan(parᵂ(w1, w2)) =
  ParPlan(contractPlan(w1), contractPlan(w2))
```

The result handles carry `HandleContract` values derived from the branch contracts.

### Admission-envelope behavior

```text
envelope(parᵂ(w1, w2)) =
  parEnvelope(envelope(w1), envelope(w2))
```

`parEnvelope` validates concurrent authority/resource compatibility.

### Authority behavior

Project or split parent authority into child scopes. Children are equal-or-less authorized than the parent. No branch may gain authority absent from the parent envelope.

### Resource behavior

Every non-copyable resource requires split/share/move behavior. Unsplittable exclusive resources cannot be used by both branches concurrently.

### Check timing

All-or-none pre-run admission before either child executes user code. Branch requirements are checked after child environment projection and before child start.

### Ensures and obligations

Branch obligations become latent obligations attached to returned handles.

### Failure behavior

Failures before valid handle creation are failures of the `par` operator. Failures after handle creation attach to child handles and surface at `await`, `join`, or `gather`.

### Provenance and reporting

Create branch/DAG provenance with child process identities. Branch reports remain branch-indexed until observation/merge.

### Laws

No duplicated linear resources. Preserve branch identity. Preserve handle-latent obligations. Preserve all-or-none pre-run admission.

## 13. Non-Goals and Deferrals

This note does not define a normative workflow runtime.

Deferred to future specs:

- exact public syntax for first-class `Workflow<A>`;
- exact Rust/internal carrier layout;
- exact parser/typechecker changes;
- user-defined contract components;
- dynamic admission beyond an audited capability sketch;
- full `spawn`, `cancel`, mailbox/channel, monitor, and subscription semantics;
- parallel/applicative workflow laws beyond `par`/handle design sketches;
- final contract lattice ordering and normalization algorithm;
- exact report sink and external audit commit behavior.

## 14. Spec and Plan Starting Points

Future specs/tasks can be derived in this order:

1. Define `Workflow<A>`, `WorkflowContract<A>`, `AdmissionEnvelope`, and `ContractPlan<A>` as semantic carriers.
2. Define the coverage preorder `C_decl ⊒cov C_body`, `Governs(C, p)`, and the operational/verification role of `CoverageEvidence`.
3. Define `Reconcile(C_header, C_body, E_coverage)` and the distinction between `HeaderContract`, `BodyContract`, `TotalContract`, `FullCoverageEvidence`, `DiagnosticEvidence`, and `ProvenanceEvidenceProjection`.
4. Define the contract inference judgment `Γ ⊢ᴾ p : Proc<A> ▷ C` for a minimal Proc subset.
5. Specify lifted `pure`, `bind`, and `from_act`.
6. Specify lifted `fail` and `with_error` using existing operational-bottom semantics.
7. Specify handle contracts for `par`, `await`, `join`, and `gather`, including handle-latent obligation lifecycle and silent-drop rejection.
8. Define contract normalization/equivalence and the first diagnostic set.
9. Only then decide whether workflow blocks are first-class `do:Workflow` blocks, declaration sugar, or both.

## 15. Suggested Future Spec Packet Decomposition

To keep future work implementation-grade, split the design into several narrow specs rather than one large workflow mega-spec.

### SPEC-A: Workflow Contract Carrier and Coverage Evidence

Owns:

```text
WorkflowContract<A>
AdmissionEnvelope
ContractPlan<A>
HeaderContract / BodyContract / TotalContract
C_decl ⊒cov C_body
Governs(C, p)
CoverageEvidence
Reconcile(C_header, C_body, E_coverage)
component variance rules
initial diagnostic evidence model
```

Default decisions for the first spec:

```text
Dynamic admission is forbidden unless a later spec introduces an explicit audited capability.
Contract normalization preserves structure; only identity rewrites are allowed.
```

### SPEC-B: Sequential Workflow Operator Lifting

Owns:

```text
pure
bind
from_act
fail
with_error
sequential check timing
intermediate obligations
failure-boundary stacking
Act requirement inference/export assumptions
```

This spec should decide how `Act<A>` exposes authority/capability requirements to workflow coverage. Opaque or imported Act values may require exported contract summaries.

### SPEC-C: Workflow Handle Contracts and Observation

Owns:

```text
WorkflowHandle<A>
HandleContract<A>
await
join
gather
observation discharge
unobserved handle rejection
aggregate failure/provenance merge
```

This spec should settle whether `WorkflowHandle<A>` is public surface syntax, internal typed metadata over `P<A>`, or both.

### SPEC-D: Parallel Workflow Lifting

Owns:

```text
par
parallel authority projection
resource split/share/move evidence
branch-local obligations
branch provenance/report merge
all-or-none pre-run admission
```

This spec should build on SPEC-C rather than defining handle behavior from scratch.

### Later specs

Defer:

```text
spawn
scatter
cancel
mailboxes/channels
monitor/subscription handles
dynamic admission
first-class do:Workflow syntax
user-defined contract components
aggressive contract normalization/equivalence
```

## 16. Design Payoff

This note turns workflow design from intuition into a repeatable method:

```text
Proc operator opᴾ
  induces contract operator opᶜ
  which induces workflow operator opᵂ.
```

Once those induced behaviors are specified, Workflow is no longer the weak link in the tower. It becomes the governed, contract-indexed lift of Proc.
