use ash_parser::surface::{Definition, Type, TypeBody};

fn parse_module(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source).expect("module should parse")
}

fn only_function_param_type(source: &str) -> Type {
    let module = parse_module(source);
    let Definition::Function(function) = &module.definitions[0] else {
        panic!("expected function definition");
    };
    function.params[0].ty.clone()
}

fn only_alias_type(source: &str) -> Type {
    alias_type(source, None)
}

fn alias_type(source: &str, expected_name: Option<&str>) -> Type {
    let module = parse_module(source);
    let type_def = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Type(type_def)
                if expected_name.is_none_or(|name| type_def.name.as_ref() == name) =>
            {
                Some(type_def)
            }
            _ => None,
        })
        .expect("expected type definition");
    let TypeBody::Alias(alias) = &type_def.body else {
        panic!("expected type alias");
    };
    alias.clone()
}

fn assert_name(ty: &Type, expected: &str) {
    match ty {
        Type::Name(name) => assert_eq!(name.as_ref(), expected),
        other => panic!("expected named type {expected}, got {other:?}"),
    }
}

fn assert_binary_int_string_to_bool(ty: &Type) {
    match ty {
        Type::Fn(params, ret) => {
            assert_eq!(params.len(), 2, "expected a two-argument callable");
            assert_name(&params[0], "Int");
            assert_name(&params[1], "String");
            assert_name(ret, "Bool");
        }
        other => panic!("expected Type::Fn, got {other:?}"),
    }
}

fn assert_binary_int_int_to_int(ty: &Type) {
    match ty {
        Type::Fn(params, ret) => {
            assert_eq!(params.len(), 2, "expected a two-argument callable");
            assert_name(&params[0], "Int");
            assert_name(&params[1], "Int");
            assert_name(ret, "Int");
        }
        other => panic!("expected Type::Fn, got {other:?}"),
    }
}

fn assert_unary_int_to_bool(ty: &Type) {
    match ty {
        Type::Fn(params, ret) => {
            assert_eq!(params.len(), 1, "expected a unary callable");
            assert_name(&params[0], "Int");
            assert_name(ret, "Bool");
        }
        other => panic!("expected Type::Fn, got {other:?}"),
    }
}

#[test]
fn module_annotation_parses_parenthesized_n_ary_callable_domain() {
    let ty = only_function_param_type("fn keep(f: (Int, String) -> Bool) -> Bool { true }");
    let int_ty = only_function_param_type("fn keep(f: (Int, Int) -> Int) -> Int { 0 }");

    assert_binary_int_string_to_bool(&ty);
    assert_binary_int_int_to_int(&int_ty);
}

#[test]
fn type_alias_parses_parenthesized_n_ary_callable_domain() {
    let ty = only_alias_type("type Predicate = (Int, String) -> Bool;");

    assert_binary_int_string_to_bool(&ty);
}

#[test]
fn legacy_fn_syntax_remains_compatible() {
    let module_ty =
        only_function_param_type("fn keep(f: Fn(Int, String) -> Bool) -> Bool { true }");
    let alias_ty = only_alias_type("type Predicate = Fn(Int, String) -> Bool;");
    let unary_module_ty = only_function_param_type("fn keep(f: Int -> Bool) -> Bool { true }");
    let unary_alias_ty = only_alias_type("type Predicate = Int -> Bool;");

    assert_binary_int_string_to_bool(&module_ty);
    assert_binary_int_string_to_bool(&alias_ty);
    assert_unary_int_to_bool(&unary_module_ty);
    assert_unary_int_to_bool(&unary_alias_ty);
}

#[test]
fn tuple_domain_is_not_silently_lowered_as_unary_argument() {
    for ty in [
        only_function_param_type("fn keep(f: (Int, String) -> Bool) -> Bool { true }"),
        only_alias_type("type Predicate = (Int, String) -> Bool;"),
    ] {
        let Type::Fn(params, _) = ty else {
            panic!("expected Type::Fn");
        };
        assert_eq!(params.len(), 2, "tuple domain was lowered as one argument");
        assert!(
            !matches!(params.as_slice(), [Type::Tuple(_)]),
            "callable domain must not be represented as a unary tuple argument"
        );
    }
}

#[test]
fn unary_tuple_argument_spelling_is_explicit_or_diagnostic() {
    let pair_alias = only_alias_type("type Pair = (Int, String);");
    assert!(matches!(pair_alias, Type::Tuple(ref items) if items.len() == 2));

    let unary_tuple_callable = alias_type(
        "type Pair = (Int, String);\ntype Predicate = Pair -> Bool;",
        Some("Predicate"),
    );

    match unary_tuple_callable {
        Type::Fn(params, ret) => {
            assert_eq!(params.len(), 1);
            assert_name(&params[0], "Pair");
            assert_name(&ret, "Bool");
        }
        other => panic!("expected explicit unary callable through Pair alias, got {other:?}"),
    }
}
