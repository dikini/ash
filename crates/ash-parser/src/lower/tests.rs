//! Lowering tests.

use super::*;
use crate::surface::{
    ActStmt, BinaryOp, Contract as SurfaceContract, DoStmt, DoTarget, EffectType, EnsuresClause,
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
                    left: Box::new(var_expr("n")),
                    right: Box::new(int_expr(0)),
                    span: dummy_span(),
                },
            },
            SurfaceRequirement::Arithmetic {
                expr: SurfaceExpr::Binary {
                    op: BinaryOp::Neq,
                    left: Box::new(var_expr("d")),
                    right: Box::new(int_expr(0)),
                    span: dummy_span(),
                },
            },
            SurfaceRequirement::Arithmetic {
                expr: SurfaceExpr::Binary {
                    op: BinaryOp::Eq,
                    left: Box::new(SurfaceExpr::Binary {
                        op: BinaryOp::Mod,
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
                left: Box::new(var_expr("result")),
                right: Box::new(int_expr(0)),
                span: dummy_span(),
            },
            span: dummy_span(),
        }],
    };

    let lowered = lower_fn_contract(Some(&contract)).expect("fn contract should lower");
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
                left: Box::new(var_expr("state")),
                right: Box::new(int_expr(0)),
                span: dummy_span(),
            },
            span: dummy_span(),
        }],
    };

    let error = lower_fn_contract(Some(&contract)).expect_err("invalid ensures should fail");
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

// ── Act block lowering tests (TASK-675) ──────────────────────────

#[test]
fn test_lower_act_block_empty_is_rejected() {
    let surface = SurfaceExpr::ActBlock {
        stmts: vec![],
        span: Span::default(),
    };
    let result = lower_expr(&surface);
    assert!(
        result.is_err(),
        "empty act block must be rejected per SPEC-047 §6.2"
    );
}

#[test]
fn test_lower_act_block_return_not_last_is_rejected() {
    let surface = SurfaceExpr::ActBlock {
        stmts: vec![
            ActStmt::Return {
                value: Box::new(SurfaceExpr::Literal(SurfaceLiteral::Int(1))),
                span: Span::default(),
            },
            ActStmt::Bind {
                name: "x".into(),
                value: Box::new(SurfaceExpr::Literal(SurfaceLiteral::Int(2))),
                span: Span::default(),
            },
        ],
        span: Span::default(),
    };
    let result = lower_expr(&surface);
    assert!(
        result.is_err(),
        "return followed by more statements must be rejected per SPEC-047 §6.2"
    );
}

#[test]
fn test_lower_act_block_return() {
    let surface = SurfaceExpr::ActBlock {
        stmts: vec![ActStmt::Return {
            value: Box::new(SurfaceExpr::Literal(SurfaceLiteral::Int(42))),
            span: Span::default(),
        }],
        span: Span::default(),
    };
    let core = lower_expr(&surface).expect("lowering act block return should succeed");
    match core {
        CoreExpr::Call {
            func, arguments, ..
        } => {
            assert_eq!(func, "unit");
            assert_eq!(arguments.len(), 1);
            assert!(matches!(
                &arguments[0],
                CoreExpr::Literal(ash_core::Value::Int(42))
            ));
        }
        _ => panic!("Expected Call, got: {:?}", core),
    }
}

#[test]
fn test_lower_act_block_bind_then_return() {
    let surface = SurfaceExpr::ActBlock {
        stmts: vec![
            ActStmt::Bind {
                name: "x".into(),
                value: Box::new(SurfaceExpr::Literal(SurfaceLiteral::Int(1))),
                span: Span::default(),
            },
            ActStmt::Return {
                value: Box::new(SurfaceExpr::Variable {
                    name: "x".into(),
                    span: Span::default(),
                }),
                span: Span::default(),
            },
        ],
        span: Span::default(),
    };
    let core = lower_expr(&surface).expect("lowering bind+return should succeed");
    // Should be: bind(1, FnDef { params: [("x", None)], body: unit(Variable "x") })
    match core {
        CoreExpr::Call {
            func, arguments, ..
        } => {
            assert_eq!(func, "bind");
            assert_eq!(arguments.len(), 2);
            // First argument: pure value is lifted with unit(1)
            assert!(matches!(
                &arguments[0],
                CoreExpr::Call { func, arguments, .. }
                    if func == "unit"
                        && arguments.len() == 1
                        && matches!(&arguments[0], CoreExpr::Literal(ash_core::Value::Int(1)))
            ));
            // Second argument: FnDef with param "x" and body unit(Variable "x")
            match &arguments[1] {
                CoreExpr::FnDef { params, body, .. } => {
                    assert_eq!(params.len(), 1);
                    assert_eq!(params[0].0, "x");
                    assert!(params[0].1.is_none());
                    // body should be unit(Variable "x")
                    match body.as_ref() {
                        CoreExpr::Call {
                            func: body_func,
                            arguments: body_args,
                            ..
                        } => {
                            assert_eq!(body_func, "unit");
                            assert_eq!(body_args.len(), 1);
                            assert!(matches!(
                                &body_args[0],
                                CoreExpr::Variable { name, .. } if name == "x"
                            ));
                        }
                        _ => panic!("Expected unit call in body, got: {:?}", body),
                    }
                }
                _ => panic!("Expected FnDef as second argument, got: {:?}", arguments[1]),
            }
        }
        _ => panic!("Expected Call (bind), got: {:?}", core),
    }
}

#[test]
fn test_lower_act_block_effectful_bind_value_not_double_wrapped() {
    let surface = SurfaceExpr::ActBlock {
        stmts: vec![
            ActStmt::Bind {
                name: "x".into(),
                value: Box::new(SurfaceExpr::Call {
                    func: "invoke".into(),
                    module: None,
                    args: vec![
                        SurfaceExpr::Literal(SurfaceLiteral::String("Fs".into())),
                        SurfaceExpr::Literal(SurfaceLiteral::String("read".into())),
                        SurfaceExpr::Literal(SurfaceLiteral::List(vec![])),
                    ],
                    span: Span::default(),
                }),
                span: Span::default(),
            },
            ActStmt::Return {
                value: Box::new(SurfaceExpr::Variable {
                    name: "x".into(),
                    span: Span::default(),
                }),
                span: Span::default(),
            },
        ],
        span: Span::default(),
    };

    let core = lower_expr(&surface).expect("lowering invoke bind should succeed");
    match core {
        CoreExpr::Call {
            func, arguments, ..
        } => {
            assert_eq!(func, "bind");
            assert_eq!(arguments.len(), 2);
            assert!(
                !matches!(&arguments[0], CoreExpr::Call { func, .. } if func == "unit"),
                "effectful-looking bind RHS should not be wrapped with unit()"
            );
        }
        _ => panic!("Expected outer bind call, got: {:?}", core),
    }
}

#[test]
fn test_lower_act_block_effectful_user_defined_bind_value_not_wrapped() {
    let effectful_names = HashSet::from([String::from("read")]);
    let surface = SurfaceExpr::ActBlock {
        stmts: vec![
            ActStmt::Bind {
                name: "x".into(),
                value: Box::new(SurfaceExpr::Call {
                    func: "read".into(),
                    module: None,
                    args: vec![SurfaceExpr::Literal(SurfaceLiteral::String(
                        "/tmp/file".into(),
                    ))],
                    span: Span::default(),
                }),
                span: Span::default(),
            },
            ActStmt::Return {
                value: Box::new(SurfaceExpr::Variable {
                    name: "x".into(),
                    span: Span::default(),
                }),
                span: Span::default(),
            },
        ],
        span: Span::default(),
    };

    let core = with_active_effectful_names(&effectful_names, || lower_expr(&surface))
        .expect("lowering effectful bind should succeed");

    match core {
        CoreExpr::Call {
            func, arguments, ..
        } => {
            assert_eq!(func, "bind");
            assert_eq!(arguments.len(), 2);
            assert!(
                !matches!(&arguments[0], CoreExpr::Call { func, .. } if func == "unit"),
                "user-defined effectful calls must not be wrapped in unit()"
            );
        }
        _ => panic!("Expected outer bind call, got: {:?}", core),
    }
}

#[test]
fn test_lower_act_block_multiple_binds() {
    let surface = SurfaceExpr::ActBlock {
        stmts: vec![
            ActStmt::Bind {
                name: "x".into(),
                value: Box::new(SurfaceExpr::Literal(SurfaceLiteral::Int(1))),
                span: Span::default(),
            },
            ActStmt::Bind {
                name: "y".into(),
                value: Box::new(SurfaceExpr::Literal(SurfaceLiteral::Int(2))),
                span: Span::default(),
            },
            ActStmt::Return {
                value: Box::new(SurfaceExpr::Variable {
                    name: "y".into(),
                    span: Span::default(),
                }),
                span: Span::default(),
            },
        ],
        span: Span::default(),
    };
    let core = lower_expr(&surface).expect("lowering multi-bind should succeed");
    // Should be: bind(unit(1), FnDef { params: ["x"], body: bind(unit(2), FnDef { params: ["y"], body: unit(Variable "y") }) })
    match core {
        CoreExpr::Call {
            func: outer_func,
            arguments: outer_args,
            ..
        } => {
            assert_eq!(outer_func, "bind");
            assert_eq!(outer_args.len(), 2);
            // First arg: unit(1)
            assert!(matches!(
                &outer_args[0],
                CoreExpr::Call { func, arguments, .. }
                    if func == "unit"
                        && arguments.len() == 1
                        && matches!(&arguments[0], CoreExpr::Literal(ash_core::Value::Int(1)))
            ));
            // Second arg: FnDef with param "x", body is another bind
            match &outer_args[1] {
                CoreExpr::FnDef {
                    params: outer_params,
                    body: outer_body,
                    ..
                } => {
                    assert_eq!(outer_params.len(), 1);
                    assert_eq!(outer_params[0].0, "x");
                    // body should be bind(2, FnDef { params: ["y"], body: unit(...) })
                    match outer_body.as_ref() {
                        CoreExpr::Call {
                            func: inner_func,
                            arguments: inner_args,
                            ..
                        } => {
                            assert_eq!(inner_func, "bind");
                            assert_eq!(inner_args.len(), 2);
                            assert!(matches!(
                                &inner_args[0],
                                CoreExpr::Call { func, arguments, .. }
                                    if func == "unit"
                                        && arguments.len() == 1
                                        && matches!(&arguments[0], CoreExpr::Literal(ash_core::Value::Int(2)))
                            ));
                            match &inner_args[1] {
                                CoreExpr::FnDef {
                                    params: inner_params,
                                    ..
                                } => {
                                    assert_eq!(inner_params.len(), 1);
                                    assert_eq!(inner_params[0].0, "y");
                                }
                                _ => panic!("Expected inner FnDef"),
                            }
                        }
                        _ => panic!("Expected inner bind call, got: {:?}", outer_body),
                    }
                }
                _ => panic!("Expected outer FnDef"),
            }
        }
        _ => panic!("Expected outer bind call, got: {:?}", core),
    }
}

// ── Act block round-trip and integration tests (TASK-676) ────────

/// Round-trip: parse source with an act block fn body, then lower.
#[test]
fn test_parse_and_lower_act_block_roundtrip() {
    let src = "fn f() { act { x = 1; ret x; } }";
    let parsed = crate::parse_surface_file(src).expect("parse should succeed");

    // Find the Function definition
    let fn_def = parsed
        .definitions
        .iter()
        .find_map(|d| match d {
            crate::surface::Definition::Function(f) => Some(f.clone()),
            _ => None,
        })
        .expect("should find a Function definition");

    assert_eq!(fn_def.name.as_ref(), "f");

    // The body is wrapped in a Block by the fn parser — extract the ActBlock
    let act_block = match &fn_def.body {
        SurfaceExpr::ActBlock { .. } => &fn_def.body,
        SurfaceExpr::Block {
            tail_expr: Some(tail),
            ..
        } => match tail.as_ref() {
            act @ SurfaceExpr::ActBlock { .. } => act,
            other => panic!("Expected ActBlock in block tail, got: {:?}", other),
        },
        other => panic!(
            "Expected ActBlock or Block wrapping ActBlock, got: {:?}",
            other
        ),
    };

    match act_block {
        SurfaceExpr::ActBlock { stmts, .. } => {
            assert_eq!(stmts.len(), 2);
            assert!(matches!(&stmts[0], ActStmt::Bind { name, .. } if name.as_ref() == "x"));
            assert!(matches!(&stmts[1], ActStmt::Return { .. }));
        }
        _ => unreachable!(),
    }

    // Lower the act block expression
    let core = lower_expr(act_block).expect("lowering act block should succeed");

    // Should be: bind(unit(1), FnDef { params: [("x", None)], body: unit(Variable "x") })
    match core {
        CoreExpr::Call {
            func, arguments, ..
        } => {
            assert_eq!(func, "bind");
            assert_eq!(arguments.len(), 2);
            // First argument: unit(1)
            assert!(matches!(
                &arguments[0],
                CoreExpr::Call { func, arguments, .. }
                    if func == "unit"
                        && arguments.len() == 1
                        && matches!(&arguments[0], CoreExpr::Literal(ash_core::Value::Int(1)))
            ));
            // Second argument: FnDef with param "x"
            match &arguments[1] {
                CoreExpr::FnDef { params, body, .. } => {
                    assert_eq!(params.len(), 1);
                    assert_eq!(params[0].0, "x");
                    // body should be unit(Variable "x")
                    match body.as_ref() {
                        CoreExpr::Call {
                            func: body_func,
                            arguments: body_args,
                            ..
                        } => {
                            assert_eq!(body_func, "unit");
                            assert_eq!(body_args.len(), 1);
                            assert!(matches!(
                                &body_args[0],
                                CoreExpr::Variable { name, .. } if name == "x"
                            ));
                        }
                        _ => panic!("Expected unit call in body, got: {:?}", body),
                    }
                }
                _ => panic!("Expected FnDef, got: {:?}", arguments[1]),
            }
        }
        _ => panic!("Expected bind call, got: {:?}", core),
    }
}

/// Nested act blocks: act { x = act { ret 1; }; ret x; }
/// The inner act block lowers to unit(1), the outer to bind(unit(1), |x| unit(x))
#[test]
fn test_lower_nested_act_blocks() {
    let inner_act = SurfaceExpr::ActBlock {
        stmts: vec![ActStmt::Return {
            value: Box::new(SurfaceExpr::Literal(SurfaceLiteral::Int(1))),
            span: Span::default(),
        }],
        span: Span::default(),
    };

    let surface = SurfaceExpr::ActBlock {
        stmts: vec![
            ActStmt::Bind {
                name: "x".into(),
                value: Box::new(inner_act),
                span: Span::default(),
            },
            ActStmt::Return {
                value: Box::new(SurfaceExpr::Variable {
                    name: "x".into(),
                    span: Span::default(),
                }),
                span: Span::default(),
            },
        ],
        span: Span::default(),
    };

    let core = lower_expr(&surface).expect("lowering nested act blocks should succeed");

    // Outer should be: bind(<inner>, FnDef { params: ["x"], body: unit(Variable "x") })
    match core {
        CoreExpr::Call {
            func, arguments, ..
        } => {
            assert_eq!(func, "bind");
            assert_eq!(arguments.len(), 2);

            // First argument: the inner act block lowered to unit(1)
            match &arguments[0] {
                CoreExpr::Call {
                    func: inner_func,
                    arguments: inner_args,
                    ..
                } => {
                    assert_eq!(inner_func, "unit");
                    assert_eq!(inner_args.len(), 1);
                    assert!(matches!(
                        &inner_args[0],
                        CoreExpr::Literal(ash_core::Value::Int(1))
                    ));
                }
                _ => panic!("Expected inner unit call, got: {:?}", arguments[0]),
            }

            // Second argument: FnDef with param "x" and body unit(Variable "x")
            match &arguments[1] {
                CoreExpr::FnDef { params, body, .. } => {
                    assert_eq!(params.len(), 1);
                    assert_eq!(params[0].0, "x");
                    match body.as_ref() {
                        CoreExpr::Call {
                            func: body_func,
                            arguments: body_args,
                            ..
                        } => {
                            assert_eq!(body_func, "unit");
                            assert_eq!(body_args.len(), 1);
                            assert!(matches!(
                                &body_args[0],
                                CoreExpr::Variable { name, .. } if name == "x"
                            ));
                        }
                        _ => panic!("Expected unit call in body, got: {:?}", body),
                    }
                }
                _ => panic!("Expected FnDef, got: {:?}", arguments[1]),
            }
        }
        _ => panic!("Expected bind call, got: {:?}", core),
    }
}

// ── Property test: act block lowering (TASK-676) ─────────────────

/// Recursively verify that a CoreExpr contains only Call, FnDef, Literal, Variable.
/// No ActBlock should survive lowering.
#[allow(dead_code)]
fn assert_act_block_uses_only_call_fndef(expr: &CoreExpr) {
    match expr {
        CoreExpr::Literal(_) | CoreExpr::Variable { .. } => {}
        CoreExpr::Call { arguments, .. } => {
            for arg in arguments {
                assert_act_block_uses_only_call_fndef(arg);
            }
        }
        CoreExpr::FnDef { body, .. } => {
            assert_act_block_uses_only_call_fndef(body);
        }
        // These are valid core IR nodes that may appear in general lowered
        // expressions but not from simple act-block lowering with literal/variable
        // sub-expressions. We still allow them for completeness since the
        // strategy could generate them indirectly.
        CoreExpr::FieldAccess { expr, .. } => {
            assert_act_block_uses_only_call_fndef(expr);
        }
        CoreExpr::IndexAccess { expr, index } => {
            assert_act_block_uses_only_call_fndef(expr);
            assert_act_block_uses_only_call_fndef(index);
        }
        CoreExpr::Unary { expr: inner, .. } => {
            assert_act_block_uses_only_call_fndef(inner);
        }
        CoreExpr::Binary { left, right, .. } => {
            assert_act_block_uses_only_call_fndef(left);
            assert_act_block_uses_only_call_fndef(right);
        }
        CoreExpr::Constructor { fields, .. } => {
            for (_, e) in fields {
                assert_act_block_uses_only_call_fndef(e);
            }
        }
        CoreExpr::Match { scrutinee, arms } => {
            assert_act_block_uses_only_call_fndef(scrutinee);
            for arm in arms {
                assert_act_block_uses_only_call_fndef(&arm.body);
            }
        }
        CoreExpr::IfLet {
            expr: inner,
            then_branch,
            else_branch,
            ..
        } => {
            assert_act_block_uses_only_call_fndef(inner);
            assert_act_block_uses_only_call_fndef(then_branch);
            assert_act_block_uses_only_call_fndef(else_branch);
        }
        CoreExpr::Spawn { init, .. } => {
            assert_act_block_uses_only_call_fndef(init);
        }
        CoreExpr::Split(inner) => {
            assert_act_block_uses_only_call_fndef(inner);
        }
        CoreExpr::CheckObligation { .. } => {}
        CoreExpr::Fail { payload } => {
            assert_act_block_uses_only_call_fndef(payload);
        }
        CoreExpr::WithError { body, arms } => {
            assert_act_block_uses_only_call_fndef(body);
            for arm in arms {
                assert_act_block_uses_only_call_fndef(&arm.body);
            }
        }
        CoreExpr::Let {
            expr: inner, body, ..
        } => {
            assert_act_block_uses_only_call_fndef(inner);
            assert_act_block_uses_only_call_fndef(body);
        }
        CoreExpr::FnApply { func, args } => {
            assert_act_block_uses_only_call_fndef(func);
            for a in args {
                assert_act_block_uses_only_call_fndef(a);
            }
        }
    }
}
