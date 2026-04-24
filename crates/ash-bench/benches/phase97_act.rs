use ash_core::{BinaryOp, Expr, Provenance, Value};
use ash_interp::{ActEnv, CapabilityContext, Context, Policy, PolicyEvaluator, eval_expr};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn unit_expr(value: i64) -> Expr {
    Expr::Call {
        func: "unit".to_string(),
        module: None,
        arguments: vec![Expr::Literal(Value::Int(value))],
    }
}

fn force_expr(expr: Expr) -> Expr {
    Expr::FnApply {
        func: Box::new(expr),
        args: vec![Expr::Literal(Value::ActEnvToken)],
    }
}

fn bind_increment_expr(depth: usize) -> Expr {
    let mut expr = unit_expr(0);
    for _ in 0..depth {
        expr = Expr::Call {
            func: "bind".to_string(),
            module: None,
            arguments: vec![
                expr,
                Expr::FnDef {
                    params: vec![("x".to_string(), None)],
                    return_type: None,
                    body: Box::new(Expr::Call {
                        func: "unit".to_string(),
                        module: None,
                        arguments: vec![Expr::Binary {
                            op: BinaryOp::Add,
                            left: Box::new(Expr::Variable {
                                name: "x".to_string(),
                                span: ash_core::ast::Span::default(),
                            }),
                            right: Box::new(Expr::Literal(Value::Int(1))),
                        }],
                    }),
                },
            ],
        };
    }
    force_expr(expr)
}

fn guard_expr(policy_name: &str) -> Expr {
    force_expr(Expr::Call {
        func: "__guard".to_string(),
        module: Some("act".to_string()),
        arguments: vec![
            Expr::Literal(Value::String(policy_name.to_string())),
            unit_expr(7),
        ],
    })
}

fn policy_context(policy_name: &str) -> Context {
    let mut policies = PolicyEvaluator::new();
    policies.register(Policy::new(policy_name).with_default(ash_core::Decision::Permit));
    let act_env = ActEnv::new(CapabilityContext::new(), policies.clone(), Provenance::new());
    Context::new().with_policy_evaluator(policies).with_act_env(act_env)
}

fn bench_phase97_act_runtime(c: &mut Criterion) {
    let mut group = c.benchmark_group("phase97_act_runtime");

    let permit_ctx = policy_context("allow");
    let guard = guard_expr("allow");
    group.bench_function("guard_force_permit", |b| {
        b.iter(|| {
            black_box(eval_expr(black_box(&guard), black_box(&permit_ctx)).unwrap());
        });
    });

    for depth in [1usize, 4, 8, 16] {
        let ctx = Context::new().with_act_env(ActEnv::default());
        let expr = bind_increment_expr(depth);
        group.bench_function(format!("bind_chain_force_{depth}"), |b| {
            b.iter(|| {
                black_box(eval_expr(black_box(&expr), black_box(&ctx)).unwrap());
            });
        });
    }

    group.finish();
}

criterion_group!(phase97_act_benches, bench_phase97_act_runtime);
criterion_main!(phase97_act_benches);
