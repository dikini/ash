//! TASK-786 regression: public builtin `await` is a module declaration.

use ash_parser::surface::{Definition, Type, Visibility};

const SOURCE: &str = "pub builtin fn await<A>(process_handle: P<A>) -> Proc<A>;";

#[test]
fn task_786_public_builtin_await_is_preserved_for_module_metadata() {
    let module = ash_parser::parse_surface_file(SOURCE)
        .unwrap_or_else(|errors| panic!("public builtin await signature should parse: {errors:?}"));

    let [Definition::BuiltinFn(await_builtin)] = module.definitions.as_slice() else {
        panic!("source must produce exactly one public builtin declaration");
    };

    assert_eq!(await_builtin.visibility, Visibility::Public);
    assert_eq!(await_builtin.name.as_ref(), "await");
    assert_eq!(await_builtin.type_params.len(), 1);
    assert_eq!(await_builtin.type_params[0].as_ref(), "A");
    assert_eq!(await_builtin.params.len(), 1);
    assert_eq!(await_builtin.params[0].name.as_ref(), "process_handle");
    assert!(matches!(
        &await_builtin.params[0].ty,
        Type::Constructor { name, args }
            if name.as_ref() == "P" && matches!(args.as_slice(), [Type::Name(arg)] if arg.as_ref() == "A")
    ));
    assert!(matches!(
        &await_builtin.return_type,
        Type::Constructor { name, args }
            if name.as_ref() == "Proc" && matches!(args.as_slice(), [Type::Name(arg)] if arg.as_ref() == "A")
    ));
}

#[test]
fn task_786_public_builtin_await_keeps_handle_reserved_as_a_parameter_name() {
    let source = "pub builtin fn await<A>(handle: P<A>) -> Proc<A>;";

    assert!(
        ash_parser::parse_surface_file(source).is_err(),
        "the contextual callable spelling must not relax the reserved `handle` parameter name"
    );
}
