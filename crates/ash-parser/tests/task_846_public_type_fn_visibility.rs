use ash_parser::surface::{Definition, Type, TypePattern, Visibility};

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source).expect("module file should parse")
}

fn parse_err(source: &str) {
    assert!(
        ash_parser::parse_surface_file(source).is_err(),
        "source should be rejected: {source}"
    );
}

fn only_type_fn(source: &str) -> ash_parser::surface::TypeFnDef {
    parse(source)
        .definitions
        .into_iter()
        .find_map(|def| match def {
            Definition::TypeFn(def) => Some(def),
            _ => None,
        })
        .expect("type fn definition should be present")
}

#[test]
fn parses_pub_type_fn_as_public_surface_definition_with_spans_and_equations() {
    let source = r#"
pub type fn Append(xs: TypeList, ys: TypeList) -> TypeList
    decreases xs
{
    case Append<Nil, ys> = ys;
    case Append<Cons<h, t>, _> = Cons<h, Append<t, ys>>;
}
"#;

    let type_fn = only_type_fn(source);

    assert_eq!(type_fn.visibility, Visibility::Public);
    assert_eq!(type_fn.name.as_ref(), "Append");
    assert_eq!(type_fn.params.len(), 2);
    assert_eq!(type_fn.params[0].name.as_ref(), "xs");
    assert_eq!(type_fn.params[0].ty, Type::Name("TypeList".into()));
    assert!(type_fn.params[0].span.end > type_fn.params[0].span.start);
    assert_eq!(type_fn.params[1].name.as_ref(), "ys");
    assert_eq!(type_fn.params[1].ty, Type::Name("TypeList".into()));
    assert_eq!(type_fn.return_type, Type::Name("TypeList".into()));
    assert!(type_fn.header_span.start >= type_fn.span.start);
    assert!(type_fn.header_span.end < type_fn.span.end);

    let decreases = type_fn.decreases.as_ref().expect("decreases clause");
    assert_eq!(decreases.param.as_ref(), "xs");
    assert!(decreases.span.end > decreases.span.start);

    assert_eq!(type_fn.equations.len(), 2);
    let first = &type_fn.equations[0];
    assert_eq!(first.head.as_ref(), "Append");
    assert!(first.head_span.end > first.head_span.start);
    assert!(first.span.end > first.span.start);
    assert_eq!(first.result, Type::Name("ys".into()));
    assert!(first.result_span.end > first.result_span.start);
    assert_eq!(first.patterns.len(), 2);
    assert!(matches!(
        &first.patterns[0],
        TypePattern::Constructor { name, args, span }
            if name.as_ref() == "Nil" && args.is_empty() && span.end > span.start
    ));
    assert!(matches!(
        &first.patterns[1],
        TypePattern::Var { name, span } if name.as_ref() == "ys" && span.end > span.start
    ));

    let second = &type_fn.equations[1];
    assert!(matches!(
        &second.patterns[0],
        TypePattern::Constructor { name, args, span }
            if name.as_ref() == "Cons" && args.len() == 2 && span.end > span.start
    ));
    assert!(matches!(
        &second.patterns[1],
        TypePattern::Wildcard { span } if span.end > span.start
    ));
    assert_eq!(
        second.result,
        Type::Constructor {
            name: "Cons".into(),
            args: vec![
                Type::Name("h".into()),
                Type::Constructor {
                    name: "Append".into(),
                    args: vec![Type::Name("t".into()), Type::Name("ys".into())],
                },
            ],
        }
    );
}

#[test]
fn parses_private_type_fn_as_inherited_visibility() {
    let type_fn = only_type_fn("type fn Id(x: Type) -> Type { case Id<x> = x; }");

    assert_eq!(type_fn.visibility, Visibility::Inherited);
    assert_eq!(type_fn.name.as_ref(), "Id");
    assert_eq!(type_fn.params.len(), 1);
    assert_eq!(type_fn.equations.len(), 1);
}

#[test]
fn rejects_malformed_public_type_fn_forms() {
    parse_err("pub type fn Unit() -> Type { case Unit<> = Type; }");
    parse_err("pub type fn Id(x: Type) -> Type { case Other<x> = x; }");
    parse_err("pub type fn Id(x: Type) -> Type { case Id x = x; }");
    parse_err("pub type fn Id(x: Type) -> Type { case Id<x> = x }");
}

#[test]
fn parses_inline_module_type_fn_with_preserved_visibility() {
    // TASK-2059 gives file and inline modules one definition-item grammar.
    let private = parse("mod inner { type fn Id(x: Type) -> Type { case Id<x> = x; } }");
    assert!(matches!(
        private.module_decls[0].definitions(),
        Some([Definition::TypeFn(type_fn)]) if type_fn.visibility == Visibility::Inherited
    ));

    let public = parse("mod inner { pub type fn Id(x: Type) -> Type { case Id<x> = x; } }");
    assert!(matches!(
        public.module_decls[0].definitions(),
        Some([Definition::TypeFn(type_fn)]) if type_fn.visibility == Visibility::Public
    ));
}
