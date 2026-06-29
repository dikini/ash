# NOTE-034: Contract ↔ Capability Boundary

**Date:** 2026-06-29
**Status:** Living document — design direction captured; resolves NOTE-014 GAP 8
**Purpose:** Define the semantic boundary between authority-bearing capability operations and authority-free contract predicates. This note explains how programs may use capability observations in contracts without granting ambient authority to the contract checker.

## Pre-Spec Delta

This note should be reconciled into the target specs as follows:

- **SPEC-096 / SPEC-096b Effect System:** clarify that contract discharge and operation/capability discharge are separate row-item kinds. A contract predicate may consume values produced by capability operations, but it must not perform capability operations itself.
- **SPEC-097b Type System:** refine predicate well-formedness so authority-bearing calls are rejected, while ordinary boundary values carrying observation provenance may be bound and checked.
- **SPEC-098b Target IR:** add explicit observation/provenance sidecar metadata for capability-produced values used in contract diagnostics, and distinguish authority/admission diagnostics from contract diagnostics.
- **SPEC-099 Core Language:** clarify that runtime predicate evaluation is authority-free observer code over captured boundary values and observation evidence.
- **SPEC-100 Core Type Checking:** add the admission/order rule: capability observations are checked as ordinary program operations before contract predicates consume their results.

## 0. Motivation

NOTE-031 and NOTE-033 reject capability calls inside contract predicates:

```ash
fn load(path: Path) -> {PosixFs::read} String
    requires { PosixFs::exists(path) }
{
    ...
}
```

This rejection is correct. `PosixFs::exists(path)` is not a logical predicate in isolation. It is an authority-bearing operation: it requires an admitted provider, may fail for authority/runtime reasons, may observe changing external state, and may reveal information controlled by policy.

But programs still need to express facts learned through capabilities:

```ash
let exists = PosixFs::exists(path)
requires { exists == true }
```

The contract should be able to see the already-produced `exists` value. It should not gain filesystem authority merely because the programmer wrote a predicate. This note defines that boundary.

## 1. Core decision

Contracts are **authority-free observers**. Capabilities are **authority-bearing operations**.

Therefore:

1. A contract predicate must not perform, acquire, install, or dispatch a capability operation.
2. A contract predicate may inspect ordinary boundary values, including values previously produced by capability operations.
3. Values produced by capability operations may carry provenance/evidence metadata into contract diagnostics.
4. Capability admission failure, capability operation failure, predicate false, and predicate evaluator fault remain distinct diagnostic classes.
5. Recoverability is explicit: default contract false is structured bottom; recoverable capability or contract behavior must be represented through the relevant explicit row-accounted path.

The slogan:

```text
Observe with authority in program code.
Check without authority in contract code.
```

## 2. Four distinct failure classes

The boundary separates four cases that are easy to conflate:

| Class | Example | Meaning | Default mechanism |
|-------|---------|---------|-------------------|
| Authority/admission denial | no admitted `PosixFs::exists` provider | Program is not allowed to perform the operation | type/admission diagnostic or unhandled operation diagnostic |
| Capability operation failure | filesystem returns I/O error | Operation was allowed but failed operationally | operation-specific `Result`/`fail`/diagnostic per capability contract |
| Predicate false | `exists == true` evaluates to `false` | Contract condition is not satisfied | `Trap { reason: ContractViolation(...) }` by default |
| Predicate evaluator fault | admitted pure predicate helper traps | Predicate could not be evaluated | `Trap { reason: ContractPredicateFault(...) }` |

A contract diagnostic may mention that a value came from a capability observation. It must not reclassify capability failure as contract failure, and it must not turn predicate false into authority denial.

## 3. Observation-before-contract pattern

The accepted pattern is to bind authority-bearing observations before the contract boundary that consumes them:

```ash
fn checked_read(path: Path) -> {PosixFs::exists, PosixFs::read} String {
    let exists = PosixFs::exists(path)
    require { exists == true }
    PosixFs::read(path)
}
```

The exact surface spelling for local `require` remains owned by the relevant surface grammar. The semantic order is the important part:

```text
1. type-check / admit operation effect PosixFs::exists
2. execute or reason about the capability operation in ordinary program semantics
3. bind its result as an ordinary value
4. evaluate the contract predicate over that value without additional authority
```

A function-level precondition cannot itself perform the observation at call-entry unless the observation value is already part of the call boundary:

```ash
fn checked_read(path: Path, exists: Bool) -> {PosixFs::read} String
    requires { exists == true }
{
    PosixFs::read(path)
}
```

If the callee wants to compute `exists`, that is not a precondition of the call; it is ordinary body code followed by a local contract check or branch.

## 4. Predicate well-formedness rule

Extend the NOTE-031/NOTE-033 predicate judgment with an authority exclusion:

```text
Γp ⊢ pred ⇓ LoweredPredicate
```

requires:

```text
AuthorityFree(pred)
```

where `AuthorityFree` rejects:

- operation/capability calls;
- provider lookup or admission checks;
- role installation or role-dependent authority acquisition;
- handler installation/dispatch;
- process/workflow operations;
- time, randomness, environment, or global mutable state observations;
- implicit forcing of lazy/memo values outside a contract-owned observation boundary.

This is not merely an SMT limitation. Even a pure-looking capability wrapper is rejected if evaluating it requires authority or external observation.

## 5. Predicate functions and authority

NOTE-033 allows admitted predicate-function calls:

```text
PredicateCall { callee: PredicateFunctionRef, args: ... }
```

NOTE-034 restricts what can be admitted as a predicate function.

A predicate function is admissible only if its body/profile is authority-free under the predicate language:

```text
PredicateAdmissible(f) iff
  f is total/partial only through predicate-fault semantics,
  f has no operation/resource/role/policy/process/workflow/failure row requirements,
  f does not inspect ambient state,
  f does not require hidden provider bindings,
  f does not implicitly force delayed values outside the contract boundary.
```

Rejected:

```ash
pred fn path_exists(path: Path) -> Bool {
    PosixFs::exists(path)
}
```

Accepted, assuming `sorted` is an admitted pure observer over an already-present list value:

```ash
pred fn sorted(xs: List<Int>) -> Bool { ... }
```

For capability-backed domain facts, the programmer must split the observation from the predicate:

```ash
let exists = PosixFs::exists(path)
requires { path_allowed_by_policy(path) && exists }
```

where `path_allowed_by_policy` must itself be authority-free. If policy evaluation is authority-bearing, it too must happen before the predicate and pass an ordinary result value into the contract.

## 6. Observation provenance

When a capability result is used in a contract predicate, diagnostics should be able to explain where the observed value came from without granting the predicate evaluator authority.

Introduce sidecar metadata conceptually shaped as:

```rust
pub struct ObservationEvidence {
    pub id: ObservationId,
    pub op: EffectOp,
    pub boundary: BoundaryId,
    pub result: ValueRef,
    pub args: Vec<ObservedValue>,
    pub authority: AuthorityEvidenceRef,
    pub source_span: Span,
    pub policy: ObservationPolicy,
    pub replay: ReplayStatus,
}
```

This evidence is not an ordinary user value. It is diagnostic/audit metadata associated with a value or binding.

The `result` may later appear in a contract diagnostic as an observed value:

```rust
ObservedValue::Full(value_ref)
ObservedValue::Summary(summary)
ObservedValue::Redacted(reason)
ObservedValue::Unavailable(reason)
```

The contract diagnostic may say:

```text
requires { exists == true } failed
exists = false
exists was produced by PosixFs::exists(path) at <span>
path = <redacted by policy>
```

The diagnostic must respect the observation policy. Contracts do not bypass secrecy, provenance, or authority controls.

## 7. Type-level consequence

A capability-produced value has an ordinary type plus optional provenance sidecar:

```text
exists : Bool
ObservationEvidence(exists) = Some(...)
```

The predicate consumes only the ordinary value:

```text
Γp, exists: Bool ⊢ exists == true ⇓ LoweredPredicate
```

The diagnostic layer may consult the sidecar:

```text
ContractDiagnostic.observed_values includes exists and its observation evidence if policy permits
```

The predicate type checker must not require provenance to accept the predicate. Provenance improves diagnostics and auditability; it is not the truth-maker for the Boolean expression.

## 8. Operational consequence

Operationally, the order is:

```text
Raise/Call capability operation
  -> operation admission and handler/provider execution
  -> produce value or capability failure
  -> attach optional ObservationEvidence to produced value
  -> enter contract boundary
  -> evaluate LoweredPredicate over captured PredicateEnvironment
  -> false => ContractViolation
  -> predicate fault => ContractPredicateFault
```

The predicate evaluator receives no provider handle and no authority environment. If a `PredicateNode` would require such authority, the predicate should have been rejected before Core.

## 9. Recoverability boundary

Capability operations own their own recoverability model:

```ash
let exists_result = PosixFs::try_exists(path)  -- e.g. Result<Bool, FsError>
```

Contracts own their own recoverability model:

```text
false predicate -> ContractViolation trap by default
explicit recoverability -> row-accounted fail
```

Do not encode capability failure as `ContractViolation` merely because the failed operation fed a later contract. If the operation failed before producing `exists`, there is no `exists == true` predicate result yet.

## 10. Worked examples

### 10.1 Rejected: capability call in precondition

```ash
fn load(path: Path) -> {PosixFs::read} String
    requires { PosixFs::exists(path) }
{
    PosixFs::read(path)
}
```

Rejected before SMT/runtime checking:

```text
ContractPredicateRejected {
  reason: RequiresAuthority(PosixFs::exists),
  span: ...
}
```

The contract checker must not obtain filesystem authority.

### 10.2 Accepted: explicit observation value

```ash
fn load_checked(path: Path, exists: Bool) -> {PosixFs::read} String
    requires { exists == true }
{
    PosixFs::read(path)
}
```

Accepted. The caller is responsible for supplying `exists`. If `exists` was produced by a capability operation, provenance may be attached to that value by the caller-side code.

### 10.3 Accepted: body observation followed by local check

```ash
fn load_if_present(path: Path) -> {PosixFs::exists, PosixFs::read} String {
    let exists = PosixFs::exists(path)
    require { exists == true }
    PosixFs::read(path)
}
```

The authority-bearing observation happens as ordinary program code. The local contract consumes `exists` without re-observing the filesystem.

### 10.4 Capability failure before contract

```ash
let exists = PosixFs::exists(path)
require { exists == true }
```

If `PosixFs::exists(path)` fails operationally, that is capability failure. The contract has not evaluated yet. No `ContractViolation` should be produced for `exists == true` because there is no `exists` value.

### 10.5 Predicate false after successful observation

```text
PosixFs::exists(path) => false
requires { exists == true } => false
```

Now the operation succeeded and produced `false`. The predicate false is a contract violation. The diagnostic may include observation provenance if policy allows.

### 10.6 Redacted diagnostic

```text
requires { owner == current_user } failed
owner = <redacted>
owner was produced by PosixFs::metadata(path).owner
path = <redacted>
```

The contract failure remains real even when diagnostic values are redacted. Redaction affects reporting, not predicate truth.

### 10.7 Predicate function boundary

Rejected:

```ash
pred fn owner(path: Path) -> User {
    PosixFs::metadata(path).owner
}
```

Accepted if the metadata is already a value:

```ash
pred fn owner_matches(meta: FileMetadata, user: User) -> Bool {
    meta.owner == user
}
```

## 11. Relation to temporal contracts

GAP 5 will extend contracts over processes, channels, traces, and workflow obligations. NOTE-034 deliberately keeps the single-boundary rule conservative:

```text
Contract monitors may consume trace events and observation evidence.
They must not acquire authority to create those events.
```

Temporal monitoring therefore inherits the same split:

- operational code emits events under authority;
- monitors/contracts inspect admitted events under their monitor boundary;
- diagnostics preserve authority/provenance metadata without granting additional authority.

## 12. Design decisions

1. Contract predicates are authority-free observer code.
2. Capability observations must be explicit ordinary program operations before predicates consume their results.
3. Capability-produced values may carry observation provenance sidecars.
4. Contract diagnostics may include provenance subject to redaction policy.
5. Capability admission failure, capability operation failure, predicate false, and predicate evaluator fault are separate classes.
6. Predicate functions must be authority-free; capability-backed predicate helpers are rejected unless split into observation plus pure predicate.
7. Temporal contracts should consume trace/observation evidence, not acquire authority themselves.

## 13. Open questions

1. **Exact sidecar carrier.** Should `ObservationEvidence` attach to `ValueRef`, binding metadata, `ContractDiagnostic`, or a general provenance table keyed by value identity?
2. **Policy result boundary.** Some policy checks are themselves authority-bearing. The same observation-before-contract rule should apply, but the exact syntax for policy result values remains owned by the policy specs.
3. **Static proof with observation evidence.** A static proof may rely on an abstract value `exists: Bool`, but not on the truth of a runtime capability observation unless that observation is represented as an assumption/evidence item. The exact proof certificate shape is deferred.
4. **Temporal monitor authority.** GAP 5 must specify which runtime component is authorized to emit/collect trace events for monitor predicates.

## 14. References

### Internal references

- **NOTE-014 — Contract Systems Unification.** Source gap register; GAP 8 is resolved by this note.
  `docs/notes/NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md`
- **NOTE-024 — Host/FFI and Extern Boundary.** Defines the trusted host boundary that motivates keeping capability operations out of predicates.
  `docs/notes/NOTE-024-HOST-FFI-AND-EXTERN.md`
- **NOTE-031 — Contract Predicate Well-Formedness and Snapshots.** Establishes predicate classification and rejection of effectful/unstable predicates.
  `docs/notes/NOTE-031-CONTRACT-PREDICATE-WELL-FORMEDNESS-AND-SNAPSHOTS.md`
- **NOTE-033 — Surface-to-Core Contract Lowering.** Defines `LoweredPredicate`, `PredicateNode`, and `RuntimeCheckPlan`; NOTE-034 constrains their authority boundary.
  `docs/notes/NOTE-033-SURFACE-TO-CORE-CONTRACT-LOWERING.md`
- **SPEC-096b — Target Effect System.** Owns row item taxonomy and discharge boundaries.
  `docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md`
- **SPEC-097b — Target Type System.** Owns predicate well-formedness and row checking.
  `docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md`
- **SPEC-098b — Target IR.** Owns contract diagnostics, observed values, and IR sidecar records.
  `docs/spec/SPEC-098b-TARGET-IR.md`
- **SPEC-099 — Core Language.** Owns Core dynamic predicate evaluation boundaries.
  `docs/spec/SPEC-099-CORE-LANGUAGE.md`
- **SPEC-100 — Core Type Checking.** Owns obligation generation and dynamic contract strategy.
  `docs/spec/SPEC-100-CORE-TYPE-CHECKING.md`

### External references

- **Object-capability model.** Capability as unforgeable authority-bearing reference; useful prior art for separating possession/authority from ordinary predicate truth.
  https://en.wikipedia.org/wiki/Object-capability_model
- **W3C PROV Overview** (2013). Provenance vocabulary and document family for representing information about entities, activities, and agents involved in producing data.
  https://www.w3.org/TR/prov-overview/
- **SMT-LIB.** Background for authority-free logical obligations; SMT encodings should consume explicit values/assumptions rather than perform ambient observations.
  https://smt-lib.org/

## 15. Changelog

| Date       | Change |
|------------|--------|
| 2026-06-29 | Initial note. Resolves NOTE-014 GAP 8 by separating authority-bearing capability operations from authority-free contract predicates, defining the observation-before-contract pattern, observation provenance, diagnostic separation, predicate-function admission rule, and temporal-monitor inheritance rule. |
