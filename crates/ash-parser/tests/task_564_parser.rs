use ash_core::ast::{TypeExpr, WhereBound as CoreWhereBound};
use ash_parser::input::new_input;
use ash_parser::lower::{lower_impl_def, lower_interface_def};
use ash_parser::parse_module::parse_module_decl;
use ash_parser::surface::{Definition, Type, WhereBound};
use winnow::Parser;

/// TASK-564: Generic impl syntax with `where` bounds parses correctly.
///
/// Example: `impl<T> Serialize<List<T>> where T: Serialize { ... }`
#[test]
fn parses_generic_impl_with_where_bounds() {
    let mut input =
        new_input("mod m { impl<T> Serialize<List<T>> where T: Serialize { serialize(x) = x } }");

    let decl = parse_module_decl
        .parse_next(&mut input)
        .expect("generic impl with where bounds should parse");

    match decl.definitions().expect("inline module definitions") {
        [Definition::Impl(impl_def)] => {
            assert_eq!(impl_def.type_params.len(), 1);
            assert_eq!(impl_def.type_params[0].as_ref(), "T");
            assert_eq!(impl_def.where_bounds.len(), 1);
            assert!(matches!(
                &impl_def.where_bounds[0],
                WhereBound { param, bound, .. }
                    if param.as_ref() == "T" && bound.as_ref() == "Serialize"
            ));
        }
        other => panic!("expected impl definition, got {other:?}"),
    }
}

/// TASK-564: Interface with associated type declaration parses correctly.
///
/// Example: `interface Serializer<S> { type Ok; serialize_bool(S, Bool) -> S::Ok }`
#[test]
fn parses_interface_with_associated_type() {
    let mut input = new_input(
        "mod m { interface Serializer<S> { type Ok; serialize_bool(S, Bool) -> S::Ok } }",
    );

    let decl = parse_module_decl
        .parse_next(&mut input)
        .expect("interface with associated type should parse");

    match decl.definitions().expect("inline module definitions") {
        [Definition::Interface(iface)] => {
            assert_eq!(iface.associated_types.len(), 1);
            assert_eq!(iface.associated_types[0].name.as_ref(), "Ok");
            assert_eq!(iface.methods.len(), 1);
            assert_eq!(iface.methods[0].name.as_ref(), "serialize_bool");
            assert!(matches!(
                &iface.methods[0].return_type,
                Type::Associated { base, name }
                    if matches!(base.as_ref(), Type::Name(n) if n.as_ref() == "S")
                        && name.as_ref() == "Ok"
            ));
        }
        other => panic!("expected interface definition, got {other:?}"),
    }
}

/// TASK-564: Impl with associated type binding parses correctly.
///
/// Example: `impl Serializer<JsonWriter> { type Ok = String; ... }`
#[test]
fn parses_impl_with_associated_type_binding() {
    let mut input = new_input(
        "mod m { impl Serializer<JsonWriter> { type Ok = String; serialize_bool(jw) = \"hi\" } }",
    );

    let decl = parse_module_decl
        .parse_next(&mut input)
        .expect("impl with associated type binding should parse");

    match decl.definitions().expect("inline module definitions") {
        [Definition::Impl(impl_def)] => {
            assert_eq!(impl_def.associated_type_bindings.len(), 1);
            assert_eq!(impl_def.associated_type_bindings[0].name.as_ref(), "Ok");
            assert!(matches!(
                &impl_def.associated_type_bindings[0].ty,
                Type::Name(n) if n.as_ref() == "String"
            ));
        }
        other => panic!("expected impl definition, got {other:?}"),
    }
}

/// TASK-564: Type contexts accept `Param::AssocName` projections.
///
/// Examples: `S::Ok`, `Map<K,V>::Entry`
#[test]
fn parses_associated_type_projections_in_type_context() {
    let mut input =
        new_input("mod m { interface Mapper<K, V> { get_entry(Map<K, V>) -> Map<K, V>::Entry } }");

    let decl = parse_module_decl
        .parse_next(&mut input)
        .expect("associated type projection should parse");

    match decl.definitions().expect("inline module definitions") {
        [Definition::Interface(iface)] => {
            assert_eq!(iface.methods.len(), 1);
            assert_eq!(iface.methods[0].name.as_ref(), "get_entry");

            // Parameter type: Map<K, V>
            assert_eq!(iface.methods[0].params.len(), 1);
            assert!(matches!(
                &iface.methods[0].params[0],
                Type::Constructor { name, args }
                    if name.as_ref() == "Map"
                        && args.len() == 2
                        && matches!(args[0], Type::Name(ref n) if n.as_ref() == "K")
                        && matches!(args[1], Type::Name(ref n) if n.as_ref() == "V")
            ));

            // Return type: Map<K, V>::Entry
            let return_type = &iface.methods[0].return_type;
            let Type::Associated { base, name } = return_type else {
                panic!("expected associated type, got {return_type:?}");
            };
            assert_eq!(name.as_ref(), "Entry");
            assert!(matches!(
                base.as_ref(),
                Type::Constructor { name: base_name, args }
                    if base_name.as_ref() == "Map"
                        && args.len() == 2
                        && matches!(args[0], Type::Name(ref n) if n.as_ref() == "K")
                        && matches!(args[1], Type::Name(ref n) if n.as_ref() == "V")
            ));
        }
        other => panic!("expected interface definition, got {other:?}"),
    }
}

/// TASK-564: Lowering from surface to core preserves type_params, where_bounds,
/// and associated_type_bindings.
#[test]
fn lowering_preserves_type_params_where_bounds_and_associated_types() {
    let mut input = new_input(
        "mod m { \
            interface Serializer<S> { type Ok; serialize_bool(S, Bool) -> S::Ok } \
            impl<T> Serialize<List<T>> where T: Serialize { type Ok = String; serialize(x) = x } \
        }",
    );

    let decl = parse_module_decl
        .parse_next(&mut input)
        .expect("module should parse");

    let defs = decl.definitions().expect("inline module definitions");

    let iface = match &defs[0] {
        Definition::Interface(i) => {
            lower_interface_def(i).expect("lowering interface should succeed")
        }
        other => panic!("expected interface definition, got {other:?}"),
    };

    assert_eq!(iface.associated_types.len(), 1);
    assert_eq!(iface.associated_types[0].name, "Ok");
    assert_eq!(iface.methods.len(), 1);
    assert!(matches!(
        &iface.methods[0].return_type,
        TypeExpr::Associated { base, name }
            if matches!(base.as_ref(), TypeExpr::Named(n) if n == "S")
                && name == "Ok"
    ));

    let impl_def = match &defs[1] {
        Definition::Impl(i) => lower_impl_def(i).expect("lowering impl should succeed"),
        other => panic!("expected impl definition, got {other:?}"),
    };

    assert_eq!(impl_def.type_params.len(), 1);
    assert_eq!(impl_def.type_params[0], "T");
    assert_eq!(impl_def.where_bounds.len(), 1);
    assert!(matches!(
        &impl_def.where_bounds[0],
        CoreWhereBound { param, bound } if param == "T" && bound == "Serialize"
    ));
    assert_eq!(impl_def.associated_type_bindings.len(), 1);
    assert_eq!(impl_def.associated_type_bindings[0].name, "Ok");
    assert!(matches!(
        &impl_def.associated_type_bindings[0].ty,
        TypeExpr::Named(n) if n == "String"
    ));
}
