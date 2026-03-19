# Lean Reference Interpreter for Differential Testing

## Overview

Implement a reference interpreter in Lean 4 that:
1. Serves as executable specification
2. Can be formally proven correct
3. Enables differential testing against Rust implementation
4. Provides high-confidence oracle for semantics validation

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    DIFFERENTIAL TESTING                          │
│                                                                  │
│   ┌─────────────────────┐      ┌─────────────────────┐         │
│   │   Rust Interpreter  │      │   Lean Reference    │         │
│   │   (Production)      │      │   (Specification)   │         │
│   │                     │      │                     │         │
│   │  - Optimized        │      │  - Direct from spec │         │
│   │  - Feature-complete │      │  - Provably correct │         │
│   │  - May have bugs    │◄────►│  - Slower           │         │
│   │                     │      │  - Trust anchor     │         │
│   └──────────┬──────────┘      └──────────┬──────────┘         │
│              │                            │                     │
│              └────────────┬───────────────┘                     │
│                           │                                     │
│                    ┌──────▼──────┐                             │
│                    │   Compare   │                             │
│                    │   Results   │                             │
│                    └─────────────┘                             │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Why Lean?

| Feature | Benefit |
|---------|---------|
| **Dependent types** | Express invariants in types (e.g., "exhaustive patterns") |
| **Theorem proving** | Prove soundness, completeness, termination |
| **Executable** | Can run programs for testing |
| **Extraction** | Generate verified code (Rust/OCaml) |
| **Metaprogramming** | Generate test cases, proofs automatically |

## Lean Implementation Structure

### 1. Core Types

```lean4
-- AST definitions mirror SPEC-020
inductive Type : Type
  | int : Type
  | string : Type
  | bool : Type
  | list : Type → Type
  | record : List (String × Type) → Type
  | sum : String → List TypeVar → List Variant → Type
  | struct : String → List TypeVar → List (String × Type) → Type
  | constructor : String → List Type → Type
  | var : TypeVar → Type
deriving Repr, BEq

structure Variant where
  name : String
  fields : List (String × Type)
deriving Repr, BEq

inductive Expr : Type
  | literal : Value → Expr
  | variable : String → Expr
  | constructor : String → List (String × Expr) → Expr
  | match : Expr → List MatchArm → Expr
  | if_let : Pattern → Expr → Expr → Expr → Expr
  | tuple : List Expr → Expr
  | call : String → List Expr → Expr
deriving Repr, BEq

structure MatchArm where
  pattern : Pattern
  body : Expr
deriving Repr, BEq

inductive Pattern : Type
  | wildcard : Pattern
  | variable : String → Pattern
  | literal : Value → Pattern
  | variant : String → List (String × Pattern) → Pattern
  | tuple : List Pattern → Pattern
  | record : List (String × Pattern) → Pattern
deriving Repr, BEq

inductive Value : Type
  | int : Int → Value
  | string : String → Value
  | bool : Bool → Value
  | null : Value
  | list : List Value → Value
  | record : List (String × Value) → Value
  | variant : String → String → List (String × Value) → Value
  | struct : String → List (String × Value) → Value
  | tuple : List Value → Value
  | instance : InstanceAddr → ControlLink → Value
deriving Repr, BEq
```

### 2. Operational Semantics (Executable)

```lean4
-- Big-step semantics directly from SPEC-004

-- Context: Γ (environment)
def Env := String → Option Value

def Env.empty : Env := fun _ => none
def Env.bind (env : Env) (x : String) (v : Value) : Env :=
  fun y => if x = y then some v else env y

-- Pattern matching with proof of totality
partial def matchPattern (p : Pattern) (v : Value) : Option Env :=
  match p, v with
  | .wildcard, _ => some Env.empty
  | .variable x, v => some (Env.bind Env.empty x v)
  | .literal l, v => if l = v then some Env.empty else none
  | .variant name fields, .variant _ vname vfields =>
      if name = vname then
        matchFields fields vfields
      else
        none
  | .tuple ps, .tuple vs =>
      if ps.length = vs.length then
        matchList ps vs
      else
        none
  | .record fields, .struct _ _ sfields =>
      matchRecord fields sfields
  | _, _ => none

partial def matchFields (ps : List (String × Pattern)) (vs : List (String × Value)) : Option Env :=
  match ps with
  | [] => some Env.empty
  | (name, p) :: rest =>
      match vs.find? (fun (n, _) => n = name) with
      | none => none
      | some (_, v) =>
          match matchPattern p v with
          | none => none
          | some env1 =>
              match matchFields rest vs with
              | none => none
              | some env2 => some (mergeEnvs env1 env2)

-- The interpreter (executable specification)
partial def eval (env : Env) : Expr → Except Error (Value × Effect × Trace)
  | .literal v => pure (v, .epistemic, .empty)
  | .variable x =>
      match env x with
      | none => throw (.unboundVariable x)
      | some v => pure (v, .epistemic, .empty)
  | .constructor name fields => do
      let values ← fields.mapM (fun (n, e) => do
        let (v, _, _) ← eval env e
        pure (n, v))
      pure (.variant "" name values, .epistemic, .empty)
  | .match scrutinee arms => do
      let (sv, se, st) ← eval env scrutinee
      -- Find matching arm
      match findMatchingArm arms sv with
      | none => throw .nonExhaustiveMatch
      | some (arm, bindings) => do
          let newEnv := mergeEnvs env bindings
          let (bv, be, bt) ← eval newEnv arm.body
          pure (bv, se ⊔ be, st ++ bt)
  | .if_let pattern expr thenBranch elseBranch => do
      let (ev, ee, et) ← eval env expr
      match matchPattern pattern ev with
      | some bindings => do
          let newEnv := mergeEnvs env bindings
          let (tv, te, tt) ← eval newEnv thenBranch
          pure (tv, ee ⊔ te, et ++ tt)
      | none => do
          let (ev', ee', et') ← eval env elseBranch
          pure (ev', ee ⊔ ee', et ++ et')
  | .tuple exprs => do
      let results ← exprs.mapM (eval env)
      let values := results.map (fun (v, _, _) => v)
      let effect := results.foldl (fun acc (_, e, _) => acc ⊔ e) .epistemic
      let trace := results.foldl (fun acc (_, _, t) => acc ++ t) .empty
      pure (.tuple values, effect, trace)
  -- ... other cases

-- Helper: Find matching arm (returns arm + bindings)
def findMatchingArm (arms : List MatchArm) (value : Value) : Option (MatchArm × Env) :=
  arms.findSome? (fun arm =>
    match matchPattern arm.pattern value with
    | none => none
    | some bindings => some (arm, bindings))
```

### 3. Proving Correctness

```lean4
-- Theorem: Well-typed programs make progress
-- (If type checking succeeds, evaluation doesn't get stuck)

def WellTyped (env : Env) (e : Expr) (τ : Type) : Prop :=
  -- Type checking relation (from SPEC-003)
  TypeCheck.env ⊢ e : τ

-- Progress theorem: Well-typed expressions either are values or can step
theorem progress {env : Env} {e : Expr} {τ : Type}
  (h : WellTyped env e τ) :
  (∃ v, e = Expr.literal v) ∨ (∃ e', Step env e e') := by
  -- Proof by case analysis on typing derivation
  cases h with
  | literal => left; aesop
  | variable =>
      -- Variable must be in environment for well-typedness
      right
      -- Can evaluate to its binding
      sorry
  | match_expr h_scrut h_arms =>
      -- Scrutinee is well-typed, so it makes progress
      cases progress h_scrut with
      | inl h_val =>
          -- Scrutinee is a value
          -- By exhaustiveness (from h_arms), there's a matching arm
          right
          sorry
      | inr h_step =>
          -- Scrutinee can step
          right
          sorry
  -- ... other cases

-- Theorem: Pattern matching is deterministic
theorem match_deterministic {p : Pattern} {v : Value} {env1 env2 : Env}
  (h1 : matchPattern p v = some env1)
  (h2 : matchPattern p v = some env2) :
  env1 = env2 := by
  -- Unfold matchPattern and prove by structural induction
  induction p generalizing v env1 env2 with
  | wildcard => simp [matchPattern] at h1 h2; simp [h1] at h2; exact h2
  | variable x => simp [matchPattern] at h1 h2; simp [h1] at h2; exact h2
  | literal l =>
      simp [matchPattern] at h1 h2
      split at h1 <;> split at h2 <;> simp [*] at *
      <;> try { contradiction }
      <;> { simp [h1] at h2; exact h2 }
  | variant name fields ih =>
      simp [matchPattern] at h1 h2
      cases v with
      | variant _ vname vfields =>
          by_cases name = vname
          · -- Names match
            simp [matchFields, *] at h1 h2
            sorry -- Use induction hypothesis for fields
          · -- Names don't match
            simp [*] at h1 h2
            contradiction
      | _ => simp [matchPattern] at h1 h2; contradiction
  -- ... other cases

-- Theorem: Evaluation preserves types (Preservation)
theorem preservation {env : Env} {e e' : Expr} {τ : Type}
  (h_type : WellTyped env e τ)
  (h_step : Step env e e') :
  WellTyped env e' τ := by
  -- Proof by case analysis on evaluation step
  sorry

-- Corollary: Type safety (combination of Progress + Preservation)
theorem type_safety {env : Env} {e : Expr} {τ : Type}
  (h : WellTyped env e τ) :
  -- Evaluation never gets stuck
  ∃ v, EvaluatesTo env e v := by
  sorry
```

### 4. Differential Testing Harness

```lean4
-- Generate random workflows that type-check

open Lean Elab Meta

-- Generator for well-typed expressions
def genWellTypedExpr (env : TypeEnv) (expectedType : Type) : Gen Expr := do
  match expectedType with
  | .int => Expr.literal ∘ Value.int <$> Gen.int
  | .string => Expr.literal ∘ Value.string <$> Gen.string
  | .bool => Expr.literal ∘ Value.bool <$> Gen.bool
  | .constructor "Option" [t] =>
      -- Generate Some or None
      Gen.oneOf [
        do let v ← genWellTypedExpr env t
           pure <| Expr.constructor "Some" [("value", v)],
        pure <| Expr.constructor "None" []
      ]
  | .constructor "Result" [t, e] =>
      Gen.oneOf [
        do let v ← genWellTypedExpr env t
           pure <| Expr.constructor "Ok" [("value", v)],
        do let err ← genWellTypedExpr env e
           pure <| Expr.constructor "Err" [("error", err)]
      ]
  | .sum name _ variants =>
      -- Generate one of the variants
      let variant ← Gen.elements variants
      let fields ← variant.fields.mapM (fun (n, t) => do
        let e ← genWellTypedExpr env t
        pure (n, e))
      pure <| Expr.constructor variant.name fields
  | _ => throw "Unsupported type"

-- Differential test
def differentialTest (workflow : Workflow) : IO (Option DiffResult) := do
  -- Run Lean reference interpreter
  let leanResult ← IO.ofExcept (eval Env.empty workflow.expr)
  
  -- Serialize workflow for Rust
  let json := toJson workflow
  
  -- Run Rust interpreter (via subprocess)
  let rustOutput ← runRustInterpreter json
  let rustResult : Except Error (Value × Effect × Trace) :=
    fromJson rustOutput
  
  -- Compare results
  match leanResult, rustResult with
  | .ok (lv, le, lt), .ok (rv, re, rt) =>
      if lv = rv ∧ le = re then
        pure none -- Match!
      else
        pure <| some {
          workflow := workflow,
          lean := (lv, le, lt),
          rust := (rv, re, rt),
          difference := computeDiff lv rv
        }
  | .error le, .error re =>
      if le = re then
        pure none -- Both failed with same error
      else
        pure <| some {
          workflow := workflow,
          leanError := le,
          rustError := re
        }
  | .ok leanRes, .error rustErr =>
      pure <| some {
        workflow := workflow,
        leanResult := leanRes,
        rustError := rustErr
      }
  | .error leanErr, .ok rustRes =>
      pure <| some {
        workflow := workflow,
        leanError := leanErr,
        rustResult := rustRes
      }

-- Property: Results should always match
#check ∀ (w : Workflow), WellTyped w → differentialTest w = none
```

### 5. CI Integration

```yaml
# .github/workflows/differential-testing.yml
name: Differential Testing

on: [push, pull_request, schedule]

jobs:
  lean-reference:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Lean
        uses: leanprover/lean-action@v1
        with:
          lake-version: latest
      
      - name: Build reference interpreter
        run: lake build
      
      - name: Run proofs
        run: lake exe check_proofs
      
      - name: Generate test corpus
        run: lake exe generate_tests --count 10000 --output tests/

  differential:
    needs: lean-reference
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Build Rust interpreter
        run: cargo build --release
      
      - name: Download test corpus
        uses: actions/download-artifact@v3
        with:
          name: test-corpus
      
      - name: Run differential tests
        run: |
          ./scripts/differential_test.sh \
            --lean ./lean_reference \
            --rust ./target/release/ash_interp \
            --corpus ./tests/ \
            --timeout 3600
```

### 6. Minimizing Discrepancies

When differential testing finds a mismatch:

```lean4
-- Test case minimization using Lean's metaprogramming

def minimizeTestCase (workflow : Workflow) : IO Workflow := do
  -- Try removing parts of the workflow while preserving the discrepancy
  let mut minimized := workflow
  
  -- Try simplifying expressions
  for expr in workflow.subexpressions do
    let simpler := simplify expr
    let testWorkflow := workflow.replace expr simpler
    if discrepancyPersists testWorkflow then
      minimized := testWorkflow
  
  -- Try removing unnecessary bindings
  for binding in workflow.bindings do
    let withoutBinding := workflow.removeBinding binding
    if discrepancyPersists withoutBinding then
      minimized := withoutBinding
  
  pure minimized

-- Pretty print minimal counterexample
def formatCounterexample (result : DiffResult) : String :=
  s!"
Differential Test Failure:

Workflow:
{result.workflow}

Lean Result:
  Value: {result.lean.1}
  Effect: {result.lean.2}

Rust Result:
  Value: {result.rust.1}
  Effect: {result.rust.2}

Difference: {result.difference}
"
```

## Benefits

### 1. High Confidence
- Lean interpreter proven correct → Rust bugs caught immediately
- Formal proofs complement testing
- Reference is specification, not just oracle

### 2. Bug Localization
- Discrepancy → look at Rust implementation
- Minimized counterexamples
- Clear semantics violation

### 3. Specification Evolution
- Update Lean first (spec change)
- Proofs guide implementation
- Rust catches up to proven spec

### 4. Documentation
- Executable specification
- Proofs are machine-checked documentation
- Reference for semantics questions

## Trade-offs

| Aspect | Lean Reference | Pure Testing |
|--------|---------------|--------------|
| **Setup cost** | High (learn Lean, implement) | Low |
| **Running cost** | Medium (slower interpreter) | Low |
| **Confidence** | Very High | Medium |
| **Bug finding** | Caught immediately | Over time |
| **Maintenance** | Keep in sync with spec | Update tests |
| **Proof burden** | Yes | No |

## Recommendation

**Phase 1: Core ADT (Now)**
- Implement Lean reference for pattern matching + constructors
- Prove exhaustiveness checking soundness
- Differential test against Rust

**Phase 2: Full Semantics (Month 2)**
- Extend to full workflow semantics
- Prove type safety theorem
- Continuous differential testing in CI

**Phase 3: Evolution (Ongoing)**
- New features: implement in Lean first
- Prove properties before Rust implementation
- Use as teaching/documentation tool

## Getting Started

```bash
# 1. Setup Lean 4 project
lake new ash_reference

# 2. Implement core types (mirror Rust AST)
# Ash/Core.lean

# 3. Implement interpreter
# Ash/Interpreter.lean

# 4. Prove key theorems
# Ash/Theorems.lean

# 5. Generate tests
# Ash/TestGen.lean

# 6. Differential testing script
# scripts/differential_test.sh
```

This approach gives you the best of both worlds:
- **Testing** for regression prevention
- **Proofs** for foundational correctness
- **Differential testing** for implementation validation
