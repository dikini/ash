use std::collections::HashSet;

use ash_core::semantic_summary::TypeDeclId;
use ash_core::{
    CanonicalTypeExpr, ConstructorVariableApp, ConstructorVariableRef, Kind, KindedTypeBinder,
    SourceAnchor, SourceOrigin,
};

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "TASK-905 HKT core carrier test".to_string(),
        },
        None,
        label,
    )
}

#[test]
fn task_905_kinded_type_binder_preserves_name_kind_anchor_and_bounds() {
    let binder = KindedTypeBinder::new("M", Kind::n_ary(1), Some(anchor("M binder")), Vec::new());

    assert_eq!(binder.name, "M");
    assert_eq!(binder.kind, Kind::n_ary(1));
    assert_eq!(
        binder.source_anchor.as_ref().map(|a| a.label.as_str()),
        Some("M binder")
    );
    assert!(binder.bounds.is_empty());
}

#[test]
fn task_905_constructor_variable_application_is_not_nominal_application() {
    let binder = KindedTypeBinder::new("M", Kind::n_ary(1), Some(anchor("M binder")), Vec::new());
    let constructor = ConstructorVariableRef::from_binder(&binder);
    let app = CanonicalTypeExpr::ConstructorVariableApp(Box::new(ConstructorVariableApp::new(
        constructor,
        vec![CanonicalTypeExpr::Primitive("Int".to_string())],
        Kind::Type,
        Some(anchor("M<Int>")),
    )));

    let nominal_m_app = CanonicalTypeExpr::NominalApp {
        origin: TypeDeclId::ordinary(
            ash_core::ModuleIdentity::new(
                None,
                ash_core::module_graph::ModuleId(905),
                vec!["task_905".to_string()],
                ash_core::ModuleSourceOrigin::Synthetic {
                    reason: "TASK-905 nominal contrast".to_string(),
                },
            ),
            "M",
        ),
        visible_name: "M".to_string(),
        args: vec![CanonicalTypeExpr::Primitive("Int".to_string())],
        kind: Kind::Type,
    };

    assert_ne!(app, nominal_m_app);

    let mut set = HashSet::new();
    set.insert(app.clone());
    set.insert(nominal_m_app);
    assert_eq!(set.len(), 2);

    let CanonicalTypeExpr::ConstructorVariableApp(app) = app else {
        panic!("expected constructor-variable app");
    };
    assert_eq!(app.constructor.name, "M");
    assert_eq!(app.constructor.kind, Kind::n_ary(1));
    assert_eq!(
        app.args,
        vec![CanonicalTypeExpr::Primitive("Int".to_string())]
    );
    assert_eq!(app.kind, Kind::Type);
}
