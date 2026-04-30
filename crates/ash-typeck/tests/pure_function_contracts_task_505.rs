use ash_parser::surface::{
    Definition, EffectType, EnsuresClause, Expr, FnDef, Literal, Param, Program,
    Requirement as SurfaceRequirement, Type as SurfaceType, Visibility, Workflow, WorkflowDef,
};
use ash_parser::token::Span;
use ash_typeck::type_check_program;

fn span() -> Span {
    Span::default()
}

fn workflow_returning_int() -> WorkflowDef {
    WorkflowDef {
        name: "main".into(),
        type_params: vec![],
        params: vec![],
        declared_return_type: Some(SurfaceType::Name("Int".into())),
        plays_roles: vec![],
        capabilities: vec![],
        owned_resources: vec![],
        used_bindings: vec![],
        header_events: vec![],
        body: Workflow::Ret {
            expr: Expr::Literal(Literal::Int(0)),
            span: span(),
        },
        contract: None,
        span: span(),
    }
}

fn arithmetic_fn_with_contract(contract: ash_parser::surface::Contract, body: Expr) -> FnDef {
    FnDef {
        visibility: Visibility::Inherited,
        name: "checked".into(),
        type_params: vec![],
        params: vec![Param {
            name: "n".into(),
            ty: SurfaceType::Name("Int".into()),
        }],
        return_type: Some(SurfaceType::Name("Int".into())),
        contract: Some(contract),
        body,
        span: span(),
    }
}

#[test]
fn fn_requires_rejects_capability_requirements() {
    let function = arithmetic_fn_with_contract(
        ash_parser::surface::Contract {
            requires: vec![SurfaceRequirement::HasCapability {
                cap: "Fs".into(),
                min_effect: EffectType::Operational,
            }],
            ensures: vec![],
        },
        Expr::Variable {
            name: "n".into(),
            span: ash_parser::token::Span::default(),
        },
    );

    let program = Program {
        definitions: vec![Definition::Function(function)],
        helper_workflows: vec![],
        workflow: workflow_returning_int(),
    };

    let error = type_check_program(&program).expect_err("capability requirement must be rejected");
    assert!(
        error
            .to_string()
            .contains("fn contracts cannot reference capabilities")
    );
}

#[test]
fn fn_ensures_rejects_non_result_predicates() {
    let function = arithmetic_fn_with_contract(
        ash_parser::surface::Contract {
            requires: vec![],
            ensures: vec![EnsuresClause {
                expr: Expr::Binary {
                    op: ash_parser::surface::BinaryOp::Geq,
                    left: Box::new(Expr::Variable {
                        name: "state".into(),
                        span: ash_parser::token::Span::default(),
                    }),
                    right: Box::new(Expr::Literal(Literal::Int(0))),
                    span: span(),
                },
                span: span(),
            }],
        },
        Expr::Variable {
            name: "n".into(),
            span: ash_parser::token::Span::default(),
        },
    );

    let program = Program {
        definitions: vec![Definition::Function(function)],
        helper_workflows: vec![],
        workflow: workflow_returning_int(),
    };

    let error = type_check_program(&program).expect_err("non-result ensures must be rejected");
    assert!(error.to_string().contains("invalid fn ensures clause"));
}

#[test]
fn fn_ensures_rejects_non_result_equalities() {
    let function = arithmetic_fn_with_contract(
        ash_parser::surface::Contract {
            requires: vec![],
            ensures: vec![EnsuresClause {
                expr: Expr::Binary {
                    op: ash_parser::surface::BinaryOp::Eq,
                    left: Box::new(Expr::Variable {
                        name: "n".into(),
                        span: ash_parser::token::Span::default(),
                    }),
                    right: Box::new(Expr::Literal(Literal::Int(1))),
                    span: span(),
                },
                span: span(),
            }],
        },
        Expr::Variable {
            name: "n".into(),
            span: ash_parser::token::Span::default(),
        },
    );

    let program = Program {
        definitions: vec![Definition::Function(function)],
        helper_workflows: vec![],
        workflow: workflow_returning_int(),
    };

    let error = type_check_program(&program).expect_err("non-result equality ensures must fail");
    assert!(error.to_string().contains("invalid fn ensures clause"));
}

#[test]
fn fn_contract_rejects_unknown_variables() {
    let function = arithmetic_fn_with_contract(
        ash_parser::surface::Contract {
            requires: vec![SurfaceRequirement::Arithmetic {
                expr: Expr::Binary {
                    op: ash_parser::surface::BinaryOp::Gt,
                    left: Box::new(Expr::Variable {
                        name: "ghost".into(),
                        span: ash_parser::token::Span::default(),
                    }),
                    right: Box::new(Expr::Literal(Literal::Int(0))),
                    span: span(),
                },
            }],
            ensures: vec![],
        },
        Expr::Variable {
            name: "n".into(),
            span: ash_parser::token::Span::default(),
        },
    );

    let program = Program {
        definitions: vec![Definition::Function(function)],
        helper_workflows: vec![],
        workflow: workflow_returning_int(),
    };

    let error = type_check_program(&program).expect_err("unknown contract vars must be rejected");
    assert!(error.to_string().contains("unknown variable 'ghost'"));
}

#[test]
fn fn_contract_boundary_is_stored_with_runtime_postconditions() {
    let function = arithmetic_fn_with_contract(
        ash_parser::surface::Contract {
            requires: vec![SurfaceRequirement::Arithmetic {
                expr: Expr::Binary {
                    op: ash_parser::surface::BinaryOp::Geq,
                    left: Box::new(Expr::Variable {
                        name: "n".into(),
                        span: ash_parser::token::Span::default(),
                    }),
                    right: Box::new(Expr::Literal(Literal::Int(0))),
                    span: span(),
                },
            }],
            ensures: vec![EnsuresClause {
                expr: Expr::Binary {
                    op: ash_parser::surface::BinaryOp::Eq,
                    left: Box::new(Expr::Variable {
                        name: "n".into(),
                        span: ash_parser::token::Span::default(),
                    }),
                    right: Box::new(Expr::Variable {
                        name: "result".into(),
                        span: ash_parser::token::Span::default(),
                    }),
                    span: span(),
                },
                span: span(),
            }],
        },
        Expr::Variable {
            name: "n".into(),
            span: ash_parser::token::Span::default(),
        },
    );

    let program = Program {
        definitions: vec![Definition::Function(function)],
        helper_workflows: vec![],
        workflow: workflow_returning_int(),
    };

    let result = type_check_program(&program).expect("fn contract should typecheck");
    let boundary = result
        .function_contracts
        .get("checked")
        .expect("stored fn contract boundary");

    assert_eq!(boundary.contract.requires.len(), 1);
    assert_eq!(boundary.runtime_postconditions.predicates.len(), 1);
    assert!(matches!(
        boundary.runtime_postconditions.predicates.as_slice(),
        [ash_core::workflow_contract::PostPredicate::Eq(left, right)]
            if left == "result" && right == "n"
    ));
}

#[test]
fn valid_stage1_fn_contract_typechecks() {
    let function = arithmetic_fn_with_contract(
        ash_parser::surface::Contract {
            requires: vec![
                SurfaceRequirement::Arithmetic {
                    expr: Expr::Binary {
                        op: ash_parser::surface::BinaryOp::Neq,
                        left: Box::new(Expr::Variable {
                            name: "n".into(),
                            span: ash_parser::token::Span::default(),
                        }),
                        right: Box::new(Expr::Literal(Literal::Int(0))),
                        span: span(),
                    },
                },
                SurfaceRequirement::Arithmetic {
                    expr: Expr::Binary {
                        op: ash_parser::surface::BinaryOp::Eq,
                        left: Box::new(Expr::Binary {
                            op: ash_parser::surface::BinaryOp::Mod,
                            left: Box::new(Expr::Variable {
                                name: "n".into(),
                                span: ash_parser::token::Span::default(),
                            }),
                            right: Box::new(Expr::Literal(Literal::Int(2))),
                            span: span(),
                        }),
                        right: Box::new(Expr::Literal(Literal::Int(1))),
                        span: span(),
                    },
                },
            ],
            ensures: vec![EnsuresClause {
                expr: Expr::Binary {
                    op: ash_parser::surface::BinaryOp::Geq,
                    left: Box::new(Expr::Variable {
                        name: "result".into(),
                        span: ash_parser::token::Span::default(),
                    }),
                    right: Box::new(Expr::Literal(Literal::Int(0))),
                    span: span(),
                },
                span: span(),
            }],
        },
        Expr::Variable {
            name: "n".into(),
            span: ash_parser::token::Span::default(),
        },
    );

    let program = Program {
        definitions: vec![Definition::Function(function)],
        helper_workflows: vec![],
        workflow: workflow_returning_int(),
    };

    let result = type_check_program(&program);
    assert!(
        result.is_ok(),
        "expected valid fn contract to typecheck, got {result:?}"
    );
}

#[test]
fn workflow_call_site_must_prove_fn_preconditions() {
    let function = arithmetic_fn_with_contract(
        ash_parser::surface::Contract {
            requires: vec![SurfaceRequirement::Arithmetic {
                expr: Expr::Binary {
                    op: ash_parser::surface::BinaryOp::Neq,
                    left: Box::new(Expr::Variable {
                        name: "n".into(),
                        span: ash_parser::token::Span::default(),
                    }),
                    right: Box::new(Expr::Literal(Literal::Int(0))),
                    span: span(),
                },
            }],
            ensures: vec![],
        },
        Expr::Variable {
            name: "n".into(),
            span: ash_parser::token::Span::default(),
        },
    );

    let workflow = WorkflowDef {
        name: "main".into(),
        type_params: vec![],
        params: vec![],
        declared_return_type: Some(SurfaceType::Name("Int".into())),
        plays_roles: vec![],
        capabilities: vec![],
        owned_resources: vec![],
        used_bindings: vec![],
        header_events: vec![],
        body: Workflow::Let {
            pattern: ash_parser::surface::Pattern::Variable {
                name: "x".into(),
                span: ash_parser::token::Span::default(),
            },
            expr: Expr::Literal(Literal::Int(0)),
            continuation: Some(Box::new(Workflow::Ret {
                expr: Expr::Call {
                    func: "checked".into(),
                    module: None,
                    args: vec![Expr::Variable {
                        name: "x".into(),
                        span: ash_parser::token::Span::default(),
                    }],
                    span: span(),
                },
                span: span(),
            })),
            span: span(),
        },
        contract: None,
        span: span(),
    };

    let error = type_check_program(&Program {
        definitions: vec![Definition::Function(function)],
        helper_workflows: vec![],
        workflow,
    })
    .expect_err("workflow call should reject unproven fn precondition");

    assert!(error.to_string().contains("fn precondition may not hold"));
}

#[test]
fn workflow_call_site_accepts_proven_fn_preconditions() {
    let function = arithmetic_fn_with_contract(
        ash_parser::surface::Contract {
            requires: vec![SurfaceRequirement::Arithmetic {
                expr: Expr::Binary {
                    op: ash_parser::surface::BinaryOp::Neq,
                    left: Box::new(Expr::Variable {
                        name: "n".into(),
                        span: ash_parser::token::Span::default(),
                    }),
                    right: Box::new(Expr::Literal(Literal::Int(0))),
                    span: span(),
                },
            }],
            ensures: vec![],
        },
        Expr::Variable {
            name: "n".into(),
            span: ash_parser::token::Span::default(),
        },
    );

    let workflow = WorkflowDef {
        name: "main".into(),
        type_params: vec![],
        params: vec![],
        declared_return_type: Some(SurfaceType::Name("Int".into())),
        plays_roles: vec![],
        capabilities: vec![],
        owned_resources: vec![],
        used_bindings: vec![],
        header_events: vec![],
        body: Workflow::Let {
            pattern: ash_parser::surface::Pattern::Variable {
                name: "x".into(),
                span: ash_parser::token::Span::default(),
            },
            expr: Expr::Literal(Literal::Int(1)),
            continuation: Some(Box::new(Workflow::Ret {
                expr: Expr::Call {
                    func: "checked".into(),
                    module: None,
                    args: vec![Expr::Variable {
                        name: "x".into(),
                        span: ash_parser::token::Span::default(),
                    }],
                    span: span(),
                },
                span: span(),
            })),
            span: span(),
        },
        contract: None,
        span: span(),
    };

    let result = type_check_program(&Program {
        definitions: vec![Definition::Function(function)],
        helper_workflows: vec![],
        workflow,
    });
    assert!(
        result.is_ok(),
        "expected proven precondition to typecheck: {result:?}"
    );
}

#[test]
fn qualified_workflow_call_site_must_prove_fn_preconditions() {
    let mut function = arithmetic_fn_with_contract(
        ash_parser::surface::Contract {
            requires: vec![SurfaceRequirement::Arithmetic {
                expr: Expr::Binary {
                    op: ash_parser::surface::BinaryOp::Neq,
                    left: Box::new(Expr::Variable {
                        name: "n".into(),
                        span: ash_parser::token::Span::default(),
                    }),
                    right: Box::new(Expr::Literal(Literal::Int(0))),
                    span: span(),
                },
            }],
            ensures: vec![],
        },
        Expr::Variable {
            name: "n".into(),
            span: ash_parser::token::Span::default(),
        },
    );
    function.name = "math::checked".into();

    let workflow = WorkflowDef {
        name: "main".into(),
        type_params: vec![],
        params: vec![],
        declared_return_type: Some(SurfaceType::Name("Int".into())),
        plays_roles: vec![],
        capabilities: vec![],
        owned_resources: vec![],
        used_bindings: vec![],
        header_events: vec![],
        body: Workflow::Ret {
            expr: Expr::Call {
                func: "checked".into(),
                module: Some("math".into()),
                args: vec![Expr::Literal(Literal::Int(0))],
                span: span(),
            },
            span: span(),
        },
        contract: None,
        span: span(),
    };

    let error = type_check_program(&Program {
        definitions: vec![Definition::Function(function)],
        helper_workflows: vec![],
        workflow,
    })
    .expect_err("qualified call should reject unproven fn precondition");

    assert!(
        error
            .to_string()
            .contains("fn precondition may not hold for call 'math::checked'")
    );
}

#[test]
fn branch_assumptions_can_prove_stage1_preconditions() {
    let function = arithmetic_fn_with_contract(
        ash_parser::surface::Contract {
            requires: vec![SurfaceRequirement::Arithmetic {
                expr: Expr::Binary {
                    op: ash_parser::surface::BinaryOp::Neq,
                    left: Box::new(Expr::Variable {
                        name: "n".into(),
                        span: ash_parser::token::Span::default(),
                    }),
                    right: Box::new(Expr::Literal(Literal::Int(0))),
                    span: span(),
                },
            }],
            ensures: vec![],
        },
        Expr::Variable {
            name: "n".into(),
            span: ash_parser::token::Span::default(),
        },
    );

    let workflow = WorkflowDef {
        name: "main".into(),
        type_params: vec![],
        params: vec![ash_parser::surface::Parameter {
            name: "x".into(),
            ty: SurfaceType::Name("Int".into()),
            span: span(),
        }],
        declared_return_type: Some(SurfaceType::Name("Int".into())),
        plays_roles: vec![],
        capabilities: vec![],
        owned_resources: vec![],
        used_bindings: vec![],
        header_events: vec![],
        body: Workflow::If {
            condition: Expr::Binary {
                op: ash_parser::surface::BinaryOp::Gt,
                left: Box::new(Expr::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                }),
                right: Box::new(Expr::Literal(Literal::Int(0))),
                span: span(),
            },
            then_branch: Box::new(Workflow::Ret {
                expr: Expr::Call {
                    func: "checked".into(),
                    module: None,
                    args: vec![Expr::Variable {
                        name: "x".into(),
                        span: ash_parser::token::Span::default(),
                    }],
                    span: span(),
                },
                span: span(),
            }),
            else_branch: Some(Box::new(Workflow::Ret {
                expr: Expr::Literal(Literal::Int(0)),
                span: span(),
            })),
            span: span(),
        },
        contract: None,
        span: span(),
    };

    let result = type_check_program(&Program {
        definitions: vec![Definition::Function(function)],
        helper_workflows: vec![],
        workflow,
    });
    assert!(
        result.is_ok(),
        "expected branch assumption to prove precondition, got {result:?}"
    );
}

#[test]
fn arithmetic_let_facts_can_prove_stage1_modulo_preconditions() {
    let function = arithmetic_fn_with_contract(
        ash_parser::surface::Contract {
            requires: vec![SurfaceRequirement::Arithmetic {
                expr: Expr::Binary {
                    op: ash_parser::surface::BinaryOp::Eq,
                    left: Box::new(Expr::Binary {
                        op: ash_parser::surface::BinaryOp::Mod,
                        left: Box::new(Expr::Variable {
                            name: "n".into(),
                            span: ash_parser::token::Span::default(),
                        }),
                        right: Box::new(Expr::Literal(Literal::Int(2))),
                        span: span(),
                    }),
                    right: Box::new(Expr::Literal(Literal::Int(1))),
                    span: span(),
                },
            }],
            ensures: vec![],
        },
        Expr::Variable {
            name: "n".into(),
            span: ash_parser::token::Span::default(),
        },
    );

    let workflow = WorkflowDef {
        name: "main".into(),
        type_params: vec![],
        params: vec![],
        declared_return_type: Some(SurfaceType::Name("Int".into())),
        plays_roles: vec![],
        capabilities: vec![],
        owned_resources: vec![],
        used_bindings: vec![],
        header_events: vec![],
        body: Workflow::Let {
            pattern: ash_parser::surface::Pattern::Variable {
                name: "x".into(),
                span: ash_parser::token::Span::default(),
            },
            expr: Expr::Binary {
                op: ash_parser::surface::BinaryOp::Add,
                left: Box::new(Expr::Literal(Literal::Int(1))),
                right: Box::new(Expr::Literal(Literal::Int(2))),
                span: span(),
            },
            continuation: Some(Box::new(Workflow::Ret {
                expr: Expr::Call {
                    func: "checked".into(),
                    module: None,
                    args: vec![Expr::Variable {
                        name: "x".into(),
                        span: ash_parser::token::Span::default(),
                    }],
                    span: span(),
                },
                span: span(),
            })),
            span: span(),
        },
        contract: None,
        span: span(),
    };

    let result = type_check_program(&Program {
        definitions: vec![Definition::Function(function)],
        helper_workflows: vec![],
        workflow,
    });
    assert!(
        result.is_ok(),
        "expected arithmetic let fact to prove modulo precondition, got {result:?}"
    );
}
