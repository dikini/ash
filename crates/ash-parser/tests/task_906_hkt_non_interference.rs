use ash_parser::input::new_input;
use ash_parser::parse_expr::expr;
use ash_parser::surface::{Definition, Expr, Type};
use winnow::Parser;

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

fn parse_expr_complete(source: &str) -> Expr {
    let mut input = new_input(source);
    let parsed = expr
        .parse_next(&mut input)
        .unwrap_or_else(|err| panic!("expression should parse: {source}\nerror: {err:?}"));
    assert_eq!(*input.input.as_ref(), "", "parser left trailing input");
    parsed
}

#[test]
fn proper_type_generics_and_existing_domains_stay_unchanged() {
    let module = parse(
        r#"
        interface Append<Xs: TypeList, Ys: TypeList> {
            append(Xs, Ys) -> TypeList
        }
        fn identity<T>(value: T) -> T { value }
        builtin fn builtin_id<T>(value: T) -> T;
        type fn Head(xs: TypeList) -> Type { case Head<Cons<H, T>> = H; }
        prop NonEmpty<Xs: TypeList>;
        "#,
    );

    let interface = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Interface(interface) => Some(interface),
            _ => None,
        })
        .expect("interface should be present");
    assert!(
        interface
            .type_params
            .iter()
            .all(|param| param.kind.is_none())
    );
    assert!(
        interface
            .type_params
            .iter()
            .all(|param| param.domain.is_some())
    );

    let function = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) => Some(function),
            _ => None,
        })
        .expect("function should be present");
    assert_eq!(function.type_params[0].as_ref(), "T");
    assert!(function.type_params[0].kind.is_none());
    assert!(function.type_params[0].bounds.is_empty());

    let builtin = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::BuiltinFn(builtin) => Some(builtin),
            _ => None,
        })
        .expect("builtin fn should be present");
    assert_eq!(builtin.type_params[0].as_ref(), "T");
    assert!(builtin.type_params[0].kind.is_none());

    let type_fn = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::TypeFn(type_fn) => Some(type_fn),
            _ => None,
        })
        .expect("type function should be present");
    assert!(type_fn.params[0].kind.is_none());
    assert_eq!(type_fn.params[0].ty, Type::Name("TypeList".into()));

    let predicate = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::PropositionPredicate(predicate) => Some(predicate),
            _ => None,
        })
        .expect("proposition predicate should be present");
    assert!(predicate.params[0].kind.is_none());
    assert_eq!(predicate.params[0].domain, Type::Name("TypeList".into()));
}

#[test]
fn malformed_kinds_and_non_audited_sites_fail_closed() {
    parse_err("interface Bad<F : * ->> { m(F) -> F }");
    parse_err("impl <M : * ->> Monad<M> { bind(ma) = ma }");
    parse_err("fn bad<F : * ->>(value: F) -> F { value }");
    parse_err("builtin fn bad<M : * ->>(value: M<Int>) -> M<Int>;");
    parse_err("type fn Bad(F : * ->) -> Type { case Bad<F> = F; }");
    parse_err("prop Bad<F : * ->>;");
    parse_err("type Box<F : * -> *> = F<Int>;");
}

#[test]
fn do_target_type_holes_stay_parser_local_and_unchanged() {
    let Expr::DoBlock { target, .. } =
        parse_expr_complete("do:Result<_, ParseError> { return value }")
    else {
        panic!("expected generalized do block");
    };

    assert_eq!(target.name.as_ref(), "Result");
    assert_eq!(target.args.len(), 2);
    assert!(matches!(&target.args[0], Type::Hole { span } if span.end > span.start));
    assert!(matches!(&target.args[1], Type::Name(name) if name.as_ref() == "ParseError"));
}
