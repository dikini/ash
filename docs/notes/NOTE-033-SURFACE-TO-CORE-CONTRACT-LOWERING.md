# NOTE-033: Surface-to-Core Contract Lowering

**Date:** 2026-06-29
**Status:** Living document — design direction captured; resolves NOTE-014 GAP 9
**Purpose:** Define the concrete boundary where surface contract clauses become typed Core predicate artifacts, proof obligations, dynamic checks, snapshots, diagnostics, and discharge metadata. NOTE-031 defined which predicates are well formed; NOTE-032 defined the soundness obligations lowering must preserve. This note specifies the Core predicate schema and lowering algorithm that connect those decisions to implementation.

Companion to NOTE-014 (contract systems unification), NOTE-027 (blame and subsumption), NOTE-028 (evaluation-mode timing), NOTE-029 (structured bottom), NOTE-030 (monadic Hoare composition), NOTE-031 (predicate well-formedness), NOTE-032 (contract soundness obligations), SPEC-096b (target effect syntax), SPEC-097b (target type system), SPEC-098b (target IR), SPEC-099 (Core language), and SPEC-100 (Core type checking).

## Pre-Spec Delta

This note resolves the remaining NOTE-014 GAP 9 residual: the concrete Core predicate schema and lowering algorithm. When promoted into target specs, reconcile:

- **SPEC-096b Target Effect System:** keep contract-position expression syntax, but make clear that every contract predicate lowers through `LoweredPredicate` before becoming a `PredicateRef`.
- **SPEC-097b Target Type System:** refine `PredicateSummary` with a lowered predicate body, binder environment, proof profile, and dynamic-evaluator profile.
- **SPEC-098b Target IR:** add `LoweredPredicate`, `PredicateNode`, `PredicateBinder`, `PredicateEnvRef`, and `RuntimeCheckPlan` sidecar records, or equivalent implementation-facing names.
- **SPEC-099 Core Language:** clarify that Core does not evaluate source text. Dynamic checks evaluate a lowered predicate evaluator over boundary atoms, snapshots, and binders.
- **SPEC-100 Core Type Checking:** specify the lowering pipeline: parse expression in contract position, scope/type/check stability, lower to Core predicate schema, classify, emit proof obligation or runtime check, and record discharge metadata.

No new user-visible contract syntax is required. The note defines an implementation boundary, not a surface feature.

## 0. Motivation

Ash has deliberately kept surface contracts pleasant to write:

```ash
fn push(xs: List<A>, x: A) -> List<A>
    requires: sorted(xs)
    ensures: result.len == old(xs.len) + 1
{ ... }
```

But Core cannot depend on an arbitrary source expression string. The checker, prover, runtime, diagnostics, and optimizer need a structured artifact that says:

- which boundary owns the predicate;
- which names are legal inside it;
- which values are snapshots;
- which portion is SMT-safe;
- which portion is pure dynamic evaluator code;
- which diagnostic and blame metadata must be preserved;
- which runtime check, if any, must be inserted.

NOTE-031 defined the predicate-language boundary. NOTE-032 defined the soundness obligations. This note turns those into a lowering algorithm.

## 1. Core decision

Ash lowers every surface contract predicate into a typed Core predicate object before proof or runtime checking.

```text
Surface predicate expression
  -> scoped contract-position AST
  -> LoweredPredicate
  -> PredicateRef
  -> proof obligation or RuntimeCheckPlan
  -> ContractDischarge metadata
```

The lowered predicate object, not source text, is the semantic input to:

- SMT/proof discharge;
- dynamic predicate evaluation;
- contract diagnostics;
- bind-composition obligations;
- optimizer evidence checks;
- cross-module evidence caching.

Source text is preserved for diagnostics only. It is not the executable or provable predicate representation.

## 2. Contract boundary inputs

Lowering starts from a contract clause and the boundary that owns it:

```text
SurfaceContractClause =
  Requires(expr, span)
| Ensures(expr, span)
| Invariant(expr, span)
| Guard { binder, expr, span }
| Law { name, params, expr, span }
```

The boundary determines the predicate environment:

```text
ContractBoundary = {
  boundary_id: BoundaryId,
  kind: Requires | Ensures | Invariant | Guard | Law,
  owner: ContractOwner,
  source_span: Span,
  lexical_env: Γ,
  result_binder: Option<Binder>,
  message_binder: Option<Binder>,
  snapshot_env: SnapshotEnv,
  blame: BlameLabel,
  recoverability: TrapDefault | ExplicitFail(FailureType),
}
```

The boundary, not the predicate text, decides whether `result`, `message`, and `old(...)` are in scope.

## 3. Core predicate schema

The implementation-facing schema is intentionally smaller than full Ash expressions.

```rust
pub struct LoweredPredicate {
    pub id: PredicateId,
    pub source_span: Span,
    pub contract_text: String,
    pub boundary: BoundaryId,
    pub env: PredicateEnvRef,
    pub root: PredicateNode,
    pub ty: Type, // must be Bool
    pub free_vars: Vec<PredicateBinderRef>,
    pub snapshot_refs: Vec<SnapshotRef>,
    pub classification: PredicateClassification,
    pub proof_fragment: Option<ProofFragment>,
    pub dynamic_plan: Option<DynamicPredicatePlan>,
    pub diagnostic_shape: DiagnosticShape,
}
```

`PredicateRef` points at a `LoweredPredicate` plus stable identity information:

```rust
pub struct PredicateRef {
    pub id: PredicateId,
    pub stable_hash: PredicateHash,
    pub boundary: BoundaryId,
    pub source_span: Span,
}
```

The stable hash is computed from the lowered tree, binder identities, snapshot references, admitted predicate-function identities, and relevant type encodings. It must not be a hash of source text alone.

### 3.1 Predicate nodes

The initial node family is:

```rust
pub enum PredicateNode {
    BoolLit(bool),
    IntLit(i128),
    StringLit(String),
    UnitLit,

    Binder(PredicateBinderRef),
    Result(BinderRef),
    Message(BinderRef),
    Snapshot(SnapshotRef),

    Field { base: Box<PredicateNode>, field: FieldId },
    TupleIndex { base: Box<PredicateNode>, index: usize },

    Not(Box<PredicateNode>),
    And(Box<PredicateNode>, Box<PredicateNode>),
    Or(Box<PredicateNode>, Box<PredicateNode>),
    Implies(Box<PredicateNode>, Box<PredicateNode>),

    Eq(Box<PredicateNode>, Box<PredicateNode>),
    Ne(Box<PredicateNode>, Box<PredicateNode>),
    Lt(Box<PredicateNode>, Box<PredicateNode>),
    Le(Box<PredicateNode>, Box<PredicateNode>),
    Gt(Box<PredicateNode>, Box<PredicateNode>),
    Ge(Box<PredicateNode>, Box<PredicateNode>),

    Add(Box<PredicateNode>, Box<PredicateNode>),
    Sub(Box<PredicateNode>, Box<PredicateNode>),
    Mul(Box<PredicateNode>, Box<PredicateNode>),
    Div(Box<PredicateNode>, Box<PredicateNode>),
    Rem(Box<PredicateNode>, Box<PredicateNode>),

    PredicateCall {
        callee: PredicateFunctionRef,
        args: Vec<PredicateNode>,
    },
}
```

This is not a new user-facing AST. It is the contract-position Core predicate representation.

### 3.2 Deliberately absent nodes

The initial schema does not include:

- general function calls;
- handler installation or dispatch;
- capability, process, workflow, or channel operations;
- allocation, pointer, force-count, time, randomness, or global environment observation;
- arbitrary `let` bindings;
- loops or recursion inside predicate syntax;
- source-level `forall` or `exists`.

`forall` and `exists` remain internal proof metadata for NOTE-030 composition. They are represented in proof obligations, not in source predicate syntax.

## 4. Binder and environment model

Every name in a lowered predicate resolves to a typed binder identity.

```rust
pub enum PredicateBinderKind {
    Lexical,
    Parameter,
    Result,
    Message,
    LawParameter,
    IntermediateBindValue,
}

pub struct PredicateBinder {
    pub id: PredicateBinderId,
    pub name: Name,
    pub kind: PredicateBinderKind,
    pub ty: Type,
    pub source_span: Span,
}
```

The environment record is:

```rust
pub struct PredicateEnvironment {
    pub id: PredicateEnvId,
    pub boundary: BoundaryId,
    pub binders: Vec<PredicateBinder>,
    pub snapshots: SnapshotEnv,
    pub admitted_predicate_fns: Vec<PredicateFunctionRef>,
    pub redaction_policy: RedactionPolicy,
}
```

Lowering must reject an unbound name even if the same spelling exists in a later continuation or outer boundary. A `PredicateBinderId` is scoped to the boundary that admitted it.

## 5. Snapshot lowering

`old(path)` lowers to a `SnapshotRef`, not to a delayed expression.

```rust
pub struct SnapshotRef {
    pub boundary: BoundaryId,
    pub root: PredicateBinderId,
    pub path: Vec<FieldId>,
    pub ty: Type,
    pub source_span: Span,
}
```

The lowering rule is:

```text
Γp.snapshots contains root x at boundary β
path x.f1...fn is field-valid and visible at β
------------------------------------------------
Γp ⊢ old(x.f1...fn) ⇓ PredicateNode::Snapshot(SnapshotRef { β, x, [f1...fn] })
```

`old(...)` does not lower by re-reading the variable after the body runs. It names a capture in `SnapshotEnv` that was taken at the owning boundary.

Rejected forms include:

```ash
old(f(x))
old(force_unsafe(x))
old(clock.now())
old(result)
```

`old(result)` is rejected because `result` is not available before the body governed by the postcondition runs.

## 6. Lowering algorithm

The lowering pass is deterministic and staged.

```text
lower_contract_clause(clause, boundary):
  1. build PredicateEnvironment from boundary
  2. parse or reuse expression AST in contract-position mode
  3. resolve every name to a PredicateBinder or PredicateFunctionRef
  4. type-check the predicate tree; require final type Bool
  5. lower allowed forms to PredicateNode
  6. lower old(path) to SnapshotRef
  7. compute stability and authority checks
  8. classify as StaticPredicate, DynamicPredicate, or rejected diagnostic
  9. compute PredicateHash and DiagnosticShape
 10. allocate LoweredPredicate and PredicateRef
 11. choose proof obligation or runtime check plan
 12. record ContractDischarge metadata
```

The key point is that rejection happens before SMT and before runtime checking. A rejected predicate is not demoted to dynamic. Only a well-formed but non-SMT predicate becomes dynamic.

### 6.1 Rejection phase

Reject if any lowered subtree would:

- require a non-empty row;
- require authority or host interaction;
- perform handler dispatch;
- perform process/workflow/channel operations;
- observe unstable operational state;
- implicitly force lazy/memo values outside a contract-owned observation boundary;
- evaluate a non-admitted function;
- produce a non-Boolean top-level predicate.

The rejection diagnostic should name the contract boundary, rejected subexpression, reason, and source span.

### 6.2 Classification phase

Classification consumes the lowered tree:

```text
SMT-safe and stable   => StaticPredicate
pure/stable but not SMT-safe => DynamicPredicate
not pure/stable/legal => rejected diagnostic
```

The static/dynamic split is not a statement about truth. It is a statement about discharge mechanism.

## 7. Static proof lowering

For `StaticPredicate`, Core emits a proof obligation over the lowered predicate tree.

```rust
pub struct ProofObligation {
    pub id: ObligationId,
    pub predicate: PredicateRef,
    pub boundary: BoundaryId,
    pub assumptions: Vec<PredicateRef>,
    pub obligation_kind: Requires | Ensures | Invariant | Guard | Subsumption | Composition,
    pub source_span: Span,
}
```

SMT encoding is a later pass:

```text
LoweredPredicate -> TheoryEncoding -> SMT-LIB query / proof backend query
```

The encoding may use:

- bit/int/real/string theories where available;
- uninterpreted functions for admitted predicate functions with explicit summaries;
- named constants for `SnapshotRef`s;
- named binders for `result`, `message`, and intermediate bind values;
- implication obligations for subsumption and bind composition.

A proof result updates the discharge:

```text
Proven    -> DischargeMode::Static or Evidence
Disproved -> type/checker diagnostic; program rejected
Unknown   -> defer or DynamicPredicate runtime plan, depending on profile
```

Unknown is not false. This preserves NOTE-032 dynamic demotion soundness.

## 8. Dynamic runtime lowering

For `DynamicPredicate`, or for an unknown static obligation demoted by policy, Core emits a runtime check plan.

```rust
pub struct RuntimeCheckPlan {
    pub predicate: PredicateRef,
    pub evaluator: DynamicPredicatePlan,
    pub boundary: BoundaryId,
    pub blame: BlameLabel,
    pub snapshots: Vec<SnapshotRef>,
    pub diagnostic_shape: DiagnosticShape,
    pub recoverability: TrapDefault | ExplicitFail(FailureType),
}
```

The Core shape is:

```text
RecordDischarge {
  discharge: ContractDischarge { mode: Dynamic, contract, predicate, blame, ... },
  body: If {
    cond: not(eval_predicate(predicate, PredicateEnvironment)),
    then_branch: Trap { reason: ContractViolation(ContractDiagnostic { ... }) },
    else_branch: body
  }
}
```

If the dynamic predicate evaluator itself traps or faults, the branch is not treated as predicate false:

```text
eval_predicate returns False  -> Trap(ContractViolation(... PredicateFalse ...))
eval_predicate faults         -> Trap(ContractPredicateFault(... PredicateFault ...))
```

If the contract surface explicitly requests recoverability, the false-predicate branch lowers to explicit `fail` and the failure row item is visible. Predicate faults remain diagnostic traps unless the language later adds an explicit, row-accounted predicate-fault recovery mechanism.

## 9. Contract-kind-specific lowering

### 9.1 `requires`

`requires P(args)` lowers at function entry or call boundary:

- `result` is not in scope;
- `old(...)` is normally not in scope unless an invariant-like boundary defines it;
- false predicate blames caller / negative party;
- successful static discharge refines admitted arguments or records evidence.

### 9.2 `ensures`

`ensures Q(args, result, old(args))` lowers around the body:

- snapshots are captured before the governed body runs;
- `result` is bound after normal return;
- false predicate blames callee/impl / positive party;
- dynamic check occurs at the return boundary.

### 9.3 `invariant`

An invariant is a paired boundary. The lowering must spell out which edge is being checked:

- entry preservation;
- exit preservation;
- construction admission;
- field update boundary.

Until the invariant polarity model is fully specified, lowering must preserve enough boundary metadata for NOTE-027 blame refinement rather than collapsing all invariant failures into one label.

### 9.4 channel `guard`

A guard lowers with an explicit message binder:

```text
message: MessageType
```

The guard is a contract over the communication boundary. Runtime behavior for guard failure remains channel-policy-specific, but the predicate lowering itself is the same: `message` is a binder, not a magic global.

### 9.5 laws and internal composition obligations

Law predicates and NOTE-030 composition obligations share the lowered predicate schema, but they are not ordinary source predicates. Composition introduces internal binders such as the intermediate value `a`:

```text
∀a. producer.Q(a) ⇒ continuation.R(a)
```

The quantifier is represented in the proof-obligation layer, while `Q(a)` and `R(a)` are `PredicateRef`s over explicit binder environments.

## 10. Worked examples

### 10.1 Simple precondition

```ash
fn divide(a: Int, b: Int) -> Int
    requires: b != 0
{ a / b }
```

Lowered predicate:

```text
PredicateNode::Ne(
  Binder(b: Int),
  IntLit(0)
)
classification = StaticPredicate
```

If proven at a call boundary, no dynamic check is installed. If unknown and the profile permits demotion, the runtime check is inserted at the entry boundary and false blames the caller.

### 10.2 Postcondition with snapshot

```ash
fn push(xs: List<A>, x: A) -> List<A>
    ensures: result.len == old(xs.len) + 1
{ ... }
```

Lowered predicate:

```text
Eq(
  Field(Result(result), len),
  Add(Snapshot(boundary=push.entry, root=xs, path=[len]), IntLit(1))
)
```

The snapshot is captured before the body runs. Optimizers must preserve that boundary identity.

### 10.3 Admitted predicate function

```ash
pred fn sorted(xs: List<Int>) -> Bool { ... }

fn binary_search(xs: List<Int>, target: Int) -> Int
    requires: sorted(xs)
{ ... }
```

Lowering resolves `sorted` to `PredicateFunctionRef(sorted)`, not a general function call. If `sorted` has an SMT summary, the predicate may be static. If it is pure and stable but lacks an SMT summary, it is dynamic. If it is not admitted as a predicate function, the contract is rejected.

### 10.4 Unsupported but pure predicate

```ash
fn f(xs: List<String>) -> Int
    requires: normalized_utf8_names(xs)
{ ... }
```

If `normalized_utf8_names` is admitted, pure, stable, and total-or-fault-classified, but not SMT-encodable, lowering produces `DynamicPredicate`. The check remains meaningful at runtime and is not silently erased.

### 10.5 Rejected effectful predicate

```ash
fn f(path: Path) -> Unit
    requires: fs.exists(path)
{ ... }
```

`fs.exists` requires capability authority. The predicate is rejected before prover/runtime checking. It is not demoted to dynamic, because a contract checker must not acquire filesystem authority implicitly.

### 10.6 Predicate fault

```ash
pred fn valid_ratio(n: Int, d: Int) -> Bool {
    n / d >= 0
}

fn f(n: Int, d: Int) -> Int
    requires: valid_ratio(n, d)
{ ... }
```

If `d == 0`, dynamic evaluation faults. The lowering routes that outcome to `ContractPredicateFault`, not `ContractViolation(PredicateFalse)`.

## 11. Design decisions

1. **Source text is diagnostic, not semantic.** Core predicates are structured lowered objects.
2. **Every contract predicate has a boundary.** The boundary owns binders, snapshots, blame, recoverability, and discharge metadata.
3. **Rejected predicates stop before proof/runtime.** Rejection is not dynamic demotion.
4. **Dynamic predicates are still pure observers.** Dynamic means not statically discharged, not effectful.
5. **`old(...)` is a snapshot reference.** It never lowers to a delayed expression or post-body variable read.
6. **Predicate functions are admitted explicitly.** General function calls do not appear in Core predicate nodes.
7. **Composition quantifiers remain proof metadata.** Source-level `forall`/`exists` remains deferred.
8. **Runtime checks evaluate lowered predicates.** They do not re-interpret arbitrary Ash source expressions.
9. **False and fault outcomes are separate control paths.** This preserves blame soundness from NOTE-032.

## 12. Open questions

1. **Exact Rust type names.** Should implementation use `LoweredPredicate`/`PredicateNode`, or reuse existing Core expression types with a restricted kind? This note specifies semantics, not final Rust names.
2. **Predicate-function spelling.** NOTE-031 proposed `pred fn` as target syntax. The classification is required; the spelling remains open.
3. **SMT theory profile.** Which exact SMT-LIB logics are initially supported for integers, strings, algebraic data, and finite maps?
4. **Dynamic evaluator representation.** Should `DynamicPredicatePlan` compile to a small interpreter, specialized Core function, or backend-native predicate thunk?
5. **Invariant polarity.** Invariant lowering needs the more precise invariant blame model left open by NOTE-027/NOTE-029.
6. **Cross-module predicate hashes.** Evidence caching needs a stable hash and invalidation model across package boundaries.

## 13. References

### Internal references

- [NOTE-014: Contract Systems Unification](NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md)
  — source gap register; this note resolves GAP 9.
- [NOTE-027: Contract Blame and Subsumption](NOTE-027-CONTRACT-BLAME-AND-SUBSUMPTION.md)
  — blame labels and contract polarity preserved by lowering.
- [NOTE-028: Purity, Evaluation Modes, and Contract Timing](NOTE-028-PURITY-EVALUATION-MODES-AND-CONTRACT-TIMING.md)
  — lazy/memo timing rules that predicate lowering must not violate.
- [NOTE-029: Structured Bottom and Contract Diagnostics](NOTE-029-STRUCTURED-BOTTOM-AND-CONTRACT-DIAGNOSTICS.md)
  — default dynamic contract failure as structured bottom and explicit recoverable `fail`.
- [NOTE-030: Monadic Hoare Logic for Ash Computations](NOTE-030-MONADIC-HOARE-LOGIC-FOR-ASH-COMPUTATIONS.md)
  — bind composition obligations over lowered predicate references.
- [NOTE-031: Contract Predicate Well-Formedness and Snapshot Semantics](NOTE-031-CONTRACT-PREDICATE-WELL-FORMEDNESS-AND-SNAPSHOTS.md)
  — predicate grammar, classification, snapshots, dynamic predicate faults.
- [NOTE-032: Contract Soundness Obligations](NOTE-032-CONTRACT-SOUNDNESS-OBLIGATIONS.md)
  — soundness obligations that this lowering algorithm must preserve.
- [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
  — contract-position predicate syntax and effect rows.
- [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
  — `ContractEffect`, `PredicateSummary`, and predicate well-formedness.
- [SPEC-098b: Target IR](../spec/SPEC-098b-TARGET-IR.md)
  — `ContractDischarge`, `SnapshotRef`, diagnostics, `ComposedContract`, and trap reasons.
- [SPEC-099: Core Language](../spec/SPEC-099-CORE-LANGUAGE.md)
  — Core contract-discharge and dynamic-check expression shapes.
- [SPEC-100: Core Type Checking](../spec/SPEC-100-CORE-TYPE-CHECKING.md)
  — predicate checking, obligation generation, dynamic demotion, and `LetCall` composition.

### External references

- Clark Barrett, Pascal Fontaine, and Cesare Tinelli et al., **SMT-LIB: The Satisfiability Modulo Theories Library**.
  Verified by browser during this session; relevant to the proof-query boundary for lowered static predicates.
  <https://smt-lib.org/>
- C. A. R. Hoare, "An Axiomatic Basis for Computer Programming" (1969).
  Foundation for pre/postcondition obligations used by lowered Hoare predicates.
  <https://doi.org/10.1145/363235.363259>
- Danel Ahman, Cătălin Hriţcu, Kenji Maillard, Guido Martínez, Gordon Plotkin, Jonathan Protzenko, Aseem Rastogi, and Nikhil Swamy, "Dijkstra Monads for Free" (POPL 2017).
  Prior art for connecting effectful computations to weakest-precondition style proof obligations.
  <https://www.fstar-lang.org/papers/dm4free/>
- Nikhil Swamy et al., "Dependent Types and Multi-monadic Effects in F*" (POPL 2016).
  Prior art for combining SMT automation, dependent typing, and effectful program verification.
  <https://www.fstar-lang.org/papers/mumon/>

## 14. Changelog

| Date | Change |
|------|--------|
| 2026-06-29 | Initial note. Resolves NOTE-014 GAP 9 by defining `LoweredPredicate`, the Core predicate node schema, binder/snapshot environment model, lowering algorithm, proof-obligation boundary, dynamic runtime-check plan, and worked examples. |
