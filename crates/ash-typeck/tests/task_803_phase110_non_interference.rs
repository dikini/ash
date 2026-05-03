use ash_parser::surface::{
    CheckTarget, ComprehensionQualifier, DoStmt, DoTarget, Expr, Literal, ObligationRef,
    PolicyInstance, Type as SurfaceType, Workflow,
};
use ash_parser::token::Span;
use ash_typeck::capability_check::CapabilityChecker;
use ash_typeck::check_expr::{check_expr, elaborate_typed_comprehension, elaborate_typed_do_block};
use ash_typeck::{Kind, QualifiedName, Type, TypeEnv};

fn span() -> Span {
    Span::default()
}

fn target(name: &str) -> DoTarget {
    DoTarget {
        name: name.into(),
        args: vec![],
        span: span(),
    }
}

fn int_lit(value: i64) -> Expr {
    Expr::Literal(Literal::Int(value))
}

fn bool_lit(value: bool) -> Expr {
    Expr::Literal(Literal::Bool(value))
}

fn var(name: &str) -> Expr {
    Expr::Variable {
        name: name.into(),
        span: span(),
    }
}

fn call(module: Option<&str>, func: &str, args: Vec<Expr>) -> Expr {
    Expr::Call {
        func: func.into(),
        module: module.map(Into::into),
        args,
        span: span(),
    }
}

fn unit_call(module: &str, value: Expr) -> Expr {
    call(Some(module), "unit", vec![value])
}

fn do_block(target_name: &str, stmts: Vec<DoStmt>) -> Expr {
    Expr::DoBlock {
        target: target(target_name),
        stmts,
        span: span(),
    }
}

fn comprehension(
    target_name: Option<&str>,
    result: Expr,
    qualifiers: Vec<ComprehensionQualifier>,
) -> Expr {
    Expr::Comprehension {
        result: Box::new(result),
        qualifiers,
        target: target_name.map(target),
        span: span(),
    }
}

fn ret(value: Expr) -> DoStmt {
    DoStmt::Return {
        value: Box::new(value),
        span: span(),
    }
}

fn let_stmt(name: &str, value: Expr) -> DoStmt {
    DoStmt::Let {
        name: name.into(),
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

fn bind_qual(name: &str, value: Expr) -> ComprehensionQualifier {
    ComprehensionQualifier::Bind {
        name: name.into(),
        value: Box::new(value),
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

fn env_with_act_unit() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable(
        "act::unit",
        Type::Fn(
            vec![Type::Int],
            Box::new(computation_type("Act", Type::Int)),
        ),
    );
    env
}

#[test]
fn task803_phase109_do_act_behavior_still_typechecks_and_elaborates() {
    let env = env_with_act_unit();
    let expr = do_block(
        "Act",
        vec![bind_stmt("x", unit_call("act", int_lit(1))), ret(var("x"))],
    );

    let checked = check_expr(&env, &expr);
    assert!(
        checked.is_ok(),
        "ordinary do:Act regression should remain green: {checked:?}"
    );
    assert_eq!(checked.ty, computation_type("Act", Type::Int));

    let elaborated = elaborate_typed_do_block(&env, &expr)
        .expect("ordinary do:Act elaboration should remain unaffected by Phase 110 substrate work");
    assert_eq!(elaborated.ty, computation_type("Act", Type::Int));
}

#[test]
fn task803_phase109_do_workflow_behavior_still_typechecks_and_preserves_artifact() {
    let env = TypeEnv::with_builtin_types();
    let expr = do_block("Workflow", vec![ret(int_lit(1))]);

    let checked = check_expr(&env, &expr);
    assert!(
        checked.is_ok(),
        "ordinary do:Workflow regression should remain green: {checked:?}"
    );
    assert_eq!(checked.ty, computation_type("Workflow", Type::Int));

    let elaborated = elaborate_typed_do_block(&env, &expr).expect(
        "ordinary do:Workflow elaboration should remain unaffected by Phase 110 substrate work",
    );
    assert_eq!(elaborated.ty, computation_type("Workflow", Type::Int));
    assert!(
        elaborated.workflow_artifact.is_some(),
        "workflow artifact preservation should remain unaffected by projection diagnostics work"
    );
}

#[test]
fn task803_phase109_comprehension_behavior_still_matches_explicit_do_elaboration() {
    let env = env_with_act_unit();
    let comp = comprehension(
        Some("Act"),
        var("x"),
        vec![bind_qual("x", unit_call("act", int_lit(1)))],
    );
    let explicit_do = do_block(
        "Act",
        vec![bind_stmt("x", unit_call("act", int_lit(1))), ret(var("x"))],
    );

    let checked = check_expr(&env, &comp);
    assert!(
        checked.is_ok(),
        "comprehension regression should remain green: {checked:?}"
    );
    assert_eq!(checked.ty, computation_type("Act", Type::Int));

    let comp_elab = elaborate_typed_comprehension(&env, &comp).expect(
        "comprehension elaboration should remain unaffected by projection diagnostics work",
    );
    let do_elab = elaborate_typed_do_block(&env, &explicit_do).expect(
        "equivalent do elaboration should remain unaffected by projection diagnostics work",
    );
    assert_eq!(comp_elab.ty, do_elab.ty);
    assert_eq!(comp_elab.expr, do_elab.expr);
}

#[test]
fn task803_capability_checker_decide_policy_behavior_is_unchanged() {
    let workflow = Workflow::Decide {
        expr: bool_lit(true),
        policy: Some("gate".into()),
        then_branch: Box::new(Workflow::Done { span: span() }),
        else_branch: None,
        span: span(),
    };

    assert!(
        CapabilityChecker::new().verify(&workflow).is_ok(),
        "Phase 110 projection diagnostics must not perturb capability-policy checking"
    );
}

#[test]
fn task803_capability_checker_obligation_check_behavior_is_unchanged() {
    let workflow = Workflow::Check {
        target: CheckTarget::Obligation(ObligationRef {
            role: "operator".into(),
            condition: bool_lit(true),
        }),
        continuation: None,
        span: span(),
    };

    assert!(
        CapabilityChecker::new().verify(&workflow).is_ok(),
        "Phase 110 projection diagnostics must not perturb workflow obligation checking"
    );
}

#[test]
fn task803_phase109_ordinary_let_scoping_inside_do_blocks_is_unchanged() {
    let env = TypeEnv::with_builtin_types();
    let expr = do_block("Proc", vec![let_stmt("x", int_lit(1)), ret(var("x"))]);

    let checked = check_expr(&env, &expr);
    assert!(
        checked.is_ok(),
        "ordinary do-block lexical binding should remain green: {checked:?}"
    );
    assert_eq!(checked.ty, computation_type("Proc", Type::Int));
}

#[test]
fn task803_phase109_policy_target_rejection_is_unchanged() {
    let workflow = Workflow::Check {
        target: CheckTarget::Policy(PolicyInstance {
            name: "RateLimit".into(),
            fields: vec![],
            span: span(),
        }),
        continuation: None,
        span: span(),
    };

    assert!(
        CapabilityChecker::new().verify(&workflow).is_err(),
        "Phase 110 projection diagnostics must not relax unrelated policy-target rejection"
    );
}

#[test]
fn task803_resource_workflow_typecheck_behavior_is_unchanged() {
    let program = ash_parser::surface::Program {
        definitions: vec![ash_parser::surface::Definition::ResourceType(
            ash_parser::surface::ResourceTypeDef {
                visibility: ash_parser::surface::Visibility::Inherited,
                name: "File".into(),
                fields: vec![],
                span: span(),
            },
        )],
        helper_workflows: vec![],
        workflow: ash_parser::surface::WorkflowDef {
            name: "copy".into(),
            type_params: vec![],
            params: vec![],
            declared_return_type: None,
            plays_roles: vec![],
            capabilities: vec![],
            owned_resources: vec![ash_parser::surface::WorkflowOwnedResource {
                name: "input".into(),
                ty: SurfaceType::Name("File".into()),
                span: span(),
            }],
            used_bindings: vec![],
            header_events: vec![],
            body: Workflow::Done { span: span() },
            contract: None,
            span: span(),
        },
    };

    assert!(
        ash_typeck::type_check_program(&program).is_ok(),
        "Phase 110 diagnostics work must not perturb registered resource ownership checking"
    );
}
