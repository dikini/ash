# NOTE-029: Structured Bottom and Contract Diagnostics

**Date:** 2026-06-28
**Status:** Living document — design direction captured; resolves NOTE-014 GAP 6
**Purpose:** Define contract failure as structured bottom. A default dynamic contract failure
is not an ordinary resumable effect and does not add a `ContractViolation` row item. It traps
with a rich `ContractDiagnostic`. Recoverable contract behavior is still possible, but it must
be explicit: the lowering uses a `fail` effect and row-accounts that failure path.

Companion to NOTE-014 (contract systems unification), NOTE-027 (blame and subsumption),
NOTE-028 (purity, evaluation modes, and contract timing), SPEC-096b (failure effects),
SPEC-098b (Target IR), SPEC-099 (Core language), and SPEC-100 (Core type checking).

## Pre-Spec Delta

This note is pre-spec and resolves NOTE-014 §12 GAP 6. The target specs already contain the
most important boundary condition: `ContractViolation` is not a row item and not a raised
operation by default. When the project moves this note into normative specs, reconcile:

- **SPEC-098b Target IR:** extend `TrapReason::ContractViolation` from
  `ContractViolation(ContractEffect)` to `ContractViolation(ContractDiagnostic)` or to a
  compact diagnostic reference. Extend `ContractDischarge` with blame/discharge metadata from
  NOTE-027.
- **SPEC-099 Core language:** preserve the current rule that default dynamic Hoare failure
  lowers to `RecordDischarge` plus `Trap { reason: ContractViolation(...) }`. Add the full
  diagnostic payload shape and memo replay semantics.
- **SPEC-100 Core type checking:** keep `Trap(reason)` checking at any expected type with row
  `{}`. Specify that recoverable contract behavior requires explicit lowering to `fail` and a
  corresponding failure row item.
- **SPEC-096b Effect system:** clarify that `fail` is the recoverable failure mechanism for
  contract recovery. `ContractViolation` itself remains diagnostic bottom metadata unless a
  surface construct explicitly maps it into `fail`.

## 0. Motivation

NOTE-014 GAP 6 identified a missing boundary: contract failure was described both as
`TrapReason::ContractViolation` and as a raised `Failure`/`ContractViolation`-like operation.
That muddied two distinct cases:

1. **Default contract failure** is terminal structured bottom. It aborts the current path and
   carries a rich diagnostic payload.
2. **Recoverable contract failure** is an explicit failure effect. A surface construct may
   choose this behavior, but then the effect row must say so.

The distinction is important because contract failure is not ordinary domain failure. A failed
precondition or postcondition means the program has violated a proof obligation. The runtime
must preserve enough information to explain that violation, but the type system should not
silently turn proof failure into an arbitrary resumable effect.

## 1. Core decision

```text
Default dynamic contract failure is structured bottom:
  Trap { reason: ContractViolation(ContractDiagnostic) }

Recoverable contract failure is explicit failure:
  Raise { op: fail ContractError, ... }
  and the function row includes {fail ContractError}

ContractViolation is not a row item.
ContractViolation is not implicitly resumable.
ContractViolation diagnostics are preserved across traps, memo replay, and handler decisions.
```

This matches SPEC-099 and SPEC-100: a dynamic Hoare check lowers to a predicate test plus
contract discharge metadata. If the predicate fails unrecoverably, the program traps. If a
surface construct chooses recoverability, it must lower to an explicit `fail` operation.

## 2. The diagnostic payload

A contract violation is structured bottom, not a string. The diagnostic payload must preserve
all information needed for blame, audit, replay, and debugging.

```rust
pub struct ContractDiagnostic {
    pub contract: ContractEffect,
    pub predicate: PredicateRef,
    pub contract_text: String,
    pub source_span: Span,
    pub blame: BlameLabel,
    pub observed_values: Vec<ObservedValue>,
    pub call_chain: Vec<CallFrame>,
    pub discharge_history: DischargeHistory,
    pub handler_history: Vec<HandlerDecision>,
    pub replay: ReplayStatus,
}

pub struct ObservedValue {
    pub name: String,
    pub type_name: String,
    pub value_repr: String,
    pub capture_policy: CapturePolicy,
}

pub enum CapturePolicy {
    Full,
    Redacted,
    Omitted,
}

pub struct CallFrame {
    pub module_path: String,
    pub function_name: String,
    pub source_span: Span,
}

pub struct DischargeHistory {
    pub declared_mode: DischargeMode,
    pub actual_mode: DischargeMode,
    pub demoted_from_static: bool,
    pub evidence: Option<EvidenceRef>,
}

pub struct HandlerDecision {
    pub handler_name: String,
    pub action: HandlerAction,
    pub source_span: Span,
}

pub enum HandlerAction {
    Propagate,
    RecoverViaFail,
    ResumeWithDefault,
    Escape,
}

pub enum ReplayStatus {
    Original,
    MemoReplay,
}
```

`BlameLabel` comes from NOTE-027. It identifies the party and polarity: caller for
`requires`, callee/impl for `ensures`, and boundary-specific blame for `invariant`.

### 2.1 Value capture is policy-governed

Diagnostics should include actual values when that is safe and useful. They must not leak
secrets by default. `CapturePolicy` is therefore part of the diagnostic, not an afterthought.
A value can be fully captured, redacted, or omitted. The predicate text and source span remain
available even when values are redacted.

This keeps contract diagnostics useful without turning contract failure into a secret-exfiltration
channel.

## 3. Lowering semantics

### 3.1 Default unrecoverable lowering

A dynamic `requires` check lowers to a runtime predicate test plus discharge metadata. On
failure, it traps.

```text
RecordDischarge {
  discharge: ContractDischarge {
    contract: requires(P),
    mode: Dynamic,
    blame: BlameLabel { party: Caller, polarity: Negative, ... },
    source_span: span(P),
  },
  body:
    if not(P(args)) {
      Trap { reason: ContractViolation(ContractDiagnostic { ... }) }
    } else {
      body
    }
}
```

Typing rule:

```text
Trap(reason) checks at any expected type and has row {}
```

The trap is bottom: it produces no value, so it can inhabit any expected type in the typing
judgment. It does not add `{ContractViolation}` to the row because it is not a resumable
operation.

### 3.2 Recoverable lowering requires explicit `fail`

A surface construct may ask for recoverable contract behavior. That must lower to an explicit
failure effect:

```text
RecordDischarge {
  discharge: ContractDischarge { contract: requires(P), mode: Dynamic, ... },
  body:
    if not(P(args)) {
      Raise { op: fail ContractError, args: [ContractDiagnostic { ... }], row: {fail ContractError} }
    } else {
      body
    }
}
```

The function type must expose that failure:

```ash
fn checked_div(a: Int, b: Int) -> {fail ContractError} Int
    dynamic requires: b != 0 recoverable
{
    a / b
}
```

The exact surface keyword (`recoverable` above) is illustrative. The normative rule is the
lowering boundary: recoverability is explicit and row-accounted.

### 3.3 No implicit `Result`, `Option`, or default value

A contract violation must not implicitly become `None`, `Err`, an empty list, or a default
value. Those are domain-level encodings. If a program wants to recover into a domain value, it
must install an explicit failure handler or use a surface construct that lowers through `fail`.

This preserves the difference between a false contract and ordinary expected failure.

## 4. Trap, fail, and handler boundaries

### 4.1 Trap is terminal for the current path

`Trap { reason: ContractViolation(diagnostic) }` terminates the current evaluation path. It
is not caught by ordinary effect handlers because it is not a `Raise`. The surrounding runtime,
workflow boundary, debugger, test harness, or crash reporter can observe the diagnostic.

This is the correct default for violated proof obligations.

### 4.2 `fail` is recoverable and row-accounted

`fail ContractError` is an ordinary failure effect from SPEC-096b. It is discharged by an
enclosing failure handler, workflow failure boundary, or failure policy.

```ash
fn parse_config(path: String) -> {PosixFs::read, fail ConfigError} Config { ... }
```

A recoverable contract check uses this same mechanism. The diagnostic can be wrapped in a
contract-specific failure type, but the row item is still `fail ...`, not
`ContractViolation`.

### 4.3 Handler decisions are diagnostic history

A handler or failure boundary can decide to propagate, recover, resume with a default, or
escape. That decision is recorded in `handler_history`; it does not rewrite the original
blame label or erase the original diagnostic.

```text
original violation:
  blame = caller myapp::payments::process_payment
  action = Original

failure handler maps diagnostic to Err(...):
  handler_history += RecoverViaFail(handler = with_contract_errors)

memo force later replays the same failure:
  replay = MemoReplay
  blame unchanged
  observed_values unchanged
```

## 5. Memo and lazy replay semantics

NOTE-028 defines contract timing for delayed computations. NOTE-029 defines what is replayed
when the terminal outcome is a contract failure.

### 5.1 Lazy

A lazy thunk re-runs its body on every force. If the contract check fails, each force produces
a fresh `ContractDiagnostic` for that evaluation. The blame label still points to the original
provider/caller/callee according to NOTE-027.

```text
force lazy_x #1 → ContractDiagnostic(replay = Original)
force lazy_x #2 → ContractDiagnostic(replay = Original)  -- new evaluation, new diagnostic
```

The two diagnostics may have different call chains or observed values if the environment has
changed.

### 5.2 Memo

A memo thunk records its first terminal outcome. If the first force fails with a contract
violation, later forces replay the same diagnostic with `replay = MemoReplay` or with a
separate replay event pointing to the original diagnostic ID.

```text
force memo_x #1 → ContractDiagnostic(id = D, replay = Original), cache failure D
force memo_x #2 → replay D (replay = MemoReplay)
```

The replay must preserve:

- original blame label;
- original observed values, subject to capture policy;
- original discharge history;
- original source span and predicate text.

It may add a replay event to the audit trail. It must not create a new blame event.

## 6. Worked examples

### 6.1 Default trap

Proposed surface example:

```ash
fn safe_div(a: Int, b: Int) -> Int
    dynamic requires: b != 0
{
    a / b
}
```

If `safe_div(10, 0)` runs, the runtime takes the default path:

```text
Trap {
  reason: ContractViolation(ContractDiagnostic {
    contract: Requires(b != 0),
    blame: BlameLabel { party: Caller, polarity: Negative, ... },
    observed_values: [a = 10, b = 0],
    replay: Original,
  })
}
```

The function type does not include `{ContractViolation}`. The failure is terminal bottom for
that path.

### 6.2 Explicit recoverable contract failure

Illustrative proposed surface:

```ash
fn safe_div_result(a: Int, b: Int) -> {fail ContractError} Int
    dynamic requires: b != 0 recoverable
{
    a / b
}
```

On failure, this does not trap. It raises the explicit failure effect:

```text
Raise {
  op: fail ContractError,
  args: [ContractDiagnostic { contract: Requires(b != 0), ... }],
  row: {fail ContractError}
}
```

A caller must handle or expose `{fail ContractError}`. Recovery is visible in the type.

### 6.3 Memo replay of a contract violation

```ash
let memo q = safe_div(10, 0);
```

First force:

```text
force q
  → Trap ContractViolation(Diagnostic D)
  → memo cell records terminal failure D
```

Second force:

```text
force q
  → replay terminal failure D
  → same blame, same predicate, same observed values
  → audit may record MemoReplay(D)
```

This preserves referential transparency for memoized failures: the same memoized computation
observes the same terminal outcome.

## 7. Relation to existing gaps

### 7.1 GAP 6 resolved

GAP 6 asked what diagnostic information survives contract failure and where the boundary lies
between trap and recoverable effect. NOTE-029 resolves it:

- default contract failure is structured bottom (`Trap`);
- the diagnostic payload survives as `ContractDiagnostic`;
- recoverability requires explicit `fail` and row accounting;
- memo replay preserves the original diagnostic and blame.

### 7.2 GAP 7 resolved in NOTE-032

Blame soundness and optimizer soundness are now stated as part of NOTE-032's five
meta-obligations. NOTE-029 specifies the diagnostic data that must be preserved for those
obligations to be meaningful; NOTE-032 states the obligations over that data.

### 7.3 GAP 5 still open

Proc/Workflow temporal contracts may need monitoring diagnostics that span multiple events,
roles, and processes. NOTE-029 covers Pure/Act-level contract failure. Temporal diagnostic
aggregation remains part of GAP 5.

## 8. Open questions

1. **Exact recoverable surface syntax.** This note uses `recoverable` illustratively. The
   normative decision is the lowering: recoverability must lower to `fail` and row-account the
   failure. The surface syntax can be designed later.

2. **Diagnostic value redaction policy.** Which values are safe to capture by default? The
   type system may need secrecy annotations or a runtime policy. Until then, capture policy
   should default conservatively.

3. **Diagnostic identity and storage.** Memo replay can either clone the diagnostic with
   `replay = MemoReplay` or store a diagnostic ID and replay by reference. The latter is better
   for audit trails but needs a diagnostic store.

4. **Invariant boundary blame.** NOTE-027 left invariant blame boundary-specific. NOTE-029
   carries that label but does not complete the invariant polarity model.

## 9. Working Principle

```text
Contract failure is structured bottom by default.
Default dynamic Hoare failure lowers to Trap { reason: ContractViolation(ContractDiagnostic) }.
Trap checks at any expected type and has row {}.
ContractViolation is not a row item and not implicitly resumable.
Recoverable contract behavior must lower to explicit fail and expose {fail ...} in the row.
ContractDiagnostic preserves predicate, source span, blame, observed values, call chain,
discharge history, handler history, and replay status.
Handler/failure-boundary decisions append diagnostic history; they do not rewrite blame.
Lazy failures produce a fresh diagnostic on each force.
Memo failures replay the first terminal diagnostic; replay is not a new blame event.
```

## 10. References

Internal references:

- [NOTE-014: Contract Systems Unification](NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md) — GAP 6
  (contract failure observability and bottom behavior)
- [NOTE-027: Contract Blame and Subsumption](NOTE-027-CONTRACT-BLAME-AND-SUBSUMPTION.md) —
  blame labels, handler decision history, `ContractDiagnostic` seed
- [NOTE-028: Purity, Evaluation Modes, and Contract Timing](NOTE-028-PURITY-EVALUATION-MODES-AND-CONTRACT-TIMING.md)
  — lazy/memo contract timing and memo replay rule
- [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md) — failure
  effects (`fail`) and failure boundaries
- [SPEC-098b: Target CPS IR](../spec/SPEC-098b-TARGET-IR.md) — `ContractDischarge`,
  `TrapReason`, `Raise`
- [SPEC-099: Core Language](../spec/SPEC-099-CORE-LANGUAGE.md) — dynamic contract check
  lowering to `RecordDischarge` plus `Trap`, recoverable path via explicit `fail`
- [SPEC-100: Core Type Checking](../spec/SPEC-100-CORE-TYPE-CHECKING.md) — dynamic contract
  strategy and `Trap(reason)` typing

External references:

- Findler, Robert Bruce; Felleisen, Matthias. "Contracts for Higher-Order Functions" (2002).
  Background for blame labels and contract diagnostics. https://doi.org/10.1145/581478.581484
- Dimoulas, Christos; Findler, Robert Bruce; Flatt, Matthew; Felleisen, Matthias. "Correct
  Blame for Contracts: No More Scapegoating" (2012). Blame soundness background.
  https://doi.org/10.1145/2103621.2103697
- Peyton Jones, Simon; Marlow, Simon; Elliott, Conal. "Stretching the Storage Manager: Weak
  Pointers and Stable Names in Haskell" (1999). Relevant to memo/replay identity and runtime
  diagnostic stores. https://doi.org/10.1007/3-540-48515-5_14

## 11. Changelog

- 2026-06-28: Initial version. Resolves NOTE-014 GAP 6. Defines contract failure as
  structured bottom by default: `Trap { reason: ContractViolation(ContractDiagnostic) }`.
  Clarifies that `ContractViolation` is not a row item and not implicitly resumable; explicit
  recoverability lowers to `fail` and row-accounts the failure. Defines `ContractDiagnostic`
  payload, value capture policy, handler decision history, lazy fresh diagnostics, and memo
  replay of cached terminal failures.
- 2026-06-29: Cross-referenced NOTE-032. GAP 7 is now resolved by explicit meta-level
  obligations; NOTE-029 remains the diagnostic-data substrate for blame and optimizer
  soundness.
