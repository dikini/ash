use ash_parser::surface::{ModuleFile, Program, Workflow, WorkflowDef};
use ash_parser::token::Span;

fn parse_module(source: &str) -> ModuleFile {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module file should parse: {source}\nerrors: {errors:?}"))
}

fn program_from_module(module: ModuleFile) -> Program {
    Program {
        definitions: module.definitions,
        helper_workflows: vec![],
        workflow: WorkflowDef {
            name: "main".into(),
            type_params: vec![],
            params: vec![],
            declared_return_type: None,
            plays_roles: vec![],
            capabilities: vec![],
            owned_resources: vec![],
            used_bindings: vec![],
            header_events: vec![],
            body: Workflow::Done {
                span: Span::default(),
            },
            contract: None,
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
fn module_law_rejects_act_returning_function_in_proposition() {
    let err = typecheck_source(
        r#"
        builtin fn effectful(x: Int) -> Act<Int>;
        law no_effects(x: Int): effectful(x)
        "#,
    )
    .expect_err("law propositions must reject Act-returning function calls");

    let message = err.to_string();
    assert!(
        message.contains("law no_effects")
            && message.contains("effectful")
            && message.contains("not pure"),
        "error should identify the law and purity violation; got: {message}"
    );
}

#[test]
fn module_law_rejects_invoke_in_proposition() {
    let err = typecheck_source(
        r#"
        law no_invoke(x: Int): invoke(x)
        "#,
    )
    .expect_err("law propositions must reject invoke calls");

    let message = err.to_string();
    assert!(
        message.contains("law no_invoke") && message.contains("invoke"),
        "error should identify the law and purity violation; got: {message}"
    );
}

#[test]
fn module_law_allows_only_pure_function_references() {
    typecheck_source(
        r#"
        fn is_zero(x: Int) -> Bool { x == 0 }
        law pure_only(x: Int): is_zero(x)
        "#,
    )
    .expect("law propositions referencing only pure functions should pass");
}

#[test]
fn interface_law_rejects_act_returning_function_in_proposition() {
    let err = typecheck_source(
        r#"
        builtin fn effectful<A>(x: A) -> Act<A>;

        interface Bad<A> {
            id(A) -> A
            law no_effects(x: A): effectful(x)
        }
        "#,
    )
    .expect_err("interface law propositions must reject Act-returning function calls");

    let message = err.to_string();
    assert!(
        message.contains("law no_effects")
            && message.contains("effectful")
            && message.contains("not pure"),
        "error should identify the interface law and purity violation; got: {message}"
    );
}

#[test]
fn interface_law_rejects_act_returning_interface_method_in_proposition() {
    let err = typecheck_source(
        r#"
        interface Effectful<A> {
            effect(A) -> Act<A>
            law no_effects(x: A): Effectful::effect(x)
        }
        "#,
    )
    .expect_err("interface law propositions must reject Act-returning interface method calls");

    let message = err.to_string();
    assert!(
        message.contains("law no_effects")
            && message.contains("Effectful::effect")
            && message.contains("not pure"),
        "error should identify the interface method purity violation; got: {message}"
    );
}
