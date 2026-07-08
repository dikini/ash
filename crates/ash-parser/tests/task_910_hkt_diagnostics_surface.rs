use ash_core::Kind;
use ash_parser::surface::{Definition, InterfaceTypeParam, Type};

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

fn interface_named<'a>(
    module: &'a ash_parser::surface::ModuleFile,
    name: &str,
) -> &'a ash_parser::surface::InterfaceDef {
    module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Interface(interface) if interface.name.as_ref() == name => Some(interface),
            _ => None,
        })
        .unwrap_or_else(|| panic!("interface {name} should be present"))
}

fn assert_constructor_kinded(param: &InterfaceTypeParam) {
    let annotation = param
        .kind
        .as_ref()
        .unwrap_or_else(|| panic!("{} should preserve an explicit kind", param.name));
    assert_eq!(annotation.kind, Kind::n_ary(1));
    assert!(
        annotation.span.end > annotation.span.start,
        "kind annotation should preserve a non-empty source span"
    );
    assert!(
        param.domain.is_none(),
        "kinded binders must not be encoded as interface domain constraints"
    );
}

#[test]
fn hkt1_parses_functor_applicative_and_monad_constructor_binders() {
    let module = parse(
        r#"
        interface Functor<F : * -> *> {
            map(F<Int>) -> F<Int>
        }

        interface Applicative<F : * -> *> {
            pure(Int) -> F<Int>
        }

        interface Monad<M : * -> *> {
            bind(M<Int>) -> M<Int>
        }
        "#,
    );

    assert_constructor_kinded(&interface_named(&module, "Functor").type_params[0]);
    assert_constructor_kinded(&interface_named(&module, "Applicative").type_params[0]);
    assert_constructor_kinded(&interface_named(&module, "Monad").type_params[0]);
}

#[test]
fn malformed_kinded_binders_remain_parser_diagnostics() {
    parse_err("interface Bad<F : * ->> { bad(F) -> F }");
    parse_err("fn bad<F : * ->>(value: F<Int>) -> F<Int> { value }");
    parse_err("prop Bad<F : * ->>;");
}

#[test]
fn kinded_binder_syntax_stays_rejected_at_non_enabled_type_alias_site() {
    parse_err("type Alias<F : * -> *> = F<Int>;");
}

#[test]
fn hkt_holes_stay_rejected_in_ordinary_function_type_positions() {
    parse_err("fn bad_param(value: _) -> Int { 1 }");
    parse_err("fn bad_return(value: Int) -> _ { value }");
}

#[test]
fn hkt_holes_stay_rejected_in_ordinary_interface_method_type_positions() {
    parse_err("interface Bad { bad_param(_) -> Int }");
    parse_err("interface Bad { bad_return(Int) -> _ }");
}

#[test]
fn hkt_holes_stay_rejected_in_ordinary_proposition_type_positions() {
    parse_err("prop Bad<X: _>;");
    parse_err("fn bad_equality<T>(value: T) -> T where T == _ { value }");
    parse_err("fn bad_predicate<T>(value: T) -> T where NonEmpty<_> { value }");
}

#[test]
fn hkt_holes_stay_rejected_in_ordinary_alias_resource_and_capability_type_positions() {
    parse_err("type Alias = _;");
    parse_err("resource type Store { value: _ }");
    let capability = ["cap", "ability"].concat();
    parse_err(&format!("{capability} read: observe(input: _) returns Int"));
    parse_err(&format!("{capability} read: observe() returns _"));
    parse_err(&format!(
        r#"
        {capability} impl StoreImpl for Store
            requires resource db: _
        {{
            observe read() returns Int {{ 1 }}
        }}
        "#
    ));
}

#[test]
fn hkt_holes_stay_rejected_in_associated_type_bindings_inside_impls() {
    parse_err(
        r#"
        interface Iterator<I> { type Item; }
        impl Iterator<List<Int>> {
            type Item = _;
        }
        "#,
    );
}

#[test]
fn underscore_prefixed_type_names_are_not_treated_as_holes() {
    let module = parse(
        r#"
        interface Monad<M : * -> *> {}
        impl Monad<_M> {}
        fn id(value: _M) -> _M { value }
        "#,
    );

    let implementation = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Impl(implementation) => Some(implementation),
            _ => None,
        })
        .expect("impl should be present");

    assert!(matches!(
        &implementation.type_args[0],
        Type::Name(name) if name.as_ref() == "_M"
    ));
}

#[test]
fn hkt4_impl_head_preserves_partial_constructor_hole_surface() {
    let module = parse(
        r#"
        interface Monad<M : * -> *> {}
        impl <E : *> Monad<Result<_, E>> {}
        "#,
    );

    let implementation = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Impl(implementation) => Some(implementation),
            _ => None,
        })
        .expect("impl should be present");

    let Type::Constructor { name, args } = &implementation.type_args[0] else {
        panic!("expected partial Result constructor head");
    };
    assert_eq!(name.as_ref(), "Result");
    assert!(matches!(&args[0], Type::Hole { span } if span.end > span.start));
    assert!(matches!(&args[1], Type::Name(name) if name.as_ref() == "E"));
}
