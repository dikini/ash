---
id: spec.ash.core-type-checking
title: Ash Core Type Checking
kind: spec
audience: [human, agent]
authority: design
status: draft
stability: alpha
last_verified: 2026-06-20
verified_against:
  specs:
    - docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md
    - docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md
    - docs/spec/SPEC-098b-TARGET-IR.md
    - docs/spec/SPEC-099-CORE-LANGUAGE.md
---

# SPEC-100: Ash Core Type Checking

**Status:** Draft -- Core-specific type-checking contract for SPEC-099.
**Scope:** Declarative and algorithmic type checking for Core Ash terms after surface elaboration and before Core-to-CPS lowering.
**Depends on:** SPEC-096b (Target Effect System), SPEC-097b (Target Type System), SPEC-098b (Target IR), SPEC-099 (Core Language).

## 1. Summary

Core Ash type checking verifies an already-elaborated Core program. It checks explicit type
annotations, synthesizes local rows needed for diagnostics and CPS field synthesis, emits
refinement and contract obligations, and produces a typed Core program suitable for lowering
to SPEC-098b CPS IR.

Core type checking is not surface elaboration. It does not own parsing, desugaring, type-class
search, dictionary insertion, arbitrary user-defined algebraic effects, or proof search. Those
features either run before Core is produced or run as separate verifier/discharge phases.

The checker has two layers:

1. **Declarative Core typing:** the end-state semantics of well-typed Core Ash.
2. **Initial algorithmic profile:** the first implementable checker, which is bidirectional
   and annotation-led. It checks materialized Core annotations and infers only local facts
   needed to validate rows, operations, refinements, handlers, and CPS lowering.

## 2. Design Goals

1. **Core is checked, not invented.** Surface and elaboration phases may infer rich source
   types. Core mostly verifies those results.
2. **Rows remain requirements.** Row annotations record what a computation requires. They do
   not grant authority.
3. **Discharge is explicit.** Removing a requirement from a residual row requires a discharge
   rule, evidence record, runtime boundary, or handler rule.
4. **Proof search is out of band.** Refinements produce obligations. The checker records and
   classifies them, but does not solve arbitrary predicates itself.
5. **CPS lowering receives typed facts.** Core-to-CPS lowering should not recompute type
   structure. It consumes checked rows, answer types, continuation rows, and operation
   signatures.

## 3. Inputs and Outputs

### 3.1 Inputs

The Core type checker receives:

- a raw or representation-validated Core expression from SPEC-099;
- type declarations, constructors, and nominal type identities available to the module;
- primitive operation signatures;
- raised operation signatures for capability, channel, process, and failure operations;
- transparent effect aliases and diagnostic effect groups;
- an ambient discharge environment;
- optional evidence records from earlier law, proof, test, or contract-elaboration phases;
- source spans and origin metadata when available.

### 3.2 Outputs

The checker produces:

- a typed Core program;
- a synthesized type and local requirement row for each checked expression boundary needed by
  lowering or diagnostics;
- a table of continuation types and rows;
- normalized row summaries for public export where required by SPEC-097b;
- refinement and contract obligations;
- discharge records for statically, evidence, or dynamically discharged contract items;
- structured diagnostics for type, row, discharge, and obligation errors.

## 4. Environments

Core typing uses these logical environments:

```text
G    value environment: value names to Core types
K    continuation environment: continuation names or labels to Cont types
T    type environment: nominal types, type constructors, type variables, and kind facts
R    row environment: row variables, row constraints, aliases, groups, and profiles
O    operation environment: raised operation signatures and operation row identities
D    discharge environment: admitted capabilities, roles, policies, resources, channels,
     evidence sinks, proof facts, and runtime boundaries
Ans  current answer type for continuation checking and CPS lowering
```

The main typing judgment is written schematically as:

```text
G; K; T; R; O; D; Ans |- expr => Type ! Row
```

This means `expr` synthesizes `Type` and has local requirement row `Row` under answer type
`Ans`. A checking judgment is also used:

```text
G; K; T; R; O; D; Ans |- expr <= Type ! Row
```

The checking judgment verifies `expr` against an expected type and expected local row.

## 5. Type Well-Formedness

The checker validates all Core types recursively:

- base types must be known built-ins;
- named types and type constructors must resolve in `T`;
- type variables must be in scope and have type kind;
- row variables must be in scope and have row kind;
- function parameter/result types must be well-formed;
- function rows must be well-formed rows;
- continuation input/answer types and rows must be well-formed;
- tuple element types must be well-formed;
- record field types must be well-formed;
- type applications must match constructor arity and kind;
- refinements must have a well-formed base type and a well-formed predicate environment.

Core may carry type variables and row variables, but it does not introduce implicit binders.
All variables must be introduced by the module/type scheme, function scheme, or surrounding
elaboration metadata.

## 6. Type Equality and Subtyping

### 6.1 Definitional Equality

Definitional equality is structural except where nominal identity is required:

- base types are equal by built-in name;
- named types are equal by resolved nominal identity, not by display text;
- type variables are equal by binder identity;
- type applications are equal when their constructor identities and arguments are equal;
- tuple types are equal by arity and element order;
- record types are semantically equal by field name and field type, not by source order;
- function types compare parameter list, result type, and row compatibility;
- continuation types compare input type, answer type, row compatibility, and multiplicity;
- refinements compare by base type and predicate identity when predicate identity is known.

The Core AST may preserve record field order for stable text, diagnostics, and future layout
work. Type equality must not rely on that order. A later layout phase may choose a fixed
physical field order after type checking.

### 6.2 Subtyping

Core subtyping is deliberately small:

```text
T | P <: T
```

A refinement type is a subtype of its base type. The reverse requires proof or a dynamic
contract/discharge strategy.

Function subtyping follows SPEC-097b:

```text
params_expected <: params_actual
result_actual <: result_expected
Requires(row_actual) <= Requires(row_expected)
------------------------------------------------
(params_actual -> {row_actual} result_actual)
  <: (params_expected -> {row_expected} result_expected)
```

The row relation is contravariant in requirements: a function requiring fewer effects may be
used where a function requiring more effects is expected.

Continuation subtyping is stricter because continuations are control-flow targets:

- input type is contravariant;
- answer type must match exactly in the initial profile;
- row compatibility follows the continuation's required row;
- multiplicity must be compatible.

For the initial algorithmic profile, `Affine` is the only operationally supported handler
resume multiplicity. `MultiShotPure` remains a well-formed type shape but is rejected for
handler resumes unless a later spec defines its proof rule and operational semantics.

## 7. Row Normalization and Compatibility

### 7.1 Normalization

Before comparison or discharge, rows are normalized:

1. expand transparent aliases;
2. preserve diagnostic group/profile names as annotations;
3. canonicalize item identities with namespaces;
4. remove exact duplicate items;
5. preserve open-row tails;
6. reject or defer ambiguous group references;
7. preserve contract and guard predicate identity.

Normalization must not expand role entailment. For example, `role admin` may discharge
`cap fs.read` in a compatible environment, but `role admin` is not normalized into
`cap fs.read`. The role item identity remains visible for audit and diagnostics.

### 7.2 Row Inclusion

`row_a <= row_b` means every normalized requirement in `row_a` also appears in `row_b`,
modulo structural row-variable solving.

Closed rows compare by set inclusion. Open rows compare by structural remainder:

```text
{cap fs.read | r} <= {cap fs.read, cap log.write}
```

may solve `r = {cap log.write}` when the expected type or checking context demands the
larger row. The solver must not infer requirements that are not used, expected, or otherwise
constrained.

### 7.3 Structural Row Solving

The declarative system admits row-variable solving consistent with SPEC-097b. The initial
algorithmic profile supports a conservative subset:

- row variables are kinded separately from type variables;
- solving is structural over normalized row item identities;
- a solver binding records the residual row assigned to the row variable;
- duplicate exact items are ignored after normalization;
- ambiguous aliases or private group names must be expanded, rejected, or deferred before
  solving;
- role entailment is not used to solve row variables.

If a future implementation wants role-sensitive solving, it must be specified as discharge
or entailment, not as row normalization. This prevents authority facts from changing the
semantic identity of a requirement row.

## 8. Discharge and Residual Rows

Environment discharge follows SPEC-096b and SPEC-097b. The checker uses:

```text
D |- Row discharged => DischargeRecords, ResidualRow
```

Each item kind has a separate rule:

- capabilities discharge through admitted capability bindings, provider frames, or admitted
  role entailment;
- roles discharge through role admission at the execution boundary;
- policies discharge through compatible named policy handlers/evaluators;
- contracts discharge through static proof, evidence, or dynamic contract strategy;
- resources discharge through ownership/borrow/provenance facts;
- channels discharge through owned endpoints with compatible mode and message type;
- process effects discharge through compatible `Proc`/`Workflow` boundaries;
- failures discharge through an explicit failure route, handler, or profile boundary;
- evidence effects discharge through available evidence/report sinks.

The checker may either reject undischargeable requirements or leave them in a residual row
for a later boundary. The phase boundary must be explicit in implementation plans. Core type
checking itself must not silently erase a requirement.

## 9. Refinements and Contracts

### 9.1 Predicate Well-Formedness

A refinement or contract predicate is well-formed when:

- all referenced names are in the predicate environment;
- the predicate has Boolean shape;
- predicate binders such as `result`, channel message names, or law parameters are introduced
  by the corresponding contract/evidence context;
- the predicate has local row `{}`;
- the predicate does not call capabilities, perform process/workflow operations, install or
  dispatch handlers, observe time/randomness/environment/global mutable state, inspect
  allocation identity, or implicitly force lazy/memo values;
- row-empty operations are also stable observers;
- the predicate can be represented as an obligation with source and binder metadata.

Per NOTE-031, predicate checking produces a summary before proof or dynamic lowering:

```text
Γp ⊢ pred ⇓ PredicateSummary
```

where the summary records Boolean type, free names, `SnapshotRef`s for `old(path)`, diagnostic
shape, lowered predicate reference, predicate environment, optional dynamic runtime-check plan,
and either `StaticPredicate` or `DynamicPredicate` classification. A rejected predicate does
not reach the prover or runtime checker.

Per NOTE-033, the checker lowers a surface contract expression through a staged pipeline before
proof or runtime checking:

```text
contract-position expression
  -> scoped and typed predicate tree
  -> `old(path)` to SnapshotRef
  -> stability and authority checks
  -> LoweredPredicate / PredicateRef
  -> StaticPredicate proof obligation or DynamicPredicate RuntimeCheckPlan
  -> ContractDischarge metadata
```

Source text may be preserved for diagnostics, but text alone is not sufficient for public
obligations, evidence caching, optimizer checks, or dynamic execution.

Per NOTE-034, authority checks reject operation/capability calls inside predicates while
allowing operation-produced values already present in the predicate environment. The required
order is: type-check/admit the operation in ordinary program semantics, bind its result as a
value, then allow the predicate to inspect that value without acquiring authority. The checker
must keep capability admission failure, capability operation failure, predicate false, and
predicate evaluator fault as separate outcomes.

### 9.2 Obligation Generation

Checking a plain value of type `T` against a refinement type `T | P` requires producing an
obligation or finding existing evidence. A value already typed as `T | P` may be used at
base type `T` by refinement subtyping without producing a new obligation:

```text
G |- value : T
WellFormed(P, G)
-------------------------------------
G |- value <= T | P, emits obligation P(value)
```

Obligations are not ordinary Core values. They are compiler metadata associated with the
typed Core boundary, contract discharge, or sidecar records.

### 9.3 Obligation Discharge

The end-state checker recognizes these outcomes:

| Outcome | Type-checking consequence |
|---------|---------------------------|
| Proven | keep static refinement and record static/evidence discharge |
| Disproved | reject with a type-checker or verifier diagnostic |
| Unknown | record obligation and either defer or demote to dynamic contract strategy |
| Statistical | record advisory evidence only; do not harden the refinement |

Proof search, SMT solving, external proof checking, QuickCheck, and SmallCheck are separate
passes. Core type checking defines the obligation shape and consumes their results.

### 9.4 Dynamic Contract Strategy

Dynamic contract discharge records `DischargeMode::Dynamic` and leaves an explicit runtime
check in Core. If the check fails unrecoverably, the program traps with
`ContractViolation(ContractDiagnostic)`. The diagnostic records the predicate, source span,
blame label, observed values, discharge history, handler decisions, and replay status.
Recoverable behavior must be represented by an explicit `fail` operation and corresponding
failure row item. `ContractViolation` is not a row item and is not a raised operation.

A dynamic predicate that returns `false` follows this rule. A dynamic predicate that traps or
faults while being evaluated produces a distinct `ContractPredicateFault` trap payload. The
checker must not report a predicate fault as a failed contract condition: it is a failure of
predicate evaluation, normally blamed on the contract author or admitted predicate-function
provider rather than on the caller/callee relation.

The dynamic check's evaluator consumes a `RuntimeCheckPlan` over a `PredicateRef` and captured
`PredicateEnvironment`; it must not re-scope or re-parse surface predicate text at runtime.

### 9.5 Trace Contract Strategy

Per NOTE-035, trace contracts use a separate staged judgment:

```text
Γtrace ⊢ formula ⇓ TraceContract
```

The checker builds `Γtrace` from event schemas, process/channel/resource identities, timer
facts, workflow ledger fact schemas, evidence/provenance policies, redaction rules, and
monitor scope boundaries. It then classifies the formula's alphabet, type-checks the temporal
formula, chooses `StaticModelChecked`, `StaticProved`, `EvidenceSurvivedTesting`,
`RuntimeMonitor`, or `Deferred` discharge, and records monitor metadata.

`Proc` and `Workflow` are not separate type-checking universes here. A formula is `Proc`-like
when it mentions operational trace facts, `Workflow`-like when it mentions
obligation/evidence/commitment/stage facts, and mixed when it relates both. Unknown temporal
proofs demote to `RuntimeMonitor` or remain deferred; they do not become value-level
`RuntimeCheckPlan`s.

## 10. Atom and Value Typing

### 10.1 Atoms

Atoms synthesize types:

- variable atoms look up their type in `G`;
- literals synthesize their base type;
- primitive names synthesize compiler-known primitive function types;
- constructor names synthesize constructor function or tag identities from `T`.

Continuation references are not atoms. They are looked up in `K` and may appear only in
continuation positions.

### 10.2 Values

Values are inert and have construction row `{}` unless their components carry administrative
metadata:

- `Atom(a)` has the type synthesized by `a`;
- `Lam(params, body, row)` checks each parameter type, checks `body`, and verifies the
  synthesized body row is compatible with `row`;
- records check each field atom and produce a record type by field name;
- tuples check each element atom and produce an ordered tuple type;
- `DischargeMarker` checks discharge metadata and produces an administrative marker type or
  unit-like metadata type determined by the implementation.

`DischargeMarker` is not user data. It must not be pattern-matched as ordinary program data
or exported as a user-visible evidence value.

## 11. Expression Typing

### 11.1 Atom Expression

An atom expression synthesizes the atom type and row `{}`.

### 11.2 LetVal

```text
G |- value <= declared_type ! {}
G, name: declared_type |- body => body_type ! body_row
-------------------------------------------------------
G |- LetVal(name, declared_type, value, body) => body_type ! body_row
```

If `declared_type` is a refinement, checking the value may emit an obligation or consume
evidence.

### 11.3 LetRec

`LetRec` first adds the declared binding to `G`, then checks the recursive value against that
type, then checks the body. The construction row of the recursive value must be compatible
with `{}`. The latent row of a recursive function is charged when the function is called, not
when the recursive binding is constructed.

### 11.4 LetPrim

Primitive operation signatures are compiler-known. `LetPrim` checks argument atom types and
binds the primitive result type in the body. Primitive operations in Core are pure. Effectful
host or capability operations must be represented as `Raise`, not `LetPrim`.

### 11.5 LetCall

`LetCall` checks the function atom as a function type, checks argument atom types, binds the
function result type to `name`, and checks the body. Its local row is:

```text
callee_body_row union body_row
```

The CPS lowering introduces a continuation for `body`; the checker must preserve enough row
facts for lowering to materialize the CPS `Call.row` as specified by SPEC-098b and SPEC-099.

If the callee result carries postcondition `Q(name)` and the checked body immediately consumes
`name` in a continuation with precondition `R(name)`, the checker emits NOTE-030's composition
obligation:

```text
∀name. Q(name) ⇒ R(name)
```

When this obligation is proven, the continuation precondition is discharged by the producer
postcondition and the checker records composed contract metadata. When it is unknown, the
active verification profile chooses rejection, deferred obligation, or dynamic demotion. A
dynamic demotion inserts a runtime check at the continuation boundary; unrecoverable failure
traps with `ContractViolation(ContractDiagnostic)`, while recoverable behavior must use an
explicit `fail` effect and visible failure row item.

### 11.6 If

The condition must check as `Bool`. Both branches must check against a compatible result
type. The local row is the normalized union of branch rows. A trapping branch type-checks at
any expected result type with row `{}`.

### 11.7 Call

Tail `Call` checks like `LetCall` but returns through the current continuation. Its local row
is the callee's body row. The total CPS row later includes the current continuation row.

### 11.8 Jump

`Jump(cont, arg)` looks up `cont` in `K`, checks `arg` against the continuation input type,
and synthesizes the continuation answer type. Its Core local row is `{}`. The target
continuation row is preserved separately so CPS lowering can set `Jump.row` to that
continuation row and compute the total row according to SPEC-098b.

### 11.9 Raise

`Raise(op, args)` checks the operation in `O`, checks argument atom types against the
operation signature, and synthesizes the operation result type. Its local row is the
operation row only:

```text
row(Raise(op, args)) = row(op)
```

The resume continuation row is not stored in Core `Raise`; it is added during CPS lowering
from the current continuation context.

### 11.10 Handle

`Handle(clause, body)` checks:

1. `clause.op` is a representable raised operation kind;
2. `clause.params` match the operation argument types;
3. `clause.resume` has type `Cont<op_result, Ans, resume_row, Affine>`;
4. the handler body checks under the operation parameters and resume binding;
5. affine use of the resume binding is valid;
6. `clause.row` matches the local row of the handler body excluding resume and outer
   continuation rows;
7. the handled body row is transformed by removing the handled raised operation from the
   delimited pre-resume segment and adding both `resume_row` and `clause.row`.

For a user-installed resumptive handler, the residual local row is:

```text
(handled_segment.local - handled_op) union resume_row union clause.row
```

The `resume_row` term preserves effects reachable after resumption, including same-operation
effects that reappear after the resume point. Provider handlers that persist across resume may
use the stronger provider-frame transformation from SPEC-098b.

`Handle` does not discharge ambient role, policy, contract, resource, or evidence items.
Those items use discharge rules, not handler frames.

### 11.11 RecordDischarge

`RecordDischarge(discharge, body)` validates the discharge metadata, records the discharge
boundary, and checks `body`. It does not by itself perform runtime effects. The row is the
body row after the recorded discharge effect has been justified or after a dynamic strategy
has been materialized.

### 11.12 Trap

`Trap(reason)` checks at any expected type and has row `{}`. `TrapReason` is diagnostic
metadata. `ContractViolation(ContractDiagnostic)` inside a trap does not create a row item;
recoverability requires an explicit `fail` effect whose row item is visible in the enclosing
function or computation type.

## 12. Handler Affinity

Handler resume continuations are affine by default:

- a resume may be jumped to at most once on any dynamic path;
- a resume may appear in multiple mutually exclusive branches;
- a resume may not be stored in records or tuples;
- a resume may not be captured by ordinary lambdas;
- a resume may not be passed as an unrestricted ordinary function argument;
- a resume may not be duplicated through aliases.

The initial algorithmic profile may conservatively reject programs it cannot prove affine.
Path-sensitive acceptance is the declarative end-state.

## 13. Public Summaries

Public Core items must export enough type and row information for downstream checking:

- normalized public function types and rows;
- type constructor identities and arities;
- public effect aliases and groups referenced by public rows;
- public contract/refinement obligation identities;
- discharge and evidence references that affect public obligations.

Private aliases or groups must either be expanded before export or rejected when they would
leak into a public row.

## 14. Initial Algorithmic Profile

The first implementation profile is intentionally smaller than the declarative system.

It should:

1. use bidirectional checking with explicit Core annotations as the source of truth;
2. validate all type and row shapes;
3. synthesize atom types from the value environment;
4. check primitive and raised operation arity and argument types;
5. check function and continuation annotations against bodies;
6. normalize rows and remove duplicate exact items before comparison;
7. support structural row solving for explicit row variables;
8. record role identities structurally, while leaving role entailment to discharge;
9. generate refinement/contract obligations with scoped metadata;
10. accept static/evidence/dynamic discharge records only when their shape is coherent;
11. enforce conservative affine handler-resume usage;
12. produce row facts required by Core-to-CPS lowering.

It may defer:

- full Hindley-Milner-style inference;
- complete row-polymorphic inference;
- role-sensitive authority entailment beyond explicit discharge checks;
- SMT/proof solving;
- arbitrary user-defined algebraic effects;
- typeclass or ad-hoc-polymorphism solving;
- higher-kinded typeclass semantics;
- `MultiShotPure` operational support;
- session-type or MPST channel checking.

## 15. Diagnostics

Core type-checker diagnostics must be structured and phase-tagged as `TypeChecker`. They must
distinguish at least:

- unknown type, value, constructor, continuation, row variable, or operation;
- malformed type application or kind mismatch;
- atom/type mismatch;
- function argument arity mismatch;
- primitive operation arity/type mismatch;
- raised operation arity/type mismatch;
- row mismatch or unsolved row variable;
- missing or invalid discharge;
- contract/refinement predicate not well-scoped, not Boolean, unstable, effectful, or
  implicitly forcing;
- disproved or invalid evidence;
- unsupported `MultiShotPure` use;
- affine resume violation;
- private alias or group leaking into public summary.

Diagnostics should include source spans when available, the normalized expected and actual
types/rows, and help text that names the missing effect kind rather than emitting only a
generic row mismatch.

## 16. Relationship to Other Specs

| Spec | Role |
|------|------|
| SPEC-096b | Effect taxonomy and discharge semantics |
| SPEC-097b | Target row polymorphism, subtyping, and type-system semantics |
| SPEC-098b | CPS IR answer type, continuation, raise/handle, and row accounting target |
| SPEC-099 | Core syntax and Core-to-CPS lowering boundary |
| **SPEC-100** | **Core-specific type-checking contract** |

## 17. Open Questions

1. How much path-sensitive affine analysis belongs in the Core checker versus a later
   validation pass?
2. Should public Core summaries preserve diagnostic group names in addition to normalized
   concrete row items?
3. Should continuation answer type compatibility eventually admit subtyping, or remain exact?
4. Which implementation phase should first materialize role entailment as discharge facts?

## Changelog

- 2026-06-20: Created Core Ash type-checking specification with declarative rules and an initial annotation-led algorithmic profile.
- 2026-06-28: Reconciled with NOTE-029. Dynamic contract failure traps with `ContractViolation(ContractDiagnostic)`, trap typing remains row `{}`, and recoverability requires explicit `fail` with a visible failure row item.
- 2026-06-28: Reconciled with NOTE-030. `LetCall`/sequencing now emits the producer-postcondition-to-continuation-precondition obligation `∀name. Q(name) ⇒ R(name)` and records composed contract metadata when discharged.
- 2026-06-29: Reconciled with NOTE-031. Predicate well-formedness now emits structured summaries and snapshot references before proof obligations; rejected predicates stop before prover/runtime checking; dynamic predicate faults are distinct from false-predicate contract violations.
- 2026-06-29: Reconciled with NOTE-033. Core type checking now specifies the Surface-to-Core predicate lowering pipeline, `LoweredPredicate`/`PredicateRef` boundary, and `RuntimeCheckPlan` use for dynamic checks.
- 2026-06-29: Reconciled with NOTE-034. Predicate authority checks now reject capability calls while allowing operation-produced values in the predicate environment, preserving separate diagnostics for admission failure, operation failure, predicate false, and predicate fault.
- 2026-06-29: Reconciled with NOTE-035. Added trace-contract checking over `Γtrace`, monitor discharge classification, and the rule that unknown temporal proofs demote to monitor plans rather than value-level runtime predicates.
