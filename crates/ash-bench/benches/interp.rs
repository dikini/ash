//! Interpreter benchmarks
//!
//! Benchmarks workflow interpretation with various configurations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ash_core::{Workflow, Expr, Value, BinaryOp, Pattern};

// ============================================================================
// Workflow Construction Helpers
// ============================================================================

fn simple_ret_workflow() -> Workflow {
    Workflow::Ret {
        expr: Expr::Literal(Value::Int(42)),
    }
}

fn nested_let_workflow(depth: usize) -> Workflow {
    if depth == 0 {
        Workflow::Ret {
            expr: Expr::Variable(format!("x{}", depth)),
        }
    } else {
        Workflow::Let {
            pattern: Pattern::Variable(format!("x{}", depth)),
            expr: Expr::Literal(Value::Int(depth as i64)),
            continuation: Box::new(nested_let_workflow(depth - 1)),
        }
    }
}

fn binary_expr_chain(length: usize) -> Expr {
    if length == 1 {
        Expr::Literal(Value::Int(1))
    } else {
        Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Literal(Value::Int(1))),
            right: Box::new(binary_expr_chain(length - 1)),
        }
    }
}

fn parallel_workflow(branches: usize) -> Workflow {
    let workflows: Vec<Workflow> = (0..branches)
        .map(|i| Workflow::Ret {
            expr: Expr::Literal(Value::Int(i as i64)),
        })
        .collect();
    
    Workflow::Par { workflows }
}

fn conditional_workflow(depth: usize) -> Workflow {
    if depth == 0 {
        Workflow::Ret {
            expr: Expr::Literal(Value::Bool(true)),
        }
    } else {
        Workflow::If {
            condition: Expr::Literal(Value::Bool(depth % 2 == 0)),
            then_branch: Box::new(conditional_workflow(depth - 1)),
            else_branch: Box::new(conditional_workflow(depth - 1)),
        }
    }
}

// ============================================================================
// Workflow Construction Benchmarks
// ============================================================================

fn bench_workflow_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("workflow_construction");
    
    group.bench_function("simple_ret", |b| {
        b.iter(|| black_box(simple_ret_workflow()));
    });
    
    for depth in [5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("nested_let", depth),
            depth,
            |b, &depth| {
                b.iter(|| black_box(nested_let_workflow(depth)));
            },
        );
    }
    
    for branches in [2, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("parallel", branches),
            branches,
            |b, &branches| {
                b.iter(|| black_box(parallel_workflow(branches)));
            },
        );
    }
    
    group.finish();
}

// ============================================================================
// Expression Construction Benchmarks
// ============================================================================

fn bench_expr_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("expr_construction");
    
    group.bench_function("literal_int", |b| {
        b.iter(|| {
            black_box(Expr::Literal(Value::Int(42)))
        });
    });
    
    group.bench_function("literal_string", |b| {
        b.iter(|| {
            black_box(Expr::Literal(Value::String("hello".to_string())))
        });
    });
    
    for length in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("binary_chain", length),
            length,
            |b, &length| {
                b.iter(|| black_box(binary_expr_chain(length)));
            },
        );
    }
    
    group.finish();
}

// ============================================================================
// Workflow Traversal Benchmarks
// ============================================================================

fn count_workflow_nodes(wf: &Workflow) -> usize {
    match wf {
        Workflow::Done => 1,
        Workflow::Ret { .. } => 1,
        Workflow::Let { continuation, .. } => 1 + count_workflow_nodes(continuation),
        Workflow::If { then_branch, else_branch, .. } => {
            1 + count_workflow_nodes(then_branch) + count_workflow_nodes(else_branch)
        }
        Workflow::Seq { first, second } => {
            1 + count_workflow_nodes(first) + count_workflow_nodes(second)
        }
        Workflow::Par { workflows } => {
            1 + workflows.iter().map(count_workflow_nodes).sum::<usize>()
        }
        Workflow::ForEach { body, .. } => 1 + count_workflow_nodes(body),
        Workflow::With { workflow, .. } => 1 + count_workflow_nodes(workflow),
        Workflow::Must { workflow } => 1 + count_workflow_nodes(workflow),
        Workflow::Maybe { primary, fallback } => {
            1 + count_workflow_nodes(primary) + count_workflow_nodes(fallback)
        }
        _ => 1,
    }
}

fn bench_workflow_traversal(c: &mut Criterion) {
    let mut group = c.benchmark_group("workflow_traversal");
    
    let simple = simple_ret_workflow();
    group.bench_function("count_simple", |b| {
        b.iter(|| black_box(count_workflow_nodes(&simple)));
    });
    
    for depth in [10, 20, 50].iter() {
        let nested = nested_let_workflow(*depth);
        group.bench_with_input(
            BenchmarkId::new("count_nested", depth),
            &nested,
            |b, wf| {
                b.iter(|| black_box(count_workflow_nodes(wf)));
            },
        );
    }
    
    for depth in [5, 10].iter() {
        let conditional = conditional_workflow(*depth);
        group.bench_with_input(
            BenchmarkId::new("count_conditional", depth),
            &conditional,
            |b, wf| {
                b.iter(|| black_box(count_workflow_nodes(wf)));
            },
        );
    }
    
    group.finish();
}

// ============================================================================
// Serialization Benchmarks
// ============================================================================

fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    
    let workflow = nested_let_workflow(10);
    
    group.bench_function("serialize_workflow", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(&workflow).unwrap())
        });
    });
    
    let serialized = serde_json::to_string(&workflow).unwrap();
    group.bench_function("deserialize_workflow", |b| {
        b.iter(|| {
            black_box(serde_json::from_str::<Workflow>(&serialized).unwrap())
        });
    });
    
    // Value serialization
    let value = Value::List((0..100).map(|i| Value::Int(i)).collect());
    
    group.bench_function("serialize_value_list_100", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(&value).unwrap())
        });
    });
    
    let value_json = serde_json::to_string(&value).unwrap();
    group.bench_function("deserialize_value_list_100", |b| {
        b.iter(|| {
            black_box(serde_json::from_str::<Value>(&value_json).unwrap())
        });
    });
    
    group.finish();
}

// ============================================================================
// Memory Usage Benchmarks
// ============================================================================

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");
    
    // Measure size of different workflow types
    for depth in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("workflow_size_nested", depth),
            depth,
            |b, &depth| {
                b.iter(|| {
                    let wf = nested_let_workflow(depth);
                    black_box(std::mem::size_of_val(&wf));
                });
            },
        );
    }
    
    // Clone overhead
    let wf = nested_let_workflow(50);
    group.bench_function("clone_workflow_50", |b| {
        b.iter(|| black_box(wf.clone()));
    });
    
    group.finish();
}

// ============================================================================
// Group all benchmarks
// ============================================================================

criterion_group!(
    interpreter_benches,
    bench_workflow_construction,
    bench_expr_construction,
    bench_workflow_traversal,
    bench_serialization,
    bench_memory_usage
);

criterion_main!(interpreter_benches);
