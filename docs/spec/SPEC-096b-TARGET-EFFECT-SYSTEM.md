---
id: spec.ash.effect-system.target
title: Ash Effect System — Target State
description: Unified effect system based on row polymorphism with kind-specific discharge for operations, roles, policies, contracts, channels, process operations, failure, and evidence
kind: spec
audience: [human, agent]
authority: design
status: draft
stability: alpha
owner: language
last_verified: 2026-06-18
verified_against:
  specs:
    - docs/spec/SPEC-095a-CURRENT-GRAMMAR.md
    - docs/spec/SPEC-095b-TARGET-GRAMMAR.md
    - docs/spec/SPEC-096a-CURRENT-EFFECT-SYSTEM.md
    - docs/spec/SPEC-097a-CURRENT-TYPE-SYSTEM.md
    - docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md
    - docs/spec/SPEC-006-POLICY-DEFINITIONS.md
    - docs/spec/SPEC-007-POLICY-COMBINATORS.md
    - docs/spec/SPEC-019-ROLE-RUNTIME-SEMANTICS.md
    - docs/spec/SPEC-024-CAPABILITY-ROLE-REDUCED.md
    - docs/spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md
---

# SPEC-096b: Ash Effect System — Target State

**Status:** Draft — target semantic model for unified computation rows
**Scope:** This document defines the effect-system semantics we want Ash to have.
It is a goal-state living document that will be refined as implementation progresses.
**Depends on:** SPEC-095b (Target Grammar), SPEC-097b (Target Type System)

## 1. Summary

Ash should move toward one effect-accounting model based on row polymorphism. Computation rows
describe the requirements of a computation: which operations, roles, policies, contracts,
channels, process operations, failures, and evidence/reporting effects the computation may
use or emit.

This spec does **not** immediately delete the public `Act`, `Proc`, and `Workflow` strata.
In the first normative slice, those strata become named row profiles over a shared
effect-accounting substrate:

```text
Pure      = computation with an empty requirement row
Act       = operation/resource/failure/evidence effects
Proc      = Act + process/channel/concurrency effects
Workflow  = Proc + contract/policy/role/obligation/reporting effects
```

A later spec may choose to collapse the public computation types into one runtime
representation. This spec only requires that their authority and contract requirements
become visible as computation rows.

## 2. Motivation

Ash currently has several related but separate mechanisms:

- the four-stratum tower `Pure < Act < Proc < Workflow`;
- current capability declarations and bindings as migration input;
- role authority and obligations;
- policy declarations and policy decisions;
- workflow contracts such as `requires` and `ensures`;
- process and channel operations such as `spawn`, `send`, and `receive`;
- operational failure and audit/reporting behavior.

These mechanisms all answer the same broad question: what must be available, checked,
discharged, or recorded for this computation to run? Keeping them separate makes it hard
to write reusable code and hard to explain why a computation is accepted in one context
but rejected in another.

Effect rows provide a common accounting layer. They do not erase semantic differences.
A policy effect, a role effect, and a channel receive effect all appear in rows, but each
has its own discharge rule.

> **Terminology (NOTE-021, NOTE-020):** The broad type-level row concept that includes
> operations, resources, roles, policies, contracts, channels, process, failure, and
> evidence is a *computation row*. The term *effect row* is reserved for the syntactic
> `{...}` between arrow and return type.

## 3. Non-goals for this draft

This draft is a precision pass over the language direction. It intentionally does not specify:

1. ~~arbitrary user-defined resumable algebraic effects~~ — **Updated (NOTE-022):** No longer a
   non-goal. Operations are declared as interface methods, and user-defined effects are now part
   of the target model. See §6.1.
2. a complete syntax for effect handlers — the handler surface is captured in NOTE-023; this
   spec defers the full syntactic grammar to that note;
3. deletion of `Act<T>`, `Proc<T>`, `Workflow<T>`, or `workflow` syntax;
4. a new runtime `Eff<A>` implementation;
5. implicit privilege grants from aliases or groups;
6. session-type or MPST protocol checking for channels.

Those may be future specs. This spec only defines the row vocabulary and the semantic
responsibilities that later parser, type-system, IR, and runtime specs must refine.

## 4. Core semantic model

### 4.1 Rows describe requirements, not grants

A computation row on a function or computation is a set of requirements. It says what the body
may need. It does not grant authority by itself.

```ash
fn read_config(path: String) -> {PosixFs::read} String { ... }
```

The row above means the computation requires authority to perform `PosixFs::read`. The caller or
admission context must provide or discharge that requirement. Merely naming `PosixFs::read` in
a type must not create a file-system authority.

### 4.2 Ambient effect environment

A checking context has an ambient effect environment. The environment contains requirements
that are already available or already discharged in the current scope.

Examples of ambient facts include:

- a role admitted at workflow/process start;
- an operation provider or legacy capability binding admitted by a workflow header;
- a policy handler installed around a region;
- a proof or runtime-check strategy for a contract;
- a channel endpoint owned by the current process;
- an evidence sink available at the workflow boundary.

A computation with requirement row `R` may run in environment `E` only if every required item
in `R` is discharged by `E` according to that item kind's rule.

### 4.3 Row profiles preserve the tower

The tower remains the default explanation of increasing operational power. Row profiles
make that power explicit by naming increasingly rich computation rows.

| Profile | Row shape | Meaning |
|---------|-----------|---------|
| `Pure` | `{}` | no effect requirements |
| `Act` | operation/resource/failure/evidence effects | runtime-managed effects without process or workflow governance |
| `Proc` | `Act` effects plus process/channel effects | process identity, concurrency, message passing, observation of process failure |
| `Workflow` | `Proc` effects plus contract/policy/role/obligation/reporting effects | governed orchestration boundary |

The profiles are not privileges. They are named constraints over rows. For example, an `Act`
computation cannot use `channel receive` unless it is lifted or rechecked in a `Proc` profile.

### 4.4 Effect discharge is kind-specific

All row items share row syntax, but they are not discharged uniformly.

> **NOTE-020:** Computation rows include more than effects — resources, roles, policies,
> contracts, channels, process operations, failure modes, and evidence are all first-class
> row items with their own kinds and discharge rules.

| Row item kind | Discharge mechanism |
|-------------|---------------------|
| operation | admitted provider/effect binding or role entailment |
| resource | ownership, borrow, split, join, or provenance over a runtime resource |
| role | role admission at the execution boundary |
| policy | named policy binding evaluated or handled by a compatible decision domain |
| contract | static proof, evidence proof/test, dynamic runtime check, or explicit recoverable `fail` path |
| channel | owned endpoint with compatible direction/message type and guard behavior |
| process | process runtime operation such as spawn/await/join/cancel |
| failure | enclosing tower/profile supports the failure route and handler/reporting policy |
| evidence/provenance | records audit, proof, test, report, or trace evidence |

A later type-system spec must define separate diagnostics for each failed discharge rule.
A generic `UnhandledEffect` error is insufficient for user-facing feedback.

## 5. Effect row syntax

The syntax below is proposed. SPEC-095a remains the current parser-derived grammar; this
section is the target grammar for the language-evolution packet.

```ebnf
effect_row = "{" [ row_contents ] "}" ;

row_contents = row_variable
             | effect_item { "," effect_item } [ "," ] [ "|" row_variable ]
             ;

row_variable = identifier ;

effect_item = operation_effect
            | resource_effect
            | role_effect
            | policy_effect
            | contract_effect
            | channel_effect
            | process_effect
            | failure_effect
            | evidence_effect
            | effect_group_ref
            ;
```

Open rows use a row variable after `|`. A whole-row variable is written as the only item
in the row:

```ash
fn map<A, B, r>(xs: List<A>, f: A -> {r} B) -> {r} List<B> { ... }
```

`{r}` is a complete row variable. `{PosixFs::read | r}` is a row extension with a tail variable.
The parser/type checker must distinguish `{r}` from an effect group reference by kind and
namespace resolution.

A closed row has no row variable:

```ash
fn read_file(path: String) -> {PosixFs::read} String { ... }
```

## 6. Effect item taxonomy

### 6.1 Operation effects

Operation effects require authority to call an admitted effect operation. Operation signatures
are declared as interface methods (NOTE-022). The operation identity in the row is
impl-type-qualified (NOTE-025): in generic code it is abstract (`F::read` where `F: Fs`);
after monomorphization it is concrete (`PosixFs::read`).

```ebnf
operation_effect = impl_type_ref "::" operation_name ;
impl_type_ref = identifier { "::" identifier } ;
operation_name = identifier ;
```

Examples:

```ash
fn read_config<F: Fs>(path: String) -> {F::read} String { ... }
-- After specialization with F = PosixFs: {PosixFs::read}
```

An operation effect is discharged by an explicit provider/effect binding, by a role that
entails the operation, or by a host/runtime admission fact. The type checker must not treat
an operation name as an ordinary value binding.

### 6.2 Resource effects

Resource effects require access to a runtime resource. They are distinct from operation
effects: an operation describes the requested effect surface, while a resource describes owned
or borrowed state used by an implementation.

```ebnf
resource_effect = "resource" resource_path [ resource_mode ] ;
resource_mode = "own" | "read" | "write" | "split" | "join" ;
```

Examples:

```ash
fn compact() -> {resource db write} Unit { ... }
fn snapshot() -> {resource db read} Snapshot { ... }
```

The exact resource split/join semantics remain owned by resource/runtime specs. This spec
requires only that resource requirements be representable in rows.

### 6.3 Role effects

A role effect requires execution under an admitted role.

```ebnf
role_effect = "role" role_path ;
```

Example:

```ash
fn approve(req: TransferRequest) -> {role manager, approve_transfer} Approval { ... }
```

Roles are not aliases for operations. A role can entail operation effects, policy effects,
or obligations, but that entailment must be checked from the role definition and admission
context.

### 6.4 Policy effects

A policy effect requires evaluation of a named policy binding.

```ebnf
policy_effect = "policy" policy_path ;
```

Example:

```ash
policy production_rate = RateLimit { requests: 1000, window_secs: 60 };

fn call_api(req: Request) -> {http.get, policy production_rate} Response { ... }
```

Policy effects preserve the SPEC-006/SPEC-007 boundary: policies are named declarations
and lowered policy programs, not arbitrary runtime `Policy` values. A future spec may add
first-class or anonymous policies, but this draft does not.

### 6.5 Contract effects

Contract effects describe predicates, obligations, or laws that must be proved, checked, or
recorded.

```ebnf
contract_effect = requires_effect
                | ensures_effect
                | invariant_effect
                | law_effect
                | obligation_effect
                | guard_effect
                ;

requires_effect = "requires" "{" predicate "}" ;
ensures_effect = "ensures" "{" predicate "}" ;
invariant_effect = "invariant" "{" predicate "}" ;
law_effect = "law" identifier "{" predicate "}" ;
obligation_effect = "obligation" obligation_path ;
guard_effect = "guard" "{" predicate "}" ;

predicate          = predicate_or ;
predicate_or       = predicate_and { "||" predicate_and } ;
predicate_and      = predicate_not { "&&" predicate_not } ;
predicate_not      = [ "!" ] predicate_cmp ;
predicate_cmp      = predicate_add [ cmp_op predicate_add ] ;
predicate_add      = predicate_mul { ("+" | "-") predicate_mul } ;
predicate_mul      = predicate_unary { ("*" | "/" | "%") predicate_unary } ;
predicate_unary    = literal
                   | identifier
                   | "result"
                   | "message"
                   | "old" "(" snapshot_expr ")"
                   | predicate_call
                   | field_projection
                   | tuple_projection
                   | "(" predicate ")"
                   ;
predicate_call     = predicate_function "(" [ predicate_args ] ")" ;
predicate_function = identifier | qualified_identifier ;
predicate_args     = predicate { "," predicate } ;
snapshot_expr      = identifier { "." identifier } ;
cmp_op             = "==" | "!=" | "<" | "<=" | ">" | ">=" ;
```

Examples:

```ash
fn divide(a: Int, b: Int) -> {requires {b != 0}} Int { ... }
fn binary_search(xs: List<Int>, target: Int) -> {requires {sorted(xs)}, ensures {result >= -1}} Int { ... }
```

Contract predicates must define their scope. For `ensures`, `result` is bound to the normal
result value. For channel guards, the received message binder must be named by the channel
operation or by a later syntax spec. `old(snapshot_expr)` names a boundary-local pre-state
snapshot; `snapshot_expr` is a field path through a boundary value, not an arbitrary
computation.

Per NOTE-031, the predicate grammar is a contract-position boundary over expression-like
syntax. Before lowering, the type checker classifies each predicate as SMT-safe static, pure
dynamic, or rejected. Rejected predicate forms include capability calls, process/workflow
operations, handler dispatch, time/randomness/environment observation, and implicit forcing of
lazy or memo values outside a contract-owned observation boundary. Unsupported but pure
predicates are dynamic rather than silently erased.

Per NOTE-033, this grammar is only the surface entry point. Every accepted contract predicate
must lower to a structured predicate artifact before it becomes a `PredicateRef`. That lowered
artifact records the owning boundary, typed binder environment, boundary-local `SnapshotRef`s,
admitted predicate-function identities, static/dynamic classification, proof fragment or
dynamic runtime-check plan, diagnostic shape, and stable predicate identity. The implementation
must not treat a source string as the predicate's semantic representation.

### 6.6 Channel effects

Channel effects require authority over a typed channel endpoint.

```ebnf
channel_effect = channel_message_effect
               | channel_close_effect
               ;

channel_message_effect = "channel" channel_message_mode channel_path message_type [ channel_guard ] ;
channel_message_mode = "send" | "receive" | "select" ;

channel_close_effect = "channel" "close" channel_path ;

channel_guard = "where" "{" predicate "}" ;
```

Examples:

```ash
fn send_order(o: Order) -> {channel send orders Order} Unit { ... }
fn worker() -> {channel receive orders Order where {message.amount <= 1000}} Unit { ... }
```

Channel guards are contract effects over communication boundaries. A future channel spec must
choose the runtime behavior for guard failure: keep waiting, reject the message, route to
a dead-letter channel, or raise operational failure. This spec only requires that the guard
appear in the effect row and not be erased into a plain boolean.

### 6.7 Process effects

Process effects require process-runtime operations.

```ebnf
process_effect = "proc" process_operation ;
process_operation = "spawn" | "await" | "join" | "cancel" | "yield" | identifier ;
```

Examples:

```ash
fn start_worker() -> {proc spawn} P<Unit> { ... }
fn wait_worker(p: P<A>) -> {proc await} A { ... }
```

Process effects imply a `Proc`-capable profile. They must not be silently accepted in pure or
`Act`-only contexts.

### 6.8 Failure effects

Failure effects represent tower-scoped operational failure, not domain failure such as
`Result<A, E>`.

```ebnf
failure_effect = "fail" [ failure_path ] ;
```

Example:

```ash
fn parse_config(path: String) -> {PosixFs::read, fail ConfigError} Config { ... }
```

A `fail` effect is discharged by an enclosing failure handler, workflow failure boundary, or
profile-specific failure policy. It must not implicitly become `None`, `Err`, an empty list,
or another domain value.

Per NOTE-029, default dynamic contract failure is not a `fail` effect: it is structured bottom
(`Trap { reason: ContractViolation(ContractDiagnostic) }`) outside ordinary row accounting. If
a surface construct chooses recoverable contract behavior, the lowering must map the
diagnostic into an explicit `fail` operation and include that failure item in the row.

### 6.9 Evidence and report effects

Evidence effects require an audit/provenance/reporting sink.

```ebnf
evidence_effect = "evidence" evidence_path
                | "report" report_path
                ;
```

Examples:

```ash
fn audited_write(msg: String) -> {StdoutLog::write, evidence audit_log} Unit { ... }
fn finish() -> {report workflow_summary} Unit { ... }
```

These effects are normally discharged by workflow or runtime admission. A pure function cannot
require evidence/report effects directly.

## 7. Roles, effects, and authority

### 7.1 Role admission

A role effect requires that the execution boundary has admitted that role. Admission is a
runtime or workflow/process boundary decision, not a local type alias.

```ash
role manager {
    entails approve_transfer
}
```

A computation requiring `{role manager}` may run only in a context where `manager` is admitted.

### 7.2 Role-to-operation entailment

A role can entail operation effects. The entailment is derived from the role declaration and
any admitted refinements.

```text
admitted(role manager)
role manager entails approve_transfer
----------------------------------------
approve_transfer is discharged
```

The entailment must be explicit and auditable. If a role definition changes, downstream
row-discharge evidence must be invalidated or rechecked.

### 7.3 Authority denial

Authority denial is not the same as a policy denial. If no admitted role or provider/effect
binding discharges an operation effect, the computation is rejected before the operation runs.

A later operational-semantics spec must distinguish at least:

- missing authority;
- policy denial;
- contract violation;
- channel guard failure;
- ordinary operational failure.

## 8. Policies as handled effects

Policies are represented in rows as named policy effects. They are handled by policy evaluators
at explicit boundaries.

```ash
fn send_invoice(inv: Invoice) -> {policy invoice_policy, email.send} Unit { ... }
```

The policy effect means:

1. resolve `invoice_policy` as a named policy binding;
2. lower or reference its canonical policy program;
3. evaluate it at the appropriate admission or action boundary;
4. interpret the decision in the consumer-specific decision domain;
5. record decision evidence where required.

A policy handler is therefore a structured runtime/admission component, not necessarily a
user-written resumable algebraic-effect handler.

### 8.1 General handler semantics (NOTE-023)

Beyond policy handlers, the target model includes a uniform handler surface for any
computation-row item. Per NOTE-023:

- A **handler** is an ordinary function that consumes a computation thunk (a deferred
  computation with its row) and produces a value.
- The `on` eliminator installs a *Handle frame* on the computation stack, binding the
  handler to one or more row items in the thunk's row.
- The **continuation** exposed to the handler body is an ordinary typed parameter — it is
  not a special-purpose construct — so handlers compose using normal function application
  and row-subsumption rules.
- **Multiplicity** of the continuation (linear, affine, or unrestricted) is derived from
  the handler's function type, not from a separate annotation. This lets the type system
  reject non-affine resumption where the row item kind forbids it.

This recovers resumable algebraic-effect semantics without introducing a distinct
handler sublanguage: handler definitions are interface implementations (NOTE-022), and
their typing reuses the existing function and row machinery.

## 9. Contracts, guards, and discharge

### 9.1 Discharge modes

A contract effect can be discharged by:

| Mode | Meaning |
|------|---------|
| static | the type checker/prover proves the predicate |
| evidence | a proof, test, law result, or checked artifact establishes the predicate under policy |
| dynamic | a runtime handler checks the predicate at the appropriate boundary |

Each contract effect must record which mode discharged it. The compiler must not silently erase
a contract because a runtime handler might exist.

### 9.2 Channel guards as contracts

A channel guard is a contract attached to a receive/select/send boundary. It differs from an
ordinary `if` condition because it controls communication admission.

```text
channel receive orders Order where {message.amount <= limit}
```

The guard predicate may mention the message binder and any lexical values allowed by ordinary
closure/effect-safety rules. A future channel spec must define whether failed guards consume
or preserve the message.

### 9.3 Laws as contract effects

A law effect names a law and its predicate. Law discharge may use proof evidence, property
evidence, small-world evidence, or a later formal proof system. This spec only requires law
effects to be representable in rows and to preserve their evidence status.

## 10. Effect aliases, groups, and authority bundles

Ash needs grouping mechanisms, but they must not blur requirements and grants.

> **NOTE-021:** The terms *effect alias* and *effect group* are retained for row-level
> grouping, but they are *computation-row* aliases/groups. The kind name for such rows is
> `Row`. An `effect alias` is therefore a `Row`-kinded alias over a computation row, not a
> distinct type constructor.

### 10.1 Transparent aliases

A transparent alias is a pure abbreviation. It expands before row checking and grants no authority.

```ash
effect alias IO = {PosixFs::read, PosixFs::write, StdoutLog::write};

fn load() -> IO Config { ... }
```

### 10.2 Diagnostic groups and profiles

An effect group/profile preserves a human-readable name for diagnostics and documentation while
still expanding to row items for checking.

```ash
effect group WorkflowIO = {
    PosixFs::read,
    StdoutLog::write,
    evidence audit_log,
};
```

If `WorkflowIO` is missing from an environment, diagnostics should explain both the group name
and the concrete missing item.

### 10.3 Authority bundles are not aliases

An admission package may grant or entail authority, but it is not an effect alias. It must
have an admission rule and an audit/provenance boundary.

```text
effect alias/group = describes requirements
role/effect admission = grants or entails authority
```

A future spec may define authority bundles, but it must keep them separate from transparent
effect aliases.

## 11. Unified `do` direction

The target surface is a single `do { ... }` form whose effect requirements are inferred from
the body and checked against the enclosing row.

```ash
fn read_config(path: String) -> {PosixFs::read} String {
    do {
        contents <- PosixFs::read(path);
        return contents
    }
}
```

During migration, existing `do:Act`, `do:Proc`, `do:Workflow`, `act { ... }`, and `workflow`
syntax remain compatibility surfaces. They should lower into the same row-checked semantic
representation once the parser, type checker, and runtime carriers exist.

A later syntax spec must decide whether explicit target annotations remain useful as row
profiles, for example `do:Proc { ... }` meaning "check this block against the `Proc` row profile."

## 12. Migration path

### 12.1 Stage 1: Row summaries

Add effect-row summaries to checked functions, closures, and computation blocks without changing
the public runtime representation.

- Preserve `Act<T>`, `Proc<T>`, and `Workflow<T>`.
- Preserve existing workflow syntax.
- Emit row summaries for diagnostics and docs.

### 12.2 Stage 2: Row-checked admission

Use row discharge for operation providers/effects, roles, policies, contracts, and
channel/process effects.

- Operation calls require operation effects.
- Role admission can discharge entailed operations.
- Policy effects reference named policy bindings.
- Contract effects preserve static/evidence/dynamic discharge status.

### 12.3 Stage 3: Row profiles for tower syntax

Define `Act`, `Proc`, and `Workflow` as named row profiles over a shared substrate.

- `Act` admits operation/resource/failure effects.
- `Proc` admits process/channel effects.
- `Workflow` admits governance/reporting effects.

### 12.4 Stage 4: Surface simplification

Only after equivalence tests exist, migrate surface syntax toward unified `do {}` and row
annotations. Legacy forms should be deprecated with clear rewrite hints, not removed first.

### 12.5 Stage 5: Optional runtime unification

A later runtime spec may collapse the implementation into one effectful computation
representation. That spec must preserve public semantics, failure attribution, process identity,
workflow reporting, and audit evidence.

## 13. Open decisions for follow-on specs

1. Exact parser spelling for effect rows and effect aliases/groups.
2. Whether role/effect-operation entailment is type-checker-only, runtime-only, or both.
3. Exact policy effect decision domains and handler boundaries.
4. Channel guard failure behavior.
5. User-defined algebraic effect handlers are captured in NOTE-023. Implementation timing
   remains to be scheduled.
6. How row effects are represented in IR and module summaries.
7. How effect aliases/groups are exported, imported, versioned, and invalidated.
8. Which legacy tower syntax becomes deprecated, and when.

## 14. See also

- [SPEC-095a: Current Grammar](SPEC-095a-CURRENT-GRAMMAR.md) — what the parser accepts today
- [SPEC-095b: Target Grammar](SPEC-095b-TARGET-GRAMMAR.md) — target surface syntax
- [SPEC-096a: Current Effect System](SPEC-096a-CURRENT-EFFECT-SYSTEM.md) — current 4-point lattice and tower
- [SPEC-097a: Current Type System](SPEC-097a-CURRENT-TYPE-SYSTEM.md) — current type system without rows
- [SPEC-097b: Target Type System](SPEC-097b-TARGET-TYPE-SYSTEM.md) — type checking for computation rows
- [SPEC-098b: Target IR Changes](SPEC-098b-TARGET-IR.md) — IR representation for computation rows
- [SPEC-099b: Target Operational Semantics](SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md) — runtime semantics for computation rows
- [SPEC-006: Policy Definition Syntax](SPEC-006-POLICY-DEFINITIONS.md)
- [SPEC-007: Policy Combinators](SPEC-007-POLICY-COMBINATORS.md)
- [SPEC-019: Role Runtime Semantics](SPEC-019-ROLE-RUNTIME-SEMANTICS.md)
- [SPEC-024: Capability-Role-Workflow Syntax](SPEC-024-CAPABILITY-ROLE-REDUCED.md)
- [SPEC-052: Capability Interfaces and Implementations](SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md)

## 15. Changelog

- 2026-06-18: Created as target-state effect system document. Defined row semantics, effect item taxonomy, discharge rules, aliases/groups, and migration path.
- 2026-06-27: Reconciled with NOTE-020 (computation row taxonomy), NOTE-021 (Row kind, evidence rows), NOTE-022 (effects as interfaces), NOTE-023 (handler surface semantics).
- 2026-06-27: Reconciled with NOTE-025 (effect identity via sorts and impls). Operation effect identity changed from interface-qualified (`fs.read`) to impl-type-qualified (`PosixFs::read`). §6.1 examples and EBNF updated.
- 2026-06-28: Reconciled with NOTE-029. Clarified §6.8: default dynamic contract failure is structured bottom outside row accounting; recoverable contract behavior must lower to explicit `fail` and expose the failure item in the row.
- 2026-06-29: Reconciled with NOTE-031. Replaced `predicate = expr` with a restricted contract-position predicate grammar, added boundary-local `old(snapshot_expr)`, and required static/dynamic/rejected predicate classification before lowering.
- 2026-06-29: Reconciled with NOTE-033. Clarified that contract-position predicate syntax lowers through structured predicate artifacts carrying binders, snapshots, classification, proof/runtime-check metadata, diagnostics, and stable identity.
