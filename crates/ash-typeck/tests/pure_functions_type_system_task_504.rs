use ash_parser::surface::{
    BlockStmt, Definition, Expr, FnDef, ImplDef, ImplMethodDef, InterfaceDef, InterfaceMethodSig,
    Literal, Param, Pattern, Program, Type as SurfaceType, Visibility, Workflow, WorkflowDef,
};
use ash_parser::token::Span;
use ash_typeck::check_expr::check_expr;
use ash_typeck::type_env::TypeEnv;
use ash_typeck::{Type, type_check_program};

fn span() -> Span {
    Span::default()
}

fn generic_identity_fn() -> FnDef {
    FnDef {
        visibility: Visibility::Inherited,
        name: "id".into(),
        type_params: vec!["T".into()],
        params: vec![Param {
            name: "x".into(),
            ty: SurfaceType::Name("T".into()),
        }],
        return_type: Some(SurfaceType::Name("T".into())),
        proposition_tail: None,
        contract: None,
        body: Expr::Variable {
            name: "x".into(),
            span: ash_parser::token::Span::default(),
        },
        span: span(),
    }
}

fn workflow_returning(expr: Expr, return_ty: SurfaceType) -> WorkflowDef {
    WorkflowDef {
        name: "main".into(),
        type_params: vec![],
        params: vec![],
        declared_return_type: Some(return_ty),
        plays_roles: vec![],
        capabilities: vec![],
        header_events: vec![],
        body: Workflow::Ret { expr, span: span() },
        contract: None,
        span: span(),
    }
}

fn explain_interface() -> InterfaceDef {
    InterfaceDef {
        visibility: Visibility::Inherited,
        name: "Explain".into(),
        type_params: vec!["T".into()],
        evidence_constraints: vec![],
        associated_types: vec![],
        methods: vec![InterfaceMethodSig {
            name: "explain".into(),
            params: vec![SurfaceType::Name("T".into())],
            return_type: SurfaceType::Name("String".into()),
            span: span(),
        }],
        laws: Vec::new(),
        span: span(),
    }
}

fn explain_string_impl() -> ImplDef {
    ImplDef {
        visibility: Visibility::Inherited,
        interface: "Explain".into(),
        type_args: vec![SurfaceType::Name("String".into())],
        type_params: vec![],
        where_bounds: vec![],
        associated_type_bindings: vec![],
        methods: vec![ImplMethodDef {
            name: "explain".into(),
            params: vec!["value".into()],
            body: Expr::Variable {
                name: "value".into(),
                span: ash_parser::token::Span::default(),
            },
            span: span(),
        }],
        proofs: Vec::new(),
        span: span(),
    }
}

fn option_explain_interface() -> InterfaceDef {
    InterfaceDef {
        visibility: Visibility::Inherited,
        name: "OptionExplain".into(),
        type_params: vec!["T".into()],
        evidence_constraints: vec![],
        associated_types: vec![],
        methods: vec![InterfaceMethodSig {
            name: "explain".into(),
            params: vec![SurfaceType::Constructor {
                name: "Option".into(),
                args: vec![SurfaceType::Name("T".into())],
            }],
            return_type: SurfaceType::Name("String".into()),
            span: span(),
        }],
        laws: Vec::new(),
        span: span(),
    }
}

fn option_explain_string_impl() -> ImplDef {
    ImplDef {
        visibility: Visibility::Inherited,
        interface: "OptionExplain".into(),
        type_args: vec![SurfaceType::Name("String".into())],
        type_params: vec![],
        where_bounds: vec![],
        associated_type_bindings: vec![],
        methods: vec![ImplMethodDef {
            name: "explain".into(),
            params: vec!["value".into()],
            body: Expr::Literal(Literal::String("string option".into())),
            span: span(),
        }],
        proofs: Vec::new(),
        span: span(),
    }
}

#[test]
fn generic_pure_fn_call_instantiates_at_workflow_call_site() {
    let program = Program {
        definitions: vec![Definition::Function(generic_identity_fn())],
        helper_workflows: vec![],
        workflow: workflow_returning(
            Expr::Call {
                func: "id".into(),
                module: None,
                args: vec![Expr::Literal(Literal::Int(7))],
                span: span(),
            },
            SurfaceType::Name("Int".into()),
        ),
    };

    let result = type_check_program(&program);
    assert!(
        result.is_ok(),
        "expected generic fn call to typecheck, got {result:?}"
    );
}

#[test]
fn one_armed_if_requires_null_then_branch() {
    let env = TypeEnv::with_builtin_types();
    let expr = Expr::If {
        condition: Box::new(Expr::Literal(Literal::Bool(true))),
        then_branch: Box::new(Expr::Literal(Literal::Int(1))),
        else_branch: None,
        span: span(),
    };

    let result = check_expr(&env, &expr);
    assert!(
        !result.is_ok(),
        "expected one-armed if with Int branch to fail"
    );
    assert!(
        result
            .errors
            .iter()
            .any(|error| error.to_string().contains("type Null")
                || error.to_string().contains("Null")),
        "expected Null typing diagnostic, got {:?}",
        result.errors
    );
}

#[test]
fn one_armed_if_with_null_then_branch_types_as_null() {
    let env = TypeEnv::with_builtin_types();
    let expr = Expr::If {
        condition: Box::new(Expr::Literal(Literal::Bool(true))),
        then_branch: Box::new(Expr::Literal(Literal::Null)),
        else_branch: None,
        span: span(),
    };

    let result = check_expr(&env, &expr);
    assert!(
        result.is_ok(),
        "expected one-armed null if to typecheck: {:?}",
        result.errors
    );
    assert_eq!(result.substitution.apply(&result.ty), Type::Null);
}

#[test]
fn fn_body_call_must_target_pure_function_type() {
    let bad_function = FnDef {
        visibility: Visibility::Inherited,
        name: "bad".into(),
        type_params: vec![],
        params: vec![Param {
            name: "not_fn".into(),
            ty: SurfaceType::Name("Int".into()),
        }],
        return_type: Some(SurfaceType::Name("Int".into())),
        proposition_tail: None,
        contract: None,
        body: Expr::Call {
            func: "not_fn".into(),
            module: None,
            args: vec![Expr::Literal(Literal::Int(1))],
            span: span(),
        },
        span: span(),
    };

    let program = Program {
        definitions: vec![Definition::Function(bad_function)],
        helper_workflows: vec![],
        workflow: workflow_returning(
            Expr::Literal(Literal::Int(0)),
            SurfaceType::Name("Int".into()),
        ),
    };

    let result = type_check_program(&program);
    assert!(
        result.is_err(),
        "expected non-function call in fn body to be rejected"
    );
    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("not pure") || error.contains("not allowed in pure function"),
        "unexpected error: {error}"
    );
}

#[test]
fn interface_method_call_is_allowed_in_pure_fn_when_impl_exists() {
    let describe_fn = FnDef {
        visibility: Visibility::Inherited,
        name: "describe".into(),
        type_params: vec![],
        params: vec![Param {
            name: "value".into(),
            ty: SurfaceType::Name("String".into()),
        }],
        return_type: Some(SurfaceType::Name("String".into())),
        proposition_tail: None,
        contract: None,
        body: Expr::Call {
            func: "explain".into(),
            module: Some("Explain".into()),
            args: vec![Expr::Variable {
                name: "value".into(),
                span: ash_parser::token::Span::default(),
            }],
            span: span(),
        },
        span: span(),
    };

    let program = Program {
        definitions: vec![
            Definition::Interface(explain_interface()),
            Definition::Impl(explain_string_impl()),
            Definition::Function(describe_fn),
            Definition::Function(generic_identity_fn()),
        ],
        helper_workflows: vec![],
        workflow: workflow_returning(
            Expr::Call {
                func: "describe".into(),
                module: None,
                args: vec![Expr::Literal(Literal::String("hello".into()))],
                span: span(),
            },
            SurfaceType::Name("String".into()),
        ),
    };

    let result = type_check_program(&program);
    assert!(
        result.is_ok(),
        "expected interface method call in pure fn to typecheck, got {result:?}"
    );
}

#[test]
fn if_let_pattern_binding_is_in_scope_for_interface_call_validation() {
    let describe_fn = FnDef {
        visibility: Visibility::Inherited,
        name: "describe_if_let".into(),
        type_params: vec![],
        params: vec![Param {
            name: "value".into(),
            ty: SurfaceType::Name("String".into()),
        }],
        return_type: Some(SurfaceType::Name("String".into())),
        proposition_tail: None,
        contract: None,
        body: Expr::IfLet {
            pattern: Pattern::Variable {
                name: "bound".into(),
                span: ash_parser::token::Span::default(),
            },
            expr: Box::new(Expr::Variable {
                name: "value".into(),
                span: ash_parser::token::Span::default(),
            }),
            then_branch: Box::new(Expr::Call {
                func: "explain".into(),
                module: Some("Explain".into()),
                args: vec![Expr::Variable {
                    name: "bound".into(),
                    span: ash_parser::token::Span::default(),
                }],
                span: span(),
            }),
            else_branch: Box::new(Expr::Literal(Literal::String("fallback".into()))),
            span: span(),
        },
        span: span(),
    };

    let program = Program {
        definitions: vec![
            Definition::Interface(explain_interface()),
            Definition::Impl(explain_string_impl()),
            Definition::Function(describe_fn),
        ],
        helper_workflows: vec![],
        workflow: workflow_returning(
            Expr::Call {
                func: "describe_if_let".into(),
                module: None,
                args: vec![Expr::Literal(Literal::String("hello".into()))],
                span: span(),
            },
            SurfaceType::Name("String".into()),
        ),
    };

    let result = type_check_program(&program);
    assert!(
        result.is_ok(),
        "expected if-let binding to be available in then branch, got {result:?}"
    );
}

#[test]
fn block_let_binding_is_in_scope_for_later_interface_call_validation() {
    let describe_fn = FnDef {
        visibility: Visibility::Inherited,
        name: "describe_block".into(),
        type_params: vec![],
        params: vec![],
        return_type: Some(SurfaceType::Name("String".into())),
        proposition_tail: None,
        contract: None,
        body: Expr::Block {
            statements: vec![BlockStmt::Let {
                pattern: Pattern::Variable {
                    name: "bound".into(),
                    span: ash_parser::token::Span::default(),
                },
                expr: Expr::Literal(Literal::String("hello".into())),
                span: span(),
            }],
            tail_expr: Some(Box::new(Expr::Call {
                func: "explain".into(),
                module: Some("Explain".into()),
                args: vec![Expr::Variable {
                    name: "bound".into(),
                    span: ash_parser::token::Span::default(),
                }],
                span: span(),
            })),
            span: span(),
        },
        span: span(),
    };

    let program = Program {
        definitions: vec![
            Definition::Interface(explain_interface()),
            Definition::Impl(explain_string_impl()),
            Definition::Function(describe_fn),
        ],
        helper_workflows: vec![],
        workflow: workflow_returning(
            Expr::Call {
                func: "describe_block".into(),
                module: None,
                args: vec![],
                span: span(),
            },
            SurfaceType::Name("String".into()),
        ),
    };

    let result = type_check_program(&program);
    assert!(
        result.is_ok(),
        "expected block let binding to be available in tail expression, got {result:?}"
    );
}

#[test]
fn non_callable_call_reports_non_callable_diagnostic() {
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable("value", Type::Int);

    let result = check_expr(
        &env,
        &Expr::Call {
            func: "value".into(),
            module: None,
            args: vec![Expr::Literal(Literal::Int(1))],
            span: span(),
        },
    );

    assert!(!result.is_ok(), "expected non-callable value call to fail");
    let messages: Vec<String> = result.errors.iter().map(ToString::to_string).collect();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("not callable") && message.contains("Int")),
        "expected non-callable diagnostic, got {messages:?}"
    );
    assert!(
        messages
            .iter()
            .all(|message| !message.contains("expected 0 args")),
        "non-callable diagnostic should not report bogus arity, got {messages:?}"
    );
}

#[test]
fn panic_can_satisfy_non_null_expected_type() {
    let program = Program {
        definitions: vec![],
        helper_workflows: vec![],
        workflow: workflow_returning(
            Expr::Panic {
                message: "boom".into(),
                span: span(),
            },
            SurfaceType::Name("Int".into()),
        ),
    };

    let result = type_check_program(&program);
    assert!(
        result.is_ok(),
        "expected panic to typecheck at Int, got {result:?}"
    );
}

#[test]
fn if_with_panic_else_branch_preserves_other_branch_type() {
    let env = TypeEnv::with_builtin_types();
    let expr = Expr::If {
        condition: Box::new(Expr::Literal(Literal::Bool(true))),
        then_branch: Box::new(Expr::Literal(Literal::Int(1))),
        else_branch: Some(Box::new(Expr::Panic {
            message: "boom".into(),
            span: span(),
        })),
        span: span(),
    };

    let result = check_expr(&env, &expr);
    assert!(
        result.is_ok(),
        "expected if with panic branch to typecheck, got {:?}",
        result.errors
    );
    assert_eq!(result.substitution.apply(&result.ty), Type::Int);
}

#[test]
fn interface_method_resolution_unifies_nested_generic_argument_types() {
    let describe_fn = FnDef {
        visibility: Visibility::Inherited,
        name: "describe_option".into(),
        type_params: vec![],
        params: vec![Param {
            name: "value".into(),
            ty: SurfaceType::Constructor {
                name: "Option".into(),
                args: vec![SurfaceType::Name("String".into())],
            },
        }],
        return_type: Some(SurfaceType::Name("String".into())),
        proposition_tail: None,
        contract: None,
        body: Expr::Call {
            func: "explain".into(),
            module: Some("OptionExplain".into()),
            args: vec![Expr::Variable {
                name: "value".into(),
                span: ash_parser::token::Span::default(),
            }],
            span: span(),
        },
        span: span(),
    };

    let program = Program {
        definitions: vec![
            Definition::Interface(option_explain_interface()),
            Definition::Impl(option_explain_string_impl()),
            Definition::Function(describe_fn),
        ],
        helper_workflows: vec![],
        workflow: workflow_returning(
            Expr::Call {
                func: "describe_option".into(),
                module: None,
                args: vec![Expr::Constructor {
                    name: "Some".into(),
                    fields: vec![(
                        "value".into(),
                        Expr::Literal(Literal::String("hello".into())),
                    )],
                    payload: ash_parser::surface::ConstructorPayload::Record(vec![(
                        "value".into(),
                        Expr::Literal(Literal::String("hello".into())),
                    )]),
                    span: span(),
                }],
                span: span(),
            },
            SurfaceType::Name("String".into()),
        ),
    };

    let result = type_check_program(&program);
    assert!(
        result.is_ok(),
        "expected nested generic interface method call to typecheck, got {result:?}"
    );
}

#[test]
fn qualified_pure_fn_call_accepts_exact_qualified_binding() {
    let passthrough_fn = FnDef {
        visibility: Visibility::Inherited,
        name: "math::passthrough".into(),
        type_params: vec![],
        params: vec![Param {
            name: "x".into(),
            ty: SurfaceType::Name("Int".into()),
        }],
        return_type: Some(SurfaceType::Name("Int".into())),
        proposition_tail: None,
        contract: None,
        body: Expr::Variable {
            name: "x".into(),
            span: ash_parser::token::Span::default(),
        },
        span: span(),
    };

    let uses_qualified_call = FnDef {
        visibility: Visibility::Inherited,
        name: "call_passthrough".into(),
        type_params: vec![],
        params: vec![Param {
            name: "value".into(),
            ty: SurfaceType::Name("Int".into()),
        }],
        return_type: Some(SurfaceType::Name("Int".into())),
        proposition_tail: None,
        contract: None,
        body: Expr::Call {
            func: "passthrough".into(),
            module: Some("math".into()),
            args: vec![Expr::Variable {
                name: "value".into(),
                span: ash_parser::token::Span::default(),
            }],
            span: span(),
        },
        span: span(),
    };

    let program = Program {
        definitions: vec![
            Definition::Function(passthrough_fn),
            Definition::Function(uses_qualified_call),
        ],
        helper_workflows: vec![],
        workflow: workflow_returning(
            Expr::Call {
                func: "call_passthrough".into(),
                module: None,
                args: vec![Expr::Literal(Literal::Int(7))],
                span: span(),
            },
            SurfaceType::Name("Int".into()),
        ),
    };

    let result = type_check_program(&program);
    assert!(
        result.is_ok(),
        "expected exact qualified binding to typecheck, got {result:?}"
    );
}

#[test]
fn qualified_pure_fn_call_requires_exact_qualified_binding() {
    let passthrough_fn = FnDef {
        visibility: Visibility::Inherited,
        name: "passthrough".into(),
        type_params: vec![],
        params: vec![Param {
            name: "x".into(),
            ty: SurfaceType::Name("Int".into()),
        }],
        return_type: Some(SurfaceType::Name("Int".into())),
        proposition_tail: None,
        contract: None,
        body: Expr::Variable {
            name: "x".into(),
            span: ash_parser::token::Span::default(),
        },
        span: span(),
    };

    let uses_qualified_call = FnDef {
        visibility: Visibility::Inherited,
        name: "call_passthrough".into(),
        type_params: vec![],
        params: vec![Param {
            name: "value".into(),
            ty: SurfaceType::Name("Int".into()),
        }],
        return_type: Some(SurfaceType::Name("Int".into())),
        proposition_tail: None,
        contract: None,
        body: Expr::Call {
            func: "passthrough".into(),
            module: Some("math".into()),
            args: vec![Expr::Variable {
                name: "value".into(),
                span: ash_parser::token::Span::default(),
            }],
            span: span(),
        },
        span: span(),
    };

    let program = Program {
        definitions: vec![
            Definition::Function(passthrough_fn),
            Definition::Function(uses_qualified_call),
        ],
        helper_workflows: vec![],
        workflow: workflow_returning(
            Expr::Call {
                func: "call_passthrough".into(),
                module: None,
                args: vec![Expr::Literal(Literal::Int(7))],
                span: span(),
            },
            SurfaceType::Name("Int".into()),
        ),
    };

    let result = type_check_program(&program);
    assert!(
        result.is_err(),
        "expected missing qualified binding to be rejected, got {result:?}"
    );
    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("math::passthrough") || error.contains("call to unknown function"),
        "unexpected error: {error}"
    );
}

#[test]
fn omitted_return_type_is_rechecked_before_downstream_callers_are_accepted() {
    let returns_int = FnDef {
        visibility: Visibility::Inherited,
        name: "returns_int".into(),
        type_params: vec![],
        params: vec![],
        return_type: None,
        proposition_tail: None,
        contract: None,
        body: Expr::Literal(Literal::Int(1)),
        span: span(),
    };

    let claims_string = FnDef {
        visibility: Visibility::Inherited,
        name: "claims_string".into(),
        type_params: vec![],
        params: vec![],
        return_type: Some(SurfaceType::Name("String".into())),
        proposition_tail: None,
        contract: None,
        body: Expr::Call {
            func: "returns_int".into(),
            module: None,
            args: vec![],
            span: span(),
        },
        span: span(),
    };

    let program = Program {
        definitions: vec![
            Definition::Function(returns_int),
            Definition::Function(claims_string),
        ],
        helper_workflows: vec![],
        workflow: workflow_returning(
            Expr::Call {
                func: "claims_string".into(),
                module: None,
                args: vec![],
                span: span(),
            },
            SurfaceType::Name("String".into()),
        ),
    };

    let result = type_check_program(&program);
    assert!(
        result.is_err(),
        "expected stale provisional return type to be rejected"
    );
    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("fn 'claims_string' declared return type String but body returns Int"),
        "unexpected error: {error}"
    );
}
