//! TASK-835: type-function signature, kind, and domain validation.

use ash_core::ast::{TypeBody, TypeDef, Visibility as CoreVisibility};
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    DomainConstructorId, DomainConstructorSummary, DomainFieldSummary, ModuleIdentity,
    ModuleSemanticSummary, ModuleSourceOrigin, SealedDomainId, SealedDomainSummary, SourceAnchor,
    SourceOrigin, SummaryVersion,
};
use ash_core::type_ir::{TypeFunctionResultConstraint, TypeFunctionResultExpr};
use ash_parser::surface::Definition;
use ash_typeck::TypeEnv;

fn module_identity(id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(835)),
        ModuleId(id),
        vec!["task835".to_string(), format!("m{id}")],
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-835-{id}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-835-test".into(),
        },
        None,
        label,
    )
}

fn domain_summary(
    module: &ModuleIdentity,
    name: &str,
    nil: &str,
    cons: &str,
) -> SealedDomainSummary {
    let domain = SealedDomainId::new(module.clone(), name);
    let nil = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), nil),
        nil,
        vec![],
        anchor(nil),
    );
    let cons = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), cons),
        cons,
        vec![
            DomainFieldSummary::unconstrained("head"),
            DomainFieldSummary::constrained_to("tail", &domain, domain.clone()),
        ],
        anchor(cons),
    );
    SealedDomainSummary::new(domain, name, CoreVisibility::Public, anchor(name))
        .with_constructor(nil)
        .with_constructor(cons)
}

fn register_domains(env: &mut TypeEnv, module: &ModuleIdentity) {
    let mut summary = ModuleSemanticSummary::new(module.clone())
        .with_exported_sealed_domain(domain_summary(module, "TypeList", "Nil", "Cons"))
        .with_exported_sealed_domain(domain_summary(module, "OtherList", "OtherNil", "OtherCons"));
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

fn assert_rejects(source: &str, expected: &str) {
    let module = module_identity(expected.len());
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

#[test]
fn rejects_public_type_function_before_spec_f() {
    let module = module_identity(100);
    let mut env = TypeEnv::new();
    register_domains(&mut env, &module);
    let mut defs = type_fns("type fn F(xs: TypeList) -> TypeList { case F<xs> = xs; }");
    defs[0].visibility = ash_parser::surface::Visibility::Public;
    let err = env
        .register_local_type_functions(&module, &defs)
        .expect_err("public type fn rejects at typechecker boundary");
    assert!(
        format!("{err}").contains("cannot be public before SPEC-F summaries"),
        "unexpected diagnostic: {err}"
    );
}

#[test]
fn rejects_no_sealed_domain_scrutinee() {
    assert_rejects(
        "type fn F(x: Type) -> Type { case F<x> = x; }",
        "type function 'F' has no sealed-domain scrutinee",
    );
}

#[test]
fn rejects_signature_unknown_type() {
    assert_rejects(
        "type fn F(xs: MissingType) -> TypeList { case F<xs> = xs; }",
        "unresolved type in type-function signature 'MissingType'",
    );
}

#[test]
fn rejects_unknown_rhs_variable() {
    assert_rejects(
        "type fn F(xs: TypeList) -> TypeList { case F<xs> = missing; }",
        "unknown RHS type variable 'missing'",
    );
}

#[test]
fn rejects_repeated_pattern_variable() {
    assert_rejects(
        "type fn F(xs: TypeList, ys: TypeList) -> TypeList { case F<x, x> = x; }",
        "repeated type pattern variable 'x'",
    );
}

#[test]
fn rejects_unknown_pattern_constructor() {
    assert_rejects(
        "type fn F(xs: TypeList) -> TypeList { case F<Missing> = Nil; }",
        "unknown marker constructor 'Missing'",
    );
}

#[test]
fn rejects_wrong_domain_pattern_constructor() {
    assert_rejects(
        "type fn F(xs: TypeList) -> TypeList { case F<OtherNil> = Nil; }",
        "marker constructor 'OtherNil' belongs to sealed domain 'OtherList'",
    );
}

#[test]
fn rejects_wrong_constructor_arity() {
    assert_rejects(
        "type fn F(xs: TypeList) -> TypeList { case F<Cons<h>> = Nil; }",
        "pattern arity mismatch: expected 2, found 1",
    );
}

#[test]
fn rejects_wrong_type_function_application_arity() {
    assert_rejects(
        r#"
        type fn Id(xs: TypeList) -> TypeList { case Id<xs> = xs; }
        type fn Use(xs: TypeList) -> TypeList { case Use<xs> = Id<xs, xs>; }
        "#,
        "type function 'Id' application arity mismatch: expected 1, found 2",
    );
}

#[test]
fn rejects_type_function_application_wrong_domain_argument() {
    assert_rejects(
        r#"
        type fn Id(xs: TypeList) -> TypeList { case Id<xs> = xs; }
        type fn Use(xs: OtherList) -> TypeList { case Use<xs> = Id<xs>; }
        "#,
        "type function 'Id' argument 0 domain mismatch",
    );
}

#[test]
fn rejects_result_domain_mismatch_even_when_kind_is_type() {
    assert_rejects(
        "type fn F(xs: TypeList) -> TypeList { case F<Cons<h, t>> = h; }",
        "result domain mismatch: expected sealed domain 'TypeList'",
    );
}

#[test]
fn rejects_result_kind_mismatch_for_unsupported_surface_shape() {
    assert_rejects(
        "type fn F(xs: TypeList) -> Type { case F<xs> = (Int, String); }",
        "result kind mismatch",
    );
}

#[test]
fn rejects_nested_result_constructor_field_domain_mismatch() {
    assert_rejects(
        "type fn F(xs: TypeList) -> TypeList { case F<xs> = Cons<Int, Int>; }",
        "result constructor field 1 domain mismatch",
    );
}

#[test]
fn rejects_nested_result_constructor_field_from_wrong_sealed_domain() {
    assert_rejects(
        r#"
        type fn F(xs: TypeList, ys: OtherList) -> TypeList {
            case F<xs, ys> = Cons<Int, ys>;
        }
        "#,
        "result constructor field 1 domain mismatch",
    );
}

#[test]
fn rejects_forward_reference_but_accepts_earlier_validated_dependency() {
    assert_rejects(
        r#"
        type fn UseLater(xs: TypeList) -> TypeList { case UseLater<xs> = Later<xs>; }
        type fn Later(xs: TypeList) -> TypeList { case Later<xs> = xs; }
        "#,
        "forward reference to later type function 'Later'",
    );

    let module = module_identity(900);
    let mut env = TypeEnv::new();
    register_domains(&mut env, &module);
    let defs = type_fns(
        r#"
        type fn Id(xs: TypeList) -> TypeList { case Id<xs> = xs; }
        type fn UseId(xs: TypeList) -> TypeList { case UseId<xs> = Id<xs>; }
        "#,
    );
    env.register_local_type_functions(&module, &defs)
        .expect("earlier dependency is usable");
}

#[test]
fn rejects_ambiguous_nominal_and_type_function_head() {
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
        .unwrap_err();
    assert!(format!("{err}").contains("ambiguous type-function/type head 'Id'"));
}

#[test]
fn rejects_ambiguous_marker_constructor_vs_nominal_head() {
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
    .expect("nominal type registers");
    let defs = type_fns("type fn F(xs: TypeList) -> TypeList { case F<xs> = Nil; }");
    let err = env
        .register_local_type_functions(&module, &defs)
        .unwrap_err();
    assert!(format!("{err}").contains("ambiguous marker constructor 'Nil'"));
}

#[test]
fn rejects_ambiguous_marker_constructor_vs_nominal_head_in_pattern_position() {
    let module = module_identity(904);
    let mut env = TypeEnv::new();
    register_domains(&mut env, &module);
    env.register_type(&TypeDef {
        name: "Nil".to_string(),
        params: vec![],
        body: TypeBody::Struct(vec![]),
        visibility: CoreVisibility::Public,
        builtin: false,
    })
    .expect("nominal type registers");
    let defs = type_fns("type fn F(xs: TypeList) -> TypeList { case F<Nil> = Nil; }");
    let err = env
        .register_local_type_functions(&module, &defs)
        .unwrap_err();
    assert!(format!("{err}").contains("ambiguous marker constructor 'Nil'"));
}

#[test]
fn rejects_wrong_domain_marker_constructor_in_rhs_position() {
    assert_rejects(
        "type fn F(xs: TypeList) -> TypeList { case F<xs> = OtherNil; }",
        "marker constructor 'OtherNil' belongs to sealed domain 'OtherList'",
    );
}

#[test]
fn rejects_current_type_function_name_that_is_also_marker_constructor_in_rhs_position() {
    let module = module_identity(905);
    let mut env = TypeEnv::new();
    register_domains(&mut env, &module);
    let defs = type_fns("type fn Nil(xs: TypeList) -> TypeList { case Nil<xs> = Nil; }");
    let err = env
        .register_local_type_functions(&module, &defs)
        .unwrap_err();
    assert!(format!("{err}").contains("ambiguous marker constructor 'Nil'"));
}

#[test]
fn lowercase_pattern_variables_are_not_lowered_as_nominal_types() {
    let module = module_identity(903);
    let mut env = TypeEnv::new();
    register_domains(&mut env, &module);
    env.register_type(&TypeDef {
        name: "h".to_string(),
        params: vec![],
        body: TypeBody::Struct(vec![]),
        visibility: CoreVisibility::Public,
        builtin: false,
    })
    .expect("lowercase nominal type registers");
    let defs = type_fns(
        r#"
        type fn F(xs: TypeList) -> TypeList {
            case F<Nil> = Nil;
            case F<Cons<h, t>> = Cons<h, t>;
        }
        "#,
    );

    env.register_local_type_functions(&module, &defs)
        .expect("lowercase pattern variable wins over nominal type");
    let def = env.lookup_local_type_function("F").unwrap();
    match &def.equations[1].result {
        TypeFunctionResultExpr::DomainConstructorApp { args, .. } => {
            assert!(matches!(
                &args[0],
                TypeFunctionResultExpr::Var { name, constraint: TypeFunctionResultConstraint::Kind(Kind::Type), .. } if name == "h"
            ));
        }
        other => panic!("expected Cons result, got {other:?}"),
    }
}
