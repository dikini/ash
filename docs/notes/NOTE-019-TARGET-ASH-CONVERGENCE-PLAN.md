# NOTE-019: Target Ash Convergence Plan

**Date:** 2026-06-24
**Status:** Draft note — convergence map, not an implementation plan
**Purpose:** Summarize the path from current Ash to target Ash across the language,
runtime, memory, contract, effect, and library surfaces. This note is intentionally not a
`docs/plan/` implementation plan: it names convergence tracks, readiness gates, and open
design dependencies so later implementation plans can be scheduled without re-opening the
same conceptual questions.

Companion to NOTE-013 through NOTE-018, NOTE-020, and the target specs SPEC-095b through
SPEC-102.

## 0. Motivation

Ash now has enough target design material to need a convergence map.

The current design corpus says, in different places:

1. the language should have one ambient computation model;
2. computation rows describe requirements and computation facts, not grants;
3. handlers and providers discharge/interpret operations;
4. contracts, laws, evidence, and properties share discharge machinery but differ in
   lifecycle;
5. `Act`, `Proc`, and `Workflow` are profiles/carriers/library surfaces, not separate
   semantic foundations;
6. applications, supervision, behaviours, streams, and graphs are runtime/library
   organization, not magical workflow syntax;
7. memory safety needs an Ash-level story above "Rust handles it";
8. current old syntax is not a compatibility contract because remaining uses live in the
   standard library, documentation examples, and tests.

The convergence problem is therefore not primarily "how do we preserve the old language?"
It is:

```text
How do we make the target model the only semantic path, then migrate the project-owned
corpus and libraries onto it?
```

## 1. Convergence Fixed Point

The intended fixed point is:

```text
surface Ash
  -> Core Ash
  -> typed Core with rows/discharge/evidence
  -> CPS IR with raise/handle/provider/continuation machinery
  -> runtime kernel/app/process/handler execution
```

At that point:

- ordinary computation is `fn` plus computation-row-bearing callable types;
- `do` is sequencing sugar for the ambient computation model;
- operation-like behavior is represented by effect operations, rows, handlers, providers,
  and admission;
- contracts and evidence are row/discharge facts, not ad hoc side channels;
- `Act`, `Proc`, and `Workflow` are library names, row profiles, carriers, or app/runtime
  concepts;
- apps, supervisors, behaviours, process services, and reactive graphs are explicit runtime
  organization;
- memory ownership is visible at process/app/boundary crossings;
- old project-owned examples/tests/stdlib uses have been rewritten or marked historical.

The target should feel smaller without being less expressive:

```text
fewer primitive islands;
more explicit rows, boundaries, evidence, handlers, and runtime blueprints.
```

## 2. What Is Already Implemented or Substantially Grounded

This section is not a conformance claim. It records where the target story already has
usable substrate.

| Area | Current evidence | Convergence implication |
|---|---|---|
| Core Ash | SPEC-099, SPEC-100, Core type-checking work | Core can become the single checked direct-style semantic layer. |
| CPS IR | SPEC-098b, SPEC-099b, CPS interpreter/runtime work | Raise/handle/provider/continuation execution has a real target substrate. |
| Rows | SPEC-096b, SPEC-097b, NOTE-020, row normalization/checking work in Core | Computation rows can be the shared accounting layer for effects, modes, failures, contracts, and requirements, but surface integration remains incomplete. |
| Continuations | SPEC-102 implemented at Core/CPS level | Handler resume multiplicity is no longer only theoretical. |
| Lazy/memo modes | SPEC-101 implemented in Core/CPS slices | Evaluation modes can be integrated with rows and contracts rather than treated as syntax tricks. |
| Contract vocabulary | NOTE-014, SPEC-097b, SPEC-098b, SPEC-100 | Discharge modes exist conceptually and in Core metadata, but surface lowering/blame remain gaps. |
| Runtime kernel | SPEC-070 and NOTE-016 | Multi-app runtime organization has a conceptual home. |
| Process model | SPEC-049, NOTE-017 | Process boundaries can anchor memory and sendability rules. |

The missing work is not a new foundation. It is alignment: making the surface, standard
library, examples, diagnostics, and runtime organization all point at the same substrate.

## 3. What Is Resolved Enough To Treat As Direction

These are design constraints for future specs and implementation plans.

### 3.1 One ambient computation model

Target Ash has one ambient monad for sequencing. Computation rows index the facts attached
to a computation: effects, evaluation modes, failures, contracts, evidence, authority,
runtime requirements, and related obligations. Other monadic stories are implemented by
effects plus providers, while lazy/memo/eager behavior is represented as
computation-row mode facts. Provider nesting composes interpretations; row order does not.

#### 3.1.1 `do` notation versus plain function composition

Plain `fn` remains the default authoring and semantic form. In expression position, users
can still write ordinary nested function composition:

```ash
fn name(...) -> ... = f(g(h(x, y)))
```

This is the smallest surface for pure expression trees and for code where the data flow is
clear when read inside-out.

`do` is useful for the other common shape: direct, imperative-looking sequencing. It should
not introduce a second execution model. It is notation for the ambient monad's `bind`, with
`return` as the ambient unit:

```ash
fn name(...) -> ... = do {
  x <- f(x);
  return x
}
```

The important design constraint is therefore:

```text
do syntax is ergonomic sequencing sugar;
function call syntax is ordinary expression composition;
both elaborate through the same row-indexed ambient computation model.
```

Open syntax question: the examples above use `=` before the body because the goal is to
avoid a visually noisy `fn ... { do { ... } }` nesting. The final target grammar still
needs to decide whether expression-bodied functions use `=`, whether block-bodied
functions can be expression bodies directly, and whether `do { ... }` is allowed as the
whole function body without an extra outer block.

### 3.2 Rows are requirements, not authority

A row item says the computation may require something. It does not grant permission.
Admission, provider installation, handler scopes, role entailment, policy decisions,
resource ownership, and evidence discharge decide whether the requirement is satisfied.

### 3.3 Core owns semantics

Surface constructs must elaborate into Core terms plus row facts, discharge metadata,
public summaries, and sidecar evidence. Surface forms may be ergonomic, but they should not
define separate execution paths.

### 3.4 Capabilities are operations plus discharge

The target capability story is:

```text
operation identity + row item + contracts + provider + admission + optional extern
```

If `capability` remains, it is a domain-friendly declaration form over this model. It is
not an independent semantic subsystem.

Ordinary row spelling should name operations directly, such as `{fs.read}` or
`{net.request}`. The authority-bearing status of an operation is discharged by
admission/provider/evidence rules, not by a `cap` prefix in the operation row item. A
provider that handles an operation contributes its own authority/admission/host/provenance
row requirements, and those requirements are introduced or discharged by the ordinary
row-environment/admission mechanisms. The exact syntax for authority/admission facts
remains open, and authority multiplicity/lifetime is a separate design topic.

### 3.5 Contracts share machinery, not lifecycle

Hoare contracts, laws, properties, obligations, and temporal contracts should meet at the
row/discharge/evidence boundary. They should not be flattened into one logical form.
Properties remain advisory/test evidence unless explicitly promoted through a proof/evidence
mechanism.

### 3.6 Runtime organization is explicit

Files and definitions do not start systems. App admission starts systems. Supervisors
organize processes. Behaviours are interfaces plus runners. Agent loops are behaviour
instances plus effects plus supervision. Graphs are declarations interpreted by runners.

### 3.7 Memory is a boundary discipline

The first target memory story is process-region ownership:

```text
process owns region;
send crosses region;
termination releases region;
long-lived state is explicit;
sharing requires an explicit resource/capability story.
```

Rust remains the implementation substrate, not the whole Ash semantics.

### 3.8 Corpus migration replaces compatibility preservation

The remaining old uses are project-owned standard library, documentation examples, and
tests. Therefore the target language does not need a broad language-level compatibility
layer. It needs target replacements, coverage, diagnostics, and scheduled corpus migration.

## 4. Convergence Tracks

The tracks below are ordered by dependency, not by calendar. Later `docs/plan/` work can
split them into phases and tasks.

### 4.1 Semantic spine

Goal:

```text
surface -> Core -> typed Core -> CPS -> runtime
```

Required convergence:

- SPEC-095b must stop describing old forms as target obligations where NOTE-015 has moved
  them to corpus migration or library space.
- SPEC-096b/SPEC-097b/SPEC-098b/SPEC-099/SPEC-100 must agree on row item taxonomy,
  discharge kinds, provider frames, failure classes, and public summaries.
- Core checking should be the authority for row facts before CPS lowering.
- CPS should remain the executable control/effect representation, not a second source
  language.

Readiness gate:

```text
Any accepted surface target form has a documented Core shape, row/discharge facts, CPS
lowering shape, and diagnostic boundary.
```

### 4.2 Surface and corpus convergence

Goal: reduce privileged surface islands while keeping useful authoring ergonomics.

Current forms should be classified as:

```text
core primitive
substrate primitive
library surface
effect operation declaration
contract/evidence declaration
app/runtime declaration
corpus migration target
removal candidate
```

The key corpus migrations are:

| Current/project-owned use | Target home |
|---|---|
| `workflow` declaration | governed `fn`, app entry, service child, or library carrier |
| workflow headers | row/admission/contract/app metadata |
| `act { ... }` | target `do` or library profile helper |
| `do:Act`, `do:Proc`, `do:Workflow` | row profile annotations only if they remain useful |
| `ret` | `return` inside `do`, or ordinary expression tail |
| capability declarations | `effect` declarations or restricted domain authoring form |
| capability calls | typed operation raise/call with provider discharge |
| workflow statements | process/channel/failure/contract/policy/library operations |

Readiness gate:

```text
Do not remove a current form until the target replacement exists, stdlib/docs/tests have
target-form coverage, and historical examples are rewritten or explicitly marked historical.
```

### 4.3 Interface, ADT, row-parameter, and inference convergence

Goal: make row parameters in interfaces, impls, and ADTs obvious in the elaborated program
without requiring the implementation to solve arbitrary global higher-kinded inference
problems.

Rows can appear in ordinary type constructors:

```ash
type Producer<r, A> =
  | Producer(next: Unit -> {r} Option<(A, Producer<r, A>)>)
```

They can also appear through type-constructor parameters:

```ash
interface Pull<P> {
  next<r, A>(p: P<r, A>) -> {r} Option<(A, P<r, A>)>
}
```

The second example is only acceptable if surface elaboration can make the hidden kinds and
types explicit:

```text
r : Row
A : Type
P : Row -> Type -> Type
```

The target inference budget should be:

| Inference class | Target posture |
|---|---|
| Local kind inference | Required. Infer `r: Row` from row position and `A: Type` from type/value position. |
| Local type and row inference | Required for direct constructor/function applications when constraints are local. |
| Declaration-local constructor inference | Required when a parameter is applied inside the same interface/ADT declaration, such as `P<r, A>`. |
| Public boundary inference | Not required. Exported signatures should elaborate to explicit kinds, rows, and associated type facts. |
| Impl selection inference | Conservative. Impl matching should use explicit heads and normalized arguments, not invent missing HKT shapes. |
| Global higher-kinded inference | Optional future capability, not a target prerequisite. |
| Inference through evidence/proof search | Not part of ordinary type inference. Evidence search may discharge constraints, but should not infer arbitrary missing types. |

This keeps simple source ergonomic while avoiding a compiler obligation to infer hairy
types from distant use sites.

Allowed local inference:

```ash
type Task<r, A> =
  | Done(A)
  | Step(Unit -> {r} Task<r, A>)
```

Elaborated shape:

```text
Task : Row -> Type -> Type
```

Boundary that should require annotations:

```ash
fn pipeline(p) {
  pull_twice(map_source(p))
}
```

If the compiler must infer the stream carrier, row, item type, selected `Pull` impl,
callback row, and associated evidence from distant uses, the source should provide an
annotation at a module/public boundary or at the ambiguous local binding.

Implementation-friendly invariant:

```text
Inference is local elaboration, not proof search. After surface elaboration, every
interface, impl, ADT, callable, and public summary has explicit kinds, types, row
parameters, row effects, and associated type/evidence facts.
```

This gives target Ash three escape hatches for complex cases:

1. explicit row/type/kind annotations;
2. associated types/families instead of full HKT parameters;
3. stable public summaries that downstream modules can check without re-inferring the
   author's intent.

Readiness gate:

```text
No row parameter may remain implicit after elaboration, and no public interface/impl/ADT
summary may require downstream global inference to discover which parameters are rows,
types, type constructors, evidence, or modes.
```

### 4.4 Effect, provider, and extern convergence

Goal: one operation identity path from declaration to runtime implementation.

Required convergence:

- decide the canonical `effect` declaration syntax;
- decide whether `capability` remains as target domain syntax or is removed after corpus
  migration;
- define provider surface syntax and how it exposes row peeling, including both explicit
  scoped installation and Frank-like `fn`/optional `operator` definitions using `on` to
  eliminate effectful computation parameters;
- make provider installation an admission event, not declaration side effect;
- choose the primary extern authoring location: effect-level canonical hook,
  provider-level adapter, provider-owned lexical adapter, or a restricted mix;
- keep ordinary Ash code from calling raw externs.

Readiness gate:

```text
For every operation-like surface, the compiler can name the canonical operation identity,
row item, contracts, provider discharge path, extern boundary, and failure classes.
```

### 4.5 Failure and contract convergence

Goal: failure, contract violation, authority denial, policy denial, host failure, process
failure, and app/report failure stop collapsing into one undifferentiated runtime error.

Required convergence:

- concrete recoverable failure row spelling;
- trap/bottom classification and diagnostic payload rules;
- contract blame model;
- interface-to-impl contract subsumption;
- monadic Hoare composition through `bind`;
- lazy/memo contract timing;
- temporal/runtime-monitoring contracts for Proc/Workflow-like libraries;
- surface-to-Core predicate structuralization for SMT/evidence.

Readiness gate:

```text
Every boundary failure can be classified as recoverable failure, trap, contract violation,
authority/admission denial, policy denial, host/provider failure, process failure, or
app/report failure, with row and diagnostic behavior stated.
```

### 4.6 Runtime organization convergence

Goal: replace workflow magic with explicit apps, supervisors, behaviours, services, and
runtime admission.

Required convergence:

- choose app definition surface: source `app`, external manifest, exported Ash value, or
  generated package metadata;
- define app instance identity, namespace isolation, and multi-app daemon policy;
- define supervisor child spec typing;
- define behaviour evidence and runner specialization;
- define service registry and process handle sendability;
- define inter-app communication grants.

Readiness gate:

```text
Loading a module, exporting a function, declaring a workflow-like value, and starting an
app are all distinct events with explicit admission, authority, and failure behavior.
```

### 4.7 Reactive and stream convergence

Goal: separate pull streams, push events, and declarative graph blueprints.

Required convergence:

- pull surface inspired by Producer/Pipe/Machine-style libraries;
- push surface as explicit event/channel operations;
- graph definitions as data interpreted by app/supervisor-started runners;
- bridge adapters with explicit buffering and backpressure;
- graph state retention and hot-reload policy;
- failure/restart behavior for graph interpreters.

Readiness gate:

```text
No reactive form may hide whether data is pulled, pushed, buffered, retained, restarted, or
interpreted by a graph runner.
```

### 4.8 Memory and boundary convergence

Goal: make ownership and lifetime visible at process/app/runtime boundaries.

Required convergence:

- process-region semantics in type/effect docs;
- sendability categories: move, copy, share, reject;
- process-local and region-local values;
- closure capture rules for region-local, provider, handler, resource, and continuation
  values;
- runtime trace events for region cleanup and retained state;
- future subregion/reuse analysis boundaries.

Readiness gate:

```text
Every process/app/channel/closure/handler boundary states what may cross, who owns it after
crossing, and how long it may live.
```

### 4.9 Standard library and documentation convergence

Goal: make the standard library the public home of former tower vocabulary where it remains
useful.

Required convergence:

- `std::act`, `std::proc`, and `std::workflow` become profile/library modules over target
  rows rather than evidence that the language has three semantic towers;
- capability-like libraries use effect declarations and provider/admission APIs;
- process/channel helpers expose boundary rules instead of hiding runtime authority;
- examples teach target forms first;
- tests prove target forms rather than preserving old syntax.

Readiness gate:

```text
The daily-use standard library and documentation can be read without learning the old
language as a separate semantic model.
```

## 5. Gap Register

This register merges the current gaps from NOTE-013 through NOTE-018 into convergence
buckets. It is intentionally high-level; individual specs/plans should own precise tasks.

| Gap | Blocks | Current home |
|---|---|---|
| Canonical effect declaration syntax | effect/provider/capability convergence | NOTE-013, NOTE-015, NOTE-018 |
| Provider surface and row peeling syntax | user-defined effects, provider diagnostics | NOTE-013, NOTE-018 |
| Resume strategy surface | handler composition, multi-shot use | NOTE-013, SPEC-102 |
| Effect-local extern placement | host/FFI safety and provider authoring | NOTE-013, NOTE-014, NOTE-018 |
| Recoverable failure spelling | failure taxonomy and runtime diagnostics | NOTE-015, NOTE-018 |
| Contract blame/subsumption | interface contracts, dynamic checks | NOTE-014, NOTE-018 |
| Monadic Hoare logic | modular contract checking through `bind` | NOTE-014 |
| Lazy/memo contract timing | SPEC-101 integration with contracts | NOTE-014, SPEC-101 |
| Surface contract structuralization | SMT/static contract checking | NOTE-014, NOTE-018 |
| App definition surface | bootstrapping and multi-app runtime | NOTE-016, NOTE-018 |
| Supervisor child typing | behaviours and service runners | NOTE-016, NOTE-018 |
| Behaviour evidence/runner model | OTP-like library design | NOTE-016, NOTE-018 |
| Pull/push/graph bridge semantics | reactive libraries and graph runners | NOTE-016, NOTE-018 |
| Process-region sendability | memory safety and process isolation | NOTE-017, NOTE-018 |
| Closure capture boundary | functions, memory, authority, continuations | NOTE-017, NOTE-018 |
| Local row/type inference budget | interface/impl/ADT ergonomics and compiler tractability | NOTE-019 |
| Computation row taxonomy and pure predicate | row terminology, lazy/memo integration, failure/contract accounting, multi-shot-pure legality | NOTE-020 |
| Corpus replacement map | stdlib/docs/tests migration | NOTE-015, NOTE-018 |

## 6. Suggested Decision Order

The order matters because some decisions unlock many others.

### 6.1 First: semantic vocabulary

Decide the vocabulary that affects all later specs:

1. effect declaration syntax;
2. row item taxonomy and aliases/groups;
3. provider/admission/authority distinction;
4. failure taxonomy;
5. contract discharge taxonomy.

Without this, app/runtime, stdlib, and diagnostics will keep inventing local terms.

### 6.2 Second: compiler boundary

Define how target surface forms lower:

1. row-bearing callable syntax;
2. `do` and profile annotations;
3. effect declarations and operation calls;
4. providers;
5. contracts/evidence;
6. app/runtime declarations if source-level.

This makes Core the only semantic entry point.

### 6.3 Third: runtime boundary

Define what runs and who admits it:

1. app definitions and instances;
2. provider/resource installation;
3. supervisor roots and child specs;
4. process/service/behaviour runner lifecycle;
5. inter-app communication and failure domains.

This removes the workflow bootstrapping magic.

### 6.4 Fourth: corpus migration

Once target replacements exist, migrate project-owned corpus:

1. standard library;
2. reference docs;
3. documentation examples;
4. parser/typechecker/runtime tests;
5. historical examples and old design notes.

This should happen after target coverage exists, not as an up-front compatibility exercise.

## 7. Non-Goals for This Note

This note does not:

- assign task IDs;
- choose implementation phases;
- require immediate removal of any current parser support;
- define final syntax for every target form;
- replace SPEC-095b through SPEC-102;
- claim current implementation conformance to the target fixed point.

It should instead help answer whether future plans move Ash toward the same target.

## 8. Convergence Checklist

A future implementation plan is aligned with target Ash if it can answer:

1. Which target boundary does this change move?
2. Which current form, if any, becomes library surface, target syntax, corpus migration, or
   removal?
3. What Core shape does the surface produce?
4. What row items and discharge facts are produced?
5. Which provider/admission path satisfies them?
6. What failure classes can occur?
7. What evidence, report, trace, or public summary is emitted?
8. What memory/ownership boundary is crossed?
9. Which stdlib/docs/tests corpus uses must be migrated?
10. Which older specs or notes become historical after the change?

If a change cannot answer these questions, it probably preserves an old island rather than
moving toward convergence.

## 9. Working Principle

The convergence rule:

```text
Every Ash feature should either be a small language primitive, a Core/CPS substrate fact, a
row/discharge/evidence boundary, an admitted runtime/app boundary, or a library surface over
those mechanisms. Anything else is probably old workflow magic in a new spelling.
```

The target is not minimalism for its own sake. It is a disciplined way to keep Ash's
capability, workflow, contract, process, and agent ideas while giving them one semantic
path.

## 10. References

Internal references:

- [NOTE-013: Ambient Monad and Handler Composition Algebra](NOTE-013-AMBIENT-MONAD-AND-HANDLER-COMPOSITION-ALGEBRA.md)
- [NOTE-014: Contract Systems Unification](NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md)
- [NOTE-015: Current-to-Target Language Forms](NOTE-015-CURRENT-TO-TARGET-LANGUAGE-FORMS.md)
- [NOTE-016: Runtime Organization, Behaviours, and Reactive Modes](NOTE-016-RUNTIME-ORGANIZATION-BEHAVIOURS-REACTIVE-MODES.md)
- [NOTE-017: Memory Regions, Ownership, and Utilization](NOTE-017-MEMORY-REGIONS-OWNERSHIP-AND-UTILIZATION.md)
- [NOTE-018: Boundary Discipline for Target Ash](NOTE-018-BOUNDARY-DISCIPLINE.md)
- [NOTE-020: Computation Row Taxonomy and Pure Computation](NOTE-020-COMPUTATION-ROW-TAXONOMY.md)
- [SPEC-095b: Target Grammar](../spec/SPEC-095b-TARGET-GRAMMAR.md)
- [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-098b: Target IR](../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-099: Core Ash](../spec/SPEC-099-CORE-LANGUAGE.md)
- [SPEC-099b: Target Operational Semantics](../spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md)
- [SPEC-100: Core Type Checking](../spec/SPEC-100-CORE-TYPE-CHECKING.md)
- [SPEC-101: Lazy and Memo Computation Modes](../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md)
- [SPEC-102: CPS Continuation Multiplicity](../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md)

## 11. Changelog

- 2026-06-25: Added the handler surface convergence checkpoint for explicit scoped handlers
  and Frank-like `fn`/optional `operator` definitions with `on` computation elimination.
- 2026-06-25: Clarified capabilities-as-providers: providers eliminate operation rows but
  contribute their own authority/admission requirements, with authority introduction and
  discharge handled by ordinary row-environment/admission mechanisms.
- 2026-06-24: Clarified that operation row items use direct operation identities rather
  than `cap` prefixes, while authority/admission syntax remains unresolved.
- 2026-06-24: Added explicit `do` notation versus plain function composition checkpoint,
  including the open expression-bodied function syntax question.
- 2026-06-24: Linked NOTE-020 and updated convergence terminology from effect rows to
  computation rows where NOTE-020 refines the target model.
- 2026-06-24: Initial draft. Synthesizes target Ash convergence tracks across the recent
  notes and target specs, distinguishing semantic convergence from implementation planning.
