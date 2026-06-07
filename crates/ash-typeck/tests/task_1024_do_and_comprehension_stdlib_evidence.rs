//! TASK-1024 RED tests for `do:K` and comprehension stdlib Monad evidence.

use std::path::PathBuf;

use ash_core::ast::Expr as CoreExpr;
use ash_parser::surface::{
    ComprehensionQualifier, Definition, DoStmt, DoTarget, Expr, Literal, Type as SurfaceType,
};
use ash_parser::token::Span;
use ash_typeck::check_expr::{check_expr, elaborate_typed_comprehension, elaborate_typed_do_block};
use ash_typeck::{Kind, QualifiedName, SelectedDoOperation, Type, TypeEnv, TypeVar};

fn span() -> Span {
    Span::default()
}

fn std_src_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../std/src")
        .join(relative)
}

fn parse_std_module(relative: &str) -> ash_parser::surface::ModuleFile {
    let path = std_src_path(relative);
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    ash_parser::parse_surface_file(&source)
        .unwrap_or_else(|errors| panic!("{relative} should parse: {errors:?}"))
}

fn computation_type(name: &str, inner: Type) -> Type {
    Type::Constructor {
        name: QualifiedName::root(name),
        args: vec![inner],
        kind: Kind::Type,
    }
}

fn result_type(inner: Type, error: Type) -> Type {
    Type::Constructor {
        name: QualifiedName::root("Result"),
        args: vec![inner, error],
        kind: Kind::Type,
    }
}

fn bind_stdlib_monad_helper_signatures(env: &mut TypeEnv) {
    let option_int = computation_type("Option", Type::Int);
    let result_int = result_type(Type::Int, Type::Var(TypeVar::fresh()));
    let option_fn = computation_type("Option", Type::Fn(vec![Type::Int], Box::new(Type::Int)));
    let result_fn = result_type(
        Type::Fn(vec![Type::Int], Box::new(Type::Int)),
        Type::Var(TypeVar::fresh()),
    );

    env.bind_variable(
        "option::map",
        Type::Fn(
            vec![
                option_int.clone(),
                Type::Fn(vec![Type::Int], Box::new(Type::Int)),
            ],
            Box::new(option_int.clone()),
        ),
    );
    env.bind_variable(
        "option::pure",
        Type::Fn(vec![Type::Int], Box::new(option_int.clone())),
    );
    env.bind_variable(
        "option::apply",
        Type::Fn(
            vec![option_fn, option_int.clone()],
            Box::new(option_int.clone()),
        ),
    );
    env.bind_variable(
        "option::and_then",
        Type::Fn(
            vec![
                option_int.clone(),
                Type::Fn(vec![Type::Int], Box::new(option_int.clone())),
            ],
            Box::new(option_int),
        ),
    );
    env.bind_variable(
        "result::map",
        Type::Fn(
            vec![
                result_int.clone(),
                Type::Fn(vec![Type::Int], Box::new(Type::Int)),
            ],
            Box::new(result_int.clone()),
        ),
    );
    env.bind_variable(
        "result::pure",
        Type::Fn(vec![Type::Int], Box::new(result_int.clone())),
    );
    env.bind_variable(
        "result::apply",
        Type::Fn(
            vec![result_fn, result_int.clone()],
            Box::new(result_int.clone()),
        ),
    );
    env.bind_variable(
        "result::and_then",
        Type::Fn(
            vec![
                result_int.clone(),
                Type::Fn(vec![Type::Int], Box::new(result_int.clone())),
            ],
            Box::new(result_int),
        ),
    );
}

fn env_with_stdlib_monad_evidence() -> TypeEnv {
    let modules = [
        parse_std_module("algebra/functor.ash"),
        parse_std_module("algebra/applicative.ash"),
        parse_std_module("algebra/monad.ash"),
        parse_std_module("option.ash"),
        parse_std_module("result.ash"),
    ];
    let mut env = TypeEnv::with_builtin_types();
    bind_stdlib_monad_helper_signatures(&mut env);

    for module in &modules {
        for definition in &module.definitions {
            if let Definition::Interface(interface) = definition {
                env.register_interface(interface)
                    .unwrap_or_else(|error| panic!("register Monad interface: {error}"));
            }
        }
    }
    for module in &modules {
        for definition in &module.definitions {
            if let Definition::Impl(implementation) = definition {
                env.register_impl(implementation)
                    .unwrap_or_else(|error| panic!("register Monad impl: {error}"));
            }
        }
    }

    env
}

fn target(name: &str) -> DoTarget {
    DoTarget {
        name: name.into(),
        args: Vec::new(),
        span: span(),
    }
}

fn result_string_target() -> DoTarget {
    DoTarget {
        name: "Result".into(),
        args: vec![
            SurfaceType::Hole { span: span() },
            SurfaceType::Name("String".into()),
        ],
        span: span(),
    }
}

fn int_lit(value: i64) -> Expr {
    Expr::Literal(Literal::Int(value))
}

fn var(name: &str) -> Expr {
    Expr::Variable {
        name: name.into(),
        span: span(),
    }
}

fn call(module: &str, func: &str, args: Vec<Expr>) -> Expr {
    Expr::Call {
        func: func.into(),
        module: Some(module.into()),
        args,
        span: span(),
    }
}

fn ret(value: Expr) -> DoStmt {
    DoStmt::Return {
        value: Box::new(value),
        span: span(),
    }
}

fn bind_stmt(name: &str, value: Expr) -> DoStmt {
    DoStmt::Bind {
        name: name.into(),
        value: Box::new(value),
        span: span(),
    }
}

fn do_block(target: DoTarget, stmts: Vec<DoStmt>) -> Expr {
    Expr::DoBlock {
        target,
        stmts,
        span: span(),
    }
}

fn bind_qual(name: &str, value: Expr) -> ComprehensionQualifier {
    ComprehensionQualifier::Bind {
        name: name.into(),
        value: Box::new(value),
        span: span(),
    }
}

fn comprehension(target: DoTarget, result: Expr, qualifiers: Vec<ComprehensionQualifier>) -> Expr {
    Expr::Comprehension {
        result: Box::new(result),
        qualifiers,
        target: Some(target),
        span: span(),
    }
}

fn assert_selected_unit_method(operation: SelectedDoOperation, evidence_key: &str) {
    let SelectedDoOperation::EvidenceMethod {
        evidence_key: actual_key,
        method,
        params,
        body,
    } = operation
    else {
        panic!("expected selected stdlib unit method body, got {operation:?}");
    };

    assert_eq!(actual_key, evidence_key);
    assert_eq!(method, "unit");
    assert_eq!(params, vec!["value".to_string()]);
    assert!(
        matches!(body, CoreExpr::Call { ref func, .. } if func == "pure"),
        "stdlib unit body should delegate to the carrier pure helper, got {body:?}"
    );
}

fn assert_selected_unit_intrinsic(
    operation: SelectedDoOperation,
    evidence_key: &str,
    module: &str,
) {
    assert_eq!(
        operation,
        SelectedDoOperation::EvidenceIntrinsic {
            evidence_key: evidence_key.to_string(),
            method: "unit".to_string(),
            shim: QualifiedName::qualified(vec![module.to_string()], "unit"),
        }
    );
}

#[test]
fn stdlib_do_evidence() {
    let env = env_with_stdlib_monad_evidence();

    let option = do_block(target("Option"), vec![ret(int_lit(1))]);
    let checked = check_expr(&env, &option);
    assert!(checked.is_ok(), "do:Option should type-check: {checked:?}");
    let option_evidence = elaborate_typed_do_block(&env, &option)
        .expect("do:Option should elaborate through stdlib Monad<Option>")
        .selected_evidence
        .expect("do:Option should preserve selected evidence");
    assert_selected_unit_method(option_evidence.return_op, "Monad<Option>");

    let result = do_block(result_string_target(), vec![ret(int_lit(1))]);
    let result_evidence = elaborate_typed_do_block(&env, &result)
        .expect("do:Result<_, String> should elaborate through stdlib Monad<Result<_, E>>")
        .selected_evidence
        .expect("do:Result should preserve selected evidence");
    assert_selected_unit_method(result_evidence.return_op, "Monad<Result<_, E>>");

    for (carrier, module) in [("Act", "act"), ("Proc", "proc"), ("Workflow", "workflow")] {
        let expr = do_block(target(carrier), vec![ret(int_lit(1))]);
        let evidence = elaborate_typed_do_block(&env, &expr)
            .unwrap_or_else(|errors| panic!("do:{carrier} should elaborate: {errors:?}"))
            .selected_evidence
            .unwrap_or_else(|| panic!("do:{carrier} should preserve selected evidence"));
        assert_selected_unit_intrinsic(evidence.return_op, &format!("Monad<{carrier}>"), module);
    }
}

#[test]
fn stdlib_comprehension_evidence() {
    let env = env_with_stdlib_monad_evidence();
    let comp = comprehension(
        target("Option"),
        var("x"),
        vec![bind_qual("x", call("option", "pure", vec![int_lit(1)]))],
    );
    let explicit_do = do_block(
        target("Option"),
        vec![
            bind_stmt("x", call("option", "pure", vec![int_lit(1)])),
            ret(var("x")),
        ],
    );

    let comp_checked = check_expr(&env, &comp);
    assert!(
        comp_checked.is_ok(),
        "Option comprehension should type-check: {comp_checked:?}"
    );

    let comp_elaborated = elaborate_typed_comprehension(&env, &comp)
        .expect("Option comprehension should elaborate through stdlib Monad<Option>");
    let do_elaborated = elaborate_typed_do_block(&env, &explicit_do)
        .expect("equivalent do:Option should elaborate");

    assert_eq!(comp_elaborated.ty, do_elaborated.ty);
    assert_eq!(comp_elaborated.expr, do_elaborated.expr);
    assert_eq!(
        comp_elaborated.selected_evidence, do_elaborated.selected_evidence,
        "comprehension must reuse the same selected evidence path as do:Option"
    );

    let evidence = comp_elaborated
        .selected_evidence
        .expect("Option comprehension should preserve selected evidence");
    assert_selected_unit_method(evidence.return_op, "Monad<Option>");
}
