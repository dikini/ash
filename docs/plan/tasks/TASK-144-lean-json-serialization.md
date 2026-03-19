# TASK-144: JSON Serialization Bridge

## Status: 🟡 Ready to Start

## Description

Implement JSON serialization and deserialization for AST types in Lean 4. This provides the bridge for differential testing between Lean and Rust implementations.

## Specification Reference

- SPEC-021: Lean Reference - Section 7.1 (JSON Serialization)
- SPEC-021: Lean Reference - Section 7.2 (Bisimulation Comparison)

## Requirements

### Functional Requirements

1. Implement `ToJson` instances for:
   - `Value` - all variants
   - `Expr` - all constructors
   - `Pattern` - all variants
   - `Effect`
   - `EvalError`
   - `EvalResult`
2. Implement `FromJson` instances for parsing Rust output
3. Ensure JSON format compatibility with Rust serialization
4. Support roundtrip serialization

### Property Requirements

```lean
-- Roundtrip property
prop_roundtrip_value(v) = 
  Value.fromJson (v.toJson) = .ok v

prop_roundtrip_expr(e) = 
  Expr.fromJson (e.toJson) = .ok e

-- JSON structure invariants
prop_json_has_type_field(v) = 
  v.toJson has field "type"

prop_variant_has_names(v) = 
  match v with
  | .variant tn vn _ => 
      v.toJson has field "type_name" = tn
      v.toJson has field "variant_name" = vn
  | _ => true
```

## TDD Steps

### Step 1: Implement Value.toJson (Red)

**File**: `lean_reference/Ash/Core/Serialize.lean`

```lean
import Lean
import Ash.Core.AST

namespace Ash

open Lean Json

def Value.toJson : Value → Json
  | .int i => json% { type: "int", value: $i }
  | .string s => json% { type: "string", value: $s }
  | .bool b => json% { type: "bool", value: $b }
  | .null => json% { type: "null" }
  | .list vs => json% { type: "list", value: $(vs.map Value.toJson) }
  | .record fields =>
      let obj := fields.foldl (fun acc (n, v) => 
        acc.setObjVal! n v.toJson) (mkObj [])
      json% { type: "record", fields: $obj }
  | .variant tn vn fields =>
      let obj := fields.foldl (fun acc (n, v) => 
        acc.setObjVal! n v.toJson) (mkObj [])
      json% { 
        type: "variant", 
        type_name: $tn, 
        variant_name: $vn,
        fields: $obj 
      }
  | .tuple elems =>
      json% { type: "tuple", elements: $(elems.map Value.toJson) }

instance : ToJson Value where
  toJson := Value.toJson

end Ash
```

**Test**:
```lean
#eval (Value.int 42).toJson
-- Expected: {"type": "int", "value": 42}

#eval (Value.string "hello").toJson
-- Expected: {"type": "string", "value": "hello"}

#eval (Value.variant "Option" "Some" [("value", Value.int 42)]).toJson
-- Expected: {"type": "variant", "type_name": "Option", 
--            "variant_name": "Some", "fields": {"value": {...}}}
```

### Step 2: Implement Value.fromJson (Green)

```lean
def Value.fromJson (json : Json) : Except String Value := do
  let type ← json.getObjValAs? String "type"
  match type with
  | "int" =>
      let v ← json.getObjValAs? Int "value"
      pure (.int v)
  | "string" =>
      let v ← json.getObjValAs? String "value"
      pure (.string v)
  | "bool" =>
      let v ← json.getObjValAs? Bool "value"
      pure (.bool v)
  | "null" =>
      pure .null
  | "list" =>
      let arr ← json.getObjVal "value"
      let elems ← arr.getArr?
      let vs ← elems.mapM Value.fromJson
      pure (.list vs.toList)
  | "record" =>
      let fieldsJson ← json.getObjVal "fields"
      let fields ← parseFields fieldsJson
      pure (.record fields)
  | "variant" =>
      let tn ← json.getObjValAs? String "type_name"
      let vn ← json.getObjValAs? String "variant_name"
      let fieldsJson ← json.getObjVal "fields"
      let fields ← parseFields fieldsJson
      pure (.variant tn vn fields)
  | "tuple" =>
      let arr ← json.getObjVal "elements"
      let elems ← arr.getArr?
      let vs ← elems.mapM Value.fromJson
      pure (.tuple vs.toList)
  | _ => throw s!"Unknown value type: {type}"

where
  parseFields (json : Json) : Except String (List (String × Value)) := do
    let obj ← json.getObj?
    obj.toList.mapM (fun (k, v) => do
      let val ← Value.fromJson v
      pure (k, val))

instance : FromJson Value where
  fromJson := Value.fromJson
```

**Test**:
```lean
def testRoundtrip (v : Value) : Bool :=
  match Value.fromJson v.toJson with
  | .ok v' => v = v'
  | .error _ => false

#eval testRoundtrip (Value.int 42)  -- Expected: true
#eval testRoundtrip (Value.string "hello")  -- Expected: true
#eval testRoundtrip (Value.list [Value.int 1, Value.int 2])  -- Expected: true
```

### Step 3: Implement Effect and EvalError JSON (Green)

```lean
def Effect.toJson : Effect → Json
  | .epistemic => "epistemic"
  | .deliberative => "deliberative"
  | .evaluative => "evaluative"
  | .operational => "operational"

instance : ToJson Effect where
  toJson := Effect.toJson

def Effect.fromJson (json : Json) : Except String Effect := do
  let s ← json.getStr?
  match s with
  | "epistemic" => pure .epistemic
  | "deliberative" => pure .deliberative
  | "evaluative" => pure .evaluative
  | "operational" => pure .operational
  | _ => throw s!"Unknown effect: {s}"

instance : FromJson Effect where
  fromJson := Effect.fromJson

def EvalError.toJson : EvalError → Json
  | .unboundVariable name => 
      json% { type: "unboundVariable", name: $name }
  | .typeMismatch expected actual =>
      json% { type: "typeMismatch", expected: $expected, actual: $actual }
  | .nonExhaustiveMatch =>
      json% { type: "nonExhaustiveMatch" }
  | .unknownConstructor name =>
      json% { type: "unknownConstructor", name: $name }
  | .missingField ctor field =>
      json% { type: "missingField", constructor: $ctor, field: $field }

instance : ToJson EvalError where
  toJson := EvalError.toJson
```

**Test**:
```lean
#eval Effect.operational.toJson  -- Expected: "operational"
#eval EvalError.nonExhaustiveMatch.toJson
-- Expected: {"type": "nonExhaustiveMatch"}
```

### Step 4: Implement EvalResult JSON (Green)

```lean
def EvalResult.toJson (r : EvalResult) : Json :=
  json% {
    value: $(r.value),
    effect: $(r.effect)
  }

instance : ToJson EvalResult where
  toJson := EvalResult.toJson

def EvalResult.fromJson (json : Json) : Except String EvalResult := do
  let value ← json.getObjValAs? Value "value"
  let effect ← json.getObjValAs? Effect "effect"
  pure { value, effect }

instance : FromJson EvalResult where
  fromJson := EvalResult.fromJson
```

**Test**:
```lean
def result : EvalResult := { value := Value.int 42, effect := .epistemic }
#eval result.toJson
-- Expected: {"value": {"type": "int", "value": 42}, "effect": "epistemic"}
```

### Step 5: Implement Pattern JSON (Green)

```lean
def Pattern.toJson : Pattern → Json
  | .wildcard => json% { type: "wildcard" }
  | .variable name => json% { type: "variable", name: $name }
  | .literal v => json% { type: "literal", value: $v }
  | .variant name fields =>
      json% { 
        type: "variant", 
        name: $name, 
        fields: $(fields.map (fun (n, p) => json% { name: $n, pattern: $(p.toJson) }))
      }
  | .tuple elems =>
      json% { type: "tuple", elements: $(elems.map Pattern.toJson) }
  | .record fields =>
      json% { 
        type: "record", 
        fields: $(fields.map (fun (n, p) => json% { name: $n, pattern: $(p.toJson) }))
      }

instance : ToJson Pattern where
  toJson := Pattern.toJson
```

**Test**:
```lean
#eval Pattern.wildcard.toJson  -- Expected: {"type": "wildcard"}
#eval (.variable "x" : Pattern).toJson  -- Expected: {"type": "variable", "name": "x"}
```

### Step 6: Implement Expr JSON (Partial) (Green)

```lean
def Expr.toJson : Expr → Json
  | .literal v => json% { type: "literal", value: $v }
  | .variable name => json% { type: "variable", name: $name }
  | .constructor name fields =>
      json% { 
        type: "constructor", 
        name: $name,
        fields: $(fields.map (fun (n, e) => json% { name: $n, expr: $(e.toJson) }))
      }
  | .tuple elems =>
      json% { type: "tuple", elements: $(elems.map Expr.toJson) }
  | .match scrutinee arms =>
      json% { 
        type: "match",
        scrutinee: $scrutinee,
        arms: $(arms.map (fun arm => json% { 
          pattern: $(arm.pattern), 
          body: $(arm.body.toJson) 
        }))
      }
  | .if_let pattern expr then_branch else_branch =>
      json% {
        type: "if_let",
        pattern: $pattern,
        expr: $expr,
        then_branch: $then_branch,
        else_branch: $else_branch
      }

instance : ToJson Expr where
  toJson := Expr.toJson
```

**Test**:
```lean
let e := Expr.constructor "Some" [("value", .literal (Value.int 42))]
#eval e.toJson
-- Complex nested structure expected
```

### Step 7: Property Tests for Roundtrip (Green)

```lean
-- Value roundtrip
#test ∀ (i : Int), 
  Value.fromJson (Value.int i).toJson = .ok (Value.int i)

#test ∀ (s : String), s.length < 100 →
  Value.fromJson (Value.string s).toJson = .ok (Value.string s)

#test ∀ (b : Bool),
  Value.fromJson (Value.bool b).toJson = .ok (Value.bool b)

-- Effect roundtrip
#test ∀ (e : Effect),
  Effect.fromJson e.toJson = .ok e

-- EvalResult roundtrip
#test ∀ (v : Value),
  match EvalResult.fromJson { value := v, effect := .epistemic : EvalResult }.toJson with
  | .ok r => r.value = v ∧ r.effect = .epistemic
  | _ => false
```

### Step 8: Integration and Rust Compatibility Test (Green)

Create test for Rust-compatible JSON format:

```lean
def runSerializationTests : IO Unit := do
  IO.println "\n=== JSON Serialization Tests ==="
  
  -- Value serialization
  let v1 := Value.int 42
  IO.println s!"Int: {v1.toJson.compress}"
  
  let v2 := Value.variant "Option" "Some" [("value", Value.int 42)]
  IO.println s!"Variant: {v2.toJson.compress}"
  
  -- Roundtrip
  match Value.fromJson v2.toJson with
  | .ok v2' =>
      IO.println s!"Roundtrip: {v2'}"
      if v2 = v2' then
        IO.println "✓ Roundtrip successful"
      else
        IO.println "✗ Roundtrip failed"
  | .error e =>
      IO.println s!"Parse error: {e}"
  
  -- Effect serialization
  IO.println s!"Effect: {Effect.operational.toJson.compress}"
  
  -- EvalResult
  let result : EvalResult := { value := v1, effect := .epistemic }
  IO.println s!"Result: {result.toJson.compress}"

def main : IO Unit := do
  runSerializationTests
```

**Run**:
```bash
lake exe ash_ref
# Expected output:
# === JSON Serialization Tests ===
# Int: {"type":"int","value":42}
# Variant: {"type":"variant","type_name":"Option",...}
# Roundtrip: variant "Option" "Some" ...
# ✓ Roundtrip successful
# Effect: "operational"
# Result: {"value":{"type":"int","value":42},"effect":"epistemic"}
```

## Completion Checklist

- [ ] `Value.toJson` with all variants
- [ ] `Value.fromJson` with all variants
- [ ] `Effect.toJson` and `Effect.fromJson`
- [ ] `EvalError.toJson`
- [ ] `EvalResult.toJson` and `EvalResult.fromJson`
- [ ] `Pattern.toJson`
- [ ] `Expr.toJson`
- [ ] Value roundtrip property tests
- [ ] Effect roundtrip property tests
- [ ] EvalResult roundtrip property tests
- [ ] Rust-compatible JSON format
- [ ] Integration tests
- [ ] Error handling for unknown types

## Self-Review Questions

1. **Compatibility**: Is the JSON format compatible with Rust?
   - Must verify against Rust serialization output

2. **Completeness**: Are all types serializable?
   - Core types for differential testing: Value, Effect, EvalResult

3. **Roundtrip fidelity**: Do values survive roundtrip?
   - Property tests verify roundtrip equality

## Estimated Effort

12 hours

## Dependencies

- TASK-137 (Lean Setup)
- TASK-138 (AST Types)
- TASK-139 (Environment)

## Blocked By

- TASK-139

## Blocks

- TASK-145 (Differential Testing)
