use super::support::*;

#[test]
fn test_runtime_mod_file_exists() {
    let path = stdlib_src_path().join("runtime/mod.ash");
    assert!(path.exists(), "runtime/mod.ash should exist");
}

#[test]
fn test_runtime_error_file_exists() {
    let path = stdlib_src_path().join("runtime/error.ash");
    assert!(path.exists(), "runtime/error.ash should exist");
}

#[test]
fn test_runtime_args_file_exists() {
    let path = stdlib_src_path().join("runtime/args.ash");
    assert!(path.exists(), "runtime/args.ash should exist");
}

#[test]
fn test_runtime_supervisor_file_exists() {
    let path = stdlib_src_path().join("runtime/supervisor.ash");
    assert!(path.exists(), "runtime/supervisor.ash should exist");
}

#[test]
fn test_runtime_error_type_definition_parses() {
    let content = read_stdlib_file("runtime/error.ash");
    let normalized = normalize_whitespace(&content);

    assert!(
        normalized.contains("pub type RuntimeError = RuntimeError(Int, String);"),
        "RuntimeError should use the canonical tuple-variant ADT syntax"
    );
    assert!(
        !normalized.contains("pub type RuntimeError = RuntimeError {"),
        "RuntimeError should reject record-payload constructor syntax in the stdlib surface"
    );

    let mut input = new_input(&content);
    let result = parse_type_def(&mut input);

    assert!(
        result.is_ok(),
        "RuntimeError type definition should parse: {:?}",
        result
    );

    let type_def = result.unwrap();
    assert_eq!(type_def.name, "RuntimeError");
    assert!(type_def.params.is_empty());

    let variants = match &type_def.body {
        ash_parser::parse_type_def::TypeBody::Enum(variants) => variants,
        other => {
            panic!("RuntimeError body should parse as a single-variant enum ADT, got {other:?}")
        }
    };

    assert_eq!(
        variants.len(),
        1,
        "RuntimeError should have exactly one variant"
    );

    let variant = &variants[0];
    assert_eq!(variant.name, "RuntimeError");
    assert!(
        variant.fields.is_empty(),
        "RuntimeError tuple variants should preserve payload shape without record field names at the parser surface"
    );
    assert!(matches!(
        variant.payload,
        ash_parser::parse_type_def::VariantPayload::Tuple(ref items) if items.len() == 2
    ));
}

#[test]
fn test_runtime_args_builtin_type_definition_parses() {
    let content = read_stdlib_file("runtime/args.ash");
    let use_line = content
        .lines()
        .find(|l| l.trim_start().starts_with("use option::Option;"))
        .expect("Should find Option import in runtime/args.ash");
    let mut use_input = new_input(use_line);
    assert!(
        parse_use(&mut use_input).is_ok(),
        "runtime/args.ash should use canonical stdlib import syntax"
    );

    assert!(
        content.contains("pub builtin type Args;"),
        "runtime/args.ash should expose Args as a builtin target type"
    );
    assert!(!content.contains("pub capability Args"));
}

#[test]
fn test_runtime_args_usage_surface_parses() {
    let source = r#"
        fn main(args: capability Args) -> Int { 0 }
    "#;

    let module = ash_parser::parse_surface_file(source)
        .expect("Args usage surface should parse as target fn");
    let ash_parser::surface::Definition::Function(function) = &module.definitions[0] else {
        panic!("expected target fn definition");
    };
    assert_eq!(function.params.len(), 1);
    assert!(matches!(
        &function.params[0].ty,
        ash_parser::Type::Capability(name) if name.as_ref() == "Args"
    ));
}

#[test]
fn test_runtime_supervisor_target_signature_is_declared() {
    let content = read_stdlib_file("runtime/supervisor.ash");
    for use_line in content
        .lines()
        .filter(|line| line.trim_start().starts_with("use "))
    {
        let mut use_input = new_input(use_line.trim());
        assert!(
            parse_use(&mut use_input).is_ok(),
            "system supervisor imports should parse: {use_line}"
        );
    }

    assert!(
        content.contains("pub fn system_supervisor(args: capability Args) -> Int {"),
        "system_supervisor contract should expose the canonical signature"
    );
    assert!(
        content.contains("use result::{Result, Err};"),
        "system_supervisor should import the canonical Result surface"
    );
    assert!(
        content.contains("Result<(), RuntimeError>"),
        "system_supervisor should document the terminal Result contract"
    );
    assert!(
        !content.contains("parser-feasible stand-in"),
        "system_supervisor should drop the parser-feasible completion placeholder wording"
    );
    assert!(
        !content.contains("supervisor_completion"),
        "system_supervisor should reject the unresolved supervisor_completion placeholder"
    );
    assert!(
        !content.contains("let completion="),
        "system_supervisor should not bind a fake completion payload"
    );
    assert!(
        !content.contains("return 0;"),
        "system_supervisor should reject the old placeholder return body"
    );
    assert!(
        !content.contains("await"),
        "system_supervisor should not introduce await syntax"
    );
    assert!(
        content.contains("if let Err"),
        "system_supervisor should keep the if-let exit-code shaping intent"
    );
    assert!(
        content.contains("Err { error: RuntimeError(code, _) }"),
        "system_supervisor should keep nested RuntimeError destructuring intent"
    );
    assert!(
        content.contains("then code else 0"),
        "system_supervisor should keep the fallback exit-code shaping intent"
    );
    assert!(
        content.contains("exit_code"),
        "system_supervisor should return the shaped exit code"
    );

    assert!(
        !content.contains("pub workflow system_supervisor"),
        "system_supervisor must not use removed workflow declaration syntax"
    );
}

#[test]
fn test_runtime_import_examples_parse_with_canonical_syntax() {
    for source in [
        "use runtime::RuntimeError;",
        "use runtime::Args;",
        "use runtime::{RuntimeError, Args};",
    ] {
        let mut input = new_input(source);
        let result = parse_use(&mut input);

        assert!(result.is_ok(), "runtime import should parse: {source}");
    }
}
