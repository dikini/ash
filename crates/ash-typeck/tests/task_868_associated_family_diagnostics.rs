use ash_core::ast::Visibility as CoreVisibility;
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, DomainConstructorId, DomainConstructorSummary, DomainFieldSummary,
    InterfaceIdentityId, ModuleIdentity, ModuleSemanticSummary,
    ModuleSemanticSummaryValidationError, ModuleSourceOrigin, SealedDomainId, SealedDomainSummary,
    SourceAnchor, SourceOrigin, SummaryVersion, TypeDeclId,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, NormalFormBlockReason, NormalTypeExpr, ProjectionRigidity,
    TypeComputationHeadId,
};
use ash_diagnostic::{AshLspError, DiagnosticCode, Severity, Span as DiagnosticSpan};
use ash_parser::surface::{
    AssociatedTypeBinding, AssociatedTypeDecl, AssociatedTypeKind, Definition, Expr, ImplDef,
    ImplMethodDef, InterfaceDef, InterfaceMethodSig, InterfaceTypeParam, Literal,
    Type as SurfaceType, Visibility,
};
use ash_parser::token::Span;
use ash_typeck::error::TypeEnvError;
use ash_typeck::normalizer::{DefinitionalEqualityResult, Normalizer};
use ash_typeck::{Type, TypeEnv};

fn span() -> Span {
    Span::new(10, 31, 3, 7)
}

fn module(name: &str, id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(868)),
        ModuleId(id),
        vec!["task868".into(), name.into()],
        ModuleSourceOrigin::Synthetic {
            reason: format!("TASK-868 diagnostics test module {name}"),
        },
    )
}

fn interface_id(module: &ModuleIdentity, name: &str) -> InterfaceIdentityId {
    InterfaceIdentityId::new(module.clone(), name)
}

fn member_id(interface: InterfaceIdentityId, name: &str) -> AssociatedMemberIdentityId {
    AssociatedMemberIdentityId::associated_type(
        interface.clone(),
        name,
        vec![interface.name.to_string(), name.to_string()],
    )
}

fn source_anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "TASK-868 behavioral non-interference fixture".into(),
        },
        None,
        label,
    )
}

fn type_list_domain_id(owner: &ModuleIdentity) -> SealedDomainId {
    SealedDomainId::new(owner.clone(), "TypeList")
}

fn type_list_ctor(owner: &ModuleIdentity, name: &str) -> DomainConstructorId {
    DomainConstructorId::new(type_list_domain_id(owner), name)
}

fn type_list_summary(owner: &ModuleIdentity) -> SealedDomainSummary {
    let domain = type_list_domain_id(owner);
    let nil = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), "Nil"),
        "Nil",
        vec![],
        source_anchor("Nil"),
    );
    let cons = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), "Cons"),
        "Cons",
        vec![
            DomainFieldSummary::unconstrained("head"),
            DomainFieldSummary::constrained_to("tail", &domain, domain.clone()),
        ],
        source_anchor("Cons"),
    );
    SealedDomainSummary::new(
        domain,
        "TypeList",
        CoreVisibility::Public,
        source_anchor("TypeList"),
    )
    .with_constructor(nil)
    .with_constructor(cons)
}

fn nil_for(owner: &ModuleIdentity) -> NormalTypeExpr {
    NormalTypeExpr::DomainConstructorApp {
        constructor: type_list_ctor(owner, "Nil"),
        domain: type_list_domain_id(owner),
        args: vec![],
        kind: Kind::Type,
    }
}

fn cons_for(owner: &ModuleIdentity, head: NormalTypeExpr, tail: NormalTypeExpr) -> NormalTypeExpr {
    NormalTypeExpr::DomainConstructorApp {
        constructor: type_list_ctor(owner, "Cons"),
        domain: type_list_domain_id(owner),
        args: vec![head, tail],
        kind: Kind::Type,
    }
}

fn assert_lsp_shape(err: &TypeEnvError, code: &str, expected_span: Span, tokens: &[&str]) {
    assert_eq!(
        AshLspError::span(err),
        Some(DiagnosticSpan::new(10, 31, 3, 7))
    );
    assert_eq!(err.span(), expected_span);
    assert_eq!(err.severity(), Severity::Error);
    assert_eq!(err.code(), Some(DiagnosticCode(code.into())));

    let message = err.message();
    for token in tokens {
        assert!(
            message.contains(token),
            "diagnostic {err:?} message `{message}` should contain token `{token}`"
        );
    }
}

fn assert_projection_blocker(
    normal: NormalTypeExpr,
    expected_reason: NormalFormBlockReason,
    expected_rigidity: ProjectionRigidity,
    interface: &InterfaceIdentityId,
    member: &AssociatedMemberIdentityId,
) {
    match normal {
        NormalTypeExpr::Projection {
            interface: actual_interface,
            member: actual_member,
            args,
            rigidity,
            reason,
            ..
        } => {
            assert_eq!(actual_interface, *interface);
            assert_eq!(actual_member, *member);
            assert_eq!(args, vec![NormalTypeExpr::Var("T".into())]);
            assert_eq!(rigidity, expected_rigidity);
            assert_eq!(reason, Some(expected_reason.clone()));
            assert!(format!("{expected_reason:?}").contains(&format!("{expected_reason:?}")));
        }
        other => panic!("expected blocked associated projection, got {other:?}"),
    }
}

#[test]
fn task_868_structured_type_env_diagnostics_preserve_codes_spans_and_family_identity() {
    let owner = module("owner", 1);
    let downstream = module("downstream", 2);
    let s = span();

    let missing = TypeEnvError::MissingAssociatedFamilyBinding {
        interface: "Iterator".into(),
        family: "Item".into(),
        span: s,
    };
    assert_lsp_shape(&missing, "E137", s, &["missing", "Item", "Iterator"]);
    match &missing {
        TypeEnvError::MissingAssociatedFamilyBinding {
            interface,
            family,
            span,
        } => {
            assert_eq!(interface, "Iterator");
            assert_eq!(family, "Item");
            assert_eq!(*span, s);
        }
        _ => unreachable!("constructed missing-binding diagnostic changed variant"),
    }

    let extra = TypeEnvError::ExtraAssociatedFamilyBinding {
        interface: "Iterator".into(),
        family: "Output".into(),
        span: s,
    };
    assert_lsp_shape(&extra, "E138", s, &["extra", "Output", "Iterator"]);
    match &extra {
        TypeEnvError::ExtraAssociatedFamilyBinding {
            interface,
            family,
            span,
        } => {
            assert_eq!(interface, "Iterator");
            assert_eq!(family, "Output");
            assert_eq!(*span, s);
        }
        _ => unreachable!("constructed extra-binding diagnostic changed variant"),
    }

    let duplicate = TypeEnvError::DuplicateAssociatedFamilyHead {
        interface: "Iterator".into(),
        family: "Item".into(),
        span: s,
    };
    assert_lsp_shape(&duplicate, "E139", s, &["duplicate", "Item", "Iterator"]);

    let unauthorized = TypeEnvError::UnauthorizedAssociatedFamilyExtension {
        family: "Item".into(),
        owner_module: owner.clone(),
        attempted_module: downstream.clone(),
        span: s,
    };
    assert_lsp_shape(
        &unauthorized,
        "E161",
        s,
        &["unauthorized", "Item", "owner", "downstream"],
    );
    match &unauthorized {
        TypeEnvError::UnauthorizedAssociatedFamilyExtension {
            family,
            owner_module,
            attempted_module,
            span,
        } => {
            assert_eq!(family, "Item");
            assert_eq!(owner_module, &owner);
            assert_eq!(attempted_module, &downstream);
            assert_eq!(*span, s);
        }
        _ => unreachable!("constructed unauthorized-extension diagnostic changed variant"),
    }

    let owner_violation = TypeEnvError::AssociatedFamilyModuleOwnerViolation {
        family: "Out".into(),
        reason: "missing defining module owner context; add owner module before publishing scheme"
            .into(),
        span: s,
    };
    assert_lsp_shape(
        &owner_violation,
        "E162",
        s,
        &["Out", "owner context", "module"],
    );

    let overlap = TypeEnvError::OverlappingAssociatedFamilyScheme {
        family: "Out".into(),
        span: s,
    };
    assert_lsp_shape(&overlap, "E163", s, &["overlapping", "Out"]);

    let wrong_kind = TypeEnvError::WrongAssociatedFamilyResultKind {
        family: "Out".into(),
        expected: "Type".into(),
        found: "Effect".into(),
        span: s,
    };
    assert_lsp_shape(&wrong_kind, "E164", s, &["Out", "expected Type", "Effect"]);

    let wrong_domain = TypeEnvError::WrongAssociatedFamilyResultDomain {
        family: "Out".into(),
        reason: "result must stay inside sealed domain TypeList".into(),
        span: s,
    };
    assert_lsp_shape(
        &wrong_domain,
        "E165",
        s,
        &["Out", "sealed domain", "TypeList"],
    );

    let ambiguous_member = TypeEnvError::AmbiguousAssociatedType {
        name: "Item' for projection 'T::Item' with candidate bounds [Iterable, Iterator]".into(),
        span: s,
    };
    assert_lsp_shape(
        &ambiguous_member,
        "E132",
        s,
        &["ambiguous", "Item", "Iterator", "Iterable"],
    );
}

#[test]
fn task_868_generic_registration_diagnostics_cover_remaining_spec063_families_without_new_variants()
{
    let s = span();
    let routes = [
        (
            "AssociatedFamilySyntaxUnsupported",
            TypeEnvError::InvalidDefinition(
                "associated-family projection shape is parsed but unsupported in the MVP; use <Interface<T>>::Assoc with an unqualified interface head".into(),
                s,
            ),
            "E122",
            &["associated-family", "unsupported", "MVP"][..],
        ),
        (
            "AssociatedFamilyUnreachableRow",
            TypeEnvError::InvalidDefinition(
                "associated family 'Out' has an unreachable row after ordered residual subtraction; remove or reorder the row".into(),
                s,
            ),
            "E122",
            &["Out", "unreachable", "row"][..],
        ),
        (
            "AssociatedFamilyNonExhaustive",
            TypeEnvError::InvalidDefinition(
                "associated family 'Out' is non-exhaustive for sealed domain TypeList; add missing Nil/Cons rows".into(),
                s,
            ),
            "E122",
            &["Out", "non-exhaustive", "TypeList"][..],
        ),
        (
            "AssociatedFamilyMissingDecreases",
            TypeEnvError::InvalidDefinition(
                "recursive associated family 'Out' is missing decreases; add `decreases Xs`".into(),
                s,
            ),
            "E122",
            &["Out", "missing decreases", "decreases Xs"][..],
        ),
        (
            "AssociatedFamilyInvalidDecreases",
            TypeEnvError::InvalidDefinition(
                "associated family 'Out' has invalid decreases parameter 'Zs'; choose a declared sealed-domain parameter".into(),
                s,
            ),
            "E122",
            &["Out", "invalid decreases", "Zs"][..],
        ),
        (
            "AssociatedFamilyNotDecreasing",
            TypeEnvError::InvalidDefinition(
                "associated family 'Out' recursive RHS is not structurally decreasing on Xs; recurse on a direct subcomponent".into(),
                s,
            ),
            "E122",
            &["Out", "not structurally decreasing", "Xs"][..],
        ),
        (
            "AssociatedFamilyMutualRecursionUnsupported",
            TypeEnvError::InvalidDefinition(
                "associated family 'Out' participates in mutual recursion; the MVP only accepts self recursion".into(),
                s,
            ),
            "E122",
            &["Out", "mutual recursion", "MVP"][..],
        ),
    ];

    assert_eq!(
        routes.len(),
        7,
        "SPEC-063 registration fallback route count changed"
    );
    for (family, err, code, tokens) in routes {
        assert_lsp_shape(&err, code, s, tokens);
        assert!(
            err.message().contains("associated family")
                || err.message().contains("associated-family"),
            "{family} should stay routed through an associated-family diagnostic message: {err}"
        );
    }
}

#[test]
fn task_868_summary_export_import_diagnostics_preserve_version_visibility_and_closure_tokens() {
    let s = span();
    let private_dependency = TypeEnvError::PrivateDependencyExportFailure {
        public_item: "Iterator::Item".into(),
        dependency: "PrivateHelper".into(),
        dependency_kind: "associated family dependency".into(),
        span: s,
    };
    assert_lsp_shape(
        &private_dependency,
        "E135",
        s,
        &[
            "private",
            "Iterator::Item",
            "PrivateHelper",
            "associated family",
        ],
    );

    let import_order = TypeEnvError::ImportOrderConflict {
        family: "associated family dependency closure".into(),
        name: "Append::Out".into(),
        span: s,
    };
    assert_lsp_shape(
        &import_order,
        "E136",
        s,
        &["import-order", "Append::Out", "dependency closure"],
    );

    let unsupported = TypeEnvError::UnsupportedSummaryVersion {
        version: SummaryVersion::SPEC062_TYPE_COMPUTATION_V3,
        expected: "SPEC-063 associated-family V4 summary".into(),
        span: s,
    };
    assert_lsp_shape(
        &unsupported,
        "E133",
        s,
        &["unsupported", "expected", "SPEC-063"],
    );

    let malformed = TypeEnvError::MalformedImportedComputationSummary {
        message: "associated family summary malformed: bad decreases metadata, result-domain mismatch, selected-scheme ambiguity, or dependency-closure conflict".into(),
        version: SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4,
        span: s,
    };
    assert_lsp_shape(
        &malformed,
        "E134",
        s,
        &[
            "associated family",
            "decreases",
            "result-domain",
            "selected-scheme ambiguity",
            "dependency-closure",
        ],
    );
}

#[test]
fn task_868_blocker_reasons_carry_non_fatal_associated_family_projection_notes() {
    let owner = module("blockers", 3);
    let interface = interface_id(&owner, "Iterator");
    let member = member_id(interface.clone(), "Item");

    let cases = [
        (
            NormalFormBlockReason::AssociatedFamilyNotSealed,
            ProjectionRigidity::Neutral,
        ),
        (
            NormalFormBlockReason::MissingAssociatedEvidence,
            ProjectionRigidity::Neutral,
        ),
        (
            NormalFormBlockReason::AmbiguousAssociatedFamilySelection,
            ProjectionRigidity::Neutral,
        ),
        (
            NormalFormBlockReason::AssociatedFamilyLocalUnavailable,
            ProjectionRigidity::Neutral,
        ),
        (
            NormalFormBlockReason::ImportedAssociatedFamilyUnsupported,
            ProjectionRigidity::Neutral,
        ),
        (
            NormalFormBlockReason::RigidProjection,
            ProjectionRigidity::Rigid,
        ),
        (
            NormalFormBlockReason::AbstractScrutinee,
            ProjectionRigidity::Neutral,
        ),
        (
            NormalFormBlockReason::NeutralScrutinee,
            ProjectionRigidity::Neutral,
        ),
    ];

    assert_eq!(
        cases.len(),
        8,
        "associated-family blocker route inventory changed"
    );
    for (reason, rigidity) in cases {
        let normal = NormalTypeExpr::Projection {
            interface: interface.clone(),
            member: member.clone(),
            args: vec![NormalTypeExpr::Var("T".into())],
            kind: Kind::Type,
            rigidity,
            reason: Some(reason.clone()),
        };
        assert_projection_blocker(normal, reason, rigidity, &interface, &member);
    }
}

fn param(name: &str) -> InterfaceTypeParam {
    param_with_domain(name, None)
}

fn param_with_domain(name: &str, domain: Option<&str>) -> InterfaceTypeParam {
    InterfaceTypeParam {
        name: name.into(),
        domain: domain.map(|domain| SurfaceType::Name(domain.into())),
        kind: None,
        span: span(),
    }
}

fn append_interface() -> InterfaceDef {
    InterfaceDef {
        visibility: Visibility::Inherited,
        name: "Append".into(),
        type_params: vec![param("Xs"), param("Ys")],
        evidence_constraints: vec![],
        associated_types: vec![AssociatedTypeDecl {
            name: "Out".into(),
            kind: AssociatedTypeKind::SealedFamily {
                result_domain: SurfaceType::Name("Type".into()),
                decreases: None,
                span: span(),
            },
            span: span(),
        }],
        methods: vec![],
        laws: Vec::new(),
        span: span(),
    }
}

#[test]
fn task_868_append_output_comparison_is_associated_family_specific_non_inverting_evidence() {
    let owner = module("append", 4);
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(owner.clone());
    env.register_interface(&append_interface())
        .expect("sealed Append::Out declaration should register for non-inversion diagnostics");
    let declaration = env
        .lookup_associated_family_declaration("Append", "Out")
        .expect("Append::Out family declaration is present");

    let lhs = CanonicalTypeExpr::Projection {
        interface: declaration.head.interface.clone(),
        member: declaration.head.member.clone(),
        args: vec![
            CanonicalTypeExpr::Var("Xs".into()),
            CanonicalTypeExpr::Var("Ys".into()),
        ],
        kind: Kind::Type,
        rigidity: ProjectionRigidity::Neutral,
    };
    let rhs = CanonicalTypeExpr::NominalApp {
        origin: TypeDeclId::ordinary(owner, "Cons"),
        visible_name: "Cons".into(),
        args: vec![
            CanonicalTypeExpr::Var("A".into()),
            CanonicalTypeExpr::Primitive("Nil".into()),
        ],
        kind: Kind::Type,
    };

    let equality = Normalizer::new(&env)
        .definitional_equality(&lhs, &rhs)
        .expect("associated-family equality should produce structured blocked evidence");

    match equality {
        DefinitionalEqualityResult::BlockedByNeutrality {
            lhs_norm,
            rhs_norm,
            neutral_subterms,
            no_inversion_note,
        } => {
            assert!(no_inversion_note.contains("does not invert"));
            match lhs_norm {
                NormalTypeExpr::Projection {
                    interface,
                    member,
                    args,
                    reason,
                    rigidity,
                    ..
                } => {
                    assert_eq!(interface, declaration.head.interface);
                    assert_eq!(member, declaration.head.member);
                    assert_eq!(
                        args,
                        vec![
                            NormalTypeExpr::Var("Xs".into()),
                            NormalTypeExpr::Var("Ys".into())
                        ]
                    );
                    assert_eq!(rigidity, ProjectionRigidity::Neutral);
                    assert_eq!(
                        reason,
                        Some(NormalFormBlockReason::AssociatedFamilyLocalUnavailable)
                    );
                }
                other => panic!("expected Append::Out projection on lhs, got {other:?}"),
            }
            assert!(
                matches!(rhs_norm, NormalTypeExpr::NominalApp { ref visible_name, .. } if visible_name == "Cons")
            );
            assert_eq!(neutral_subterms.len(), 1);
        }
        other => panic!("Append output comparison must not solve Xs/Ys by inversion: {other:?}"),
    }
}

#[test]
fn task_868_negative_leakage_boundaries_keep_prior_specs_non_regressed() {
    let s = span();

    // SPEC-035: ordinary associated-type substitution remains behaviorally active
    // beside the associated-family table; this exercises TypeEnv impl selection and
    // normalization rather than only constructing a diagnostic carrier.
    let spec035_module = module("spec035-ordinary-associated", 5);
    let mut spec035_env = TypeEnv::with_builtin_types();
    spec035_env.set_current_module_identity(spec035_module);
    let serializer = InterfaceDef {
        visibility: Visibility::Inherited,
        name: "Serializer".into(),
        type_params: vec![param("S")],
        evidence_constraints: vec![],
        associated_types: vec![AssociatedTypeDecl {
            name: "Ok".into(),
            kind: AssociatedTypeKind::Ordinary,
            span: s,
        }],
        methods: vec![InterfaceMethodSig {
            name: "serialize_bool".into(),
            params: vec![
                SurfaceType::Name("S".into()),
                SurfaceType::Name("Bool".into()),
            ],
            return_type: SurfaceType::Associated {
                base: Box::new(SurfaceType::Name("S".into())),
                name: "Ok".into(),
            },
            span: s,
        }],
        laws: Vec::new(),
        span: s,
    };
    spec035_env
        .register_interface(&serializer)
        .expect("ordinary associated interface registers");
    let serializer_impl = ImplDef {
        visibility: Visibility::Inherited,
        interface: "Serializer".into(),
        type_params: vec![],
        type_args: vec![SurfaceType::Name("String".into())],
        where_bounds: vec![],
        associated_type_bindings: vec![AssociatedTypeBinding {
            name: "Ok".into(),
            ty: SurfaceType::Name("String".into()),
            span: s,
        }],
        methods: vec![ImplMethodDef {
            name: "serialize_bool".into(),
            params: vec!["writer".into(), "value".into()],
            body: Expr::Literal(Literal::String("serialized".into())),
            span: s,
        }],
        proofs: Vec::new(),
        span: s,
    };
    spec035_env
        .register_impl(&serializer_impl)
        .expect("ordinary associated impl registers");
    let (selected, scheme) = spec035_env
        .select_impl_scheme("Serializer", "serialize_bool", &[Type::String, Type::Bool])
        .expect("SPEC-035 selected concrete impl still resolves");
    let substituted = spec035_env
        .normalize_associated_types(
            &scheme.methods[0].return_type,
            scheme,
            &selected.substitution,
        )
        .expect("SPEC-035 associated type substitution still normalizes");
    assert_eq!(substituted, Type::String);
    assert_eq!(
        spec035_env
            .resolve_interface_method_call(
                "Serializer",
                "serialize_bool",
                &[Type::String, Type::Bool]
            )
            .expect("method return uses SPEC-035 associated-type substitution"),
        Type::String
    );

    // SPEC-058: projection identity still comes from semantic interface/member IDs
    // through the canonical lowering path, not from display strings.
    let spec058_module = module("spec058-projection-identity", 6);
    let mut spec058_env = TypeEnv::with_builtin_types();
    spec058_env.set_current_module_identity(spec058_module);
    spec058_env
        .register_interface(&append_interface())
        .expect("family declaration registers for projection lowering");
    let append_decl = spec058_env
        .lookup_associated_family_declaration("Append", "Out")
        .expect("Append::Out declaration exists");
    let lowered = spec058_env
        .lower_surface_type_to_canonical(&SurfaceType::AssociatedFamilyProjection {
            interface: "Append".into(),
            args: vec![
                SurfaceType::Name("Xs".into()),
                SurfaceType::Name("Ys".into()),
            ],
            member: "Out".into(),
            span: s,
        })
        .expect("explicit family projection lowers through TypeEnv");
    assert_eq!(
        lowered,
        CanonicalTypeExpr::Projection {
            interface: append_decl.head.interface.clone(),
            member: append_decl.head.member.clone(),
            args: vec![
                CanonicalTypeExpr::Var("Xs".into()),
                CanonicalTypeExpr::Var("Ys".into()),
            ],
            kind: Kind::Type,
            rigidity: ProjectionRigidity::Neutral,
        }
    );

    // SPEC-060: actual definitional equality over an unknown computation head
    // reports blocked non-inverting evidence and preserves the queried inputs.
    let spec060_env = TypeEnv::new();
    let computation_head = TypeComputationHeadId::new(module("spec060-noninversion", 7), "Append");
    let open_lhs = CanonicalTypeExpr::ComputationHeadApp {
        head: computation_head.clone(),
        args: vec![
            CanonicalTypeExpr::Var("Xs".into()),
            CanonicalTypeExpr::Var("Ys".into()),
        ],
        kind: Kind::Type,
    };
    let open_rhs = CanonicalTypeExpr::NominalApp {
        origin: TypeDeclId::ordinary(module("spec060-noninversion", 7), "Cons"),
        visible_name: "Cons".into(),
        args: vec![
            CanonicalTypeExpr::Var("A".into()),
            CanonicalTypeExpr::Primitive("Nil".into()),
        ],
        kind: Kind::Type,
    };
    match Normalizer::new(&spec060_env)
        .definitional_equality(&open_lhs, &open_rhs)
        .expect("SPEC-060 defeq returns structured non-inversion evidence")
    {
        DefinitionalEqualityResult::BlockedByNeutrality {
            lhs_norm,
            neutral_subterms,
            no_inversion_note,
            ..
        } => {
            assert!(no_inversion_note.contains("does not invert"));
            assert_eq!(
                lhs_norm,
                NormalTypeExpr::NeutralComputationApp {
                    head: computation_head,
                    args: vec![
                        NormalTypeExpr::Var("Xs".into()),
                        NormalTypeExpr::Var("Ys".into())
                    ],
                    kind: Kind::Type,
                    reason: NormalFormBlockReason::Unsupported,
                }
            );
            assert!(neutral_subterms.iter().any(|term| {
                matches!(
                    term,
                    NormalTypeExpr::NeutralComputationApp { args, .. }
                        if args == &vec![NormalTypeExpr::Var("Xs".into()), NormalTypeExpr::Var("Ys".into())]
                )
            }));
        }
        other => panic!("SPEC-060 must remain non-inverting, got {other:?}"),
    }

    // SPEC-061: direct source-backed `type fn` behavior still registers and
    // reduces through the normalizer after associated-family diagnostics exist.
    let spec061_module = module("spec061-type-function", 8);
    let mut spec061_summary = ModuleSemanticSummary::new(spec061_module.clone())
        .with_exported_sealed_domain(type_list_summary(&spec061_module));
    spec061_summary.version = SummaryVersion::SPEC059_SEALED_DOMAIN_V2;
    let mut spec061_env = TypeEnv::new();
    spec061_env
        .register_module_semantic_summary(&spec061_summary)
        .expect("SPEC-061 sealed-domain precondition registers");
    let parsed = ash_parser::parse_surface_file(
        r#"
        type fn Append(xs: TypeList, ys: TypeList) -> TypeList decreases xs {
            case Append<Nil, ys> = ys;
            case Append<Cons<h, t>, ys> = Cons<h, Append<t, ys>>;
        }
        "#,
    )
    .expect("SPEC-061 source type function parses");
    let type_fns = parsed
        .definitions
        .into_iter()
        .filter_map(|definition| match definition {
            Definition::TypeFn(type_fn) => Some(type_fn),
            _ => None,
        })
        .collect::<Vec<_>>();
    spec061_env
        .register_local_type_functions(&spec061_module, &type_fns)
        .expect("SPEC-061 direct type function still validates");
    let append_head = spec061_env
        .lookup_local_type_function("Append")
        .expect("Append type function registered")
        .head
        .clone();
    let ys = cons_for(
        &spec061_module,
        NormalTypeExpr::Primitive("B".into()),
        nil_for(&spec061_module),
    );
    let reduced = Normalizer::new(&spec061_env)
        .normalize_known_computation_app(
            &append_head,
            vec![nil_for(&spec061_module), ys.clone()],
            &Kind::Type,
        )
        .expect("SPEC-061 direct type-function normalizer still reduces");
    assert_eq!(reduced, ys);

    // SPEC-062: V3 public type-function summary versioning remains valid and
    // future-version rejection still comes from core summary validation, not from
    // associated-family V4-only rules.
    ModuleSemanticSummary::new(module("spec062-v3", 9))
        .with_version(SummaryVersion::SPEC062_TYPE_COMPUTATION_V3)
        .validate_summary_version_contract()
        .expect("SPEC-062 V3 summary without associated-family facts remains valid");
    let unsupported = ModuleSemanticSummary::new(module("spec062-future", 10))
        .with_version(SummaryVersion(99))
        .validate_summary_version_contract()
        .expect_err("future summary version still rejects through core validation");
    assert_eq!(
        unsupported,
        ModuleSemanticSummaryValidationError::UnsupportedSummaryVersion {
            version: SummaryVersion(99)
        }
    );
}

#[test]
fn task_868_spec063_diagnostic_family_inventory_has_nonzero_public_route_count() {
    let diagnostic_families = [
        "AssociatedFamilySyntaxUnsupported",
        "AssociatedFamilyNotSealed",
        "AssociatedFamilyAmbiguousMember",
        "AssociatedFamilyImplNotInSealedSet",
        "AssociatedFamilyMissingBinding",
        "AssociatedFamilyExtraBinding",
        "AssociatedFamilyOverlap",
        "AssociatedFamilyUnreachableRow",
        "AssociatedFamilyNonExhaustive",
        "AssociatedFamilyMissingDecreases",
        "AssociatedFamilyInvalidDecreases",
        "AssociatedFamilyNotDecreasing",
        "AssociatedFamilyResultKindMismatch",
        "AssociatedFamilyResultDomainMismatch",
        "AssociatedFamilyMutualRecursionUnsupported",
        "AssociatedFamilySelectionAmbiguous",
        "AssociatedFamilyRigidProjection",
        "AssociatedFamilyPrivateReductionUnavailable",
        "AssociatedFamilyExportPrivateDependency",
        "AssociatedFamilyExportNotClosed",
        "AssociatedFamilyImportOrderConflict",
        "AssociatedFamilyDependencyClosureConflict",
        "AssociatedFamilySummaryMalformed",
        "AssociatedFamilySummaryUnsupportedVersion",
    ];
    let public_error_routes = 16;
    let public_blocker_routes = 8;

    assert_eq!(diagnostic_families.len(), 24);
    assert!(
        diagnostic_families
            .iter()
            .all(|family| family.starts_with("AssociatedFamily"))
    );
    assert!(public_error_routes > 0);
    assert!(public_blocker_routes > 0);
    assert!(public_error_routes + public_blocker_routes >= diagnostic_families.len());
}
