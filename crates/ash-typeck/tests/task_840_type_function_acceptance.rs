//! TASK-840: SPEC-061 diagnostics and acceptance/non-regression matrix.
//!
//! This suite aggregates the typechecker/normalizer acceptance matrix that is not
//! already owned directly by the parser/core/engine focused suites. Individual
//! tests intentionally assert stable diagnostic substrings for the SPEC-061 §14
//! diagnostic families while staying within the module-local SPEC-E scope.

use ash_core::ast::{TypeBody, TypeDef, Visibility as CoreVisibility};
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    DomainConstructorId, DomainConstructorSummary, DomainFieldSummary, ModuleIdentity,
    ModuleSemanticSummary, ModuleSourceOrigin, SealedDomainId, SealedDomainSummary, SourceAnchor,
    SourceOrigin, SummaryVersion,
};
use ash_core::type_ir::{NormalFormBlockReason, NormalTypeExpr, TypeComputationHeadId};
use ash_parser::surface::Definition;
use ash_typeck::TypeEnv;
use ash_typeck::normalizer::Normalizer;

fn module_identity(id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(840)),
        ModuleId(id),
        vec!["task840".to_string(), format!("m{id}")],
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-840-{id}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-840-test".into(),
        },
        None,
        label,
    )
}

fn type_list_domain(
    module: &ModuleIdentity,
    name: &str,
    nil: &str,
    cons: &str,
) -> SealedDomainSummary {
    let domain = SealedDomainId::new(module.clone(), name);
    let nil_summary = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), nil),
        nil,
        vec![],
        anchor(nil),
    );
    let cons_summary = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), cons),
        cons,
        vec![
            DomainFieldSummary::unconstrained("head"),
            DomainFieldSummary::constrained_to("tail", &domain, domain.clone()),
        ],
        anchor(cons),
    );
    SealedDomainSummary::new(domain, name, CoreVisibility::Public, anchor(name))
        .with_constructor(nil_summary)
        .with_constructor(cons_summary)
}

fn lower_list_domain(module: &ModuleIdentity) -> SealedDomainSummary {
    type_list_domain(module, "LowerList", "nil", "cons")
}

fn bool_domain(module: &ModuleIdentity) -> SealedDomainSummary {
    let domain = SealedDomainId::new(module.clone(), "BoolDomain");
    let true_summary = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), "True"),
        "True",
        vec![],
        anchor("True"),
    );
    let false_summary = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), "False"),
        "False",
        vec![],
        anchor("False"),
    );
    SealedDomainSummary::new(
        domain,
        "BoolDomain",
        CoreVisibility::Public,
        anchor("BoolDomain"),
    )
    .with_constructor(true_summary)
    .with_constructor(false_summary)
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
        .with_exported_sealed_domain(type_list_domain(module, "TypeList", "Nil", "Cons"))
        .with_exported_sealed_domain(type_list_domain(
            module,
            "OtherList",
            "OtherNil",
            "OtherCons",
        ))
        .with_exported_sealed_domain(lower_list_domain(module))
        .with_exported_sealed_domain(bool_domain(module))
        .with_exported_sealed_domain(flat_domain(module));
    summary.version = SummaryVersion::SPEC059_SEALED_DOMAIN_V2;
    env.register_module_semantic_summary(&summary)
        .expect("domains register");
}

fn type_fns(source: &str) -> Vec<ash_parser::surface::TypeFnDef> {
    let parsed = ash_parser::parse_surface_file(source).expect("source parses");
    parsed
        .definitions
        .into_iter()
        .filter_map(|def| match def {
            Definition::TypeFn(type_fn) => Some(type_fn),
            _ => None,
        })
        .collect()
}

fn env_with_defs(id: usize, source: &str) -> TypeEnv {
    let module = module_identity(id);
    let mut env = TypeEnv::new();
    register_domains(&mut env, &module);
    let defs = type_fns(source);
    env.register_local_type_functions(&module, &defs)
        .expect("definitions should register");
    env
}

fn assert_rejects(source: &str, expected: &str) {
    let module = module_identity(source.len() + expected.len());
    let mut env = TypeEnv::new();
    register_domains(&mut env, &module);
    let defs = type_fns(source);
    let err = env
        .register_local_type_functions(&module, &defs)
        .expect_err("definition should reject");
    let actual = format!("{err}");
    assert!(
        actual.contains(expected),
        "expected diagnostic containing {expected:?}, got {actual}"
    );
}

fn head(env: &TypeEnv, name: &str) -> TypeComputationHeadId {
    env.lookup_local_type_function(name)
        .expect("type function exists")
        .head
        .clone()
}

fn domain(module: &ModuleIdentity, name: &str) -> SealedDomainId {
    SealedDomainId::new(module.clone(), name)
}

fn ctor(module: &ModuleIdentity, domain_name: &str, ctor_name: &str) -> DomainConstructorId {
    DomainConstructorId::new(domain(module, domain_name), ctor_name)
}

fn nil(module: &ModuleIdentity) -> NormalTypeExpr {
    NormalTypeExpr::DomainConstructorApp {
        constructor: ctor(module, "TypeList", "Nil"),
        domain: domain(module, "TypeList"),
        args: vec![],
        kind: Kind::Type,
    }
}

fn cons(module: &ModuleIdentity, head: NormalTypeExpr, tail: NormalTypeExpr) -> NormalTypeExpr {
    NormalTypeExpr::DomainConstructorApp {
        constructor: ctor(module, "TypeList", "Cons"),
        domain: domain(module, "TypeList"),
        args: vec![head, tail],
        kind: Kind::Type,
    }
}

fn bool_ctor(module: &ModuleIdentity, name: &str) -> NormalTypeExpr {
    NormalTypeExpr::DomainConstructorApp {
        constructor: ctor(module, "BoolDomain", name),
        domain: domain(module, "BoolDomain"),
        args: vec![],
        kind: Kind::Type,
    }
}

fn var(name: &str) -> NormalTypeExpr {
    NormalTypeExpr::Var(name.to_string())
}

fn prim(name: &str) -> NormalTypeExpr {
    NormalTypeExpr::Primitive(name.to_string())
}

#[test]
fn named_spec061_diagnostic_families_have_stable_rejection_evidence() {
    let cases = [
        (
            "TypeFunctionNoSealedScrutinee",
            "type fn F(x: Type) -> Type { case F<x> = x; }",
            "type function 'F' has no sealed-domain scrutinee",
        ),
        (
            "TypePatternUnknownConstructor",
            "type fn F(xs: TypeList) -> TypeList { case F<Missing> = Nil; }",
            "unknown marker constructor 'Missing'",
        ),
        (
            "TypePatternWrongDomain",
            "type fn F(xs: TypeList) -> TypeList { case F<OtherNil> = Nil; }",
            "marker constructor 'OtherNil' belongs to sealed domain 'OtherList'",
        ),
        (
            "TypePatternRepeatedVariable",
            "type fn F(xs: TypeList, ys: TypeList) -> TypeList { case F<x, x> = x; }",
            "repeated type pattern variable 'x'",
        ),
        (
            "TypeFunctionNonExhaustive",
            "type fn Head(xs: TypeList) -> Type { case Head<Cons<h, t>> = h; }",
            "non-exhaustive type function 'Head'",
        ),
        (
            "TypeFunctionOverlappingEquation",
            r#"type fn F(xs: TypeList) -> TypeList {
                case F<Nil> = Nil;
                case F<Nil> = Nil;
                case F<Cons<h, t>> = t;
            }"#,
            "overlapping type function equation",
        ),
        (
            "TypeFunctionUnreachableEquation",
            r#"type fn F(xs: TypeList) -> TypeList {
                case F<_> = Nil;
                case F<Cons<h, t>> = t;
            }"#,
            "unreachable type function equation",
        ),
        (
            "TypeFunctionEmptyDefault",
            r#"type fn F(xs: TypeList) -> TypeList {
                case F<Nil> = Nil;
                case F<Cons<h, t>> = t;
                case F<_> = Nil;
            }"#,
            "empty residual default",
        ),
        (
            "TypeFunctionMissingDecreases",
            r#"type fn Append(xs: TypeList, ys: TypeList) -> TypeList {
                case Append<Nil, ys> = ys;
                case Append<Cons<h, t>, ys> = Cons<h, Append<t, ys>>;
            }"#,
            "missing decreases clause for recursive type function 'Append'",
        ),
        (
            "TypeFunctionInvalidDecreases",
            "type fn F(xs: TypeList) -> TypeList decreases missing { case F<xs> = xs; }",
            "unknown decreases parameter 'missing' in type function 'F'",
        ),
        (
            "TypeFunctionNonDecreasingRecursion",
            r#"type fn Bad(xs: TypeList) -> TypeList decreases xs {
                case Bad<xs> = Bad<xs>;
            }"#,
            "non-decreasing recursive call in type function 'Bad'",
        ),
        (
            "TypeFunctionResultDomainMismatch",
            "type fn F(xs: TypeList) -> TypeList { case F<Cons<h, t>> = h; }",
            "result domain mismatch: expected sealed domain 'TypeList'",
        ),
        (
            "TypeFunctionForwardReferenceUnsupported",
            r#"type fn UseLater(xs: TypeList) -> TypeList { case UseLater<xs> = Later<xs>; }
               type fn Later(xs: TypeList) -> TypeList { case Later<xs> = xs; }"#,
            "forward reference to later type function 'Later'",
        ),
    ];

    for (family, source, expected) in cases {
        assert_rejects(source, expected);
        assert!(!family.is_empty(), "SPEC-061 family name should be cited");
    }
}

#[test]
fn ambiguous_heads_and_marker_constructors_reject_with_spec061_families() {
    let module = module_identity(901);
    let mut env = TypeEnv::new();
    register_domains(&mut env, &module);
    env.register_type(&TypeDef {
        name: "Id".to_string(),
        params: vec!["T".to_string()],
        body: TypeBody::Struct(vec![]),
        visibility: CoreVisibility::Public,
        builtin: false,
    })
    .expect("nominal type registers");
    let defs = type_fns(
        r#"
        type fn Id(xs: TypeList) -> TypeList { case Id<xs> = xs; }
        type fn Use(xs: TypeList) -> TypeList { case Use<xs> = Id<xs>; }
        "#,
    );
    let err = env
        .register_local_type_functions(&module, &defs)
        .expect_err("ambiguous nominal/type-function head rejects");
    assert!(format!("{err}").contains("ambiguous type-function/type head 'Id'"));

    let module = module_identity(902);
    let mut env = TypeEnv::new();
    register_domains(&mut env, &module);
    env.register_type(&TypeDef {
        name: "Nil".to_string(),
        params: vec![],
        body: TypeBody::Struct(vec![]),
        visibility: CoreVisibility::Public,
        builtin: false,
    })
    .expect("nominal marker-name type registers");
    let defs = type_fns("type fn F(xs: TypeList) -> TypeList { case F<xs> = Nil; }");
    let err = env
        .register_local_type_functions(&module, &defs)
        .expect_err("ambiguous marker constructor rejects");
    assert!(format!("{err}").contains("ambiguous marker constructor 'Nil'"));
}

#[test]
fn rhs_pattern_variables_are_scoped_and_substituted_while_unknown_rhs_variables_reject() {
    assert_rejects(
        "type fn F(xs: TypeList) -> TypeList { case F<xs> = missing; }",
        "unknown RHS type variable 'missing'",
    );

    let module = module_identity(903);
    let source = r#"
        type fn Append(xs: TypeList, ys: TypeList) -> TypeList decreases xs {
            case Append<Nil, ys> = ys;
            case Append<Cons<h, t>, ys> = Cons<h, Append<t, ys>>;
        }
    "#;
    let env = env_with_defs(903, source);
    let result = Normalizer::new(&env)
        .normalize_known_computation_app(
            &head(&env, "Append"),
            vec![
                cons(&module, prim("A"), nil(&module)),
                cons(&module, prim("B"), nil(&module)),
            ],
            &Kind::Type,
        )
        .expect("normalization succeeds");
    assert_eq!(
        result,
        cons(&module, prim("A"), cons(&module, prim("B"), nil(&module)))
    );
}

#[test]
fn default_rows_reduce_known_residual_constructors_but_not_abstract_scrutinees() {
    let module = module_identity(904);
    let source = r#"
        type fn NilTag(xs: TypeList) -> BoolDomain {
            case NilTag<Nil> = True;
            case NilTag<_> = False;
        }
    "#;
    let env = env_with_defs(904, source);
    let normalizer = Normalizer::new(&env);
    let head = head(&env, "NilTag");

    assert_eq!(
        normalizer
            .normalize_known_computation_app(
                &head,
                vec![cons(&module, prim("A"), nil(&module))],
                &Kind::Type,
            )
            .expect("known residual Cons reduces through default"),
        bool_ctor(&module, "False")
    );
    assert_eq!(
        normalizer
            .normalize_known_computation_app(&head, vec![var("Xs")], &Kind::Type)
            .expect("abstract scrutinee stays neutral"),
        NormalTypeExpr::NeutralComputationApp {
            head,
            args: vec![var("Xs")],
            kind: Kind::Type,
            reason: NormalFormBlockReason::AbstractScrutinee,
        }
    );
}

#[test]
fn accepts_nested_defaults_positive_multiple_defaults_and_lowercase_marker_disambiguation() {
    let source = r#"
        type fn TailKind(xs: TypeList) -> TypeList {
            case TailKind<Nil> = Nil;
            case TailKind<Cons<h, Nil>> = Nil;
            case TailKind<Cons<h, _>> = Nil;
        }
        type fn Multi(xs: TypeList, ys: TypeList) -> TypeList {
            case Multi<Nil, _> = Nil;
            case Multi<_, Nil> = Nil;
            case Multi<_, _> = Nil;
        }
        type fn Lower(xs: LowerList) -> LowerList {
            case Lower<nil> = nil;
            case Lower<cons<h, t>> = t;
        }
    "#;
    let _env = env_with_defs(905, source);
}

#[test]
fn rejects_recursive_negative_matrix_including_nested_calls_and_computed_arguments() {
    assert_rejects(
        r#"
        type fn Bad(xs: TypeList) -> TypeList decreases xs {
            case Bad<Nil> = Nil;
            case Bad<Cons<h, t>> = Bad<Cons<h, t>>;
        }
        "#,
        "non-decreasing recursive call in type function 'Bad'",
    );
    assert_rejects(
        r#"
        type fn Id(xs: TypeList) -> TypeList { case Id<xs> = xs; }
        type fn Bad(xs: TypeList) -> TypeList decreases xs {
            case Bad<Nil> = Nil;
            case Bad<Cons<h, t>> = Bad<Id<t>>;
        }
        "#,
        "non-decreasing recursive call in type function 'Bad'",
    );
    assert_rejects(
        r#"
        type fn Bad(xs: TypeList) -> TypeList decreases xs {
            case Bad<Nil> = Nil;
            case Bad<Cons<h, t>> = Cons<h, Cons<h, Bad<Cons<h, t>>>>;
        }
        "#,
        "non-decreasing recursive call in type function 'Bad'",
    );
    assert_rejects(
        r#"
        type fn A(xs: TypeList) -> TypeList decreases xs {
            case A<Nil> = Nil;
            case A<Cons<h, t>> = B<t>;
        }
        type fn B(xs: TypeList) -> TypeList decreases xs {
            case B<Nil> = Nil;
            case B<Cons<h, t>> = A<t>;
        }
        "#,
        "forward reference to later type function 'B'",
    );
}

#[test]
fn invalid_decreases_requires_sealed_structural_parameter_metadata() {
    assert_rejects(
        r#"
        type fn Bad(x: Type, xs: TypeList) -> Type decreases x {
            case Bad<x, Nil> = Int;
            case Bad<x, Cons<h, t>> = Bad<x, t>;
        }
        "#,
        "invalid decreases parameter 'x' in type function 'Bad'",
    );
    assert_rejects(
        r#"
        type fn F(xs: Flat) -> Flat decreases xs {
            case F<Z> = Z;
            case F<S<x>> = Z;
        }
        "#,
        "invalid decreases parameter 'xs' in type function 'F'",
    );
}
