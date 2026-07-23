//! TASK-1971 regression coverage for generic callable signature lowering.

use ash_parser::surface::Definition;
use ash_typeck::types::Type;
use ash_typeck::{TypeEnv, builtin_fn_signature_type, fn_signature_type};

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module should parse: {source}\nerrors: {errors:?}"))
}

fn assert_list_parameter_and_return_share_a_fresh_variable(signature: Type) {
    let Type::Fn(parameters, result) = signature else {
        panic!("generic signature must lower to a pure function type");
    };
    assert_eq!(
        parameters.len(),
        1,
        "generic signature must have one parameter"
    );

    let Type::Constructor {
        name: parameter_name,
        args: parameter_args,
        ..
    } = &parameters[0]
    else {
        panic!(
            "generic parameter must lower as List<a>: {:?}",
            parameters[0]
        );
    };
    assert_eq!(parameter_name.name, "List");
    let [Type::Var(parameter_variable)] = parameter_args.as_slice() else {
        panic!("List parameter must contain one fresh variable: {parameter_args:?}");
    };

    let Type::Constructor {
        name: result_name,
        args: result_args,
        ..
    } = result.as_ref()
    else {
        panic!("generic result must lower as List<a>: {result:?}");
    };
    assert_eq!(result_name.name, "List");
    let [Type::Var(result_variable)] = result_args.as_slice() else {
        panic!("List result must contain one fresh variable: {result_args:?}");
    };
    assert_eq!(
        parameter_variable, result_variable,
        "parameter and result must share the declared generic binder"
    );
}

#[test]
fn generic_fn_signature_binds_list_element_parameter_before_lowering() {
    let module = parse("fn keep<a>(items: List<a>) -> List<a> { items }");
    let function = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) => Some(function),
            _ => None,
        })
        .expect("generic function declaration");
    assert_eq!(function.type_params.len(), 1, "parser preserves one binder");
    assert_eq!(function.type_params[0].name.as_ref(), "a");

    let signature = fn_signature_type(&TypeEnv::with_builtin_types(), function)
        .expect("generic fn List<a> signature must bind a before lowering");
    assert_list_parameter_and_return_share_a_fresh_variable(signature);
}

#[test]
fn generic_builtin_fn_signature_binds_list_element_parameter_before_lowering() {
    let module = parse("builtin fn keep<a>(items: List<a>) -> List<a>;");
    let builtin = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::BuiltinFn(builtin) => Some(builtin),
            _ => None,
        })
        .expect("generic builtin function declaration");
    assert_eq!(builtin.type_params.len(), 1, "parser preserves one binder");
    assert_eq!(builtin.type_params[0].name.as_ref(), "a");

    let signature = builtin_fn_signature_type(&TypeEnv::with_builtin_types(), builtin)
        .expect("generic builtin List<a> signature must bind a before lowering");
    assert_list_parameter_and_return_share_a_fresh_variable(signature);
}
