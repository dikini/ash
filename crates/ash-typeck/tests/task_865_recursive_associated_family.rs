use ash_core::ast::Visibility as CoreVisibility;
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, DomainConstructorId, DomainConstructorSummary, DomainFieldSummary,
    ModuleIdentity, ModuleSemanticSummary, ModuleSourceOrigin, SealedDomainId, SealedDomainSummary,
    SourceAnchor, SourceOrigin, SummaryVersion,
};
use ash_core::type_ir::{
    AssociatedFamilyEquation, AssociatedFamilyPattern, AssociatedFamilyResultConstraint,
    AssociatedFamilyResultExpr, AssociatedFamilyScheme, AssociatedFamilySchemeParam,
    CanonicalTypeExpr, ProjectionRigidity,
};
use ash_parser::surface::{
    AssociatedFamilyDecreases, AssociatedTypeDecl, AssociatedTypeKind, InterfaceDef,
    InterfaceTypeParam, Type as SurfaceType, Visibility,
};
use ash_parser::token::Span;
use ash_typeck::TypeEnv;

fn module(id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(865)),
        ModuleId(id),
        vec!["task865".into(), format!("m{id}")],
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-865-{id}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-865-test".into(),
        },
        None,
        label,
    )
}

fn span() -> Span {
    Span::default()
}

fn type_list_domain(module: &ModuleIdentity) -> SealedDomainSummary {
    let domain = SealedDomainId::new(module.clone(), "TypeList");
    let nil = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), "Nil"),
        "Nil",
        vec![],
        anchor("Nil"),
    );
    let cons = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), "Cons"),
        "Cons",
        vec![
            DomainFieldSummary::unconstrained("head"),
            DomainFieldSummary::constrained_to("tail", &domain, domain.clone()),
        ],
        anchor("Cons"),
    );
    SealedDomainSummary::new(
        domain,
        "TypeList",
        CoreVisibility::Public,
        anchor("TypeList"),
    )
    .with_constructor(nil)
    .with_constructor(cons)
}

fn flat_domain(module: &ModuleIdentity) -> SealedDomainSummary {
    let domain = SealedDomainId::new(module.clone(), "Flat");
    let z = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), "Z"),
        "Z",
        vec![],
        anchor("Z"),
    );
    let s = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), "S"),
        "S",
        vec![DomainFieldSummary::unconstrained("payload")],
        anchor("S"),
    );
    SealedDomainSummary::new(domain, "Flat", CoreVisibility::Public, anchor("Flat"))
        .with_constructor(z)
        .with_constructor(s)
}

fn register_domains(env: &mut TypeEnv, module: &ModuleIdentity) {
    let mut summary = ModuleSemanticSummary::new(module.clone())
        .with_exported_sealed_domain(type_list_domain(module))
        .with_exported_sealed_domain(flat_domain(module));
    summary.version = SummaryVersion::SPEC059_SEALED_DOMAIN_V2;
    env.register_module_semantic_summary(&summary)
        .expect("sealed domains register");
}

fn param(name: &str, domain: Option<&str>) -> InterfaceTypeParam {
    InterfaceTypeParam {
        name: name.into(),
        domain: domain.map(|name| SurfaceType::Name(name.into())),
        kind: None,
        span: span(),
    }
}

fn family_interface(
    decreases: Option<&str>,
    param_domain: Option<&str>,
    result_domain: &str,
) -> InterfaceDef {
    InterfaceDef {
        visibility: Visibility::Inherited,
        name: "AppendFamily".into(),
        type_params: vec![param("Xs", param_domain), param("Ys", Some("TypeList"))],
        associated_types: vec![AssociatedTypeDecl {
            name: "Out".into(),
            kind: AssociatedTypeKind::SealedFamily {
                result_domain: SurfaceType::Name(result_domain.into()),
                decreases: decreases.map(|param| AssociatedFamilyDecreases {
                    param: param.into(),
                    span: span(),
                }),
                span: span(),
            },
            span: span(),
        }],
        methods: vec![],
        span: span(),
    }
}

fn env_with_family(
    decreases: Option<&str>,
    param_domain: Option<&str>,
    result_domain: &str,
) -> (TypeEnv, ModuleIdentity, SealedDomainId) {
    let module = module(1 + decreases.map_or(0, str::len) + result_domain.len());
    let typelist = SealedDomainId::new(module.clone(), "TypeList");
    let mut env = TypeEnv::new();
    register_domains(&mut env, &module);
    env.set_current_module_identity(module.clone());
    env.register_interface(&family_interface(decreases, param_domain, result_domain))
        .expect("family interface registers");
    (env, module, typelist)
}

fn constraint(domain: &SealedDomainId) -> AssociatedFamilyResultConstraint {
    AssociatedFamilyResultConstraint::Domain(domain.clone())
}

fn param_meta(name: &str, domain: &SealedDomainId) -> AssociatedFamilySchemeParam {
    AssociatedFamilySchemeParam {
        name: name.into(),
        ty: CanonicalTypeExpr::Var(name.into()),
        kind: Kind::Type,
        domain_constraint: Some(domain.clone()),
        source_anchor: anchor(name),
    }
}

fn var_pat(name: &str, domain: &SealedDomainId) -> AssociatedFamilyPattern {
    AssociatedFamilyPattern::Var {
        name: name.into(),
        constraint: constraint(domain),
        source_anchor: anchor(name),
    }
}

fn kind_var_pat(name: &str) -> AssociatedFamilyPattern {
    AssociatedFamilyPattern::Var {
        name: name.into(),
        constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
        source_anchor: anchor(name),
    }
}

fn wild_pat(domain: &SealedDomainId) -> AssociatedFamilyPattern {
    AssociatedFamilyPattern::Wildcard {
        constraint: constraint(domain),
        source_anchor: anchor("_"),
    }
}

fn ctor_pat(
    name: &str,
    domain: &SealedDomainId,
    fields: Vec<AssociatedFamilyPattern>,
) -> AssociatedFamilyPattern {
    AssociatedFamilyPattern::DomainConstructor {
        constructor: Box::new(DomainConstructorId::new(domain.clone(), name)),
        domain: Box::new(domain.clone()),
        fields,
        constraint: constraint(domain),
        source_anchor: anchor(name),
    }
}

fn var_result(name: &str, domain: &SealedDomainId) -> AssociatedFamilyResultExpr {
    AssociatedFamilyResultExpr::Var {
        name: name.into(),
        kind: Kind::Type,
        constraint: constraint(domain),
        source_anchor: anchor(name),
    }
}

fn kind_var_result(name: &str) -> AssociatedFamilyResultExpr {
    AssociatedFamilyResultExpr::Var {
        name: name.into(),
        kind: Kind::Type,
        constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
        source_anchor: anchor(name),
    }
}

fn ctor_result(
    name: &str,
    domain: &SealedDomainId,
    args: Vec<AssociatedFamilyResultExpr>,
) -> AssociatedFamilyResultExpr {
    AssociatedFamilyResultExpr::DomainConstructorApp {
        constructor: DomainConstructorId::new(domain.clone(), name),
        domain: domain.clone(),
        args,
        kind: Kind::Type,
        constraint: constraint(domain),
        source_anchor: anchor(name),
    }
}

fn scheme(
    env: &TypeEnv,
    module: &ModuleIdentity,
    domain: &SealedDomainId,
    equations: Vec<(Vec<AssociatedFamilyPattern>, AssociatedFamilyResultExpr)>,
) -> AssociatedFamilyScheme {
    let head = env
        .lookup_associated_family_declaration("AppendFamily", "Out")
        .expect("family declaration exists")
        .head
        .clone();
    AssociatedFamilyScheme {
        head: head.clone(),
        params: vec![param_meta("Xs", domain), param_meta("Ys", domain)],
        result_domain: CanonicalTypeExpr::Var("TypeList".into()),
        result_kind: Kind::Type,
        equations: equations
            .into_iter()
            .enumerate()
            .map(
                |(ordinal, (interface_arg_patterns, result))| AssociatedFamilyEquation {
                    head: head.clone(),
                    ordinal,
                    interface_arg_patterns,
                    result,
                    decreases: None,
                    source_anchor: anchor(&format!("equation-{ordinal}")),
                    case_head_anchor: anchor(&format!("case-{ordinal}")),
                },
            )
            .collect(),
        source_anchor: SourceAnchor::new(
            SourceOrigin::Synthetic {
                reason: "task-865-scheme".into(),
            },
            None,
            format!("scheme-{}", module.module_id.0),
        ),
    }
}

fn recursive_call(
    env: &TypeEnv,
    arg: AssociatedFamilyResultExpr,
    ys: AssociatedFamilyResultExpr,
) -> AssociatedFamilyResultExpr {
    let head = env
        .lookup_associated_family_declaration("AppendFamily", "Out")
        .expect("family declaration exists")
        .head
        .clone();
    AssociatedFamilyResultExpr::AssociatedFamilyProjection {
        head,
        interface_args: vec![arg, ys],
        kind: Kind::Type,
        constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
        rigidity: ProjectionRigidity::Rigid,
        source_anchor: anchor("recursive call"),
    }
}

fn good_append_scheme(
    env: &TypeEnv,
    module: &ModuleIdentity,
    domain: &SealedDomainId,
) -> AssociatedFamilyScheme {
    scheme(
        env,
        module,
        domain,
        vec![
            (
                vec![ctor_pat("Nil", domain, vec![]), var_pat("Ys", domain)],
                var_result("Ys", domain),
            ),
            (
                vec![
                    ctor_pat(
                        "Cons",
                        domain,
                        vec![kind_var_pat("Head"), var_pat("Tail", domain)],
                    ),
                    var_pat("Ys", domain),
                ],
                ctor_result(
                    "Cons",
                    domain,
                    vec![
                        kind_var_result("Head"),
                        recursive_call(env, var_result("Tail", domain), var_result("Ys", domain)),
                    ],
                ),
            ),
        ],
    )
}

fn assert_rejects(
    mut env: TypeEnv,
    module: ModuleIdentity,
    scheme: AssociatedFamilyScheme,
    expected: &str,
) {
    let err = env
        .register_associated_family_scheme(scheme, module)
        .expect_err("associated family scheme should reject");
    let message = err.to_string();
    assert!(
        message.contains(expected),
        "expected diagnostic containing {expected:?}, got {message}"
    );
}

#[test]
fn task_865_accepts_append_like_recursive_associated_family() {
    let (mut env, module, domain) = env_with_family(Some("Xs"), Some("TypeList"), "TypeList");
    let scheme = good_append_scheme(&env, &module, &domain);
    env.register_associated_family_scheme(scheme, module)
        .expect("Append-style associated family recursion should validate");
}

#[test]
fn task_865_rejects_missing_decreases_on_recursive_family() {
    let (env, module, domain) = env_with_family(None, Some("TypeList"), "TypeList");
    let scheme = good_append_scheme(&env, &module, &domain);
    assert_rejects(
        env,
        module,
        scheme,
        "missing decreases clause for recursive associated family",
    );
}

#[test]
fn task_865_rejects_nonsealed_and_nonstructural_decreases_parameters() {
    let (env, module, domain) = env_with_family(Some("Xs"), Some("TypeList"), "TypeList");
    let mut nonsealed = good_append_scheme(&env, &module, &domain);
    nonsealed.params[0].domain_constraint = None;
    assert_rejects(env, module, nonsealed, "parameter is not a sealed domain");

    let (env, module, domain) = env_with_family(Some("Xs"), Some("Flat"), "TypeList");
    let flat = SealedDomainId::new(module.clone(), "Flat");
    let mut scheme = scheme(
        &env,
        &module,
        &domain,
        vec![
            (
                vec![ctor_pat("Z", &flat, vec![]), var_pat("Ys", &domain)],
                var_result("Ys", &domain),
            ),
            (
                vec![
                    ctor_pat("S", &flat, vec![kind_var_pat("Payload")]),
                    var_pat("Ys", &domain),
                ],
                recursive_call(
                    &env,
                    var_result("Payload", &flat),
                    var_result("Ys", &domain),
                ),
            ),
        ],
    );
    scheme.params[0].domain_constraint = Some(flat);
    assert_rejects(
        env,
        module,
        scheme,
        "sealed domain has no structural subcomponent metadata",
    );
}

#[test]
fn task_865_rejects_non_exhaustive_and_overlapping_rows() {
    let (env, module, domain) = env_with_family(Some("Xs"), Some("TypeList"), "TypeList");
    let non_exhaustive = scheme(
        &env,
        &module,
        &domain,
        vec![(
            vec![ctor_pat("Nil", &domain, vec![]), var_pat("Ys", &domain)],
            var_result("Ys", &domain),
        )],
    );
    assert_rejects(env, module, non_exhaustive, "non-exhaustive type function");

    let (env, module, domain) = env_with_family(Some("Xs"), Some("TypeList"), "TypeList");
    let overlapping = scheme(
        &env,
        &module,
        &domain,
        vec![
            (
                vec![wild_pat(&domain), var_pat("Ys", &domain)],
                var_result("Ys", &domain),
            ),
            (
                vec![ctor_pat("Nil", &domain, vec![]), var_pat("Ys", &domain)],
                var_result("Ys", &domain),
            ),
        ],
    );
    assert_rejects(
        env,
        module,
        overlapping,
        "unreachable type function equation",
    );
}

#[test]
fn task_865_rejects_same_rebuilt_and_computed_recursive_arguments() {
    let (env, module, domain) = env_with_family(Some("Xs"), Some("TypeList"), "TypeList");
    let same = scheme(
        &env,
        &module,
        &domain,
        vec![(
            vec![var_pat("Xs", &domain), var_pat("Ys", &domain)],
            recursive_call(&env, var_result("Xs", &domain), var_result("Ys", &domain)),
        )],
    );
    assert_rejects(
        env,
        module,
        same,
        "non-decreasing recursive call in associated family",
    );

    let (env, module, domain) = env_with_family(Some("Xs"), Some("TypeList"), "TypeList");
    let rebuilt = scheme(
        &env,
        &module,
        &domain,
        vec![
            (
                vec![ctor_pat("Nil", &domain, vec![]), var_pat("Ys", &domain)],
                var_result("Ys", &domain),
            ),
            (
                vec![
                    ctor_pat(
                        "Cons",
                        &domain,
                        vec![kind_var_pat("Head"), var_pat("Tail", &domain)],
                    ),
                    var_pat("Ys", &domain),
                ],
                recursive_call(
                    &env,
                    ctor_result(
                        "Cons",
                        &domain,
                        vec![kind_var_result("Head"), var_result("Tail", &domain)],
                    ),
                    var_result("Ys", &domain),
                ),
            ),
        ],
    );
    assert_rejects(
        env,
        module,
        rebuilt,
        "non-decreasing recursive call in associated family",
    );

    let (env, module, domain) = env_with_family(Some("Xs"), Some("TypeList"), "TypeList");
    let computed = scheme(
        &env,
        &module,
        &domain,
        vec![
            (
                vec![ctor_pat("Nil", &domain, vec![]), var_pat("Ys", &domain)],
                var_result("Ys", &domain),
            ),
            (
                vec![
                    ctor_pat(
                        "Cons",
                        &domain,
                        vec![kind_var_pat("Head"), var_pat("Tail", &domain)],
                    ),
                    var_pat("Ys", &domain),
                ],
                recursive_call(
                    &env,
                    recursive_call(&env, var_result("Tail", &domain), var_result("Ys", &domain)),
                    var_result("Ys", &domain),
                ),
            ),
        ],
    );
    assert_rejects(
        env,
        module,
        computed,
        "non-decreasing recursive call in associated family",
    );
}

#[test]
fn task_865_rejects_mutual_recursion_and_result_domain_mismatch() {
    let (env, module, domain) = env_with_family(Some("Xs"), Some("TypeList"), "TypeList");
    let mut bad = good_append_scheme(&env, &module, &domain);
    let other_member =
        AssociatedMemberIdentityId::associated_type(bad.head.interface.clone(), "Other", vec![]);
    let other_head = ash_core::type_ir::AssociatedFamilyHeadId {
        interface: bad.head.interface.clone(),
        member: other_member,
    };
    bad.equations[0].result = AssociatedFamilyResultExpr::AssociatedFamilyProjection {
        head: other_head,
        interface_args: vec![var_result("Xs", &domain), var_result("Ys", &domain)],
        kind: Kind::Type,
        constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
        rigidity: ProjectionRigidity::Rigid,
        source_anchor: anchor("other family"),
    };
    assert_rejects(env, module, bad, "mutual recursion in associated family");

    let (env, module, domain) = env_with_family(Some("Xs"), Some("TypeList"), "TypeList");
    let mut mismatch = good_append_scheme(&env, &module, &domain);
    mismatch.equations[0].result = AssociatedFamilyResultExpr::Primitive {
        name: "Int".into(),
        kind: Kind::Type,
        constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
        source_anchor: anchor("Int"),
    };
    assert_rejects(env, module, mismatch, "RHS does not conform");
}
