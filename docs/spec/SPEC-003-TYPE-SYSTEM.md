# SPEC-003: Type System

## Status: Draft

## 1. Overview

The type system tracks:
1. **Value types**: What kind of data flows through
2. **Effect types**: What computational power is used
3. **Obligation types**: What deontic constraints apply

Canonical workflow effect vocabulary used throughout this spec:
- **Epistemic** — input acquisition and read-only observation
- **Deliberative** — analysis, planning, and proposal formation
- **Evaluative** — policy and obligation evaluation
- **Operational** — external side effects and irreversible outputs

## 2. Type Judgment

```
Γ, Σ, Ω ⊢ w : τ / ε ⊣ Ω'

Where:
  Γ   = value type environment (variables → types)
  Σ   = capability signature context
  Ω   = incoming obligations
  w   = workflow
  τ   = result type
  ε   = effect type (from lattice)
  Ω'  = outgoing obligations (discharged or incurred)
```

## 3. Value Types

```rust
pub enum Type {
    Int,
    String,
    Bool,
    Null,
    Time,
    Ref(Box<str>),
    List(Box<Type>),
    Record(Vec<(Box<str>, Type)>),
    Cap(Box<str>, Effect),  // Capability with effect
    Fun(Vec<Type>, Box<Type>, Effect),  // Function type
    Var(TypeVar),           // Type variable (for inference)
}

pub struct TypeVar(pub u32);
```

## 4. Type Rules

### 4.1 Epistemic Layer

```
(OBSERVE-T)
  Σ(cap) = τ_obs → σ    σ ≤ epistemic
  bind(pat, τ_obs) = Γ'
  Γ ∪ Γ', Σ, Ω ⊢ cont : τ / ε ⊣ Ω'
  ─────────────────────────────────────────────────────────────
  Γ, Σ, Ω ⊢ OBSERVE cap as pat in cont : τ / epistemic⊔ε ⊣ Ω'

(RECEIVE-T)
  ∀i. bind(receive_pat_i, τ_msg_i) = Γ_i
      guard_i = None ∨ Γ ∪ Γ_i ⊢ guard_i : bool / ε_guard_i
      Γ ∪ Γ_i, Σ, Ω ⊢ body_i : τ / ε_i ⊣ Ω_i
  Ω_out = ⋂ Ω_i
  ε_arms = ⊔i (ε_guard_i ⊔ ε_i)
  ─────────────────────────────────────────────────────────────
  Γ, Σ, Ω ⊢ RECEIVE mode { arms_i } : τ / epistemic⊔ε_arms ⊣ Ω_out
```

**Property**: `OBSERVE` and `RECEIVE` never contribute a base effect above `epistemic`; any larger workflow effect comes from guards or branch bodies.

### 4.2 Deliberative Layer

```
(ORIENT-T)
  Γ ⊢ expr : τ_expr / ε_expr
  ε_expr ≤ deliberative
  Γ, Σ, Ω ⊢ cont : τ / ε ⊣ Ω'
  ─────────────────────────────────────────────────────────────
  Γ, Σ, Ω ⊢ ORIENT expr in cont : τ / ε_expr⊔ε ⊣ Ω'

(PROPOSE-T)
  action : τ_action / σ
  σ ≤ deliberative
  Γ, Σ, Ω ⊢ cont : τ / ε ⊣ Ω'
  ─────────────────────────────────────────────────────────────
  Γ, Σ, Ω ⊢ PROPOSE action in cont : τ / deliberative⊔ε ⊣ Ω'
```

### 4.3 Evaluative Layer

```
(DECIDE-T)
  Γ ⊢ expr : bool / ε_expr
  ε_expr ≤ evaluative
  lookup(Σ, policy) = NamedPolicy { subject: bool, core: CorePolicy }
  Γ, Σ, Ω ⊢ cont : τ / ε ⊣ Ω'
  ─────────────────────────────────────────────────────────────
  Γ, Σ, Ω ⊢ DECIDE expr under policy in cont : τ / evaluative⊔ε ⊣ Ω'

(CHECK-T)
  lookup(Ω, obligation) = Obligation(role, condition)
  Γ ⊢ condition : bool / ε_check
  discharge(Ω, obligation) = Ω'
  Γ, Σ, Ω' ⊢ cont : τ / ε ⊣ Ω''
  ─────────────────────────────────────────────────────────────
  Γ, Σ, Ω ⊢ CHECK obligation in cont : τ / ε_check⊔ε ⊣ Ω''
```

`DECIDE` is well-formed only when the policy name is explicit and resolves to a named lowered policy binding.

Workflow-level `DECIDE` sites may only reference policies whose terminal decisions are `Permit` or `Deny`. Capability-verification sites may use the same `CorePolicy` model with richer terminal decisions such as `RequireApproval` or `Transform`.

`CHECK` ranges only over obligations in `Ω`; policy evaluation belongs to `DECIDE`.

### 4.4 Operational Layer

```
(ACT-T)
  Σ(action) : τ_args → τ_ret / σ
  σ ≤ operational
  Γ ⊢ guard : bool / ε_guard
  ─────────────────────────────────────────────────────────────
  Γ, Σ, Ω ⊢ ACT action(args) where guard : τ_ret / ε_guard⊔operational ⊣ Ω
```

### 4.5 Control Flow

```
(SEQ-T)
  Γ, Σ, Ω ⊢ w1 : τ1 / ε1 ⊣ Ω1
  Γ, Σ, Ω1 ⊢ w2 : τ2 / ε2 ⊣ Ω2
  ─────────────────────────────────────────────────────────────
  Γ, Σ, Ω ⊢ SEQ w1 w2 : τ2 / ε1⊔ε2 ⊣ Ω2

(PAR-T)
  ∀i. Γ, Σ, Ω ⊢ wi : τi / εi ⊣ Ωi
  ε_par = ⊔ εi
  Ω_out = ∩ Ωi  (obligations that survive all branches)
  ─────────────────────────────────────────────────────────────
  Γ, Σ, Ω ⊢ PAR [w1..wn] : (τ1,..,τn) / ε_par ⊣ Ω_out

(IF-T)
  Γ ⊢ cond : bool / ε_cond
  Γ, Σ, Ω ⊢ then : τ / ε_then ⊣ Ω_then
  Γ, Σ, Ω ⊢ else : τ / ε_else ⊣ Ω_else
  ─────────────────────────────────────────────────────────────
  Γ, Σ, Ω ⊢ IF cond then else : τ / ε_cond⊔ε_then⊔ε_else ⊣ Ω_then∩Ω_else

(LET-T)
  Γ ⊢ expr : τ_expr / ε_expr
  bind(pat, τ_expr) = Γ'
  Γ ∪ Γ', Σ, Ω ⊢ cont : τ / ε ⊣ Ω'
  ─────────────────────────────────────────────────────────────
  Γ, Σ, Ω ⊢ LET pat = expr in cont : τ / ε_expr⊔ε ⊣ Ω'
```

### 4.6 Modal Constructs

```
(WITH-T)
  Σ, cap:τ→σ ⊢ w : τ' / ε ⊣ Ω
  ─────────────────────────────────────────────────────────────
  Γ, Σ, Ω ⊢ WITH cap DO w : τ' / σ⊔ε ⊣ Ω

(MAYBE-T)
  Γ, Σ, Ω ⊢ primary : τ / ε1 ⊣ Ω1
  Γ, Σ, Ω ⊢ fallback : τ / ε2 ⊣ Ω2
  ─────────────────────────────────────────────────────────────
  Γ, Σ, Ω ⊢ MAYBE primary else fallback : τ / ε1⊔ε2 ⊣ Ω1∩Ω2

(MUST-T)
  Γ, Σ, Ω ⊢ w : τ / ε ⊣ Ω'
  ─────────────────────────────────────────────────────────────
  Γ, Σ, Ω ⊢ MUST w : τ / ε ⊣ Ω'
  [Must verification happens at runtime]
```

## 5. Effect Inference

Effects are inferred bottom-up:

```rust
fn infer_effect(workflow: &Workflow) -> Effect {
    match workflow {
        Observe { continuation, .. } => 
            Effect::Epistemic.join(infer_effect(continuation)),
      Receive { arms, .. } =>
        arms.iter().fold(Effect::Epistemic, |acc, arm| {
            acc.join(infer_guard_effect(arm.guard.as_ref()))
              .join(infer_effect(&arm.body))
        }),
        Act { .. } => Effect::Operational,
        Seq { first, second } => 
            infer_effect(first).join(infer_effect(second)),
        Par { workflows } => 
            workflows.iter().map(infer_effect).fold(
                Effect::Epistemic, 
                Effect::join
            ),
        // ... etc
    }
}
```

## 6. Proof Obligations

The type checker generates proof obligations:

```rust
pub enum ProofObligation {
    /// Every ACT must be preceded by DECIDE
    EffectSafety {
        action: Action,
        required_decision: Policy,
    },
    
    /// Obligations must be discharged
    ObligationFulfillment {
        obligation: Obligation,
        required_before: WorkflowId,
    },
    
    /// Role separation of duties
    RoleSeparation {
        role1: Role,
        role2: Role,
        reason: SoDReason,
    },
    
    /// Guards must be decidable
    GuardDecidable {
        guard: Guard,
    },
}
```

## 7. Type Inference Algorithm

Uses Hindley-Milner style unification:

```
1. Assign fresh type variables to all un-annotated bindings
2. Collect constraints from typing rules
3. Unify constraints
4. Generalize polymorphic types
5. Report unresolved constraints as errors
```

### 7.1 Constraint Generation

```rust
enum Constraint {
    TypeEqual(Type, Type),
    EffectLeq(Effect, Effect),  // ε1 ≤ ε2
    HasCapability(Name, Effect),
    SatisfiesObligation(Obligation),
}
```

### 7.2 Unification

```rust
fn unify(c1: Type, c2: Type) -> Result<Substitution, TypeError> {
    match (c1, c2) {
        (Type::Int, Type::Int) => Ok(empty_subst()),
        (Type::Var(v), t) => bind(v, t),
        (t, Type::Var(v)) => bind(v, t),
        (Type::List(a), Type::List(b)) => unify(*a, *b),
        (Type::Fun(args1, ret1, _), Type::Fun(args2, ret2, _)) => {
            // Unify argument and return types
        }
        _ => Err(TypeError::Mismatch(c1, c2)),
    }
}
```

## 8. Property Testing

```rust
// Type safety: well-typed programs don't get stuck
proptest! {
    #[test]
    fn prop_type_safety(w in arbitrary_well_typed_workflow()) {
        let result = interpret(w);
        assert!(!result.is_stuck());
    }
}

// Effect monotonicity: effects only increase
proptest! {
    #[test]
    fn prop_effect_monotonicity(w in arbitrary_workflow()) {
        let effect = infer_effect(&w);
        let sub_effects: Vec<_> = sub_workflows(&w)
            .map(infer_effect)
            .collect();
        for sub in sub_effects {
            assert!(sub <= effect);
        }
    }
}

// Type preservation under substitution
proptest! {
    #[test]
    fn prop_type_preservation(
        w in arbitrary_workflow(),
        subst in arbitrary_substitution()
    ) {
        let ty_before = type_check(&w).unwrap();
        let w_subst = apply_subst(&w, &subst);
        let ty_after = type_check(&w_subst).unwrap();
        assert_eq!(ty_before, ty_after);
    }
}
```

## 9. Error Messages

Rich, actionable error messages:

```
error[E001]: Effect mismatch
  --> examples/workflow.ash:15:3
   |
15 |   act delete_file(path);
   |   ^^^^^^^^^^^^^^^^^^^^^
   |
   = note: This action has effect `operational`
   = note: But no preceding `decide` statement was found
   = help: Add a policy decision before this action:
   |
15 |   decide { is_safe(path) } under destructive_policy then {
16 |     act delete_file(path);
17 |   }
   |
```

## 10. Related Documents

- SPEC-001: IR
- SPEC-002: Surface Language
- SPEC-004: Operational Semantics
