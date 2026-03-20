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
Value      ::= Int(i) | String(s) | Bool(b) | Null 
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

Context    ::= Γ × C × Ω × π
  where Γ  = Variable → Value
        C  = Capability → Implementation
        Ω  = Set(Obligation)
        π  = Provenance

Result     ::= Ok(Value, Effect, Trace, Provenance)
             | Err(Error, Trace, Provenance)
```

Variant values are the canonical runtime representation for enum constructors. They store the
constructor name plus its named payload fields. The enclosing type name is not stored in the
runtime value itself.

## 3. Big-Step Judgment

```
Γ, C, Ω, π ⊢ w ⇓ v, ε, T, π'

Reads: In context (Γ, C, Ω, π), workflow w evaluates to:
  - value v
  - accumulated effect ε
  - trace T
  - updated provenance π'

The evaluation relation is execution-neutral:

- it is not a specification for a tree-walking interpreter,
- it is not a specification for bytecode execution,
- it is not a specification for a future JIT,
- it is the contract that all such execution strategies must preserve.
```

## 4. Inference Rules

### 4.1 Epistemic Layer

```
(OBSERVE)
  lookup(C, cap) = impl
  impl.execute(Γ) ↝ v
  Γ' = bind(pat, v, Γ)
  Γ', C, Ω, π ⊢ cont ⇓ v', ε, T, π'
  ─────────────────────────────────────────────────────────────────
  Γ, C, Ω, π ⊢ OBSERVE cap as pat in cont ⇓ v', 
               epistemic⊔ε,
               Obs(cap, v, now()) :: T,
               π'
```

**Properties**:
- Effect is at most epistemic
- Value bound to pattern
- Observation recorded in trace

```
(RECEIVE)
  select_receive_match(mode, control, source_scheduling_modifier, arms, Γ) ↝ (msg, Γ', body)
  Γ ∪ Γ', C, Ω, π ⊢ body ⇓ v, ε, T, π'
  ─────────────────────────────────────────────────────────────────
  Γ, C, Ω, π ⊢ RECEIVE mode control { arms } ⇓ v,
               epistemic⊔ε,
               Receive(msg, now()) :: T,
               π'
```

`RECEIVE` is the mailbox-input form. Its base effect is `epistemic` because it only selects from already-arrived workflow input; blocking and timeout behavior are determined by `mode` rather than by a higher effect classification.

The selection relation searches according to SPEC-013: it probes declared stream mailboxes or the
implicit control mailbox under the current source scheduling modifier, then selects the oldest
queued entry whose arm matches and whose guard succeeds. It is not a single-message poll followed
by separate arm testing.

Canonical `RECEIVE` runtime behavior:

- `receive { ... }` uses the runtime scheduler and the current source scheduling modifier to select a source mailbox, then checks arms in declaration order. Pattern matching happens before guard evaluation; a message is removed from the mailbox only after the selected arm's guard succeeds. If no source yields a match, `_` runs if present; otherwise control falls through to the next workflow step with no error.
- `receive wait { ... }` uses the same source-selection and arm-order model, but blocks until a matching event is available, then runs the first matching arm.
- `receive wait DURATION { ... }` uses one timeout budget for the whole receive operation. It blocks until a matching event arrives or the budget expires. On timeout, `_` runs if present; otherwise control falls through with no error.
- `receive control ... { ... }` polls only the implicit control mailbox and does not consume normal stream events.

### 4.2 Deliberative Layer

```
(ORIENT)
  eval(Γ, expr) ↝ v
  analyze(v) ↝ v'
  Γ, C, Ω, π ⊢ cont ⇓ v'', ε, T, π'
  ─────────────────────────────────────────────────────────────────
  Γ, C, Ω, π ⊢ ORIENT expr in cont ⇓ v'',
               deliberative⊔ε,
               Orient(expr, v, v', now()) :: T,
               π'
```

```
(PROPOSE)
  Γ, C, Ω, π ⊢ cont ⇓ v, ε, T, π'
  ─────────────────────────────────────────────────────────────────
  Γ, C, Ω, π ⊢ PROPOSE action in cont ⇓ v,
               deliberative⊔ε,
               Propose(action, Γ, now()) :: T,
               π'
  
  [Note: PROPOSE does not execute, only records intent]
```

### 4.3 Evaluative Layer

```
(DECIDE-PERMIT)
  eval(Γ, expr) ↝ v
  lookup(policies, policy).eval(v, Γ) = PolicyDecision::Permit
  Γ, C, Ω, π ⊢ cont ⇓ v', ε, T, π'
  ─────────────────────────────────────────────────────────────────
  Γ, C, Ω, π ⊢ DECIDE expr under policy in cont ⇓ v',
               evaluative⊔ε,
               Decide(policy, Permit, v, now()) :: T,
               π'

(DECIDE-DENY)
  eval(Γ, expr) ↝ v
  lookup(policies, policy).eval(v, Γ) = PolicyDecision::Deny
  ─────────────────────────────────────────────────────────────────
  Γ, C, Ω, π ⊢ DECIDE expr under policy in cont ⇓ ⊥,
               evaluative,
               Decide(policy, Deny, v, now()) :: ε,
               π,
               error: PolicyViolation(policy, v)
```

`DECIDE` is the workflow-level policy gate and therefore always names an explicit lowered policy binding. It consumes the same normalized policy representation used by capability verification, but only admits `Permit` and `Deny` outcomes at the workflow layer. Capability-level checks may still be applied at concrete `observe`, `receive`, `set`, `send`, or `act` operations by the capability-verification runtime, which may also consume `RequireApproval` or `Transform` outcomes.

The `DECIDE` judgment models workflow-level policy gates only. Capability-verification outcomes
such as `RequireApproval` and `Transform` are owned by SPEC-017 and SPEC-018, and verification
warnings are separate metadata rather than runtime `PolicyDecision` values.

```
(CHECK-SATISFIED)
  check_obligation(role, condition, Γ) = true
  discharge(Ω, obligation) = Ω'
  Γ, C, Ω', π ⊢ cont ⇓ v, ε, T, π'
  ─────────────────────────────────────────────────────────────────
  Γ, C, Ω, π ⊢ CHECK obligation in cont ⇓ v,
               evaluative⊔ε,
               Oblig(obligation, true, now()) :: T,
               π'

(CHECK-VIOLATED)
  check_obligation(role, condition, Γ) = false
  ─────────────────────────────────────────────────────────────────
  Γ, C, Ω, π ⊢ CHECK obligation in cont ⇓ ⊥,
               evaluative,
               Oblig(obligation, false, now()) :: ε,
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
  lookup(C, action) = impl
  policy_check(action, Γ) = Permit
  impl.execute(Γ) ↝ v
  π' = extend_provenance(π, action, guard, v)
  ─────────────────────────────────────────────────────────────────
  Γ, C, Ω, π ⊢ ACT action where guard ⇓ v,
               operational,
               Act(action, v, guard, now()) :: ε,
               π'

(ACT-GUARD-FAIL)
  eval_guard(Γ, guard) = false
  ─────────────────────────────────────────────────────────────────
  Γ, C, Ω, π ⊢ ACT action where guard ⇓ ⊥,
               ε,
               GuardFail(action, guard, now()) :: ε,
               π,
               error: GuardViolation(action, guard)
```

### 4.6 Control Flow

```
(SEQ)
  Γ, C, Ω, π ⊢ w1 ⇓ v1, ε1, T1, π1
  Γ, C, Ω, π1 ⊢ w2 ⇓ v2, ε2, T2, π2
  ─────────────────────────────────────────────────────────────────
  Γ, C, Ω, π ⊢ SEQ w1 w2 ⇓ v2, ε1⊔ε2, T1 ++ T2, π2

(PAR)
  ∀i. Γ, C, Ω, fork(π) ⊢ wi ⇓ vi, εi, Ti, πi
  v = [v1, ..., vn]
  ε = ⊔ εi
  T = merge_traces(T1, ..., Tn)
  π' = join_provenance(π, π1, ..., πn)
  ─────────────────────────────────────────────────────────────────
  Γ, C, Ω, π ⊢ PAR [w1, ..., wn] ⇓ v, ε, T, π'

(IF-TRUE)
  eval(Γ, cond) = true
  Γ, C, Ω, π ⊢ then_branch ⇓ v, ε, T, π'
  ─────────────────────────────────────────────────────────────────
  Γ, C, Ω, π ⊢ IF cond then else ⇓ v, ε, T, π'

(IF-FALSE)
  eval(Γ, cond) = false
  Γ, C, Ω, π ⊢ else_branch ⇓ v, ε, T, π'
  ─────────────────────────────────────────────────────────────────
  Γ, C, Ω, π ⊢ IF cond then else ⇓ v, ε, T, π'

(LET)
  eval(Γ, expr) ↝ v
  Γ' = bind(pat, v, Γ)
  Γ', C, Ω, π ⊢ cont ⇓ v', ε, T, π'
  ─────────────────────────────────────────────────────────────────
  Γ, C, Ω, π ⊢ LET pat = expr in cont ⇓ v', ε, T, π'
```

### 4.7 Recoverable Result Handling

Recoverable failures are represented explicitly as `Result` values in the canonical language.
Workflows handle recoverable failures by pattern matching on `Ok` / `Err` values, and the meaning
of that matching is given by the ADT `Match` semantics in SPEC-020 and the core `Match` rules in
this document.

Examples of recoverable handling are therefore written as explicit `Result` construction and
`Match`.

### 4.8 Match and Constructor Semantics

`Constructor` is the expression-level core form for ADT value formation. It evaluates each payload
expression in source order, then yields the canonical runtime variant value
`Variant(name, {field: value, ...})`. The enclosing type name is not stored in the runtime value
itself; type identity is resolved by the type system and the constructor name.

`Match` is the expression-level core form for ADT case analysis. It evaluates the scrutinee, then
selects the first arm whose pattern binds successfully and whose guard succeeds. Pattern binding is
attempted in arm order, and a guard is evaluated only after its pattern has matched. The selected
arm body evaluates with the resulting bindings in scope. If no arm matches, evaluation fails with
a pattern-match error; well-typed exhaustive matches are guaranteed by the ADT typing rules in
SPEC-020, but the operational rule here is the meaning of the core form itself.

`if let` is surface sugar for the same `Match` behavior with a wildcard fallback arm. It does not
change workflow-form semantics or introduce a separate recoverable-failure mechanism.

### 4.9 Expression-Level Surface Convenience Notes

The following expression-level constructs may appear in the surface language or parser
conveniences, but they are not additional semantic families:

- `if let` is shorthand for canonical matching behavior with a wildcard fallback arm
- surface-only spellings do not expand the set of semantic laws
- implementation convenience nodes may exist internally, but they must not change the core meaning
```

## 5. Auxiliary Functions

### 5.1 Pattern Binding

```
bind(PVar(x), v, Γ)        = Γ[x ↦ v]
bind(PTuple(ps), [v1,...], Γ) = fold(bind, Γ, zip(ps, vs))
bind(PRecord(fs), {k: v, ...}, Γ) = fold(bind_field, Γ, fs)
  where bind_field((k, p), Γ) = bind(p, lookup(k, record), Γ)
bind(PVariant(C, fs), Variant(C, payload), Γ) = fold(bind_variant_field, Γ, fs)
  where bind_variant_field((k, p), Γ) = bind(p, lookup(k, payload), Γ)
bind(PWildcard, v, Γ)      = Γ
bind(PLiteral(lit), v, Γ)  = if lit == v then Γ else error
```

Variant-pattern execution matches constructor name first and then recursively binds named
payload fields. Synthetic record tags such as `__variant` are not part of the runtime contract.

### 5.2 Effect Join

```
epistemic ⊔ e       = e
deliberative ⊔ epistemic = deliberative
deliberative ⊔ deliberative = deliberative
deliberative ⊔ e    = e  (for e > deliberative)
evaluative ⊔ e      = evaluative  (for e ≤ evaluative)
evaluative ⊔ operational = operational
operational ⊔ e     = operational
```

### 5.3 Provenance Operations

```
fork(Prov { id, parent, lineage }) = 
  Prov { 
    id: fresh_id(),
    parent: Some(id),
    lineage: push(lineage, id)
  }

extend_provenance(π, action, guard, value) =
  Prov { 
    ...π,
    action_history: push(π.action_history, (action, guard, value))
  }
```

## 6. Concurrent Semantics

### 6.1 Parallel Composition

Parallel workflows execute concurrently with shared read-only context:

```
(PAR-CONCURRENT)
  Each wi executes in separate task with:
    - Immutable snapshot of Γ
    - Reference to C (capability providers are thread-safe)
    - Copy of Ω (each branch must satisfy all obligations)
    - Forked provenance π_i = fork(π)
  
  Results collected via:
    - join! for value aggregation
    - merge_traces for trace interleaving
    - join_provenance for lineage reconstruction
```

### 6.2 Race Conditions

No data races possible because:
- Γ is immutable during execution
- C implements Sync trait
- Trace is per-task, merged after

## 7. Property Testing

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

## 8. Related Documents

- SPEC-001: IR
- SPEC-003: Type System
