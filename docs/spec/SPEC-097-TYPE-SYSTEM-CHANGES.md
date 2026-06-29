---
id: spec.ash.type-system-changes
title: Type System Changes for Unified Effect System
kind: spec
audience: [human, agent]
authority: design
status: draft
stability: alpha
owner: language
last_verified: 2026-06-18
verified_against:
  git_commit: e61f2792
  specs:
    - docs/spec/SPEC-096-UNIFIED-EFFECT-SYSTEM.md
  code:
    - crates/ash-core/src/ast.rs
    - crates/ash-core/src/effect.rs
    - crates/ash-typeck/src/
---

# SPEC-097: Type System Changes for Unified Effect System

## 1. Summary

This spec defines the type-system obligations for the unified effect-row direction in SPEC-096. The central rule is:

```text
a computation's requirement row must be discharged by the ambient effect environment
```

Rows are requirement sets, not authority grants. The type checker may infer, normalize, compare, and report rows, but authority is provided only by admitted roles, effect providers, resources, policy handlers, channel endpoints, contract evidence, or runtime/workflow boundaries.

This draft replaces the earlier loose statement “`{fs} <: {fs, log}`” with explicit relations:

1. requirement inclusion: what a computation requires;
2. environment discharge: what the context provides or proves;
3. function subtyping: when one callable can stand in for another.

## 2. Current type-system baseline

### 2.1 Existing types

Current parser/core/typechecker code does not yet carry a complete effect-row type in ordinary function signatures. The live type surfaces include named types, type constructors, tuples, records, associated types, and callable types, while workflow/effect information is tracked separately in existing workflow and legacy capability machinery.

The existing 4-point `Effect` lattice in `crates/ash-core/src/effect.rs` is a coarse workflow/effect classification:

```rust
pub enum Effect {
    Epistemic = 0,
    Deliberative = 1,
    Evaluative = 2,
    Operational = 3,
}
```

That lattice is not a row system. The row system introduced here must preserve compatibility with existing effect classifications during migration rather than pretending the old representation already has row structure.

### 2.2 Migration constraint

The first implementation slice should add row summaries and row checking around existing carriers. It must not require immediate deletion of `Type::Fn`, `Type::Fun`, `Act<T>`, `Proc<T>`, `Workflow<T>`, workflow headers, or current legacy capability declarations.

## 3. Type-level representation

### 3.1 Row carrier

A conforming implementation needs a shared row carrier. The exact Rust home is an implementation decision, but the semantic shape is:

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

A closed row has `tail: None`. An open row has `tail: Some(r)` and represents the listed requirements plus an unknown remainder.

### 3.2 Effect item identity

Rows contain typed effect items, not bare strings.

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

Every effect item must have a canonical identity used for duplicate elimination, row comparison, diagnostics, and module-summary export. The identity must include its namespace. For example, `fs.read`, `policy fs.read`, and `role fs.read` are distinct even if their textual tails match.

### 3.3 Operation effect

```rust
pub struct OperationEffect {
    pub interface: NamePath,
    pub operation: Option<Name>,
}
```

A whole-interface requirement such as `fs` is broader than an operation-specific requirement such as `fs.read`. A future implementation must define whether `fs` expands into all known operations or remains an abstract interface requirement. It must not silently treat the two as identical.

### 3.4 Role effect

```rust
pub struct RoleEffect {
    pub role: NamePath,
}
```

A role effect requires role admission. It does not by itself expand into operations until the role definition and admission context are known.

### 3.5 Policy effect

```rust
pub struct PolicyEffect {
    pub binding: NamePath,
    pub decision_domain: Option<PolicyDecisionDomain>,
}
```

Policy effects reference named policy bindings, following SPEC-006/SPEC-007. Anonymous policy expressions are out of scope for this spec.

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

Predicate references must preserve source scope, binder information, and discharge status. A predicate printed in a row is not enough; the type checker must know which names are legal inside it.

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

Channel guards are contracts over communication. The guard's message binder must be explicit in the typed representation, even if the surface syntax later chooses a concise spelling such as `message`. `close` has no message type and cannot carry a message guard in this draft.

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

These effects are profile-sensitive. `proc spawn` must not type-check in a pure or `Act`-only profile unless the computation is explicitly lifted or checked in a `Proc`-capable environment.

## 4. Row syntax and kinds

### 4.1 Proposed source syntax

SPEC-095 remains the current parser-derived grammar. The target row syntax for this type-system packet is:

```ash
{}                                      -- empty row
{fs.read}                                -- closed row
{fs.read, policy production_rate}         -- multiple requirements
{fs.read | r}                             -- open row
{r}                                      -- whole-row variable
{IO}                                     -- transparent alias or group reference
```

Rows are not ordinary record types. A parser/typechecker implementation must distinguish record type `{x: Int}` from effect row `{fs.read}` by grammar context or an explicit row-introducing token chosen by the syntax spec. It must also distinguish `{r}` as a row variable from `{IO}` as an alias/group reference by kind and namespace resolution.

### 4.2 Row variable kind

Row variables have a distinct kind, for example `EffectRow` or `Effect`. This spec uses `EffectRow` in prose to avoid confusion with the existing 4-point `Effect` lattice.

```ash
fn map<A, B, r: EffectRow>(xs: List<A>, f: A -> {r} B) -> {r} List<B> { ... }
```

The implementation may infer row-variable kinds when unambiguous.

### 4.3 Row constraints

A row variable may carry constraints:

```ash
fn log_and_return<A, r>(x: A) -> {log.write | r} A { ... }
```

This means the resulting computation requires at least `log.write` plus whatever `r` requires. It does not mean `r <: {log.write}`.

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
fs           != fs.read, unless a later effect-interface rule defines expansion
role admin   != fs.read, even if admin can entail fs.read
```

Role entailment happens during discharge, not normalization.

## 6. Requirement inclusion, discharge, and subtyping

### 6.1 Requirement inclusion

`Requires(A) ⊆ Requires(B)` means every requirement in `A` also appears in `B` after normalization and alias expansion.

This relation is useful for comparing two requirement rows, but it is not the same as checking whether a program may run. Running a program needs environment discharge.

### 6.2 Environment discharge

`Env ⊢ R discharged` means the ambient environment discharges every item in requirement row `R`.

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

The type checker may statically reject missing discharge, or it may produce a residual row that must be discharged by a later admission/runtime boundary. The chosen phase boundary must be explicit in implementation tasks.

### 6.3 Function subtyping

Function subtyping is contravariant in requirements: a function that requires fewer effects may be used where a function requiring more effects is expected.

```text
Requires(f_actual) ⊆ Requires(f_expected)
------------------------------------------------
(A -{f_actual}-> B) <: (A -{f_expected}-> B)
```

Example:

```text
(A -{fs.read}-> B) <: (A -{fs.read, log.write}-> B)
```

The reverse is not valid. A function that requires logging cannot be used where only file-read authority was expected.

### 6.4 Empty row

The empty row is the least requirement row:

```text
Requires({}) ⊆ Requires(R)
```

This means pure functions can be used in effectful contexts. It does not mean an effectful context is pure.

### 6.5 Contract subsumption

A contract item may be removed from a residual row only when the checker records a discharge:

```text
prove(p) or evidence(p) or runtime_handler(requires p)
------------------------------------------------------
requires {p} is discharged
```

The type checker must not silently coerce `{requires {p}}` to `{}` without recording the mode and evidence boundary.

## 7. Row polymorphism

### 7.1 Higher-order functions

A higher-order function can preserve the row of a callback.

```ash
fn map<A, B, r: EffectRow>(xs: List<A>, f: A -> {r} B) -> {r} List<B> { ... }
```

The result row is whatever the callback requires, plus any requirements from `map` itself. If `map` logs internally, the row must include that requirement:

```ash
fn map_logged<A, B, r: EffectRow>(xs: List<A>, f: A -> {r} B) -> {log.write | r} List<B> { ... }
```

### 7.2 Inference

Effect inference may infer closed rows for ordinary functions:

```ash
fn add(a: Int, b: Int) -> Int { a + b }        -- inferred requirement row {}
fn read(path: String) -> String { fs.read(path) } -- inferred row includes fs.read
```

Whether the inferred row appears in the surface type, module summary, or diagnostics is an implementation detail. The semantic row must be available to checking and export/import if the function is public.

### 7.3 Open-row solving

Open-row solving must avoid accidental privilege loss or gain.

For a call requiring `{fs.read | r}` in an environment containing `{fs.read, log.write}`, the solver may instantiate `r` with `{log.write}` if the expected type demands the larger row. It must not infer that `log.write` is required unless it is used, expected, or otherwise constrained.

### 7.4 No implicit tower lifts

Row polymorphism does not add implicit lifts across `Pure`, `Act`, `Proc`, and `Workflow` profiles. If a `Proc` computation binds an `Act` computation, the explicit lift or embedding rule remains required until a separate spec changes it.

## 8. Type checking rules by effect kind

### 8.1 Operation calls

An effect operation call contributes an operation row item. Current capability declarations
migrate to effect operation surfaces, but the target row item is the operation identity.

```text
call C.op(args) : A
------------------------
row includes C.op
```

If the operation is called through a binding name, the effect identity must resolve through the binding to the effect interface/operation identity. Diagnostics should show both the binding and canonical operation when helpful.

### 8.2 Role use

A function or computation requiring a role includes a role effect. The role effect can also discharge entailed operation effects, but only when the role is admitted in the ambient environment.

The type checker must not expand role definitions at declaration sites in a way that loses the role identity. Role identity matters for audit and diagnostics.

### 8.3 Policy use

Policy effects must resolve to named policy bindings. The type checker must reject or defer:

- unknown policy names;
- anonymous inline policy expressions in row position;
- policy effects used in a decision domain incompatible with the consumer;
- policy effects that cannot be exported because their named binding is private.

### 8.4 Contracts and predicates

Contract predicates must be checked for well-scoped names and purity/effect safety according to their contract kind.

- `requires` may mention parameters and earlier lexical bindings.
- `ensures` may mention parameters and `result`.
- `guard` may mention the channel message binder plus allowed lexical bindings.
- `law` may mention its law parameters and any names permitted by the law/proof specs.

A contract predicate that itself requires effects must be rejected unless a later spec admits effectful predicates explicitly.

### 8.5 Channels

A send operation contributes a `channel send` effect. A receive operation contributes a `channel receive` effect. A guarded receive also contributes or embeds a guard contract.

The type checker must verify at least:

1. endpoint name resolution;
2. direction compatibility;
3. message type compatibility;
4. guard predicate well-scoping;
5. profile compatibility (`Proc` or `Workflow`, not `Pure`/plain `Act`).

Full protocol/session compatibility is a future spec.

### 8.6 Failure

A `fail` expression contributes a failure effect. It must not type-check by converting the failure into a domain result unless the source syntax explicitly constructs that domain result.

### 8.7 Evidence/reporting

Evidence and report effects must be checked against available evidence sinks or workflow/reporting boundaries. Public functions exporting evidence effects must preserve those effects in module summaries.

## 9. Effect aliases and groups

### 9.1 Transparent aliases

Transparent aliases expand during normalization.

```ash
effect alias IO = {fs.read, fs.write, log.write};
```

Alias expansion must be cycle-checked. Cycles are rejected.

### 9.2 Diagnostic groups

Groups preserve a name for diagnostics while expanding to concrete row items.

```ash
effect group WorkflowIO = {
    fs.read,
    log.write,
    evidence audit_log,
};
```

A missing requirement diagnostic should be allowed to say:

```text
missing WorkflowIO (specifically log.write)
```

### 9.3 Authority bundles are different

An authority bundle, if added, is not a transparent alias. It must have an admission rule and provenance. The type checker must not treat `effect alias Admin = {fs.write}` as granting write authority.

### 9.4 Export/import

Public aliases/groups used in public function rows must be exported in module summaries. Private aliases in public rows must either be expanded before export or rejected with a diagnostic that preserves opacity rules.

## 10. Integration with existing features

### 10.1 Generics

Rows participate in generic signatures through row variables. Row variables are separate from ordinary type variables.

### 10.2 Interfaces

Interface methods may carry effect rows, but the parser and typechecker must respect the live interface method syntax. If method signatures currently use positional parameter types, a syntax task must update that grammar before examples with named parameters become normative.

Example target shape:

```ash
interface EffectfulMap<F> {
    map<A, B, r: EffectRow>(F<A>, A -> {r} B) -> {r} F<B>;
}
```

### 10.3 Associated types and families

Associated types may mention row-bearing callable types only after the row carrier is available in the canonical type-expression IR and module-summary format. Until then, such examples are design sketches, not implementation-ready syntax.

### 10.4 Closures and capture safety

Closure capture remains governed by effect-safe capture rules. A closure's row includes the effects required by its body, but capture permission is separately constrained by the creation context. A pure closure must not become effectful by silently capturing an effectful value.

## 11. Diagnostics

A conforming implementation must provide diagnostics that name the missing item kind and likely fix. Examples:

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

Generic `RowMismatch` is allowed only as an internal error category. User-facing diagnostics must classify the row item kind.

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
- Reject process/channel/governance effects in lower profiles unless an explicit lift or boundary handles them.

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

- [SPEC-096: Unified Effect System](SPEC-096-UNIFIED-EFFECT-SYSTEM.md)
- [SPEC-098: IR Changes for Unified Effect System](SPEC-098-IR-CHANGES.md)
- [SPEC-099: Operational Semantics](SPEC-099-OPERATIONAL-SEMANTICS.md)
- [SPEC-006: Policy Definition Syntax](SPEC-006-POLICY-DEFINITIONS.md)
- [SPEC-019: Role Runtime Semantics](SPEC-019-ROLE-RUNTIME-SEMANTICS.md)
- [SPEC-052: Capability Interfaces and Implementations](SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md)

## 15. Changelog

- 2026-06-18: Tightened row semantics, effect item taxonomy, discharge rules, alias/group behavior, and acceptance criteria.
- 2026-06-17: Initial draft.
- 2026-06-29: Swept stale contract-discharge wording so dynamic contracts lower to runtime checks by default rather than requiring hidden runtime contract handlers.
