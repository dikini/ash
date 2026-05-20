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

For the current promoted coarse-grade contract, workflow effect classification is computed from
canonical workflow forms plus source-level contracts. Provider effect metadata may be checked for
compatibility at embedding/runtime boundaries, but it is not the primary source of source-level
effect typing.

The possible addition of an explicit surfaced `Pure` bottom element remains follow-up work rather
than current normative type-system text.

## 1.1 Phase-Owned Boundaries

The type system owns judgments that prove or reject type, effect, obligation, and ADT
compatibility. It does not own parser acceptance or runtime execution outcomes.

- Parser rejection belongs to SPEC-002 and parser boundary references.
- Lowering rejection belongs to SPEC-001 and lowering boundary references.
- Type rejection belongs here when a workflow, expression, pattern, or declaration cannot be
  assigned a valid type, effect, or obligation shape.
- Entry-workflow signature rejection is a specialized workflow-typing judgment owned by
    SPEC-022. This spec does not restate that canonical rule; it only establishes that entry
    signature validation belongs to typing rather than parsing or runtime execution.
- Runtime rejection belongs to SPEC-004 and the runtime-observable contract family.
- Verification-time availability checks belong to runtime verification, not pure typing.
- Workflow effect ceilings are compared by runtime verification after the type layer establishes
  the workflow effect classification.

Policy typing is split by consumer:

- workflow `decide` sites type-check only against policies whose terminal decisions are
  `Permit` / `Deny`,
- capability-verification sites may type-check against policies that can lower to
  `{Permit, Deny, RequireApproval, Transform}`,
- `Warn` is not a policy decision and is handled as verification metadata, not as a policy
  typing outcome.

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
    Float,
    String,
    Bool,
    Null,
    Time,
    Ref(Box<str>),
    List(Box<Type>),
    Record(Vec<(Box<str>, Type)>),
    Cap(Box<str>, Effect),  // Usage-site capability witness type with implementation-carried effect metadata
    Fn(Vec<Type>, Box<Type>),           // Pure function type
    Fun(Vec<Type>, Box<Type>, Effect),  // Effectful/capability-linked function type
    Var(TypeVar),           // Type variable (for inference)
    Constructor {
        name: QualifiedName,
        args: Vec<Type>,
        kind: Kind,
    },
}

pub struct TypeVar(pub u32);
```

Function types are split normatively as follows:

- `Type::Fn(args, ret)` is the canonical type representation for pure functions. It carries only
  argument and result types because pure functions do not contribute an effect grade of their own
  to the function type.
- `Type::Fun(args, ret, effect)` is reserved for effectful or capability-linked function values
  whose callable contract must retain an explicit effect classification.
- Specs and implementations MUST NOT encode pure functions as `Type::Fun(..., effect)` merely by
  choosing a low effect grade. Pure functions use `Type::Fn`; effect-annotated callable values use
  `Type::Fun`.

`Type::Null` is the canonical no-value result type in this spec. Any construct that yields “no
meaningful value”, including omitted-else `if`, uses `Type::Null` rather than an implicit `Unit` or
another unnamed bottom/result placeholder.

The `Type::Constructor` variant represents user-defined ADT instances and generic type
applications. It carries a qualified name (potentially with module path), type arguments
for generic instantiation, and a kind annotation for higher-kinded type checking.

### 3.1 User-Defined ADT Definitions

User-defined ADT declarations are specified in source form by `TypeDef`, `TypeBody`,
`VariantDef`, and `TypeExpr` as defined in `SPEC-020`.

That source model is canonical:

- `TypeDef` introduces a named type with generic parameters and visibility.
- `TypeBody::Enum` defines constructors together with their source payload shape: unit, record, or
  tuple.
- `TypeBody::Struct` and `TypeBody::Alias` define nominal wrappers over `TypeExpr`.
- `TypeExpr` is the source-level type language used inside ADT declarations.

Implementations may elaborate these declarations into internal type metadata for inference,
constructor lookup, or exhaustiveness checking, but that elaborated representation is derived
from the source model rather than replacing it with a second specification-level contract.

### 3.2 Kind System

```rust
/// Kind annotations for type constructors
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Kind {
    /// The kind of types: *
    Type,
    /// Function kind: K1 -> K2 (curried)
    Arrow(Box<Kind>, Box<Kind>),
}
```

Kinds classify types and type constructors:

- `Kind::Type` (written `*`) is the kind of all concrete value types
- `Kind::Arrow(k1, k2)` (written `k1 -> k2`) is the kind of type constructors
  taking one type argument of kind `k1` and producing a type of kind `k2`
- Kinds are curried: binary constructors like `Result` have kind `* -> * -> *`

Examples:

- `Int`, `String` have kind `Type` (or `*`)
- `Option` has kind `Type -> Type` (or `* -> *`)
- `Result` has kind `Type -> Type -> Type` (or `* -> * -> *`)
- `* -> *` is parsed as `Arrow(Box::new(Type), Box::new(Type))`

Phase 97 uses the existing `Type::Constructor` machinery to represent `Act<A>` as a unary
constructor of kind `* -> *`; no new `Type` variant or kind rule is introduced, and the existing
`Type::Fn` / `Type::Fun` split remains intact.

### 3.3 Qualified Names

```rust
/// A fully qualified type name with optional module path
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QualifiedName {
    /// Module path components (e.g., ["std", "option"])
    pub path: Vec<Box<str>>,
    /// The base name (e.g., "Option")
    pub name: Box<str>,
}

impl QualifiedName {
    /// Create a simple unqualified name
    pub fn simple(name: impl Into<Box<str>>) -> Self {
        Self {
            path: Vec::new(),
            name: name.into(),
        }
    }

    /// Create a qualified name with module path
    pub fn qualified(
        path: impl IntoIterator<Item = impl Into<Box<str>>>,
        name: impl Into<Box<str>>,
    ) -> Self {
        Self {
            path: path.into_iter().map(Into::into).collect(),
            name: name.into(),
        }
    }
}
```

Qualified names enable type definitions from different modules to coexist without
naming collisions. The empty path denotes a name from the current scope.

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

### 4.8 Rejection Boundaries

Type checking rejects:

- unresolved named policy references for workflow `decide`
- workflow `decide` sites whose resolved policy can lower to outcomes outside `{Permit, Deny}`
- capability-verification sites whose resolved policy cannot lower to the verification outcome
  set required by the consumer: `{Permit, Deny, RequireApproval, Transform}`
- non-boolean `receive` guards
- unknown ADT constructors or variant patterns
- constructor payload mismatches against resolved enum metadata
- non-exhaustive ADT `match` where exhaustiveness is required by the contract

These are type-layer boundary failures. They must not be deferred to runtime execution or treated
as parser or lowering ambiguities.

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

(PAR-HISTORICAL)
  Historical note only: earlier drafts modeled `PAR [w1..wn]` here.
  `PAR` is not part of the current sequential workflow contract, so this
  spec defines no active typing rule for it.

(IF-T)
  Γ ⊢ cond : bool / ε_cond
  Γ, Σ, Ω ⊢ then : τ / ε_then ⊣ Ω_then
  Γ, Σ, Ω ⊢ else : τ / ε_else ⊣ Ω_else
  ─────────────────────────────────────────────────────────────
  Γ, Σ, Ω ⊢ IF cond then else : τ / ε_cond⊔ε_then⊔ε_else ⊣ Ω_then∩Ω_else

(IF-NO-ELSE-T)
  Γ ⊢ cond : bool / ε_cond
  Γ, Σ, Ω ⊢ then : Null / ε_then ⊣ Ω_then
  ─────────────────────────────────────────────────────────────
  Γ, Σ, Ω ⊢ IF cond then : Null / ε_cond⊔ε_then ⊣ Ω_then∩Ω
```

An omitted-else `if` is typed as the canonical no-value form. Normatively, `if cond { then }`
uses result type `Type::Null` and is equivalent for typing purposes to `if cond { then } else {
null }`. This does not introduce a new effect grade: its effect is still the join of the condition
and branch effects under the existing lattice.

```
(LET-T)
  Γ ⊢ expr : τ_expr / ε_expr
  bind(pat, τ_expr) = Γ'
  Γ ∪ Γ', Σ, Ω ⊢ cont : τ / ε ⊣ Ω'
  ─────────────────────────────────────────────────────────────
  Γ, Σ, Ω ⊢ LET pat = expr in cont : τ / ε_expr⊔ε ⊣ Ω'
```

**Type-environment consequence for surface statement lists:**

Surface statement lists are lowered to nested `LET ... in cont` forms before type checking
(see [SPEC-002](../SPEC-002-SURFACE.md) §4.4). Therefore, the type environment extension from a
binding statement is immediately visible in subsequent statements of the same block, and the
`Γ ∪ Γ'` environment extension in `(LET-T)` reflects the lexical scope established by the
canonical lowering.

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

### 4.7 ADT Typing Contract

Constructor typing, variant-pattern typing, and exhaustiveness analysis all operate over the
same resolved enum definition derived from the source `TypeDef`.

`if let` is typed as the same pattern-matching shape as the corresponding `match` with a wildcard
fallback branch; it does not introduce a separate ADT typing rule.

- A constructor expression such as `Some { value: 42 }` has the instantiated parent enum type.
- A tuple-variant constructor expression such as `RuntimeError(2, "missing config")` has that same
  instantiated parent enum type, with payload checking by positional arity and order.
- A variant pattern such as `Some { value: x }` is typed against that same enum definition and
  binds fields using the declared field types.
- A tuple-variant pattern such as `RuntimeError(code, msg)` is typed against the same enum
  definition and binds payload positions using the declared tuple element types.
- Exhaustiveness analyzes constructor coverage for the resolved enum type, not record fields or
  synthetic tag names.
- Internal checker approximations such as `__variant`-tagged records are implementation details
  and are not part of the language contract. The same holds for any implementation-defined names
  used internally to elaborate tuple payload positions.

## 5. Effect Inference

Effects are inferred bottom-up from workflow forms and source-level contracts, then composed by
join over the current coarse lattice. Under the current promoted contract, control/modal forms such
as `Let`, `If`, `Seq`, `ForEach`, `Ret`, `Done`, `With`, `Maybe`, and `Must` do not add a
new effect classification of their own beyond the effects of their constituent expressions,
branches, or subworkflows. Because the normative lattice has not yet adopted surfaced `Pure`, this
spec records those forms as effect-neutral in themselves without renaming that neutrality to a new
fifth grade.

Current workflow-form classification summary:

| Workflow form | Current coarse effect classification |
|---|---|
| `Observe` | `Epistemic` base effect, joined with continuation |
| `Receive` | `Epistemic` base effect, joined with guard/body effects |
| `Orient` | `Deliberative` base effect, joined with continuation |
| `Propose` | `Deliberative` base effect, joined with continuation |
| `Decide` | `Evaluative` base effect, joined with continuation |
| `Check` | `Evaluative` base effect, joined with continuation |
| `Act` | `Operational` |
| `Let` / `If` / `Seq` / `ForEach` | join-based composition over constituent effects; no extra grade of their own |
| `Ret` / `Done` | no extra grade of their own |
| `With` / `Maybe` / `Must` | no extra grade of their own beyond enclosed workflows |
| `Oblig` | governance/obligation form; no separate coarse grade beyond enclosed workflow unless later revised |

Phase 122 keeps this historical workflow-form table for existing IR/type
references, while alpha AMIR/bytecode lowering follows SPEC-069: OODA-shaped
Observe/Orient/Decide/Act examples are ordinary library/template/lint
compatibility, not privileged primitive alpha execution roots.

The `Pure` bottom-element question remains explicit follow-up work before any corpus-wide lattice
rewrite.

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
        Orient { continuation, .. } =>
            Effect::Deliberative.join(infer_effect(continuation)),
        Propose { continuation, .. } =>
            Effect::Deliberative.join(infer_effect(continuation)),
        Decide { continuation, .. } =>
            Effect::Evaluative.join(infer_effect(continuation)),
        Check { continuation, .. } =>
            Effect::Evaluative.join(infer_effect(continuation)),
        Act { .. } => Effect::Operational,
        Oblig { workflow, .. } =>
            infer_effect(workflow),
        Let { expr, continuation, .. } =>
            infer_expr_effect(expr).join(infer_effect(continuation)),
        If { condition, then_branch, else_branch } =>
            infer_expr_effect(condition)
                .join(infer_effect(then_branch))
                .join(infer_effect(else_branch)),
        Seq { first, second } => 
            infer_effect(first).join(infer_effect(second)),
        ForEach { collection, body, .. } =>
            infer_expr_effect(collection).join(infer_effect(body)),
        Ret { expr } =>
            infer_expr_effect(expr),
        With { workflow, .. } =>
            infer_effect(workflow),
        Maybe { primary, fallback } =>
            infer_effect(primary).join(infer_effect(fallback)),
        Must { workflow } =>
            infer_effect(workflow),
        Done => Effect::Epistemic,
        // ... etc
    }
}
```

The pseudocode above is illustrative rather than a full executable algorithm. Its role is to make
the current contract explicit: workflow-form classification is syntax-directed and join-based.
Where examples use `Epistemic` as the current identity element, that reflects today's surfaced
coarse lattice, not a silent decision that `Pure` has already been adopted normatively.

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
        // Base type unification
        (Type::Int, Type::Int) => Ok(Substitution::empty()),
        (Type::String, Type::String) => Ok(Substitution::empty()),
        (Type::Bool, Type::Bool) => Ok(Substitution::empty()),
        (Type::Null, Type::Null) => Ok(Substitution::empty()),
        
        // Variable binding (occurs check)
        (Type::Var(v), t) => bind(v, t),
        (t, Type::Var(v)) => bind(v, t),
        
        // Structural type unification
        (Type::List(a), Type::List(b)) => unify(*a, *b),
        (Type::Ref(a), Type::Ref(b)) if a == b => Ok(Substitution::empty()),
        
        // Record unification (structural, contravariant in fields)
        (Type::Record(fs1), Type::Record(fs2)) => {
            unify_records(fs1, fs2)
        }
        
        // Pure function type unification (contravariant in args, covariant in ret)
        (Type::Fn(args1, ret1), Type::Fn(args2, ret2)) => {
            if args1.len() != args2.len() {
                return Err(TypeError::ArityMismatch(args1.len(), args2.len()));
            }
            let mut subst = Substitution::empty();
            // Unify arguments contravariantly
            for (a1, a2) in args1.iter().zip(args2.iter()) {
                let s = unify(a2.clone(), a1.clone())?;
                subst = subst.compose(s);
            }
            // Unify return type covariantly
            let s = unify(*ret1, *ret2)?;
            subst = subst.compose(s);
            Ok(subst)
        }

        // Effectful/capability-linked function type unification
        // (contravariant in args, covariant in ret, invariant in explicit effect contract)
        (Type::Fun(args1, ret1, eff1), Type::Fun(args2, ret2, eff2)) => {
            if args1.len() != args2.len() {
                return Err(TypeError::ArityMismatch(args1.len(), args2.len()));
            }
            if eff1 != eff2 {
                return Err(TypeError::EffectMismatch {
                    expected: eff1,
                    found: eff2,
                });
            }
            let mut subst = Substitution::empty();
            // Unify arguments contravariantly
            for (a1, a2) in args1.iter().zip(args2.iter()) {
                let s = unify(a2.clone(), a1.clone())?;
                subst = subst.compose(s);
            }
            // Unify return type covariantly
            let s = unify(*ret1, *ret2)?;
            subst = subst.compose(s);
            Ok(subst)
        }
        
        // Constructor unification: Constructor vs Constructor
        (
            Type::Constructor { name: n1, args: a1, .. },
            Type::Constructor { name: n2, args: a2, .. }
        ) => {
            if n1 != n2 {
                return Err(TypeError::ConstructorMismatch(n1, n2));
            }
            if a1.len() != a2.len() {
                return Err(TypeError::ArityMismatch(a1.len(), a2.len()));
            }
            let mut subst = Substitution::empty();
            for (arg1, arg2) in a1.iter().zip(a2.iter()) {
                let s = unify(arg1.clone(), arg2.clone())?;
                subst = subst.compose(s);
            }
            Ok(subst)
        }
        
        // Constructor unification: Constructor vs Variable
        (Type::Constructor { .. }, Type::Var(v)) => bind(v, c1),
        (Type::Var(v), Type::Constructor { .. }) => bind(v, c2),
        
        // Mismatch
        _ => Err(TypeError::Mismatch(c1, c2)),
    }
}

/// Bind a type variable to a type, performing occurs check
fn bind(var: TypeVar, ty: Type) -> Result<Substitution, TypeError> {
    if occurs_in(var, &ty) {
        return Err(TypeError::InfiniteType(var, ty));
    }
    Ok(Substitution::singleton(var, ty))
}

/// Check if a type variable occurs in a type (occurs check)
fn occurs_in(var: TypeVar, ty: &Type) -> bool {
    match ty {
        Type::Var(v) => v == &var,
        Type::List(inner) => occurs_in(var, inner),
        Type::Record(fields) => fields.iter().any(|(_, t)| occurs_in(var, t)),
        Type::Fn(args, ret) => {
            args.iter().any(|t| occurs_in(var, t)) || occurs_in(var, ret)
        }
        Type::Fun(args, ret, _) => {
            args.iter().any(|t| occurs_in(var, t)) || occurs_in(var, ret)
        }
        Type::Constructor { args, .. } => args.iter().any(|t| occurs_in(var, t)),
        _ => false,
    }
}
```

For `Type::Fun`, the effect slot is part of the callable contract rather than incidental metadata.
Unification therefore MUST compare that slot explicitly. Two effectful function values with the
same argument and return types but different workflow effects do not unify silently; they produce
an effect mismatch that the checker reports separately from ordinary argument/return mismatches.

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

### 9.1 Type Variable Naming

Type variables are assigned human-readable names for error messages:

```rust
/// Assigns and tracks names for type variables in error messages
pub struct TypeVarNames {
    next_id: u32,
    names: HashMap<TypeVar, Box<str>>,
}

impl TypeVarNames {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            names: HashMap::new(),
        }
    }

    /// Get or assign a name for a type variable
    pub fn name_for(&mut self, var: TypeVar) -> &str {
        self.names.entry(var).or_insert_with(|| {
            let name = format!("{}{}",
                (b'a' + (self.next_id % 26) as u8) as char,
                if self.next_id >= 26 { format!("{}", self.next_id / 26) } else { String::new() }
            );
            self.next_id += 1;
            name.into()
        })
    }
}
```

Naming scheme:

- First 26 variables: `a`, `b`, `c`, ..., `z`
- Variables 26-51: `a1`, `b1`, ..., `z1`
- Variables 52-77: `a2`, `b2`, ..., `z2`
- And so on...

This produces readable error messages like:

```
error[E003]: Type mismatch
  --> example.ash:10:5
   |
10 |   let x: Option<Int> = Some { value: "hello" };
   |       ^               ------------------------
   |       |               |
   |       |               found `Option<String>`
   |       expected `Option<Int>`
   |
   = note: Type `a` (String) does not match type `a` (Int)
```

### 9.2 Type Difference Reporting

When two types don't match, the error reporter identifies and highlights structural differences:

```rust
/// Represents the structural difference between two types
#[derive(Debug, Clone)]
pub enum TypeDiff {
    /// Types are completely different
    Mismatch { expected: Type, found: Type },
    /// Constructor name differs
    ConstructorName { expected: QualifiedName, found: QualifiedName },
    /// Type argument differs at position
    TypeArgument { position: usize, diff: Box<TypeDiff> },
    /// Record field differs
    Field { name: Box<str>, expected: Type, found: Type },
    /// Function argument differs (contravariant)
    FunctionArg { position: usize, diff: Box<TypeDiff> },
    /// Function return differs
    FunctionReturn { diff: Box<TypeDiff> },
    /// Effectful function contract differs in explicit effect slot
    FunctionEffect { expected: Effect, found: Effect },
}

impl TypeDiff {
    /// Compute the structural difference between two types
    pub fn compute(expected: &Type, found: &Type) -> Self {
        match (expected, found) {
            (
                Type::Constructor { name: n1, args: a1, .. },
                Type::Constructor { name: n2, args: a2, .. }
            ) if n1 == n2 && a1.len() == a2.len() => {
                // Find first differing argument
                for (i, (arg1, arg2)) in a1.iter().zip(a2.iter()).enumerate() {
                    if arg1 != arg2 {
                        return TypeDiff::TypeArgument {
                            position: i,
                            diff: Box::new(TypeDiff::compute(arg1, arg2)),
                        };
                    }
                }
                TypeDiff::Mismatch {
                    expected: expected.clone(),
                    found: found.clone(),
                }
            }
            (Type::Fn(args1, ret1), Type::Fn(args2, ret2)) => {
                // Check return type first (more important)
                if ret1 != ret2 {
                    return TypeDiff::FunctionReturn {
                        diff: Box::new(TypeDiff::compute(ret1, ret2)),
                    };
                }
                // Check arguments
                for (i, (a1, a2)) in args1.iter().zip(args2.iter()).enumerate() {
                    if a1 != a2 {
                        return TypeDiff::FunctionArg {
                            position: i,
                            diff: Box::new(TypeDiff::compute(a2, a1)), // Contravariant
                        };
                    }
                }
                TypeDiff::Mismatch {
                    expected: expected.clone(),
                    found: found.clone(),
                }
            }
            (Type::Fun(args1, ret1, eff1), Type::Fun(args2, ret2, eff2)) => {
                // Check return type first (more important)
                if ret1 != ret2 {
                    return TypeDiff::FunctionReturn {
                        diff: Box::new(TypeDiff::compute(ret1, ret2)),
                    };
                }
                // Check arguments
                for (i, (a1, a2)) in args1.iter().zip(args2.iter()).enumerate() {
                    if a1 != a2 {
                        return TypeDiff::FunctionArg {
                            position: i,
                            diff: Box::new(TypeDiff::compute(a2, a1)), // Contravariant
                        };
                    }
                }
                if eff1 != eff2 {
                    return TypeDiff::FunctionEffect {
                        expected: eff1.clone(),
                        found: eff2.clone(),
                    };
                }
                TypeDiff::Mismatch {
                    expected: expected.clone(),
                    found: found.clone(),
                }
            }
            _ => TypeDiff::Mismatch {
                expected: expected.clone(),
                found: found.clone(),
            },
        }
    }
}
```

`TypeDiff::compute` treats `Type::Fun` effects as first-class diagnostic data. An effect mismatch
between otherwise-identical callable signatures must surface as `TypeDiff::FunctionEffect` rather
than falling through as if the effect slot were absent.

Example error with structural difference highlighting:

```
error[E004]: Type mismatch
  --> example.ash:15:20
   |
15 |   fn process(x: Result<Option<Int>, String>) -> Int { ... }
   |                    ^^^^^^^^^^^^^^^^^^^^^^
   |                    |
   |                    expected: Result<Option<Int>, String>
16 |   process(Ok { value: Some { value: "hello" } })
   |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |          |
   |          found: Result<Option<String>, String>
   |
   = note: In type argument 0 of Result:
   = note:   In type argument 0 of Option:
   = note:     Expected Int, found String
```

## 10. Error Handling Conventions

### 10.1 Boxed Error Types for Stack Efficiency

Type errors in the Ash type system use boxed types to maintain reasonable stack sizes. This follows the pattern used by serde_json and other mature Rust libraries.

**Rationale:**

The `Type` enum contains large variants (e.g., `Constructor` with `QualifiedName` and `Vec<Type>`). When `TypeError` contains unboxed `Type` values, the error type can exceed 200 bytes, causing:

- Stack overflow in deeply recursive type checking
- Poor cache locality when passing errors by value
- Binary bloat from large memcpy operations

**Convention:**

Error variants that contain `Type` should use `Box<Type>`:

```rust
// GOOD: Boxed types keep error size small
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TypeError {
    #[error("Type mismatch: expected {expected:?}, found {found:?}")]
    Mismatch { 
        expected: Box<Type>, 
        found: Box<Type> 
    },
    
    #[error("Infinite type: type variable {var:?} occurs in {typ:?}")]
    InfiniteType { 
        var: TypeVar, 
        typ: Box<Type> 
    },
    
    #[error("Pattern mismatch: expected {expected:?}, got {actual:?}")]
    PatternMismatch { 
        expected: Box<Type>, 
        actual: Box<Type> 
    },

    #[error("Function effect mismatch: expected {expected:?}, found {found:?}")]
    EffectMismatch {
        expected: Effect,
        found: Effect,
    },
    
    // Small types don't need boxing
    #[error("Unbound variable: {0}")]
    UnboundVariable(String),
    
    #[error("Unknown variant: {0}")]
    UnknownVariant(String),
}
```

**Anti-pattern (DO NOT DO):**

```rust
// BAD: Large error type causes stack bloat
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TypeError {
    #[error("Type mismatch: expected {expected:?}, found {found:?}")]
    Mismatch { 
        expected: Type,  // Type can be 100+ bytes
        found: Type,     // Error type now > 200 bytes
    },
}
```

**Reference Implementation:**

See serde_json's error type for the canonical example:
<https://docs.rs/serde_json/latest/src/serde_json/error.rs.html#15-20>

### 10.2 Error Type Size Target

Error types should aim to stay under 64 bytes on the stack. This provides:

- Efficient register passing on x86_64 and ARM64
- Cache-friendly error propagation
- Stack safety in recursive algorithms

Use `std::mem::size_of::<TypeError>()` to verify size after changes.

### 10.3 Result Type Aliases

Use boxed error types in result aliases:

```rust
// For functions that may fail with type errors
pub type TypeResult<T> = Result<T, TypeError>;

// For functions with multiple error sources
pub type CheckResult<T> = Result<T, Box<dyn std::error::Error>>;
```

## 11. Related Documents

- SPEC-001: IR
- SPEC-002: Surface Language
- SPEC-004: Operational Semantics
