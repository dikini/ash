# SPEC-004: Operational Semantics

## Status: Draft

## 1. Overview

Big-step operational semantics for the Ash workflow language. Tracks values, effects, traces, and provenance.

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
  poll_mailbox(mode, control, Γ) ↝ msg
  select_receive_arm(arms, msg, Γ) = (Γ', body)
  Γ ∪ Γ', C, Ω, π ⊢ body ⇓ v, ε, T, π'
  ─────────────────────────────────────────────────────────────────
  Γ, C, Ω, π ⊢ RECEIVE mode control { arms } ⇓ v,
               epistemic⊔ε,
               Receive(msg, now()) :: T,
               π'
```

`RECEIVE` is the mailbox-input form. Its base effect is `epistemic` because it only selects from already-arrived workflow input; blocking and timeout behavior are determined by `mode` rather than by a higher effect classification.

Canonical `RECEIVE` runtime behavior:

- `receive { ... }` scans declared stream mailboxes once in arm order. If no arm matches, `_` runs if present; otherwise control falls through to the next workflow step with no error.
- `receive wait { ... }` blocks until a matching event is available, then runs the first matching arm.
- `receive wait DURATION { ... }` blocks until a matching event arrives or the timeout expires. On timeout, `_` runs if present; otherwise control falls through with no error.
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

### 4.4 Operational Layer

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

### 4.5 Control Flow

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

### 4.6 Error Handling

```
(ATTEMPT-SUCCESS)
  Γ, C, Ω, π ⊢ w1 ⇓ v, ε, T, π'
  ─────────────────────────────────────────────────────────────────
  Γ, C, Ω, π ⊢ ATTEMPT w1 catch w2 ⇓ v, ε, T, π'

(ATTEMPT-CATCH)
  Γ, C, Ω, π ⊢ w1 ⇓ ⊥, ε, T, π, error:e
  Γ, C, Ω, π ⊢ w2 ⇓ v, ε', T', π'
  ─────────────────────────────────────────────────────────────────
  Γ, C, Ω, π ⊢ ATTEMPT w1 catch w2 ⇓ v,
               ε⊔ε',
               T ++ [Catch(e, now())] ++ T',
               π'
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
