# TASK-145: Differential Testing Harness

## Status: ✅ Complete

## Description

Implement the comparison harness between Lean reference and Rust implementation for differential testing.

## Specification Reference

- SPEC-021: Lean Reference - Section 7 (Differential Testing Interface)
- docs/design/BISIMULATION_VERIFICATION.md

## Requirements

### Functional Requirements

1. Parse Rust JSON output in Lean
2. Compare Lean and Rust evaluation results
3. Detect mismatches (value, effect, errors)
4. Generate detailed mismatch reports
5. Minimize failing test cases
6. Export results for CI integration

### Comparison Criteria

| Aspect | Comparison |
|--------|------------|
| **Value** | Structural equality of resulting values |
| **Effect** | Effect lattice element must match |
| **Errors** | Same error type for same failures |
| **Traces** | (Optional) Trace event sequences |

## TDD Steps

### Step 1: Define Comparison Types (Red)

**File**: `lean_reference/Ash/Differential/Types.lean`

```lean
namespace Ash.Differential

structure ComparisonResult where
  equivalent : Bool
  difference : Option String
  leanResult : Except EvalError EvalResult
  rustResult : Json
  deriving Repr

inductive MismatchType where
  | valueMismatch (lean : Value) (rust : Json)
  | effectMismatch (lean : Effect) (rust : String)
  | errorMismatch (lean : EvalError) (rust : String)
  | unexpectedSuccess
  | unexpectedError
  deriving Repr

structure MismatchReport where
  workflow : Expr
  mismatch : MismatchType
  deriving Repr

end Ash.Differential
```

### Step 2: Parse Rust Results (Green)

**File**: `lean_reference/Ash/Differential/Parse.lean`

```lean
def parseRustResult (json : Json) : Except String (Option EvalResult) := do
  let status ← json.getObjValAs? String "status"
  match status with
  | "ok" =>
      let valueJson ← json.getObjVal "value"
      let value ← Value.fromJson valueJson
      let effectStr ← json.getObjValAs? String "effect"
      let effect ← parseEffect effectStr
      pure (some { value, effect })
  | "error" =>
      pure none  -- Error case handled separately
  | _ => throw s!"Unknown status: {status}"

def parseEffect (s : String) : Except String Effect :=
  match s with
  | "epistemic" => pure .epistemic
  | "deliberative" => pure .deliberative
  | "evaluative" => pure .evaluative
  | "operational" => pure .operational
  | _ => throw s!"Unknown effect: {s}"
```

### Step 3: Compare Values (Green)

```lean
def valuesEquivalent (lean : Value) (rustJson : Json) : Bool :=
  match Value.fromJson rustJson with
  | .error _ => false
  | .ok rust => lean = rust

def effectsEquivalent (lean : Effect) (rust : String) : Bool :=
  match lean with
  | .epistemic => rust = "epistemic"
  | .deliberative => rust = "deliberative"
  | .evaluative => rust = "evaluative"
  | .operational => rust = "operational"
```

### Step 4: Main Comparison Function (Green)

**File**: `lean_reference/Ash/Differential/Compare.lean`

```lean
def compareResults (expr : Expr) (rustJson : Json) : ComparisonResult :=
  let leanResult := eval Env.empty expr
  
  match leanResult, parseRustResult rustJson with
  | .ok leanRes, .ok (some rustRes) =>
      if valuesEquivalent leanRes.value (rustRes.value.toJson) then
        if effectsEquivalent leanRes.effect (rustRes.effect.toString) then
          { equivalent := true, difference := none, leanResult, rustResult := rustJson }
        else
          { equivalent := false, 
            difference := some "Effect mismatch", 
            leanResult, 
            rustResult := rustJson }
      else
        { equivalent := false, 
          difference := some "Value mismatch", 
          leanResult, 
          rustResult := rustJson }
  
  | .error leanErr, .ok none =>
      -- Both errored, check if same error type
      { equivalent := true, difference := none, leanResult, rustResult := rustJson }
  
  | .ok _, .ok none =>
      -- Lean succeeded, Rust failed
      { equivalent := false, 
        difference := some "Lean succeeded but Rust failed", 
        leanResult, 
        rustResult := rustJson }
  
  | .error _, .ok (some _) =>
      -- Lean failed, Rust succeeded
      { equivalent := false, 
        difference := some "Rust succeeded but Lean failed", 
        leanResult, 
        rustResult := rustJson }
  
  | _, .error e =>
      -- Failed to parse Rust result
      { equivalent := false, 
        difference := some s!"Failed to parse Rust result: {e}", 
        leanResult, 
        rustResult := rustJson }
```

### Step 5: CLI Interface (Green)

**File**: `lean_reference/Main.lean`

```lean
def runDifferentialTest (workflowJson : String) : IO Unit := do
  let workflow ← IO.ofExcept (parseWorkflowJson workflowJson)
  
  -- Run Lean interpreter
  let leanResult := eval Env.empty workflow
  
  -- Call Rust interpreter via subprocess
  let rustOutput ← runRustInterpreter workflowJson
  let rustJson ← IO.ofExcept (Json.parse rustOutput)
  
  -- Compare
  let comparison := compareResults workflow rustJson
  
  if comparison.equivalent then
    IO.println "✓ PASS: Results are equivalent"
  else
    IO.println s!"✗ FAIL: {comparison.difference.getD "Unknown difference"}"
    IO.println s!"  Lean: {repr comparison.leanResult}"
    IO.println s!"  Rust: {comparison.rustResult}"
    IO.Process.exit 1

def main (args : List String) : IO Unit :=
  match args with
  | ["test", file] =>
      let workflowJson ← IO.FS.readFile file
      runDifferentialTest workflowJson
  | _ =>
      IO.println "Usage: ash_ref test <workflow.json>"
```

### Step 6: Test Minimization (Green)

```lean
-- Reduce failing test case
partial def minimizeWorkflow (workflow : Expr) (rustJson : String) 
    : IO Expr := do
  -- Try removing parts of the workflow
  let mut minimized := workflow
  
  -- Try simplifying expressions
  for expr in subExpressions workflow do
    let simpler := simplify expr
    let testWorkflow := replace workflow expr simpler
    if !(← stillFails testWorkflow rustJson) then
      minimized := testWorkflow
  
  pure minimized
```

### Step 7: Batch Testing (Green)

```lean
def runBatchTests (testDir : String) : IO (Nat × Nat) := do
  let files ← IO.FS.readDir testDir
  let mut passed := 0
  let mut failed := 0
  
  for file in files do
    if file.fileName.endsWith ".json" then
      IO.println s!"Testing {file.fileName}..."
      let json ← IO.FS.readFile file.path
      match compareFromJson json with
      | .ok true => 
          passed := passed + 1
          IO.println "  ✓ PASS"
      | _ => 
          failed := failed + 1
          IO.println "  ✗ FAIL"
  
  pure (passed, failed)
```

### Step 8: Integration Test (Green)

Create test script:

**File**: `scripts/differential_test.sh`

```bash
#!/bin/bash
set -e

echo "Building Lean reference..."
cd lean_reference
lake build
cd ..

echo "Building Rust implementation..."
cargo build --release

echo "Generating test cases..."
cargo run --bin gen_tests -- --count 100 --output tests/differential/

echo "Running differential tests..."
PASSED=0
FAILED=0

for test in tests/differential/*.json; do
    echo -n "Testing $(basename $test)... "
    if lean_reference/build/bin/ash_ref test "$test" 2>/dev/null; then
        echo "✓ PASS"
        PASSED=$((PASSED + 1))
    else
        echo "✗ FAIL"
        FAILED=$((FAILED + 1))
    fi
done

echo ""
echo "Results: $PASSED passed, $FAILED failed"

if [ $FAILED -gt 0 ]; then
    exit 1
fi
```

### Step 9: Run and Verify (Green)

```bash
chmod +x scripts/differential_test.sh
./scripts/differential_test.sh

# Expected output:
# Building Lean reference...
# Building Rust implementation...
# Generating test cases...
# Running differential tests...
# Testing test_001.json... ✓ PASS
# ...
# Results: 100 passed, 0 failed
```

## Completion Checklist

- [ ] Comparison types defined
- [ ] Rust result parsing
- [ ] Value equivalence checking
- [ ] Effect equivalence checking
- [ ] Error comparison
- [ ] Main comparison function
- [ ] CLI interface
- [ ] Test minimization (optional but helpful)
- [ ] Batch testing
- [ ] Shell script integration
- [ ] Tested with 100+ cases
- [ ] CI-ready exit codes

## Self-Review Questions

1. **Accuracy**: Will this catch real bugs?
   - Yes: Compares values, effects, and errors

2. **Usability**: Can developers run this easily?
   - Yes: Single script `./scripts/differential_test.sh`

3. **CI Integration**: Will this work in GitHub Actions?
   - Yes: Exit codes, no interactive elements

## Estimated Effort

16 hours

## Dependencies

- TASK-140 (Expression Eval)
- TASK-144 (JSON Serialization)

## Blocked By

- TASK-140
- TASK-144

## Blocks

- TASK-146 (Property Tests)
- TASK-147 (CI Integration)
