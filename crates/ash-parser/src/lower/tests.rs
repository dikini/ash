//! Lowering tests.

use super::*;
use crate::surface::{
    BinaryOp, Contract as SurfaceContract, DoStmt, DoTarget, EffectType, EnsuresClause,
    Expr as SurfaceExpr, Literal as SurfaceLiteral, Pattern, Requirement as SurfaceRequirement,
    RoleDef, Workflow as SurfaceWorkflow,
};
use crate::token::Span;
use std::collections::HashSet;

fn dummy_span() -> Span {
    Span::new(0, 0, 1, 1)
}

fn int_expr(value: i64) -> SurfaceExpr {
    SurfaceExpr::Literal(SurfaceLiteral::Int(value))
}

fn var_expr(name: &str) -> SurfaceExpr {
    SurfaceExpr::Variable {
        name: name.into(),
        span: crate::token::Span::default(),
    }
}

#[test]
fn test_lower_do_block_act_return_rejects_parser_only_lowering() {
    let surface = SurfaceExpr::DoBlock {
        target: DoTarget {
            name: "Act".into(),
            args: vec![],
            span: Span::default(),
        },
        stmts: vec![DoStmt::Return {
            value: Box::new(int_expr(1)),
            span: Span::default(),
        }],
        span: Span::default(),
    };

    let err = lower_expr(&surface).expect_err("generic do block must require typed elaboration");
    assert!(matches!(
        err,
        LoweringError::ExprNotLowerable { kind }
            if kind.contains("typed do elaboration")
    ));
}

#[test]
fn test_lower_do_block_proc_bind_rejects_parser_only_lowering() {
    let surface = SurfaceExpr::DoBlock {
        target: DoTarget {
            name: "Proc".into(),
            args: vec![],
            span: Span::default(),
        },
        stmts: vec![
            DoStmt::Bind {
                name: "x".into(),
                value: Box::new(SurfaceExpr::Call {
                    func: "unit".into(),
                    module: Some("proc".into()),
                    args: vec![int_expr(1)],
                    span: Span::default(),
                }),
                span: Span::default(),
            },
            DoStmt::Return {
                value: Box::new(var_expr("x")),
                span: Span::default(),
            },
        ],
        span: Span::default(),
    };

    let err = lower_expr(&surface).expect_err("generic do block must require typed elaboration");
    assert!(matches!(
        err,
        LoweringError::ExprNotLowerable { kind }
            if kind.contains("typed do elaboration")
    ));
}

#[test]
fn test_lower_done() {
    let surface = SurfaceWorkflow::Done { span: dummy_span() };
    let core = lower_workflow_body(&surface, &Provenance::new(), &LoweringContext::new()).unwrap();
    assert!(matches!(core, CoreWorkflow::Done));
}

#[test]
fn test_lower_let() {
    let surface = SurfaceWorkflow::Let {
        pattern: Pattern::Variable {
            name: "x".into(),
            span: crate::token::Span::default(),
        },
        expr: SurfaceExpr::Literal(SurfaceLiteral::Int(42)),
        continuation: Some(Box::new(SurfaceWorkflow::Done { span: dummy_span() })),
        span: dummy_span(),
    };
    let core = lower_workflow_body(&surface, &Provenance::new(), &LoweringContext::new()).unwrap();
    assert!(matches!(core, CoreWorkflow::Let { .. }));
}

#[test]
fn test_lower_expr_literal() {
    let surface = SurfaceExpr::Literal(SurfaceLiteral::Int(42));
    let core = lower_expr(&surface).unwrap();
    assert!(matches!(core, CoreExpr::Literal(ash_core::Value::Int(42))));
}

#[test]
fn test_lower_expr_variable() {
    let surface = SurfaceExpr::Variable {
        name: "my_var".into(),
        span: crate::token::Span::default(),
    };
    let core = lower_expr(&surface).unwrap();
    assert!(matches!(core, CoreExpr::Variable { name, .. } if name == "my_var"));
}

#[test]
fn test_lower_expr_binary() {
    let surface = SurfaceExpr::Binary {
        op: BinaryOp::Add,
        raw_operator: None,
        left: Box::new(SurfaceExpr::Literal(SurfaceLiteral::Int(1))),
        right: Box::new(SurfaceExpr::Literal(SurfaceLiteral::Int(2))),
        span: dummy_span(),
    };
    let core = lower_expr(&surface).unwrap();
    assert!(matches!(
        core,
        CoreExpr::Binary {
            op: ash_core::BinaryOp::Add,
            ..
        }
    ));
}

#[test]
#[allow(clippy::approx_constant)]
fn test_lower_expr_float_literal_error() {
    let surface = SurfaceExpr::Literal(SurfaceLiteral::Float(ordered_float::OrderedFloat(3.14)));
    let result = lower_expr(&surface);
    assert!(matches!(result, Err(LoweringError::FloatNotSupported)));
}

#[test]
fn test_interface_method_call_lowers_as_call() {
    // After TASK-561, interface method calls use Expr::Call with module qualifier
    let surface = SurfaceExpr::Call {
        func: "explain".into(),
        module: Some("Explain".into()),
        args: vec![SurfaceExpr::Variable {
            name: "value".into(),
            span: crate::token::Span::default(),
        }],
        span: crate::token::Span::new(0, 22, 1, 1),
    };

    let result = lower_expr(&surface);
    assert!(result.is_ok());
    let core = result.unwrap();
    match &core {
        CoreExpr::Call {
            func,
            module,
            arguments,
        } => {
            assert_eq!(func, "explain");
            assert_eq!(module.as_deref(), Some("Explain"));
            assert_eq!(arguments.len(), 1);
        }
        other => panic!("expected CoreExpr::Call, got {other:?}"),
    }
}

#[test]
fn test_lower_pattern_variable() {
    let surface = Pattern::Variable {
        name: "x".into(),
        span: crate::token::Span::default(),
    };
    let core = lower_pattern(&surface).unwrap();
    assert!(matches!(core, CorePattern::Variable { name, .. } if name == "x"));
}

#[test]
fn test_lower_pattern_wildcard() {
    let surface = Pattern::Wildcard;
    let core = lower_pattern(&surface).unwrap();
    assert!(matches!(core, CorePattern::Wildcard));
}

#[test]
fn test_lower_pattern_tuple() {
    let surface = Pattern::Tuple(vec![
        Pattern::Variable {
            name: "a".into(),
            span: crate::token::Span::default(),
        },
        Pattern::Variable {
            name: "b".into(),
            span: crate::token::Span::default(),
        },
    ]);
    let core = lower_pattern(&surface).unwrap();
    assert!(matches!(core, CorePattern::Tuple(pats) if pats.len() == 2));
}

#[test]
fn test_lower_literal_int() {
    let surface = SurfaceLiteral::Int(42);
    let core = lower_literal(&surface).unwrap();
    assert!(matches!(core, ash_core::Value::Int(42)));
}

#[test]
fn test_lower_literal_string() {
    let surface = SurfaceLiteral::String("hello".into());
    let core = lower_literal(&surface).unwrap();
    assert!(matches!(core, ash_core::Value::String(s) if s == "hello"));
}

#[test]
fn test_lower_obligation_uses_simplified_role_shape() {
    let surface = ObligationRef {
        role: "manager".into(),
        condition: SurfaceExpr::Variable {
            name: "approved".into(),
            span: crate::token::Span::default(),
        },
    };

    let core = lower_obligation(&surface).unwrap();

    assert!(matches!(
        core,
        CoreObligation::Obliged {
            role: CoreRole {
                name,
                authority,
                obligations,
            },
            condition: CoreExpr::Variable { name: condition, .. },
        } if name == "manager"
            && authority.is_empty()
            && obligations.is_empty()
            && condition == "approved"
    ));
}

#[test]
fn test_lower_role_def_preserves_named_capability_refs_and_obligation_refs() {
    let surface = RoleDef {
        name: "reviewer".into(),
        capabilities: vec![
            crate::surface::CapabilityDecl {
                capability: "approve".into(),
                constraints: None,
                span: dummy_span(),
            },
            crate::surface::CapabilityDecl {
                capability: "review".into(),
                constraints: None,
                span: dummy_span(),
            },
        ],
        obligations: vec!["check_tests".into()],
        span: dummy_span(),
    };

    let definitions = vec![
        crate::surface::Definition::Capability(crate::surface::CapabilityDef {
            visibility: crate::surface::Visibility::Inherited,
            name: "approve".into(),
            effect: crate::surface::EffectType::Decide,
            params: vec![],
            return_type: None,
            constraints: vec![],
            target_provider: None,
            target_action: None,
            span: dummy_span(),
        }),
        crate::surface::Definition::Capability(crate::surface::CapabilityDef {
            visibility: crate::surface::Visibility::Inherited,
            name: "review".into(),
            effect: crate::surface::EffectType::Analyze,
            params: vec![],
            return_type: None,
            constraints: vec![],
            target_provider: None,
            target_action: None,
            span: dummy_span(),
        }),
    ];

    let core = lower_role_def_with_definitions(&surface, &definitions)
        .expect("matching capability definitions should lower authority metadata");

    assert_eq!(core.name, "reviewer");
    assert_eq!(core.authority.len(), 2);
    assert!(matches!(
        &core.authority[0],
        Capability { name, .. } if name == "approve"
    ));
    assert!(matches!(
        &core.authority[1],
        Capability { name, .. } if name == "review"
    ));
    assert!(matches!(
        &core.obligations[..],
        [ash_core::RoleObligationRef { name }] if name == "check_tests"
    ));
}

#[test]
fn test_lower_module_role_definitions_only_lowers_roles() {
    let module = crate::module::ModuleDecl::inline(
        "governance".into(),
        crate::surface::Visibility::Inherited,
        vec![
            crate::surface::Definition::Capability(crate::surface::CapabilityDef {
                visibility: crate::surface::Visibility::Inherited,
                name: "approve".into(),
                effect: crate::surface::EffectType::Read,
                params: vec![],
                return_type: None,
                constraints: vec![],
                target_provider: None,
                target_action: None,
                span: dummy_span(),
            }),
            crate::surface::Definition::Role(RoleDef {
                name: "reviewer".into(),
                capabilities: vec![crate::surface::CapabilityDecl {
                    capability: "approve".into(),
                    constraints: None,
                    span: dummy_span(),
                }],
                obligations: vec!["check_tests".into()],
                span: dummy_span(),
            }),
        ],
        dummy_span(),
    );

    let roles = lower_module_role_definitions(&module)
        .expect("matching capability definitions should lower authority metadata");

    assert_eq!(roles.len(), 1);
    assert_eq!(roles[0].name, "reviewer");
    assert!(matches!(
        &roles[0].obligations[..],
        [ash_core::RoleObligationRef { name }] if name == "check_tests"
    ));
}

#[test]
fn test_lower_module_role_definitions_preserves_authority_metadata_from_module_capabilities() {
    let module = crate::module::ModuleDecl::inline(
        "governance".into(),
        crate::surface::Visibility::Inherited,
        vec![
            crate::surface::Definition::Capability(crate::surface::CapabilityDef {
                visibility: crate::surface::Visibility::Inherited,
                name: "approve".into(),
                effect: crate::surface::EffectType::Decide,
                params: vec![],
                return_type: None,
                constraints: vec![crate::surface::Constraint {
                    predicate: crate::surface::Predicate {
                        name: "requires_mfa".into(),
                        args: vec![],
                    },
                }],
                target_provider: None,
                target_action: None,
                span: dummy_span(),
            }),
            crate::surface::Definition::Role(RoleDef {
                name: "reviewer".into(),
                capabilities: vec![crate::surface::CapabilityDecl {
                    capability: "approve".into(),
                    constraints: None,
                    span: dummy_span(),
                }],
                obligations: vec!["check_tests".into()],
                span: dummy_span(),
            }),
        ],
        dummy_span(),
    );

    let roles = lower_module_role_definitions(&module)
        .expect("matching capability definitions should lower authority metadata");

    assert_eq!(roles.len(), 1);
    assert!(matches!(
        &roles[0].authority[..],
        [Capability {
            name,
            effect: Effect::Evaluative,
            constraints,
        }] if name == "approve"
            && matches!(
                &constraints[..],
                [ash_core::Constraint {
                    predicate: ash_core::Predicate { name: predicate_name, arguments }
                }] if predicate_name == "requires_mfa" && arguments.is_empty()
            )
    ));
}

#[test]
fn test_lower_unary_op() {
    assert!(matches!(
        lower_unary_op(UnaryOp::Not),
        ash_core::UnaryOp::Not
    ));
    assert!(matches!(
        lower_unary_op(UnaryOp::Neg),
        ash_core::UnaryOp::Neg
    ));
}

#[test]
fn test_lower_binary_op() {
    assert!(matches!(
        lower_binary_op(BinaryOp::Add).unwrap(),
        ash_core::BinaryOp::Add
    ));
    assert!(matches!(
        lower_binary_op(BinaryOp::Sub).unwrap(),
        ash_core::BinaryOp::Sub
    ));
    assert!(matches!(
        lower_binary_op(BinaryOp::Mul).unwrap(),
        ash_core::BinaryOp::Mul
    ));
    assert!(matches!(
        lower_binary_op(BinaryOp::Div).unwrap(),
        ash_core::BinaryOp::Div
    ));
    assert!(matches!(
        lower_binary_op(BinaryOp::Mod).unwrap(),
        ash_core::BinaryOp::Mod
    ));
    assert!(matches!(
        lower_binary_op(BinaryOp::Eq).unwrap(),
        ash_core::BinaryOp::Eq
    ));
    assert!(matches!(
        lower_binary_op(BinaryOp::And).unwrap(),
        ash_core::BinaryOp::And
    ));
    assert!(matches!(
        lower_binary_op(BinaryOp::Or).unwrap(),
        ash_core::BinaryOp::Or
    ));
}

#[test]
fn test_lower_fn_contract_stage1_predicates() {
    let contract = SurfaceContract {
        requires: vec![
            SurfaceRequirement::Arithmetic {
                expr: SurfaceExpr::Binary {
                    op: BinaryOp::Geq,
                    raw_operator: None,
                    left: Box::new(var_expr("n")),
                    right: Box::new(int_expr(0)),
                    span: dummy_span(),
                },
            },
            SurfaceRequirement::Arithmetic {
                expr: SurfaceExpr::Binary {
                    op: BinaryOp::Neq,
                    raw_operator: None,
                    left: Box::new(var_expr("d")),
                    right: Box::new(int_expr(0)),
                    span: dummy_span(),
                },
            },
            SurfaceRequirement::Arithmetic {
                expr: SurfaceExpr::Binary {
                    op: BinaryOp::Eq,
                    raw_operator: None,
                    left: Box::new(SurfaceExpr::Binary {
                        op: BinaryOp::Mod,
                        raw_operator: None,
                        left: Box::new(var_expr("n")),
                        right: Box::new(int_expr(2)),
                        span: dummy_span(),
                    }),
                    right: Box::new(int_expr(1)),
                    span: dummy_span(),
                },
            },
        ],
        ensures: vec![EnsuresClause {
            expr: SurfaceExpr::Binary {
                op: BinaryOp::Geq,
                raw_operator: None,
                left: Box::new(var_expr("result")),
                right: Box::new(int_expr(0)),
                span: dummy_span(),
            },
            span: dummy_span(),
        }],
    };

    let ctx = FnContractLoweringContext {
        name: "safe_div",
        params: &[
            (
                "n".to_string(),
                ash_core::core_ash::CoreType::Base("Int".to_string()),
            ),
            (
                "d".to_string(),
                ash_core::core_ash::CoreType::Base("Int".to_string()),
            ),
        ],
        result: Some(ash_core::core_ash::CoreType::Base("Int".to_string())),
    };

    let lowered = lower_fn_contract(Some(&contract), &ctx).expect("fn contract should lower");
    assert_eq!(lowered.contract.requires.len(), 3);
    assert_eq!(lowered.runtime_postconditions.predicates.len(), 1);
    assert!(matches!(
        &lowered.contract.requires[0],
        ash_core::workflow_contract::Requirement::Arithmetic { var, constraint }
            if var == "n"
                && matches!(constraint, ash_core::workflow_contract::ArithConstraint::Gte(0))
    ));
    assert!(matches!(
        &lowered.contract.requires[1],
        ash_core::workflow_contract::Requirement::Arithmetic { var, constraint }
            if var == "d"
                && matches!(constraint, ash_core::workflow_contract::ArithConstraint::NotEq(0))
    ));
    assert!(matches!(
        &lowered.contract.requires[2],
        ash_core::workflow_contract::Requirement::Arithmetic { var, constraint }
            if var == "n"
                && matches!(
                    constraint,
                    ash_core::workflow_contract::ArithConstraint::Modulo { div: 2, rem: 1 }
                )
    ));
    assert!(matches!(
        &lowered.runtime_postconditions.predicates[0],
        ash_core::workflow_contract::PostPredicate::ResultSatisfies(
            ash_core::workflow_contract::ArithConstraint::Gte(0)
        )
    ));
}

#[test]
fn test_lower_fn_contract_rejects_non_value_ensures() {
    let contract = SurfaceContract {
        requires: vec![],
        ensures: vec![EnsuresClause {
            expr: SurfaceExpr::Binary {
                op: BinaryOp::Geq,
                raw_operator: None,
                left: Box::new(var_expr("state")),
                right: Box::new(int_expr(0)),
                span: dummy_span(),
            },
            span: dummy_span(),
        }],
    };

    let ctx = FnContractLoweringContext {
        name: "_test",
        params: &[(
            "state".to_string(),
            ash_core::core_ash::CoreType::Base("Int".to_string()),
        )],
        result: Some(ash_core::core_ash::CoreType::Base("Int".to_string())),
    };

    let error = lower_fn_contract(Some(&contract), &ctx).expect_err("invalid ensures should fail");
    assert!(matches!(
        error,
        FnContractLoweringError::InvalidEnsures { .. }
    ));
}

#[test]
fn test_lower_if() {
    let surface = SurfaceWorkflow::If {
        condition: SurfaceExpr::Literal(SurfaceLiteral::Bool(true)),
        then_branch: Box::new(SurfaceWorkflow::Done { span: dummy_span() }),
        else_branch: Some(Box::new(SurfaceWorkflow::Done { span: dummy_span() })),
        span: dummy_span(),
    };
    let core = lower_workflow_body(&surface, &Provenance::new(), &LoweringContext::new()).unwrap();
    assert!(matches!(core, CoreWorkflow::If { .. }));
}

#[test]
fn test_lower_seq() {
    let surface = SurfaceWorkflow::Seq {
        first: Box::new(SurfaceWorkflow::Observe {
            capability: "read".into(),
            binding: None,
            continuation: None,
            span: dummy_span(),
        }),
        second: Box::new(SurfaceWorkflow::Done { span: dummy_span() }),
        span: dummy_span(),
    };
    let core = lower_workflow_body(&surface, &Provenance::new(), &LoweringContext::new()).unwrap();
    assert!(matches!(core, CoreWorkflow::Seq { .. }));
}

#[test]
fn test_lower_observe() {
    let surface = SurfaceWorkflow::Observe {
        capability: "read".into(),
        binding: Some(Pattern::Variable {
            name: "x".into(),
            span: crate::token::Span::default(),
        }),
        continuation: None,
        span: dummy_span(),
    };
    let core = lower_workflow_body(&surface, &Provenance::new(), &LoweringContext::new()).unwrap();
    assert!(matches!(core, CoreWorkflow::Observe { .. }));
}

#[test]
fn test_lower_orient() {
    let surface = SurfaceWorkflow::Orient {
        expr: SurfaceExpr::Literal(SurfaceLiteral::Int(42)),
        binding: None,
        continuation: None,
        span: dummy_span(),
    };
    let core = lower_workflow_body(&surface, &Provenance::new(), &LoweringContext::new()).unwrap();
    assert!(matches!(core, CoreWorkflow::Orient { .. }));
}

fn module_identity_for_type_lowering_tests() -> ash_core::semantic_summary::ModuleIdentity {
    ash_core::semantic_summary::ModuleIdentity::new(
        Some(ash_core::module_graph::CrateId(1)),
        ash_core::module_graph::ModuleId(7),
        vec!["crate".into(), "domain".into()],
        ash_core::semantic_summary::ModuleSourceOrigin::File("/repo/domain.ash".into()),
    )
}

fn parse_module_for_type_lowering(source: &str) -> crate::surface::ModuleFile {
    crate::parse_surface_file_with_path(source, Some(std::path::Path::new("/repo/domain.ash")))
        .expect("module with ordinary type definitions should parse")
}

#[test]
fn task784_lowers_alias_type_to_core_and_summary_with_source_anchor() {
    let module = parse_module_for_type_lowering("pub type UserId = String;");
    let module_identity = module_identity_for_type_lowering_tests();

    let lowered = lower_module_type_metadata(&module, module_identity.clone());

    assert_eq!(lowered.type_defs.len(), 1);
    assert_eq!(lowered.summary.exported_types.len(), 1);
    let core = &lowered.type_defs[0];
    assert_eq!(core.name, "UserId");
    assert_eq!(core.visibility, ash_core::ast::Visibility::Public);
    assert!(!core.builtin);
    assert_eq!(
        core.body,
        ash_core::ast::TypeBody::Alias(ash_core::ast::TypeExpr::Named("String".into()))
    );
    let summary = &lowered.summary.exported_types[0];
    assert_eq!(summary.id.module, module_identity);
    assert_eq!(summary.id.name, "UserId");
    assert_eq!(summary.exported_name, "UserId");
    assert_eq!(
        summary.source_anchor.span,
        Some(ash_core::ast::Span { start: 0, end: 25 })
    );
    assert_eq!(
        summary.source_anchor.origin,
        ash_core::semantic_summary::SourceOrigin::File("/repo/domain.ash".into())
    );
}

#[test]
fn task784_lowers_struct_type_preserving_fields_and_generic_params() {
    let module = parse_module_for_type_lowering("pub type Box<T> = { value: T };");
    let lowered = lower_module_type_metadata(&module, module_identity_for_type_lowering_tests());

    assert_eq!(lowered.type_defs[0].params, vec!["T"]);
    assert_eq!(
        lowered.type_defs[0].body,
        ash_core::ast::TypeBody::Struct(vec![(
            "value".into(),
            ash_core::ast::TypeExpr::Named("T".into())
        )])
    );
    assert_eq!(lowered.summary.exported_types[0].params, vec!["T"]);
    assert!(matches!(
        lowered.summary.exported_types[0].representation,
        ash_core::semantic_summary::TypeRepresentationSummary::Exposed(_)
    ));
}

#[test]
fn task784_lowers_enum_variants_and_constructor_summaries_with_payload_kinds() {
    let module = parse_module_for_type_lowering(
        "pub type Result<T> = Ok(T) | Err { message: String } | Pending;",
    );
    let lowered = lower_module_type_metadata(&module, module_identity_for_type_lowering_tests());

    assert_eq!(lowered.summary.exported_constructors.len(), 3);
    assert_eq!(lowered.summary.exported_constructors[0].exported_name, "Ok");
    assert_eq!(
        lowered.summary.exported_constructors[0].payload_kind,
        ash_core::semantic_summary::ConstructorPayloadKind::Tuple
    );
    assert_eq!(
        lowered.summary.exported_constructors[1].exported_name,
        "Err"
    );
    assert_eq!(
        lowered.summary.exported_constructors[1].payload_kind,
        ash_core::semantic_summary::ConstructorPayloadKind::Record
    );
    assert_eq!(
        lowered.summary.exported_constructors[2].exported_name,
        "Pending"
    );
    assert_eq!(
        lowered.summary.exported_constructors[2].payload_kind,
        ash_core::semantic_summary::ConstructorPayloadKind::Unit
    );
    assert!(matches!(
        &lowered.type_defs[0].body,
        ash_core::ast::TypeBody::Enum(variants) if variants.len() == 3
    ));
}

#[test]
fn task784_lowers_builtin_opaque_type_as_core_builtin_and_opaque_summary() {
    let module = parse_module_for_type_lowering("pub builtin type NativeHandle;");
    let lowered = lower_module_type_metadata(&module, module_identity_for_type_lowering_tests());

    assert!(lowered.type_defs[0].builtin);
    assert_eq!(
        lowered.summary.exported_types[0].representation_exposure,
        ash_core::semantic_summary::RepresentationExposure::Opaque
    );
    assert_eq!(
        lowered.summary.exported_types[0].representation,
        ash_core::semantic_summary::TypeRepresentationSummary::Opaque { builtin: true }
    );
    assert_eq!(lowered.summary.diagnostic_anchors.len(), 2);
}

#[test]
fn test_lower_effect_type() {
    assert!(matches!(
        lower_effect_type(EffectType::Observe),
        Effect::Epistemic
    ));
    assert!(matches!(
        lower_effect_type(EffectType::Read),
        Effect::Epistemic
    ));
    assert!(matches!(
        lower_effect_type(EffectType::Analyze),
        Effect::Deliberative
    ));
    assert!(matches!(
        lower_effect_type(EffectType::Decide),
        Effect::Evaluative
    ));
    assert!(matches!(
        lower_effect_type(EffectType::Act),
        Effect::Operational
    ));
    assert!(matches!(
        lower_effect_type(EffectType::Write),
        Effect::Operational
    ));
    assert!(matches!(
        lower_effect_type(EffectType::External),
        Effect::Operational
    ));
}

// =========================================================================
// Module-Owned Capability Resolution Tests (TASK-475)
// =========================================================================

#[test]
fn test_lower_act_with_explicit_target_bypasses_resolution() {
    // Explicit provider:action calls should bypass capability resolution
    let surface = SurfaceWorkflow::Act {
        action: crate::surface::ActionRef {
            target: crate::surface::OperationalTarget::Explicit {
                provider: "io".into(),
                action: "fs_read".into(),
            },
            args: vec![],
        },
        guard: None,
        result_name: None,
        continuation: None,
        span: dummy_span(),
    };

    // Should work even without capability context
    let ctx = LoweringContext::new();
    let core = lower_workflow_body(&surface, &Provenance::new(), &ctx).unwrap();

    match core {
        CoreWorkflow::Act {
            provider_name,
            action_name,
            ..
        } => {
            assert_eq!(provider_name, "io");
            assert_eq!(action_name, "fs_read");
        }
        _ => panic!("expected Act workflow, got {:?}", core),
    }
}

#[test]
fn test_lower_act_with_unmarked_symbolic_target_lowers_as_function_call() {
    // Phase 158: symbolic act syntax is also used for user-defined functions.
    // Names that are not known builtins or effectful declarations lower as pure
    // function calls rather than unresolved capabilities.
    let surface = SurfaceWorkflow::Act {
        action: crate::surface::ActionRef {
            target: crate::surface::OperationalTarget::Symbolic {
                capability_name: "fs_read".into(),
            },
            args: vec![],
        },
        guard: None,
        result_name: None,
        continuation: None,
        span: dummy_span(),
    };

    let ctx = LoweringContext::new();
    let core = lower_workflow_body(&surface, &Provenance::new(), &ctx).unwrap();
    match core {
        CoreWorkflow::Orient { expr, .. } => match expr {
            CoreExpr::FnApply { func, args } => {
                assert!(matches!(*func, CoreExpr::Variable { name, .. } if name == "fs_read"));
                assert!(args.is_empty());
            }
            other => panic!("expected FnApply expression, got {other:?}"),
        },
        other => panic!("expected Orient workflow, got {other:?}"),
    }
}

#[test]
fn test_lower_act_with_effectful_symbolic_target_requires_context() {
    // Known effectful symbolic capability calls still require resolution context.
    let surface = SurfaceWorkflow::Act {
        action: crate::surface::ActionRef {
            target: crate::surface::OperationalTarget::Symbolic {
                capability_name: "fs_read".into(),
            },
            args: vec![],
        },
        guard: None,
        result_name: None,
        continuation: None,
        span: dummy_span(),
    };

    let ctx = LoweringContext::new();
    let effectful_names = HashSet::from([String::from("fs_read")]);
    let result = with_active_effectful_names(&effectful_names, || {
        lower_workflow_body(&surface, &Provenance::new(), &ctx)
    });
    assert!(
        matches!(result, Err(LoweringError::UnresolvedCapability { name }) if name == "fs_read")
    );
}

#[test]
fn decide_else_branch_lowering_error_uses_removed_form_vocabulary() {
    let surface = SurfaceWorkflow::Decide {
        expr: int_expr(1),
        policy: Some("allow".into()),
        then_branch: Box::new(SurfaceWorkflow::Done { span: dummy_span() }),
        else_branch: Some(Box::new(SurfaceWorkflow::Done { span: dummy_span() })),
        span: dummy_span(),
    };

    let err = lower_workflow_body(&surface, &Provenance::new(), &LoweringContext::new())
        .expect_err("decide else branches are removed from canonical lowering");
    let LoweringError::InvalidTarget(message) = err else {
        panic!("expected invalid target error");
    };
    assert!(
        message.contains("removed decide else-branches"),
        "diagnostic should identify the removed form: {message}"
    );
}

#[test]
fn test_lower_act_with_capability_context_resolves_symbolic() {
    // Symbolic capability calls resolve when context has the mapping
    use crate::capability_export::{
        CapabilityEffect, CapabilityExport, CapabilityResolutionContext,
    };
    use ash_core::module_graph::ModuleId;

    let surface = SurfaceWorkflow::Act {
        action: crate::surface::ActionRef {
            target: crate::surface::OperationalTarget::Symbolic {
                capability_name: "fs_read".into(),
            },
            args: vec![],
        },
        guard: None,
        result_name: None,
        continuation: None,
        span: dummy_span(),
    };

    // Build a capability resolution context with the mapping
    let mut cap_context = CapabilityResolutionContext::new();
    let export = CapabilityExport {
        visible_name: "fs_read".into(),
        declaring_module: ModuleId(0),
        target_provider: "io".into(),
        target_action: "fs_read".into(),
        visibility: crate::surface::Visibility::Public,
        effect: CapabilityEffect::Act,
    };
    cap_context.register(&export);

    let ctx = LoweringContext::with_capability_context_for_module(cap_context, ModuleId(0));
    let effectful_names = HashSet::from([String::from("fs_read")]);
    let core = with_active_effectful_names(&effectful_names, || {
        lower_workflow_body(&surface, &Provenance::new(), &ctx)
    })
    .unwrap();

    match core {
        CoreWorkflow::Act {
            provider_name,
            action_name,
            ..
        } => {
            assert_eq!(provider_name, "io");
            assert_eq!(action_name, "fs_read");
        }
        _ => panic!("expected Act workflow, got {:?}", core),
    }
}

// --- BuiltinFnDef lowering tests ---

#[test]
fn test_lower_builtin_fn_simple() {
    use crate::surface::{BuiltinFnDef, Param, Type, Visibility};

    let def = BuiltinFnDef {
        visibility: Visibility::Inherited,
        name: "foo".into(),
        type_params: vec![],
        params: vec![Param {
            name: "x".into(),
            ty: Type::Name("Int".into()),
        }],
        return_type: Type::Name("Int".into()),
        proposition_tail: None,
        span: dummy_span(),
    };

    let core = lower_builtin_fn_def(&def).expect("builtin fn should lower");

    assert_eq!(core.name, "foo");
    assert!(core.type_params.is_empty());
    assert_eq!(core.params.len(), 1);
    assert_eq!(core.params[0].0, "x");
    assert_eq!(
        core.params[0].1,
        ash_core::ast::TypeExpr::Named("Int".to_string())
    );
    assert_eq!(
        core.return_type,
        ash_core::ast::TypeExpr::Named("Int".to_string())
    );
    assert_eq!(core.visibility, ash_core::ast::Visibility::Private);
}

#[test]
fn test_lower_builtin_fn_with_type_params() {
    use crate::surface::{BuiltinFnDef, Param, Type, Visibility};

    let def = BuiltinFnDef {
        visibility: Visibility::Public,
        name: "id".into(),
        type_params: vec!["T".into()],
        params: vec![Param {
            name: "value".into(),
            ty: Type::Name("T".into()),
        }],
        return_type: Type::Name("T".into()),
        proposition_tail: None,
        span: dummy_span(),
    };

    let core = lower_builtin_fn_def(&def).expect("builtin fn should lower");

    assert_eq!(core.name, "id");
    assert_eq!(core.type_params, vec!["T".to_string()]);
    assert_eq!(core.params.len(), 1);
    assert_eq!(core.params[0].0, "value");
    assert_eq!(core.visibility, ash_core::ast::Visibility::Public);
}

#[test]
fn test_lower_builtin_fn_rejects_kinded_type_params() {
    use crate::surface::{BuiltinFnDef, KindAnnotation, Param, Type, TypeParam, Visibility};

    let def = BuiltinFnDef {
        visibility: Visibility::Public,
        name: "pure".into(),
        type_params: vec![TypeParam {
            name: "M".into(),
            kind: Some(KindAnnotation {
                kind: ash_core::Kind::arrow(ash_core::Kind::Type, ash_core::Kind::Type),
                span: dummy_span(),
            }),
            bounds: Vec::new(),
            span: dummy_span(),
        }],
        params: vec![Param {
            name: "value".into(),
            ty: Type::Name("Int".into()),
        }],
        return_type: Type::Constructor {
            name: "M".into(),
            args: vec![Type::Name("Int".into())],
        },
        proposition_tail: None,
        span: dummy_span(),
    };

    let err = lower_builtin_fn_def(&def).expect_err("kinded builtin fn should not lower yet");

    assert_eq!(
        err,
        LoweringError::UnsupportedFeature(
            "kinded builtin function type parameters are parsed by TASK-906 but lowered by TASK-907"
                .to_string()
        )
    );
}

#[test]
fn test_lower_builtin_fn_multi_param() {
    use crate::surface::{BuiltinFnDef, Param, Type, Visibility};

    let def = BuiltinFnDef {
        visibility: Visibility::Crate,
        name: "add".into(),
        type_params: vec![],
        params: vec![
            Param {
                name: "a".into(),
                ty: Type::Name("Int".into()),
            },
            Param {
                name: "b".into(),
                ty: Type::Name("Int".into()),
            },
        ],
        return_type: Type::Name("Int".into()),
        proposition_tail: None,
        span: dummy_span(),
    };

    let core = lower_builtin_fn_def(&def).expect("builtin fn should lower");

    assert_eq!(core.name, "add");
    assert_eq!(core.params.len(), 2);
    assert_eq!(core.params[0].0, "a");
    assert_eq!(core.params[1].0, "b");
    assert_eq!(core.visibility, ash_core::ast::Visibility::Crate);
}

#[test]
fn test_lower_builtin_fn_complex_return_type() {
    use crate::surface::{BuiltinFnDef, Param, Type, Visibility};

    let def = BuiltinFnDef {
        visibility: Visibility::Inherited,
        name: "make_list".into(),
        type_params: vec!["T".into()],
        params: vec![Param {
            name: "x".into(),
            ty: Type::Name("T".into()),
        }],
        return_type: Type::Constructor {
            name: "List".into(),
            args: vec![Type::Name("T".into())],
        },
        proposition_tail: None,
        span: dummy_span(),
    };

    let core = lower_builtin_fn_def(&def).expect("builtin fn should lower");

    assert_eq!(core.name, "make_list");
    assert_eq!(
        core.return_type,
        ash_core::ast::TypeExpr::Constructor {
            name: "List".to_string(),
            args: vec![ash_core::ast::TypeExpr::Named("T".to_string())],
        }
    );
}

#[test]
fn test_lower_builtin_fn_no_params() {
    // Zero-parameter builtin fn (e.g., builtin fn get_time() -> Int;)
    use crate::surface::{BuiltinFnDef, Type, Visibility};

    let def = BuiltinFnDef {
        visibility: Visibility::Public,
        name: "get_time".into(),
        type_params: vec![],
        params: vec![],
        return_type: Type::Name("Int".into()),
        proposition_tail: None,
        span: dummy_span(),
    };

    let core = lower_builtin_fn_def(&def).expect("builtin fn should lower");

    assert_eq!(core.name, "get_time");
    assert!(core.params.is_empty());
    assert_eq!(core.visibility, ash_core::ast::Visibility::Public);
}

#[test]
fn test_lower_builtin_fn_parse_and_lower_roundtrip() {
    // Parse a builtin fn from source and lower it
    let source = "builtin fn foo(x: Int) -> Int;";
    let parsed = crate::parse_surface_file(source).expect("parse should succeed");

    // Find the BuiltinFn definition
    let builtin_def = parsed
        .definitions
        .iter()
        .find_map(|d| match d {
            crate::surface::Definition::BuiltinFn(b) => Some(b.clone()),
            _ => None,
        })
        .expect("should find a BuiltinFn definition");

    assert_eq!(builtin_def.name.as_ref(), "foo");

    let core = lower_builtin_fn_def(&builtin_def).expect("builtin fn should lower");

    assert_eq!(core.name, "foo");
    assert!(core.type_params.is_empty());
    assert_eq!(core.params.len(), 1);
    assert_eq!(core.params[0].0, "x");
    assert_eq!(
        core.params[0].1,
        ash_core::ast::TypeExpr::Named("Int".to_string())
    );
    assert_eq!(
        core.return_type,
        ash_core::ast::TypeExpr::Named("Int".to_string())
    );
}

#[test]
fn test_lower_builtin_fn_parse_generic_roundtrip() {
    // Parse a generic builtin fn from source and lower it
    let source = "pub builtin fn map<T>(f: T, x: Int) -> T;";
    let parsed = crate::parse_surface_file(source).expect("parse should succeed");

    let builtin_def = parsed
        .definitions
        .iter()
        .find_map(|d| match d {
            crate::surface::Definition::BuiltinFn(b) => Some(b.clone()),
            _ => None,
        })
        .expect("should find a BuiltinFn definition");

    assert_eq!(builtin_def.name.as_ref(), "map");

    let core = lower_builtin_fn_def(&builtin_def).expect("builtin fn should lower");

    assert_eq!(core.name, "map");
    assert_eq!(core.type_params, vec!["T".to_string()]);
    assert_eq!(core.params.len(), 2);
    assert_eq!(core.params[0].0, "f");
    assert_eq!(core.params[1].0, "x");
    assert_eq!(core.visibility, ash_core::ast::Visibility::Public);
}
