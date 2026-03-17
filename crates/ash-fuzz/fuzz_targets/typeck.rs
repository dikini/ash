//! Type checker fuzzing target
//!
//! This fuzzer generates random AST structures and tests the type checker
//! for crashes and panics. It does not verify correctness of type checking,
//! only that the type checker doesn't crash.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    
    // Parse fuzz data as type checking scenario
    // First byte determines the operation type
    let operation = data[0] % 8;
    
    match operation {
        0 => fuzz_type_unification(&data[1..]),
        1 => fuzz_constraint_generation(&data[1..]),
        2 => fuzz_substitution_application(&data[1..]),
        3 => fuzz_effect_inference(&data[1..]),
        4 => fuzz_name_resolution(&data[1..]),
        5 => fuzz_obligation_tracking(&data[1..]),
        6 => fuzz_type_equality(&data[1..]),
        7 => fuzz_complex_workflow(&data[1..]),
        _ => unreachable!(),
    }
});

/// Fuzz type unification operations
fn fuzz_type_unification(data: &[u8]) {
    use ash_typeck::types::{unify, Substitution};
    
    if data.len() < 4 {
        return;
    }
    
    // Generate two random types
    let type1 = byte_to_type(data[0]);
    let type2 = byte_to_type(data[1]);
    
    // Attempt unification
    let subst = unify(&type1, &type2);
    
    // Apply substitution if successful (should not panic)
    if let Ok(subst) = subst {
        let _ = subst.apply(&type1);
        let _ = subst.apply(&type2);
    }
}

/// Fuzz constraint generation
fn fuzz_constraint_generation(data: &[u8]) {
    use ash_typeck::constraints::ConstraintContext;
    
    let mut ctx = ConstraintContext::new();
    
    // Generate random constraint scenarios
    for &byte in data.iter().take(20) {
        let op = byte % 3;
        match op {
            0 => {
                // Add equality constraint
                let t1 = byte_to_type(byte);
                let t2 = byte_to_type(byte.wrapping_add(1));
                ctx.add_equal(t1, t2);
            }
            1 => {
                // Add effect constraint
                let e1 = byte_to_effect(byte);
                let e2 = byte_to_effect(byte.wrapping_add(1));
                ctx.add_effect_leq(e1, e2);
            }
            2 => {
                // Bind a variable
                let name = format!("var_{}", byte);
                let ty = byte_to_type(byte);
                ctx.bind_var(name.into(), ty);
            }
            _ => {}
        }
    }
    
    // Get constraints (should not panic)
    let _ = ctx.constraints();
}

/// Fuzz substitution application
fn fuzz_substitution_application(data: &[u8]) {
    use ash_typeck::types::{Type, Substitution, TypeVar};
    
    let mut subst = Substitution::new();
    
    // Build up a substitution from fuzz data
    for chunk in data.chunks(2) {
        if chunk.len() < 2 {
            break;
        }
        let var_id = TypeVar(chunk[0] as u32);
        let ty = byte_to_type(chunk[1]);
        subst.insert(var_id, ty);
    }
    
    // Apply to various types
    for &byte in data.iter().take(10) {
        let ty = byte_to_type(byte);
        let _ = subst.apply(&ty);
    }
}

/// Fuzz effect inference
fn fuzz_effect_inference(data: &[u8]) {
    use ash_core::effect::Effect;
    
    if data.is_empty() {
        return;
    }
    
    // Combine effects in various ways
    let effects: Vec<Effect> = data.iter()
        .map(|&b| byte_to_effect(b))
        .collect();
    
    // Compute aggregate effect
    let mut aggregate = Effect::Epistemic;
    for effect in effects {
        aggregate = aggregate.join(effect);
    }
    
    // Verify properties (should not panic)
    let _ = aggregate.at_least(Effect::Epistemic);
    let _ = aggregate.meet(Effect::Operational);
}

/// Fuzz name resolution
fn fuzz_name_resolution(data: &[u8]) {
    use ash_typeck::names::NameResolver;
    use ash_parser::surface::Workflow;
    
    let mut resolver = NameResolver::new();
    
    // Simulate scope entering and name binding
    for &byte in data.iter().take(50) {
        let op = byte % 4;
        match op {
            0 => resolver.push_scope(),
            1 => {
                resolver.pop_scope();
            }
            2 => {
                // Bind a name
                let name = format!("var_{}", byte);
                resolver.bind(name);
            }
            3 => {
                // Try to resolve on a simple workflow
                let wf = Workflow::Done { span: ash_parser::token::Span::default() };
                let _ = resolver.resolve_workflow(&wf);
            }
            _ => {}
        }
    }
}

/// Fuzz obligation tracking
fn fuzz_obligation_tracking(data: &[u8]) {
    use ash_typeck::obligations::{ObligationTracker, ProofObligation, ProofWitness};
    
    let mut tracker = ObligationTracker::new();
    let mut obligation_ids: Vec<usize> = Vec::new();
    
    // Add various obligations
    for &byte in data.iter().take(30) {
        let op = byte % 3;
        match op {
            0 => {
                // Add obligation
                let obl = ProofObligation::satisfy_policy(format!("policy_{}", byte));
                let id = tracker.add(obl);
                obligation_ids.push(id);
            }
            1 => {
                // Mark as satisfied
                if let Some(&id) = obligation_ids.get(byte as usize % obligation_ids.len().max(1)) {
                    let _ = tracker.satisfy(id, Some(ProofWitness::Direct));
                }
            }
            2 => {
                // Check obligations
                let _ = tracker.check_obligations();
            }
            _ => {}
        }
    }
    
    // Final check
    let _ = tracker.check_obligations();
}

/// Fuzz type equality
fn fuzz_type_equality(data: &[u8]) {
    use ash_typeck::types::Type;
    
    if data.len() < 2 {
        return;
    }
    
    let t1 = byte_to_type(data[0]);
    let t2 = byte_to_type(data[1]);
    
    // Equality comparison (should not panic)
    let _ = t1 == t2;
}

/// Fuzz complex workflow type checking
fn fuzz_complex_workflow(data: &[u8]) {
    // Generate a simple workflow from fuzz data
    let workflow = generate_random_workflow(data);
    
    // Attempt to type check (ignoring errors, just checking for panics)
    // In a full implementation, this would call the actual type checker
    let _ = workflow;
}

// Helper functions

fn byte_to_type(byte: u8) -> ash_typeck::types::Type {
    use ash_typeck::types::TypeVar;
    use ash_typeck::types::Type;
    use ash_core::Effect;
    
    match byte % 10 {
        0 => Type::Int,
        1 => Type::Null,
        2 => Type::String,
        3 => Type::Bool,
        4 => Type::Time,
        5 => Type::List(Box::new(Type::Int)),
        6 => Type::Var(TypeVar(byte as u32)),
        7 => Type::Fun(
            vec![Type::Int, Type::Int],
            Box::new(Type::Int),
            Effect::Epistemic,
        ),
        8 => Type::Record(vec![
            ("x".into(), Type::Int),
            ("y".into(), Type::Int),
        ]),
        _ => Type::Ref,
    }
}

fn byte_to_effect(byte: u8) -> ash_core::effect::Effect {
    use ash_core::effect::Effect;
    
    match byte % 4 {
        0 => Effect::Epistemic,
        1 => Effect::Deliberative,
        2 => Effect::Evaluative,
        _ => Effect::Operational,
    }
}

fn generate_random_workflow(data: &[u8]) -> ash_core::Workflow {
    use ash_core::{Workflow, Expr, Value, Pattern};
    
    if data.is_empty() {
        return Workflow::Done;
    }
    
    match data[0] % 5 {
        0 => Workflow::Done,
        1 => Workflow::Ret {
            expr: Expr::Literal(Value::Int(data[0] as i64)),
        },
        2 => Workflow::Let {
            pattern: Pattern::Variable(format!("x{}", data[0])),
            expr: Expr::Literal(Value::Int(data.get(1).copied().unwrap_or(0) as i64)),
            continuation: Box::new(if data.len() > 2 {
                generate_random_workflow(&data[2..])
            } else {
                Workflow::Done
            }),
        },
        3 => Workflow::If {
            condition: Expr::Literal(Value::Bool(data[0] % 2 == 0)),
            then_branch: Box::new(generate_random_workflow(&data[1..data.len()/2+1])),
            else_branch: Box::new(generate_random_workflow(&data[data.len()/2+1..])),
        },
        4 => Workflow::Par {
            workflows: vec![
                generate_random_workflow(&data[1..]),
                generate_random_workflow(&data[1..]),
            ],
        },
        _ => Workflow::Done,
    }
}
