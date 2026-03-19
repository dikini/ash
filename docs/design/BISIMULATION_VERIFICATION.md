# Bisimulation-Based Verification

## Overview

The differential testing approach is essentially checking for **bisimilarity** between the Lean reference interpreter and the Rust implementation. This document formalizes this connection.

## What is Bisimulation?

Two systems are **bisimilar** if they can simulate each other's moves indefinitely, maintaining equivalent observable behavior.

```
                    Rust Impl                              Lean Ref
                    ─────────                              ────────
                       │                                      │
                       │ Transition: e --[a]--> e'            │
                       │                                      │
                       ▼                                      ▼
                     e' ─────────────────────────────────►  e'_ref
                       │           Bisimulation            │
                       │           Relation R              │
                       │                                      │
                       │ Transition: e'_ref --[a]--> e''_ref │
                       │                                      │
                       ▼                                      ▼
                     e'' ◄───────────────────────────────── e''_ref
```

## Formal Definition

### Labeled Transition System (LTS)

```lean4
-- States: (Expression, Environment, Effect accumulator)
def State := Expr × Env × Effect

-- Transitions labeled by actions
def Action :=
  | Constructor (type : String) (variant : String)
  | PatternMatch (pattern : Pattern) (matched : Bool)
  | EffectStep (effect : Effect)
  | Value (v : Value)
  | Error (e : Error)

-- Transition relation: s --[a]--> s'
def Step : State → Action → State → Prop
```

### Bisimulation Relation

```lean4
-- A relation R is a bisimulation if:
def IsBisimulation (R : State × State → Prop) : Prop :=
  ∀ (s₁ s₂ : State), R (s₁, s₂) →
    -- Forward simulation: If s₁ can take a step, s₂ can match it
    (∀ a s₁', Step s₁ a s₁' →
      ∃ s₂', Step s₂ a s₂' ∧ R (s₁', s₂'))
    ∧
    -- Backward simulation: If s₂ can take a step, s₁ can match it
    (∀ a s₂', Step s₂ a s₂' →
      ∃ s₁', Step s₁ a s₁' ∧ R (s₁', s₂'))

-- Two states are bisimilar if there exists a bisimulation relating them
def Bisimilar (s₁ s₂ : State) : Prop :=
  ∃ R, IsBisimulation R ∧ R (s₁, s₂)
```

## Bisimulation for Ash Interpreters

### 1. Weak Bisimulation (Observational Equivalence)

We don't need step-by-step equivalence, just observable equivalence:

```lean4
-- Weak bisimulation ignores internal (τ) steps
def IsWeakBisimulation (R : State × State → Prop) : Prop :=
  ∀ (s₁ s₂ : State), R (s₁, s₂) →
    -- Forward
    (∀ a s₁', Step s₁ a s₁' →
      if a = τ then
        -- Internal step: s₂ can stay or match
        R (s₁', s₂) ∨ ∃ s₂', Step s₂ τ s₂' ∧ R (s₁', s₂')
      else
        -- Observable step: s₂ must match
        ∃ s₂', WeakStep s₂ a s₂' ∧ R (s₁', s₂'))
    ∧
    -- Backward (symmetric)
    (∀ a s₂', Step s₂ a s₂' → ...)

-- Observable actions only
def WeakStep (s : State) (a : Action) (s' : State) : Prop :=
  ∃ s'', Steps s τ s'' ∧ Step s'' a s'
```

### 2. The Bisimulation Relation for Rust/Lean

```lean4
-- The concrete relation between Rust and Lean states
inductive AshBisim : (Rust.State × Lean.State) → Prop
  | expr_equiv {re le env} :
      -- ASTs are structurally equivalent
      ASTEquiv re le →
      -- Environments bind equivalent values
      EnvEquiv Rust.env env Lean.env env →
      AshBisim ((re, Rust.env, Rust.efx), (le, Lean.env, Lean.efx))

  | value_equiv {rv lv} :
      ValueEquiv rv lv →
      AshBisim ((Value rv, _, _), (Value lv, _, _))

  | error_equiv {err} :
      AshBisim ((Error err, _, _), (Error err, _, _))
```

### 3. Proving the Bisimulation

```lean4
-- Main theorem: Rust and Lean interpreters are bisimilar
theorem rust_lean_bisimulation :
  ∀ (re : Rust.Expr) (le : Lean.Expr),
    ASTEquiv re le →
    Bisimilar
      (Rust.initialState re)
      (Lean.initialState le) := by
  -- Strategy: Show AshBisim is a bisimulation
  apply exists_bisimulation AshBisim
  
  -- Prove AshBisim satisfies bisimulation conditions
  unfold IsBisimulation
  intro (rs, ls) h_bisim
  
  -- Case analysis on the relation
  cases h_bisim with
  | expr_equiv h_ast h_env =>
      constructor
      · -- Forward simulation
        intro a rs' h_step
        cases h_step with
        | constructor_step h_ctor =>
            -- Rust constructor step
            -- Show Lean has matching step
            have h_lean : ∃ ls', Lean.Step ls a ls' := by
              apply lean_has_constructor_step h_ast h_ctor
            obtain ⟨ls', h_step'⟩ := h_lean
            use ls'
            constructor
            · exact h_step'
            · -- Show resulting states are related
              apply AshBisim.expr_equiv
              apply ast_equiv_after_constructor h_ast h_ctor
              apply env_equiv_after_constructor h_env h_ctor
        | pattern_match_step h_match =>
            -- Similar for pattern matching
            sorry
        -- ... other cases
      · -- Backward simulation (symmetric)
        sorry
```

## Practical Bisimulation Checking

### 1. Step-by-Step Bisimulation Testing

Instead of just comparing final results, check each step:

```rust
/// Bisimulation-aware differential testing
pub fn bisimulation_test(workflow: &Workflow) -> Result<(), BisimError> {
    let mut rust_state = RustInterpreter::initial(workflow);
    let mut lean_state = LeanInterpreter::initial(workflow);
    
    loop {
        // Check if states are observationally equivalent
        if !observationally_equivalent(&rust_state, &lean_state) {
            return Err(BisimError::StateMismatch {
                rust: rust_state,
                lean: lean_state,
            });
        }
        
        // Get possible transitions from Rust
        let rust_transitions = rust_state.possible_transitions();
        
        // For each Rust transition, Lean must have matching
        for (action, rust_next) in rust_transitions {
            match lean_state.find_transition(&action) {
                None => {
                    return Err(BisimError::MissingTransition {
                        action,
                        rust_state: rust_state.clone(),
                        lean_state: lean_state.clone(),
                    });
                }
                Some(lean_next) => {
                    // Recursively check bisimulation
                    bisimulation_test_step(rust_next, lean_next)?;
                }
            }
        }
        
        // Also check backward (Lean transitions Rust can match)
        // ...
        
        // If both are final states, we're done
        if rust_state.is_final() && lean_state.is_final() {
            break;
        }
    }
    
    Ok(())
}
```

### 2. Coinductive Bisimulation

```lean4
-- Coinductive definition allows infinite behaviors
coinductive Bisim : State → State → Prop
  | mk (s₁ s₂ : State)
      (h_forward : ∀ a s₁', Step s₁ a s₁' →
        ∃ s₂', Step s₂ a s₂' ∧ Bisim s₁' s₂')
      (h_backward : ∀ a s₂', Step s₂ a s₂' →
        ∃ s₁', Step s₁ a s₁' ∧ Bisim s₁' s₂') :
      Bisim s₁ s₂

-- Example: Constructor evaluation is bisimilar
theorem constructor_bisim :
  ∀ (fields : List (String × Expr)) (env : Env),
    Bisim
      (Rust.Constructor fields, env, .epistemic)
      (Lean.Constructor fields, env, .epistemic) := by
  cofix h_coind  -- Coinductive hypothesis
  intro fields env
  apply Bisim.mk
  · -- Forward
    intro a s' h_step
    cases h_step
    -- Show Lean matches
    sorry
  · -- Backward
    sorry
```

## Bisimulation Variants for Ash

### 1. Effect-Preserving Bisimulation

```lean4
-- Bisimulation that preserves effect traces
def EffectPreservingBisim (s₁ s₂ : State) : Prop :=
  Bisim s₁ s₂ ∧
  -- Effects are identical
  EffectTrace s₁ = EffectTrace s₂

-- Theorem: Both interpreters produce same effect trace
theorem effect_trace_equivalence :
  ∀ (e : Expr),
    EffectPreservingBisim
      (Rust.initial e)
      (Lean.initial e) := by
  sorry
```

### 2. Value-Preserving Bisimulation

```lean4
-- For pure computations (no effects)
def PureBisim (s₁ s₂ : State) : Prop :=
  ∀ v, EvaluatesTo s₁ v ↔ EvaluatesTo s₂ v

-- Theorem: Constructor evaluation is pure
theorem constructor_pure_bisim :
  ∀ (c : ConstructorExpr) (env : Env),
    PureBisim
      (Rust.evalConstructor c env)
      (Lean.evalConstructor c env) := by
  sorry
```

### 3. Crash-Preserving Bisimulation

```lean4
-- Both crash or neither crashes
def CrashPreservingBisim (s₁ s₂ : State) : Prop :=
  (∃ err, Crashes s₁ err ↔ Crashes s₂ err) ∧
  Bisim (non_crash_states s₁) (non_crash_states s₂)

-- Important: Pattern match failures should behave same
theorem match_failure_bisim :
  ∀ (m : MatchExpr) (env : Env),
    CrashPreservingBisim
      (Rust.evalMatch m env)
      (Lean.evalMatch m env) := by
  sorry
```

## Using Bisimulation for Optimization Verification

When optimizing the Rust interpreter, prove it remains bisimilar to Lean:

```lean4
-- Original Rust (reference for this proof)
def RustOriginal : Interpreter

-- Optimized Rust
def RustOptimized : Interpreter

-- Prove optimized is bisimilar to original
theorem optimization_correct :
  ∀ (e : Expr),
    Bisim
      (RustOriginal.initial e)
      (RustOptimized.initial e) := by
  -- Proof by establishing bisimulation relation
  -- that relates original and optimized states
  sorry
```

## Bisimulation-Based Test Generation

Generate tests that specifically target bisimulation boundaries:

```rust
/// Generate tests near bisimulation "edge cases"
struct BisimulationGenerator;

impl BisimulationGenerator {
    /// Generate programs that stress pattern matching
    fn generate_pattern_stress_tests() -> Vec<Workflow> {
        vec![
            // Nested patterns
            gen_nested_variant_match(depth: 10),
            
            // Wide matches (many arms)
            gen_wide_match(arms: 100),
            
            // Deep struct patterns
            gen_deep_struct_pattern(depth: 20),
            
            // Mixed patterns
            gen_mixed_pattern_match(),
        ]
    }
    
    /// Generate programs that stress control flow
    fn generate_control_flow_tests() -> Vec<Workflow> {
        vec![
            // Deeply nested if-let
            gen_nested_if_let(depth: 50),
            
            // Interleaved match and if-let
            gen_interleaved_control(),
            
            // Transfer sequences
            gen_transfer_sequence(length: 100),
        ]
    }
}
```

## Checking Bisimulation in CI

```yaml
name: Bisimulation Verification

on: [push, pull_request]

jobs:
  bisimulation-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Build Lean reference
        run: lake build
      
      - name: Build Rust implementation
        run: cargo build --release
      
      - name: Run bisimulation tests
        run: |
          ./scripts/bisimulation_check.sh \
            --step-limit 10000 \
            --coverage-target 0.95 \
            --report bisim_report.json
      
      - name: Check coverage
        run: |
          COVERAGE=$(jq '.coverage' bisim_report.json)
          if (( $(echo "$COVERAGE < 0.95" | bc -l) )); then
            echo "Bisimulation coverage too low: $COVERAGE"
            exit 1
          fi
      
      - name: Upload report
        uses: actions/upload-artifact@v3
        with:
          name: bisimulation-report
          path: bisim_report.json
```

## Benefits of Bisimulation View

| Aspect | Traditional Testing | Bisimulation |
|--------|---------------------|--------------|
| **What it checks** | Final results match | All intermediate steps match |
| **Bug localization** | "Results differ" | "Step 47: Rust takes τ, Lean takes constructor" |
| **Proof structure** | Ad-hoc | Formal, compositional |
| **Optimization validation** | Re-test everything | Prove bisimulation preserved |
| **Confidence** | High | Very High |

## Summary

The differential testing approach **is** bisimulation testing:

1. **Reference (Lean)** = Specification LTS
2. **Implementation (Rust)** = Implementation LTS  
3. **Differential tests** = Checking for bisimilarity
4. **Proofs in Lean** = Proving bisimulation properties

Formalizing it as bisimulation gives us:
- Clear theoretical foundation
- Compositional reasoning (prove components bisimilar)
- Stronger guarantees than just testing
- Path to proving correctness, not just finding bugs

## Next Steps

1. **Immediate**: Implement step-tracing in both interpreters
2. **Short-term**: Build bisimulation comparison tool
3. **Medium-term**: Prove key components bisimilar in Lean
4. **Long-term**: Full bisimulation proof of equivalence

```
"Testing shows the presence, not the absence of bugs"
"Bisimulation proves the absence of behavioral differences"
```
