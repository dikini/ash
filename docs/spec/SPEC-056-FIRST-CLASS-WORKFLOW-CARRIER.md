# SPEC-056: First-Class Workflow Carrier

**Status:** Implemented MVP / historical WorkflowForm carrier language superseded for target planning
**Date:** 2026-04-29
**Promotes:** [DESIGN-033](../design/DESIGN-033-WORKFLOW-CONTRACT-OPERATOR-LIFTING.md)
**Builds on:** [SPEC-048](SPEC-048-PROC-LIBRARY.md), [SPEC-049](SPEC-049-PROCESS-RUNTIME-SEMANTICS.md), [SPEC-050](SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md), [SPEC-051](SPEC-051-WORKFLOW-SEMANTICS.md), [SPEC-054](SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md), [SPEC-055](SPEC-055-MONAD-COMPREHENSION-SYNTAX.md)
**Related:** [SPEC-003](SPEC-003-TYPE-SYSTEM.md), [SPEC-004](SPEC-004-SEMANTICS.md), [SPEC-047](SPEC-047-ACT-MONAD.md), [SPEC-052](SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md), [SPEC-053](SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md)
**Plan:** [PLAN-104](../plan/PLAN-104-FIRST-CLASS-WORKFLOW-CARRIER.md)
**Implementation Tasks:** [TASK-768](../plan/tasks/TASK-768-first-class-workflow-spec-plan-packet.md) through [TASK-779](../plan/tasks/TASK-779-first-class-workflow-closeout.md)

## Post-target reconciliation: WorkflowForm is not revived

This spec records the Phase 104/108-era first-class workflow MVP. Its `WorkflowForm` language is historical compatibility and implementation-slice evidence, not a target-state mandate.

WorkflowForm is not revived as a primary syntax, type, Core term, CPS term, or runtime carrier. Target-state planning should route workflow-like facts through the ambient computation model: computation rows, Core/CPS carriers, trace and monitor sidecars, contract/evidence artifacts, obligations, admission/authority facts, provenance, and reporting metadata.

Legacy workflow declarations and the first-class workflow MVP may still translate through compatibility paths described here, but new implementation work should be framed as ambient workflow-fact reconciliation. Do not create a new WorkflowForm implementation backlog from this spec.

## 1. Summary

Ash adds `Workflow<A>` as a first-class computation constructor and monadic target.

The historical MVP semantic model was described as a synchronized product, not a sum or pair of independently maintained artifacts:

```text
Workflow<A> = synchronized_product(Proc<A>, WorkflowContract<A>) via WorkflowForm<A>
WorkflowContract<A> = AdmissionEnvelope + ContractPlan<A>
```

`Proc<A>` owns the process computation. `WorkflowContract<A>` owns the governance projection: admission envelope, staged contract plan, coverage evidence, failure/reporting boundary, and provenance obligations. In this historical MVP, `Workflow<A>` was described as an aligned carrier whose projections are derived from the same preserved `WorkflowForm<A>`; target-state docs supersede that carrier story with ambient workflow facts.

This spec defines the first implementation slice for first-class workflows:

1. `Workflow<A>` as a public type constructor.
2. A compiler-known qualified `workflow` namespace with Monad-shaped operations analogous to `proc`.
3. Compiler-known `do:Workflow` target resolution through the existing SPEC-054 typed-do machinery.
4. `[...]: Workflow` comprehensions through the existing SPEC-055 comprehension machinery.
5. Historical blocking workflow-form/projection semantic gate: `WorkflowForm`, stable node/alignment identities, projection events, staged `ContractPlan`, obligation vocabulary, and equality strata. Target-state follow-up must migrate any still-useful facts into ambient rows/Core/CPS/sidecar carriers, not revive `WorkflowForm`.
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
- Historical structure-preserving `WorkflowForm` lowering with Proc/Contract/check/resource/failure/provenance projections; target planning treats these as workflow facts to reconcile into ambient rows/Core/CPS/sidecars.
- Public `workflow` namespace operations:
  - `workflow::unit`
  - `workflow::bind`
  - `workflow::then`
  - `workflow::from_proc`
  - `workflow::from_act`
  - `workflow::requires`
  - `workflow::ensures`
- Closed first-slice `WorkflowForm` grammar, projection-event vocabulary, alignment identity model, staged `ContractPlan`, obligation vocabulary, and equality strata before the historical Rust carrier implementation began. This is not a new target implementation mandate.
- Compiler-known `Workflow` typed-do dictionary.
- `do:Workflow { ... }` typed elaboration.
- `[...]: Workflow` comprehension typed elaboration.
- Header/body/total contract reconciliation model for existing workflow declarations.
- Export/import of enough workflow type/contract summaries for modular checking.
- Diagnostics for unsupported workflow targets, wrong bind RHS constructor, missing coverage, and explicit-lift requirements.

Out of scope for the first implementation slice:

- User-defined `Monad<M>` implementations.
- Public type parameterization such as `Workflow<C, A>`.
- Public construction of arbitrary contract values by user syntax, except compiler-known opaque contract arguments accepted by `workflow::requires`, `workflow::ensures`, and their statement-form sugar.
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

The workflow namespace is:

```text
workflow
```

First-slice `workflow::...` names are compiler-known qualified builtins registered in the same namespace style as the existing `proc::...` builtins. They are not implicitly imported as unqualified names by `do:Workflow`, and this spec does not require an ordinary Ash stdlib module implementation before the compiler-known surface exists. A future stdlib/module-backed implementation may provide the same exports, but it must preserve the public qualified names and module summary behavior described in §11.

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

Qualified builtin resolution requirements:

- `workflow::unit`, `workflow::bind`, `workflow::then`, `workflow::from_proc`, `workflow::from_act`, `workflow::requires`, and `workflow::ensures` resolve only as compiler-known qualified names in the first slice.
- Unqualified `unit`, `bind`, `then`, `from_proc`, `from_act`, `requires`, or `ensures` are not introduced by selecting `do:Workflow` and must resolve only if the user has explicitly defined or imported ordinary names with those spellings.
- The compiler-known `workflow` namespace must coexist with any future stdlib/module backing without changing the accepted qualified surface or silently exposing unqualified operations.


### 5.2.1 Contract argument surface and opacity

`Requirement` and `OpenPostcondition` are compiler-known opaque contract argument classes, not ordinary user-constructible public data types in the first slice.

`Requirement` and `OpenPostcondition` are semantic classifier products / intrinsic parameter classes, not Ash-denotable public types. They must not be registered as ordinary source-level `TypeEnv` types, constructors, variables, importable names, parameter types, record fields, return types, pattern types, or constructor payload types. Rust may use internal enums/structs with those names, but user Ash code cannot store, pass, return, pattern-match, import, or export contract argument values.

`workflow::requires` and `workflow::ensures` are public workflow-algebra operations, but their arguments are accepted through special intrinsic elaboration of contract-expression syntax. They must not be implemented as ordinary `Type::Fn([Requirement], Workflow<Unit>)` or `Type::Fn([OpenPostcondition], Workflow<Unit>)` calls that require first-class value typing of their arguments. This allows users and migrated legacy declarations to express the same contracts as today without exposing arbitrary contract-value construction.

Permitted first-slice source forms are:

```ash
// Preferred first-class workflow statement forms inside do:Workflow.
requires: contract_expr;
ensures: post_expr;

// Direct intrinsic-call spelling for expression-level composition.
_ <- workflow::requires(contract_expr);
_ <- workflow::ensures(post_expr);
```

The statement forms intentionally keep the legacy `requires:` / `ensures:` colon spelling. The semicolon is required inside `do:Workflow` because these are statements. Legacy workflow declaration headers keep their existing no-semicolon header position.

First-slice grammar, in specification notation:

```text
WorkflowDoStmt ::=
    let name = Expr ;
  | name <- Expr ;
  | _ <- Expr ;
  | requires : ContractExpr ;
  | ensures  : PostExpr ;
  | return Expr

ContractExpr ::= Expr parsed in contract-expression context
PostExpr     ::= Expr parsed in postcondition context with delayed result binding
```

`ContractExpr` / `PostExpr` initially reuse the ordinary expression parser and are classified after parsing. This is deliberate compatibility with existing `requires: <expr>` / `ensures: <expr>` declarations. The parser should preserve the expression and source span; the workflow-form builder classifies it into `Requirement` / `OpenPostcondition` events.

The direct-call spelling is an intrinsic elaboration rule, not evidence that `Requirement` or `OpenPostcondition` are first-class values. A call expression whose callee resolves exactly to compiler-known `workflow::requires` or `workflow::ensures` in a Workflow construction context captures its argument expression as a contract argument before ordinary value typing of that parameter. Passing a variable of type `Requirement`, storing a `Requirement` in a record, returning one from a function, partially applying/taking `workflow::requires` as a value, or pattern-matching on one remains out of scope.

Allowed intrinsic-call contexts are Workflow construction contexts: RHS of `<-` / `_ <-` inside `do:Workflow`, Workflow comprehension qualifier RHS after SPEC-055 normalization, compiler-known workflow algebra composition, checked initialization/composition of named/local/imported `Workflow` values, and internal legacy declaration translation. Calls outside a Workflow construction context reject with an opaque intrinsic-parameter diagnostic rather than producing source-level contract values.

### 5.2.2 Conservative contract name resolution

Contract expressions are resolved conservatively to preserve legacy behavior. The first-slice resolver must not introduce new ambient authority or new dot-path contract namespaces.

Resolution order for `ContractExpr` is:

1. Preserve the parsed ordinary `Expr` and its lexical scope. Ordinary names in arithmetic/boolean predicates resolve exactly as they do in existing workflow/function contracts.
2. In contract-expression context only, recognize legacy contract helper calls such as:

   ```ash
   role(admin)
   any_role([admin, manager])
   ```

   `role(name)` classifies to `Requirement::HasRole(name)`. `any_role([a, b, ...])` classifies to a single OR-role requirement such as `Requirement::AnyRole(Vec<RoleRef>)` or `Requirement::RolePolicy(RolePolicy::AnyOf(...))`; it must not lower to multiple independent `HasRole` requirements, because that would implement AND rather than the intended legacy-compatible OR semantics. `any_role([])` rejects during classification. Bare identifiers inside the role list are symbolic role names in this context, matching the examples already present in the corpus.
3. Preserve the existing legacy/core contract vocabulary underneath the new form. The first-slice classifier must be able to produce the same semantic contract cases as the current implementation, including `Requirement::HasRole`, `Requirement::HasCapability`, arithmetic/precondition requirements, and postcondition predicates equivalent to `PostPredicate::Eq`, `PostPredicate::ResultSatisfies`, and `PostPredicate::StateAssertion` where the live legacy path can already express them.
4. Existing legacy/core contract variants such as capability requirements remain representable internally. If a current parser/import/API path already produces them, the workflow-form builder must preserve them. New user-facing capability/resource/policy contract syntax beyond legacy-compatible expressions is not invented here, but legacy headers that already express capability/resource/binding semantics must translate into equivalent workflow events and obligations.
5. The distinguished name `result` is special only in `PostExpr` target resolution. Before the suffix workflow result type is known, it is an open binder, not an ordinary unresolved variable. When `Bind(Ensures(Q), _, rest : Workflow<A>)` is checked, `result : A` is inserted only for checking `Q` against the successful result boundary of `rest`.

Normative classifier mapping:

| Source form / context | Classifier result |
|-----------------------|-------------------|
| legacy `plays role(R)` | `Requirement::HasRole(R)` event |
| `requires: role(R)` / `workflow::requires(role(R))` | `Requirement::HasRole(R)` |
| `requires: any_role([R1, R2, ...])` / intrinsic call equivalent | single implemented OR-role requirement carrier |
| bare identifiers inside role helpers | symbolic `RoleRef`, not ordinary lexical lookup |
| legacy capability header | capability requirement/header event preserving current capability constraints and span |
| legacy `owns` / `uses` header | resource/capability-binding header event plus corresponding authority/provenance obligations |
| arithmetic/boolean precondition expression | current compatible `Arithmetic` / `Precondition(CheckExpr)` carrier |
| existing internal `HasCapability` path | `Requirement::HasCapability { ... }` preserved where already produced |
| `ensures: result == expr` | open postcondition equivalent to supported `PostPredicate::Eq` |
| `ensures: result > n` / related arithmetic comparisons | open postcondition equivalent to `PostPredicate::ResultSatisfies(...)` where supported |
| legacy state/assertion postcondition | open postcondition equivalent to `PostPredicate::StateAssertion(...)` where supported |
| unclassified contract expression | hard contract-classification diagnostic; never ordinary value fallback |

Examples accepted by compatibility grammar:

```ash
requires: role(admin);
requires: any_role([admin, maintainer]);
requires: input_size > 0;
ensures: result > 0;
ensures: result.valid;      // only if ordinary field access exists at that implementation point
```

If an expression parses but cannot be classified or typechecked as a contract expression, checking must fail with a contract-expression diagnostic; it must not fall back to ordinary value construction of a `Requirement`.

### 5.2.3 Legacy workflow declaration deprecation and translation

The current workflow declaration form remains accepted for compatibility but is deprecated once this spec is implemented. Using it must produce a warning, not an error, in the first migration window.

Legacy source form:

```ash
workflow name(params) -> A
  plays role(R)
  capabilities: [...]
  owns ...
  uses ...
  requires: R1
  ensures: Q1
{
  legacy_workflow_body
}
```

is translated before type/constraint checking to the same `WorkflowForm` implementation path as first-class workflow expressions:

```text
Scope(name,
  Bind(Requires(role/header-derived requirements), _,
  Bind(Requires(R1), _,
  Bind(Ensures(Q1), _,
    FromProc(legacy_body_as_proc_summary)))))
```

The exact number and order of leading `Requires` nodes follows source order for `plays role`, capability/resource/admission headers, and explicit `requires:` clauses. Legacy `ensures:` clauses become leading `Ensures` nodes whose targets are resolved against the successful result boundary of the translated body.

To make this implementable, the parser/surface layer must preserve a source-ordered compatibility carrier:

```text
WorkflowHeaderEvent = {
  ordinal : SourceOrdinal,
  span    : SourceSpan,
  origin  : SourceOrigin,
  kind    : WorkflowHeaderEventKind,
}

WorkflowHeaderEventKind ::=
    PlaysRole(RoleRef)
  | CapabilityHeader(...)
  | OwnsHeader(...)
  | UsesHeader(...)
  | RequiresRaw(Expr)
  | EnsuresRaw(Expr)
```

Existing aggregate fields such as `plays_roles`, `capabilities`, `owned_resources`, `used_bindings`, and `contract.requires` / `contract.ensures` may remain as compatibility views, but they are not authoritative for SPEC-056 lowering because they cannot reconstruct interleaving order. WorkflowForm legacy translation iterates `WorkflowHeaderEvent`s in source order and converts each event into the corresponding leading WorkflowForm node/event.

Important compatibility rule:

- First-class `do:Workflow` statement forms and deprecated workflow declarations are different source surfaces.
- After parsing and deprecation-warning emission, both lower to `WorkflowForm` and use the identical workflow-form projection, obligation, coverage, lowering, and runtime boundary implementation.
- New implementation work must not add a second semantic path for legacy declarations.

Initial compatibility may translate the legacy syntax-heavy body through the existing workflow-body-to-Proc adapter and wrap it as `FromProc(legacy_body_as_proc_summary)`. Follow-on work may progressively desugar individual legacy workflow statements into ordinary workflow algebra operations, but that must preserve the same `WorkflowForm` boundary.

The adapter contract is explicit:

```text
legacy_body_as_proc_summary(
  legacy_body,
  params,
  source_origin,
  checking_env,
) -> Result<LegacyBodyProcSummary, Diagnostic>

LegacyBodyProcSummary = {
  proc_projection             : ProcArtifact,
  proc_contract_summary       : ProcContractSummary<A>,
  failure_summary             : FailureSummary,
  resource_authority_summary  : AuthorityResourceSummary,
  provenance_summary          : ProvenanceSummary,
  origin                      : SourceOrigin,
}
```

The adapter is a compatibility boundary only. It must preserve lower Proc summaries and emit lower-coverage obligations, must not create an independent legacy runtime/typechecking path, and must conservatively reject legacy bodies that cannot produce sufficient static summaries in Phase 108. Header events and the body summary are reconciled by the same WorkflowForm coverage/obligation machinery used by first-class `do:Workflow`.

Required warning family: `DeprecatedLegacyWorkflowDeclaration`, emitted with the workflow declaration span and a fix hint to rewrite as a named `Workflow<A>` value or `do:Workflow` expression once the new surface is available. The warning must be diagnostics-plumbed through parser/typechecker/engine/CLI or an equivalent check path, must be emitted exactly once per deprecated declaration, and must not cause `ash check` to fail when no errors exist.

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
  | Bind(form : WorkflowForm<A>, binder, body : binder-scoped WorkflowForm<B>)
  | FromProc(proc : Proc<A>)
  | FromAct(act : Act<A>)
  | Requires(requirement : Requirement)
  | Ensures(postcondition : OpenPostcondition)
  | Scope(scope : WorkflowScope, form : WorkflowForm<A>)
```

The `Bind` continuation/body is binder-scoped and may be value-dependent: the suffix form is checked under a scope extended by the binder produced by the left-hand workflow. Specification notation writes it as a `WorkflowForm<B>` body, but implementations may represent it as a typed continuation artifact, delayed elaboration node, or equivalent non-runtime closure. It must not be treated as an already-closed form whose contract/check projections have been detached from binder scope.

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

### 6.2.5 Crate ownership and dependency boundaries

Phase 108 carrier ownership is normative and must avoid dependency cycles:

- `ash-core` owns shared semantic/runtime carrier definitions used across crates, including `WorkflowForm`, `WorkflowNodeId`, `ProjectionEvent`, `WorkflowContract`, `AdmissionEnvelope`, `ContractPlan`, `CoverageEvidence`, `CoverageError`, `WorkflowContractSummary`, lower Proc/Act contract summary carriers needed by workflow coverage, and public workflow summary/source-anchor types. The exact Rust module names may differ, but these shared carriers must be available without depending on parser or typechecker internals.
- `ash-parser` owns raw surface carriers only: source-fidelity `DoStmt` forms for raw `requires:` / `ensures:` statements and `WorkflowHeaderEvent` raw clauses/spans/origin/order. It must not own semantic `WorkflowForm`, coverage, or executable workflow metadata.
- `ash-typeck` builds `WorkflowTypedArtifact` using `ash-core` carriers. Typechecker-private elaboration helpers may exist, but `ash-engine` and `ash-interp` must not require typeck-private structs to import summaries, lower executable metadata, or run workflows.
- `ash-engine` serializes, imports, and exports public workflow summaries using `ash-core` summary types. It may coordinate typechecking and module loading, but it must not serialize parser ASTs or typeck-private artifacts as the public contract-summary format.
- `ash-interp` consumes executable projection/runtime metadata derived from `ash-core` workflow carriers. It must not depend on parser ASTs, raw `WorkflowHeaderEvent`s, or typeck-private `WorkflowTypedArtifact` internals at runtime.

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

This rule prevents `from_proc` from becoming an authority-smuggling operation while preserving useful staged composition such as `requires: role(admin); x <- workflow::from_proc(admin_proc);`.

### 7.5 `workflow::from_act`

```text
workflow::from_act : Act<A> -> Workflow<A>
```

`from_act` is equivalent to explicit Act-to-Proc embedding followed by workflow lift, or to a distinct `FromAct` form whose projections are equivalent:

```text
workflow::from_act(a) = workflow::from_proc(proc::from_act(a))
```

In either representation, `from_act(a)` preserves the lower Act/Proc summary and emits delayed lower-summary coverage obligations. It must not become an authority-smuggling operation or a metadata-free `proc::unit` wrapper.

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

A requirement may refine the continuation checking environment. For example, `requires: role(admin);` may allow a following `workflow::from_proc(admin_proc)` to be checked under a provisional role/admission assumption.

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

Workflow blocks additionally admit contract-injection statements as syntax for opaque contract arguments to ordinary `Workflow<Unit>` forms:

```text
requires: requirement_expr;       // lowers as _ <- workflow::requires(requirement_expr);
ensures: postcondition_expr;      // lowers as _ <- workflow::ensures(open(postcondition_expr));
```

The colon is required for compatibility with the legacy workflow declaration contract grammar. The semicolon is required because these are `do:Workflow` statements.

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

Typed elaboration must reuse SPEC-054's nested bind/return checking path, but the target artifact for `Workflow` is a preserved `WorkflowForm` before any evidence-preserving optimization. The live Act/Proc path may continue to produce CoreExpr-only dictionary calls, but Workflow elaboration must carry an additional artifact such as:

```text
WorkflowTypedArtifact = {
  form          : WorkflowForm<A>,
  events        : Vec<ProjectionEvent>,
  contract_plan : ContractPlan<A>,
  obligations   : Vec<WorkflowObligation>,
  origins       : Vec<SourceOrigin>,
}
```

A CoreExpr or Proc projection may be derived from this artifact for compatibility with existing lowering/runtime paths. It is not the source of truth for Workflow semantics.

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

requires: R; rest
  => Bind(Requires(classify_contract_expr(R)), _, elaborateWorkflow(rest))

ensures: Q; rest
  => Bind(Ensures(open_postcondition(Q)), _, elaborateWorkflow(rest))
```

Conceptual source:

```ash
do:Workflow {
    requires: role(analyst);
    ensures: result.valid;
    x <- a;
    y <- b(x);
    return f(x, y)
}
```

elaborates to the same workflow-form skeleton as:

```text
Bind(Requires(role(analyst)), _,
  Bind(Ensures(open(result.valid)), _,
    Bind(a, x,
      Bind(b(x), y,
        Unit(f(x, y))))))
```

and has the same Proc-projection shape as the dictionary expression:

```text
workflow::bind(workflow::requires(role(analyst)), λ_.
  workflow::bind(workflow::ensures(open(result.valid)), λ_.
    workflow::bind(a, λx.
      workflow::bind(b(x), λy.
        workflow::unit(f(x, y))))))
```

Modulo spans/origin metadata, ordinary bind/return checking is the same path used for Act/Proc targets. The difference is that workflow contract-injection forms contribute non-Proc projected events that remain present for zipper interpretation and later verification/lowering.

### 8.4 Workflow-aware ordinary expression elaboration

Workflow construction contexts are not limited to `do` statements. In any context where the compiler is constructing a `WorkflowForm` -- `do:Workflow`, `[...]: Workflow` after SPEC-055 normalization, compiler-known workflow algebra composition, legacy declaration translation, and checked initialization of named/local/imported `Workflow` values -- ordinary calls whose callee resolves exactly to a compiler-known workflow algebra builtin must be handled by a `WorkflowForm`-aware expression elaborator.

Normative elaboration rules:

```text
workflow::unit(e)
  => Unit(e)

workflow::bind(w, f)
  => Bind(form(w), binder, form(f binder))
     or reject if the continuation cannot be checked under binder scope
     to yield a WorkflowForm

workflow::then(w1, w2)
  => Bind(form(w1), _, form(w2))

workflow::from_proc(p)
  => FromProc(p) plus lower Proc summary obligations

workflow::from_act(a)
  => FromAct(a) plus lower Act summary obligations
     or an equivalent FromProc(proc::from_act(a)) form plus the same obligations

workflow::requires(R)
  => Requires(classify_contract_expr(R))

workflow::ensures(Q)
  => Ensures(open_postcondition(Q))
```

This elaborator owns all seven first-slice compiler-known qualified builtins: `workflow::unit`, `workflow::bind`, `workflow::then`, `workflow::from_proc`, `workflow::from_act`, `workflow::requires`, and `workflow::ensures`. It must preserve or reconstruct `WorkflowForm` artifacts for ordinary compiler-known calls rather than lowering them only to CoreExpr dictionary calls.

Named, local, and imported values of type `Workflow<A>` are usable in workflow algebra composition only if they carry or reference one of:

- a live `WorkflowTypedArtifact` in the current compilation unit; or
- a public `WorkflowContractSummary<A>` / workflow summary imported from module metadata with sufficient contract, admission, failure/report/provenance, and public alignment-anchor information.

A `Workflow<A>` value whose form/summary is absent is opaque for first-class composition. Binding it, sequencing it, lifting it into another `WorkflowForm`, or exporting a composed summary from it must reject with an opaque-summary diagnostic rather than silently treating the value as a metadata-free Proc wrapper.

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

Existing workflow declarations remain valid but become deprecated compatibility syntax when this spec is implemented. The compiler must emit a deprecation warning for the legacy declaration surface and then translate it into the same `WorkflowForm` implementation path used by first-class workflow expressions.

This spec adds a semantic interpretation:

```text
workflow declaration = named/scoped/exported Workflow<A> expression plus host/runtime entrypoint metadata
```

A new-style declaration body is target-typed as a workflow block and lowers to a `WorkflowForm<A>` / `Workflow<A>` value. Header-like clauses such as `requires:` and `ensures:` are contract-injection workflow forms in that block, not a separate semantic island.

Conceptual declaration rule for new-style bodies:

```text
Γ, params ⊢ block ⇝ form : WorkflowForm<A>
proc(form)     = p : Proc<A>
contract(form) = C_expr : WorkflowContract<A>
Governs(C_expr, p, E_expr)
C_total = Scope(name, C_expr)
```

The declaration exports a workflow summary sufficient for imported callers to type-check uses of the named workflow as a `Workflow<A>` value.

Legacy declaration compatibility rule:

```text
legacy workflow header/body
  ⇝ warning DeprecatedLegacyWorkflowDeclaration
  ⇝ Scope(name, leading_contract_events ⋄ translated_body_form)
```

`leading_contract_events` is constructed in source order from legacy `plays role(...)`, capability/resource/admission headers, `requires: <expr>`, and `ensures: <expr>` clauses. The explicit `requires:` and `ensures:` expressions are parsed/classified by the same contract-expression rules used for `do:Workflow` statement forms and `workflow::requires` / `workflow::ensures` intrinsic calls.

The first compatibility implementation may translate the legacy syntax-heavy body through an existing legacy workflow-body-to-Proc adapter and wrap it as `FromProc(legacy_body_as_proc_summary)`. That adapter is a compatibility boundary only; it must still feed the same `WorkflowForm` projection, obligation, coverage, lowering, and runtime boundary implementation. New code must not add a parallel semantic path for deprecated declarations.

Future phases may progressively desugar individual legacy workflow body statements into ordinary workflow algebra operations. Such desugaring is a refactoring of the translation boundary, not a semantic change.

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

Imported workflow values without a contract summary must be rejected for first-class composition or treated as opaque values that cannot be safely bound under `Workflow` until a summary exists. If a future stdlib module backs compiler-known `workflow::...` builtins with ordinary exports, module metadata must preserve those qualified exports and their intrinsic markers/summaries so importing `workflow` does not weaken the compiler-known namespace rules or expose unqualified operations implicitly.

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
13. Deprecated legacy workflow declaration: accepted compatibility syntax translated to `WorkflowForm`, with warning `DeprecatedLegacyWorkflowDeclaration` and a rewrite hint.
14. Public contract-value misuse: attempts to store/pass/return `Requirement` or `OpenPostcondition` as ordinary values must state that these are opaque compiler-known contract arguments in the first slice.

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
3. Parser/surface AST accepts `requires: expr;` and `ensures: expr;` statement forms in `do:Workflow` without changing `do:Act` / `do:Proc` semantics.
4. Source-ordered `WorkflowHeaderEvent`s preserve mixed legacy header interleaving even when aggregate compatibility fields remain populated.
5. Contract-expression classification can produce all current legacy/core contract cases expressible today, including role, `any_role` OR, capability, resource/header, arithmetic/precondition, and postcondition predicate cases.
6. `Requirement` and `OpenPostcondition` are non-denotable in Ash source; attempts to store/pass/return/partially apply/pattern-match them reject.
7. Direct intrinsic calls `workflow::requires(expr)` and `workflow::ensures(expr)` elaborate to the same contract events as the statement forms.
8. TypeEnv registers `Workflow` as a builtin unary constructor.
9. `Workflow` do target resolves with a workflow dictionary.
10. `do:Workflow { return x }` synthesizes `Workflow<A>`.
11. `do:Workflow` binds only `Workflow<A>` RHS values.
12. `do:Workflow` rejects `Proc<A>` and `Act<A>` RHS values without explicit lifts.
13. `workflow::from_proc` and `workflow::from_act` allow explicit lifts where lower summaries exist and emit delayed coverage obligations.
14. `workflow::unit`, `workflow::bind`, `workflow::then`, `workflow::from_proc`, and `workflow::from_act` elaborate through the WorkflowForm-aware expression elaborator and produce/preserve `Unit`, binder-scoped `Bind`, `FromProc`, and `FromAct` (or equivalent `FromProc(proc::from_act(...))`) artifacts with lower-summary obligations.
15. `requires` and `ensures` in a workflow block lower to preserved `workflow::requires` / `workflow::ensures` workflow forms.
16. Qualified `workflow::...` names resolve in the same compiler-known builtin namespace style as qualified `proc::...` names.
17. Unqualified `unit`, `bind`, `then`, `from_proc`, `from_act`, `requires`, and `ensures` are not implicitly imported by selecting `do:Workflow`.
18. Named/local/imported `Workflow` values without a live `WorkflowTypedArtifact` or public workflow summary reject as opaque when bound or sequenced.
19. `requires` may refine continuation checking context but produces coverage obligations that must be proven later.
20. `ensures` targets the successful result boundary of the suffix workflow and typechecks under `result : A`.
21. Deprecated legacy workflow declarations emit a warning and translate to the same `WorkflowForm` path as new first-class workflow expressions.
22. `legacy_body_as_proc_summary` preserves lower Proc/failure/authority/provenance summaries or rejects conservatively when it cannot.
23. Typed elaboration of `do:Workflow` produces a `WorkflowTypedArtifact` with nested workflow-form `Bind` / `Unit` shape without erasing neutral-Proc contract-injection nodes.
24. `[result | x <- wf]: Workflow` elaborates equivalently to `do:Workflow` and the same `WorkflowForm` / projection alignment.
25. Coverage/obligation failures produce component-specific diagnostics.
26. Imported workflow summaries are preserved across module boundaries or rejected if absent.
27. Existing `do:Act`, `do:Proc`, Act blocks, and Act/Proc comprehensions remain unchanged.

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

- Full coverage solving/proof search and dynamic residualization beyond the conservative Phase 108 checker.
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
- Clarified Phase 108 review ownership for Workflow algebra expression elaboration, crate carrier boundaries, and namespace exports: all compiler-known ordinary calls to `workflow::unit` / `bind` / `then` / `from_proc` / `from_act` / `requires` / `ensures` in Workflow construction contexts preserve `WorkflowForm` artifacts; `Bind` continuations are binder-scoped; shared carriers live in `ash-core`; parser/typeck/engine/interp dependency boundaries are explicit; first-slice `workflow::...` names are qualified compiler-known builtins rather than implicit unqualified stdlib imports; and the summary now states `Workflow<A>` as a synchronized product via `WorkflowForm` rather than an ambiguous `+` expression.
- Added conservative, legacy-compatible `requires:` / `ensures:` contract-expression grammar; clarified `workflow::requires` / `workflow::ensures` as compiler-known intrinsic operations over non-denotable contract argument classes; specified source-ordered legacy `WorkflowHeaderEvent`s, concrete classifier mapping including `any_role` OR semantics, WorkflowForm-preserving typed-do artifacts, executable lowering/runtime projection ownership, explicit legacy-body adapter behavior, deprecation warning plumbing, and same-`WorkflowForm` translation for deprecated workflow declarations.

### 2026-04-29

- Initial draft promoted from DESIGN-033 into a normative first-class `Workflow<A>` carrier, Monad target, typed-do, and comprehension integration spec.
