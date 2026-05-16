use ash_parser::surface::{AssociatedTypeKind, Definition, InterfaceTypeParam, Type, TypeBody};

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module file should parse: {source}\nerrors: {errors:?}"))
}

fn parse_err(source: &str) {
    assert!(
        ash_parser::parse_surface_file(source).is_err(),
        "source should be rejected: {source}"
    );
}

fn parsed_alias_type(source: &str) -> Type {
    let module = parse(source);
    let type_def = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Type(type_def) => Some(type_def),
            _ => None,
        })
        .expect("type alias definition should be present");

    match &type_def.body {
        TypeBody::Alias(ty) => ty.clone(),
        other => panic!("expected type alias body, got {other:?}"),
    }
}

fn parsed_interface(source: &str) -> ash_parser::surface::InterfaceDef {
    let module = parse(source);
    module
        .definitions
        .into_iter()
        .find_map(|definition| match definition {
            Definition::Interface(interface) => Some(interface),
            _ => None,
        })
        .expect("interface definition should be present")
}

#[test]
fn task_859_associated_family_explicit_projection_type_syntax() {
    let ty = parsed_alias_type("type Projected = <Iterator<List<A>>>::Item;");

    let Type::AssociatedFamilyProjection {
        interface,
        args,
        member,
        span,
    } = ty
    else {
        panic!("expected explicit associated-family projection, got {ty:?}");
    };

    assert_eq!(interface.as_ref(), "Iterator");
    assert_eq!(member.as_ref(), "Item");
    assert!(
        span.end > span.start,
        "projection must preserve a source span"
    );
    assert_eq!(args.len(), 1);
    assert!(matches!(
        &args[0],
        Type::Constructor { name, args }
            if name.as_ref() == "List"
                && matches!(args.as_slice(), [Type::Name(name)] if name.as_ref() == "A")
    ));
}

#[test]
fn task_859_associated_family_nested_explicit_projection_type_syntax() {
    let ty = parsed_alias_type("type Projected = <Append<Cons<H, T>, Ys>>::Out;");

    let Type::AssociatedFamilyProjection {
        interface,
        args,
        member,
        span,
    } = ty
    else {
        panic!("expected explicit associated-family projection, got {ty:?}");
    };

    assert_eq!(interface.as_ref(), "Append");
    assert_eq!(member.as_ref(), "Out");
    assert!(
        span.end > span.start,
        "projection must preserve a source span"
    );
    assert_eq!(args.len(), 2);
    assert!(matches!(
        &args[0],
        Type::Constructor { name, args }
            if name.as_ref() == "Cons"
                && matches!(args.as_slice(), [Type::Name(head), Type::Name(tail)] if head.as_ref() == "H" && tail.as_ref() == "T")
    ));
    assert!(matches!(&args[1], Type::Name(name) if name.as_ref() == "Ys"));
}

#[test]
fn task_859_associated_family_keeps_spec035_compat_projection_syntax() {
    let ty = parsed_alias_type("type Projected = T::Item;");

    let Type::Associated { base, name } = ty else {
        panic!("expected existing compatibility associated projection, got {ty:?}");
    };

    assert_eq!(name.as_ref(), "Item");
    assert!(matches!(base.as_ref(), Type::Name(base) if base.as_ref() == "T"));
}

#[test]
fn task_859_associated_family_typed_interface_params_preserve_domains_and_spans() {
    let interface = parsed_interface(
        "interface Append<Xs: TypeList, Ys: TypeList> { append(Xs, Ys) -> TypeList }",
    );

    assert_eq!(interface.name.as_ref(), "Append");
    assert_eq!(interface.type_params.len(), 2);

    let [
        InterfaceTypeParam {
            name: xs,
            domain: Some(xs_domain),
            kind: None,
            span: xs_span,
            ..
        },
        InterfaceTypeParam {
            name: ys,
            domain: Some(ys_domain),
            kind: None,
            span: ys_span,
            ..
        },
    ] = interface.type_params.as_slice()
    else {
        panic!(
            "expected two domain-annotated interface params, got {:?}",
            interface.type_params
        );
    };

    assert_eq!(xs.as_ref(), "Xs");
    assert_eq!(ys.as_ref(), "Ys");
    assert_eq!(*xs_domain, Type::Name("TypeList".into()));
    assert_eq!(*ys_domain, Type::Name("TypeList".into()));
    assert!(xs_span.end > xs_span.start);
    assert!(ys_span.end > ys_span.start);
}

#[test]
fn task_859_associated_family_raw_sealed_family_item_decl_preserves_domain() {
    let interface = parsed_interface("interface Iterator<I> { sealed type family Item: Type }");

    assert_eq!(interface.associated_types.len(), 1);
    let member = &interface.associated_types[0];
    assert_eq!(member.name.as_ref(), "Item");
    assert!(member.span.end > member.span.start);

    let AssociatedTypeKind::SealedFamily {
        result_domain,
        decreases,
        span,
    } = &member.kind
    else {
        panic!("expected sealed associated-family member, got {member:?}");
    };

    assert_eq!(*result_domain, Type::Name("Type".into()));
    assert!(decreases.is_none());
    assert!(span.end > span.start);
}

#[test]
fn task_859_associated_family_raw_sealed_family_out_decl_preserves_domain_and_decreases() {
    let interface = parsed_interface(
        "interface Append<Xs: TypeList, Ys: TypeList> { sealed type family Out: TypeList decreases Xs }",
    );

    assert_eq!(interface.associated_types.len(), 1);
    let member = &interface.associated_types[0];
    assert_eq!(member.name.as_ref(), "Out");

    let AssociatedTypeKind::SealedFamily {
        result_domain,
        decreases: Some(decreases),
        span,
    } = &member.kind
    else {
        panic!("expected sealed associated-family member with decreases, got {member:?}");
    };

    assert_eq!(*result_domain, Type::Name("TypeList".into()));
    assert_eq!(decreases.param.as_ref(), "Xs");
    assert!(decreases.span.end > decreases.span.start);
    assert!(span.end > span.start);
}

#[test]
fn task_859_associated_family_missing_result_domain_is_rejected() {
    parse_err("interface Iterator<I> { sealed type family Item }");
    parse_err(
        "interface Append<Xs: TypeList, Ys: TypeList> { sealed type family Out decreases Xs }",
    );
}

#[test]
fn task_859_associated_family_rejects_unsupported_qualified_projection_heads() {
    parse_err("type Projected = <collections::Iterator<List<A>>>::Item;");
    parse_err("type Projected = <crate::Append<Cons<H, T>, Ys>>::Out;");
}

#[test]
fn task_859_associated_family_rejects_malformed_explicit_projection_forms() {
    parse_err("type Projected = <Iterator<List<A>>::Item;");
    parse_err("type Projected = <Iterator<List<A>>>Item;");
    parse_err("type Projected = <Iterator<List<A>>>::;");
    parse_err("type Projected = <>::Item;");
    parse_err("type Projected = <Append<Cons<H, T>, Ys>::Out;");
}

#[test]
fn task_859_associated_family_rejects_malformed_declaration_forms() {
    parse_err("interface Iterator<I> { sealed family Item: Type }");
    parse_err("interface Iterator<I> { sealed type Item: Type }");
    parse_err("interface Iterator<I> { sealed type family: Type }");
    parse_err("interface Iterator<I> { sealed type family Item Type }");
    parse_err("interface Iterator<I> { sealed type family Item: Type decreases }");
    parse_err("interface Iterator<I> { sealed type family Item: Type decreases Xs extra }");
    parse_err("interface Iterator<I> { type family Item: Type }");
}
