# SPEC-004: Operational Semantics

## Status: Draft

## 1. Overview

Big-step operational semantics for the Ash workflow language. Tracks values, effects, traces, and provenance.

These rules define the meaning of the canonical core IR from SPEC-001. Surface syntax may carry
additional convenience forms, but those forms are only semantically relevant insofar as they lower
to the canonical core contract.

## 1.1 Runtime Authority and Advisory Interaction

This document defines canonical runtime meaning. The runtime owns authoritative state,
validation, rejection, commitment, trace, and provenance.

If an external reasoner participates in a workflow, its outputs remain advisory until the runtime
accepts them. That interaction is governed by separate interaction contracts and does not change
the canonical operational rules below.

This spec remains execution-neutral. It does not define reasoner projection mechanics, transport,
or any monitor/exposure behavior.

## 2. Semantic Domains

```
Value      ::= Int(i) | Float(f) | String(s) | Bool(b) | Null 
             | Time(t) | Ref(r) | List([v, ...]) | Record({k: v, ...})
             | Cap(c)
             | Variant(name, {k: v, ...})

Effect     ::= Epistemic | Deliberative | Evaluative | Operational
             -- Epistemic: input acquisition and read-only observation
             -- Deliberative: analysis, planning, and proposal formation
             -- Evaluative: policy and obligation evaluation
             -- Operational: external side effects and irreversible outputs

Trace      ::= ε | TraceEvent :: Trace

Provenance ::= Prov { id, parent, lineage, ... }

PolicyEnv  ::= PolicyName → Policy

ObligationState ::= Set(Obligation)

Error      ::= PolicyViolation(policy, v)
             | ObligationViolation(obligation)
             | GuardViolation(action, guard)
             | PatternBindFailure
             | PatternMatchFailure(v)
             | RuntimeFailure(reason)

Context    ::= Γ × C × P × Ω × π
  where Γ  = Variable → Value
        C  = Capability → Implementation
        P  = PolicyEnv
        Ω  = ObligationState
        π  = Provenance

WorkflowOutcome ::= Return(Value, Effect, Trace, ObligationState, Provenance)
                  | Reject(Error, Effect, Trace, ObligationState, Provenance)
```

Notation used below:

- `eff`, `eff1`, `eff2`, ... range over `Effect` values.
- `T`, `T1`, `T2`, ... range over traces.
- `P` ranges over policy environments.
- `Γ`, `Γ'`, `ΔΓ`, ... range over environments.
- `Ω`, `Ω'`, `Ω1`, `Ω2`, ... range over obligation states.
- `Return(...)` and `Reject(...)` name the authoritative workflow outcomes defined by the runtime.

Variant values are the canonical runtime representation for enum constructors. They store the
constructor name plus its named payload fields. The enclosing type name is not stored in the
runtime value itself.

The canonical runtime value domain does not store a separate tuple value. Tuple-shaped pattern
matching therefore operates over fixed-length `List` values.

## 2.1 Value Display Representation

When values are converted to strings for output (e.g., `ret` workflow results, printed output),
the following canonical representations are used:

| Value Type | Display Format | Example |
|------------|----------------|---------|
| `Int(i)` | Decimal integer | `42`, `-17` |
| `Float(f)` | Decimal floating-point | `3.14`, `-0.5` |
| `String(s)` | Raw string content | `hello` (not quoted) |
| `Bool(true)` | `"true"` | `true` |
| `Bool(false)` | `"false"` | `false` |
| `Null` | `"null"` | `null` |
| `List([v, ...])` | `[elem1, elem2, ...]` | `[1, 2, 3]` |
| `Record({k: v, ...})` | `{k1: v1, k2: v2}` | `{name: "x", val: 5}` |

**Design Rationale:**

- Boolean values use lowercase `"true"` and `"false"` (not "on"/"off") for consistency with language literals
- Strings display raw content without quotes to match user expectations for text output
- This representation is used for CLI output, logging, and debugging displays

## 2.2 Effect Order and Join

Effects form the following total order:

$$
  \mathrm{Epistemic} \le \mathrm{Deliberative} \le \mathrm{Evaluative} \le \mathrm{Operational}
$$

`eff1 ⊔ eff2` denotes the least upper bound of `eff1` and `eff2` in that order. Equivalently, it
is the maximum effect layer reached by either premise.

Consequences used throughout the rules:

- `Epistemic ⊔ eff = eff` whenever `eff` is at least epistemic.
- `Operational ⊔ eff = Operational` for every `eff`.
- Join is associative, commutative, and idempotent.

The explanatory layer comments in the domain declaration above are normative:

- `Epistemic` is input acquisition and read-only observation.
- `Deliberative` is analysis, planning, and proposal formation.
- `Evaluative` is policy and obligation evaluation.
- `Operational` is external side effects and irreversible outputs.

Rule schemata may use lowercase aliases `epistemic`, `deliberative`, `evaluative`, and
`operational` for those same four lattice elements.

## 2.3 Trace and Concatenation

`ε` denotes the empty trace. `T1 ++ T2` denotes trace concatenation in execution order: every event
in `T1` precedes every event in `T2`.

The trace algebra used by later rules assumes:

- `ε ++ T = T`
- `T ++ ε = T`
- `(T1 ++ T2) ++ T3 = T1 ++ (T2 ++ T3)`

Rules may still use event-prefix notation such as `Evt(...) :: T`; this is the singleton-prefix
presentation of the same trace domain.

## 2.4 Environment Extension and Shadowing

`Γ[x ↦ v]` denotes environment extension that binds `x` to `v`.

`Γ ⊕ ΔΓ` denotes right-biased environment extension by a finite binding set `ΔΓ`:

- every binding already in `Γ` remains available unless shadowed by `ΔΓ`;
- when both environments bind the same variable, the binding from `ΔΓ` wins;
- bindings introduced by a successful pattern match are fresh relative to one another.

When existing rules write `Γ ∪ ΔΓ`, read that form as this right-biased extension operator rather
than as set union without shadowing.

## 2.5 Expression and Pattern Domains

`Expr` ranges over the canonical core expression forms from SPEC-001. These are the pure,
read-oriented computations used by workflow rules for data construction, branching, and guard
conditions. Until guard lowering is normalized completely, some existing workflow rules still write
`eval_guard(...)` as a transitional boolean-valued helper over guard syntax; that helper is an
intermediate presentation device rather than a separate semantic domain.

`Pattern` ranges over the canonical binding and destructuring patterns from SPEC-001. Pattern
semantics are defined over runtime `Value`s and, on successful derivation, produce fresh bindings
for continuation evaluation. Pattern-owned dynamic failure categories are assigned at the enclosing
workflow or helper boundary rather than through a separate pattern-level rejection result.

This section establishes the judgment backbone for these domains. The complete pure-expression
rule family appears in §4.6 and is the single canonical source of expression meaning in this
document. The complete pure-pattern rule family appears in §4.7 and is the single canonical
source of pattern meaning in this document.

## 2.6 Runtime Failure Categories

Runtime rejections in this document fall into the following semantic categories:

- `PolicyViolation(policy, v)` for workflow-level `decide` denials.
- `ObligationViolation(obligation)` for workflow-level `check` failures.
- `GuardViolation(action, guard)` for `act` guard failures.
- `PatternBindFailure` and `PatternMatchFailure(v)` for pattern-owned binding or match failure.
- `RuntimeFailure(reason)` for runtime-boundary failures such as missing capabilities, provider
  failures, mailbox/runtime selection failures, or invalid control/runtime state.

These categories preserve runtime authority: parser, lowering, and static type-check failures are
outside this operational layer.

## 2.7 Propagation Conventions

Unless a rule states otherwise, premises are read and discharged top-to-bottom. For rule schemata
with several recursive or helper premises, that order is the normative left-to-right evaluation
order of the semantic presentation.

Propagation follows these conventions:

- if an earlier recursive workflow premise yields `Reject(err, eff, T, Ω', π')`, later premises in
  the same rule are not evaluated;
- any local trace event or base effect incurred before the failing premise remains visible in the
  propagated result as a prefix contribution or effect join, as specified by the enclosing rule;
- unless a rule performs an explicit local state update before the failing premise, the propagated
  rejection keeps the obligation/provenance state returned by the failing premise itself;
- success-only subjudgments such as `Γ ⊢e expr ⇓ v` and `Γ ⊢p pat ⇐ v ⇓ ΔΓ` do not introduce a
  second rejection channel; their dynamic failure ownership remains at the enclosing workflow or
  helper boundary.
- concurrent rules such as `PAR` are the exception to this sequential early-propagation scheme:
  branch outcomes are combined by the rule- or helper-level concurrent combination contract rather
  than by left-to-right short-circuiting.

These conventions prevent later premises from observing partial progress that the rule has not made
semantically visible.

## 2.8 Lookup Failure Conventions

Lookup-like helper failures are owned by the first workflow or helper boundary that requires the
lookup result.

Unless a more specific pattern-owned failure is stated, a missing or unusable lookup target maps to
`RuntimeFailure(reason)`. This includes, for example:

- missing capability or provider bindings;
- missing policy bindings or runtime policy handles;
- mailbox, scheduler, or control-runtime lookup failures;
- helper-owned field, variant, or runtime-resource lookup failures not already classified under the
  pattern or pure-expression misuse rules.

This convention fixes the semantic class of lookup failure without over-specifying implementation
error text.

## 2.9 Post-Lowering Assumptions

The rules in this document describe the canonical post-lowering core language from SPEC-001 rather
than surface syntax directly.

Accordingly, the semantics assume that parsing, lowering, and static checking have already ruled
out ordinary front-end failures such as malformed syntax, unresolved sugar, and statically obvious
type errors. If a malformed state nevertheless reaches runtime, the semantics must not become
stuck. Instead, the first owning boundary maps it into one of the declared runtime failure classes:

- inadmissible binding patterns map to `PatternBindFailure`;
- exhausted runtime `match` search maps to `PatternMatchFailure(v)` for the evaluated scrutinee;
- other malformed dynamic states that survive lowering or validation map to `RuntimeFailure(reason)`.

## 3. Judgment Backbone

The semantics use explicit subjudgments rather than a single undifferentiated evaluation relation.
This section introduces the normative contracts for those judgments; later tasks align the full
rule families to these contracts.

### 3.1 Workflow Big-Step Judgment

```text
Γ, C, P, Ω, π ⊢wf w ⇓ out

out ::= Return(v, eff, T, Ω', π')
      | Reject(err, eff, T, Ω', π')
```

Reads: In context (Γ, C, P, Ω, π), workflow w evaluates to:

- either a normal return value `v` or a runtime rejection `err`
- the greatest effect layer `eff` reached during evaluation
- the trace `T` emitted in execution order
- the updated obligation state `Ω'`
- the updated provenance `π'`

Unless a rule states otherwise, a rejected recursive premise propagates according to the
conventions in §2.7, after prefixing any local trace event and joining any effect already incurred
before that premise.

For readability, some later rule schemata still use a compact tuple presentation for workflow
outcomes. Read

```text
Γ, C, P, Ω, π ⊢ w ⇓ v, eff, T, Ω', π'
```

as shorthand for

```text
Γ, C, P, Ω, π ⊢wf w ⇓ Return(v, eff, T, Ω', π')
```

and read tuple conclusions with trailing `error: err` as shorthand for the corresponding
`Reject(err, eff, T, Ω', π')` outcome.

Unless a rule says otherwise, this document specifies post-verification execution only.
Capability-verification outcomes such as `Deny`, `Transform`, and `RequireApproval` at concrete
`observe`, `receive`, `set`, `send`, and `act` sites remain owned by SPEC-017 and SPEC-018; the
rules below describe the execution branch after the runtime has admitted the operation for
execution.

The evaluation relation is execution-neutral:

- it is not a specification for a tree-walking interpreter,
- it is not a specification for bytecode execution,
- it is not a specification for a future JIT,
- it is the contract that all such execution strategies must preserve.

### 3.2 Expression Evaluation Judgment

```text
Γ ⊢e expr ⇓ v
```

Reads: under environment `Γ`, expression `expr` evaluates to runtime value `v`.

This is the pure/core expression judgment used by workflow premises such as `let`, `if`,
`orient`, and `decide`, and by `match` guards once guard syntax has been lowered to canonical
expressions. The complete canonical rule family for core `Expr` forms appears in §4.6.

`Γ ⊢e expr ⇓ v` is a success judgment only: it defines the value of pure expressions when
evaluation is dynamically well-formed. If a workflow reaches a dynamically ill-shaped expression
site, rejection ownership remains at the enclosing workflow boundary as described in §4.4 and
§4.6.1 rather than introducing a second expression-level error channel.

`act` guards remain written as `eval_guard(Γ, guard)` in the current workflow rules as a
transitional helper relation for guard-specific boolean checking. That notation is not yet a full,
separate semantic family; later normalization may align it with the expression judgment once guard
lowering is specified explicitly.

### 3.3 Pattern Matching Judgment

```text
Γ ⊢p pat ⇐ v ⇓ ΔΓ
```

Reads: under base environment `Γ`, pattern `pat` matches runtime value `v` and yields the fresh
binding environment `ΔΓ`. Continuations then evaluate under `Γ ⊕ ΔΓ`.

`Γ ⊢p pat ⇐ v ⇓ ΔΓ` is a success-only judgment. For admissible patterns, failure to derive this
judgment means branch-local non-match rather than an implicit rejection result. The complete
canonical rule family appears in §4.7.

Duplicate binders are not ordinary non-match: a pattern whose binder set is not fresh relative to
itself is invalid, and any enclosing site that attempts to use it must reject with
`PatternBindFailure` rather than treating it as a branch miss. Ordinary pattern-owned failures
still map to `PatternBindFailure` or `PatternMatchFailure(v)` at the enclosing workflow or
expression site rather than through a separate hidden error channel.

### 3.4 Helper Relations

Helper relations abstract runtime-owned or algebraic operations that are semantically relevant but
not themselves workflow forms. They are written schematically as:

```
helper(args...) ↝ result
```

At this stage, helper relations include contracts such as capability lookup, receive-arm
selection, obligation discharge, obligation join, provenance extension, provenance join, and trace
merge. Their role is to constrain observable outcomes without over-specifying runtime internals.
When a helper can
fail, that failure must map into one of the runtime failure categories in §2.6 or into an explicit
pattern-owned failure.

For the pattern-sensitive helpers used below:

- `require_pattern(Γ, pat, v) ↝ ΔΓ` succeeds exactly when `admissible(pat)` holds and the
  canonical judgment `Γ ⊢p pat ⇐ v ⇓ ΔΓ` derives. Otherwise it fails as `PatternBindFailure`.
- `select_match_arm(Γ, v, arms) ↝ (ΔΓ, body)` selects the first arm in declaration order whose
  pattern is admissible, whose match derives under `⊢p`, and whose optional guard evaluates to
  `Bool(true)`. If any arm is inadmissible, the helper fails as `PatternBindFailure`. If all
  admissible arms fail to match or have false guards, it fails as `PatternMatchFailure(v)`.
- `select_receive_outcome(...)` and `combine_parallel_outcomes(...)` are helper-backed runtime
  relations whose full contracts live normatively in §6.2 and §6.5.

## 4. Inference Rules

### 4.1 Epistemic Layer

```
(OBSERVE)
  lookup(C, cap) ↝ impl
  impl.execute(Γ) ↝ v
  require_pattern(Γ, pat, v) ↝ ΔΓ
  Γ ⊕ ΔΓ, C, P, Ω, π ⊢ cont ⇓ v', eff, T, Ω', π'
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢ OBSERVE cap as pat in cont ⇓ v',
               epistemic⊔eff,
               Obs(cap, v, now()) :: T,
               Ω',
               π'

(OBSERVE-LOOKUP-FAIL)
  lookup(C, cap) ↝ error reason
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢ OBSERVE cap as pat in cont ⇓ ⊥,
               epistemic,
               ε,
               Ω,
               π,
               error: RuntimeFailure(reason)

(OBSERVE-BIND-FAIL)
  lookup(C, cap) ↝ impl
  impl.execute(Γ) ↝ v
  require_pattern(Γ, pat, v) ↝ error PatternBindFailure
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢ OBSERVE cap as pat in cont ⇓ ⊥,
               epistemic,
               Obs(cap, v, now()) :: ε,
               Ω,
               π,
               error: PatternBindFailure
```

**Properties**:

- Effect is at most epistemic
- Value bound to pattern
- Observation recorded in trace

```
ReceiveOutcome ::= Selected(msg, ΔΓ, body, τr)
                 | Fallback(body, τr)
                 | Fallthrough(τr)
                 | ReceiveReject(err, τr)

(RECEIVE-SELECTED)
  select_receive_outcome(mode, control, source_scheduling_modifier, arms, Γ)
    ↝ Selected(msg, ΔΓ, body, τr)
  Γ ⊕ ΔΓ, C, P, Ω, π ⊢ body ⇓ v, eff, T, Ω', π'
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢ RECEIVE mode control { arms } ⇓ v,
               epistemic⊔eff,
               τr ++ T,
               Ω',
               π'

(RECEIVE-FALLBACK)
  select_receive_outcome(mode, control, source_scheduling_modifier, arms, Γ)
    ↝ Fallback(body, τr)
  Γ, C, P, Ω, π ⊢ body ⇓ v, eff, T, Ω', π'
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢ RECEIVE mode control { arms } ⇓ v,
               epistemic⊔eff,
               τr ++ T,
               Ω',
               π'

(RECEIVE-FALLTHROUGH)
  select_receive_outcome(mode, control, source_scheduling_modifier, arms, Γ)
    ↝ Fallthrough(τr)
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢ RECEIVE mode control { arms } ⇓ Null,
               epistemic,
               τr,
               Ω,
               π

(RECEIVE-REJECT)
  select_receive_outcome(mode, control, source_scheduling_modifier, arms, Γ)
    ↝ ReceiveReject(err, τr)
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢ RECEIVE mode control { arms } ⇓ ⊥,
               epistemic,
               τr,
               Ω,
               π,
               error: err
```

`RECEIVE` is the mailbox-input form. Its base effect is `epistemic` because it only selects from
already-arrived workflow input; blocking and timeout behavior are determined by `mode` rather than
by a higher effect classification.

The selection relation searches according to SPEC-013: it probes declared stream mailboxes or the
implicit control mailbox under the current source scheduling modifier, then selects the oldest
queued entry whose arm matches and whose guard succeeds. It is not a single-message poll followed
by separate arm testing.

Canonical `RECEIVE` runtime behavior:

The bullet summary below is explanatory only. The normative receive-selection laws are the helper
contract in §6.2 plus the `RECEIVE-*` rules above.

- `receive { ... }` uses the runtime scheduler and the current source scheduling modifier to select
  a source mailbox, then checks arms in declaration order. Pattern matching happens before guard
  evaluation; a message is removed from the mailbox only after the selected arm's guard succeeds.
  If no source yields a match, `_` runs if present; otherwise control falls through to the next
  workflow step with no error.
- `receive wait { ... }` uses the same source-selection and arm-order model, but blocks until a
  matching event is available, then runs the first matching arm.
- `receive wait DURATION { ... }` uses one timeout budget for the whole receive operation. It
  blocks until a matching event arrives or the budget expires. On timeout, `_` runs if present;
  otherwise control falls through with no error.
- `receive control ... { ... }` polls only the implicit control mailbox and does not consume normal
  stream events.

### 4.2 Deliberative Layer

```
(ORIENT)
  Γ ⊢e expr ⇓ v
  analyze(v) ↝ v'
  Γ, C, P, Ω, π ⊢ cont ⇓ v'', eff, T, Ω', π'
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢ ORIENT expr in cont ⇓ v'',
               deliberative⊔eff,
               Orient(expr, v, v', now()) :: T,
               Ω',
               π'
```

```
(PROPOSE)
  Γ, C, P, Ω, π ⊢ cont ⇓ v, eff, T, Ω', π'
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢ PROPOSE action in cont ⇓ v,
               deliberative⊔eff,
               Propose(action, Γ, now()) :: T,
               Ω',
               π'
  
  [Note: PROPOSE does not execute, only records intent]
```

### 4.3 Evaluative Layer

```
(DECIDE-PERMIT)
  Γ ⊢e expr ⇓ v
  policy_decision(P, policy, v, Γ) = PolicyDecision::Permit
  Γ, C, P, Ω, π ⊢ cont ⇓ v', eff, T, Ω', π'
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢ DECIDE expr under policy in cont ⇓ v',
               evaluative⊔eff,
               Decide(policy, Permit, v, now()) :: T,
               Ω',
               π'

(DECIDE-LOOKUP-FAIL)
  Γ ⊢e expr ⇓ v
  policy_decision(P, policy, v, Γ) ↝ error reason
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢ DECIDE expr under policy in cont ⇓ ⊥,
               evaluative,
               ε,
               Ω,
               π,
               error: RuntimeFailure(reason)

(DECIDE-DENY)
  Γ ⊢e expr ⇓ v
  policy_decision(P, policy, v, Γ) = PolicyDecision::Deny
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢ DECIDE expr under policy in cont ⇓ ⊥,
               evaluative,
               Decide(policy, Deny, v, now()) :: ε,
               Ω,
               π,
               error: PolicyViolation(policy, v)
```

`DECIDE` is the workflow-level policy gate and therefore always names an explicit lowered policy binding. It consumes the same normalized policy representation used by capability verification, but only admits `Permit` and `Deny` outcomes at the workflow layer. Capability-level checks may still be applied at concrete `observe`, `receive`, `set`, `send`, or `act` operations by the capability-verification runtime, which may also consume `RequireApproval` or `Transform` outcomes.

The `DECIDE` judgment models workflow-level policy gates only. Capability-verification outcomes
such as `RequireApproval` and `Transform` are owned by SPEC-017 and SPEC-018, and verification
warnings are separate metadata rather than runtime `PolicyDecision` values.

```
(CHECK-SATISFIED)
  check_obligation(obligation, Ω, Γ) ↝ true
  discharge(Ω, obligation) = Ω'
  Γ, C, P, Ω', π ⊢ cont ⇓ v, eff, T, Ω'', π'
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢ CHECK obligation in cont ⇓ v,
               evaluative⊔eff,
               Oblig(obligation, true, now()) :: T,
               Ω'',
               π'

(CHECK-VIOLATED)
  check_obligation(obligation, Ω, Γ) ↝ false
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢ CHECK obligation in cont ⇓ ⊥,
               evaluative,
               Oblig(obligation, false, now()) :: ε,
               Ω,
               π,
               error: ObligationViolation(obligation)
```

`CHECK` evaluates obligations only; policies are not valid `CHECK` targets.

### 4.4 Rejection Boundaries

Runtime and evaluation reject:

- policy denials at workflow `decide` sites
- obligation violations at `check` sites
- guard failures at `act` sites
- missing or unavailable runtime capabilities and providers
- mailbox or provider failures that prevent a receive arm or action from completing at runtime
- provider-level input/output mismatches that arise from actual runtime values
- invalid control operations caused by missing authority, unknown instance state, or terminally
  shut down instances

These are runtime boundary failures. They are not parser or lowering failures, and they are not
type-checking failures once the type layer has validated the relevant shapes.

Timeout expiry and receive fallthrough remain normal control flow under the canonical `RECEIVE`
contract; they are not runtime rejections by themselves.

Control authority is reusable unless an operation is terminal. In particular, health checks,
pause, and resume do not by themselves consume or invalidate a valid `ControlLink`; terminal
control such as kill invalidates future control operations for the target instance.

While the owning runtime state remains alive, terminally controlled instances remain retained as
runtime-owned tombstones rather than being silently forgotten. Later control attempts therefore
continue to fail as terminal-control runtime failures, not as unknown-link failures caused by
background cleanup in the same runtime state.

### 4.5 Operational Layer

```
(ACT)
  eval_guard(Γ, guard) = true
  policy_check(P, action, Γ) ↝ Permit
  perform_action(action, Γ, C) ↝ v
  π' = extend_provenance(π, action, guard, v)
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢ ACT action where guard ⇓ v,
               operational,
               Act(action, v, guard, now()) :: ε,
               Ω,
               π'

(ACT-POLICY-FAIL)
  eval_guard(Γ, guard) = true
  policy_check(P, action, Γ) ↝ error reason
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢ ACT action where guard ⇓ ⊥,
               operational,
               ε,
               Ω,
               π,
               error: RuntimeFailure(reason)

(ACT-RUNTIME-FAIL)
  eval_guard(Γ, guard) = true
  policy_check(P, action, Γ) ↝ Permit
  perform_action(action, Γ, C) ↝ error reason
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢ ACT action where guard ⇓ ⊥,
               operational,
               ε,
               Ω,
               π,
               error: RuntimeFailure(reason)

(ACT-GUARD-FAIL)
  eval_guard(Γ, guard) = false
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢ ACT action where guard ⇓ ⊥,
               operational,
               GuardFail(action, guard, now()) :: ε,
               Ω,
               π,
               error: GuardViolation(action, guard)
```

These `ACT` rules likewise describe execution after capability verification has admitted the
operation. Verification-time `Deny`, `Transform`, and `RequireApproval` outcomes remain defined by
SPEC-017 and SPEC-018 rather than by this operational semantics layer.

### 4.6 Expression Evaluation

`Γ ⊢e expr ⇓ v` is the single canonical pure-expression semantics in this document. Evaluation is
deterministic and reads subexpressions left-to-right wherever a form contains multiple
subexpressions.

The rule schemata below use direct runtime-value notation where convenient. In particular,
`List([...])` and `Record({...})` appear only as value-side notation inside literal carriers or
projection premises; they are not additional canonical `Expr` forms beyond the SPEC-001 core.

When a core form requires operator- or runtime-owned details, the expression judgment delegates
to a helper contract rather than duplicating that lower-level behavior inline. In particular:

- `project_index(vbase, vidx) ↝ v` is the pure indexing/projection helper for canonical
  `IndexAccess` expressions;
- `apply_unary(op, v) ↝ v'` covers canonical unary operators such as `Not` and `Neg`;
- `apply_binary(op, v1, v2) ↝ v` covers canonical binary operators other than the explicitly
  short-circuiting `And`/`Or` cases and the separately spelled-out structural equality rules;
- `eval_pure_call(vf, [v1, ..., vn]) ↝ v` covers canonical pure calls for the `Call` form after
  the callee expression itself has been evaluated.

If any of those helper contracts is undefined at runtime for the evaluated arguments, the
enclosing workflow boundary owns rejection as described in §4.6.1.

```
(EXPR-LITERAL)
  literal ∈ LiteralCarrier
  ─────────────────────────────────────────────────────────────────
  Γ ⊢e Literal(literal) ⇓ literal

(EXPR-VAR)
  Γ(x) = v
  ─────────────────────────────────────────────────────────────────
  Γ ⊢e Variable(x) ⇓ v

(EXPR-VARIANT)
  Γ ⊢e expr1 ⇓ v1
  ...
  Γ ⊢e exprn ⇓ vn
  ─────────────────────────────────────────────────────────────────
  Γ ⊢e Constructor(name, {k1: expr1, ..., kn: exprn})
       ⇓ Variant(name, {k1: v1, ..., kn: vn})

(EXPR-FIELD)
  Γ ⊢e expr ⇓ Record({..., field: v, ...})
  ─────────────────────────────────────────────────────────────────
  Γ ⊢e FieldAccess(expr, field) ⇓ v

(EXPR-INDEX)
  Γ ⊢e expr ⇓ vbase
  Γ ⊢e index ⇓ vidx
  project_index(vbase, vidx) ↝ v
  ─────────────────────────────────────────────────────────────────
  Γ ⊢e IndexAccess(expr, index) ⇓ v

(EXPR-UNARY)
  Γ ⊢e expr ⇓ v
  apply_unary(op, v) ↝ v'
  ─────────────────────────────────────────────────────────────────
  Γ ⊢e Unary(op, expr) ⇓ v'

(EXPR-EQ-TRUE)
  Γ ⊢e lhs ⇓ v1
  Γ ⊢e rhs ⇓ v2
  v1 = v2
  ─────────────────────────────────────────────────────────────────
  Γ ⊢e Binary(Eq, lhs, rhs) ⇓ Bool(true)

(EXPR-EQ-FALSE)
  Γ ⊢e lhs ⇓ v1
  Γ ⊢e rhs ⇓ v2
  v1 ≠ v2
  ─────────────────────────────────────────────────────────────────
  Γ ⊢e Binary(Eq, lhs, rhs) ⇓ Bool(false)

(EXPR-BINARY)
  Γ ⊢e lhs ⇓ v1
  Γ ⊢e rhs ⇓ v2
  apply_binary(op, v1, v2) ↝ v
  op ∉ {Eq, And, Or}
  ─────────────────────────────────────────────────────────────────
  Γ ⊢e Binary(op, lhs, rhs) ⇓ v

(EXPR-AND-FALSE)
  Γ ⊢e lhs ⇓ Bool(false)
  ─────────────────────────────────────────────────────────────────
  Γ ⊢e Binary(And, lhs, rhs) ⇓ Bool(false)

(EXPR-AND-TRUE)
  Γ ⊢e lhs ⇓ Bool(true)
  Γ ⊢e rhs ⇓ Bool(b)
  ─────────────────────────────────────────────────────────────────
  Γ ⊢e Binary(And, lhs, rhs) ⇓ Bool(b)

(EXPR-OR-TRUE)
  Γ ⊢e lhs ⇓ Bool(true)
  ─────────────────────────────────────────────────────────────────
  Γ ⊢e Binary(Or, lhs, rhs) ⇓ Bool(true)

(EXPR-OR-FALSE)
  Γ ⊢e lhs ⇓ Bool(false)
  Γ ⊢e rhs ⇓ Bool(b)
  ─────────────────────────────────────────────────────────────────
  Γ ⊢e Binary(Or, lhs, rhs) ⇓ Bool(b)

(EXPR-CALL)
  Γ ⊢e callee ⇓ vf
  Γ ⊢e arg1 ⇓ v1
  ...
  Γ ⊢e argn ⇓ vn
  eval_pure_call(vf, [v1, ..., vn]) ↝ v
  ─────────────────────────────────────────────────────────────────
  Γ ⊢e Call(callee, [arg1, ..., argn]) ⇓ v

(EXPR-MATCH)
  Γ ⊢e scrutinee ⇓ v
  select_match_arm(Γ, v, [arm1, ..., armn]) ↝ (ΔΓ, body)
  Γ ⊕ ΔΓ ⊢e body ⇓ v'
  ─────────────────────────────────────────────────────────────────
  Γ ⊢e Match(scrutinee, [arm1, ..., armn]) ⇓ v'
```

`LiteralCarrier` ranges over the runtime literal values admitted by the canonical `Literal` form,
including scalar literals and literal containers already fully formed before expression
evaluation. `Expr::Constructor` from SPEC-001 is the canonical variant-construction form; this
section therefore subsumes the earlier standalone constructor prose. Together, the rules above
cover the canonical core expression forms listed in SPEC-001: `Literal`, `Variable`,
`FieldAccess`, `IndexAccess`, `Unary`, `Binary`, `Call`, `Match`, and `Constructor`.

`EXPR-MATCH` defines `Match` in terms of the explicit expression and pattern judgments already
declared in §3 together with the helper contract for arm selection. It selects the first arm whose
pattern matches and whose optional guard evaluates to `Bool(true)`, then evaluates that arm body
in the extended environment `Γ ⊕ ΔΓ`. An earlier inadmissible arm is not a branch miss: it causes
the enclosing site to reject with `PatternBindFailure` as stated in §4.6.1.

#### 4.6.1 Dynamic Expression Misuse Ownership

The expression judgment does not introduce a separate rejection result. When a workflow evaluates
an expression at a dynamically ill-shaped site, the enclosing workflow rule owns the rejection:

- unknown variables, missing fields, or non-boolean operands for boolean connectives are rejected
  by the enclosing workflow boundary as `RuntimeFailure(reason)`;
- invalid indexing, unsupported unary/binary operator applications, or undefined pure-call helper
  resolution are likewise rejected by the enclosing workflow boundary as `RuntimeFailure(reason)`;
- an inadmissible pattern at a required-binding site or in any `Match` arm is rejected by the
  enclosing site as `PatternBindFailure` rather than treated as branch-local non-match;
- a matched arm whose guard evaluates to a non-boolean value is rejected by the enclosing
  workflow boundary as `RuntimeFailure(reason)`;
- a `Match` expression with no selected arm is rejected by the enclosing workflow boundary as
  `PatternMatchFailure(v)` for the already-evaluated scrutinee value `v`;
- the expression judgment itself remains the success-only relation for the pure fragment.

This ownership split keeps pure expression reasoning canonical in one place while preserving the
runtime-authoritative rejection model from §4.4.

### 4.7 Pattern Matching

`Γ ⊢p pat ⇐ v ⇓ ΔΓ` is the single canonical pattern semantics in this document. It is defined only
for admissible patterns and is deterministic on that domain.

Pattern-side conventions used by the rules below:

- `∅` is the empty binding environment.
- `dom(ΔΓ)` is the set of variable names bound by `ΔΓ`.
- `admissible(pat)` holds exactly when `binders(pat)` is fresh relative to itself, so the pattern
  contains no duplicate binders.
- In any successful composite match, the environments produced by subpatterns must have pairwise
  disjoint domains. This is the operational form of the freshness requirement from §2.4.
- Freshness is internal to one successful pattern match. Because continuations run under
  `Γ ⊕ ΔΓ`, a fresh binder in `ΔΓ` may still shadow an older binding already present in `Γ`.
- `binders(pat)` is the multiset of names syntactically bound by `pat`, including an optional list
  rest binder when present. A pattern with duplicate binders is invalid and must be rejected by the
  enclosing site as `PatternBindFailure`; it is not merely a branch-local non-match.
- If an implementation surface permits `_` in the optional list-rest position, treat it as a
  wildcard rest that discards the suffix and contributes no binding to `ΔΓ`.
- If a concrete implementation surface spells a unit variant pattern without an explicit field map,
  read that spelling as the zero-field canonical form shown below.

The rule family distinguishes branch-local non-match from runtime rejection:

- A missing derivation for one `PAT-*` rule because of a literal mismatch, constructor mismatch,
  missing field, or incompatible value shape is a local non-match only.
- `Match` expressions use such non-matches to continue searching later arms.
- Required-binding sites such as `LET`, `OBSERVE`, and receive-arm selection map invalid patterns
  or missing required matches to `PatternBindFailure`.
- Exhausting every `match` arm without a selected branch yields `PatternMatchFailure(v)` for the
  already-evaluated scrutinee `v`, as stated in §4.6.1.

```
(PAT-WILDCARD)
  ─────────────────────────────────────────────────────────────────
  Γ ⊢p Wildcard ⇐ v ⇓ ∅

(PAT-BIND)
  ─────────────────────────────────────────────────────────────────
  Γ ⊢p Variable(x) ⇐ v ⇓ [x ↦ v]

(PAT-LIT)
  literal = v
  ─────────────────────────────────────────────────────────────────
  Γ ⊢p Literal(literal) ⇐ v ⇓ ∅

(PAT-TUPLE)
  Γ ⊢p p1 ⇐ v1 ⇓ ΔΓ1
  ...
  Γ ⊢p pn ⇐ vn ⇓ ΔΓn
  dom(ΔΓi) ∩ dom(ΔΓj) = ∅ for all i ≠ j
  ─────────────────────────────────────────────────────────────────
  Γ ⊢p Tuple([p1, ..., pn]) ⇐ List([v1, ..., vn]) ⇓ ΔΓ1 ⊕ ... ⊕ ΔΓn

(PAT-LIST)
  Γ ⊢p p1 ⇐ v1 ⇓ ΔΓ1
  ...
  Γ ⊢p pn ⇐ vn ⇓ ΔΓn
  dom(ΔΓi) ∩ dom(ΔΓj) = ∅ for all i ≠ j
  ─────────────────────────────────────────────────────────────────
  Γ ⊢p List([p1, ..., pn], None) ⇐ List([v1, ..., vn]) ⇓ ΔΓ1 ⊕ ... ⊕ ΔΓn

(PAT-LIST-REST)
  m ≥ n
  Γ ⊢p p1 ⇐ v1 ⇓ ΔΓ1
  ...
  Γ ⊢p pn ⇐ vn ⇓ ΔΓn
  dom(ΔΓi) ∩ dom(ΔΓj) = ∅ for all i ≠ j
  rest ∉ dom(ΔΓ1 ⊕ ... ⊕ ΔΓn)
  ΔΓrest = [rest ↦ List([v(n+1), ..., vm])]
  ─────────────────────────────────────────────────────────────────
  Γ ⊢p List([p1, ..., pn], Some(rest)) ⇐ List([v1, ..., vm])
    ⇓ ΔΓ1 ⊕ ... ⊕ ΔΓn ⊕ ΔΓrest

(PAT-RECORD)
  lookup(k1, record) = v1
  ...
  lookup(kn, record) = vn
  Γ ⊢p p1 ⇐ v1 ⇓ ΔΓ1
  ...
  Γ ⊢p pn ⇐ vn ⇓ ΔΓn
  dom(ΔΓi) ∩ dom(ΔΓj) = ∅ for all i ≠ j
  ─────────────────────────────────────────────────────────────────
  Γ ⊢p Record({k1: p1, ..., kn: pn}) ⇐ Record(record) ⇓ ΔΓ1 ⊕ ... ⊕ ΔΓn

(PAT-VARIANT)
  lookup_variant_name(v) = name
  lookup_variant_field(v, k1) = v1
  ...
  lookup_variant_field(v, kn) = vn
  Γ ⊢p p1 ⇐ v1 ⇓ ΔΓ1
  ...
  Γ ⊢p pn ⇐ vn ⇓ ΔΓn
  dom(ΔΓi) ∩ dom(ΔΓj) = ∅ for all i ≠ j
  ─────────────────────────────────────────────────────────────────
  Γ ⊢p Variant(name, {k1: p1, ..., kn: pn}) ⇐ v ⇓ ΔΓ1 ⊕ ... ⊕ ΔΓn
```

`PAT-TUPLE` and `PAT-LIST` both consume runtime `List(...)` values, but they describe distinct
canonical pattern constructors from SPEC-001. `PAT-TUPLE` requires exact list length because tuple
patterns are fixed-arity destructors; `PAT-LIST-REST` is the only rule that admits suffix capture.
If the rest position is spelled `_`, read `PAT-LIST-REST` with `ΔΓrest = ∅`.

`PAT-RECORD` and `PAT-VARIANT` use field-subset matching: the runtime value may contain fields not
mentioned in the pattern, but every field named by the pattern must be present and must match its
subpattern. `PAT-VARIANT` additionally requires exact constructor-name equality.

### 4.8 Control Flow

```
(SEQ)
  Γ, C, P, Ω, π ⊢ w1 ⇓ v1, eff1, T1, Ω1, π1
  Γ, C, P, Ω1, π1 ⊢ w2 ⇓ v2, eff2, T2, Ω2, π2
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢ SEQ w1 w2 ⇓ v2, eff1⊔eff2, T1 ++ T2, Ω2, π2

(PAR)
  ∀i. Γ, C, P, Ω, fork(π) ⊢wf wi ⇓ outi
  combine_parallel_outcomes([out1, ..., outn], π) ↝ out
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢wf PAR [w1, ..., wn] ⇓ out

  [Concurrent branch rejection is owned by `combine_parallel_outcomes(...)` rather than by
   left-to-right short-circuit propagation; see §2.7.]

(IF-TRUE)
  Γ ⊢e cond ⇓ Bool(true)
  Γ, C, P, Ω, π ⊢ then_branch ⇓ v, eff, T, Ω', π'
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢ IF cond then else ⇓ v, eff, T, Ω', π'

(IF-FALSE)
  Γ ⊢e cond ⇓ Bool(false)
  Γ, C, P, Ω, π ⊢ else_branch ⇓ v, eff, T, Ω', π'
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢ IF cond then else ⇓ v, eff, T, Ω', π'

(LET)
  Γ ⊢e expr ⇓ v
  require_pattern(Γ, pat, v) ↝ ΔΓ
  Γ ⊕ ΔΓ, C, P, Ω, π ⊢ cont ⇓ v', eff, T, Ω', π'
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢ LET pat = expr in cont ⇓ v', eff, T, Ω', π'

(LET-BIND-FAIL)
  Γ ⊢e expr ⇓ v
  require_pattern(Γ, pat, v) ↝ error PatternBindFailure
  ─────────────────────────────────────────────────────────────────
  Γ, C, P, Ω, π ⊢ LET pat = expr in cont ⇓ ⊥,
               epistemic,
               ε,
               Ω,
               π,
               error: PatternBindFailure
```

### 4.9 Recoverable Result Handling

Recoverable failures are represented explicitly as `Result` values in the canonical language.
Workflows handle recoverable failures by pattern matching on `Ok` / `Err` values, and the meaning
of that matching is given by the ADT `Match` semantics in SPEC-020 and the canonical `EXPR-MATCH`
rule in §4.6.

Examples of recoverable handling are therefore written as explicit `Result` construction and
`Match`.

### 4.10 Expression-Level Surface Convenience Notes

The following expression-level constructs may appear in the surface language or parser
conveniences, but they are not additional semantic families:

- `if let` is shorthand for canonical matching behavior with a wildcard fallback arm
- surface-only spellings do not expand the set of semantic laws
- implementation convenience nodes may exist internally, but they must not change the core meaning

```

## 5. Auxiliary Functions

### 5.1 Historical Note on Legacy `bind(...)` Notation

This subsection is non-normative. The only normative pattern semantics are the `⊢p` judgment in
§3.3 and the `PAT-*` rules in §4.7.

If an older note or review comment still writes `bind(pat, v, Γ)`, read it only as shorthand for
first deriving `Γ ⊢p pat ⇐ v ⇓ ΔΓ` and then continuing under `Γ ⊕ ΔΓ`. It does not define any
additional per-pattern behavior, failure mode, or competing source of truth.

### 5.2 Effect Join

`⊔` in all workflow and helper rules means the effect join defined normatively in §2.2. This
section is only a reminder that later uses of `⊔` are references back to that definition; it does
not add a second equation set or any additional laws.

## 6. Helper Contract Summary

The rules above cite helpers abstractly. This section gives the minimum normative contract for
their domain, range, determinism status, failure mapping, and required semantic laws.

### 6.1 Capability and Policy Lookup

**Capability lookup**

```text
lookup(C, capability_ref) ↝ provider | error reason
```

- Domain: runtime capability registry `C` plus the lowered capability reference required by the
  owning workflow form.
- Range: a provider handle admitted by the capability-verification runtime.
- Determinism: deterministic for a fixed runtime context.
- Failure mapping: missing or unusable capability bindings map to `RuntimeFailure(reason)` at the
  first owning workflow boundary, as in `OBSERVE-LOOKUP-FAIL` and the runtime-failure branch of
  `perform_action(...)` used by `ACT-RUNTIME-FAIL`.
- Laws: lookup does not mutate trace, obligations, or provenance; its success result must satisfy
  the capability-availability contracts from [SPEC-017](SPEC-017-CAPABILITY-INTEGRATION.md) and
  [SPEC-018](SPEC-018-CAPABILITY-MATRIX.md).

**Policy decision**

```text
policy_decision(P, policy, v, Γ) ↝ decision | error reason
```

- Domain: lowered policy environment `P`, named policy handle `policy`, subject value `v`, and the
  current environment `Γ`.
- Range: `PolicyDecision::Permit` or `PolicyDecision::Deny` at the workflow layer.
- Determinism: deterministic for a fixed lowered policy environment and subject value.
- Failure mapping: missing or unusable runtime policy bindings map to `RuntimeFailure(reason)` at
  the `DECIDE` boundary.
- Laws: workflow-level `DECIDE` consumes only the lowered policy representation and must remain
  consistent with capability-verification ownership in [SPEC-018](SPEC-018-CAPABILITY-MATRIX.md).

**Action policy check**

```text
policy_check(P, action, Γ) ↝ decision | error reason
```

- Domain: lowered policy environment `P`, lowered action reference `action`, and current
  environment `Γ`.
- Range: the workflow-facing action policy decision required by `ACT`.
- Determinism: deterministic for a fixed runtime policy environment and action/environment pair.
- Failure mapping: missing or unusable runtime policy state maps to `RuntimeFailure(reason)` at the
  first owning workflow boundary.
- Laws: `policy_check(...)` must stay consistent with the runtime verification matrix in
  [SPEC-018](SPEC-018-CAPABILITY-MATRIX.md) and must not bypass the action-performance or guard
  checks owned separately by `ACT`.

### 6.2 Receive Selection

```text
select_receive_outcome(mode, control, source_scheduling_modifier, arms, Γ) ↝ outcome
```

where

```text
outcome ::= Selected(msg, ΔΓ, body, τr)
          | Fallback(body, τr)
          | Fallthrough(τr)
          | ReceiveReject(err, τr)
```

- Domain: receive mode, optional control-only selector, the current source scheduling modifier,
  lowered receive arms, and environment `Γ`.
- Range: one authoritative receive outcome for the whole receive operation.
- Determinism: scheduler-defined but constrained by [SPEC-013](SPEC-013-STREAMS.md); the helper may
  be nondeterministic only where the source scheduling modifier permits multiple valid source
  choices.
- Failure mapping: inadmissible receive-arm patterns yield `ReceiveReject(PatternBindFailure, τr)`;
  mailbox/runtime failures yield `ReceiveReject(RuntimeFailure(reason), τr)`.
- Laws:
  - selected-arm bindings must come from the canonical `⊢p` judgment;
  - admissible non-match keeps searching later arms or later eligible sources;
  - message consumption happens only after pattern match and guard success;
  - fallback and fallthrough semantics must match [SPEC-013](SPEC-013-STREAMS.md).

### 6.3 Action Performance

```text
perform_action(action, Γ, C) ↝ v | error reason
```

- Domain: lowered action capability reference, current environment `Γ`, and capability registry `C`.
- Range: the runtime value returned by the provider.
- Determinism: runtime-defined; may be nondeterministic when the underlying provider is
  nondeterministic.
- Failure mapping: provider/runtime execution failures map to `RuntimeFailure(reason)` unless a more
  specific workflow rule already owns the failure category.
- Laws:
  - the helper does not bypass policy or guard checks owned by the enclosing `ACT` rule;
  - on success, the returned value is the one recorded in the corresponding `Act(...)` trace event;
  - provider-specific behavior remains abstract, but observable success/failure must respect
    [SPEC-017](SPEC-017-CAPABILITY-INTEGRATION.md) and [SPEC-018](SPEC-018-CAPABILITY-MATRIX.md).

### 6.4 Obligation Checking

```text
check_obligation(obligation, Ω, Γ) ↝ bool
discharge(Ω, obligation) ↝ Ω'
```

- Domain: named obligation, obligation state `Ω`, and current environment `Γ` for checking, plus
  obligation state `Ω` for discharge.
- Range: a boolean satisfaction result and an updated obligation state.
- Determinism: deterministic for fixed inputs.
- Failure mapping: missing or malformed runtime obligation state maps to `RuntimeFailure(reason)`;
  an unmet obligation at the workflow layer maps to `ObligationViolation(obligation)` through the
  owning `CHECK` rule.
- Laws:
  - successful discharge updates only the named obligation slot;
  - failed discharge does not silently consume unrelated obligations;
  - repeated checking behavior must remain consistent with the obligation-state model already named
    in SPEC-004.

### 6.5 Parallel Outcome Combination

```text
combine_parallel_outcomes([out1, ..., outn], π) ↝ out
```

- Domain: the list of branch workflow outcomes plus the parent provenance seed `π`.
- Range: one authoritative workflow `out` for the enclosing `PAR` form.
- Determinism: deterministic except for any permitted trace interleaving chosen by `merge_traces`.
- Failure mapping: if any branch rejects, the helper returns a combined `Reject(...)` outcome rather
  than relying on sequential short-circuit propagation.
- Laws:
  - all-success case returns list-valued branch results with joined effect `⊔ effi`;
  - trace combination preserves each branch's internal order;
  - obligation aggregation uses `join_obligations(...)`, which deterministically computes the
    combined obligation state from branch-local obligation states without inventing unrelated
    obligations;
  - provenance aggregation uses `join_provenance(...)` rooted at the incoming `π`.

**Obligation join**

```text
join_obligations(Ω1, ..., Ωn) ↝ Ω'
```

- Domain: the branch-local obligation states produced by concurrent evaluation.
- Range: one combined obligation state `Ω'` for the enclosing parallel form.
- Determinism: deterministic for fixed branch-local obligation states.
- Failure mapping: impossible or malformed obligation-state combinations map to
  `RuntimeFailure(reason)` at the owning parallel-outcome boundary.
- Laws: the result preserves every obligation state transition already made visible by a branch and
  does not invent unrelated obligations.

### 6.6 Provenance and Trace Helpers

```text
fork(π) ↝ πfork
extend_provenance(π, action, guard, value) ↝ π'
join_provenance(π, [π1, ..., πn]) ↝ π'
merge_traces([T1, ..., Tn]) ↝ T
```

- Domain: current provenance/trace values and the helper-specific action or branch inputs.
- Range: updated provenance objects or merged traces.
- Determinism:
  - `extend_provenance` is deterministic;
  - `fork` and `join_provenance` are deterministic up to fresh identity generation;
  - `merge_traces` may be nondeterministic only in the choice of interleaving, subject to the
    order-preservation law.
- Failure mapping: helper misuse or impossible runtime provenance state maps to
  `RuntimeFailure(reason)` at the owning boundary.
- Laws:
  - `fork` creates a fresh child provenance whose parent is the input provenance identifier;
  - `extend_provenance` preserves prior lineage and appends the new action record;
  - `join_provenance` preserves every input lineage as an ancestor in the result;
  - `merge_traces` preserves the internal order of each input trace while allowing implementation-
    defined interleaving across distinct traces.

  ## 7. Determinism and Nondeterminism

  This section classifies which parts of the semantics are intended to be deterministic, which parts
  admit controlled nondeterminism, and which equalities should be read modulo freshness or trace
  interleaving rather than literal syntactic identity.

  ### 7.1 Deterministic Fragment

  The following fragment is deterministic for fixed runtime inputs, helper results, and initial
  state:

  - the pure expression judgment `Γ ⊢e expr ⇓ v` from §4.6;
  - the pure pattern judgment `Γ ⊢p pat ⇐ v ⇓ ΔΓ` from §4.7 on admissible patterns;
  - sequential workflow composition that does not invoke runtime-defined nondeterministic helpers;
  - helper relations explicitly marked deterministic in §6.

  In particular, the canonical pure/core fragment is intended to support proofs of uniqueness of
  derived result values, bindings, effects, and failure classes whenever the helper contracts it uses
  are themselves deterministic.

  ### 7.2 Permitted Nondeterminism

  The semantics intentionally permits nondeterminism only at explicit runtime-owned boundaries:

  - receive selection when the current source scheduling modifier permits more than one eligible
    source or message choice;
  - action performance when the underlying runtime/provider is nondeterministic;
  - provenance freshness (`fork`, fresh identifiers) up to freshness-preserving equivalence;
  - parallel trace interleaving and concurrent branch aggregation under `PAR` and its helper-backed
    combination rules.

  No other semantic family may introduce additional nondeterminism without extending the canonical
  helper contracts that own it.

  ### 7.3 Determinism Modulo Freshness and Interleaving

  Some equalities in this document should be read modulo observationally irrelevant variation:

  - provenance values may differ by fresh identifiers while preserving the same parent/lineage
    structure and action-history content;
  - merged traces may differ by interleaving across concurrent branches while preserving each input
    trace's internal order;
  - helper-backed runtime choices may differ only where the relevant helper contract explicitly
    permits such variation.

  These are not semantic contradictions. They are the intended equivalence classes for proofs and
  conformance arguments over runtime-backed executions.

  ## 8. Semantic Invariants

  The rules and helper contracts above are intended to preserve the following global invariants.

  ### 8.1 No-Stuck Runtime Boundary

  Post-lowering canonical states must not become semantically stuck. If evaluation cannot continue,
  the owning rule or helper boundary must classify the situation as one of:

  - `PatternBindFailure`
  - `PatternMatchFailure(v)`
  - `PolicyViolation(policy, v)`
  - `ObligationViolation(obligation)`
  - `GuardViolation(action, guard)`
  - `RuntimeFailure(reason)`

  This invariant is the runtime-side restatement of §§2.7–2.9.

  ### 8.2 Effect and Trace Monotonicity

  Evaluation may accumulate effects and trace entries, but it does not retract already visible
  semantic progress:

  - later effects are joined with earlier effects using `⊔`;
  - in sequential propagation, local trace prefixes remain prefixes of propagated outcomes; in
    helper-backed concurrent combination, the weaker but still normative requirement is the
    branch-order preservation law stated for `merge_traces(...)` in §6.6;
  - rejected results still preserve any effect or trace contribution already made visible by the
    owning rule.

  ### 8.3 Environment and Binding Freshness

  Successful pattern evaluation yields fresh bindings relative to the pattern itself, and
  continuations evaluate under right-biased extension `Γ ⊕ ΔΓ`.

  Consequences:

  - duplicate binders are invalid and never count as branch-local non-match;
  - fresh pattern bindings may shadow older bindings already present in `Γ`;
  - helper-backed binding sites must use the same admissibility and freshness rules as the canonical
    `⊢p` judgment.

  ### 8.4 Rejection Ownership and Failure Classification

  Each dynamic failure class has one owning semantic boundary:

  - pure expression and pattern judgments are success-only;
  - enclosing workflow rules or helper-backed boundaries own their rejection classification;
  - helper failures must map into the declared runtime failure classes rather than inventing new
    ad hoc categories.

  This invariant supports proof obligations about uniqueness of failure class and conformance across
  different implementations.

  ### 8.5 Helper-Law Preservation

  Every helper-backed rule relies on the laws declared in §6. Implementations conform to SPEC-004
  only if those helper laws remain true observationally:

  - receive selection preserves pattern-before-guard ordering and fallback/fallthrough behavior;
  - parallel combination preserves branch-local trace order and obligation/provenance aggregation
    laws;
  - provenance helpers preserve lineage ancestry;
  - lookup and policy helpers preserve the declared failure classifications.

  ## 9. Proof Targets and Conformance

  This section makes the proof-facing reading of SPEC-004 explicit.

  ### 9.1 Proof Targets for the Big-Step Core

  The first proof targets local to SPEC-004 should be judgment-shaped:

  1. **Expression determinism**: the pure expression judgment yields a unique result value when its
     helper dependencies are deterministic.
  2. **Pattern determinism and freshness**: admissible patterns yield unique binding environments and
     preserve the duplicate-binder/freshness invariants.
  3. **Failure ownership**: dynamic failure states are classified by the owning workflow/helper
     boundary and do not create hidden extra error channels.
  4. **Effect/trace soundness**: derived effects and traces satisfy the monotonicity conventions from
     §8.2.
  5. **Parallel conformance modulo helper laws**: `PAR` preserves the aggregation laws declared for
     `combine_parallel_outcomes`, `join_obligations`, `join_provenance`, and `merge_traces`.

  ### 9.2 Conformance Obligations for Runtime Implementations

  An interpreter, VM, reference evaluator, or future JIT conforms to SPEC-004 only if it preserves:

  - the workflow outcome shapes from §3.1;
  - the pure expression and pattern judgments from §§4.6–4.7;
  - the propagation, lookup-failure, and post-lowering ownership conventions from §§2.7–2.9;
  - the helper contracts from §6, including permitted nondeterminism only where declared.

  Execution strategy is intentionally left open. Observable conformance matters; internal evaluator
  architecture does not.

  ### 9.3 Conformance Obligations for Helper-Backed Relations

  Runtime-owned helper implementations may vary internally, but they must preserve the externally
  observable laws declared in SPEC-004 and any tighter contracts imported from adjacent specs such as
  [SPEC-013](SPEC-013-STREAMS.md), [SPEC-017](SPEC-017-CAPABILITY-INTEGRATION.md), and
  [SPEC-018](SPEC-018-CAPABILITY-MATRIX.md).

  In particular:

  - helper implementations may refine operational detail but may not reclassify failure ownership;
  - helper implementations may introduce implementation-private bookkeeping but may not change the
    declared effect, trace, obligation, or provenance laws;
  - proofs over the canonical big-step core may treat compliant helper implementations as any model
    satisfying these contracts.

  ## 10. Property Testing

```rust
// Trace completeness: every action is recorded
proptest! {
    #[test]
    fn prop_trace_completeness(w in arbitrary_workflow()) {
        let result = interpret(w);
        for action in w.actions() {
            assert!(result.trace.contains_action(action));
        }
    }
}

// Effect correctness: observed effect ≥ actual effect
proptest! {
    #[test]
    fn prop_effect_soundness(w in arbitrary_workflow()) {
        let predicted = infer_effect(&w);
        let result = interpret(w);
        assert!(predicted >= result.effect);
    }
}

// Determinism: same input → same output (for pure workflows)
proptest! {
    #[test]
    fn prop_determinism(w in arbitrary_pure_workflow()) {
        let result1 = interpret(w.clone());
        let result2 = interpret(w);
        assert_eq!(result1.value, result2.value);
    }
}

// Provenance integrity: lineage is acyclic
proptest! {
    #[test]
    fn prop_provenance_acyclic(w in arbitrary_workflow()) {
        let result = interpret(w);
        assert!(is_acyclic(result.provenance.lineage));
    }
}
```

## 11. Related Documents

- SPEC-001: IR
- SPEC-003: Type System
