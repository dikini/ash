# NOTE-040: Minimal Authority Composition Without Role or Policy Forms

**Date:** 2026-08-05
**Status:** Living document — minimal authority-composition exploration
**Purpose:** Record future research for expressing role-like relations and policy-like decisions
compositionally without dedicated declarations or row-item kinds. The current implementation has
removed those language forms; this note does not authorize reintroducing them or adding authority
semantics. Any future work must preserve explicit facts, requirements, admission, discharge, and
provenance through the canonical source → finalization → Core/CPS → Engine route.

Companion to NOTE-020 through NOTE-025 and the component/resource explorations in `docs/ideas/`.
This is a pre-spec research note. It is not part of the current execution route and does not
authorize implementation work.

## Pre-Spec Delta

The current implementation has no first-class `role` or `policy` declarations or row items. This
note records a future composition baseline:

- no dedicated `role_definition` or `policy_definition` grammar production;
- no `role` or `policy` computation-row item in the minimal profile;
- no capability declaration, capability grant, or first-class capability token;
- role-like and policy-like behavior expressed through types, ordinary values, functions, interfaces/impls, operations, resources, facts/evidence, obligations, admission, and provenance;
- ordinary nominal/component identity may name a reusable composition later, but identity does not add authority merely by existing.

The research question is not whether roles and policies are useful concepts. It is whether their semantic content needs language packaging after the compositional baseline has been specified and evaluated.

## 0. Motivation

Ash needs to express authorization situations such as:

> Which subject may request which operation against which target and resources, under which facts, evidence, obligations, and runtime policy?

The historical capability vocabulary compressed too many answers into one word. The implementation
keeps operations, resources, evidence, obligations, and admission as separate concerns; it does
not add dedicated role/policy packages.

That packaging may be useful, but it creates a new grammar family, namespace, row normalization and discharge rules, export model, Core carrier, runtime admission path, provenance representation, and diagnostic taxonomy. Target Ash should not pay that cost merely to abbreviate compositions already expressible through ordinary mechanisms.

The minimal hypothesis is therefore:

```text
role-like standing relations and policy-like decisions
are library/composition patterns over explicit facts and requirements,
not foundational source-language forms.
```

The hypothesis survives only if it preserves authority boundaries, supports useful parameterization, remains auditable, and does not make ordinary programs unreasonably opaque or repetitive.

## 1. Separate essence from identity

A stable name is useful for diagnostics, audit, export, versioning, and reuse. It is not the semantic essence of either a role or a policy.

| Concern | Essence without identity | Optional identity wrapper |
|---|---|---|
| Role-like relation | A validated standing relation between a subject and a scope, plus its consequences. | An ordinary nominal type, component, or exported binding naming that reusable relation. |
| Policy-like decision | A decision relation over a concrete authorization request, validated facts, and evidence. | An ordinary nominal type, component, or exported binding naming/versioning that evaluator. |

The distinction prevents invalid implications:

```text
name of a role-like relation       != a subject's validated membership
name of a policy-like evaluator    != a permit decision
membership fact                    != unrestricted operation authority
policy decision                     != provider/handler frame
row requirement                     != any of the above
```

### 1.1 Role-like essence

A role-like relation packages a standing, subject-relative classification and the consequences that follow from a validated instance of it:

```text
RoleEssence = {
    membership: Subject x Scope x FactSnapshot x EvidenceSet -> ValidatedMembershipFact | rejection,
    candidate_entitlements: ValidatedMembershipFact -> CandidateEntitlementScopes,
    obligations: ValidatedMembershipFact -> ObligationSet,
    delegation: ValidatedMembershipFact x DelegationEvidence
        -> DerivedValidatedMembershipFact | rejection,
    attenuation: CandidateEntitlementScopes -> narrower CandidateEntitlementScopes,
}
```

This is semantic notation, not Ash syntax. A `ValidatedMembershipFact` is a provenance-carrying fact whose evidence has been checked; it must not be freely forged as an ordinary record. It can produce only *candidate* entitlement scopes. It is not an admitted entitlement and cannot itself discharge an operation. Only the later policy and admission steps may turn compatible candidates into request-scoped `AdmittedEntitlement` facts associated with an admitted frame.

### 1.2 Policy-like essence

A policy-like decision packages judgment of one concrete request:

```text
PolicyEssence = {
    domain: AuthorizationRequestShape,
    evaluate: AuthorizationRequest x FactSnapshot x EvidenceSet -> Decision,
    explain: Decision -> DecisionEvidence,
}

Decision = Permit | Deny(reason) | Require(requirements) | Defer(unresolved_facts)
```

The eventual decision algebra may differ, but it must distinguish a final permit from denial, missing preconditions, and unavailable decision facts.

### 1.3 The essential difference

```text
role-like relation   = standing relation -> consequences
policy-like decision = request x facts -> decision
```

A validated role-like relation may be input to a policy-like decision. It is not a substitute for one. A policy may deny a request from a standingly eligible subject because of amount limits, separation of duty, expiry, delegation scope, target ownership, or workflow state.

## 2. Minimal compositional baseline

The baseline should use existing or already-targeted Ash concepts before adding grammar.

| Need | Minimal compositional carrier |
|---|---|
| Action requested | Impl-type-qualified interface operation identity. |
| State domain and access | Resource kind, lexical slot, access mode, lifecycle/process policy. |
| Subject | Ordinary typed value supplied by a caller or runtime boundary. |
| Trusted subject/fact claim | Validated evidence/fact carrier with provenance, not an arbitrary user-created value. |
| Standing relation | Ordinary function, interface implementation, or library composition over validated facts. |
| Decision rule | Ordinary pure function over an explicit request/fact snapshot, or an operation when evaluation must consult explicit resources. |
| External fact acquisition | Ordinary operation and resource requirements. |
| Obligation | Existing/future obligation or evidence/fact carrier, with its own discharge lifecycle. |
| Enforcement | Admitted provider/handler frame and resource bindings. |
| Audit | Evidence, decision, resource, and frame provenance sidecars. |

This does not require authority to be reducible to pure values. It separates pure evaluation of a supplied snapshot from effectful, explicitly admitted acquisition and validation of that snapshot.

### 2.1 Facts and evidence are not ordinary assertions

An ordinary `Principal` value may claim an identity. It does not establish that the computation acts on that principal's behalf.

The authority boundary must distinguish:

```text
subject identifier       -- ordinary value or boundary input
fact claim               -- asserted relation about a subject/context
evidence                 -- material supporting a fact claim
validated fact           -- evidence checked under an admitted verifier/policy
admitted authority fact  -- validated fact accepted for one execution context
```

A minimal design may represent some of these through opaque runtime-backed values or sidecars rather than a new source construct. The decisive invariant is semantic: a caller cannot gain authority merely by substituting `s := Alice` into an open subject parameter.

### 2.2 Parameterization remains ordinary

A reusable computation may leave the subject, scope, target, facts, or evidence open through ordinary parameters and type parameters. It should express a symbolic requirement such as:

```text
an admitted subject s must satisfy relation Reviewer(s, tenant)
for this operation and target
```

The implementation remains checked with that requirement open. A later boundary supplies a concrete subject and the validating facts. Parameterization must not create implicit subject lookup, implicit delegation, or value-to-authority promotion.

## 3. Computation rows, requirements, and entitlements

The minimal proposal changes where authority-related information lives. It does not make it disappear.

### 3.1 Direct requirements remain row items

A computation row should retain direct operational requirements:

```text
operation requirements
resource requirements
failure/process/channel requirements where applicable
evidence or obligation requirements when their independent design admits them
```

For example, a provider realization that must fetch verified identity and record an audit event has ordinary dependencies on those operations and resources. Those dependencies must remain visible in its row or checked dependency summary.

### 3.2 No role/policy shortcut in the minimal profile

The minimal profile does not add these as primitive row kinds:

```text
role TenantReviewer
policy TransferApproval
```

Their consequences instead arise from explicit composition. A function may request `Transfer::approve`; the admitted realization for that operation may require identity lookup, validated relation evidence, policy evaluation, audit recording, and ledger access. Each dependency is represented using its own ordinary operation/resource/evidence/obligation mechanism.

This keeps the row honest about what the computation requires. It avoids a named role or policy item concealing the operations, resources, evidence, and obligations that actually matter.

### 3.3 Entitlement is not a requirement

This distinction is central:

| Item | Meaning | May appear as a row requirement? |
|---|---|---|
| Operation requirement | The computation needs a matching admitted operation frame. | Yes. |
| Resource requirement | The realization needs a compatible admitted resource binding. | Yes. |
| Evidence requirement | The computation needs a valid fact/evidence strategy. | Potentially, once independently designed. |
| Obligation | Something must be performed, recorded, or remain tracked. | Potentially, with its own lifecycle. |
| Entitlement | A validated relation/decision may support discharge of a requirement under a scope. | No: it is an admission/discharge fact, not a request. |
| Permit decision | A policy-like evaluator accepted one concrete request. | No: it is a decision/provenance fact. |

A row never acquires an entitlement by mentioning it. Entitlements belong in the admitted environment and provenance, where they can be scoped to the exact subject, operation, target, resources, policy version, and execution instance.

### 3.4 Consequence composition

A role-like relation may entail candidate operation/resource scopes and obligations. A policy-like decision may accept, narrow, or reject those candidates. Admission then checks the final combination.

```text
validated relation facts
    -> candidate entitlements and obligations
    -> policy-like decision for concrete request
    -> admission checks provider/resource/evidence compatibility
    -> admitted frame may discharge operation
```

No step silently turns an alias, type, imported name, relation name, or row item into authority.

## 4. Admission, discharge, and provenance

The authority boundary remains explicit.

### 4.1 Admission input

Admission receives at least:

```text
checked operation/resource requirements
concrete subject and request inputs
admitted evidence-verification route
validated fact snapshot or explicit fact-acquisition operations
selected provider/handler realizations
selected resource instances and access/lifecycle policies
runtime profile and process scope
```

It validates that all facts, relation consequences, decisions, and resource bindings support the requested execution. Missing facts, invalid evidence, incompatible policy decisions, or unavailable frames reject before the protected operation runs.

### 4.2 Discharge

At runtime, an operation is discharged only by an admitted handler/provider frame. A role-like relation or policy-like decision may justify that frame's availability for a request, but neither is itself the frame.

```text
operation raise
    -> matching admitted frame lookup
    -> frame's validated authority context
    -> typed operation body or host primitive
    -> provenance/report outcome
```

This preserves the target rule that rows are requirements, not grants, and that handlers/providers rather than rows install dynamic discharge behavior.

### 4.3 Provenance

The admitted result must preserve enough information to answer:

```text
which subject acted?
which operation and target were requested?
which evidence and verifier supported the facts?
which relation/policy compositions were applied?
which decision and obligations resulted?
which provider frame and resource instances executed the action?
which upstream authorities were delegated, narrowed, or derived?
```

Nominal identity is useful here, but it can be ordinary component/type/binding identity. It does not require a dedicated role or policy declaration category.

## 5. Grammar, types, semantics, and worked example

This section records the four dimensions any eventual decision must preserve.

### 5.1 Grammar delta under the minimal hypothesis

The minimal profile adds no `role` or `policy` grammar form and removes them as primitive computation-row item families. Existing forms remain sufficient:

```text
ordinary nominal types/newtypes
ordinary values and records
functions and callable types
interfaces and impls
operation/resource/evidence/obligation rows where independently justified
handlers and explicit runtime/application admission boundaries
```

This is a research baseline, not a target grammar amendment. Dedicated role/policy grammar remains
removed unless a future approved specification explicitly changes that decision.

### 5.2 Type-level model

At the semantic level, a checked computation may carry ordinary rows plus authority-sidecar requirements:

```text
Sigma ; Gamma ; Delta ; Phi ; Omega |- e : A ! rho
```

where:

- `Sigma` contains static declarations, interfaces, impl identities, types, and component facts;
- `Gamma` contains ordinary values, including a subject value when supplied by the caller;
- `Delta` contains resource slots, kinds, and access/ownership requirements;
- `Phi` contains statically resolved provider/handler requirements;
- `Omega` records unresolved authority/evidence/obligation conditions needed by admission;
- `rho` contains direct computation requirements.

`Omega` is explanatory notation for the research problem, not a committed new source kind or Core carrier. A key work item is to determine whether existing evidence/obligation sidecars can represent it without introducing a parallel authority-row system.

### 5.3 Semantic model

A compact admission relation is:

```text
H ; F ; V |- Omega ; Delta ; Phi  => admitted(frame, provenance)
```

where:

- `H` contains resource instances, lifecycle state, and resource provenance;
- `F` contains provider/handler frames;
- `V` contains evidence-validation and fact-provenance results.

The relation must establish that the concrete request is permitted under validated facts and must never manufacture missing host authority. It may install a narrowed or derived frame only when its declared inputs and provenance justify that result.

### 5.4 Worked composition example

The following is semantic pseudo-code, not proposed Ash syntax:

```text
request = {
    subject: s,
    operation: Transfer::approve,
    target: transfer,
    resources: { ledger @ write },
}

identity_fact = Identity::verify(s, identity_evidence)
membership = reviewer_membership(s, transfer.tenant, identity_fact, assignment_evidence)

candidates = reviewer_consequences(membership)
decision = transfer_approval(request, candidates, approval_facts, delegation_facts)

admit(
    request,
    decision,
    provider = TransferProvider,
    resources = { ledger -> iota_ledger },
    provenance = { identity_fact, membership, decision }
)
```

`reviewer_membership` and `transfer_approval` can be ordinary compositional functions over supplied validated facts. If they need external facts, those facts are obtained through explicit operations/resources before their pure decision phase. The final `admit` operation is a runtime boundary, not an ordinary source-level authority grant.

### 5.5 Static decision contracts

A decision is a runtime result, but its evaluator can have a static contract describing when
evaluation is well-formed. The type system should establish that a decision is *possible to
evaluate* from a declared request shape, fact context, and dependency row; it must not attempt to
decide whether the runtime result is `Permit`.

The useful distinction is:

```text
requires    = facts that must be available before evaluation
uses        = operations/resources the evaluator may inspect
considered  = facts and predicates actually inspected on the runtime path
```

`requires` and `uses` are static contract information. `considered` belongs in runtime decision
provenance because branch selection and data-dependent evaluation are dynamic.

A conceptual decision contract is:

```text
DecisionSpec<
    Request,
    RequiredFacts,
    Reads,
    Result
>
```

For example:

```text
TransferApprovalSpec<S, T> =
    DecisionSpec<
        Request       = TransferRequest<S, T>,
        RequiredFacts = AllOf<
            Identity<S>,
            ReviewerMembership<S, T>,
            TransferLimit<T>,
            NotSelfApproval<S, T>
        >,
        Reads         = TransferApprovalReads,
        Result        = TransferDecision<S, T>
    >
```

The evaluator may then be modeled as:

```text
decide(
    request: TransferRequest<S, T>,
    facts: FactContext<RequiredFacts>
) -> {TransferApprovalReads} TransferDecision<S, T>
```

`FactContext<RequiredFacts>` is explanatory notation. It may eventually be represented by
explicit typed parameters, a typed record, or a library-level evidence context. It must contain
validated, scope-bound facts rather than ordinary claims. Fact types should bind the subject and
scope to the request where possible, so `ReviewerMembership<S, T>` cannot silently satisfy a
requirement for another subject or tenant.

The `Reads` component remains an ordinary computation row and may use a transparent alias or
diagnostic group:

```text
effect group TransferApprovalReads = {
    Ledger::read,
    ApprovalHistory::read,
    Audit::append,
};
```

This group records the evaluator's dependencies. It is not a policy item, does not grant
authority, and does not prove that the runtime decision will permit the request.

The static checker should establish:

1. the request has the expected subject, operation, target, and resource shape;
2. every required fact is supplied or is statically known to be available;
3. fact parameters and scopes align with the request;
4. external fact acquisition and validation are represented by explicit operations/resources;
5. the evaluator returns a complete decision algebra such as `Permit`, `Deny`, `Require`, or
   `Defer`.

It should not claim to establish that runtime facts are true, current, unrevoked, or sufficient for
`Permit`. A permit result is still a request-scoped runtime fact that admission may accept or
reject; it is not a row requirement and is not an authority grant created by the type checker.

### 5.6 Decision fact categories

A decision evaluator may use the following categories of input:

| Input | Static treatment | Runtime responsibility |
|---|---|---|
| Request data | Ordinary typed values with a checked request shape | Check concrete values and predicates. |
| Validated facts | Required fact types indexed by subject, scope, and relevant domain | Validate freshness, revocation, provenance, and bindings. |
| Derived facts | Typed outputs of earlier fact/relation computations | Preserve derivation and scope provenance. |
| Prior decisions | Scoped decision facts tied to the exact request domain | Check expiry, independence, quorum, and replay constraints. |
| External state | Explicit operation/resource requirements in `Reads` | Acquire a concrete snapshot or reject admission. |
| Raw evidence | Input to an explicit verifier, not direct policy input | Verify evidence and produce validated facts. |
| Obligations | Explicit required inputs or runtime outputs with their own lifecycle | Track discharge, failure, and audit outcome. |

Negative requirements must be represented by validated facts such as `NotSelfApproval<S>`, not by
the mere absence of a positive fact. Likewise, a decision must not receive an unscoped `Permit`
value and treat it as authority; any prior decision must be bound to the request, subject, target,
resource scope, and relevant decision version.

The initial static baseline may use `AllOf` requirements. `AnyOf`, conditional requirements, and
quorum requirements need an explicit type-level composition model. Where the type system cannot
determine a branch's exact fact needs, the static requirement should conservatively include every
fact the evaluator may inspect, while runtime provenance records the facts actually considered.

### 5.7 Opaque evidence-carrying fact types

Validated facts should be represented by opaque nominal values whose constructors are private to
the defining module or verifier component. Public code receives them only through smart
constructors or verifier operations that check the supplied claim, evidence, scope, and runtime
validation route.

Conceptually:

```text
opaque type ReviewerMembership<S, T> = {
    proposition: Membership<S, Reviewer, T>,
    provenance: FactProvenance,
    scope: FactScope,
}

verify_reviewer_membership(
    subject: S,
    tenant: T,
    evidence: Evidence
) -> {ReviewerVerificationReads} ReviewerMembership<S, T>
```

The private constructor gives the type-level contract an anti-forgery boundary:

```text
ordinary value or claim       != validated fact
type name                     != validated fact
deserialized payload          != validated fact
smart-constructor result      = validated fact, subject to runtime validity
```

The type proves that the designated construction path has completed and that the fact is indexed
by the expected subject and scope. It does not prove that the underlying external world is
permanently true. Expiry, revocation, replay, deserialization, and host trust remain runtime
validation concerns. Imported or deserialized fact values must therefore retain provenance and be
revalidated before they can satisfy a decision contract.

Smart constructors may derive a narrower fact or entitlement scope, but must not widen scope,
replace the subject, discard provenance, or manufacture a permit. If validation requires external
state, the verifier's operation/resource dependencies remain visible in its computation row.
This keeps constructor privacy and row accounting complementary: privacy prevents source-level
forgery, while rows and admission account for the authority needed to create and use the fact.

## 6. Research and design programme

### 6.1 Define the minimal authority vocabulary

Specify a minimal typed vocabulary for:

```text
Subject
AuthorizationRequest
Target/Scope
FactClaim
Evidence
ValidatedFact
Decision
Obligation
EntitlementScope
Provenance
Delegation/attenuation relation
```

Questions:

1. Which are ordinary user-visible values, opaque values, or runtime sidecars?
2. Which identities are static, which are dynamic, and which are lexical brands?
3. How are fact scope, expiry, revocation, and subject binding represented without ambient state?

### 6.2 Define evidence and fact validation

Specify how an ordinary claim becomes a validated fact:

```text
claim + evidence + admitted verifier + request scope -> ValidatedFact | rejection
```

The research must establish anti-forgery, replay, substitution, delegation, and revocation properties. It must also decide whether validation is represented by explicit operations in `rho`, evidence obligations in `Omega`, or both.

### 6.3 Define relation and decision combinators

Develop ordinary library-level combinators for:

```text
all_of / any_of / not
scope restriction
amount or target limitation
separation of duty
delegation and attenuation
obligation accumulation
permit/deny/require/defer composition
```

The combinators need algebraic laws, especially monotonicity or non-monotonicity, short-circuit behavior, provenance merge rules, and conflict behavior. This is where a later packaged form might earn value if ordinary composition proves inadequate.

### 6.4 Determine row and sidecar interaction

For each authority-relevant action, determine whether it belongs in:

```text
rho    direct operation/resource/failure/process requirement
Omega  evidence/authorization/obligation condition pending admission
H/F/V  dynamic resource/frame/validated-fact environment
trace  provenance, decision, and obligation outcome
```

The design must reject two bad outcomes:

1. hiding real effectful evidence acquisition behind a pure policy/role abstraction;
2. treating an entitlement or permit as a row requirement that source code can manufacture.

### 6.5 Define provider and handler integration

Specify how a provider/handler realization receives an admitted authority context and how it proves non-escalation:

```text
provider dependencies + admitted facts + admitted resource bindings
    -> one scoped frame
```

Host-backed and Ash-defined realizations must have the same declared dependency and provenance contract. A provider may narrow, compose, or record authority, but may not discover or manufacture undeclared external authority.

### 6.6 Define diagnostics and audit output

Develop diagnostics that distinguish:

```text
unresolved operation requirement
missing resource binding
missing/invalid evidence
unvalidated standing relation
policy-like denial
required additional evidence or approval
expired or revoked delegation
missing provider/handler frame
```

Audit output must name the concrete subject, operation, target, resource/frame identities, evidence summaries, decision path, and outstanding or discharged obligations without exposing secrets.

### 6.7 Evaluate representative workloads

Before choosing new syntax, evaluate the composition against at least:

1. tenant-scoped reviewer/editor/approver relations;
2. amount-limited transfer approval with a second-approval requirement;
3. separation of duty and self-approval prohibition;
4. delegated, attenuated service authority;
5. host-backed external resource access with an audit obligation;
6. process-scoped resource sharing and revocation/expiry;
7. a reusable library that exports an open-subject authorization contract.

For each workload, record source verbosity, type/row summaries, Core/CPS shape, admission inputs, provenance output, diagnostic quality, and whether a named role/policy package removes genuine complexity rather than merely shortening text.

## 7. Criteria for any later language package

A special role or policy form should be added only if the compositional baseline fails a concrete criterion:

1. **Semantic gain:** it enforces a law or scope discipline unavailable through ordinary types, interfaces, impls, rows, and admission records.
2. **Static gain:** it permits sound checking of membership, delegation, attenuation, or decision compatibility that ordinary composition cannot express.
3. **Operational gain:** it reduces dynamic ambiguity without creating a second evaluator or authority route.
4. **Audit gain:** it supplies stable identity/provenance that ordinary nominal/component identity cannot provide.
5. **Ergonomic gain:** multiple representative workloads duplicate the same composition and the package removes error-prone structure, not merely words.
6. **Cost discipline:** it has a defined grammar, type rule, row behavior, Core/CPS representation, admission rule, provenance form, and diagnostic model that are simpler overall than the composition it replaces.

Until those criteria are met, `role` and `policy` remain concepts and library patterns, not target-Ash primitives.

## 8. Provisional decisions and non-goals

1. **Composition first.** Target exploration begins from ordinary language primitives and explicit admission, not from a new capability/role/policy syntax.
2. **No authority from names.** A type, component, imported binding, relation name, or policy name never grants or discharges authority on its own.
3. **No ambient fact lookup.** External facts, verifiers, policy data, and mutable state require explicit operations/resources or boundary-supplied validated snapshots.
4. **Entitlement is not a row item.** It is an admitted, scoped, provenance-carrying fact that may justify discharge.
5. **Identity is separable.** Ordinary nominal/component identity may name reusable compositions without creating a language category.
6. **No implementation mandate.** This note defines research/design work; it does not authorize removal of existing grammar or runtime paths.

## 9. Open questions

1. Can existing evidence and obligation designs represent `Omega`, or is a distinct authority-sidecar carrier necessary?
2. Which decision algebra preserves enough information for obligations, denial, deferred facts, and compositional provenance?
3. Are role-like standing relations best represented as typed predicates, interfaces, records of functions, or a mixture of these ordinary forms?
4. What is the minimum opacity/branding discipline needed to prevent validated fact or entitlement forgery?
5. How should revocation and expiry affect cached fact snapshots, provider frames, and long-running process/workflow execution?
6. Which relation/policy combinators are monotone, and where must Ash make non-monotonicity explicit?
7. How do delegation and attenuation compose with resource scopes and process split/join policies?
8. Which facts may be evaluated purely from a snapshot, and which require explicit admitted operations at decision time?
9. Should a reusable authorization evaluator be an ordinary interface operation, a pure function over a snapshot, or both at different boundaries?
10. After workload evaluation, do named role/policy forms meet the criteria in Section 7, or should ordinary nominal/component identity remain sufficient?
11. Should `RequiredFacts` be represented as a proposition/evidence constraint set, a typed context
    interface, or both, while keeping it separate from the computation row?
12. Which conditional fact requirements (`AnyOf`, branch-specific requirements, and quorum) are
    necessary for the first useful static decision-contract slice?
13. What provenance, revalidation, expiry, and revocation rules apply when opaque fact values cross
    module, process, persistence, or serialization boundaries?

## 10. References

### Internal references

- **NOTE-020: Computation Row Taxonomy and Pure Computation.** Defines computation rows as typed requirement accounting rather than authority grants.
  [`docs/notes/NOTE-020-COMPUTATION-ROW-TAXONOMY.md`](NOTE-020-COMPUTATION-ROW-TAXONOMY.md)
- **NOTE-021: Row, Callable, Where, and Fact Syntax.** Explores row/evidence syntax and named fact declarations; its current role/policy row examples are a pre-spec comparison point.
  [`docs/notes/NOTE-021-ROW-CALLABLE-WHERE-AND-FACT-SYNTAX.md`](NOTE-021-ROW-CALLABLE-WHERE-AND-FACT-SYNTAX.md)
- **NOTE-022: Effects as Interfaces — Declaration Side.** Establishes interfaces as operation sorts rather than capability declarations.
  [`docs/notes/NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md`](NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md)
- **NOTE-023: Handler Surface — Dispatch Side.** Defines handler installation and row discharge direction.
  [`docs/notes/NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md`](NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md)
- **NOTE-025: Effect Identity via Sorts and Impls.** Establishes impl types as operation-identity carriers and separates identity from behavior.
  [`docs/notes/NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md`](NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md)
- **NOTE-034: Contract-Capability Boundary.** Separates pure predicates from authority/evaluator access; historical capability terminology requires target reconciliation.
  [`docs/notes/NOTE-034-CONTRACT-CAPABILITY-BOUNDARY.md`](NOTE-034-CONTRACT-CAPABILITY-BOUNDARY.md)
- **TYPES-005: Component Abstraction with Interfaces and Private Types.** Explores static component identity without conflating it with runtime realization.
  [`docs/ideas/type-system/TYPES-005-component-abstraction-with-interfaces.md`](../ideas/type-system/TYPES-005-component-abstraction-with-interfaces.md)
- **RESOURCES-001: Resource Providers and Runtime Identity.** Explores admitted resource/provider realization and provenance.
  [`docs/ideas/runtime/RESOURCES-001-resource-providers-and-runtime-identity.md`](../ideas/runtime/RESOURCES-001-resource-providers-and-runtime-identity.md)
- **Component-resource phase boundary.** Defines the static/dynamic identity and admission boundary used by this note.
  [`docs/ideas/architecture/COMPONENT-RESOURCE-PHASE-BOUNDARY.md`](../ideas/architecture/COMPONENT-RESOURCE-PHASE-BOUNDARY.md)
- **SPEC-095b: Target Grammar; SPEC-096b: Target Effect System; SPEC-097b: Target Type System.** Current draft role/policy syntax, row kinds, and target discharge model to reconcile against the minimal baseline.
  [`docs/spec/SPEC-095b-TARGET-GRAMMAR.md`](../spec/SPEC-095b-TARGET-GRAMMAR.md)
  [`docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md`](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
  [`docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md`](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- **PLAN-183: Operation and Authority Model.** Implemented bounded operation-authority/non-granting row model.
  [`docs/plan/PLAN-183-OPERATION-AUTHORITY-MODEL.md`](../plan/PLAN-183-OPERATION-AUTHORITY-MODEL.md)

### External references

- **Martín Abadi, Michael Burrows, Butler Lampson, and Gordon Plotkin, "A Logic of Authentication"** (1993). Foundational account of principals, statements, delegation, and authentication reasoning; useful for distinguishing asserted identities from validated authority facts.
  https://doi.org/10.1145/210332.210334
- **Ruichuan Chen et al., "Zanzibar: Google's Consistent, Global Authorization System"** (2019). A production authorization-system reference for relationship facts, consistency, and request-scoped authorization decisions.
  https://research.google/pubs/zanzibar-googles-consistent-global-authorization-system/
- **W3C, "Verifiable Credentials Data Model v2.0"** (2025). Canonical model for separating claims, evidence, issuers, holders, verification, and credential status.
  https://www.w3.org/TR/vc-data-model-2.0/
- **Open Policy Agent, "Policy Language"**. Practical prior art for keeping policy evaluation over explicit input data distinct from policy-data acquisition and enforcement.
  https://www.openpolicyagent.org/docs/latest/policy-language/

## 11. Changelog

| Date | Change |
|------|--------|
| 2026-08-05 | Initial version. Defines the composition-first research baseline for role-like relations and policy-like decisions without dedicated target-Ash role/policy forms. |
| 2026-08-05 | Clarified that validated membership produces only candidate entitlement scopes; request-scoped admitted entitlements arise only after policy and admission checks. Linked internal references. |
| 2026-08-06 | Added the static decision-contract model: required validated facts and computation-row dependencies are checked at type level, while runtime decisions and actually considered facts remain dynamic and provenance-carrying. |
| 2026-08-06 | Added opaque evidence-carrying fact types with private constructors and public smart constructors as the anti-forgery boundary; runtime validity, expiry, revocation, and revalidation remain dynamic. |
