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

#[test]
fn parses_module_level_type_fn_with_raw_spans_patterns_and_rhs() {
    let source = r#"sealed type domain TypeList {
    Nil;
    Cons<head: Type, tail: TypeList>;
}

type fn Append(xs: TypeList, ys: TypeList) -> TypeList
    decreases xs
{
    case Append<Nil, ys> = ys;
    case Append<Cons<h, t>, _> = Cons<h, Append<t, ys>>;
}"#;

    let module = parse(source);
    let type_fn = module
        .definitions
        .iter()
        .find_map(|def| match def {
            Definition::TypeFn(def) => Some(def),
            _ => None,
        })
        .expect("type fn definition should be present");

    assert_eq!(type_fn.visibility, Visibility::Inherited);
    assert_eq!(type_fn.name.as_ref(), "Append");
    assert_eq!(type_fn.params.len(), 2);
    assert_eq!(type_fn.params[0].name.as_ref(), "xs");
    assert_eq!(type_fn.params[0].ty, Type::Name("TypeList".into()));
    assert_eq!(type_fn.params[1].name.as_ref(), "ys");
    assert_eq!(type_fn.return_type, Type::Name("TypeList".into()));

    let decreases = type_fn.decreases.as_ref().expect("decreases clause");
    assert_eq!(decreases.param.as_ref(), "xs");
    assert!(decreases.span.end > decreases.span.start);
    assert!(type_fn.header_span.start >= type_fn.span.start);
    assert!(type_fn.header_span.end < type_fn.span.end);

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

    // Lowercase bare names remain raw syntactic candidates in the parser. Later
    // type checking owns expected-domain disambiguation and may reinterpret this
    // spelling as a lowercase marker constructor when a sealed-domain constructor
    // namespace resolves it that way.
    let lowercase_ctor = parse("type fn Lower(x: Type) -> Type { case Lower<nil> = x; }");
    let lower_def = lowercase_ctor
        .definitions
        .iter()
        .find_map(|def| match def {
            Definition::TypeFn(def) => Some(def),
            _ => None,
        })
        .expect("lowercase bare-name pattern parses raw");
    assert!(matches!(
        &lower_def.equations[0].patterns[0],
        TypePattern::Var { name, span } if name.as_ref() == "nil" && span.end > span.start
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
fn dispatches_type_fn_before_ordinary_type_parser() {
    let source = "type fn Id(x: Type) -> Type { case Id<x> = x; }\ntype Alias = Int;";
    let module = parse(source);
    assert!(matches!(module.definitions[0], Definition::TypeFn(_)));
    assert!(matches!(module.definitions[1], Definition::Type(_)));
}

#[test]
fn preserves_visibility_prefixed_type_fn_for_downstream_spec_f_validation() {
    let public = parse("pub type fn Id(x: Type) -> Type { case Id<x> = x; }");
    let Definition::TypeFn(public) = &public.definitions[0] else {
        panic!("pub type fn should parse as a type function definition");
    };
    assert_eq!(public.visibility, Visibility::Public);

    let crate_visible = parse("pub(crate) type fn Id(x: Type) -> Type { case Id<x> = x; }");
    let Definition::TypeFn(crate_visible) = &crate_visible.definitions[0] else {
        panic!("pub(crate) type fn should parse as a type function definition");
    };
    assert_eq!(crate_visible.visibility, Visibility::Crate);
}

#[test]
fn rejects_zero_parameter_type_fn() {
    parse_err("type fn Unit() -> Type { case Unit<> = Type; }");
}

#[test]
fn rejects_malformed_case_heads_and_missing_semicolons() {
    parse_err("type fn Id(x: Type) -> Type { case Other<x> = x; }");
    parse_err("type fn Id(x: Type) -> Type { case Id x = x; }");
    parse_err("type fn Id(x: Type) -> Type { case Id<x> = x }");
}

#[test]
fn parses_inline_module_type_fn() {
    // TASK-2059 gives file and inline modules one definition-item grammar.
    let module = parse("mod inner { type fn Id(x: Type) -> Type { case Id<x> = x; } }");
    assert!(matches!(
        module.module_decls[0].definitions(),
        Some([Definition::TypeFn(type_fn)]) if type_fn.name.as_ref() == "Id"
    ));
}
