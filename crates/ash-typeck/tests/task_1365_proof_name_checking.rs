use ash_parser::surface::{Definition, ModuleFile, Program, Workflow, WorkflowDef};
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

fn env_with_interface(source: &str) -> ash_typeck::TypeEnv {
    let module = parse_module(source);
    let interface = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Interface(interface) => Some(interface),
            _ => None,
        })
        .expect("source should contain an interface definition");

    let mut env = ash_typeck::TypeEnv::with_builtin_types();
    env.register_interface(interface)
        .expect("interface should register in test environment");
    env
}

#[test]
fn module_proof_for_unknown_law_is_rejected() {
    let err = typecheck_source(
        r#"
        law reflexive(x: Int): x == x
        proof symmetric(x: Int) {
            by_definition
        }
        "#,
    )
    .expect_err("module proof for an undeclared law should reject the program");

    let message = err.to_string();
    assert!(
        message.contains("proof symmetric") && message.contains("law"),
        "error should identify the unmatched proof and law scope; got: {message}"
    );
}

#[test]
fn module_proof_for_known_law_passes() {
    typecheck_source(
        r#"
        law reflexive(x: Int): x == x
        proof reflexive(x: Int) {
            by_definition
        }
        "#,
    )
    .expect("module proof for a module-scope law should pass TASK-1365 name checking");
}

#[test]
fn impl_proof_for_unknown_interface_law_is_rejected() {
    let err = typecheck_source(
        r#"
        interface Eq<A> {
            equiv(A, A) -> Bool
            law reflexive(x: A): equiv(x, x)
        }

        impl Eq<Int> {
            equiv(a, b) = a == b
            proof symmetric(x: Int, y: Int) {
                by_definition
            }
        }
        "#,
    )
    .expect_err("impl proof for an undeclared interface law should reject the program");

    let message = err.to_string();
    assert!(
        message.contains("proof symmetric") && message.contains("Eq") && message.contains("law"),
        "error should identify the unmatched impl proof and interface law scope; got: {message}"
    );
}

#[test]
fn impl_proof_for_known_interface_law_passes() {
    typecheck_source(
        r#"
        interface Eq<A> {
            equiv(A, A) -> Bool
            law reflexive(x: A): equiv(x, x)
        }

        impl Eq<Int> {
            equiv(a, b) = a == b
            proof reflexive(x: Int) {
                by_definition
            }
        }
        "#,
    )
    .expect("impl proof for a declared interface law should pass TASK-1365 name checking");
}

#[test]
fn impl_without_proofs_for_pre_registered_interface_passes() {
    let env = env_with_interface(
        r#"
        interface Eq<A> {
            equiv(A, A) -> Bool
        }
        "#,
    );
    let program = program_from_module(parse_module(
        r#"
        impl Eq<Int> {
            equiv(a, b) = a == b
        }
        "#,
    ));

    ash_typeck::type_check_program_in_env(&env, &program)
        .expect("impls without proofs should not require an in-program interface definition");
}

#[test]
fn impl_proof_for_pre_registered_interface_law_passes() {
    let env = env_with_interface(
        r#"
        interface Eq<A> {
            equiv(A, A) -> Bool
            law reflexive(x: A): equiv(x, x)
        }
        "#,
    );
    let program = program_from_module(parse_module(
        r#"
        impl Eq<Int> {
            equiv(a, b) = a == b
            proof reflexive(x: Int) {
                by_definition
            }
        }
        "#,
    ));

    ash_typeck::type_check_program_in_env(&env, &program)
        .expect("impl proofs should match laws stored on pre-registered interface metadata");
}
