# Troubleshooting Guide

Common issues and solutions when working with the Ash Lean Reference Interpreter.

## Build Issues

### "unknown package 'Std'"

**Cause**: Lake dependencies not updated

**Fix**:
```bash
cd lean_reference
lake update
lake build
```

### "error: no such file or directory"

**Cause**: Not in correct directory

**Fix**:
```bash
cd lean_reference
lake build
```

### "lake: command not found"

**Cause**: elan not in PATH

**Fix**:
```bash
source $HOME/.elan/env
# Or add to your shell profile:
echo 'source $HOME/.elan/env' >> ~/.bashrc
```

### Build hangs or consumes too much memory

**Cause**: Lean compiler can be memory-intensive

**Fix**:
```bash
# Build with reduced parallelism
lake build --jobs 1

# Or increase system swap/memory
```

### "invalid Lake configuration"

**Cause**: lakefile.lean has syntax error

**Fix**:
```bash
# Check lakefile.lean syntax
lake check

# Reset dependencies
rm -rf .lake
lake update
```

## Runtime Issues

### Stack overflow in pattern matching

**Cause**: Deeply nested patterns or values

**Fix**: 
- Increase stack size: `ulimit -s unlimited`
- Simplify patterns
- Use iterative instead of recursive patterns

### "maximum recursion depth reached"

**Cause**: Infinite recursion in evaluation

**Fix**: 
- Check for cycles in expression definitions
- Ensure pattern matching terminates
- Add termination proofs

### "unknown identifier 'eval'"

**Cause**: Missing import

**Fix**:
```lean
import Ash  -- Import the main module
-- or
import Ash.Eval.Expr  -- Import specific module
```

### "type mismatch"

**Common causes:**

```lean
-- Wrong constructor name
Value.int 42        -- ✓ Correct
Value.Int 42        -- ✗ Wrong case

-- Missing namespace
.eval Env.empty ... -- ✗ Needs namespace
Eval.eval ...       -- ✓ Correct
```

## Test Failures

### Property test counterexamples

Plausible will show the failing case:

```
#test ∀ (e1 e2 : Effect), e1.join e2 = e2.join e1
-- stdout:
-- Found a counter-example!
-- e1 := operational
-- e2 := epistemic
```

**Fix**: Debug the specific case:
```lean
#eval Effect.operational.join Effect.epistemic
#eval Effect.epistemic.join Effect.operational
```

### Differential test failures

1. Check the failing test input:
   ```bash
   cat tests/differential/failures/test_042.json
   ```

2. Run both implementations separately:
   ```bash
   ./lean_reference/.lake/build/bin/ash_ref eval test.json
   ./target/release/ash eval test.json
   ```

3. Compare outputs and determine which is correct per SPEC-004

4. Fix the bug and re-run

### "Test runner not found"

**Cause**: Test executable not built

**Fix**:
```bash
cd lean_reference
lake build
lake exe test
```

## JSON Serialization Issues

### "Unknown value type: ..."

**Cause**: JSON format mismatch with Rust

**Fix**: Update `Ash/Core/Serialize.lean`:

```lean
-- Add handling for the new type
def Value.fromJson (json : Json) : Except String Value := do
  let obj ← json.getObj? ...
  match type with
  | "int" => ...
  | "new_type" => ...  -- Add this
  | _ => throw s!"Unknown value type: {type}"
```

### Field order differs

**Cause**: JSON object field order not guaranteed

**Fix**: Use structural comparison, not string comparison:

```lean
-- Wrong
json1.toString = json2.toString

-- Right
json1 == json2  -- Deep structural equality
```

### "cannot parse JSON"

**Cause**: Malformed JSON input

**Fix**:
```bash
# Validate JSON
jq . test_input.json

# Or use Python
python3 -m json.tool test_input.json
```

## Common Mistakes

### Forgetting to merge environments

```lean
-- Wrong: bindings not available in body
let newEnv := bindings
eval newEnv body

-- Right: merge with existing environment
let newEnv := mergeEnvs env bindings
eval newEnv body
```

### Not accumulating effects

```lean
-- Wrong: only body effect
pure { value := v, effect := bodyEffect }

-- Right: combined effect
pure { value := v, effect := scrutineeEffect.join bodyEffect }
```

### Pattern match order

```lean
-- Wrong: specific pattern after wildcard
[
  { pattern := .wildcard, body := ... },
  { pattern := .literal (Value.int 42), body := ... }  -- Never reached!
]

-- Right: specific patterns first
[
  { pattern := .literal (Value.int 42), body := ... },
  { pattern := .wildcard, body := ... }
]
```

### Forgetting do notation

```lean
-- Wrong: can't sequence Except operations
def bad (env : Env) (e1 e2 : Expr) : Except EvalError EvalResult :=
  let r1 ← eval env e1  -- Error: ← not allowed here
  eval env e2

-- Right: use do

def good (env : Env) (e1 e2 : Expr) : Except EvalError EvalResult := do
  let r1 ← eval env e1
  eval env e2
```

### Confusing Option and Except

```lean
-- matchPattern returns Option Env
#eval matchPattern (.variable "x") (Value.int 42)
-- some (env with x = 42)

-- eval returns Except EvalError EvalResult
#eval eval Env.empty (.variable "x")
-- error (unboundVariable "x")
```

## Editor Issues

### Lean language server not starting

**VS Code:**
1. Install "Lean 4" extension
2. Open command palette: `Ctrl+Shift+P`
3. Run "Lean 4: Restart Server"

**Emacs:**
```elisp
# Ensure lean4-mode is loaded
M-x lean4-mode
```

### "File not in project"

**Cause**: File outside `lean_reference` directory

**Fix**: 
- Open the `lean_reference` folder in your editor
- Not the parent Ash directory

### Goals not displaying

**VS Code:**
- Check InfoView panel is open
- Cursor must be in a proof/editor context

## Debugging Tips

### Print intermediate values

```lean
def evalSomething (env : Env) (expr : Expr) : Except EvalError EvalResult := do
  let intermediate ← eval env expr
  dbg_trace s!"Intermediate: {intermediate}"
  -- ... rest of evaluation
  pure intermediate
```

### Inspect environment

```lean
def printEnv (env : Env) (vars : List String) : IO Unit := do
  for var in vars do
    match env.lookup var with
    | some v => IO.println s!"{var} = {repr v}"
    | none => IO.println s!"{var} = <unbound>"
```

### Reduce test case

When differential testing finds a failure, minimize:

1. Remove unnecessary fields
2. Simplify nested expressions
3. Use literal values instead of variables

**Example:**
```json
// Complex failing test
{"type": "match", "scrutinee": {...}, "arms": [...]}

// Reduced
{"type": "literal", "value": {"type": "int", "value": 42}}
```

### Use #eval for debugging

```lean
-- Test individual components
#eval Value.int 42
#eval Pattern.variable "x"
#eval matchPattern (.variable "x") (Value.int 42)

-- Test full evaluation
#eval eval Env.empty (.literal (Value.int 42))
```

## Performance Optimization

### Slow evaluation

- Check for repeated environment lookups
- Consider using `let` to cache values
- Profile with `lake build --profile`

### Large test corpus

- Run tests in parallel: `./scripts/differential_test.sh --jobs 4`
- Reduce test count for faster feedback: `--count 50`
- Use subset of tests: `--corpus ./small_tests/`

## Getting Help

1. Check [SPEC-004](../../docs/spec/SPEC-004-SEMANTICS.md) for semantics questions
2. Check [SPEC-021](../../docs/spec/SPEC-021-LEAN-REFERENCE.md) for implementation details
3. Run with `#eval` to debug specific cases
4. Use `dbg_trace` for print debugging

## Useful Commands

```bash
# Clean build
lake clean && lake build

# Verbose build
lake build --verbose

# Check for errors without building
lake check

# Update dependencies
lake update

# Run tests
lake exe test

# Build specific module
lake build Ash.Core.AST
```

## Known Limitations

1. **Pattern matching is partial**: Termination not yet proven
2. **No REPL**: Must use `#eval` in files or build executable
3. **Limited error messages**: Working on better diagnostics
4. **JSON format**: Must match Rust exactly

## Reporting Issues

If you find a bug:

1. Create a minimal reproduction
2. Include Lean version: `lean --version`
3. Include elan version: `elan --version`
4. Check if it's a known limitation above
