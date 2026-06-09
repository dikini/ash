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
fn test_runtime_args_capability_definition_parses() {
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

    let capability_line = content
        .lines()
        .find(|l| l.contains("pub capability Args"))
        .expect("Should find Args capability definition");

    let capability = parse_capability(capability_line).expect("Args capability should parse");

    assert_eq!(capability.name.as_ref(), "Args");
    assert_eq!(capability.params.len(), 1);
    assert_eq!(capability.params[0].name.as_ref(), "index");
    assert!(capability.return_type.is_some());
}

#[test]
fn test_runtime_args_usage_surface_parses() {
    let source = r#"
        workflow main(args: cap Args) {
            observe Args 0;
            done;
        }
    "#;

    let mut input = new_input(source);
    let result = workflow_def(&mut input);

    assert!(
        result.is_ok(),
        "Args usage surface should parse: {:?}",
        result
    );

    let workflow = result.unwrap();
    assert_eq!(workflow.params.len(), 1);
    assert!(matches!(
        &workflow.params[0].ty,
        ash_parser::Type::Capability(name) if name.as_ref() == "Args"
    ));

    match workflow.body {
        Workflow::Seq { first, .. } => match *first {
            Workflow::Observe { capability, .. } => {
                assert_eq!(capability.as_ref(), "Args:0");
            }
            other => panic!("Expected observe statement, got {other:?}"),
        },
        other => panic!("Expected sequential workflow body, got {other:?}"),
    }
}

#[test]
fn test_runtime_supervisor_workflow_definition_parses() {
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
        content.contains("pub workflow system_supervisor(args: cap Args) -> Int {"),
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
        !content.contains("ret 0;"),
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
        content.contains("ret exit_code;"),
        "system_supervisor should return the shaped exit code"
    );

    let workflow_source = content
        .lines()
        .skip_while(|line| {
            !line
                .trim_start()
                .starts_with("pub workflow system_supervisor")
        })
        .collect::<Vec<_>>()
        .join("\n");

    let workflow_body_start = workflow_source
        .find('{')
        .expect("system_supervisor definition should contain an opening brace");
    let workflow_body_end = workflow_source
        .rfind('}')
        .expect("system_supervisor definition should contain a closing brace");
    let body_source = &workflow_source[(workflow_body_start + 1)..workflow_body_end];

    let mut input = new_input(body_source);
    let result = workflow(&mut input);

    assert!(
        result.is_ok(),
        "system_supervisor body should parse: {:?}",
        result
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
