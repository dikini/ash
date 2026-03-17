# ash-bench - Benchmarks for Ash

This crate contains Criterion.rs benchmarks for performance-critical components of Ash.

## Running Benchmarks

### All benchmarks
```bash
cargo bench
```

### Specific benchmark
```bash
cargo bench --bench parser
cargo bench --bench effect_lattice
cargo bench --bench interpreter
```

### Compare against baseline
```bash
cargo bench -- --baseline main
```

## Available Benchmarks

### `parser` - Parser Performance
- `parse_simple_workflow`: Basic OODA workflow
- `parse_complex_workflow`: Parallel branches, guards
- `parse_nested_workflows`: Nested workflow definitions

### `effect_lattice` - Effect System
- `effect_join_*`: Lattice join operations
- `effect_meet_*`: Lattice meet operations
- `effect_comparison`: Partial ordering checks

### `interpreter` - Runtime Performance
- `interpret_observe_epistemic`: Epistemic operations
- `interpret_full_ooda`: Complete OODA loop
- `interpret_with_provenance`: Provenance tracking overhead

## CI Integration

Benchmarks run on PRs to detect regressions. Results are compared against `main` branch baseline.

Thresholds:
- Parser: < 10% regression
- Effect operations: < 5% regression (hot path)
- Interpreter: < 10% regression
