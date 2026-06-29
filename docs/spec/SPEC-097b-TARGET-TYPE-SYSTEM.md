---
id: spec.ash.type-system.target
title: Ash Type System — Target State
description: Type system with unified computation rows, row polymorphism, kind-specific discharge, and effect aliases/groups
kind: spec
audience: [human, agent]
authority: design
status: draft
stability: alpha
owner: language
last_verified: 2026-06-20
verified_against:
  specs:
    - docs/spec/SPEC-095a-CURRENT-GRAMMAR.md
    - docs/spec/SPEC-095b-TARGET-GRAMMAR.md
    - docs/spec/SPEC-096a-CURRENT-EFFECT-SYSTEM.md
    - docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md
    - docs/spec/SPEC-006-POLICY-DEFINITIONS.md
    - docs/spec/SPEC-019-ROLE-RUNTIME-SEMANTICS.md
    - docs/spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md
---

# SPEC-097b: Ash Type System — Target State

**Status:** Draft — target type system for unified computation rows
**Scope:** This document defines the type-system semantics we want Ash to have.
It is a goal-state living document that will be refined as implementation progresses.
**Depends on:** SPEC-095b (Target Grammar), SPEC-096b (Target Effect System)

## 1. Summary

This spec defines the type-system obligations for the unified computation-row direction in
SPEC-096b. The central rule is:

```text
a computation's requirement row must be discharged by the ambient effect environment
```

Rows are requirement sets, not authority grants. The type checker may infer, normalize,
compare, and report rows, but authority is provided only by admitted roles, effect providers,
resources, policy handlers, channel endpoints, contract evidence, or runtime/workflow
boundaries.

**Terminology (NOTE-021).** Throughout this document, *computation row* denotes the broad
type-level row concept (the requirement set carried by a computation). *Effect row* is
reserved for referring to the syntactic row literal in source text (e.g. `{PosixFs::read, StdoutLog::write}`).

This draft replaces the earlier loose statement "`{fs} <: {fs, log}`" with explicit relations:

1. requirement inclusion: what a computation requires;
2. environment discharge: what the context provides or proves;
3. function subtyping: when one callable can stand in for another.

## 2. Current type-system baseline

### 2.1 Existing types

Current parser/core/typechecker code does not yet carry a complete effect-row type in ordinary
function signatures. The live type surfaces include named types, type constructors, tuples,
records, associated types, and callable types, while workflow/effect information is tracked
separately in existing workflow and legacy capability machinery.

The existing 4-point `Effect` lattice in `crates/ash-core/src/effect.rs` is a coarse
workflow/effect classification:

```rust
pub enum Effect {
    Epistemic = 0,
    Deliberative = 1,
    Evaluative = 2,
    Operational = 3,
}
```

That lattice is not a row system. The row system introduced here must preserve compatibility
with existing effect classifications during migration rather than pretending the old
representation already has row structure.

### 2.2 Migration constraint

The first implementation slice should add row summaries and row checking around existing
carriers. It must not require immediate deletion of `Type::Fn`, `Type::Fun`, `Act<T>`,
`Proc<T>`, `Workflow<T>`, workflow headers, or current legacy capability declarations.

## 3. Type-level representation

### 3.1 Row carrier

A conforming implementation needs a shared row carrier. The exact Rust home is an
implementation decision, but the semantic shape is:

```rust
pub struct EffectRow {
    pub items: Vec<EffectItem>,
    pub tail: Option<RowVar>,
}

pub struct RowVar {
    pub name: Name,
    pub constraints: Vec<RowConstraint>,
}
```

A closed row has `tail: None`. An open row has `tail: Some(r)` and represents the listed
requirements plus an unknown remainder.

> **Kind note (NOTE-021).** The source-level kind is `Row` (e.g. `r: Row`). The Rust type
> name (`EffectRow`) is an implementation detail and is not part of the source surface.

### 3.2 Effect item identity

Rows contain typed row items, not bare strings. (The Rust enum variant is named `EffectItem`
as an implementation detail; the source-level concept is a *row item* or *computation row item*,
per NOTE-021.)

```rust
pub enum EffectItem {
    Operation(OperationEffect),
    Resource(ResourceEffect),
    Role(RoleEffect),
    Policy(PolicyEffect),
    Contract(ContractEffect),
    Channel(ChannelEffect),
    Process(ProcessEffect),
    Failure(FailureEffect),
    Evidence(EvidenceEffect),
    Group(EffectGroupRef),
}
```

Every row item must have a canonical identity used for duplicate elimination, row
comparison, diagnostics, and module-summary export. The identity must include its namespace.
For example, `PosixFs::read`, `policy PosixFs::read`, and `role PosixFs::read` are distinct even if their
textual tails match.

### 3.3 Operation effect

An operation effect references an impl-type-qualified operation identity (NOTE-025). The
interface declares the operation signature (the sort); the impl type is the identity
qualifier. In generic code, the row item is abstract: `F::read` where `F: Fs`. After
monomorphization, it is concrete: `PosixFs::read`.

```rust
pub struct OperationEffect {
    pub impl_type: NamePath,
    pub operation: Option<Name>,
}
```

The `impl_type` field is the concrete impl type after monomorphization (e.g., `PosixFs`).
Different impl types produce distinct operation identities even for the same interface
method — enabling multiple simultaneous handlers.

A whole-interface sort constraint such as `F: Fs` bounds the operation set available to
generic code. It does not itself appear as a row item; row items are always
impl-type-qualified operations.

**Revision (NOTE-025):** The earlier draft used `interface: NamePath` qualified by the
interface name (e.g. `Fs.read`). NOTE-025 revises this: the impl type is the identity qualifier,
not the interface.

### 3.4 Role effect

```rust
pub struct RoleEffect {
    pub role: NamePath,
}
```

A role effect requires role admission. It does not by itself expand into operations until
the role definition and admission context are known.

### 3.5 Policy effect

```rust
pub struct PolicyEffect {
    pub binding: NamePath,
    pub decision_domain: Option<PolicyDecisionDomain>,
}
```

Policy effects reference named policy bindings, following SPEC-006/SPEC-007. Anonymous policy
expressions are out of scope for this spec.

### 3.6 Contract effect

```rust
pub enum ContractEffect {
    Requires(PredicateRef),
    Ensures(PredicateRef),
    Invariant(PredicateRef),
    Law { name: Name, predicate: PredicateRef },
    Obligation(NamePath),
    Guard(PredicateRef),
}
```

Predicate references must preserve source scope, binder information, and discharge status.
A predicate printed in a row is not enough; the type checker must know which names are legal
inside it.

Per NOTE-031, a predicate reference also carries a checked predicate summary:

```rust
pub enum PredicateClassification {
    StaticPredicate,
    DynamicPredicate,
}

pub struct PredicateSummary {
    pub ty: Type,                         // must be Bool
    pub free_vars: Vec<Name>,
    pub snapshot_refs: Vec<SnapshotRef>,
    pub classification: PredicateClassification,
    pub proof_fragment: Option<ProofFragment>,
    pub lowered_predicate: PredicateRef,
    pub predicate_env: PredicateEnvRef,
    pub dynamic_plan: Option<DynamicPredicatePlan>,
    pub diagnostic_shape: DiagnosticShape,
}
```

A rejected predicate is not represented as a `PredicateRef`; it is a type-checking diagnostic.
The checker rejects predicates that require authority, perform process/workflow operations,
install or dispatch handlers, observe time/randomness/environment/global mutable state, inspect
allocation identity, or force lazy/memo values outside a contract-owned observation boundary.

Per NOTE-034, values produced by authority-bearing operations may still be bound in the
predicate environment as ordinary values. The predicate may inspect such a value, but it may
not perform the operation that produced it or use hidden provider/role admission. Capability
admission failure, operation failure, predicate false, and predicate evaluator fault remain
distinct typing/diagnostic classes.

Per NOTE-035, trace contracts are checked by a separate trace-contract judgment:

```text
Γtrace ⊢ formula ⇓ TraceContract
```

`Γtrace` contains event schemas, process/channel/resource identities, timer facts, workflow
ledger fact schemas, evidence/provenance policies, redaction rules, and monitor scope
boundaries. The checker rejects formulas that mention facts outside the monitor scope or
require authority not represented as a recorded fact. A formula that mentions only operational
trace facts is `Proc`-like; a formula that mentions obligation/evidence/commitment facts is
`Workflow`-like. This is classification by ambient feature profile, not by a separate
`Proc<A>` or `Workflow<A>` constructor.

`old(path)` references lower to boundary-local `SnapshotRef` metadata. The path is a field path
through a value visible at the contract boundary, not an arbitrary computation.

Per NOTE-033, a `PredicateRef` points at a typed lowered predicate artifact rather than source
text. That artifact records the owning `BoundaryId`, typed predicate binders, admitted
predicate-function identities, the lowered predicate tree, `SnapshotRef`s, proof fragment, and
optional dynamic runtime-check plan. The stable identity for evidence caching and optimizer
checks is computed from the lowered tree, binder identities, snapshot references, admitted
predicate-function identities, and relevant type encodings, not from contract source text
alone.

### 3.7 Channel effect

```rust
pub enum ChannelEffect {
    Message {
        channel: NamePath,
        mode: ChannelMessageMode,
        message_type: Type,
        guard: Option<PredicateRef>,
    },
    Close {
        channel: NamePath,
    },
}

pub enum ChannelMessageMode {
    Send,
    Receive,
    Select,
}
```

Channel guards are contracts over communication. The guard's message binder must be explicit
in the typed representation, even if the surface syntax later chooses a concise spelling such
as `message`. `close` has no message type and cannot carry a message guard in this draft.

### 3.8 Process, failure, and evidence effects

```rust
pub struct ProcessEffect {
    pub operation: ProcessOperation,
}

pub struct FailureEffect {
    pub failure_type: Option<Type>,
}

pub struct EvidenceEffect {
    pub sink: NamePath,
    pub kind: EvidenceKind,
}
```

These effects are profile-sensitive. `proc spawn` must not type-check in a pure or `Act`-only
profile unless the computation is explicitly lifted or checked in a `Proc`-capable environment.

### 3.9 Newtype identity and representation

Per SPEC-095b §6.7 and NOTE-026, a `newtype` introduces a fresh nominal type constructor and
a value-level wrapper constructor around one inhabited representation type:

```ash
newtype CustomFs = CustomFs(PosixFs);
newtype Tagged<Label> = Tagged(String);
```

The type checker treats the newtype as distinct from its representation:

```text
CustomFs ≠ PosixFs
Tagged<Admin> ≠ Tagged<User>
```

This distinction is definitional. A transparent alias may canonicalize to its origin head;
a newtype must not. At runtime, the newtype shares the representation of its wrapped type and
does not add a layout field beyond that representation.

The wrapper constructor and pattern are explicit zero-cost conversions:

```ash
let fs = CustomFs(posix);
let CustomFs(inner) = fs;
```

There is no automatic coercion between a newtype and its representation. If a future phase adds
unsafe coercions or derived impls, those features must preserve the nominal identity rule. Type
parameters absent from the representation type are phantom parameters: they affect identity and
type equality, but not runtime layout.

## 4. Row syntax and kinds

### 4.1 Proposed source syntax

SPEC-095a remains the current parser-derived grammar. The target row syntax for this
type-system packet is:

```ash
{}                                      -- empty row
{PosixFs::read}                                -- closed row
{PosixFs::read, policy production_rate}         -- multiple requirements
{PosixFs::read | r}                             -- open row
{r}                                      -- whole-row variable
{IO}                                     -- transparent alias or group reference
```

Rows are not ordinary record types. A parser/typechecker implementation must distinguish
record type `{x: Int}` from the computation row `{PosixFs::read}` by grammar context or an
explicit row-introducing token chosen by the syntax spec. It must also distinguish `{r}`
from an alias/group reference by kind and namespace resolution.

### 4.2 Row variable kind

Row variables have a distinct kind, `Row` (NOTE-021). This spec uses `Row` in source-level
prose and examples. (Earlier drafts used `EffectRow`; the Rust type name `EffectRow` may
persist as an implementation detail — see §3.1.)

```ash
fn map<A, B, r: Row>(xs: List<A>, f: A -> {r} B) -> {r} List<B> { ... }
```

The implementation may infer row-variable kinds when unambiguous.

### 4.3 Row constraints

A row variable may carry constraints:

```ash
fn log_and_return<A, r>(x: A) -> {StdoutLog::write | r} A { ... }
```

This means the resulting computation requires at least `StdoutLog::write` plus whatever `r`
requires. It does not mean `r <: {StdoutLog::write}`.

If explicit constraint syntax is added, it should name the intended relation directly:

```ash
where r discharges {policy production_rate}
where r excludes {proc spawn}
where r profile Act
```

The earlier wording `r <: {fs}` is too ambiguous and should not be normative.

## 5. Row normalization

Before comparing rows, the type checker normalizes them.

Normalization must:

1. expand transparent aliases;
2. preserve diagnostic group/profile names as annotations;
3. canonicalize item identities with namespaces;
4. remove exact duplicate items;
5. preserve open-row tails;
6. reject or defer ambiguous group references;
7. preserve predicate identity for contracts and guards rather than stringifying predicates.

Duplicate elimination is exact. For example:

```text
Fs           != PosixFs::read, unless a later effect-sort rule defines expansion
role admin   != PosixFs::read, even if admin can entail PosixFs::read
```

Role entailment happens during discharge, not normalization.

## 6. Requirement inclusion, discharge, and subtyping

### 6.1 Requirement inclusion

`Requires(A) ⊆ Requires(B)` means every requirement in `A` also appears in `B` after
normalization and alias expansion.

This relation is useful for comparing two requirement rows, but it is not the same as checking
whether a program may run. Running a program needs environment discharge.

### 6.2 Environment discharge

`Env ⊢ R discharged` means the ambient environment discharges every item in requirement
row `R`.

The discharge rule is kind-specific:

```text
Env ⊢ C.op discharged          if Env has admitted provider/effect C.op
                                or Env has admitted role R and role R entails C.op

Env ⊢ role R discharged        if Env has admitted role R

Env ⊢ policy P discharged      if Env has a policy handler/evaluator for named policy P
                                in the required decision domain

Env ⊢ requires {p} discharged  if p is statically proved, evidence-proved,
                                or lowered to a dynamic runtime check

Env ⊢ channel receive C T discharged
                              if Env owns a receive-capable endpoint C with message type T
                              and a guard strategy is defined if a guard exists
```

The type checker may statically reject missing discharge, or it may produce a residual row that
must be discharged by a later admission/runtime boundary. The chosen phase boundary must be
explicit in implementation tasks.

### 6.3 Function subtyping

Function subtyping is contravariant in requirements: a function that requires fewer effects may
be used where a function requiring more effects is expected.

```text
Requires(f_actual) ⊆ Requires(f_expected)
------------------------------------------------
(A -{f_actual}-> B) <: (A -{f_expected}-> B)
```

Example:

```text
(A -{PosixFs::read}-> B) <: (A -{PosixFs::read, StdoutLog::write}-> B)
```

The reverse is not valid. A function that requires logging cannot be used where only
file-read authority was expected.

### 6.4 Empty row

The empty row is the least requirement row:

```text
Requires({}) ⊆ Requires(R)
```

This means pure functions can be used in effectful contexts. It does not mean an effectful
context is pure.

### 6.5 Contract subsumption

Contract discharge and behavioral contract subsumption are separate checks.

A contract item may be removed from a residual row only when the checker records a discharge:

```text
prove(p) or evidence(p) or runtime_handler(requires p)
------------------------------------------------------
requires {p} is discharged
```

The type checker must not silently coerce `{requires {p}}` to `{}` without recording the mode
and evidence boundary.

When an `impl` method refines an interface method's Hoare contract, the checker must also
verify NOTE-027's behavioral subtyping rule eagerly at the `impl` definition site:

```text
interface contract: {P} C {Q}
impl contract:      {P'} C {Q'}

{P'} C {Q'} ⊑ {P} C {Q}
  iff
P ⇒ P'       -- impl precondition is weaker or equal
Q' ⇒ Q       -- impl postcondition is stronger or equal
```

The impl may accept more inputs and promise more outputs. It must not demand more from callers
than the interface promised, and it must not guarantee less than the interface contract.

Blame follows polarity. A failed `requires` check blames the caller/provider of the argument;
a failed `ensures` check blames the callee or impl that promised the result. Subsumption checks
must preserve the source contract clause so diagnostics can report which party declared the
failed obligation.

### 6.6 Contract composition through sequencing

Per NOTE-030, computation rows and contract predicates compose differently. For sequencing or
`bind`, rows compose by union, while contracts compose through a predicate-transformer rule.

Given:

```text
m : Comp<ρm, A>
  requires P
  ensures  Q(a)

k : A -> Comp<ρk, B>
  requires R(a)
  ensures  S(a, b)
```

The composed computation has:

```text
bind(m, k) : Comp<ρm ∪ ρk, B>
  requires P ∧ ∀a. Q(a) ⇒ R(a)
  ensures  ∃a. Q(a) ∧ S(a, b)
```

The central proof obligation is `∀a. Q(a) ⇒ R(a)`: the producer's postcondition must
establish the continuation's precondition for every intermediate value the producer may
return. If the checker proves this implication, the continuation precondition is discharged by
the producer postcondition and the discharge is recorded. If the checker cannot prove it, the
obligation may be rejected, deferred, or demoted to a dynamic contract check according to the
active profile.

The existential postcondition is the strongest generic summary. A public function may expose a
simpler postcondition `T(b)` when the checker proves:

```text
∀b. (∃a. Q(a) ∧ S(a, b)) ⇒ T(b)
```

This rule applies to lowered sequencing forms such as `let a = m(); k(a)`. `bind` is
meta-notation, not required surface syntax.

## 7. Row polymorphism

### 7.1 Higher-order functions

A higher-order function can preserve the row of a callback.

```ash
fn map<A, B, r: Row>(xs: List<A>, f: A -> {r} B) -> {r} List<B> { ... }
```

The result row is whatever the callback requires, plus any requirements from `map` itself. If
`map` logs internally, the row must include that requirement:

```ash
fn map_logged<A, B, r: Row>(xs: List<A>, f: A -> {r} B) -> {StdoutLog::write | r} List<B> { ... }
```

### 7.2 Inference

Effect inference may infer closed rows for ordinary functions:

```ash
fn add(a: Int, b: Int) -> Int { a + b }        -- inferred requirement row {}
fn read(path: String) -> String { PosixFs::read(path) } -- inferred row includes PosixFs::read
```

Whether the inferred row appears in the surface type, module summary, or diagnostics is an
implementation detail. The semantic row must be available to checking and export/import if
the function is public.

### 7.3 Open-row solving

Open-row solving must avoid accidental privilege loss or gain.

For a call requiring `{PosixFs::read | r}` in an environment containing `{PosixFs::read, StdoutLog::write}`,
the solver may instantiate `r` with `{StdoutLog::write}` if the expected type demands the larger row.
It must not infer that `StdoutLog::write` is required unless it is used, expected, or otherwise
constrained.

### 7.4 No implicit tower lifts

Row polymorphism does not add implicit lifts across `Pure`, `Act`, `Proc`, and `Workflow`
profiles. If a `Proc` computation binds an `Act` computation, the explicit lift or embedding
rule remains required until a separate spec changes it.

## 8. Type checking rules by effect kind

### 8.1 Operation calls

An interface method call that is not locally resolved contributes an operation row item. The
operation identity is impl-type-qualified after monomorphization (NOTE-025). In generic
code, `F.read(path)` where `F: Fs` contributes `F::read`; after specialization, this becomes
`PosixFs::read`.

```text
call F.op(args) : A     where F: Iface
------------------------
row includes F::op      (abstract; becomes ImplType::op after monomorphization)
```

If the operation is called through a binding name, the row identity must resolve through the
binding to the impl-type-qualified operation identity. Diagnostics should show both the
binding and canonical impl-type-qualified operation when helpful.

### 8.2 Role use

A function or computation requiring a role includes a role effect. The role effect can also
discharge entailed operation effects, but only when the role is admitted in the ambient
environment.

The type checker must not expand role definitions at declaration sites in a way that loses the
role identity. Role identity matters for audit and diagnostics.

### 8.3 Policy use

Policy effects must resolve to named policy bindings. The type checker must reject or defer:

- unknown policy names;
- anonymous inline policy expressions in row position;
- policy effects used in a decision domain incompatible with the consumer;
- policy effects that cannot be exported because their named binding is private.

### 8.4 Contracts and predicates

Contract predicates must be checked for well-scoped names and purity/effect safety according
to their contract kind.

- `requires` may mention parameters and earlier lexical bindings.
- `ensures` may mention parameters and `result`.
- `guard` may mention the channel message binder plus allowed lexical bindings.
- `law` may mention its law parameters and any names permitted by the law/proof specs.

A contract predicate that itself requires effects must be rejected unless a later spec admits
effectful predicates explicitly.

Per NOTE-031, predicate well-formedness is stricter than row emptiness:

```text
Γp ⊢ e : Bool
row(e) = {}
no_forces(e)
no_authority(e)
no_handler_dispatch(e)
stable_observer(e)
---------------------------------
Γp ⊢ e ⇓ PredicateSummary
```

The `stable_observer` premise rejects row-empty but operationally unstable observations such as
time, randomness, pointer/allocation identity, and force-count inspection. Well-formed
predicates are classified as SMT-safe static predicates or pure dynamic predicates. Pure forms
outside the current proof fragment are dynamic, not invalid. Rejected predicates stop before
SMT, dynamic lowering, or runtime checking.

### 8.5 Channels

A send operation contributes a `channel send` effect. A receive operation contributes a
`channel receive` effect. A guarded receive also contributes or embeds a guard contract.

The type checker must verify at least:

1. endpoint name resolution;
2. direction compatibility;
3. message type compatibility;
4. guard predicate well-scoping;
5. profile compatibility (`Proc` or `Workflow`, not `Pure`/plain `Act`).

Full protocol/session compatibility is a future spec.

### 8.6 Failure

A `fail` expression contributes a failure effect. It must not type-check by converting the failure
into a domain result unless the source syntax explicitly constructs that domain result.

### 8.7 Evidence/reporting

Evidence and report effects must be checked against available evidence sinks or
workflow/reporting boundaries. Public functions exporting evidence effects must preserve those
effects in module summaries.

### 8.8 Handler typing

A handler is a function whose parameter is a computation thunk of type `Unit -> {ImplType::op | r} A`
and whose return type is the *answer type* `Ans`. The handler's output row is `{r}`: the
peeled operations are removed from the requirement row while the residual row `r` is
propagated to the handler's caller (NOTE-023). Operation identities are impl-type-qualified
(NOTE-025).

For each peeled operation `ImplType::op` with result type `B_op` declared by the interface,
the handler branch receives a *continuation* parameter of type `B_op -> {r} Ans`. The
continuation is an ordinary function parameter; its row is `{r}`, the residual row after
peeling. The result type `Ans` is shared across all branches and the handler's own return
type.

**Multiplicity.** The continuation's row `{r}` determines whether it may be invoked more than
once:

- When `{r}` is non-empty, the continuation is *affine*: it may be called at most once. The
  residual operations in `{r}` are not re-entrant under the handler, so a second invocation
  cannot be well-typed.
- When `{r}` normalizes to `{}` (the empty row), the continuation is *multi-shot* (copyable):
  it may be invoked zero or more times, because no residual operation can be violated by
  re-entry.

This is consistent with the SPEC-102 Core/CPS multiplicity encoding, where an empty residual
row yields a pure continuation that the runtime may freely duplicate.

```text
handler : (Unit -> {ImplType::op | r} A) -> {r} Ans
branch_op(k : B_op -> {r} Ans) -> {r} Ans
```

Typical shapes:

```ash
-- Affine continuation (residual row non-empty)
handler handle_fsWithLogging<A, r: Row>(
    thunk: Unit -> {PosixFs::read, Log::write | r} A
) -> {r} A { ... }

-- Multi-shot continuation (residual row empty after peeling)
handler handle_pureRetry<A>(
    thunk: Unit -> {PureRetry::retry | {}} A
) -> A { ... }
```

## 9. Effect aliases and groups

### 9.1 Transparent aliases

Transparent aliases expand during normalization.

```ash
effect alias IO = {PosixFs::read, PosixFs::write, StdoutLog::write};
```

Alias expansion must be cycle-checked. Cycles are rejected.

### 9.2 Diagnostic groups

Groups preserve a name for diagnostics while expanding to concrete row items.

```ash
effect group WorkflowIO = {
    PosixFs::read,
    StdoutLog::write,
    evidence audit_log,
};
```

A missing requirement diagnostic should be allowed to say:

```text
missing WorkflowIO (specifically StdoutLog::write)
```

### 9.3 Authority bundles are different

An authority bundle, if added, is not a transparent alias. It must have an admission rule and
provenance. The type checker must not treat `effect alias Admin = {PosixFs::write}` as granting
write authority.

### 9.4 Export/import

Public aliases/groups used in public function rows must be exported in module summaries. Private
aliases in public rows must either be expanded before export or rejected with a diagnostic that
preserves opacity rules.

## 10. Integration with existing features

### 10.1 Generics

Rows participate in generic signatures through row variables. Row variables are separate from
ordinary type variables.

### 10.2 Interfaces

Interface methods may carry computation rows, but the parser and typechecker must respect the live
interface method syntax. If method signatures currently use positional parameter types, a
syntax task must update that grammar before examples with named parameters become normative.

Example target shape:

```ash
interface EffectfulMap<F> {
    map<A, B, r: Row>(F<A>, A -> {r} B) -> {r} F<B>;
}
```

### 10.3 Associated types and families

Associated types may mention row-bearing callable types only after the row carrier is available
in the canonical type-expression IR and module-summary format. Until then, such examples are
design sketches, not implementation-ready syntax.

### 10.4 Closures and capture safety

Closure capture remains governed by effect-safe capture rules. A closure's row includes the
effects required by its body, but capture permission is separately constrained by the creation
context. A pure closure must not become effectful by silently capturing an effectful value.

## 11. Diagnostics

A conforming implementation must provide diagnostics that name the missing item kind and likely
fix. Examples:

| Case | Required diagnostic content |
|------|-----------------------------|
| missing authority | effect/interface operation, current admitted roles/bindings, possible role/effect admission fix |
| missing role | role name, execution boundary that must admit it |
| policy not found | policy binding name and namespace searched |
| policy decision mismatch | required decision domain and policy's available decision domain |
| contract not discharged | contract kind, predicate source, allowed discharge modes |
| channel direction mismatch | endpoint name, expected direction, found direction |
| guarded channel without guard behavior | guard source and required runtime/spec decision |
| process effect in Act/Pure | process operation and required `Proc`/`Workflow` profile |
| alias cycle | cycle path |
| private alias in public row | private alias, public item, suggested expansion/export fix |

Generic `RowMismatch` is allowed only as an internal error category. User-facing diagnostics
must classify the row item kind.

## 12. Implementation slices

### 12.1 Slice A: Row data and normalization

- Add typed row carriers.
- Add effect item identities and namespaces.
- Add transparent alias/group expansion in the checker or a pre-check elaboration step.
- Add duplicate elimination and cycle rejection.

### 12.2 Slice B: Row summaries

- Infer row summaries for functions and closures.
- Preserve row summaries in public module exports.
- Keep existing runtime representation unchanged.

### 12.3 Slice C: Operation and role discharge

- Operation calls contribute operation effects.
- Admitted role effects discharge entailed operations.
- Missing authority diagnostics distinguish role/effect/policy failures.

### 12.4 Slice D: Policy and contract discharge

- Policy effects resolve to named policy bindings.
- Contract effects preserve static/evidence/dynamic discharge status.
- Channel guards are represented as contract effects or embedded guarded channel effects.

### 12.5 Slice E: Profile checking

- Define `Pure`, `Act`, `Proc`, and `Workflow` row profiles.
- Reject process/channel/governance effects in lower profiles unless an explicit lift or
  boundary handles them.

## 13. Acceptance criteria

A future implementation plan should include tests for:

1. pure function inferred as empty row;
2. operation call contributes an operation effect;
3. role admission discharges an entailed operation;
4. role name alone does not grant authority when not admitted;
5. named policy effect resolves and unknown policy fails;
6. anonymous policy expression in a row is rejected or explicitly deferred;
7. `requires` contract cannot be erased without discharge evidence;
8. `ensures` predicate has access to `result` and parameters only;
9. guarded receive preserves channel effect plus guard predicate;
10. channel receive is rejected in a pure/Act-only profile;
11. transparent alias expands and grants no authority;
12. group name survives diagnostics;
13. alias cycles are rejected;
14. function subtyping accepts fewer requirements where more were expected and rejects the reverse;
15. public functions export row summaries without leaking private aliases.

## 14. See also

- [SPEC-095a: Current Grammar](SPEC-095a-CURRENT-GRAMMAR.md) — what the parser accepts today
- [SPEC-095b: Target Grammar](SPEC-095b-TARGET-GRAMMAR.md) — target surface syntax
- [SPEC-096a: Current Effect System](SPEC-096a-CURRENT-EFFECT-SYSTEM.md) — current 4-point lattice and tower
- [SPEC-096b: Target Effect System](SPEC-096b-TARGET-EFFECT-SYSTEM.md) — semantic model for effect rows
- [SPEC-098b: Target IR Changes](SPEC-098b-TARGET-IR.md) — IR representation for effect rows
- [SPEC-099b: Target Operational Semantics](SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md) — runtime semantics for effect rows
- [SPEC-006: Policy Definition Syntax](SPEC-006-POLICY-DEFINITIONS.md)
- [SPEC-019: Role Runtime Semantics](SPEC-019-ROLE-RUNTIME-SEMANTICS.md)
- [SPEC-052: Capability Interfaces and Implementations](SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md)

## 15. Evaluation Modes

### 15.1 Purpose

Evaluation modes are algorithmic contracts that control when expressions are evaluated, not performance hints. They affect control-flow shape, stack usage, sharing behavior, and asymptotic complexity. Modes are preserved across function boundaries and checked at call sites.

### 15.2 Modes

| Mode | Evaluation timing | Sharing | Use case |
|------|------------------|---------|----------|
| `strict` | Immediate, at binding or call site | None | Accumulating folds, in-place mutation, resource acquisition |
| `lazy` | On demand, at use site | None | Streams, generators, short-circuiting, infinite structures |
| `memo` | On demand, once, then cached | Yes | Recursive patterns, dynamic programming, shared subcomputations |

Default mode is `strict`.

### 15.3 Syntax

```ash
-- Binding site:
let x = expr;              -- strict (default)
let lazy x = expr;         -- lazy
let memo x = expr;         -- memo

-- Parameter site:
fn foo(x: Int) -> Int;                 -- strict parameter
fn bar(lazy x: Int) -> Int;            -- lazy parameter
fn baz(memo x: Int) -> Int;            -- memo parameter

-- Return site:
fn gen() -> lazy List<Int>;            -- lazy return
fn compute() -> Int;                   -- strict return (default)
```

### 15.4 Invariance

Modes are **invariant** in the type system. No implicit conversion between modes.

```text
strict A  ≮:  lazy A
lazy A    ≮:  strict A
memo A    ≮:  lazy A
lazy A    ≮:  memo A
```

Mode mismatch at a boundary is a type error.

### 15.5 Explicit Conversions

All mode changes are explicit function calls:

```ash
-- Safe (no observable behavior change):
let lazy_thunk = delay(value);      -- strict -> lazy: wrap in thunk
let memo_thunk = delay_memo(value); -- strict -> memo: wrap in memo-thunk

-- Unsafe (changes when effects fire, including bottom):
let forced = force_unsafe(lazy_val);     -- lazy -> strict: force thunk
let memoized = memoize_unsafe(lazy_val); -- lazy -> memo: add cache
let stripped = strip_cache_unsafe(memo_val); -- memo -> lazy: remove cache
```

The `_unsafe` suffix indicates that the conversion changes temporal behavior: effects (including divergence) may fire at a different time or a different number of times.

### 15.6 Row Accounting

The total computation row is the same regardless of mode. Mode affects **when** effects fire, not **what** effects are present.

```text
let lazy x = {db.read} expr;  -- row of binding site: {}
                               -- row of x's body: {db.read}
                               -- row of force_unsafe(x): {db.read}

let memo y = {db.read} expr;  -- row of binding site: {}
                               -- row of first force: {db.read}
                               -- static row of each force site: {db.read}
                               -- dynamic cache-hit row: {}
```

The checker must use the static force-site row unless a later state-sensitive analysis proves
the memo cell is already filled on that path. A cache hit may perform no dynamic effects at
runtime, but ordinary type-checker summaries and diagnostics must not erase the latent row
from a force site merely because the thunk is memoized.

### 15.7 Purity and contract timing

Purity is denotational. A type-level attribute is purity-preserving when it preserves
referential transparency at the relevant observation boundary. `strict`, `lazy`, `memo`, and
the handler marker are not row items and do not by themselves make a computation impure.

The residual or latent row determines purity:

```text
pure(strict A)  iff the current residual row is {}
pure(lazy A ρ)  iff ρ = {} at force sites
pure(memo A ρ)  iff ρ = {} at force sites
```

`memo` may allocate and write a runtime cache cell, but that cache mutation is not an
Ash-visible row effect unless a future feature exposes it as one. A handler-marked function is
pure when applying it leaves an empty residual row; impurity comes from residual effects in the
handler body, not from the marker.

Contracts fire at observation boundaries:

| Mode | Contract timing |
|------|-----------------|
| `strict` | check at call, return, or ordinary data boundary |
| `lazy` | check on every force |
| `memo` | check on first force, cache the terminal outcome, and replay success/failure/trap thereafter |

For memoized contract failures, replay preserves the original diagnostic and blame label. A
later force may record a replay event, but it must not create a new blame event.

Predicate evaluation must not introduce an implicit force. A predicate may inspect a strict
value that is already present at the boundary, or the strict result produced by a force owned by
that boundary. It must not force a lazy or memo value merely to decide whether a contract holds.

### 15.8 CPS Lowering

In the CPS IR (SPEC-098b), modes become calling conventions:

- **strict parameter**: pass value directly
- **lazy parameter**: pass a `ThunkClosure` whose body is a zero-argument CPS lambda and whose value stores the creation-time handler/provider chain
- **memo parameter**: pass a `ThunkClosure` whose body is a zero-argument CPS lambda and whose value stores the creation-time handler/provider chain plus a memo cell

The mode is visible in the IR as the presence or absence of thunk wrapping. For effectful
thunks, the wrapper is semantically required: a bare zero-argument `Lam` does not preserve
the creation-time authority boundary specified by SPEC-101.

### 15.9 Algorithmic Implications

Some algorithms are naturally expressed in one mode:

```ash
-- Left fold: naturally strict, tail-recursive
fn foldl<A, B>(f: B -> A -> B, acc: B, xs: List<A>) -> B { ... }

-- Right fold: naturally lazy, builds thunk chain
fn foldr<A, B>(f: A -> B -> B, base: B, xs: List<A>) -> lazy B { ... }

-- Stream processing: lazy by design
fn map<A, B>(f: A -> B, lazy xs: List<A>) -> lazy List<B> { ... }
```

Mode mismatch is a type error, not a performance warning. The user must explicitly convert.

## 16. Changelog

- 2026-06-18: Created as target-state type system document. Defined row semantics, effect item taxonomy, discharge rules, alias/group behavior, and acceptance criteria.
- 2026-06-20: Added §15 Evaluation Modes. Defined strict/lazy/memo as invariant algorithmic contracts with explicit `_unsafe` conversions.
- 2026-06-21: Clarified memo force row accounting so static force sites retain the thunk latent row while dynamic cache hits may perform no effects, and aligned CPS lowering text with SPEC-101 `ThunkClosure` chain-capture semantics.
- 2026-06-27: Reconciled with NOTE-021 (Row kind, computation row terminology), NOTE-022 (operations as interface methods), NOTE-023 (handler typing: continuation as ordinary parameter, multiplicity via function type).
- 2026-06-27: Reconciled with NOTE-025 (effect identity via sorts and impls). OperationEffect identity changed from interface-qualified to impl-type-qualified. Handler typing examples updated. §3.3 and §8.1 revised.
- 2026-06-28: Reconciled with NOTE-026 through NOTE-029. Added §3.9 newtype identity/representation semantics, expanded §6.5 with Hoare contract subsumption and blame polarity, and added §15.7 denotational purity plus lazy/memo contract timing.
- 2026-06-28: Reconciled with NOTE-030. Added §6.6 contract composition through sequencing: rows compose by union, producer postconditions discharge continuation preconditions via `∀a. Q(a) ⇒ R(a)`, and composed postconditions existentially thread the intermediate value.
- 2026-06-29: Reconciled with NOTE-031. Predicate references now carry checked summaries and boundary-local snapshot metadata; predicates are classified as static or dynamic after rejecting effectful, unstable, or implicit-forcing forms.
- 2026-06-29: Reconciled with NOTE-033. Refined `PredicateSummary` so predicate references point at typed lowered predicate artifacts with binder environments, snapshots, proof fragments, optional dynamic-check plans, diagnostics, and stable identities.
- 2026-06-29: Swept stale contract-discharge wording so dynamic contracts lower to runtime checks by default rather than requiring hidden runtime contract handlers.
- 2026-06-29: Reconciled with NOTE-034. Predicate well-formedness now explicitly separates authority-bearing operation calls from ordinary operation-produced values that contracts may inspect.
- 2026-06-29: Reconciled with NOTE-035. Added trace-contract well-formedness over `Γtrace`, with `Proc`/`Workflow` treated as semantic anchors classified by referenced facts rather than separate contract mechanisms.
