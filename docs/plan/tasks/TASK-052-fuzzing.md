# TASK-052: Fuzzing Setup

## Status: ✅ Complete

## Description

Set up fuzzing infrastructure using cargo-fuzz to find edge cases and bugs in the parser and runtime.

## Specification Reference

- rust-skills - Testing guidelines
- Fuzz testing best practices

## Requirements

### Fuzzing Targets

1. **Parser fuzzing**
   - Random input generation
   - Crash detection
   - Hang detection

2. **Type checker fuzzing**
   - Semantic fuzzing
   - Error handling

3. **Provenance fuzzing**
   - Trace generation
   - Merkle tree operations

### Fuzzing Setup

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Create fuzz crate
cargo fuzz init
```

### Fuzz Targets

**Parser fuzzing:**

```rust
// fuzz/fuzz_targets/parse.rs
#![no_main]

use libfuzzer_sys::fuzz_target;
use ash_parser::parse;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Should not panic
        let _ = parse(s);
    }
});
```

**Type checker fuzzing:**

```rust
// fuzz/fuzz_targets/typeck.rs
#![no_main]

use libfuzzer_sys::fuzz_target;
use ash_parser::parse;
use ash_typeck::type_check;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(program) = parse(s) {
            // Should not panic
            let _ = type_check(&program);
        }
    }
});
```

**Expression evaluation fuzzing:**

```rust
// fuzz/fuzz_targets/eval_expr.rs
#![no_main]

use libfuzzer_sys::fuzz_target;
use ash_core::{Expr, Value};
use ash_interp::eval_expr;

fuzz_target!(|data: &[u8]| {
    // Deserialize expression and value
    if let Ok((expr, ctx)) = bincode::deserialize::<(Expr, RuntimeContext)>(data) {
        // Should not panic
        let _ = eval_expr(&ctx, &expr);
    }
});
```

### Corpus Generation

```rust
// fuzz/corpus/generate.rs

use std::fs;

fn main() {
    // Generate valid inputs
    let valid_inputs = vec![
        "workflow test { done }",
        "workflow test { observe read; done }",
        // ... more valid inputs
    ];
    
    for (i, input) in valid_inputs.iter().enumerate() {
        fs::write(format!("fuzz/corpus/parse/valid{}", i), input).unwrap();
    }
    
    // Generate edge cases
    let edge_cases = vec![
        "",                    // Empty
        "{}",                  // Just braces
        "workflow",            // Incomplete
        "workflow {",          // Unclosed
        // ... more edge cases
    ];
    
    for (i, input) in edge_cases.iter().enumerate() {
        fs::write(format!("fuzz/corpus/parse/edge{}", i), input).unwrap();
    }
}
```

### Running Fuzzing

```bash
# Run parser fuzzing
cargo fuzz run parse

# Run type checker fuzzing
cargo fuzz run typeck

# Run with timeout
cargo fuzz run parse -- -max_total_time=60

# Minimize corpus
cargo fuzz tmin parse

# Show coverage
cargo fuzz coverage parse
```

### Fuzzing CI Integration

```yaml
# .github/workflows/fuzz.yml
name: Fuzz

on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly

jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-cache@v1
      - run: cargo install cargo-fuzz
      - run: cargo fuzz run parse -- -max_total_time=300
      - run: cargo fuzz run typeck -- -max_total_time=300
```

### Property Fuzzing

```rust
// fuzz/fuzz_targets/property.rs
#![no_main]

use libfuzzer_sys::fuzz_target;
use ash_core::Effect;
use proptest::prelude::*;

fuzz_target!(|data: &[u8]| {
    // Effect lattice properties
    proptest!(|(a in arb_effect(), b in arb_effect(), c in arb_effect())| {
        // Associativity: (a ⊔ b) ⊔ c = a ⊔ (b ⊔ c)
        prop_assert_eq!(
            a.join(b).join(c),
            a.join(b.join(c))
        );
        
        // Commutativity: a ⊔ b = b ⊔ a
        prop_assert_eq!(a.join(b), b.join(a));
        
        // Idempotence: a ⊔ a = a
        prop_assert_eq!(a.join(a), a);
    });
});

fn arb_effect() -> impl Strategy<Value = Effect> {
    prop_oneof![
        Just(Effect::Epistemic),
        Just(Effect::Deliberative),
        Just(Effect::Evaluative),
        Just(Effect::Operational),
    ]
}
```

### Bug Tracking

```rust
// fuzz/fuzz_targets/regression.rs
#![no_main]

use libfuzzer_sys::fuzz_target;

// Regression tests for found bugs
fuzz_target!(|data: &[u8]| {
    // Test case from bug #123
    if data == b"workflow { observe" {
        // This used to hang, now should error
        assert!(parse(std::str::from_utf8(data).unwrap()).is_err());
    }
});
```

## TDD Steps

### Step 1: Set up cargo-fuzz

Install and initialize.

### Step 2: Create Parser Fuzz Target

Add parser fuzzing.

### Step 3: Create Type Checker Fuzz Target

Add type checker fuzzing.

### Step 4: Create Expression Eval Fuzz Target

Add expression fuzzing.

### Step 5: Generate Corpus

Create seed inputs.

### Step 6: Run Fuzzing

Find bugs and fix them.

### Step 7: Set up CI

Add fuzzing to CI.

## Completion Checklist

- [ ] cargo-fuzz installed
- [ ] Parser fuzz target
- [ ] Type checker fuzz target
- [ ] Expression eval fuzz target
- [ ] Corpus generated
- [ ] Fuzzing run successfully
- [ ] Bugs found and fixed
- [ ] CI integration
- [ ] Documentation

## Self-Review Questions

1. **Coverage**: Are all entry points fuzzed?
2. **Corpus**: Is the corpus diverse?
3. **Automation**: Is fuzzing automated?

## Estimated Effort

6 hours

## Dependencies

- cargo-fuzz
- libfuzzer-sys

## Blocked By

- All implementation tasks

## Blocks

- None (final task)
