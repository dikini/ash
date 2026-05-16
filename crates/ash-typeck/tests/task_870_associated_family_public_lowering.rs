use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{ModuleIdentity, ModuleSourceOrigin};
use ash_core::type_ir::{CanonicalTypeExpr, NormalTypeExpr, ProjectionRigidity};
use ash_parser::surface::{
    AssociatedTypeBinding, AssociatedTypeDecl, AssociatedTypeKind, Expr, FnDef, ImplDef,
    InterfaceDef, InterfaceTypeParam, Literal, Param, Type as SurfaceType, Visibility,
};
use ash_parser::token::Span;
use ash_typeck::normalizer::{NormalizationEvidence, Normalizer};
use ash_typeck::{Type, TypeEnv, fn_signature_type};

fn span() -> Span {
    Span::default()
}

fn module(name: &str, id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(870)),
        ModuleId(id),
        vec!["task870".into(), name.into()],
        ModuleSourceOrigin::Synthetic {
            reason: format!("TASK-870 {name}"),
        },
    )
}

fn param(name: &str) -> InterfaceTypeParam {
    InterfaceTypeParam {
        name: name.into(),
        domain: None,
        kind: None,
        span: span(),
    }
}

fn name_ty(name: &str) -> SurfaceType {
    SurfaceType::Name(name.into())
}

fn list_ty(item: SurfaceType) -> SurfaceType {
    SurfaceType::Constructor {
        name: "List".into(),
        args: vec![item],
    }
}

fn iterator_app(arg: SurfaceType) -> SurfaceType {
    SurfaceType::Constructor {
        name: "Iterator".into(),
        args: vec![arg],
    }
}

fn explicit_iterator_item(arg: SurfaceType) -> SurfaceType {
    SurfaceType::AssociatedFamilyProjection {
        interface: "Iterator".into(),
        args: vec![arg],
        member: "Item".into(),
        span: span(),
    }
}

fn compat_iterator_item(arg: SurfaceType) -> SurfaceType {
    SurfaceType::Associated {
        base: Box::new(iterator_app(arg)),
        name: "Item".into(),
    }
}

fn iterator_interface_def() -> InterfaceDef {
    InterfaceDef {
        visibility: Visibility::Inherited,
        name: "Iterator".into(),
        type_params: vec![param("Self")],
        associated_types: vec![AssociatedTypeDecl {
            name: "Item".into(),
            kind: AssociatedTypeKind::SealedFamily {
                result_domain: SurfaceType::Name("Type".into()),
                decreases: None,
                span: span(),
            },
            span: span(),
        }],
        methods: vec![],
        span: span(),
    }
}

fn iterator_list_impl(param_name: &str) -> ImplDef {
    ImplDef {
        visibility: Visibility::Inherited,
        interface: "Iterator".into(),
        type_params: vec![param(param_name)],
        type_args: vec![list_ty(name_ty(param_name))],
        where_bounds: vec![],
        associated_type_bindings: vec![AssociatedTypeBinding {
            name: "Item".into(),
            ty: name_ty(param_name),
            span: span(),
        }],
        methods: vec![],
        span: span(),
    }
}

fn env_with_iterator_family() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(module("iterator", 1));
    env.register_interface(&iterator_interface_def())
        .expect("Iterator associated family registers");
    env.register_impl(&iterator_list_impl("A"))
        .expect("Iterator<List<A>>::Item scheme registers");
    env
}

#[test]
fn task_870_explicit_family_projection_is_accepted_in_public_fn_signature_positions() {
    let env = env_with_iterator_family();
    let projected = explicit_iterator_item(list_ty(name_ty("String")));
    let function = FnDef {
        visibility: Visibility::Public,
        name: "projected".into(),
        type_params: vec![],
        params: vec![Param {
            name: "value".into(),
            ty: projected.clone(),
        }],
        return_type: Some(projected),
        proposition_tail: None,
        contract: None,
        body: Expr::Literal(Literal::String("ok".into())),
        span: span(),
    };

    let signature = fn_signature_type(&env, &function)
        .expect("explicit associated-family projection must lower in public fn signatures");
    let Type::Fn(params, ret) = signature else {
        panic!("expected fn signature, got {signature:?}");
    };
    assert_eq!(params.len(), 1);
    assert_eq!(params[0], *ret);
    match ret.as_ref() {
        Type::Associated {
            interface,
            base,
            name,
        } => {
            assert_eq!(interface, "Iterator");
            assert_eq!(name, "Item");
            match base.as_ref() {
                Type::Constructor { name, args, kind } => {
                    assert_eq!(name.name, "Iterator");
                    assert_eq!(args.len(), 1);
                    assert_eq!(kind, &Kind::Type);
                }
                other => panic!(
                    "expected explicit projection base encoded as Iterator<...>, got {other:?}"
                ),
            }
        }
        other => panic!("expected associated projection return type, got {other:?}"),
    }
}

#[test]
fn task_870_explicit_family_projection_is_accepted_in_type_env_surface_type_positions() {
    let mut env = env_with_iterator_family();
    let provider = InterfaceDef {
        visibility: Visibility::Public,
        name: "Provider".into(),
        type_params: vec![],
        associated_types: vec![],
        methods: vec![ash_parser::surface::InterfaceMethodSig {
            name: "next".into(),
            params: vec![explicit_iterator_item(list_ty(name_ty("String")))],
            return_type: explicit_iterator_item(list_ty(name_ty("String"))),
            span: span(),
        }],
        span: span(),
    };

    env.register_interface(&provider)
        .expect("explicit associated-family projection must lower in interface method types");
    let provider_info = env
        .lookup_interface("Provider")
        .expect("Provider interface registered");
    let method = provider_info
        .methods
        .get("next")
        .expect("method registered");
    assert_eq!(method.params.len(), 1);
    assert_eq!(method.params[0], method.return_type);
}

#[test]
fn task_870_compat_and_explicit_iterator_list_x_item_are_canonical_equivalent_and_reduce() {
    let env = env_with_iterator_family();
    let abstract_list = list_ty(name_ty("X"));

    let compat = env
        .lower_surface_type_to_canonical(&compat_iterator_item(abstract_list.clone()))
        .expect("SPEC-035 compatibility projection lowers");
    let explicit = env
        .lower_surface_type_to_canonical(&explicit_iterator_item(abstract_list))
        .expect("explicit associated-family projection lowers");

    assert_eq!(compat, explicit);
    let CanonicalTypeExpr::Projection { rigidity, .. } = &compat else {
        panic!("expected canonical projection, got {compat:?}");
    };
    assert_eq!(
        rigidity,
        &ProjectionRigidity::Neutral,
        "abstract nominal arguments must make compatibility projection rigidity match explicit lowering"
    );

    let outcome = Normalizer::new(&env)
        .normalize(&compat)
        .expect("Iterator<List<X>>::Item should reduce despite abstract element");
    assert_eq!(
        outcome.evidence,
        NormalizationEvidence::AssociatedFamilyProjectionReduced
    );
    assert_eq!(outcome.normal, NormalTypeExpr::Var("X".into()));
}
