//! TASK-1023 RED tests for tower algebra evidence and hidden bridge remediation.

use ash_core::ast::Expr as CoreExpr;
use ash_parser::surface::{Definition, DoStmt, DoTarget, Expr, Literal, Type as SurfaceType};
use ash_parser::token::Span;
use ash_typeck::check_expr::{check_expr, elaborate_typed_do_block};
use ash_typeck::{Kind, QualifiedName, SelectedDoOperation, Type, TypeEnv};

fn span() -> Span {
    Span::default()
}

fn parse_std_module(relative: &str) -> ash_parser::surface::ModuleFile {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../std/src")
        .join(relative);
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    ash_parser::parse_surface_file(&source)
        .unwrap_or_else(|errors| panic!("{relative} should parse: {errors:?}"))
}

fn env_with_stdlib_monad_interface() -> TypeEnv {
    let module = parse_std_module("algebra/monad.ash");
    let mut env = TypeEnv::with_builtin_types();

    for definition in &module.definitions {
        if let Definition::Interface(interface) = definition {
            env.register_interface(interface)
                .unwrap_or_else(|error| panic!("register Monad interface: {error}"));
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

fn do_block(target_name: &str, stmts: Vec<DoStmt>) -> Expr {
    Expr::DoBlock {
        target: target(target_name),
        stmts,
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

fn computation_type(name: &str, inner: Type) -> Type {
    Type::Constructor {
        name: QualifiedName::root(name),
        args: vec![inner],
        kind: Kind::Type,
    }
}

fn assert_tower_evidence(env: &TypeEnv, carrier: &str, unit_module: &str) {
    let evidence = env
        .resolve_interface_evidence("Monad", &[SurfaceType::Name(carrier.into())])
        .unwrap_or_else(|error| panic!("expected named Monad<{carrier}> evidence: {error}"));

    assert_eq!(evidence.interface, "Monad");
    assert!(
        evidence.methods.is_empty(),
        "tower Monad<{carrier}> evidence should be named compiler-prelude evidence until source bodies can express the opaque carrier"
    );

    let expr = do_block(carrier, vec![ret(int_lit(1))]);
    let elaborated = elaborate_typed_do_block(env, &expr)
        .unwrap_or_else(|error| panic!("do:{carrier} should elaborate: {error:?}"));
    assert_eq!(elaborated.ty, computation_type(carrier, Type::Int));

    let selected = elaborated
        .selected_evidence
        .expect("tower do elaboration should preserve selected named evidence");
    assert_eq!(selected.target, QualifiedName::root(carrier));
    assert_eq!(selected.value_constructor, QualifiedName::root(carrier));
    assert_eq!(
        selected.return_op,
        SelectedDoOperation::EvidenceIntrinsic {
            evidence_key: format!("Monad<{carrier}>"),
            method: "unit".to_string(),
            shim: QualifiedName::qualified(vec![unit_module.to_string()], "unit"),
        }
    );
    assert_eq!(
        selected.bind_op,
        SelectedDoOperation::EvidenceIntrinsic {
            evidence_key: format!("Monad<{carrier}>"),
            method: "bind".to_string(),
            shim: QualifiedName::qualified(vec![unit_module.to_string()], "bind"),
        }
    );
    assert!(matches!(
        elaborated.expr,
        CoreExpr::Call { ref func, module: Some(ref module), .. }
            if func == "unit" && module == unit_module
    ));
}

#[test]
fn task1023_tower_algebra_instances_resolve_named_monad_evidence_for_act_proc_workflow() {
    let env = env_with_stdlib_monad_interface();

    assert_tower_evidence(&env, "Act", "act");
    assert_tower_evidence(&env, "Proc", "proc");
    assert_tower_evidence(&env, "Workflow", "workflow");
}

#[test]
fn task1023_tower_algebra_instances_do_bind_uses_public_bind_shims() {
    let env = env_with_stdlib_monad_interface();

    for (carrier, module) in [("Act", "act"), ("Proc", "proc"), ("Workflow", "workflow")] {
        let expr = do_block(
            carrier,
            vec![
                bind_stmt("x", call(module, "unit", vec![int_lit(1)])),
                ret(var("x")),
            ],
        );
        let result = check_expr(&env, &expr);
        assert!(result.is_ok(), "do:{carrier} should type-check: {result:?}");

        let elaborated = elaborate_typed_do_block(&env, &expr)
            .unwrap_or_else(|error| panic!("do:{carrier} bind should elaborate: {error:?}"));
        let selected = elaborated
            .selected_evidence
            .expect("selected tower evidence should be preserved");
        assert_eq!(
            selected.bind_op,
            SelectedDoOperation::EvidenceIntrinsic {
                evidence_key: format!("Monad<{carrier}>"),
                method: "bind".to_string(),
                shim: QualifiedName::qualified(vec![module.to_string()], "bind"),
            }
        );
        assert!(matches!(
            elaborated.expr,
            CoreExpr::Call { ref func, module: Some(ref actual_module), .. }
                if func == "bind" && actual_module == module
        ));
    }
}

#[test]
fn task1023_hidden_bridge_leakage_act_no_longer_selects_anonymous_hidden_ops() {
    let env = env_with_stdlib_monad_interface();
    let expr = do_block("Act", vec![ret(int_lit(1))]);

    let selected = elaborate_typed_do_block(&env, &expr)
        .expect("do:Act should elaborate through named evidence")
        .selected_evidence
        .expect("do:Act should preserve selected evidence");

    assert!(!matches!(
        selected.return_op,
        SelectedDoOperation::HiddenActReturn
    ));
    assert!(!matches!(
        selected.bind_op,
        SelectedDoOperation::HiddenActBind
    ));
}

#[test]
fn task1023_hidden_bridge_leakage_public_act_names_are_not_lexical_authority() {
    let env = env_with_stdlib_monad_interface();
    let _ = elaborate_typed_do_block(&env, &do_block("Act", vec![ret(int_lit(1))]))
        .expect("do:Act should elaborate through named evidence");

    assert!(
        env.lookup_variable("unit").is_none(),
        "named tower evidence must not leak unqualified unit into lexical scope"
    );
    assert!(
        env.lookup_variable("bind").is_none(),
        "named tower evidence must not leak unqualified bind into lexical scope"
    );
    assert!(
        env.lookup_variable("act::unit").is_some(),
        "named Act evidence must be tied to public act::unit"
    );
    assert!(
        env.lookup_variable("act::bind").is_some(),
        "named Act evidence must be tied to public act::bind"
    );
}
