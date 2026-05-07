use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use ash_core::ast::{Span, Visibility};
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, DomainConstructorId, InterfaceIdentityId, ModuleIdentity,
    ModuleSourceOrigin, SealedDomainId, SourceAnchor, SourceOrigin, TypeDeclId,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, ProjectionRigidity, TypeComputationHeadId, TypeFunctionDef,
    TypeFunctionEquation, TypeFunctionParam, TypeFunctionPattern, TypeFunctionPatternConstraint,
    TypeFunctionResultConstraint, TypeFunctionResultExpr, TypeFunctionSourceAnchors,
};

fn module_identity() -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(833)),
        ModuleId(113),
        vec!["phase113".to_string(), "task833".to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: "task-833 core type-function carrier test".to_string(),
        },
    )
}

fn anchor(label: &str, start: usize, end: usize) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-833 focused carrier test".to_string(),
        },
        Some(Span { start, end }),
        label,
    )
}

fn hash_of<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn typelist_domain() -> SealedDomainId {
    SealedDomainId::new(module_identity(), "TypeList")
}

fn ctor(domain: &SealedDomainId, name: &str) -> DomainConstructorId {
    DomainConstructorId::new(domain.clone(), name)
}

fn append_def() -> TypeFunctionDef {
    let module = module_identity();
    let typelist = typelist_domain();
    let nil = ctor(&typelist, "Nil");
    let cons = ctor(&typelist, "Cons");
    let head = TypeComputationHeadId::new(module, "Append");

    TypeFunctionDef {
        visibility: Visibility::Private,
        head: head.clone(),
        name: "Append".to_string(),
        params: vec![
            TypeFunctionParam {
                name: "xs".to_string(),
                ty: CanonicalTypeExpr::Primitive("TypeList".to_string()),
                kind: Kind::Type,
                domain_constraint: Some(typelist.clone()),
                source_anchor: anchor("parameter xs", 15, 27),
            },
            TypeFunctionParam {
                name: "ys".to_string(),
                ty: CanonicalTypeExpr::Primitive("TypeList".to_string()),
                kind: Kind::Type,
                domain_constraint: Some(typelist.clone()),
                source_anchor: anchor("parameter ys", 29, 41),
            },
        ],
        return_type: CanonicalTypeExpr::Primitive("TypeList".to_string()),
        return_kind: Kind::Type,
        result_constraint: TypeFunctionResultConstraint::Domain(typelist.clone()),
        decreases: Some("xs".to_string()),
        source_anchors: TypeFunctionSourceAnchors {
            definition: anchor("type fn Append", 0, 180),
            decreases: Some(anchor("decreases xs", 55, 67)),
        },
        equations: vec![
            TypeFunctionEquation {
                head: head.clone(),
                ordinal: 0,
                patterns: vec![
                    TypeFunctionPattern::DomainConstructor {
                        constructor: Box::new(nil.clone()),
                        domain: Box::new(typelist.clone()),
                        fields: vec![],
                        constraint: TypeFunctionPatternConstraint::Domain(typelist.clone()),
                        source_anchor: anchor("pattern Nil", 83, 86),
                    },
                    TypeFunctionPattern::Var {
                        name: "ys".to_string(),
                        constraint: TypeFunctionPatternConstraint::Domain(typelist.clone()),
                        source_anchor: anchor("pattern ys", 88, 90),
                    },
                ],
                result: TypeFunctionResultExpr::Var {
                    name: "ys".to_string(),
                    kind: Kind::Type,
                    constraint: TypeFunctionResultConstraint::Domain(typelist.clone()),
                    source_anchor: anchor("result ys", 94, 96),
                },
                source_anchor: anchor("case Append<Nil, ys>", 70, 97),
                case_head_anchor: anchor("case head Append", 75, 81),
            },
            TypeFunctionEquation {
                head: head.clone(),
                ordinal: 1,
                patterns: vec![
                    TypeFunctionPattern::DomainConstructor {
                        constructor: Box::new(cons.clone()),
                        domain: Box::new(typelist.clone()),
                        fields: vec![
                            TypeFunctionPattern::Var {
                                name: "h".to_string(),
                                constraint: TypeFunctionPatternConstraint::Kind(Kind::Type),
                                source_anchor: anchor("pattern h", 116, 117),
                            },
                            TypeFunctionPattern::Var {
                                name: "t".to_string(),
                                constraint: TypeFunctionPatternConstraint::Domain(typelist.clone()),
                                source_anchor: anchor("pattern t", 119, 120),
                            },
                        ],
                        constraint: TypeFunctionPatternConstraint::Domain(typelist.clone()),
                        source_anchor: anchor("pattern Cons<h, t>", 111, 121),
                    },
                    TypeFunctionPattern::Var {
                        name: "ys".to_string(),
                        constraint: TypeFunctionPatternConstraint::Domain(typelist.clone()),
                        source_anchor: anchor("pattern ys", 123, 125),
                    },
                ],
                result: TypeFunctionResultExpr::DomainConstructorApp {
                    constructor: cons,
                    domain: typelist.clone(),
                    args: vec![
                        TypeFunctionResultExpr::Var {
                            name: "h".to_string(),
                            kind: Kind::Type,
                            constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
                            source_anchor: anchor("result h", 134, 135),
                        },
                        TypeFunctionResultExpr::ComputationHeadApp {
                            head,
                            args: vec![
                                TypeFunctionResultExpr::Var {
                                    name: "t".to_string(),
                                    kind: Kind::Type,
                                    constraint: TypeFunctionResultConstraint::Domain(
                                        typelist.clone(),
                                    ),
                                    source_anchor: anchor("result t", 144, 145),
                                },
                                TypeFunctionResultExpr::Var {
                                    name: "ys".to_string(),
                                    kind: Kind::Type,
                                    constraint: TypeFunctionResultConstraint::Domain(
                                        typelist.clone(),
                                    ),
                                    source_anchor: anchor("result ys", 147, 149),
                                },
                            ],
                            kind: Kind::Type,
                            constraint: TypeFunctionResultConstraint::Domain(typelist.clone()),
                            source_anchor: anchor("recursive Append<t, ys>", 137, 150),
                        },
                    ],
                    kind: Kind::Type,
                    constraint: TypeFunctionResultConstraint::Domain(typelist),
                    source_anchor: anchor("result Cons<h, Append<t, ys>>", 129, 151),
                },
                source_anchor: anchor("case Append<Cons<h, t>, ys>", 100, 152),
                case_head_anchor: anchor("case head Append", 105, 111),
            },
        ],
    }
}

#[test]
fn type_function_def_preserves_signature_decreases_equation_order_and_source_anchors() {
    let def = append_def();

    assert_eq!(def.name, "Append");
    assert_eq!(def.head.name, "Append");
    assert_eq!(
        def.params
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>(),
        ["xs", "ys"]
    );
    assert!(def.params.iter().all(|p| p.domain_constraint.is_some()));
    assert_eq!(def.decreases.as_deref(), Some("xs"));
    assert_eq!(def.source_anchors.definition.label, "type fn Append");
    assert_eq!(
        def.source_anchors
            .decreases
            .as_ref()
            .map(|a| a.label.as_str()),
        Some("decreases xs")
    );
    assert_eq!(
        def.equations
            .iter()
            .map(|eq| eq.ordinal)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert!(def.equations.iter().all(|eq| eq.head == def.head));
    assert_eq!(def.equations[0].case_head_anchor.label, "case head Append");
    assert_eq!(
        def.equations[1].source_anchor.label,
        "case Append<Cons<h, t>, ys>"
    );
}

#[test]
fn patterns_and_results_use_sealed_domain_constructor_ids_and_constraints() {
    let def = append_def();
    let typelist = typelist_domain();

    match &def.equations[1].patterns[0] {
        TypeFunctionPattern::DomainConstructor {
            constructor,
            domain,
            fields,
            constraint,
            source_anchor,
        } => {
            assert_eq!(domain.as_ref(), &typelist);
            assert_eq!(constructor.domain, typelist);
            assert_eq!(constructor.name, "Cons");
            assert_eq!(
                *constraint,
                TypeFunctionPatternConstraint::Domain(domain.as_ref().clone())
            );
            assert_eq!(source_anchor.label, "pattern Cons<h, t>");
            assert_eq!(fields.len(), 2);
        }
        other => panic!("expected sealed-domain constructor pattern, got {other:?}"),
    }

    match &def.equations[1].result {
        TypeFunctionResultExpr::DomainConstructorApp {
            constructor,
            domain,
            args,
            constraint,
            source_anchor,
            ..
        } => {
            assert_eq!(domain, &typelist_domain());
            assert_eq!(constructor.domain, typelist_domain());
            assert_eq!(constructor.name, "Cons");
            assert_eq!(
                *constraint,
                TypeFunctionResultConstraint::Domain(domain.clone())
            );
            assert_eq!(source_anchor.label, "result Cons<h, Append<t, ys>>");
            assert!(matches!(
                args[1],
                TypeFunctionResultExpr::ComputationHeadApp { .. }
            ));
        }
        other => panic!("expected sealed-domain constructor result app, got {other:?}"),
    }
}

#[test]
fn result_expr_supports_all_canonical_heads_plus_marker_constructor_apps() {
    let module = module_identity();
    let domain = typelist_domain();
    let nominal = TypeDeclId::ordinary(module.clone(), "Box");
    let interface = InterfaceIdentityId::new(module.clone(), "Iterable");
    let member = AssociatedMemberIdentityId::associated_type(
        interface.clone(),
        "Item",
        vec!["Item".to_string()],
    );
    let constraint = TypeFunctionResultConstraint::Kind(Kind::Type);

    let expressions = [
        TypeFunctionResultExpr::Primitive {
            name: "Type".to_string(),
            kind: Kind::Type,
            constraint: constraint.clone(),
            source_anchor: anchor("primitive", 0, 4),
        },
        TypeFunctionResultExpr::Var {
            name: "T".to_string(),
            kind: Kind::Type,
            constraint: constraint.clone(),
            source_anchor: anchor("var", 5, 6),
        },
        TypeFunctionResultExpr::NominalApp {
            origin: nominal,
            visible_name: "Box".to_string(),
            args: vec![],
            kind: Kind::Type,
            constraint: constraint.clone(),
            source_anchor: anchor("nominal", 7, 10),
        },
        TypeFunctionResultExpr::DomainConstructorApp {
            constructor: ctor(&domain, "Nil"),
            domain: domain.clone(),
            args: vec![],
            kind: Kind::Type,
            constraint: TypeFunctionResultConstraint::Domain(domain),
            source_anchor: anchor("marker", 11, 14),
        },
        TypeFunctionResultExpr::Projection {
            interface,
            member,
            args: vec![TypeFunctionResultExpr::Var {
                name: "T".to_string(),
                kind: Kind::Type,
                constraint: constraint.clone(),
                source_anchor: anchor("projection arg", 15, 16),
            }],
            kind: Kind::Type,
            constraint: constraint.clone(),
            rigidity: ProjectionRigidity::Neutral,
            source_anchor: anchor("projection", 15, 22),
        },
        TypeFunctionResultExpr::ComputationHeadApp {
            head: TypeComputationHeadId::new(module, "Append"),
            args: vec![],
            kind: Kind::Type,
            constraint,
            source_anchor: anchor("computation", 23, 29),
        },
    ];

    assert_eq!(expressions.len(), 6);
    assert!(
        expressions
            .iter()
            .any(|expr| matches!(expr, TypeFunctionResultExpr::DomainConstructorApp { .. }))
    );
    assert!(
        expressions
            .iter()
            .any(|expr| matches!(expr, TypeFunctionResultExpr::ComputationHeadApp { .. }))
    );
}

#[test]
fn carriers_are_equal_hashable_and_serde_roundtrip_across_crate_boundaries() {
    let def = append_def();
    let same = append_def();

    assert_eq!(def, same);
    assert_eq!(hash_of(&def), hash_of(&same));

    let json = serde_json::to_string(&def).expect("type-function carrier serializes");
    assert!(json.contains("DomainConstructorApp"));
    assert!(json.contains("ComputationHeadApp"));
    assert!(json.contains("decreases xs"));
    let decoded: TypeFunctionDef =
        serde_json::from_str(&json).expect("type-function carrier deserializes");
    assert_eq!(decoded, def);
    assert_eq!(hash_of(&decoded), hash_of(&def));
}

#[test]
fn wildcard_patterns_preserve_kind_or_domain_constraint_metadata() {
    let domain = typelist_domain();
    let domain_wildcard = TypeFunctionPattern::Wildcard {
        constraint: TypeFunctionPatternConstraint::Domain(domain.clone()),
        source_anchor: anchor("domain wildcard", 0, 1),
    };
    let kind_wildcard = TypeFunctionPattern::Wildcard {
        constraint: TypeFunctionPatternConstraint::Kind(Kind::Type),
        source_anchor: anchor("kind wildcard", 2, 3),
    };

    assert_ne!(domain_wildcard, kind_wildcard);
    assert_ne!(hash_of(&domain_wildcard), hash_of(&kind_wildcard));

    match domain_wildcard {
        TypeFunctionPattern::Wildcard {
            constraint,
            source_anchor,
        } => {
            assert_eq!(constraint, TypeFunctionPatternConstraint::Domain(domain));
            assert_eq!(source_anchor.label, "domain wildcard");
        }
        other => panic!("expected wildcard pattern, got {other:?}"),
    }
}
