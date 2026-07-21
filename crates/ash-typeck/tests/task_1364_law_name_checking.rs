use ash_parser::surface::{
    Definition, Expr, FnDef, Literal, ModuleFile, Program, ProgramEntry, Visibility,
};
use ash_parser::token::Span;

fn parse_module(source: &str) -> ModuleFile {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module file should parse: {source}\nerrors: {errors:?}"))
}

fn program_from_module(module: ModuleFile) -> Program {
    let mut definitions = module.definitions;
    definitions.push(Definition::Function(FnDef {
        visibility: Visibility::Inherited,
        name: "main".into(),
        type_params: vec![],
        params: vec![],
        return_type: None,
        proposition_tail: None,
        contract: None,
        body: Expr::Literal(Literal::Null),
        span: Span::default(),
    }));
    Program {
        definitions,
        entry: ProgramEntry {
            function: "main".into(),
            span: Span::default(),
        },
    }
}

fn typecheck_source(
    source: &str,
) -> Result<ash_typeck::TypeCheckResult, ash_typeck::TypeCheckError> {
    ash_typeck::type_check_program(&program_from_module(parse_module(source)))
}

#[test]
fn interface_law_unknown_proposition_name_is_rejected() {
    let err = typecheck_source(
        r#"
        interface Eq<A> {
            equiv(A, A) -> Bool
            law reflexive(x: A): missing_predicate(x)
        }
        "#,
    )
    .expect_err("unknown names in interface law propositions should reject the program");

    let message = err.to_string();
    assert!(
        message.contains("law reflexive") && message.contains("missing_predicate"),
        "error should identify the law and missing name; got: {message}"
    );
}

#[test]
fn module_law_unknown_proposition_name_is_rejected() {
    let err = typecheck_source(
        r#"
        law known_names_only(x: Int): missing_predicate(x)
        "#,
    )
    .expect_err("unknown names in module law propositions should reject the program");

    let message = err.to_string();
    assert!(
        message.contains("law known_names_only") && message.contains("missing_predicate"),
        "error should identify the module law and missing name; got: {message}"
    );
}

#[test]
fn law_propositions_can_reference_law_params_and_registered_functions() {
    typecheck_source(
        r#"
        fn is_zero(x: Int) -> Bool { x == 0 }

        interface ZeroLike<A> {
            zero(A) -> Int
            law zero_is_zero(x: A): is_zero(zero(x))
        }

        law zero_int_is_zero(x: Int): is_zero(x)
        "#,
    )
    .expect("law propositions may reference law parameters, interface methods, and registered functions");
}
