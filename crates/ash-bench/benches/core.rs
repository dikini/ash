//! Core benchmarks for ash-core operations
//!
//! Benchmarks effect lattice operations, value operations, and pattern matching.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ash_core::{Effect, Value, Pattern};
use std::collections::HashMap;

// ============================================================================
// Effect Lattice Benchmarks
// ============================================================================

fn bench_effect_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("effect_lattice");
    
    // Individual operations
    group.bench_function("join_epistemic_deliberative", |b| {
        b.iter(|| {
            black_box(Effect::Epistemic.join(Effect::Deliberative))
        });
    });
    
    group.bench_function("join_operational_evaluative", |b| {
        b.iter(|| {
            black_box(Effect::Operational.join(Effect::Evaluative))
        });
    });
    
    group.bench_function("meet_epistemic_deliberative", |b| {
        b.iter(|| {
            black_box(Effect::Epistemic.meet(Effect::Deliberative))
        });
    });
    
    group.bench_function("meet_operational_evaluative", |b| {
        b.iter(|| {
            black_box(Effect::Operational.meet(Effect::Evaluative))
        });
    });
    
    group.bench_function("at_least_check", |b| {
        b.iter(|| {
            black_box(Effect::Operational.at_least(Effect::Deliberative))
        });
    });
    
    // Batch operations
    group.bench_function("join_all_pairs", |b| {
        let effects = [
            Effect::Epistemic,
            Effect::Deliberative,
            Effect::Evaluative,
            Effect::Operational,
        ];
        b.iter(|| {
            for e1 in &effects {
                for e2 in &effects {
                    black_box(e1.join(*e2));
                }
            }
        });
    });
    
    group.bench_function("comparison_chain", |b| {
        b.iter(|| {
            black_box(
                Effect::Epistemic <= Effect::Deliberative 
                && Effect::Deliberative <= Effect::Evaluative
                && Effect::Evaluative <= Effect::Operational
            )
        });
    });
    
    group.finish();
}

// ============================================================================
// Value Operations Benchmarks
// ============================================================================

fn create_nested_value(depth: usize) -> Value {
    if depth == 0 {
        Value::Int(42)
    } else {
        let mut map = HashMap::new();
        map.insert("value".to_string(), Value::Int(depth as i64));
        map.insert("nested".to_string(), create_nested_value(depth - 1));
        Value::Record(map)
    }
}

fn bench_value_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_creation");
    
    group.bench_function("create_int", |b| {
        b.iter(|| black_box(Value::Int(42)));
    });
    
    group.bench_function("create_string", |b| {
        b.iter(|| black_box(Value::String("hello world".to_string())));
    });
    
    group.bench_function("create_list_small", |b| {
        b.iter(|| {
            let list: Vec<Value> = (0..10).map(|i| Value::Int(i)).collect();
            black_box(Value::List(list))
        });
    });
    
    group.bench_function("create_list_large", |b| {
        b.iter(|| {
            let list: Vec<Value> = (0..1000).map(|i| Value::Int(i)).collect();
            black_box(Value::List(list))
        });
    });
    
    group.bench_function("create_record_small", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            map.insert("name".to_string(), Value::String("test".to_string()));
            map.insert("value".to_string(), Value::Int(42));
            black_box(Value::Record(map))
        });
    });
    
    group.finish();
}

fn bench_value_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_operations");
    
    // Equality checks
    let v1 = Value::Int(42);
    let v2 = Value::Int(42);
    group.bench_function("equality_int", |b| {
        b.iter(|| black_box(v1 == v2));
    });
    
    let list1 = Value::List((0..100).map(|i| Value::Int(i)).collect());
    let list2 = Value::List((0..100).map(|i| Value::Int(i)).collect());
    group.bench_function("equality_list_100", |b| {
        b.iter(|| black_box(&list1 == &list2));
    });
    
    // Clone operations
    group.bench_function("clone_int", |b| {
        b.iter(|| black_box(v1.clone()));
    });
    
    group.bench_function("clone_list_100", |b| {
        b.iter(|| black_box(list1.clone()));
    });
    
    // Display formatting
    group.bench_function("display_int", |b| {
        b.iter(|| black_box(format!("{}", v1)));
    });
    
    group.bench_function("display_list_100", |b| {
        b.iter(|| black_box(format!("{}", list1)));
    });
    
    // Accessor methods
    group.bench_function("as_int_some", |b| {
        b.iter(|| black_box(v1.as_int()));
    });
    
    group.bench_function("as_int_none", |b| {
        let s = Value::String("not int".to_string());
        b.iter(|| black_box(s.as_int()));
    });
    
    group.finish();
}

fn bench_nested_values(c: &mut Criterion) {
    let mut group = c.benchmark_group("nested_values");
    
    for depth in [1, 3, 5, 10].iter() {
        group.bench_with_input(BenchmarkId::new("create_nested", depth), depth, |b, &depth| {
            b.iter(|| black_box(create_nested_value(depth)));
        });
        
        let nested = create_nested_value(*depth);
        group.bench_with_input(BenchmarkId::new("clone_nested", depth), depth, |b, _depth| {
            b.iter(|| black_box(nested.clone()));
        });
    }
    
    group.finish();
}

// ============================================================================
// Pattern Matching Benchmarks
// ============================================================================

fn bench_pattern_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_matching");
    
    // Variable pattern
    let var_pattern = Pattern::Variable("x".to_string());
    let int_value = Value::Int(42);
    
    group.bench_function("bindings_variable", |b| {
        b.iter(|| black_box(var_pattern.bindings()));
    });
    
    // Tuple pattern
    let tuple_pattern = Pattern::Tuple(vec![
        Pattern::Variable("a".to_string()),
        Pattern::Variable("b".to_string()),
    ]);
    let tuple_value = Value::List(Box::new(vec![Value::Int(1), Value::Int(2)]));
    
    group.bench_function("bindings_tuple_2", |b| {
        b.iter(|| black_box(tuple_pattern.bindings()));
    });
    
    // Complex pattern
    let complex_pattern = Pattern::Tuple(vec![
        Pattern::Variable("x".to_string()),
        Pattern::List(vec![
            Pattern::Variable("head".to_string()),
        ], Some("tail".to_string())),
    ]);
    
    group.bench_function("bindings_complex", |b| {
        b.iter(|| black_box(complex_pattern.bindings()));
    });
    
    // Is refutable checks
    group.bench_function("is_refutable_variable", |b| {
        b.iter(|| black_box(Pattern::Variable("x".to_string()).is_refutable()));
    });
    
    group.bench_function("is_refutable_tuple", |b| {
        b.iter(|| black_box(tuple_pattern.is_refutable()));
    });
    
    group.finish();
}

// ============================================================================
// Group all benchmarks
// ============================================================================

criterion_group!(
    core_benches,
    bench_effect_operations,
    bench_value_creation,
    bench_value_operations,
    bench_nested_values,
    bench_pattern_matching
);

criterion_main!(core_benches);
