//! TASK-1971 regression coverage for inline callable-row declaration normalization.

use ash_parser::surface::{
    Definition, Expr, FnDef, Literal, Program, ProgramEntry, Type as SurfaceType, Visibility,
};
use ash_parser::token::Span;
use ash_typeck::types::Type;
use ash_typeck::{TypeEnv, fn_signature_type, type_check_program};

fn parse_program(source: &str) -> ash_parser::surface::Program {
    let module = ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("source should parse: {source}\nerrors: {errors:?}"));
    let entry = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == "main" => {
                Some(ash_parser::surface::ProgramEntry {
                    function: function.name.clone(),
                    span: function.span,
                })
            }
            _ => None,
        })
        .expect("program has main");
    ash_parser::surface::Program {
        definitions: module.definitions,
        entry,
    }
}

fn function<'a>(
    program: &'a ash_parser::surface::Program,
    name: &str,
) -> &'a ash_parser::surface::FnDef {
    program
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == name => Some(function),
            _ => None,
        })
        .unwrap_or_else(|| panic!("function {name} should exist"))
}

#[test]
fn inline_row_callable_result_normalizes_to_body_result_while_retaining_surface_row() {
    let program = parse_program(
        r#"
        fn read() -> {posixfs.read} String { "ok" }
        fn main() -> Int { 0 }
        "#,
    );
    let read = function(&program, "read");
    assert!(matches!(
        read.return_type.as_ref(),
        Some(SurfaceType::Fn(parameters, Some(_), result))
            if parameters.is_empty() && matches!(result.as_ref(), SurfaceType::Name(name) if name.as_ref() == "String")
    ));

    let signature = fn_signature_type(&TypeEnv::with_builtin_types(), read)
        .expect("inline-row callable signature should lower");
    assert!(
        matches!(&signature, Type::Fn(parameters, result) if parameters.is_empty() && result.as_ref() == &Type::String),
        "an inline callable row annotates the callable boundary; it is not a returned closure: {signature:?}"
    );

    type_check_program(&program)
        .expect("String body must satisfy inline-row declared String result");
    assert!(matches!(
        function(&program, "read").return_type.as_ref(),
        Some(SurfaceType::Fn(_, Some(_), _))
    ));
}

#[test]
fn plain_closure_return_is_not_unwrapped_as_an_inline_callable_row() {
    let span = Span::default();
    let program = Program {
        definitions: vec![
            Definition::Function(FnDef {
                visibility: Visibility::Inherited,
                name: "factory".into(),
                type_params: vec![],
                params: vec![],
                return_type: Some(SurfaceType::Fn(
                    vec![],
                    None,
                    Box::new(SurfaceType::Name("String".into())),
                )),
                proposition_tail: None,
                contract: None,
                body: Expr::FnDef {
                    params: vec![],
                    return_type: Some("String".into()),
                    body: Box::new(Expr::Literal(Literal::String("ok".into()))),
                    span,
                },
                span,
            }),
            Definition::Function(FnDef {
                visibility: Visibility::Inherited,
                name: "main".into(),
                type_params: vec![],
                params: vec![],
                return_type: Some(SurfaceType::Name("Int".into())),
                proposition_tail: None,
                contract: None,
                body: Expr::Literal(Literal::Int(0)),
                span,
            }),
        ],
        entry: ProgramEntry {
            function: "main".into(),
            span,
        },
    };
    let factory = function(&program, "factory");
    assert!(matches!(
        factory.return_type.as_ref(),
        Some(SurfaceType::Fn(parameters, None, result))
            if parameters.is_empty() && matches!(result.as_ref(), SurfaceType::Name(name) if name.as_ref() == "String")
    ));

    let signature = fn_signature_type(&TypeEnv::with_builtin_types(), factory)
        .expect("plain closure return signature should lower");
    assert!(
        matches!(&signature, Type::Fn(parameters, result)
            if parameters.is_empty()
                && matches!(result.as_ref(), Type::Fn(closure_parameters, closure_result)
                    if closure_parameters.is_empty() && closure_result.as_ref() == &Type::String)),
        "TASK-959 plain closure returns must remain closures: {signature:?}"
    );
    type_check_program(&program).expect("plain closure return remains a valid function result");
}
