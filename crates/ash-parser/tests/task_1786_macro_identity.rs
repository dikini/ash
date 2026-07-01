use ash_parser::surface::{
    CallableDeclarationKind, MacroIdentityOrigin, collect_local_callable_identities,
    collect_local_macro_identities, collect_public_macro_summaries, resolve_local_macro_identity,
};

#[test]
fn macro_identity_is_distinct_from_same_named_callable_identity() {
    let source = "pub fn id(x: Int) -> Int { x }\npub macro id(x: Int) => x;";
    let module = ash_parser::parse_surface_file(source).expect("parse ok");

    let macros = collect_local_macro_identities(&module).expect("macro table ok");
    let callables = collect_local_callable_identities(&module);

    assert_eq!(macros.len(), 1);
    assert_eq!(callables.len(), 1);
    assert_eq!(macros[0].local_name.as_ref(), "id");
    assert_eq!(callables[0].name.as_ref(), "id");
    assert_eq!(callables[0].kind, CallableDeclarationKind::Function);
    assert_ne!(macros[0].origin_span, callables[0].origin_span);
}

#[test]
fn public_macro_summary_carries_importable_origin_identity() {
    let source = "pub macro inc(x: Int) => x + 1;";
    let module = ash_parser::parse_surface_file(source).expect("parse ok");

    let summaries = collect_public_macro_summaries(&module, "math").expect("summaries ok");

    assert_eq!(summaries.len(), 1);
    let summary = &summaries[0];
    assert_eq!(summary.identity.local_name.as_ref(), "inc");
    assert_eq!(summary.identity.param_count, 1);
    match &summary.identity.origin {
        MacroIdentityOrigin::Imported {
            module_path,
            exported_name,
        } => {
            assert_eq!(module_path.as_ref(), "math");
            assert_eq!(exported_name.as_ref(), "inc");
        }
        MacroIdentityOrigin::Local => panic!("public summaries use importable origin identity"),
    }
}

#[test]
fn imported_macro_identity_preserves_origin_under_alias() {
    let source = "pub macro inc(x: Int) => x + 1;";
    let module = ash_parser::parse_surface_file(source).expect("parse ok");
    let summary = collect_public_macro_summaries(&module, "math")
        .expect("summaries ok")
        .pop()
        .expect("one summary");

    let aliased =
        ash_parser::surface::MacroDeclarationIdentity::imported(&summary, "plus_one".into());

    assert_eq!(aliased.local_name.as_ref(), "plus_one");
    match aliased.origin {
        MacroIdentityOrigin::Imported {
            module_path,
            exported_name,
        } => {
            assert_eq!(module_path.as_ref(), "math");
            assert_eq!(exported_name.as_ref(), "inc");
        }
        MacroIdentityOrigin::Local => panic!("aliased summary identity must remain imported"),
    }
}

#[test]
fn local_macro_identity_resolution_fails_closed_for_absent_names() {
    let module = ash_parser::parse_surface_file("macro id(x) => x;").expect("parse ok");

    let found = resolve_local_macro_identity(&module, "id")
        .expect("macro table ok")
        .expect("id resolves");
    assert_eq!(found.local_name.as_ref(), "id");
    assert!(
        resolve_local_macro_identity(&module, "missing")
            .expect("macro table ok")
            .is_none()
    );
}
