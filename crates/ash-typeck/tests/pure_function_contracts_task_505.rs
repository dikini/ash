use ash_parser::surface::{
    BlockStmt, Definition, EffectType, EnsuresClause, Expr, FnDef, Literal, Param, Program,
    ProgramEntry, Requirement as SurfaceRequirement, Type as SurfaceType, Visibility,
};
use ash_parser::token::Span;
use ash_typeck::type_check_program;

fn span() -> Span {
    Span::default()
}

fn entry_returning_int() -> FnDef {
    FnDef {
        visibility: Visibility::Inherited,
        name: "main".into(),
        type_params: vec![],
        params: vec![],
        return_type: Some(SurfaceType::Name("Int".into())),
        proposition_tail: None,
        contract: None,
        body: Expr::Literal(Literal::Int(0)),
        span: span(),
    }
}

fn program_with_entry(mut definitions: Vec<Definition>, entry: FnDef) -> Program {
    let entry_name = entry.name.clone();
    let entry_span = entry.span;
    definitions.push(Definition::Function(entry));
    Program {
        definitions,
        entry: ProgramEntry {
            function: entry_name,
            span: entry_span,
        },
    }
}

fn arithmetic_fn_with_contract(contract: ash_parser::surface::Contract, body: Expr) -> FnDef {
    FnDef {
        visibility: Visibility::Inherited,
        name: "checked".into(),
        type_params: vec![],
        params: vec![Param {
            name: "n".into(),
            name_span: span(),
            ty: SurfaceType::Name("Int".into()),
        }],
        return_type: Some(SurfaceType::Name("Int".into())),
        proposition_tail: None,
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

    let program = program_with_entry(vec![Definition::Function(function)], entry_returning_int());

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
                    raw_operator: None,
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

    let program = program_with_entry(vec![Definition::Function(function)], entry_returning_int());

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
                    raw_operator: None,
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

    let program = program_with_entry(vec![Definition::Function(function)], entry_returning_int());

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
                    raw_operator: None,
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

    let program = program_with_entry(vec![Definition::Function(function)], entry_returning_int());

    let error = type_check_program(&program).expect_err("unknown contract vars must be rejected");
    assert!(
        error
            .to_string()
            .contains("unknown contract variable 'ghost'")
    );
}

#[test]
fn fn_contract_boundary_is_stored_with_runtime_postconditions() {
    let function = arithmetic_fn_with_contract(
        ash_parser::surface::Contract {
            requires: vec![SurfaceRequirement::Arithmetic {
                expr: Expr::Binary {
                    op: ash_parser::surface::BinaryOp::Geq,
                    raw_operator: None,
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
                    raw_operator: None,
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

    let program = program_with_entry(vec![Definition::Function(function)], entry_returning_int());

    let result = type_check_program(&program).expect("fn contract should typecheck");
    let boundary = result
        .function_contracts
        .get("checked")
        .expect("stored fn contract boundary");

    assert_eq!(boundary.contract.requires.len(), 1);
    assert_eq!(boundary.runtime_postconditions.predicates.len(), 1);
    assert!(matches!(
        boundary.runtime_postconditions.predicates.as_slice(),
        [ash_core::contract::PostPredicate::Eq(left, right)]
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
                        raw_operator: None,
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
                        raw_operator: None,
                        left: Box::new(Expr::Binary {
                            op: ash_parser::surface::BinaryOp::Mod,
                            raw_operator: None,
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
                    raw_operator: None,
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

    let program = program_with_entry(vec![Definition::Function(function)], entry_returning_int());

    let result = type_check_program(&program);
    assert!(
        result.is_ok(),
        "expected valid fn contract to typecheck, got {result:?}"
    );
}

#[test]
fn fn_body_call_site_must_prove_fn_preconditions() {
    let function = arithmetic_fn_with_contract(
        ash_parser::surface::Contract {
            requires: vec![SurfaceRequirement::Arithmetic {
                expr: Expr::Binary {
                    op: ash_parser::surface::BinaryOp::Neq,
                    raw_operator: None,
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

    let entry = FnDef {
        visibility: Visibility::Inherited,
        name: "main".into(),
        type_params: vec![],
        params: vec![],
        return_type: Some(SurfaceType::Name("Int".into())),
        proposition_tail: None,
        contract: None,
        body: Expr::Block {
            statements: vec![BlockStmt::Let {
                pattern: ash_parser::surface::Pattern::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                },
                expr: Expr::Literal(Literal::Int(0)),
                span: span(),
            }],
            tail_expr: Some(Box::new(Expr::Call {
                func: "checked".into(),
                module: None,
                args: vec![Expr::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                }],
                span: span(),
            })),
            span: span(),
        },
        span: span(),
    };

    let error = type_check_program(&program_with_entry(
        vec![Definition::Function(function)],
        entry,
    ))
    .expect_err("fn body call should reject unproven fn precondition");

    assert!(error.to_string().contains("fn precondition may not hold"));
}

#[test]
fn fn_body_call_site_accepts_proven_fn_preconditions() {
    let function = arithmetic_fn_with_contract(
        ash_parser::surface::Contract {
            requires: vec![SurfaceRequirement::Arithmetic {
                expr: Expr::Binary {
                    op: ash_parser::surface::BinaryOp::Neq,
                    raw_operator: None,
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

    let entry = FnDef {
        visibility: Visibility::Inherited,
        name: "main".into(),
        type_params: vec![],
        params: vec![],
        return_type: Some(SurfaceType::Name("Int".into())),
        proposition_tail: None,
        contract: None,
        body: Expr::Block {
            statements: vec![BlockStmt::Let {
                pattern: ash_parser::surface::Pattern::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                },
                expr: Expr::Literal(Literal::Int(1)),
                span: span(),
            }],
            tail_expr: Some(Box::new(Expr::Call {
                func: "checked".into(),
                module: None,
                args: vec![Expr::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                }],
                span: span(),
            })),
            span: span(),
        },
        span: span(),
    };

    let result = type_check_program(&program_with_entry(
        vec![Definition::Function(function)],
        entry,
    ));
    assert!(
        result.is_ok(),
        "expected proven precondition to typecheck: {result:?}"
    );
}

#[test]
fn qualified_fn_body_call_site_must_prove_fn_preconditions() {
    let mut function = arithmetic_fn_with_contract(
        ash_parser::surface::Contract {
            requires: vec![SurfaceRequirement::Arithmetic {
                expr: Expr::Binary {
                    op: ash_parser::surface::BinaryOp::Neq,
                    raw_operator: None,
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

    let entry = FnDef {
        visibility: Visibility::Inherited,
        name: "main".into(),
        type_params: vec![],
        params: vec![],
        return_type: Some(SurfaceType::Name("Int".into())),
        proposition_tail: None,
        contract: None,
        body: Expr::Call {
            func: "checked".into(),
            module: Some("math".into()),
            args: vec![Expr::Literal(Literal::Int(0))],
            span: span(),
        },
        span: span(),
    };

    let error = type_check_program(&program_with_entry(
        vec![Definition::Function(function)],
        entry,
    ))
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
                    raw_operator: None,
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

    let entry = FnDef {
        visibility: Visibility::Inherited,
        name: "main".into(),
        type_params: vec![],
        params: vec![Param {
            name: "x".into(),
            name_span: span(),
            ty: SurfaceType::Name("Int".into()),
        }],
        return_type: Some(SurfaceType::Name("Int".into())),
        proposition_tail: None,
        contract: None,
        body: Expr::If {
            condition: Box::new(Expr::Binary {
                op: ash_parser::surface::BinaryOp::Gt,
                raw_operator: None,
                left: Box::new(Expr::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                }),
                right: Box::new(Expr::Literal(Literal::Int(0))),
                span: span(),
            }),
            then_branch: Box::new(Expr::Call {
                func: "checked".into(),
                module: None,
                args: vec![Expr::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                }],
                span: span(),
            }),
            else_branch: Some(Box::new(Expr::Literal(Literal::Int(0)))),
            span: span(),
        },
        span: span(),
    };

    let result = type_check_program(&program_with_entry(
        vec![Definition::Function(function)],
        entry,
    ));
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
                    raw_operator: None,
                    left: Box::new(Expr::Binary {
                        op: ash_parser::surface::BinaryOp::Mod,
                        raw_operator: None,
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

    let entry = FnDef {
        visibility: Visibility::Inherited,
        name: "main".into(),
        type_params: vec![],
        params: vec![],
        return_type: Some(SurfaceType::Name("Int".into())),
        proposition_tail: None,
        contract: None,
        body: Expr::Block {
            statements: vec![BlockStmt::Let {
                pattern: ash_parser::surface::Pattern::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                },
                expr: Expr::Binary {
                    op: ash_parser::surface::BinaryOp::Add,
                    raw_operator: None,
                    left: Box::new(Expr::Literal(Literal::Int(1))),
                    right: Box::new(Expr::Literal(Literal::Int(2))),
                    span: span(),
                },
                span: span(),
            }],
            tail_expr: Some(Box::new(Expr::Call {
                func: "checked".into(),
                module: None,
                args: vec![Expr::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                }],
                span: span(),
            })),
            span: span(),
        },
        span: span(),
    };

    let result = type_check_program(&program_with_entry(
        vec![Definition::Function(function)],
        entry,
    ));
    assert!(
        result.is_ok(),
        "expected arithmetic let fact to prove modulo precondition, got {result:?}"
    );
}

#[test]
fn do_block_return_must_prove_fn_preconditions() {
    let function = arithmetic_fn_with_contract(
        ash_parser::surface::Contract {
            requires: vec![SurfaceRequirement::Arithmetic {
                expr: Expr::Binary {
                    op: ash_parser::surface::BinaryOp::Neq,
                    raw_operator: None,
                    left: Box::new(Expr::Variable {
                        name: "n".into(),
                        span: span(),
                    }),
                    right: Box::new(Expr::Literal(Literal::Int(0))),
                    span: span(),
                },
            }],
            ensures: vec![],
        },
        Expr::Variable {
            name: "n".into(),
            span: span(),
        },
    );
    let mut entry = entry_returning_int();
    entry.body = Expr::DoBlock {
        target: ash_parser::surface::DoTarget {
            name: "__ambient".into(),
            args: vec![],
            span: span(),
        },
        stmts: vec![ash_parser::surface::DoStmt::Return {
            value: Box::new(Expr::Call {
                func: "checked".into(),
                module: None,
                args: vec![Expr::Literal(Literal::Int(0))],
                span: span(),
            }),
            span: span(),
        }],
        span: span(),
    };

    let error = type_check_program(&program_with_entry(
        vec![Definition::Function(function)],
        entry,
    ))
    .expect_err("do-block calls must not bypass fn preconditions");
    assert!(error.to_string().contains("fn precondition may not hold"));
}
