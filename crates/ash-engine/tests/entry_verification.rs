//! Integration tests for canonical entry signature verification.

use ash_engine::{
    Engine, EntryBootstrapError, EntryVerificationError, load_runtime_entry_stdlib_sources,
};

fn parse_entry_artifact(engine: &Engine, source: &str) -> ash_engine::Entry {
    engine.parse(source).expect("entry source should parse")
}

const ENTRY_SOURCE_WITH_RUNTIME_IMPORTS: &str = r"
    use result::Result
    use runtime::RuntimeError
    use runtime::Args

    fn main() -> Result<(), RuntimeError> { Ok { value: {} } }
";

#[test]
fn accepts_main_with_canonical_result_return_and_no_params() {
    let engine = Engine::new().build().expect("engine builds");
    let entry = parse_entry_artifact(
        &engine,
        "fn main() -> Result<(), RuntimeError> { Ok { value: {} } }",
    );

    let result = engine.verify_entry_definition(&entry);

    assert!(result.is_ok(), "canonical entry should verify");
}

#[test]
fn accepts_main_with_capability_args_parameter() {
    let engine = Engine::new().build().expect("engine builds");
    let entry = parse_entry_artifact(
        &engine,
        "fn main(args: capability Args) -> Result<(), RuntimeError> { Ok { value: {} } }",
    );

    let result = engine.verify_entry_definition(&entry);

    assert!(result.is_ok(), "capability parameter entry should verify");
}

#[test]
fn rejects_wrong_return_type() {
    let engine = Engine::new().build().expect("engine builds");
    let entry = parse_entry_artifact(&engine, "fn main() -> Int { {} }");

    let err = engine
        .verify_entry_definition(&entry)
        .expect_err("wrong return type should be rejected");

    assert!(matches!(
        err,
        EntryVerificationError::WrongReturnType { .. }
    ));
}

#[test]
fn rejects_non_capability_parameter() {
    let engine = Engine::new().build().expect("engine builds");
    let entry = parse_entry_artifact(
        &engine,
        "fn main(args: Args) -> Result<(), RuntimeError> { Ok { value: {} } }",
    );

    let err = engine
        .verify_entry_definition(&entry)
        .expect_err("non-capability parameter should be rejected");

    assert!(matches!(
        err,
        EntryVerificationError::NonCapabilityParameter { .. }
    ));
}

#[test]
fn rejects_missing_main_entry_at_parse_boundary() {
    let engine = Engine::new().build().expect("engine builds");

    let err = engine
        .parse("fn other() -> Result<(), RuntimeError> { Ok { value: {} } }")
        .expect_err("non-main entry should be rejected");

    assert!(err.to_string().contains("expected fn main entry"));
}

#[test]
fn loads_runtime_entry_stdlib_sources() {
    let modules = load_runtime_entry_stdlib_sources().expect("runtime stdlib loads");

    assert!(
        modules.iter().any(|module| {
            module.module_path == "result" && module.source.contains("pub type Result")
        }),
        "result stdlib module should be available"
    );
    assert!(
        modules.iter().any(|module| {
            module.module_path == "runtime" && module.source.contains("pub use args::Args")
        }),
        "runtime stdlib root module should be available"
    );
    assert!(
        modules.iter().any(|module| {
            module.module_path == "runtime::error"
                && module.source.contains("pub type RuntimeError")
        }),
        "runtime error stdlib module should be available"
    );
    assert!(
        modules.iter().any(|module| {
            module.module_path == "runtime::args" && module.source.contains("pub builtin type Args")
        }),
        "runtime args stdlib module should be available"
    );
}

#[test]
fn loads_registered_runtime_stdlib_modules() {
    let engine = Engine::new().build().expect("engine builds");

    engine
        .load_runtime_stdlib()
        .expect("runtime stdlib registers on engine");

    assert!(
        engine.has_registered_runtime_module("result"),
        "engine should register the result module"
    );
    assert!(
        engine.has_registered_runtime_module("runtime"),
        "engine should register the runtime root module"
    );
    assert!(
        engine.has_registered_runtime_module("runtime::error"),
        "engine should register the runtime error module"
    );
    assert!(
        engine.has_registered_runtime_module("runtime::args"),
        "engine should register the runtime args module"
    );
}

#[test]
fn parses_checks_and_verifies_entry_source_with_runtime_imports() {
    let engine = Engine::new().build().expect("engine builds");
    engine
        .load_runtime_stdlib()
        .expect("runtime stdlib registers on engine");
    let mut entry = engine
        .parse_entry_source(ENTRY_SOURCE_WITH_RUNTIME_IMPORTS)
        .expect("entry source should parse");

    engine.check(&mut entry).expect("entry should type check");
    engine
        .verify_entry_definition(&entry)
        .expect("entry should verify");
}

#[test]
fn rejects_entry_source_with_runtime_imports_when_runtime_registry_is_empty() {
    let engine = Engine::new().build().expect("engine builds");

    let _err = engine
        .parse_entry_source(ENTRY_SOURCE_WITH_RUNTIME_IMPORTS)
        .expect_err("entry source should reject unregistered runtime imports");
}

#[test]
fn parses_entry_source_with_leading_comments_before_runtime_imports() {
    let engine = Engine::new().build().expect("engine builds");
    engine
        .load_runtime_stdlib()
        .expect("runtime stdlib registers on engine");

    let mut entry = engine
        .parse_entry_source(
            r"
            -- entrypoint comment header
            -- another header line
            use result::Result
            use runtime::RuntimeError
            use runtime::Args

            fn main() -> Result<(), RuntimeError> { Ok { value: {} } }
        ",
        )
        .expect("entry source with leading comments should parse");

    engine.check(&mut entry).expect("entry should type check");
    engine
        .verify_entry_definition(&entry)
        .expect("entry should verify");
}

#[test]
fn parses_entry_source_with_block_comments_before_and_between_runtime_imports() {
    let engine = Engine::new().build().expect("engine builds");
    engine
        .load_runtime_stdlib()
        .expect("runtime stdlib registers on engine");

    let mut entry = engine
        .parse_entry_source(
            r"
            /* entrypoint block comment header
               spanning multiple lines */
            use result::Result
            /* runtime error import separator */
            use runtime::RuntimeError
            /* capability import separator */
            use runtime::Args

            fn main() -> Result<(), RuntimeError> { Ok { value: {} } }
        ",
        )
        .expect("entry source with leading block comments should parse");

    engine.check(&mut entry).expect("entry should type check");
    engine
        .verify_entry_definition(&entry)
        .expect("entry should verify");
}

#[test]
fn parses_entry_source_with_whitespace_variants_in_runtime_imports() {
    let engine = Engine::new().build().expect("engine builds");
    engine
        .load_runtime_stdlib()
        .expect("runtime stdlib registers on engine");

    let mut entry = engine
        .parse_entry_source(
            "use\tresult :: Result\nuse runtime :: RuntimeError\nuse runtime::Args\n\nfn main() -> Result<(), RuntimeError> { Ok { value: {} } }\n",
        )
        .expect("entry source with whitespace variants should parse");

    engine.check(&mut entry).expect("entry should type check");
    engine
        .verify_entry_definition(&entry)
        .expect("entry should verify");
}

#[test]
fn parses_entry_source_with_inline_block_comments_inside_runtime_import_paths() {
    let engine = Engine::new().build().expect("engine builds");
    engine
        .load_runtime_stdlib()
        .expect("runtime stdlib registers on engine");

    let mut entry = engine
        .parse_entry_source(
            r"
            use result/* inline */::Result
            use runtime/* inline */::RuntimeError
            use runtime::Args

            fn main() -> Result<(), RuntimeError> { Ok { value: {} } }
        ",
        )
        .expect("entry source with inline block comments in imports should parse");

    engine.check(&mut entry).expect("entry should type check");
    engine
        .verify_entry_definition(&entry)
        .expect("entry should verify");
}

#[test]
fn parses_entry_source_with_trailing_line_comments_on_runtime_imports() {
    let engine = Engine::new().build().expect("engine builds");
    engine
        .load_runtime_stdlib()
        .expect("runtime stdlib registers on engine");

    let mut entry = engine
        .parse_entry_source(
            r"
            use result::Result -- result alias for entry main
            use runtime::RuntimeError -- exit payload type
            use runtime::Args -- injected runtime capability

            fn main() -> Result<(), RuntimeError> { Ok { value: {} } }
        ",
        )
        .expect("entry source with trailing line comments should parse");

    engine.check(&mut entry).expect("entry should type check");
    engine
        .verify_entry_definition(&entry)
        .expect("entry should verify");
}

#[tokio::test]
async fn bootstraps_successful_entry_to_exit_zero() {
    let engine = Engine::new().build().expect("engine builds");

    let exit_code = engine
        .bootstrap_entry_source(
            r"
            use result::Result
            use runtime::RuntimeError

            fn main() -> Result<(), RuntimeError> { Ok { value: {} } }
        ",
        )
        .await
        .expect("bootstrap succeeds");

    assert_eq!(exit_code, 0);
}

#[tokio::test]
async fn bootstraps_runtime_error_to_declared_exit_code() {
    let engine = Engine::new().build().expect("engine builds");

    let exit_code = engine
        .bootstrap_entry_source(
            r#"
            use result::Result
            use runtime::RuntimeError

            fn main() -> Result<(), RuntimeError> {
                Err { error: RuntimeError(42, "boom") }
            }
        "#,
        )
        .await
        .expect("bootstrap succeeds");

    assert_eq!(exit_code, 42);
}

#[tokio::test]
async fn bootstrap_retains_completed_terminal_value_when_runtime_exit_code_is_invalid() {
    use ash_core::Value;
    use ash_core::adt::tuple_field_name;

    let engine = Engine::new().build().expect("engine builds");

    let error = engine
        .bootstrap_entry_source(
            r#"
            use result::Result
            use runtime::RuntimeError

            fn main() -> Result<(), RuntimeError> {
                Err { error: RuntimeError(999, "boom") }
            }
        "#,
        )
        .await
        .expect_err("out-of-range runtime exit code must remain an error");

    assert!(matches!(
        error,
        EntryBootstrapError::InvalidExitCode {
            code: 999,
            terminal_value: Value::Variant { name, fields },
        } if name == "Err" && fields == Box::new(vec![(
            "error".to_owned(),
            Value::Variant {
                name: "RuntimeError".to_owned(),
                fields: Box::new(vec![
                    (tuple_field_name(0), Value::Int(999)),
                    (tuple_field_name(1), Value::String("boom".to_owned())),
                ]),
            },
        )])
    ));
}

#[tokio::test]
async fn bootstrap_rejects_missing_main() {
    let engine = Engine::new().build().expect("engine builds");

    let err = engine
        .bootstrap_entry_source(
            r"
            use result::Result
            use runtime::RuntimeError

            fn other() -> Result<(), RuntimeError> { Ok { value: {} } }
        ",
        )
        .await
        .expect_err("bootstrap should reject missing main");

    assert!(matches!(err, EntryBootstrapError::Engine(_)));
    assert!(err.to_string().contains("expected fn main entry"));
}

#[tokio::test]
async fn bootstrap_tolerates_leading_comments_before_runtime_imports() {
    let engine = Engine::new().build().expect("engine builds");

    let exit_code = engine
        .bootstrap_entry_source(
            r"
            -- entrypoint comment header
            -- another header line
            use result::Result
            use runtime::RuntimeError

            fn main() -> Result<(), RuntimeError> { Ok { value: {} } }
        ",
        )
        .await
        .expect("bootstrap succeeds with leading comments before imports");

    assert_eq!(exit_code, 0);
}

#[tokio::test]
async fn bootstrap_tolerates_block_comments_before_and_between_runtime_imports() {
    let engine = Engine::new().build().expect("engine builds");

    let exit_code = engine
        .bootstrap_entry_source(
            r"
            /* entrypoint block comment header */
            use result::Result
            /* runtime error import separator */
            use runtime::RuntimeError

            fn main() -> Result<(), RuntimeError> { Ok { value: {} } }
        ",
        )
        .await
        .expect("bootstrap succeeds with leading block comments before imports");

    assert_eq!(exit_code, 0);
}

#[tokio::test]
async fn bootstrap_tolerates_inline_block_comments_inside_runtime_import_paths() {
    let engine = Engine::new().build().expect("engine builds");

    let exit_code = engine
        .bootstrap_entry_source(
            r"
            use result/* inline */::Result
            use runtime/* inline */::RuntimeError

            fn main() -> Result<(), RuntimeError> { Ok { value: {} } }
        ",
        )
        .await
        .expect("bootstrap succeeds with inline block comments in imports");

    assert_eq!(exit_code, 0);
}
