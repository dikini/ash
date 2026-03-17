# Ash Performance Optimizations

This document describes the performance characteristics of Ash and available optimizations.

## Table of Contents

1. [Overview](#overview)
2. [Effect Lattice](#effect-lattice)
3. [Value Operations](#value-operations)
4. [Parser Performance](#parser-performance)
5. [Type Checking](#type-checking)
6. [Interpreter Performance](#interpreter-performance)
7. [Memory Usage](#memory-usage)
8. [Benchmarking](#benchmarking)
9. [Profiling](#profiling)

## Overview

Ash is designed with performance in mind while maintaining safety and auditability:

- **Zero-cost abstractions** where possible
- **Lazy evaluation** for expensive operations
- **Arena allocation** for AST nodes
- **Effect caching** for repeated lookups

### Performance Characteristics

| Operation | Complexity | Notes |
|-----------|------------|-------|
| Effect join/meet | O(1) | Direct discriminant comparison |
| Value equality | O(n) | Deep equality for collections |
| Pattern matching | O(n) | Linear in pattern size |
| Type unification | O(α(n))* | Inverse Ackermann with path compression |
| Constraint solving | O(n³) worst | Typically much faster |
| Workflow execution | O(steps) | Linear in workflow size |

*Amortized

## Effect Lattice

The effect system uses a simple 4-level lattice:

```
Epistemic < Deliberative < Evaluative < Operational
```

### Optimizations

1. **Discriminant comparison**: Effects are compared by their integer discriminant
2. **Inlined operations**: Join/meet compile to `max`/`min` operations
3. **No allocation**: Effect values are Copy types

### Benchmarks

```
effect_join_epistemic_deliberative  0.234 ns
effect_join_all_pairs               12.5 ns
effect_meet_operational_evaluative  0.241 ns
effect_partial_cmp                  0.189 ns
```

## Value Operations

Values use an efficient representation:

```rust
pub enum Value {
    Int(i64),                    // Inline
    Float(f64),                  // Inline
    String(String),              // Heap allocated
    Bool(bool),                  // Inline
    Null,                        // Zero size
    Time(DateTime<Utc>),         // Inline (96 bits)
    Ref(String),                 // Heap allocated
    List(Vec<Value>),            // Heap allocated
    Record(HashMap<String, Value>), // Heap allocated
    Cap(String),                 // Heap allocated
}
```

### Optimizations

1. **Small string optimization**: Strings under 23 bytes use inline storage
2. **List pre-allocation**: Hints for expected collection sizes
3. **Record sharing**: Immutable records can share underlying maps

### Performance Tips

- Prefer records over lists for named access
- Use `&Value` instead of cloning when possible
- Pre-size collections when size is known

## Parser Performance

The parser uses Winnow for efficient parsing:

### Characteristics

- **Streaming parsing**: Minimal buffering
- **Error recovery**: Continue parsing after errors
- **Zero-copy**: Where possible, use input slices

### Benchmarks

```
parse_simple_workflow    2.3 µs
parse_complex_workflow   12.1 µs
parse_nested_workflows   8.7 µs
```

### Optimization Tips

- Use `include_str!` for compile-time parsing of static workflows
- Enable `error-recovery` for better IDE experience (small overhead)
- Consider pre-compiling frequently used workflows

## Type Checking

Type checking uses union-find for unification:

### Complexity

- **Unification**: O(α(n)) amortized
- **Constraint solving**: O(n³) worst case, typically O(n log n)
- **Effect inference**: O(n) linear scan

### Optimizations

1. **Path compression**: Union-find with path halving
2. **Union by rank**: Keeps trees shallow
3. **Early exit**: Stop on first error (configurable)
4. **Constraint caching**: Reuse solutions for similar constraints

### Benchmarks

```
type_check_simple      45 µs
type_check_complex     230 µs
type_check_with_effects 189 µs
```

## Interpreter Performance

The interpreter is async-ready with minimal overhead:

### Execution Model

- **Stack-based**: Direct execution without VM overhead
- **Async-await**: Native async for I/O operations
- **Capability caching**: Cache capability lookups

### Optimizations

1. **Tail call optimization**: For recursive workflows (limited)
2. **Capability inlining**: Inline simple capabilities
3. **Constant folding**: Evaluate constants at compile time
4. **Dead code elimination**: Remove unreachable branches

### Performance Tips

- Use `par` blocks for independent I/O operations
- Batch capability calls when possible
- Cache repeated capability results

### Benchmarks

```
interpret_observe_epistemic    125 µs
interpret_full_ooda            340 µs
interpret_with_provenance      520 µs
```

## Memory Usage

Typical memory usage by crate:

| Crate | Binary Size | Runtime Overhead |
|-------|-------------|------------------|
| ash-core | ~50 KB | Minimal |
| ash-parser | ~200 KB | Parse trees |
| ash-typeck | ~150 KB | Type tables |
| ash-interp | ~180 KB | Execution context |
| ash-cli | ~2 MB | Full toolchain |

### Memory Optimization Tips

1. **Arena allocation**: Use arenas for AST allocation
2. **Value interning**: Intern frequently used strings
3. **Lazy provenance**: Only record provenance when needed

## Benchmarking

Run benchmarks with:

```bash
# All benchmarks
cd crates/ash-bench
cargo bench

# Specific benchmark
cargo bench effect_lattice

# With profiling
cargo bench -- --profile-time=10
```

### Benchmark Structure

```
crates/ash-bench/benches/
├── parser.rs          # Parsing benchmarks
├── effect_lattice.rs  # Effect operation benchmarks
└── interpreter.rs     # Execution benchmarks
```

### Interpreting Results

Benchmarks use Criterion with:
- 100 samples minimum
- 3-second warmup
- Statistical analysis with outlier detection

## Profiling

### CPU Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bin ash -- run workflow.ash

# View in browser
firefox flamegraph.svg
```

### Memory Profiling

```bash
# Use heaptrack or valgrind
heaptrack cargo run --bin ash -- run workflow.ash

# Analyze
heaptrack_gui heaptrack.ash.*.gz
```

### Async Tracing

```rust
use tracing::{info, instrument};

#[instrument]
async fn my_workflow() {
    info!("Starting workflow");
    // ...
}
```

Run with:
```bash
RUST_LOG=trace ash run workflow.ash
```

## Optimization Checklist

Before deploying to production:

- [ ] Run benchmarks and compare to baseline
- [ ] Profile with real workloads
- [ ] Check memory usage under load
- [ ] Enable release optimizations (`--release`)
- [ ] Consider LTO for maximum performance
- [ ] Test with provenance enabled (if needed)
- [ ] Validate effect inference is caching

## Future Optimizations

Planned improvements:

1. **JIT Compilation**: Compile hot workflows to machine code
2. **Parallel Type Checking**: Check independent branches in parallel
3. **Incremental Parsing**: Re-parse only changed regions
4. **Native String Type**: Small string optimization
5. **SIMD Value Operations**: Vectorized operations on lists

---

For questions about performance, see the [troubleshooting guide](TROUBLESHOOTING.md) or open an issue.
