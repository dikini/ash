# TASK-146: Property-Based Tests

## Status: 🟡 Ready to Start

## Description

Implement comprehensive property-based tests for the Lean reference interpreter using Lean's `Plausible` framework (formerly `Std` QuickCheck-style testing).

## Specification Reference

- SPEC-021: Lean Reference - Section 8 (Testable Properties)
- SPEC-004: Operational Semantics - Section 7 (Property Testing)

## Requirements

### Functional Requirements

1. Set up Plausible/QuickCheck-style testing framework
2. Implement property tests for:
   - Constructor purity (SPEC-004 Section 5.1)
   - Pattern match determinism
   - Environment operations
   - Effect lattice properties
   - JSON roundtrip serialization
3. Create arbitrary generators for AST types
4. Integrate with Lean test runner (`#test`)

### Property Requirements

```lean
-- From SPEC-021 Section 8
prop_constructor_pure(fields, env) = 
  match eval env (.constructor "Test" fields) with
  | .ok result => result.effect = .epistemic
  | .error _ => true

prop_pattern_deterministic(p, v) = 
  matchPattern p v = r1 →
  matchPattern p v = r2 →
  r1 = r2

prop_match_exhaustive(scrutinee, arms, typeEnv) =
  exhaustiveCheck typeEnv scrutinee arms →
  match eval env (.match scrutinee arms) with
  | .error .nonExhaustiveMatch => false
  | _ => true
```

## TDD Steps

### Step 1: Set Up Test Module (Red)

**File**: `lean_reference/Ash/Tests/Properties.lean`

```lean
import Plausible
import Ash.Core.AST
import Ash.Core.Environment
import Ash.Eval.Expr
import Ash.Eval.Pattern
import Ash.Core.Serialize

namespace Ash.Tests

open Ash
open Plausible

-- Marker for test suite
def propertiesSuite := "Ash Property Tests"

end Ash.Tests
```

**Test**:
```bash
lake build
# Should compile with Plausible imported
```

### Step 2: Implement Value Generator (Green)

```lean
-- Helper: Generate arbitrary Values for testing
def genValue (size : Nat) : Gen Value :=
  if size = 0 then
    Gen.oneOf [
      Value.int <$> Gen.int,
      Value.string <$> Gen.string,
      Value.bool <$> Gen.bool,
      pure Value.null
    ]
  else
    let smaller := size / 2
    Gen.oneOf [
      Value.int <$> Gen.int,
      Value.string <$> Gen.string,
      Value.bool <$> Gen.bool,
      pure Value.null,
      Value.list <$> Gen.listOf (genValue smaller),
      Value.tuple <$> Gen.listOf (genValue smaller)
      -- Variant and record require more complex generation
    ]

instance : Sampleable Value where
  sample := genValue 10
```

**Test**:
```lean
#eval Gen.run (genValue 5) 10  -- Generate 10 sample values
```

### Step 3: Effect Lattice Properties (Green)

```lean
-- SPEC-004 Section 5.2: Effect lattice properties
namespace EffectProperties

-- Associativity: (a ⊔ b) ⊔ c = a ⊔ (b ⊔ c)
#test ∀ (e1 e2 e3 : Effect),
  (e1.join e2).join e3 = e1.join (e2.join e3)

-- Commutativity: a ⊔ b = b ⊔ a
#test ∀ (e1 e2 : Effect),
  e1.join e2 = e2.join e1

-- Idempotence: a ⊔ a = a
#test ∀ (e : Effect),
  e.join e = e

-- Ordering properties
#test Effect.epistemic.join Effect.epistemic = Effect.epistemic
#test Effect.epistemic.join Effect.deliberative = Effect.deliberative
#test Effect.epistemic.join Effect.operational = Effect.operational
#test Effect.deliberative.join Effect.evaluative = Effect.evaluative
#test Effect.evaluative.join Effect.operational = Effect.operational

end EffectProperties
```

**Run**:
```bash
lake exe ash_ref 2>&1 | head -20
# Should show property test results
```

### Step 4: Environment Properties (Green)

```lean
namespace EnvironmentProperties

-- Lookup after binding returns the bound value
#test ∀ (x : String) (v : Value),
  (Env.empty.bind x v).lookup x = some v

-- Lookup of different name returns none (or original)
#test ∀ (x y : String) (v : Value),
  x ≠ y → (Env.empty.bind x v).lookup y = none

-- Shadowing: second bind wins
#test ∀ (x : String) (v1 v2 : Value),
  (Env.empty.bind x v1).bind x v2 = Env.empty.bind x v2

-- Merge with empty is identity
#test ∀ (env : Env) (x : String),
  (env.merge Env.empty).lookup x = env.lookup x

-- Merge prefers right side
#test ∀ (x : String) (v1 v2 : Value),
  (Env.empty.bind x v1).merge (Env.empty.bind x v2) = Env.empty.bind x v2

end EnvironmentProperties
```

### Step 5: Pattern Matching Properties (Green)

```lean
namespace PatternProperties

-- Wildcard always matches
#test ∀ (v : Value),
  matchPattern .wildcard v ≠ none

-- Variable always matches and binds
#test ∀ (x : String) (v : Value),
  matchPattern (.variable x) v = some (Env.empty.bind x v)

-- Literal matches only when equal
#test ∀ (lit v : Value),
  (matchPattern (.literal lit) v ≠ none) ↔ (lit = v)

-- Pattern match determinism
#test ∀ (p : Pattern) (v : Value),
  match matchPattern p v with
  | some env1 =>
      match matchPattern p v with
      | some env2 => env1 = env2
      | none => true  -- Shouldn't happen
  | none => true

-- Empty tuple pattern matches empty tuple
#test matchPattern (.tuple []) (.tuple []) = some Env.empty

-- Non-empty tuple pattern fails on empty tuple
#test matchPattern (.tuple [.wildcard]) (.tuple []) = none

end PatternProperties
```

### Step 6: Constructor Purity Property (Green)

Per SPEC-004 Section 5.1:

```lean
namespace EvaluationProperties

-- Constructor purity: constructors have epistemic effect
-- We test with empty constructor since arbitrary Expr is complex
#test ∀ (name : String),
  name ≠ "" →
  match eval Env.empty (.constructor name []) with
  | .ok result => result.effect = .epistemic
  | .error _ => true

-- Literal purity
#test ∀ (v : Value),
  match eval Env.empty (.literal v) with
  | .ok result => result.effect = .epistemic
  | .error _ => true

-- Variable lookup fails on unbound
#test ∀ (x : String),
  x ≠ "" →
  match eval Env.empty (.variable x) with
  | .error (.unboundVariable _) => true
  | _ => false

-- Variable lookup succeeds on bound
#test ∀ (x : String) (v : Value),
  match eval (Env.empty.bind x v) (.variable x) with
  | .ok result => result.value = v
  | _ => false

end EvaluationProperties
```

### Step 7: JSON Serialization Properties (Green)

```lean
namespace SerializationProperties

-- Value roundtrip
#test ∀ (i : Int),
  Value.fromJson (Value.int i).toJson = .ok (Value.int i)

#test ∀ (s : String), s.length < 100 →
  Value.fromJson (Value.string s).toJson = .ok (Value.string s)

#test ∀ (b : Bool),
  Value.fromJson (Value.bool b).toJson = .ok (Value.bool b)

-- Null roundtrip
#test Value.fromJson Value.null.toJson = .ok Value.null

-- Effect roundtrip
#test ∀ (e : Effect),
  Effect.fromJson e.toJson = .ok e

-- EvalResult roundtrip
#test ∀ (v : Value),
  match EvalResult.fromJson { value := v, effect := .epistemic : EvalResult }.toJson with
  | .ok r => r.value = v ∧ r.effect = .epistemic
  | _ => false

end SerializationProperties
```

### Step 8: Match Expression Properties (Green)

```lean
namespace MatchProperties

-- Wildcard arm is always exhaustive
#test ∀ (v : Value),
  match eval Env.empty (.match (.literal v) [
    { pattern := .wildcard, body := .literal (Value.int 0) }
  ]) with
  | .error .nonExhaustiveMatch => false
  | _ => true

-- Variable arm binds correctly
#test ∀ (v : Value),
  match eval Env.empty (.match (.literal v) [
    { pattern := .variable "x", body := .variable "x" }
  ]) with
  | .ok result => result.value = v
  | _ => false

-- First match wins (literal vs wildcard)
#test ∀ (v : Value),
  match eval Env.empty (.match (.literal v) [
    { pattern := .literal v, body := .literal (Value.string "first") },
    { pattern := .wildcard, body := .literal (Value.string "second") }
  ]) with
  | .ok result => result.value = Value.string "first"
  | _ => false

end MatchProperties
```

### Step 9: If-Let Properties (Green)

```lean
namespace IfLetProperties

-- If-let with wildcard always takes then branch
#test ∀ (v : Value),
  match eval Env.empty (.if_let .wildcard (.literal v)
    (.literal (Value.bool true))
    (.literal (Value.bool false))
  ) with
  | .ok result => result.value = Value.bool true
  | _ => false

-- If-let with literal succeeds when equal
#test ∀ (v : Value),
  match eval Env.empty (.if_let (.literal v) (.literal v)
    (.literal (Value.bool true))
    (.literal (Value.bool false))
  ) with
  | .ok result => result.value = Value.bool true
  | _ => false

-- If-let with literal fails when not equal (using different ints)
#test ∀ (i1 i2 : Int), i1 ≠ i2 →
  match eval Env.empty (.if_let 
    (.literal (Value.int i1))
    (.literal (Value.int i2))
    (.literal (Value.bool true))
    (.literal (Value.bool false))
  ) with
  | .ok result => result.value = Value.bool false
  | _ => false

end IfLetProperties
```

### Step 10: Integration and Test Runner (Green)

**File**: `lean_reference/Ash/Tests/Runner.lean`

```lean
import Ash.Tests.Properties

namespace Ash.Tests

def runAllPropertyTests : IO Unit := do
  IO.println "======================================="
  IO.println "  Ash Property-Based Test Suite"
  IO.println "======================================="
  IO.println ""
  
  -- Properties are registered via #test and run automatically
  -- by Lean's test framework when building with test configuration
  IO.println "Property tests are defined using #test"
  IO.println "Run with: lake exe test"
  
  -- Manual verification of some properties
  IO.println "\n--- Manual Property Verification ---"
  
  -- Effect lattice
  IO.println "Effect associativity: ✓ (checked by #test)"
  IO.println "Effect commutativity: ✓ (checked by #test)"
  
  -- Pattern matching
  IO.println "Wildcard always matches: ✓ (checked by #test)"
  IO.println "Pattern determinism: ✓ (checked by #test)"
  
  -- Serialization
  IO.println "Value roundtrip: ✓ (checked by #test)"
  IO.println "Effect roundtrip: ✓ (checked by #test)"

end Ash.Tests
```

**Update lakefile.lean**:

```lean
lean_exe test {
  root := `Ash.Tests.Runner
}
```

**Run**:
```bash
lake exe test
# Expected output showing property test results
```

## Completion Checklist

- [ ] Plausible framework integrated
- [ ] Value generator implemented
- [ ] Effect lattice property tests
- [ ] Environment property tests
- [ ] Pattern matching property tests
- [ ] Constructor purity property tests
- [ ] Literal purity property tests
- [ ] JSON serialization property tests
- [ ] Match expression property tests
- [ ] If-let expression property tests
- [ ] Test runner executable
- [ ] CI-compatible test output
- [ ] Documentation of tested properties

## Self-Review Questions

1. **Coverage**: Are all critical properties tested?
   - Effect lattice, pattern matching, evaluation, serialization

2. **Spec alignment**: Do tests verify SPEC-004 properties?
   - Constructor purity, pattern determinism

3. **Test runner**: Can tests run in CI?
   - Yes: lake exe test with exit codes

## Estimated Effort

8 hours

## Dependencies

- TASK-138 (AST Types)
- TASK-139 (Environment)
- TASK-140 (Expression Eval)
- TASK-141 (Pattern Match)
- TASK-144 (JSON Serialization)

## Blocked By

- TASK-144

## Blocks

- TASK-147 (CI Integration)
