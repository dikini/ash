# TASK-050: Criterion Benchmarks

## Status: ✅ Complete

## Description

Implement Criterion benchmarks for performance-critical components of the Ash system.

## Specification Reference

- SPEC-001: IR
- rust-skills - Performance testing

## Requirements

### Benchmark Suite

**Core benchmarks:**
- Effect lattice operations
- Value operations
- Pattern matching
- Workflow construction

**Parser benchmarks:**
- Lexing speed
- Parsing speed
- Error recovery

**Type checker benchmarks:**
- Type inference
- Unification
- Constraint solving

**Interpreter benchmarks:**
- Expression evaluation
- Workflow execution
- Capability invocation

**Provenance benchmarks:**
- Trace recording
- Lineage tracking
- Merkle tree operations

### Benchmark Organization

```
crates/ash-bench/
├── Cargo.toml
├── benches/
│   ├── core.rs
│   ├── parser.rs
│   ├── typeck.rs
│   ├── interp.rs
│   └── provenance.rs
└── src/
    └── lib.rs
```

### Benchmark Examples

**Effect operations:**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ash_core::Effect;

fn effect_join(c: &mut Criterion) {
    c.bench_function("effect_join", |b| {
        b.iter(|| {
            let e1 = black_box(Effect::Epistemic);
            let e2 = black_box(Effect::Operational);
            black_box(e1.join(e2))
        })
    });
}

fn effect_compare(c: &mut Criterion) {
    c.bench_function("effect_compare", |b| {
        b.iter(|| {
            let e1 = black_box(Effect::Deliberative);
            let e2 = black_box(Effect::Evaluative);
            black_box(e1 < e2)
        })
    });
}

criterion_group!(effect_benches, effect_join, effect_compare);
criterion_main!(effect_benches);
```

**Parser benchmarks:**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ash_parser::parse;

fn parse_simple_workflow(c: &mut Criterion) {
    let source = r#"
        workflow test {
            observe read with path: "/tmp/data" as content;
            act print with message: content;
            done
        }
    "#;
    
    c.bench_function("parse_simple", |b| {
        b.iter(|| {
            black_box(parse(black_box(source)).unwrap())
        })
    });
}

fn parse_complex_workflow(c: &mut Criterion) {
    let source = r#"
        workflow complex {
            observe read as data;
            
            par {
                orient { analyze(data) } as analysis;
                orient { validate(data) } as validation;
            };
            
            decide { analysis.valid } under valid_data then {
                act write with data: analysis.result;
            } else {
                act error with message: "Invalid data";
            }
            
            done
        }
    "#;
    
    c.bench_function("parse_complex", |b| {
        b.iter(|| {
            black_box(parse(black_box(source)).unwrap())
        })
    });
}

criterion_group!(parser_benches, parse_simple_workflow, parse_complex_workflow);
criterion_main!(parser_benches);
```

**Interpreter benchmarks:**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, async_executor::FuturesExecutor};
use ash_interp::{RuntimeContext, execute_workflow};

fn execute_simple_workflow(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    let ctx = RuntimeContext::new(
        Arc::new(CapabilityRegistry::new()),
        Arc::new(PolicyRegistry::new()),
    );
    
    let workflow = parse(r#"
        workflow test {
            let x = 1;
            let y = 2;
            let z = x + y;
            done
        }
    "#).unwrap().workflow;
    
    c.bench_function("exec_simple", |b| {
        b.to_async(&runtime).iter(|| async {
            let ctx = ctx.clone();
            black_box(execute_workflow(&ctx, &workflow).await.unwrap())
        })
    });
}

criterion_group!(interp_benches, execute_simple_workflow);
criterion_main!(interp_benches);
```

### Benchmark Configuration

```toml
# crates/ash-bench/Cargo.toml
[package]
name = "ash-bench"
version = "0.1.0"
edition = "2024"

[dependencies]
ash-core = { path = "../ash-core" }
ash-parser = { path = "../ash-parser" }
ash-typeck = { path = "../ash-typeck" }
ash-interp = { path = "../ash-interp" }
ash-provenance = { path = "../ash-provenance" }
tokio = { version = "1", features = ["full"] }

[dev-dependencies]
criterion = { version = "0.5", features = ["async_tokio"] }

[[bench]]
name = "core"
harness = false

[[bench]]
name = "parser"
harness = false

[[bench]]
name = "interp"
harness = false
```

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench -p ash-bench

# Run specific benchmark
cargo bench -p ash-bench effect_join

# Save baseline
cargo bench -p ash-bench -- --save-baseline main

# Compare against baseline
cargo bench -p ash-bench -- --baseline main
```

## TDD Steps

### Step 1: Create ash-bench Crate

Set up the benchmark crate.

### Step 2: Add Core Benchmarks

Benchmark effect and value operations.

### Step 3: Add Parser Benchmarks

Benchmark lexing and parsing.

### Step 4: Add Type Checker Benchmarks

Benchmark type inference.

### Step 5: Add Interpreter Benchmarks

Benchmark workflow execution.

### Step 6: Add Provenance Benchmarks

Benchmark trace and lineage operations.

### Step 7: Document Benchmarks

Add documentation on running and interpreting benchmarks.

## Completion Checklist

- [ ] ash-bench crate
- [ ] Core benchmarks
- [ ] Parser benchmarks
- [ ] Type checker benchmarks
- [ ] Interpreter benchmarks
- [ ] Provenance benchmarks
- [ ] Benchmark documentation
- [ ] cargo bench runs successfully

## Self-Review Questions

1. **Coverage**: Are critical paths benchmarked?
2. **Stability**: Are benchmarks reproducible?
3. **Utility**: Do benchmarks provide actionable data?

## Estimated Effort

6 hours

## Dependencies

- criterion
- All implementation crates

## Blocked By

- All implementation tasks

## Blocks

- TASK-051: Optimizations
