# API Documentation

Complete reference for the Ash Lean Reference Interpreter API.

## Core Types

### Value

The runtime representation of values in Ash.

```lean
inductive Value where
  | int (i : Int)
  | string (s : String)
  | bool (b : Bool)
  | null
  | list (vs : List Value)
  | record (fields : List (String × Value))
  | variant (type_name : String) (variant_name : String) (fields : List (String × Value))
  | tuple (elements : List Value)
```

**Semantics**: Defined in [SPEC-004 Section 3.1](../../docs/spec/SPEC-004-SEMANTICS.md)

**JSON Format:**
```json
{"type": "int", "value": 42}
{"type": "string", "value": "hello"}
{"type": "variant", "type_name": "Option", "variant_name": "Some", "fields": [{"name": "value", "value": {...}}]}
```

### Expr

Expression AST nodes.

```lean
inductive Expr where
  | literal (v : Value)
  | variable (name : String)
  | constructor (name : String) (fields : List (String × Expr))
  | tuple (elements : List Expr)
  | match (scrutinee : Expr) (arms : List MatchArm)
  | if_let (pattern : Pattern) (expr : Expr) (then_branch : Expr) (else_branch : Expr)
```

**Semantics**: Defined in [SPEC-004 Section 3.2](../../docs/spec/SPEC-004-SEMANTICS.md)

### Pattern

Patterns for destructuring values.

```lean
inductive Pattern where
  | wildcard
  | variable (name : String)
  | literal (v : Value)
  | variant (name : String) (fields : List (String × Pattern))
  | tuple (elements : List Pattern)
  | record (fields : List (String × Pattern))
```

**Semantics**: Defined in [SPEC-004 Section 3.3](../../docs/spec/SPEC-004-SEMANTICS.md)

### MatchArm

A match arm combining pattern and body.

```lean
structure MatchArm where
  pattern : Pattern
  body : Expr
```

### Effect

Effect lattice tracking computational power.

```lean
inductive Effect where
  | epistemic      -- Read-only, pure
  | deliberative   -- Analysis
  | evaluative     -- Decision
  | operational    -- Side effects
```

**Lattice Structure**: `epistemic < deliberative < evaluative < operational`

**Join Operation** (`⊔`): Returns the least upper bound of two effects.

```lean
#eval Effect.epistemic.join Effect.deliberative  -- deliberative
#eval Effect.evaluative.join Effect.operational  -- operational
```

**Semantics**: Defined in [SPEC-004 Section 4](../../docs/spec/SPEC-004-SEMANTICS.md)

### Env

Variable environment mapping names to values.

```lean
def Env : Type := String → Option Value
```

Implemented as a function for immutable, composable semantics.

**Operations:**

| Operation | Signature | Description |
|-----------|-----------|-------------|
| `Env.empty` | `Env` | Empty environment (all lookups return `none`) |
| `env.bind name value` | `Env` | Add binding, shadowing existing |
| `env.lookup name` | `Option Value` | Look up variable |
| `mergeEnvs env1 env2` | `Env` | Merge, right-biased |

**Examples:**
```lean
let env := Env.empty.bind "x" (Value.int 42)
env.lookup "x"  -- some (int 42)
env.lookup "y"  -- none

let env2 := env.bind "y" (Value.string "hello")
env2.lookup "y"  -- some (string "hello")
```

### EvalResult

Result of expression evaluation.

```lean
structure EvalResult where
  value : Value
  effect : Effect
```

### EvalError

Evaluation errors.

```lean
inductive EvalError where
  | unboundVariable (name : String)
  | typeMismatch (expected : String) (actual : String)
  | nonExhaustiveMatch
  | unknownConstructor (name : String)
  | missingField (constructor : String) (field : String)
```

## Evaluation Functions

### eval

Main expression evaluator.

```lean
def eval (env : Env) (expr : Expr) : Except EvalError EvalResult
```

Implements big-step semantics from [SPEC-004 Section 5](../../docs/spec/SPEC-004-SEMANTICS.md).

**Rules:**
- Literals: Pure (epistemic effect)
- Variables: Look up in environment
- Constructors: Evaluate fields, combine effects
- Tuples: Evaluate elements, combine effects
- Match: Evaluate scrutinee, try arms in order
- If-let: Evaluate expr, match, choose branch

**Examples:**
```lean
eval Env.empty (.literal (Value.int 42))
-- ok { value := int 42, effect := epistemic }

eval Env.empty (.variable "x")
-- error (unboundVariable "x")

let env := Env.empty.bind "x" (Value.int 100)
eval env (.variable "x")
-- ok { value := int 100, effect := epistemic }
```

### matchPattern

Pattern matching function.

```lean
def matchPattern (p : Pattern) (v : Value) : Option Env
```

Returns `some env` with bindings on success, `none` on failure.

**Rules:**
- `wildcard`: Always succeeds, empty env
- `variable x`: Always succeeds, binds x to value
- `literal l`: Succeeds if value equals l
- `variant n fs`: Succeeds if value is variant n with matching fields
- `tuple ps`: Succeeds if value is tuple with matching elements
- `record fs`: Succeeds if value has matching fields

**Examples:**
```lean
matchPattern (.variable "x") (Value.int 42)
-- some (env with x = 42)

matchPattern (.literal (Value.int 42)) (Value.int 42)
-- some Env.empty

matchPattern (.literal (Value.int 42)) (Value.int 43)
-- none

matchPattern (.variant "Some" [("value", .variable "x")])
             (Value.variant "Option" "Some" [("value", Value.int 42)])
-- some (env with x = 42)
```

### evalMatch

Match expression evaluator.

```lean
def evalMatch (env : Env) (scrutinee : Expr) (arms : List MatchArm) : Except EvalError EvalResult
```

Evaluates scrutinee, then tries each arm in order. First matching arm's body is evaluated with pattern bindings merged into environment.

Returns `error nonExhaustiveMatch` if no arm matches.

**Effect Semantics**: `scrutinee_effect ⊔ body_effect`

### evalIfLet

If-let expression evaluator.

```lean
def evalIfLet (env : Env) (pattern : Pattern) (expr : Expr)
  (then_branch : Expr) (else_branch : Expr) : Except EvalError EvalResult
```

1. Evaluates `expr` to get value
2. Matches value against `pattern`
3. If match succeeds: evaluates `then_branch` with bindings
4. If match fails: evaluates `else_branch` with original env

**Effect Semantics**: `expr_effect ⊔ (then_effect | else_effect)`

## Serialization

### JSON Output

All core types implement `ToJson`:

```lean
Value.toJson : Value → Json
Effect.toJson : Effect → Json
EvalResult.toJson : EvalResult → Json
EvalError.toJson : EvalError → Json
```

**Example:**
```lean
#eval (Value.int 42).toJson
-- {"type": "int", "value": 42}

#eval { value := Value.int 42, effect := Effect.epistemic : EvalResult }.toJson
-- {"value": {"type": "int", "value": 42}, "effect": "epistemic"}
```

### JSON Input

Parsing Rust output:

```lean
Value.fromJson : Json → Except String Value
Effect.fromJson : Json → Except String Effect
EvalResult.fromJson : Json → Except String EvalResult
```

**Example:**
```lean
#eval Value.fromJson (Json.parse "{\"type\": \"int\", \"value\": 42}")
-- ok (int 42)
```

## Testing

### Property Tests

Using Plausible framework:

```lean
#test ∀ (e1 e2 : Effect), e1.join e2 = e2.join e1  -- Commutativity
#test ∀ (v : Value), matchPattern .wildcard v ≠ none  -- Wildcard always matches
```

Run with: `lake exe test`

See `Ash/Tests/Properties.lean` for complete test suite.

### Test Runner

```lean
-- Run all tests
lake exe test

-- Run with verbose output
lake exe test -- --verbose
```

## Type Class Instances

### Value

- `Repr` - For debugging output
- `BEq` - Equality comparison
- `ToJson` / `FromJson` - JSON serialization

### Effect

- `Repr` - For debugging output
- `BEq` - Equality comparison
- `ToJson` / `FromJson` - JSON serialization
- `Ord` - Lattice ordering

### Expr / Pattern

- `Repr` - For debugging output
- `BEq` - Equality comparison

## Module Reference

| Module | Description |
|--------|-------------|
| `Ash.Core.AST` | Value, Expr, Pattern types |
| `Ash.Core.Environment` | Env, Effect, EvalResult, EvalError |
| `Ash.Core.Serialize` | JSON serialization |
| `Ash.Eval.Expr` | Expression evaluator |
| `Ash.Eval.Pattern` | Pattern matching |
| `Ash.Eval.Match` | Match expressions |
| `Ash.Eval.IfLet` | If-let expressions |
| `Ash.Differential.Types` | Comparison types |
| `Ash.Differential.Parse` | Rust result parsing |
| `Ash.Differential.Compare` | Result comparison |
| `Ash.Tests.Properties` | Property-based tests |
| `Ash.Tests.Runner` | Test runner |
| `Ash.Tests.CI` | CI integration |
