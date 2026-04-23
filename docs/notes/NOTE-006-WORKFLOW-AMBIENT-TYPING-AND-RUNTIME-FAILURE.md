# NOTE-006: Workflow Ambient Typing and Runtime Failure Boundary

**Date:** 2026-04-23
**Status:** Draft
**Priority:** High — clarifies workflow typing direction without prematurely fixing orchestration policy
**Related:** NOTE-001, NOTE-005, SPEC-001, SPEC-003, SPEC-004, SPEC-022, SPEC-025, SPEC-047

## 1. Problem

Ash workflows now sit between two partially aligned stories:

1. `Act<A>` is emerging as the expression-layer effectful computation type with an opaque runtime-managed `ActEnv`.
2. Workflows already carry structured header declarations that influence execution in ways that are not ordinary value parameters.
3. Workflow execution can fail at different phases (admission, runtime step, completion), but the language should not commit too early to supervisor trees or orchestration-specific recovery semantics.

The current shorthand

```text
f : Params -> Act<A>
```

is too weak to explain workflow headers such as:

- `plays role(...)`
- `capabilities: [...]`
- `requires: ...`
- `ensures: ...`

At the same time, replacing it with a single undifferentiated

```text
f : AmbientParams -> Params -> Act<A>
```

is still too coarse, because the ambient declarations have distinct semantic intents.

This note records a semi-stable design direction: workflows should continue to produce `Act<A>` instances, but workflow typing should track a structured projection of the hidden runtime environment rather than collapsing all ambient information into ordinary parameters.

## 2. Design Direction

### 2.1 Workflow instance type

A workflow instance remains an effectful computation:

```text
workflow-instance : Act<A>
```

This preserves alignment with the broader Act direction and avoids inventing a separate workflow-result wrapper type.

### 2.2 Workflow definition shape

A workflow definition is best understood as introducing a constructor of workflow instances:

```text
f : Params -> Act<A>
```

but only under a richer workflow typing judgment that incorporates ambient context shaping and contract obligations.

The key point is:

- `Params` are explicit call-time value/type inputs.
- workflow headers shape the admissible execution environment and the contract under which the resulting `Act<A>` may start and successfully complete.

### 2.3 `ActEnv` is opaque; workflow typing tracks its projections

`Act<A>` is conceptually runtime-managed state-threading over an opaque `ActEnv`, but workflow typing should not expose or equate `ActEnv` with a surface parameter record.

Instead, the type system should track structured, language-level projections of the runtime environment.

Informally:

```text
Act<A> ≈ ActEnv -> (ActEnv, A)    -- semantic intuition only
```

but workflow typing reasons over tracked projections such as:

```text
Φ = ⟨Avail, Roles, Facts⟩
Ω = obligation state
```

where:

- `Avail` = currently available/refined capabilities
- `Roles` = active role/feature bundles
- `Facts` = logical facts known at the current workflow point
- `Ω` = pending/discharged obligations or equivalent workflow-local linear markers

This avoids exposing the actual internal representation of `ActEnv` while still letting workflow headers affect typing in principled ways.

## 3. Header Declarations Have Distinct Intents

The current workflow header already separates several classes of declarations. They should not be modeled as one generic ambient parameter bucket.

### 3.1 `capabilities: [...]` — admitted capability surface

Intent:
- explicitly declare the capability surface admitted to the workflow body

Typing effect:
- narrows/refines `Avail`
- affects which operational forms are admissible/typable inside the body

Visibility rule:
- capability definitions in lexical scope are available for reference while elaborating the workflow definition and its parameter/header surface
- they are not automatically body-visible merely because they exist lexically
- the workflow body sees only the capability subset explicitly admitted by the workflow definition

Current minimal stance:
- avoid heavy capability algebra or eager derivation beyond what is needed for workflow-definition checking and admission
- future list/composition forms are acceptable, but they should still feed the same explicit admission boundary

Important distinction:
- this is not just a precondition predicate
- it changes the admitted body capability context directly

### 3.2 `plays role(...)` — admitted role context

Intent:
- explicitly declare the role context under which the workflow body operates

Typing effect:
- extends/refines `Roles` for the workflow body
- may later support richer list/composition forms, but no such elaboration is required yet beyond workflow-definition checking and admission

Visibility rule:
- role definitions in lexical scope are available for reference while elaborating the workflow definition and its parameter/header surface
- they are not automatically body-visible merely because they exist lexically
- the workflow body sees only the role context explicitly admitted by the workflow definition

Current minimal stance:
- roles should not yet be expanded into large derived structures unless a specific typing/runtime rule requires it
- future role lists/compositions should remain acceptable as long as they still define the explicitly admitted role context for the workflow

Important distinction:
- roles are not treated as ordinary ambient leakage from declaration scope
- the header remains the admission boundary for body-visible role context

### 3.3 `requires: ...` — entry precondition

Intent:
- express facts that must hold when the workflow instance is admitted for execution

Typing effect:
- becomes an entry proof/admissibility obligation over the initial tracked context
- may inspect `Avail`, `Roles`, `Facts`, and explicit parameters

Important distinction:
- `requires` constrains admissible starts
- it does not by itself reshape the body context the way capability restriction or role introduction do

### 3.4 `ensures: ...` — exit postcondition

Intent:
- express facts that must hold when a workflow claims successful completion

Typing effect:
- becomes a completion validity obligation over the final tracked context plus the returned value

Important distinction:
- `ensures` is neither an entry condition nor an ambient availability declaration
- it constrains what counts as a valid terminal completion

## 4. Proposed Typing Shape

This note does not freeze final notation, but the intended direction is a structured workflow typing judgment rather than a plain arrow type alone.

A useful first-pass judgment form is:

```text
Σ ; Γ ; Φ ; Ω ⊢ w : A ▷ Φ' ; Ω'
```

Read:
- `Σ` = global declarations/environment
- `Γ` = ordinary value/type environment
- `Φ` = tracked ambient workflow context (`Avail`, `Roles`, `Facts`)
- `Ω` = obligation state
- `w : A` = workflow body produces a value of type `A`
- `Φ' ; Ω'` = updated context/obligation state after the body

Then workflow-definition validity is checked by combining:

1. header elaboration
2. entry admissibility
3. body typing
4. completion validity

Informally:

```text
header ⇓ ⟨CapRestrict, RoleIntro, Pre, Post⟩
Φ0 = apply_roles(Φbase, RoleIntro)
Φ1 = apply_cap_restriction(Φ0, CapRestrict)
Σ ⊢ Φ1, Γparams ⊨ Pre
Σ ; Γparams ; Φ1 ; ∅ ⊢ body : A ▷ Φ2 ; ∅
Σ ⊢ Φ2, result ⊨ Post
────────────────────────────────────────
Σ ⊢ workflow f(params) -> A valid
```

This is deliberately abstract, but it captures the essential asymmetry:

- lexical role/capability definitions are available for workflow-definition elaboration and parameter/header reference
- the workflow body sees only the explicitly admitted role/capability context selected by the workflow definition
- capability declarations modify the admitted body capability context
- role declarations define the admitted body role context
- `requires` checks entry admissibility
- `ensures` checks valid completion

## 5. Failure Boundary

Workflow typing alone is not enough. We also need a runtime failure model that does not prematurely bake in supervisors, restart trees, or orchestration topology.

### 5.1 Static failure

Static failures remain compile/check-time rejection:

- malformed workflow header/body
- type mismatch
- unprovable static obligation
- impossible ambient requirement shape
- undischarged obligations in static control-flow checking (where enforced)

These are not runtime failures. No workflow instance is produced.

### 5.2 Runtime failure model

For dynamic execution, prefer the single familiar outcome shape:

```text
Result<A, WorkflowFailure>
```

Do not introduce synonym-like wrappers such as `WorkflowOutcome<A>` unless implementation pressure later proves them necessary.

The runtime owns `WorkflowFailure`. The type system should not be coupled to any specific orchestration regime.

### 5.3 Canonical `WorkflowFailure` partition

The current design direction is:

```text
WorkflowFailure =
  | AdmissionFailure(...)
  | RuntimeFailure(...)
  | CompletionFailure(...)
```

where:

- `AdmissionFailure` = the workflow was well-typed, but this invocation could not start because the entry contract was not satisfied
- `RuntimeFailure` = failure during execution before a candidate terminal result was reached
- `CompletionFailure` = a candidate terminal state/result was reached, but obligations or postconditions invalidated successful completion

Examples:

- `AdmissionFailure`: missing role, missing capability refinement, false `requires` predicate at instantiation
- `RuntimeFailure`: provider crash, timeout, explicit abort, policy denial during an effectful step
- `CompletionFailure`: remaining undischarged obligations, false `ensures`, final compliance/governance invalidation

### 5.4 Why completion failure is distinct

Completion failure must remain distinct from ordinary runtime-step failure.

It means:
- the workflow body reached a terminal candidate result
- but the contract says this does not count as a valid success

This distinction is especially important for asynchronous execution, where “panic at end” is too imprecise. The runtime should report a failed completion, not force the typing story to encode supervision assumptions.

## 6. Runtime Policy Boundary

The runtime should decide what to do with `WorkflowFailure`:

- return it directly to a synchronous caller
- resolve an async handle with it
- retry
- restart
- escalate
- compensate
- audit/log and stop

This note intentionally does **not** bind workflow typing to any of those policy choices.

Design rule:

- workflow typing tracks admissibility and completion contracts
- runtime execution returns `Result<A, WorkflowFailure>`
- orchestration/recovery policy is a later runtime concern

This keeps the language substrate neutral while leaving room for future supervisor-like mechanisms if they prove useful.

## 7. Process Substrate Direction

A stronger process-oriented reading has emerged from this design work.

Current working position:

- a workflow is operationally an isolated process running in its own `ActEnv`
- workflows share nothing directly with other workflows
- workflow-to-workflow interaction should occur only through explicit parameters, channels, and terminal result/failure observation
- effectful functions reduce pressure to model nested "sub-workflows" that close over a parent workflow's live environment

This suggests a future stratification:

1. `Act<A>` — sequential effectful computation
2. `Proc<A>` — runnable isolated process computation
3. `workflow` — richer syntax that elaborates into `Proc` construction and enrichment

### 7.1 Why a `Proc` layer is attractive

A dedicated `Proc` layer may be worth introducing because it provides:

- a primitive runnable isolation boundary simpler than full workflows
- a focused substrate for mailbox/channel semantics, spawnability, and scheduling
- a better testing target for runtime/process behavior without workflow-level role/capability/contract complexity
- a cleaner place to define workflow in terms of process machinery rather than making workflow the lowest-level runnable abstraction

The current preference is therefore positive but still exploratory:

- `Proc` looks worth introducing
- temporary overlap during migration is acceptable if workflow ultimately reuses process machinery

### 7.2 Minimal process properties

A minimal process form should likely have only these core properties:

- explicit parameters
- intrinsic mailbox
- channel-oriented `send` / `receive`
- fresh private `ActEnv`
- no direct sharing of another process/workflow's live local environment
- no external mutation of its environment once started

Strong invariant:

- once a process or workflow starts, its environment may evolve only through its own execution semantics; it is never mutated directly from the outside world

### 7.3 `Act` vs `Proc`

Current settled direction:

- `Act` and `Proc` should be treated as different monads
- `Act` remains the monad of sequential effectful computation, with `bind` as its characteristic composition operator
- `Proc` is a different monad/algebra for runnable isolated processes
- the distinction should be strengthened rather than weakened: `Proc` is not merely "an `Act` with mailbox" or a thin wrapper to be identified with `Act`

At the same time, one especially valuable relationship remains in view:

- `Proc<Act<A>>` is a particularly important and useful way to think about a process carrying an effectful sequential payload
- this does **not** require introducing a higher-kinded public form such as `Proc<F, A>`
- the public/process-level type should stay simple as `Proc<A>` unless real pressure later proves otherwise

Current preference:

- keep `Proc<A>` as the process type
- allow the implementation and semantics to exploit especially important cases such as `Proc<Act<A>>`
- but do not define `Proc` away as reducible to `Act`

A process may therefore be understood as a runnable/process-structured computation whose payload is executed/applied by `run`, with `Act` as the richest and most valuable current case, while still preserving `Proc` as its own algebra and abstraction boundary.

### 7.4 Parallel composition and process algebra

For `Proc`, the characteristic composition law appears to be parallel/process composition rather than sequential dependency.

Working intuition:

```text
Act . Act . Act      -- sequential effect composition via bind
Proc || Proc || Proc -- parallel process composition
```

This suggests:

- `bind` on `Proc` still matters for dependent sequential process composition
- but `||` is likely more central than `bind` for the concurrency story
- `||` feels closer to applicative/monoidal composition than to monadic sequencing

The exact environment-distribution law for `||` remains open:

- branches may share some common process context
- or the runtime may partition/duplicate relevant environment projections
- but process isolation and explicit communication remain the governing principles

### 7.5 Library-facing combinators and workflow sugar

It is worth considering a `proc` library implemented independently of workflow syntax, but in a way that stays compatible with the longer-term workflow goals.
That would allow focused runtime/process testing and iteration before fully migrating workflow machinery onto the same substrate.

Because `ActEnv` remains opaque, process enrichment should not be modeled as direct environment mutation.
Instead, it should be expressed through process-level constructors/combinators.

A plausible future `proc` library may expose ordinary unsuffixed names such as:

- `unit`
- `bind`
- `then`
- `send`
- `receive`
- `scatter`
- `gather`
- possibly `par`, `spawn`, and related process combinators

Likewise, the `act` library may expose its own unsuffixed `unit` / `bind` / `then` within its own namespace.
The distinction is by algebra/module, not by globally bloated names.

For process/workflow enrichment, neutral `with_*` names currently look better than `add_*`, because they do not prematurely force an additive-only interpretation. Examples:

- `with_capabilities`
- `with_roles`
- `with_requires`
- `with_ensures`

Current direction:

- workflow syntax may later become sugar over `Proc` construction plus such `proc`-level combinators
- this should preserve `ActEnv` opacity and keep role/capability/contract shaping at the process boundary rather than exposing raw environment algebra

## 8. Consequences for Future Spec Work

This note suggests the following spec amendments or follow-ons:

1. `SPEC-022` should distinguish more sharply between:
   - ambient context shaping (`capabilities`, `plays role`)
   - entry proof obligations (`requires`)
   - completion proof obligations (`ensures`, obligation discharge)

2. `SPEC-047` / Act-related work should remain explicit that:
   - workflow instances are `Act<A>`
   - `ActEnv` remains opaque/runtime-owned
   - workflow typing reasons over tracked projections, not raw `ActEnv`

3. `DESIGN-030` and `SPEC-048` now capture the first tight proc packet. Follow-on process work should evaluate whether:
   - `Proc<A>` becomes a first-class surface type or an internal elaboration target first
   - workflow is best specified as an enriched process form over shared runtime machinery
   - `Proc` should be centered semantically on applicative/monoidal composition in addition to monadic structure
   - an independently useful `proc` library can be landed before workflow migration while preserving compatibility with the longer-term workflow lowering target

4. Workflow failure reporting should be defined in runtime/spec terms as:

```text
run(workflow-instance) : Result<A, WorkflowFailure>
```

without prematurely fixing supervisor hierarchies or restart semantics.

5. A future note/spec should explore whether and how workflows may proactively handle or convert completion threats before terminal return, while preserving the hard completion boundary.

## 9. Open Questions

1. Which parts of `requires` / `ensures` are intended to be statically provable versus dynamically checked?
2. Should role introduction be modeled as pure context extension, or can it also synthesize obligations directly?
3. How much of `Φ` should be visible in user-level reasoning/documentation versus remaining purely internal to type checking?
4. What is the minimal informative initial shape of `WorkflowFailure` before any supervisor or recovery model exists?
5. How should workflow calls compose their caller/callee ambient contexts without leaking raw `ActEnv` representation?
6. For explicit capability parameters, should the parameter itself count as admitted body capability context automatically, or should header admission remain the single visibility boundary?
7. Should `Proc<A>` first appear as a surface-visible type/form, or initially as an internal elaboration target beneath process/workflow syntax?
8. What exact semantic law should govern `Proc` parallel composition (`||`) with respect to environment distribution, failure propagation, and branch isolation?
9. How should `run` be specified so that `Proc` remains clearly distinct from `Act` while still admitting especially valuable cases such as `Proc<Act<A>>`?

## 10. Current Working Position

Until further amendment, use the following as the semi-stable reference point:

- A workflow definition introduces a constructor of `Act<A>` instances.
- Workflow typing is richer than `Params -> Act<A>` alone and must track structured ambient context and obligation state.
- `capabilities`, `plays role`, `requires`, and `ensures` have distinct semantic roles and should appear differently in the typing rules.
- Dynamic workflow execution yields `Result<A, WorkflowFailure>`.
- `WorkflowFailure` belongs to the runtime contract, not to supervisor-specific typing or orchestration semantics.
