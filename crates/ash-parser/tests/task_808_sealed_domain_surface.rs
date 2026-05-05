use ash_parser::surface::{Definition, DomainSlot, SealedDomainDef, Visibility};

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source).expect("module file should parse")
}

fn sealed_domains(module: &ash_parser::surface::ModuleFile) -> Vec<&SealedDomainDef> {
    module
        .definitions
        .iter()
        .filter_map(|def| match def {
            Definition::SealedDomain(sd) => Some(sd),
            _ => None,
        })
        .collect()
}

// --- Acceptance tests ---

#[test]
fn parse_simple_sealed_domain_with_unit_constructors() {
    let source = "sealed type domain Boolish { Yes; No; }\nfn id(x: Int) -> Int { x }";
    let module = parse(source);

    let domains = sealed_domains(&module);
    assert_eq!(domains.len(), 1);

    let sd = &domains[0];
    assert_eq!(sd.visibility, Visibility::Inherited);
    assert_eq!(sd.name.as_ref(), "Boolish");
    assert_eq!(sd.constructors.len(), 2);
    assert_eq!(sd.constructors[0].name.as_ref(), "Yes");
    assert_eq!(sd.constructors[0].fields.len(), 0);
    assert_eq!(sd.constructors[1].name.as_ref(), "No");
    assert_eq!(sd.constructors[1].fields.len(), 0);
}

#[test]
fn parse_public_sealed_domain() {
    let source = "pub sealed type domain TypeList { Nil; Cons<head: Type, tail: TypeList>; }";
    let module = parse(source);

    let domains = sealed_domains(&module);
    assert_eq!(domains.len(), 1);

    let sd = &domains[0];
    assert_eq!(sd.visibility, Visibility::Public);
    assert_eq!(sd.name.as_ref(), "TypeList");
    assert_eq!(sd.constructors.len(), 2);

    // Nil - unit constructor
    assert_eq!(sd.constructors[0].name.as_ref(), "Nil");
    assert!(sd.constructors[0].fields.is_empty());

    // Cons - with fields
    let cons = &sd.constructors[1];
    assert_eq!(cons.name.as_ref(), "Cons");
    assert_eq!(cons.fields.len(), 2);

    // head: Type
    assert_eq!(cons.fields[0].name.as_ref(), "head");
    assert_eq!(cons.fields[0].slot, DomainSlot::Type);

    // tail: TypeList
    assert_eq!(cons.fields[1].name.as_ref(), "tail");
    assert_eq!(
        cons.fields[1].slot,
        DomainSlot::DomainRef("TypeList".into())
    );
}

#[test]
fn parse_sealed_domain_with_only_type_fields() {
    let source = "sealed type domain Pair { MkPair<first: Type, second: Type>; }";
    let module = parse(source);

    let domains = sealed_domains(&module);
    assert_eq!(domains.len(), 1);
    let sd = &domains[0];
    assert_eq!(sd.constructors.len(), 1);

    let ctor = &sd.constructors[0];
    assert_eq!(ctor.name.as_ref(), "MkPair");
    assert_eq!(ctor.fields.len(), 2);
    assert_eq!(ctor.fields[0].slot, DomainSlot::Type);
    assert_eq!(ctor.fields[1].slot, DomainSlot::Type);
}

#[test]
fn parse_sealed_domain_self_referencing() {
    let source = "sealed type domain Nat { Zero; Succ<pred: Nat>; }";
    let module = parse(source);

    let domains = sealed_domains(&module);
    assert_eq!(domains.len(), 1);
    let sd = &domains[0];
    assert_eq!(
        sd.constructors[1].fields[0].slot,
        DomainSlot::DomainRef("Nat".into())
    );
}

#[test]
fn parse_file_with_only_sealed_domain() {
    let source = "sealed type domain Unit { MkUnit; }";
    let module = parse(source);

    let domains = sealed_domains(&module);
    assert_eq!(domains.len(), 1);
    assert_eq!(module.definitions.len(), 1);
}

#[test]
fn parse_multiple_sealed_domains_in_same_file() {
    let source = r"sealed type domain Color { Red; Green; Blue; }
sealed type domain Shape { Circle; Square; }
fn foo() -> Int { 1 }";
    let module = parse(source);

    let domains = sealed_domains(&module);
    assert_eq!(domains.len(), 2);
    assert_eq!(domains[0].name.as_ref(), "Color");
    assert_eq!(domains[1].name.as_ref(), "Shape");
}

#[test]
fn parse_pub_crate_sealed_domain() {
    let source = "pub(crate) sealed type domain Internal { A; B; }";
    let module = parse(source);

    let domains = sealed_domains(&module);
    assert_eq!(domains.len(), 1);
    assert_eq!(domains[0].visibility, Visibility::Crate);
}

// --- Rejection boundary tests ---

#[test]
fn reject_generic_domain_parameters() {
    let source = "sealed type domain Bad<T> { X; }";
    let result = ash_parser::parse_surface_file(source);
    assert!(
        result.is_err(),
        "generic domain parameters should be rejected"
    );
}

#[test]
fn reject_per_constructor_visibility() {
    // Per-constructor visibility is not supported in first slice.
    // The parser sees `pub` as a visibility keyword and tries to parse
    // it as a definition start, which should fail or produce an error
    // since `pub Nil;` is not a valid constructor syntax.
    let source = "sealed type domain Bad { pub X; }";
    let result = ash_parser::parse_surface_file(source);
    assert!(
        result.is_err(),
        "per-constructor visibility should be rejected"
    );
}

#[test]
fn reject_inline_module_sealed_domains() {
    // Inline-module sealed domains are explicitly unsupported.
    let source = r#"mod inline_mod {
sealed type domain Bad { X; }
}"#;
    let result = ash_parser::parse_surface_file(source);
    assert!(
        result.is_err(),
        "inline-module sealed domains should be rejected by the parser"
    );
}

// --- Non-interference tests ---

#[test]
fn ordinary_type_definition_still_parses_alongside_sealed_domain() {
    let source = r"pub type Option<T> = Some { value: T } | None;
sealed type domain Direction { North; South; East; West; }
fn go(d: Direction) -> Int { 1 }";
    let module = parse(source);

    // Check ordinary type
    let types: Vec<_> = module
        .definitions
        .iter()
        .filter_map(|d| match d {
            Definition::Type(t) => Some(t.name.as_ref().to_string()),
            _ => None,
        })
        .collect();
    assert!(types.contains(&"Option".to_string()));

    // Check sealed domain
    let domains = sealed_domains(&module);
    assert_eq!(domains.len(), 1);
    assert_eq!(domains[0].constructors.len(), 4);
}

#[test]
fn sealed_domain_has_source_spans() {
    let source = "sealed type domain Spanned { X; Y<z: Type>; }";
    let module = parse(source);

    let domains = sealed_domains(&module);
    let sd = &domains[0];

    // Spans should be non-trivial
    assert!(
        sd.span.end > sd.span.start,
        "domain span should be non-trivial"
    );
    assert!(
        sd.constructors[0].span.end > sd.constructors[0].span.start,
        "constructor span should be non-trivial"
    );
    assert!(
        sd.constructors[1].span.end > sd.constructors[1].span.start,
        "constructor span should be non-trivial"
    );
}

#[test]
fn sealed_domain_constructor_with_multiple_self_refs_is_preserved_for_type_env_validation() {
    // A constructor with two fields referencing the same domain is a semantic
    // validation concern for TypeEnv/TASK-812, not a parser rejection.
    let source = "sealed type domain Tree { Leaf; Branch<left: Tree, right: Tree>; }";
    let module = parse(source);

    let domains = sealed_domains(&module);
    assert_eq!(domains[0].constructors[1].fields.len(), 2);
    assert_eq!(
        domains[0].constructors[1].fields[0].slot,
        DomainSlot::DomainRef("Tree".into())
    );
    assert_eq!(
        domains[0].constructors[1].fields[1].slot,
        DomainSlot::DomainRef("Tree".into())
    );
}
