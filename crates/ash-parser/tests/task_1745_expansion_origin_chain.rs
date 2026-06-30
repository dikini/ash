use ash_parser::surface::{ExpansionId, SurfaceOrigin, expand_surface_module};

#[test]
fn expanded_origins_receive_stable_distinct_expansion_ids() {
    let module = ash_parser::parse_surface_file(
        r#"
        fn first() -> Int { (+) }
        fn second() -> Int { (*) }
        "#,
    )
    .expect("sections parse");

    let expanded = expand_surface_module(module).expect("built-in sections elaborate");

    assert_eq!(expanded.origins.len(), 2);
    assert_eq!(expanded.origins[0].expansion_id, ExpansionId(0));
    assert_eq!(expanded.origins[1].expansion_id, ExpansionId(1));
    assert_ne!(
        expanded.origins[0].expansion_id,
        expanded.origins[1].expansion_id
    );
    assert!(
        expanded
            .origins
            .iter()
            .all(|origin| origin.parent.is_none())
    );
}

#[test]
fn local_notation_origin_records_expansion_id_and_target() {
    let module = ash_parser::parse_surface_file(
        r#"
        infixl 6 <+> = Math::combine
        fn section(x: Int) -> Int { (x <+>) }
        "#,
    )
    .expect("local notation section parses");

    let expanded = expand_surface_module(module).expect("local notation section elaborates");

    assert_eq!(expanded.origins.len(), 1);
    let origin = &expanded.origins[0];
    assert_eq!(origin.expansion_id, ExpansionId(0));
    assert!(origin.parent.is_none());
    assert!(matches!(
        &origin.origin,
        SurfaceOrigin::NotationExpansion {
            notation_span,
            target,
        } if target.as_ref() == "Math::combine" && notation_span.start < notation_span.end
    ));
}

#[test]
fn nested_expansion_origin_preserves_parent_chain() {
    let module = ash_parser::parse_surface_file(
        r#"
        infixl 6 <+> = combine
        fn section() -> Int { (<+> (+)) }
        "#,
    )
    .expect("nested section parses");

    let expanded = expand_surface_module(module).expect("nested sections elaborate");

    assert_eq!(expanded.origins.len(), 2);
    let outer = &expanded.origins[0];
    let inner = &expanded.origins[1];
    assert_eq!(outer.expansion_id, ExpansionId(0));
    assert_eq!(inner.expansion_id, ExpansionId(1));
    assert!(matches!(
        outer.origin,
        SurfaceOrigin::NotationExpansion { .. }
    ));
    assert!(matches!(
        inner.origin,
        SurfaceOrigin::OperatorSection { .. }
    ));
    assert!(matches!(
        inner.parent.as_deref(),
        Some(SurfaceOrigin::NotationExpansion { target, .. }) if target.as_ref() == "combine"
    ));
}

#[test]
fn nested_expansion_origin_survives_non_call_recursive_shapes() {
    let module = ash_parser::parse_surface_file(
        r#"
        infixl 6 <+> = combine
        fn section() -> Int { (<+> (1 + (+))) }
        "#,
    )
    .expect("nested binary section parses");

    let expanded = expand_surface_module(module).expect("nested binary sections elaborate");

    assert_eq!(expanded.origins.len(), 2);
    let outer = &expanded.origins[0];
    let inner = &expanded.origins[1];
    assert_eq!(outer.expansion_id, ExpansionId(0));
    assert_eq!(inner.expansion_id, ExpansionId(1));
    assert!(matches!(
        outer.origin,
        SurfaceOrigin::NotationExpansion { .. }
    ));
    assert!(matches!(
        inner.origin,
        SurfaceOrigin::OperatorSection { .. }
    ));
    assert!(matches!(
        inner.parent.as_deref(),
        Some(SurfaceOrigin::NotationExpansion { target, .. }) if target.as_ref() == "combine"
    ));
}
