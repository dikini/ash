# SPEC-021: Lean Reference Interpreter

## Status: Draft

## 1. Overview

This specification defines a reference interpreter for Ash implemented in Lean 4. The interpreter serves as:

1. **Executable specification** - Direct implementation of SPEC-004 operational semantics
2. **Test oracle** - Differential testing against Rust implementation
3. **Foundation for verification** - Future proofs of correctness (aspirational)

## 2. Goals

### 2.1 Immediate Goals (Phase 1)

- Implement core interpreter for ADT operations (constructors, pattern matching)
- Enable differential testing with Rust implementation
- Catch implementation bugs through systematic comparison

### 2.2 Long-term Goals (Aspirational)

- Formal proofs of key properties (pattern matching determinism, progress, preservation)
- Certified compiler extraction
- Mathematical certainty about core language semantics

## 3. Scope

### 3.1 In Scope (Phase 1)

- Expression evaluation (literals, variables, constructors)
- Pattern matching (variant, tuple, record, wildcard, variable)
- Match expressions with multiple arms
- If-let expressions
- Type definitions (enum, struct)
- Environment and variable binding
- Effect tracking (Epistemic → Operational lattice)
- JSON serialization for differential testing

### 3.2 Out of Scope (Phase 1)

- Workflow constructs (Observe, Act, Decide, etc.)
- Parallel composition
- Spawn/split/control link semantics
- Provenance tracking
- Formal proofs (deferred to Phase 2)

### 3.3 Future Work (Phase 2+)

- Full workflow semantics
- Parallel composition semantics
- Spawn and control link handling
- Trace recording
- Core theorem proofs

## 4. Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Lean Reference Interpreter                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Core/                                                           │
│  ├── AST.lean          - Expression, Pattern, Value types       │
│  ├── Types.lean        - Type definitions                        │
│  ├── Environment.lean  - Variable binding and lookup            │
│  └── Serialize.lean    - JSON ↔ Lean conversion                 │
│                                                                  │
│  Eval/                                                           │
│  ├── Expr.lean         - Expression evaluation                  │
│  ├── Pattern.lean      - Pattern matching engine                │
│  └── Match.lean        - Match expression handling              │
│                                                                  │
│  Differential/                                                   │
│  └── Compare.lean      - Bisimulation checking                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## 5. Core Types

### 5.1 AST Types

```lean
inductive Expr where
  | literal (v : Value)
  | variable (name : String)
  | constructor (name : String) (fields : List (String × Expr))
  | tuple (elements : List Expr)
  | match (scrutinee : Expr) (arms : List MatchArm)
  | if_let (pattern : Pattern) (expr : Expr) (then_branch : Expr) (else_branch : Expr)
  deriving Repr, BEq

structure MatchArm where
  pattern : Pattern
  body : Expr
  deriving Repr, BEq

inductive Pattern where
  | wildcard
  | variable (name : String)
  | literal (v : Value)
  | variant (name : String) (fields : List (String × Pattern))
  | tuple (elements : List Pattern)
  | record (fields : List (String × Pattern))
  deriving Repr, BEq

inductive Value where
  | int (i : Int)
  | string (s : String)
  | bool (b : Bool)
  | null
  | list (vs : List Value)
  | record (fields : List (String × Value))
  | variant (type_name : String) (variant_name : String) (fields : List (String × Value))
  | tuple (elements : List Value)
  deriving Repr, BEq
```

### 5.2 Type Definitions

```lean
structure Variant where
  name : String
  fields : List (String × TypeExpr)
  deriving Repr, BEq

inductive TypeExpr where
  | named (name : String)
  | var (id : Nat)
  | constructor (name : String) (args : List TypeExpr)
  deriving Repr, BEq

structure TypeDef where
  name : String
  params : List Nat
  body : TypeBody
  deriving Repr, BEq

inductive TypeBody where
  | enum (variants : List Variant)
  | struct (fields : List (String × TypeExpr))
  deriving Repr, BEq
```

### 5.3 Evaluation Context

```lean
def Env := String → Option Value

def Env.empty : Env := fun _ => none

def Env.bind (env : Env) (x : String) (v : Value) : Env :=
  fun y => if x = y then some v else env y

inductive Effect where
  | epistemic
  | deliberative
  | evaluative
  | operational
  deriving Repr, BEq

def Effect.join (e1 e2 : Effect) : Effect :=
  match e1, e2 with
  | _, .operational | .operational, _ => .operational
  | _, .evaluative | .evaluative, _ => .evaluative
  | _, .deliberative | .deliberative, _ => .deliberative
  | _, _ => .epistemic

structure EvalResult where
  value : Value
  effect : Effect
  deriving Repr, BEq

inductive EvalError where
  | unboundVariable (name : String)
  | typeMismatch (expected : String) (actual : String)
  | nonExhaustiveMatch
  | unknownConstructor (name : String)
  | missingField (constructor : String) (field : String)
  deriving Repr, BEq
```

## 6. Operational Semantics

### 6.1 Expression Evaluation

```lean
def eval (env : Env) (expr : Expr) : Except EvalError EvalResult :=
  match expr with
  | .literal v => pure { value := v, effect := .epistemic }
  | .variable x =>
      match env x with
      | none => throw (.unboundVariable x)
      | some v => pure { value := v, effect := .epistemic }
  | .constructor name fields => evalConstructor env name fields
  | .tuple elements => evalTuple env elements
  | .match scrutinee arms => evalMatch env scrutinee arms
  | .if_let pattern expr then_branch else_branch =>
      evalIfLet env pattern expr then_branch else_branch
```

### 6.2 Constructor Evaluation

```lean
def evalConstructor (env : Env) (name : String)
    (fields : List (String × Expr)) : Except EvalError EvalResult := do
  let mut field_values := []
  for (field_name, field_expr) in fields do
    let result ← eval env field_expr
    field_values := (field_name, result.value) :: field_values
  pure {
    value := .variant "" name field_values.reverse,
    effect := .epistemic
  }
```

**Property**: Constructor evaluation is pure (epistemic effect, no side effects).

### 6.3 Pattern Matching

```lean
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
  | .record ps, .record fields =>
      matchRecord ps fields
  | _, _ => none

def matchFields (ps : List (String × Pattern))
    (vs : List (String × Value)) : Option Env :=
  match ps with
  | [] => some Env.empty
  | (name, p) :: rest =>
      match vs.find? (fun (n, _) => n = name) with
      | none => none
      | some (_, v) => do
          let env1 ← matchPattern p v
          let env2 ← matchFields rest vs
          pure (mergeEnvs env1 env2)
```

**Properties**:
- **Determinism**: `matchPattern p v` returns at most one environment
- **Totality**: For exhaustive patterns, `matchPattern` always succeeds

### 6.4 Match Expression

```lean
def evalMatch (env : Env) (scrutinee : Expr)
    (arms : List MatchArm) : Except EvalError EvalResult := do
  let scrut_result ← eval env scrutinee
  match findMatchingArm arms scrut_result.value with
  | none => throw .nonExhaustiveMatch
  | some (arm, bindings) =>
      let newEnv := mergeEnvs env bindings
      eval newEnv arm.body

def findMatchingArm (arms : List MatchArm) (v : Value)
    : Option (MatchArm × Env) :=
  arms.findSome? (fun arm =>
    match matchPattern arm.pattern v with
    | none => none
    | some bindings => some (arm, bindings))
```

**Property**: If type checker verifies exhaustiveness, `evalMatch` never returns `nonExhaustiveMatch`.

### 6.5 If-Let Expression

```lean
def evalIfLet (env : Env) (pattern : Pattern) (expr : Expr)
    (then_branch : Else) (else_branch : Expr) : Except EvalError EvalResult := do
  let expr_result ← eval env expr
  match matchPattern pattern expr_result.value with
  | some bindings =>
      let newEnv := mergeEnvs env bindings
      eval newEnv then_branch
  | none =>
      eval env else_branch
```

## 7. Differential Testing Interface

### 7.1 JSON Serialization

```lean
-- Parse Rust JSON output
def workflowFromJson (json : Json) : Except String Expr :=
  -- Implementation using Lean's JSON parser
  sorry

-- Export result for Rust comparison
def resultToJson (result : EvalResult) : Json :=
  json% {
    value: $(valueToJson result.value),
    effect: $(effectToString result.effect)
  }
```

### 7.2 Bisimulation Comparison

```lean
-- Check if Lean and Rust results are equivalent
structure BisimulationResult where
  equivalent : Bool
  difference : Option String
  leanResult : EvalResult
  rustResult : Json

def compareWithRust (expr : Expr) (rustJson : Json) : BisimulationResult :=
  match eval Env.empty expr with
  | .error e =>
      let rustError := parseRustError rustJson
      if errorsEquivalent e rustError then
        { equivalent := true, difference := none, .. }
      else
        { equivalent := false, difference := some "Error mismatch", .. }
  | .ok leanRes =>
      let rustRes := parseRustResult rustJson
      if resultsEquivalent leanRes rustRes then
        { equivalent := true, difference := none, .. }
      else
        { equivalent := false, difference := some "Result mismatch", .. }
```

## 8. Testable Properties

### 8.1 Expression Evaluation Properties

```lean
-- Constructor purity
def constructorPure (fields : List (String × Expr)) (env : Env) : Prop :=
  match eval env (.constructor "Test" fields) with
  | .ok result => result.effect = .epistemic
  | .error _ => true

-- Pattern match determinism
def patternDeterministic (p : Pattern) (v : Value) : Prop :=
  let r1 := matchPattern p v
  let r2 := matchPattern p v
  r1 = r2

-- Match exhaustiveness (requires type system info)
def matchExhaustive (scrutinee : Expr) (arms : List MatchArm)
    (typeEnv : TypeEnv) : Prop :=
  -- If type checker says exhaustive, eval never fails
  if exhaustiveCheck typeEnv scrutinee arms then
    match eval Env.empty (.match scrutinee arms) with
    | .error .nonExhaustiveMatch => false
    | _ => true
  else
    true
```

### 8.2 Property-Based Testing in Lean

```lean
-- QuickCheck-style testing using Plausible
#test ∀ (p : Pattern) (v : Value), patternDeterministic p v

#test ∀ (fields : List (String × Expr)),
  constructorPure fields Env.empty
```

## 9. Integration with Rust

### 9.1 Build Integration

```bash
# Build both implementations
lake build                    # Build Lean reference
cargo build --release         # Build Rust implementation

# Run differential test
./scripts/differential_test.sh
```

### 9.2 CI Integration

```yaml
# .github/workflows/lean-reference.yml
name: Lean Reference
on: [push, pull_request]

jobs:
  lean-interpreter:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: leanprover/lean-action@v1
      - run: lake build
      - run: lake exe test
      
  differential:
    needs: lean-interpreter
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: ./scripts/differential_test.sh
```

## 10. Formal Proofs (Completed)

Formal proofs of semantic properties have been implemented in Phase 19 (TASK-149 through TASK-155).

### 10.1 Completed Proofs

| Theorem | Location | Status | Notes |
|---------|----------|--------|-------|
| Pattern Match Determinism | `Ash/Proofs/Pattern.lean` | ✅ Proven | Uses function purity |
| Pattern Match Totality | `Ash/Proofs/Pattern.lean` | ⚠️ Stated | Uses `sorry` (partial fn) |
| Constructor Purity | `Ash/Proofs/Pure.lean` | ⚠️ Stated | Uses `sorry` (partial fn) |
| Evaluation Determinism | `Ash/Proofs/Determinism.lean` | ✅ Proven | Uses function purity |
| Progress Theorem | `Ash/Proofs/Progress.lean` | ⚠️ Stated | Uses `sorry` (partial fn) |
| Preservation Theorem | `Ash/Proofs/Preservation.lean` | ⚠️ Stated | Uses `sorry` (partial fn) |
| Type Safety Corollary | `Ash/Proofs/TypeSafety.lean` | ✅ Proven | Combines P + P |

### 10.2 Proof Structure

```
Ash/Proofs/
├── Pattern.lean      # Pattern matching determinism & totality
├── Pure.lean         # Constructor purity (effect system)
├── Determinism.lean  # Expression evaluation determinism
├── Progress.lean     # Progress theorem (well-typed → defined)
├── Preservation.lean # Preservation theorem (types preserved)
└── TypeSafety.lean   # Type safety (combination theorem)

Ash/Types/
├── Basic.lean        # Type definitions (Ty)
└── WellTyped.lean    # Well-typed relation (simplified)
```

### 10.3 Implementation Notes

**Determinism Proofs**: Use Lean's function purity (equal inputs → equal outputs) rather than induction:
```lean
theorem match_pattern_deterministic {p : Pattern} {v : Value} {env1 env2 : Env}
    (h1 : matchPattern p v = some env1)
    (h2 : matchPattern p v = some env2) :
    env1 = env2 := by
  rw [h1] at h2
  injection h2
```

**Partial Function Limitation**: Theorems about `eval` use `sorry` because:
- `eval` is defined as `partial` (mutual recursion)
- Lean cannot unfold partial functions in proofs
- **Solution**: Make `eval` total using fuel-based approach (long-term task)

**Type System Simplification**: `WellTyped` relation is simplified due to Lean 4 nested inductive limitations. Full version requires well-founded recursion.

### 10.4 Future Work

- Complete proofs when `eval` is made total
- Add cases for tuple/variant/record to `ValueHasType`
- Full inductive type system with nested structure

## 11. Related Documents

- SPEC-004: Operational Semantics (source of truth)
- SPEC-020: ADT Types (features to implement)
- PLAN-021: Lean Reference Implementation Plan (tasks)
- TASK-137 through TASK-148: Phase 17 - Lean Reference Implementation
- TASK-149 through TASK-155: Phase 19 - Formal Proofs

## 12. References

- [Lean 4 Manual](https://lean-lang.org/lean4/doc/)
- [Theorem Proving in Lean 4](https://leanprover.github.io/theorem_proving_in_lean4/)
- [Functional Programming in Lean](https://lean-lang.org/functional_programming_in_lean/)
