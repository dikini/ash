use ash_core::Kind;
use ash_parser::surface::{
    Definition, InterfaceTypeParam, PropositionPredicateParam, TypeFnParam, TypeParam,
};

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module file should parse: {source}\nerrors: {errors:?}"))
}

fn kind_string(kind: &Kind) -> String {
    kind.to_string()
}

fn assert_type_param_kind(param: &TypeParam, expected: &str) {
    let annotation = param
        .kind
        .as_ref()
        .unwrap_or_else(|| panic!("{} should preserve an explicit kind", param.name));
    assert_eq!(kind_string(&annotation.kind), expected);
    assert!(
        annotation.span.end > annotation.span.start,
        "kind annotation should preserve a non-empty span"
    );
}

fn assert_interface_param_kind(param: &InterfaceTypeParam, expected: &str) {
    let annotation = param
        .kind
        .as_ref()
        .unwrap_or_else(|| panic!("{} should preserve an explicit kind", param.name));
    assert_eq!(kind_string(&annotation.kind), expected);
    assert!(
        annotation.span.end > annotation.span.start,
        "kind annotation should preserve a non-empty span"
    );
    assert!(
        param.domain.is_none(),
        "kinded interface binders must not be encoded as domain types"
    );
}

fn assert_type_fn_param_kind(param: &TypeFnParam, expected: &str) {
    let annotation = param
        .kind
        .as_ref()
        .unwrap_or_else(|| panic!("{} should preserve an explicit kind", param.name));
    assert_eq!(kind_string(&annotation.kind), expected);
    assert!(annotation.span.end > annotation.span.start);
}

fn assert_predicate_param_kind(param: &PropositionPredicateParam, expected: &str) {
    let annotation = param
        .kind
        .as_ref()
        .unwrap_or_else(|| panic!("{} should preserve an explicit kind", param.name));
    assert_eq!(kind_string(&annotation.kind), expected);
    assert!(annotation.span.end > annotation.span.start);
}

#[test]
fn parses_interface_and_impl_kinded_binders() {
    let module = parse(
        r#"
        interface Functor<F : * -> *, A : *> {
            map(F<A>) -> F<A>
        }

        impl <M : * -> *> Monad<M> {
            bind(ma) = ma
        }
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
    assert_interface_param_kind(&interface.type_params[0], "* -> *");
    assert_interface_param_kind(&interface.type_params[1], "*");

    let implementation = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Impl(implementation) => Some(implementation),
            _ => None,
        })
        .expect("impl should be present");
    assert_interface_param_kind(&implementation.type_params[0], "* -> *");
}

#[test]
fn parses_function_and_builtin_kinded_type_params() {
    let module = parse(
        r#"
        fn lift<F : * -> *, A : *>(value: A) -> F<A> { value }
        builtin fn pure<M : * -> *, A : *>(value: A) -> M<A>;
        "#,
    );

    let function = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) => Some(function),
            _ => None,
        })
        .expect("function should be present");
    assert_type_param_kind(&function.type_params[0], "* -> *");
    assert_type_param_kind(&function.type_params[1], "*");

    let builtin = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::BuiltinFn(builtin) => Some(builtin),
            _ => None,
        })
        .expect("builtin fn should be present");
    assert_type_param_kind(&builtin.type_params[0], "* -> *");
    assert_type_param_kind(&builtin.type_params[1], "*");
}

#[test]
fn parses_type_function_and_proposition_predicate_kinded_params() {
    let module = parse(
        r#"
        type fn Apply(F : * -> *, A : *) -> Type {
            case Apply<F, A> = F<A>;
        }

        prop Maps<F : * -> *, A : *>;
        "#,
    );

    let type_fn = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::TypeFn(type_fn) => Some(type_fn),
            _ => None,
        })
        .expect("type function should be present");
    assert_type_fn_param_kind(&type_fn.params[0], "* -> *");
    assert_type_fn_param_kind(&type_fn.params[1], "*");

    let predicate = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::PropositionPredicate(predicate) => Some(predicate),
            _ => None,
        })
        .expect("proposition predicate should be present");
    assert_predicate_param_kind(&predicate.params[0], "* -> *");
    assert_predicate_param_kind(&predicate.params[1], "*");
}
