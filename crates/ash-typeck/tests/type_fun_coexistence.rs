//! TASK-681: Coexistence tests for Phase 97's Act<T> typing additions.
//!
//! Proves that the new `Type::Constructor("Act", [T])` type (Phase 97) does NOT
//! disturb any existing `Type::Fun(...)` or `Type::Fn(...)` semantics.
//!
//! Test categories:
//!  1. Type::Fun construction and matching
//!  2. Type::Fun vs Type::Fn non-unification
//!  3. Type::Fun vs Type::Constructor non-collapse
//!  4. Unification boundary
//!  5. Substitution through Type::Fun
//!  6. Proptest property-based invariants

use ash_core::Effect;
use ash_parser::surface::{Expr, Literal};
use ash_parser::token::Span;
use ash_typeck::check_expr::check_expr;
use ash_typeck::type_env::TypeEnv;
use ash_typeck::types::{Substitution, Type, TypeVar, occurs_in, type_contains_fun, unify};
use ash_typeck::{Kind, QualifiedName};

// ── Helpers ──────────────────────────────────────────────────────────

fn span() -> Span {
    Span::default()
}

/// Construct `Act<T>` as `Type::Constructor`.
fn act_t(inner: Type) -> Type {
    Type::Constructor {
        name: QualifiedName::root("Act"),
        args: vec![inner],
        kind: Kind::Type,
    }
}

fn base_env() -> TypeEnv {
    TypeEnv::with_builtin_types()
}

// ════════════════════════════════════════════════════════════════════
// 1. Type::Fun construction and matching
// ════════════════════════════════════════════════════════════════════

#[test]
fn fun_constructs_with_params_ret_effect() {
    let t = Type::Fun(
        vec![Type::Int, Type::String],
        Box::new(Type::Bool),
        Effect::Operational,
    );

    // Pattern-match to verify all three components
    match &t {
        Type::Fun(params, ret, effect) => {
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], Type::Int);
            assert_eq!(params[1], Type::String);
            assert_eq!(**ret, Type::Bool);
            assert_eq!(*effect, Effect::Operational);
        }
        other => panic!("Expected Type::Fun, got {:?}", other),
    }
}

#[test]
fn fun_is_function_type_true() {
    let t = Type::Fun(vec![Type::Int], Box::new(Type::Bool), Effect::Epistemic);
    assert!(t.is_function_type(), "Type::Fun should be a function type");
}

#[test]
fn fun_fn_arity_returns_params_len() {
    let t = Type::Fun(
        vec![Type::Int, Type::String, Type::Bool],
        Box::new(Type::Null),
        Effect::Deliberative,
    );
    assert_eq!(t.fn_arity(), Some(3));
}

#[test]
fn fun_zero_arity() {
    let t = Type::Fun(vec![], Box::new(Type::Int), Effect::Epistemic);
    assert_eq!(t.fn_arity(), Some(0));
    assert!(t.is_function_type());
}

#[test]
fn fun_as_effect_fn_destructures_correctly() {
    let t = Type::Fun(vec![Type::String], Box::new(Type::Int), Effect::Evaluative);
    let (params, ret, effect) = t.as_effect_fn().expect("should destructure");
    assert_eq!(params.len(), 1);
    assert_eq!(params[0], Type::String);
    assert_eq!(*ret, Type::Int);
    assert_eq!(*effect, Effect::Evaluative);
}

#[test]
fn fun_as_pure_fn_returns_none() {
    let t = Type::Fun(vec![Type::Int], Box::new(Type::Bool), Effect::Operational);
    assert!(t.as_pure_fn().is_none(), "Type::Fun is not a pure Fn");
}

#[test]
fn fn_is_function_type_true() {
    let t = Type::Fn(vec![Type::Int], Box::new(Type::Bool));
    assert!(t.is_function_type());
}

#[test]
fn fn_as_pure_fn_destructures() {
    let t = Type::Fn(vec![Type::String], Box::new(Type::Int));
    let (params, ret) = t.as_pure_fn().expect("should destructure");
    assert_eq!(params.len(), 1);
    assert_eq!(*ret, Type::Int);
}

#[test]
fn fn_as_effect_fn_returns_none() {
    let t = Type::Fn(vec![Type::Int], Box::new(Type::Bool));
    assert!(t.as_effect_fn().is_none(), "Type::Fn has no effect");
}

#[test]
fn constructor_act_is_not_function_type() {
    let t = act_t(Type::Int);
    assert!(!t.is_function_type(), "Act<T> is not a function type");
}

#[test]
fn int_is_not_function_type() {
    assert!(!Type::Int.is_function_type());
}

// ════════════════════════════════════════════════════════════════════
// 2. Type::Fun vs Type::Fn non-unification
// ════════════════════════════════════════════════════════════════════

#[test]
fn fun_and_fn_do_not_unify_same_signature() {
    // Even with identical param/ret shapes, Fun and Fn are distinct.
    let fun = Type::Fun(vec![Type::Int], Box::new(Type::Bool), Effect::Operational);
    let fn_ = Type::Fn(vec![Type::Int], Box::new(Type::Bool));

    let result = unify(&fun, &fn_);
    assert!(
        result.is_err(),
        "Type::Fun and Type::Fn with same params/ret should NOT unify"
    );
}

#[test]
fn fn_and_fun_do_not_unify_reverse_direction() {
    let fun = Type::Fun(vec![Type::Int], Box::new(Type::Bool), Effect::Epistemic);
    let fn_ = Type::Fn(vec![Type::Int], Box::new(Type::Bool));

    let result = unify(&fn_, &fun);
    assert!(
        result.is_err(),
        "Type::Fn and Type::Fun should NOT unify (reversed)"
    );
}

// ════════════════════════════════════════════════════════════════════
// 3. Type::Fun vs Type::Constructor non-collapse
// ════════════════════════════════════════════════════════════════════

#[test]
fn act_int_is_not_function_type() {
    let act = act_t(Type::Int);
    assert!(!act.is_function_type(), "Act<Int> should not be callable");
}

#[test]
fn act_fn_arity_is_none() {
    let act = act_t(Type::Int);
    assert_eq!(act.fn_arity(), None, "Act<Int>.fn_arity() should be None");
}

#[test]
fn act_as_pure_fn_is_none() {
    let act = act_t(Type::Fun(vec![], Box::new(Type::Int), Effect::Epistemic));
    assert!(
        act.as_pure_fn().is_none(),
        "Act<T> wrapping a Fun should not destructure as pure Fn"
    );
}

#[test]
fn act_as_effect_fn_is_none() {
    let act = act_t(Type::Fun(vec![], Box::new(Type::Int), Effect::Operational));
    assert!(
        act.as_effect_fn().is_none(),
        "Act<T> wrapping a Fun should not destructure as effectful Fun"
    );
}

#[test]
fn type_contains_fun_false_for_act_int() {
    let act = act_t(Type::Int);
    assert!(
        !type_contains_fun(&act),
        "Act<Int> does not contain Type::Fun"
    );
}

#[test]
fn type_contains_fun_false_for_act_string() {
    let act = act_t(Type::String);
    assert!(!type_contains_fun(&act));
}

#[test]
fn type_contains_fun_true_for_act_wrapping_fun() {
    // Act<Fun(Int -> Bool)> does contain Fun at the top level of its arg
    let inner = Type::Fun(vec![Type::Int], Box::new(Type::Bool), Effect::Operational);
    let act = act_t(inner);
    assert!(
        type_contains_fun(&act),
        "Act<Fun(...)> should detect Fun inside the constructor arg"
    );
}

#[test]
fn occurs_in_traverses_act_args() {
    // occurs_in should look inside Constructor args
    let v = TypeVar(42);
    let act = act_t(Type::Var(v));
    assert!(
        occurs_in(v, &act),
        "occurs_in should find TypeVar inside Act<T>'s args"
    );
}

#[test]
fn occurs_in_not_triggered_by_act_name_for_unrelated_var() {
    let v = TypeVar(99);
    let act = act_t(Type::Int);
    assert!(
        !occurs_in(v, &act),
        "occurs_in should not find unrelated var in Act<Int>"
    );
}

#[test]
fn occurs_in_traverses_fun_params_and_ret() {
    let v = TypeVar(7);
    let fun = Type::Fun(
        vec![Type::Var(v), Type::Int],
        Box::new(Type::Var(v)),
        Effect::Epistemic,
    );
    assert!(
        occurs_in(v, &fun),
        "occurs_in should find var in Fun params/ret"
    );
}

#[test]
fn occurs_in_detects_cycle_through_fun() {
    let v = TypeVar(0);
    let fun = Type::Fun(vec![Type::Int], Box::new(Type::Var(v)), Effect::Operational);
    // v appears in the return type of fun; unifying v with fun should fail
    let result = unify(&Type::Var(v), &fun);
    assert!(
        matches!(
            result,
            Err(ash_typeck::types::UnifyError::InfiniteType(_, _))
        ),
        "occurs check should catch v = fn(Int) -> v"
    );
}

// ════════════════════════════════════════════════════════════════════
// 4. Unification boundary
// ════════════════════════════════════════════════════════════════════

#[test]
fn fun_unifies_with_fun_same_arity_effect() {
    let f1 = Type::Fun(vec![Type::Int], Box::new(Type::Bool), Effect::Operational);
    let f2 = Type::Fun(vec![Type::Int], Box::new(Type::Bool), Effect::Operational);
    let result = unify(&f1, &f2);
    assert!(result.is_ok(), "identical Fun types should unify");
}

#[test]
fn fun_unifies_with_fun_different_arity_fails() {
    let f1 = Type::Fun(vec![Type::Int], Box::new(Type::Bool), Effect::Operational);
    let f2 = Type::Fun(
        vec![Type::Int, Type::String],
        Box::new(Type::Bool),
        Effect::Operational,
    );
    assert!(unify(&f1, &f2).is_err());
}

#[test]
fn fun_unifies_with_fun_different_effect_fails() {
    let f1 = Type::Fun(vec![Type::Int], Box::new(Type::Bool), Effect::Epistemic);
    let f2 = Type::Fun(vec![Type::Int], Box::new(Type::Bool), Effect::Operational);
    assert!(unify(&f1, &f2).is_err());
}

#[test]
fn fun_does_not_unify_with_act() {
    let fun = Type::Fun(vec![Type::Int], Box::new(Type::Bool), Effect::Operational);
    let act = act_t(Type::Int);
    assert!(
        unify(&fun, &act).is_err(),
        "Type::Fun should NOT unify with Type::Constructor(\"Act\", _)"
    );
}

#[test]
fn act_does_not_unify_with_fun() {
    let act = act_t(Type::Bool);
    let fun = Type::Fun(vec![], Box::new(Type::Bool), Effect::Epistemic);
    assert!(
        unify(&act, &fun).is_err(),
        "Type::Constructor(\"Act\", _) should NOT unify with Type::Fun"
    );
}

#[test]
fn fn_does_not_unify_with_act() {
    let fn_ = Type::Fn(vec![Type::Int], Box::new(Type::Bool));
    let act = act_t(Type::Int);
    assert!(
        unify(&fn_, &act).is_err(),
        "Type::Fn should NOT unify with Type::Constructor(\"Act\", _)"
    );
}

#[test]
fn act_int_unifies_with_act_int() {
    let a1 = act_t(Type::Int);
    let a2 = act_t(Type::Int);
    assert!(
        unify(&a1, &a2).is_ok(),
        "Act<Int> should unify with Act<Int>"
    );
}

#[test]
fn act_int_does_not_unify_with_act_string() {
    let a1 = act_t(Type::Int);
    let a2 = act_t(Type::String);
    assert!(unify(&a1, &a2).is_err());
}

#[test]
fn act_var_unifies_with_act_int_produces_subst() {
    let v = TypeVar(100);
    let a1 = act_t(Type::Var(v));
    let a2 = act_t(Type::Int);
    let sub = unify(&a1, &a2).expect("should unify Act<Var> with Act<Int>");
    assert_eq!(sub.apply(&Type::Var(v)), Type::Int);
}

#[test]
fn fn_unifies_with_fn_same_signature() {
    let f1 = Type::Fn(vec![Type::Int], Box::new(Type::Bool));
    let f2 = Type::Fn(vec![Type::Int], Box::new(Type::Bool));
    assert!(unify(&f1, &f2).is_ok());
}

#[test]
fn fn_unifies_with_fn_different_arity_fails() {
    let f1 = Type::Fn(vec![Type::Int], Box::new(Type::Bool));
    let f2 = Type::Fn(vec![Type::Int, Type::String], Box::new(Type::Bool));
    assert!(unify(&f1, &f2).is_err());
}

// ════════════════════════════════════════════════════════════════════
// 5. Substitution through Type::Fun
// ════════════════════════════════════════════════════════════════════

#[test]
fn substitution_applies_through_fun_params_and_ret() {
    let v1 = TypeVar(1);
    let v2 = TypeVar(2);
    let mut sub = Substitution::new();
    sub.insert(v1, Type::String);
    sub.insert(v2, Type::Bool);

    let fun = Type::Fun(
        vec![Type::Var(v1), Type::Int],
        Box::new(Type::Var(v2)),
        Effect::Deliberative,
    );
    let result = sub.apply(&fun);

    assert_eq!(
        result,
        Type::Fun(
            vec![Type::String, Type::Int],
            Box::new(Type::Bool),
            Effect::Deliberative
        ),
        "substitution should apply to Fun params and ret, preserving effect"
    );
}

#[test]
fn substitution_does_not_modify_fun_effect() {
    let v = TypeVar(10);
    let mut sub = Substitution::new();
    sub.insert(v, Type::Int);

    let fun = Type::Fun(
        vec![Type::Var(v)],
        Box::new(Type::Var(v)),
        Effect::Evaluative,
    );
    let result = sub.apply(&fun);

    match result {
        Type::Fun(_, _, eff) => assert_eq!(eff, Effect::Evaluative),
        other => panic!("expected Fun, got {:?}", other),
    }
}

#[test]
fn substitution_applies_through_fn_params_and_ret() {
    let v = TypeVar(5);
    let mut sub = Substitution::new();
    sub.insert(v, Type::Float);

    let fn_ = Type::Fn(vec![Type::Var(v)], Box::new(Type::Var(v)));
    let result = sub.apply(&fn_);

    assert_eq!(
        result,
        Type::Fn(vec![Type::Float], Box::new(Type::Float)),
        "substitution should apply to Fn params and ret"
    );
}

#[test]
fn substitution_applies_through_act_args() {
    let v = TypeVar(20);
    let mut sub = Substitution::new();
    sub.insert(v, Type::Int);

    let act = act_t(Type::Var(v));
    let result = sub.apply(&act);

    assert_eq!(
        result,
        act_t(Type::Int),
        "substitution should apply inside Act<T>"
    );
}

#[test]
fn substitution_applies_through_nested_fun_inside_act() {
    let v = TypeVar(30);
    let mut sub = Substitution::new();
    sub.insert(v, Type::String);

    let inner_fun = Type::Fun(
        vec![Type::Var(v)],
        Box::new(Type::Var(v)),
        Effect::Operational,
    );
    let act = act_t(inner_fun);
    let result = sub.apply(&act);

    let expected_inner = Type::Fun(
        vec![Type::String],
        Box::new(Type::String),
        Effect::Operational,
    );
    assert_eq!(result, act_t(expected_inner));
}

#[test]
fn substitution_through_fun_independent_of_act_in_env() {
    // Even if the TypeEnv has Act registered, substituting through Type::Fun
    // should work exactly as before.
    let _env = base_env(); // forces Act to be registered
    let v = TypeVar(50);
    let mut sub = Substitution::new();
    sub.insert(v, Type::Bool);

    let fun = Type::Fun(vec![Type::Var(v)], Box::new(Type::Int), Effect::Epistemic);
    let result = sub.apply(&fun);

    assert_eq!(
        result,
        Type::Fun(vec![Type::Bool], Box::new(Type::Int), Effect::Epistemic),
        "substitution through Fun is unaffected by Act registration in TypeEnv"
    );
}

#[test]
fn substitution_empty_leaves_fun_unchanged() {
    let sub = Substitution::new();
    let fun = Type::Fun(vec![Type::Int], Box::new(Type::Bool), Effect::Operational);
    assert_eq!(sub.apply(&fun), fun);
}

#[test]
fn substitution_nested_var_chain_through_fun() {
    let v1 = TypeVar(1);
    let v2 = TypeVar(2);
    let mut sub = Substitution::new();
    sub.insert(v1, Type::Var(v2));
    sub.insert(v2, Type::Int);

    let fun = Type::Fun(
        vec![Type::Var(v1)],
        Box::new(Type::Var(v1)),
        Effect::Operational,
    );
    let result = sub.apply(&fun);

    assert_eq!(
        result,
        Type::Fun(vec![Type::Int], Box::new(Type::Int), Effect::Operational),
        "chained substitution should resolve through Fun"
    );
}

// ════════════════════════════════════════════════════════════════════
// 5b. type_contains_fun edge cases
// ════════════════════════════════════════════════════════════════════

#[test]
fn type_contains_fun_true_for_bare_fun() {
    let fun = Type::Fun(vec![Type::Int], Box::new(Type::Bool), Effect::Operational);
    assert!(type_contains_fun(&fun));
}

#[test]
fn type_contains_fun_false_for_fn() {
    let fn_ = Type::Fn(vec![Type::Int], Box::new(Type::Bool));
    assert!(!type_contains_fun(&fn_), "Type::Fn is not Type::Fun");
}

#[test]
fn type_contains_fun_true_for_list_of_fun() {
    let fun = Type::Fun(vec![], Box::new(Type::Int), Effect::Epistemic);
    let list = Type::List(Box::new(fun));
    assert!(type_contains_fun(&list));
}

#[test]
fn type_contains_fun_true_for_record_with_fun_field() {
    let fun = Type::Fun(
        vec![Type::String],
        Box::new(Type::Bool),
        Effect::Deliberative,
    );
    let rec = Type::Record(vec![(Box::from("callback"), fun)]);
    assert!(type_contains_fun(&rec));
}

#[test]
fn type_contains_fun_false_for_record_without_fun() {
    let rec = Type::Record(vec![
        (Box::from("x"), Type::Int),
        (Box::from("y"), Type::String),
    ]);
    assert!(!type_contains_fun(&rec));
}

#[test]
fn type_contains_fun_false_for_act_of_fn() {
    let fn_ = Type::Fn(vec![Type::Int], Box::new(Type::Bool));
    let act = act_t(fn_);
    assert!(
        !type_contains_fun(&act),
        "Act<Type::Fn(...)> does not contain Type::Fun"
    );
}

#[test]
fn type_contains_fun_true_for_fn_returning_fun() {
    // Fn(Int) -> Fun(String -> Bool) — the return contains Fun
    let inner = Type::Fun(
        vec![Type::String],
        Box::new(Type::Bool),
        Effect::Operational,
    );
    let outer = Type::Fn(vec![Type::Int], Box::new(inner));
    assert!(type_contains_fun(&outer));
}

// ════════════════════════════════════════════════════════════════════
// 6. Proptest — property-based invariants
// ════════════════════════════════════════════════════════════════════

use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_fun_is_always_callable_and_has_arity(
        n in 0usize..5,
        effect_seed in 0u8..4,
    ) {
        let effects = [Effect::Epistemic, Effect::Deliberative, Effect::Evaluative, Effect::Operational];
        let effect = effects[effect_seed as usize];

        let params: Vec<Type> = (0..n).map(|i| {
            match i % 3 {
                0 => Type::Int,
                1 => Type::String,
                _ => Type::Bool,
            }
        }).collect();

        let fun = Type::Fun(params.clone(), Box::new(Type::Int), effect);

        prop_assert!(fun.is_function_type(), "Fun should always be a function type");
        prop_assert_eq!(fun.fn_arity(), Some(n), "Fun arity should be Some(params.len())");
        prop_assert!(fun.as_effect_fn().is_some(), "Fun should have effect parts");
        prop_assert!(fun.as_pure_fn().is_none(), "Fun is not pure");
    }

    #[test]
    fn prop_act_is_never_callable(
        inner_seed in 0u8..4,
    ) {
        let inners = [Type::Int, Type::String, Type::Bool, Type::Null];
        let inner = inners[inner_seed as usize].clone();
        let act = act_t(inner);

        prop_assert!(!act.is_function_type(), "Act<T> should never be callable");
        prop_assert_eq!(act.fn_arity(), None, "Act<T>.fn_arity() should be None");
        prop_assert!(act.as_pure_fn().is_none(), "Act<T> has no pure fn parts");
        prop_assert!(act.as_effect_fn().is_none(), "Act<T> has no effect fn parts");
    }

    #[test]
    fn prop_fn_is_always_callable_and_has_arity(
        n in 0usize..5,
    ) {
        let params: Vec<Type> = (0..n).map(|i| {
            match i % 3 {
                0 => Type::Int,
                1 => Type::String,
                _ => Type::Bool,
            }
        }).collect();

        let fn_ = Type::Fn(params.clone(), Box::new(Type::Bool));

        prop_assert!(fn_.is_function_type());
        prop_assert_eq!(fn_.fn_arity(), Some(n));
        prop_assert!(fn_.as_pure_fn().is_some());
        prop_assert!(fn_.as_effect_fn().is_none());
    }

    #[test]
    fn prop_act_int_never_equals_fun_int_bool(
        effect_seed in 0u8..4,
    ) {
        let effects = [Effect::Epistemic, Effect::Deliberative, Effect::Evaluative, Effect::Operational];
        let effect = effects[effect_seed as usize];
        let fun = Type::Fun(vec![Type::Int], Box::new(Type::Bool), effect);
        let act = act_t(Type::Int);

        prop_assert_ne!(fun, act, "Fun and Act should never be equal");
    }
}

// ════════════════════════════════════════════════════════════════════
// 7. Integration: check_expr fun/act separation
// ════════════════════════════════════════════════════════════════════

#[test]
fn check_expr_fn_call_remains_fn_typed() {
    // A call to a pure function should still infer as the function's return type,
    // not accidentally as Act<T>.
    let mut env = base_env();
    env.bind_variable("identity", Type::Fn(vec![Type::Int], Box::new(Type::Int)));

    let expr = Expr::Call {
        func: "identity".into(),
        module: None,
        args: vec![Expr::Literal(Literal::Int(42))],
        span: span(),
    };

    let result = check_expr(&env, &expr);
    assert!(result.is_ok(), "identity(42) should typecheck");
    // The return type should be Int, NOT Act<Int>
    assert_eq!(
        result.substitution.apply(&result.ty),
        Type::Int,
        "pure Fn call should return Int, not Act<Int>"
    );
}

#[test]
fn check_expr_act_block_returns_act_not_fun() {
    // act { ret 42; } should be Act<Int>, not Fun([], Int, _)
    let env = base_env();
    let expr = Expr::ActBlock {
        stmts: vec![ash_parser::surface::ActStmt::Return {
            value: Box::new(Expr::Literal(Literal::Int(42))),
            span: span(),
        }],
        span: span(),
    };

    let result = check_expr(&env, &expr);
    assert!(result.is_ok());

    let ty = result.substitution.apply(&result.ty);
    match &ty {
        Type::Constructor { name, args, .. } => {
            assert_eq!(name.name, "Act", "act block should return Act<_>");
            assert_eq!(args.len(), 1);
        }
        other => panic!("expected Act<Int>, got {:?}", other),
    }
    // Also verify it's not a Fun type
    assert!(
        !ty.is_function_type(),
        "act block result should not be a function type"
    );
}
