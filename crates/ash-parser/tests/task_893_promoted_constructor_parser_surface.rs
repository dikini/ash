use ash_parser::surface::{DataKindDef, Definition, Visibility};

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source).expect("module file should parse")
}

fn data_kinds(module: &ash_parser::surface::ModuleFile) -> Vec<&DataKindDef> {
    module
        .definitions
        .iter()
        .filter_map(|definition| match definition {
            Definition::DataKind(data_kind) => Some(data_kind),
            _ => None,
        })
        .collect()
}

fn assert_nonempty_span(span: ash_parser::token::Span) {
    assert!(
        span.end > span.start,
        "expected non-empty span, got {span:?}"
    );
}

#[test]
fn task_893_parses_private_data_kind_declaration() {
    let module = parse("data kind NatKind from type Nat;");

    let kinds = data_kinds(&module);
    assert_eq!(kinds.len(), 1);
    let kind = kinds[0];
    assert_eq!(kind.visibility, Visibility::Inherited);
    assert_eq!(kind.name.as_ref(), "NatKind");
    assert_eq!(kind.source_adt.as_ref(), "Nat");
    assert_eq!(kind.span.start, 0);
    assert_eq!(kind.span.end, "data kind NatKind from type Nat;".len());
    assert_nonempty_span(kind.span);
}

#[test]
fn task_893_parses_public_data_kind_declaration() {
    let module = parse("pub data kind PublicNat from type Nat;");

    let kinds = data_kinds(&module);
    assert_eq!(kinds.len(), 1);
    let kind = kinds[0];
    assert_eq!(kind.visibility, Visibility::Public);
    assert_eq!(kind.name.as_ref(), "PublicNat");
    assert_eq!(kind.source_adt.as_ref(), "Nat");
    assert_eq!(kind.span.start, 0);
    assert_eq!(
        kind.span.end,
        "pub data kind PublicNat from type Nat;".len()
    );
    assert_nonempty_span(kind.span);
}

#[test]
fn task_893_parses_multiple_data_kind_declarations_in_module_file() {
    let module = parse(
        r#"type Nat = Z | S(Nat);
data kind PrivateNat from type Nat;
pub data kind ExportedNat from type Nat;
fn id(x: Int) -> Int { x }"#,
    );

    assert_eq!(module.definitions.len(), 4);
    let kinds = data_kinds(&module);
    assert_eq!(kinds.len(), 2);
    assert_eq!(kinds[0].visibility, Visibility::Inherited);
    assert_eq!(kinds[0].name.as_ref(), "PrivateNat");
    assert_eq!(kinds[0].source_adt.as_ref(), "Nat");
    assert_eq!(kinds[1].visibility, Visibility::Public);
    assert_eq!(kinds[1].name.as_ref(), "ExportedNat");
    assert_eq!(kinds[1].source_adt.as_ref(), "Nat");
}

#[test]
fn task_893_rejects_unsupported_data_kind_shorthand_forms() {
    for source in [
        "data kind Nat;",
        "data kind Nat from Nat;",
        "pub data kind Nat = type Nat;",
    ] {
        assert!(
            ash_parser::parse_surface_file(source).is_err(),
            "unsupported promoted-data-kind shorthand should be rejected: {source}"
        );
    }
}

#[test]
fn task_893_rejects_attribute_promotion_shorthand() {
    assert!(
        ash_parser::parse_surface_file("@promote\npub type Nat = Z | S(Nat);").is_err(),
        "attribute promotion shorthand must be rejected instead of silently skipped"
    );
}
