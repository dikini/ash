# Differential Testing Tutorial

This tutorial explains how to use the Lean reference interpreter for differential testing against the Rust implementation.

## What is Differential Testing?

Differential testing compares two implementations of the same specification:
- **Lean reference**: Direct from SPEC-004 (trusted)
- **Rust implementation**: Production code (being tested)

If they disagree, the Rust implementation likely has a bug.

```
┌──────────────┐      ┌──────────────┐
│  Test Input  │─────▶│     Lean     │────┐
│   (JSON)     │      │  Reference   │    │
└──────────────┘      └──────────────┘    │    ┌──────────┐
                                          ├───▶│ Compare  │──▶ Pass/Fail
┌──────────────┐      ┌──────────────┐    │    └──────────┘
│  Test Input  │─────▶│     Rust     │────┘
│   (JSON)     │      │Implementation│
└──────────────┘      └──────────────┘
```

## Prerequisites

Both implementations must be built:

```bash
# Build Lean reference
cd lean_reference
lake build
cd ..

# Build Rust implementation
cargo build --release
```

## Running Differential Tests

### Basic Usage

```bash
./scripts/differential_test.sh
```

This will:
1. Generate 100 test cases
2. Run both interpreters on each case
3. Compare results
4. Report any discrepancies

### Options

```bash
# Specify number of tests
./scripts/differential_test.sh --count 1000

# Use specific binaries
./scripts/differential_test.sh \
  --lean ./lean_reference/.lake/build/bin/ash_ref \
  --rust ./target/release/ash

# Use existing corpus
./scripts/differential_test.sh --corpus ./my_tests/

# Parallel execution
./scripts/differential_test.sh --jobs 4
```

## Interpreting Results

### Success

```
✓ test_001.json
✓ test_002.json
...
Results: 100 passed, 0 failed
```

All tests passed! The implementations agree.

### Failure

```
✗ test_042.json
  Lean: {"value": {"type": "int", "value": 42}, "effect": "epistemic"}
  Rust: {"value": {"type": "int", "value": 42}, "effect": "deliberative"}
Results: 99 passed, 1 failed
Failing tests saved to: tests/differential/failures/
```

A discrepancy was found. Investigate:
1. Check `tests/differential/failures/test_042.json` for the input
2. Compare Lean and Rust outputs
3. Determine which implementation is correct per SPEC-004
4. Fix the bug and re-run

## Creating Custom Test Cases

Create a JSON file with an Ash expression:

```json
{
  "type": "match",
  "scrutinee": {
    "type": "constructor",
    "name": "Some",
    "fields": [
      {"name": "value", "expr": {"type": "literal", "value": {"type": "int", "value": 42}}}
    ]
  },
  "arms": [
    {
      "pattern": {"type": "variant", "name": "Some", "fields": [{"name": "value", "pattern": {"type": "variable", "name": "x"}}]},
      "body": {"type": "variable", "name": "x"}
    },
    {
      "pattern": {"type": "variant", "name": "None", "fields": []},
      "body": {"type": "literal", "value": {"type": "int", "value": 0}}
    }
  ]
}
```

Run differential test on it:

```bash
./scripts/differential_test.sh --corpus ./my_test.json
```

## JSON Format Reference

### Expression Format

See [SPEC-004: Operational Semantics](../../docs/spec/SPEC-004-SEMANTICS.md) for the complete AST definition.

Common patterns:

```json
// Literal
{"type": "literal", "value": {"type": "int", "value": 42}}

// Variable
{"type": "variable", "name": "x"}

// Constructor
{
  "type": "constructor",
  "name": "Some",
  "fields": [{"name": "value", "expr": {...}}]
}

// Match
{
  "type": "match",
  "scrutinee": {...},
  "arms": [
    {"pattern": {...}, "body": {...}}
  ]
}

// If-let
{
  "type": "if_let",
  "pattern": {...},
  "expr": {...},
  "then_branch": {...},
  "else_branch": {...}
}
```

### Result Format

```json
{
  "value": {
    "type": "int",
    "value": 42
  },
  "effect": "epistemic"
}
```

Effects: `epistemic`, `deliberative`, `evaluative`, `operational`

## Continuous Integration

Differential tests run automatically on PRs affecting interpreter code.

See `.github/workflows/differential-testing.yml`

## Troubleshooting

### "Lean binary not found"

Build the Lean reference:
```bash
cd lean_reference && lake build
```

### "Rust binary not found"

Build the Rust implementation:
```bash
cargo build --release
```

### "Test corpus not found"

The script will auto-generate tests. To manually generate:

```bash
./scripts/generate_test_corpus.sh --count 100
```

### Different JSON formats

If JSON formats differ between implementations, update the serialization
code in `Ash/Core/Serialize.lean` to match Rust output.

## Best Practices

1. **Start small**: Test with 10-50 cases first
2. **Check edge cases**: Empty lists, nested patterns, type mismatches
3. **Property tests**: Use Plausible in Lean for invariants
4. **Save failures**: Failed tests are saved for analysis
5. **Iterate**: Fix one issue at a time

## Next Steps

- Add property tests: See `Ash/Tests/Properties.lean`
- Read SPEC-004 for semantics details
- Contribute to formal proofs (future work)
