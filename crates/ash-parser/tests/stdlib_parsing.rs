//! Integration tests for stdlib parsing
//!
//! These tests verify that the standard library .ash files can be parsed correctly.

use std::fs;
use std::path::PathBuf;

use ash_parser::surface::{Expr, FnDef, Type as SurfaceType};
use ash_parser::{
    Definition, Workflow, input::new_input, parse_module::parse_fn_definition, parse_module_decl,
    parse_type_def::parse_type_def, parse_use, parse_utils::skip_whitespace_and_comments, workflow,
    workflow_def,
};
use winnow::prelude::*;

/// Get the path to the stdlib source directory
fn stdlib_src_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // ash-parser
    path.pop(); // crates
    path.push("std");
    path.push("src");
    path
}

/// Helper to read and return a stdlib file's content
fn read_stdlib_file(filename: &str) -> String {
    let path = stdlib_src_path().join(filename);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e))
}

fn normalize_whitespace(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn parse_capability(source: &str) -> Result<ash_parser::CapabilityDef, String> {
    let normalized = source
        .trim()
        .trim_start_matches("pub ")
        .trim_end_matches(';');
    let wrapped = format!("mod runtime {{ {} }}", normalized);
    let mut input = new_input(&wrapped);
    let decl = parse_module_decl
        .parse_next(&mut input)
        .map_err(|e| format!("{e:?}"))?;

    let definitions = decl.definitions().ok_or("expected inline module")?;

    match &definitions[0] {
        Definition::Capability(cap) => Ok(cap.clone()),
        _ => Err("first definition is not a capability".into()),
    }
}

fn extract_public_fn_sources(source: &str) -> Vec<String> {
    let mut functions = Vec::new();
    let mut current = Vec::new();
    let mut capturing = false;
    let mut brace_depth = 0usize;
    let mut saw_open_brace = false;

    for line in source.lines() {
        let trimmed = line.trim_start();
        if !capturing && trimmed.starts_with("pub fn ") {
            capturing = true;
            current.clear();
            brace_depth = 0;
            saw_open_brace = false;
        }

        if capturing {
            current.push(line);
            for ch in line.chars() {
                match ch {
                    '{' => {
                        brace_depth += 1;
                        saw_open_brace = true;
                    }
                    '}' => {
                        brace_depth = brace_depth.saturating_sub(1);
                    }
                    _ => {}
                }
            }

            if saw_open_brace && brace_depth == 0 {
                functions.push(current.join("\n"));
                current.clear();
                capturing = false;
            }
        }
    }

    functions
}

fn parse_public_functions(filename: &str) -> Vec<FnDef> {
    extract_public_fn_sources(&read_stdlib_file(filename))
        .into_iter()
        .map(|source| {
            let mut input = new_input(&source);
            skip_whitespace_and_comments(&mut input);
            match parse_fn_definition.parse_next(&mut input) {
                Ok(Definition::Function(function)) => function,
                Ok(other) => panic!("expected function definition, got {other:?}"),
                Err(error) => panic!("failed to parse function definition {source:?}: {error}"),
            }
        })
        .collect()
}

#[test]
fn test_option_file_exists() {
    let path = stdlib_src_path().join("option.ash");
    assert!(path.exists(), "option.ash should exist");
}

#[test]
fn test_result_file_exists() {
    let path = stdlib_src_path().join("result.ash");
    assert!(path.exists(), "result.ash should exist");
}

#[test]
fn test_prelude_file_exists() {
    let path = stdlib_src_path().join("prelude.ash");
    assert!(path.exists(), "prelude.ash should exist");
}

#[test]
fn test_lib_file_exists() {
    let path = stdlib_src_path().join("lib.ash");
    assert!(path.exists(), "lib.ash should exist");
}

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
fn test_option_type_definition_parses() {
    let content = read_stdlib_file("option.ash");

    // Extract the type definition line
    let type_def_line = content
        .lines()
        .find(|l| l.contains("pub type Option"))
        .expect("Should find Option type definition");

    let mut input = new_input(type_def_line);
    let result = parse_type_def(&mut input);

    assert!(
        result.is_ok(),
        "Option type definition should parse: {:?}",
        result
    );

    let type_def = result.unwrap();
    assert_eq!(type_def.name, "Option");
    assert_eq!(type_def.params.len(), 1);
    assert_eq!(type_def.params[0], "T");
}

#[test]
fn test_result_type_definition_parses() {
    let content = read_stdlib_file("result.ash");

    // Extract the type definition line
    let type_def_line = content
        .lines()
        .find(|l| l.contains("pub type Result"))
        .expect("Should find Result type definition");

    let mut input = new_input(type_def_line);
    let result = parse_type_def(&mut input);

    assert!(
        result.is_ok(),
        "Result type definition should parse: {:?}",
        result
    );

    let type_def = result.unwrap();
    assert_eq!(type_def.name, "Result");
    assert_eq!(type_def.params.len(), 2);
    assert_eq!(type_def.params[0], "T");
    assert_eq!(type_def.params[1], "E");
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

#[test]
fn test_option_public_functions_parse_as_real_fn_definitions() {
    let functions = parse_public_functions("option.ash");
    let names = functions
        .iter()
        .map(|function| function.name.as_ref())
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        vec![
            "is_some",
            "is_none",
            "unwrap",
            "unwrap_or",
            "map",
            "and_opt",
            "or_opt",
            "ok_or"
        ]
    );

    let unwrap = functions
        .iter()
        .find(|function| function.name.as_ref() == "unwrap")
        .expect("unwrap function should parse");
    assert!(matches!(unwrap.body, Expr::Block { .. }));

    let map = functions
        .iter()
        .find(|function| function.name.as_ref() == "map")
        .expect("map function should parse");
    assert!(matches!(
        map.params[1].ty,
        SurfaceType::Fn(ref params, ref ret)
            if params.len() == 1
                && matches!(params[0], SurfaceType::Name(ref name) if name.as_ref() == "T")
                && matches!(ret.as_ref(), SurfaceType::Name(name) if name.as_ref() == "U")
    ));
}

#[test]
fn test_result_public_functions_parse_as_real_fn_definitions() {
    let functions = parse_public_functions("result.ash");
    let names = functions
        .iter()
        .map(|function| function.name.as_ref())
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        vec![
            "is_ok",
            "is_err",
            "unwrap",
            "unwrap_err",
            "unwrap_or",
            "map",
            "map_err",
            "and_then",
            "ok",
            "err",
        ]
    );

    for function_name in ["map", "map_err", "and_then"] {
        let function = functions
            .iter()
            .find(|function| function.name.as_ref() == function_name)
            .unwrap_or_else(|| panic!("{function_name} function should parse"));
        assert!(matches!(function.params[1].ty, SurfaceType::Fn(_, _)));
    }
}

#[test]
fn test_prelude_contains_use_declarations() {
    let content = read_stdlib_file("prelude.ash");
    assert!(
        content.contains("use option::"),
        "prelude.ash should import from option"
    );
    assert!(
        content.contains("use result::"),
        "prelude.ash should import from result"
    );
}

#[test]
fn test_prelude_contains_re_exports() {
    let content = read_stdlib_file("prelude.ash");
    assert!(
        content.contains("pub use option::"),
        "prelude.ash should re-export from option"
    );
    assert!(
        content.contains("pub use result::"),
        "prelude.ash should re-export from result"
    );
}

#[test]
fn test_lib_contains_all_re_exports() {
    let content = read_stdlib_file("lib.ash");

    // Check for Option and Result types
    assert!(content.contains("Option"), "lib.ash should export Option");
    assert!(content.contains("Result"), "lib.ash should export Result");

    // Check for Some, None, Ok, Err constructors
    assert!(content.contains("Some"), "lib.ash should export Some");
    assert!(content.contains("None"), "lib.ash should export None");
    assert!(content.contains("Ok"), "lib.ash should export Ok");
    assert!(content.contains("Err"), "lib.ash should export Err");
}

#[test]
fn test_option_has_documentation_comments() {
    let content = read_stdlib_file("option.ash");
    // Check for module-level doc comment
    assert!(
        content.contains("-- Option type"),
        "option.ash should have module documentation"
    );
    // Check for function-level doc comments
    assert!(
        content.contains("-- Returns true"),
        "option.ash functions should have documentation"
    );
}

#[test]
fn test_result_has_documentation_comments() {
    let content = read_stdlib_file("result.ash");
    // Check for module-level doc comment
    assert!(
        content.contains("-- Result type"),
        "result.ash should have module documentation"
    );
    // Check for function-level doc comments
    assert!(
        content.contains("-- Returns true"),
        "result.ash functions should have documentation"
    );
}

#[test]
fn test_option_has_all_required_functions() {
    let content = read_stdlib_file("option.ash");

    let required_functions = [
        "is_some",
        "is_none",
        "unwrap",
        "unwrap_or",
        "map",
        "and_opt",
        "or_opt",
        "ok_or",
    ];

    for func in &required_functions {
        assert!(
            content.contains(&format!("pub fn {}", func)),
            "option.ash should contain {} function",
            func
        );
    }
}

#[test]
fn test_result_has_all_required_functions() {
    let content = read_stdlib_file("result.ash");

    let required_functions = [
        "is_ok",
        "is_err",
        "unwrap",
        "unwrap_err",
        "unwrap_or",
        "map",
        "map_err",
        "and_then",
        "ok",
        "err",
    ];

    for func in &required_functions {
        assert!(
            content.contains(&format!("pub fn {}", func)),
            "result.ash should contain {} function",
            func
        );
    }
}

#[test]
fn test_stdlib_readme_exists() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // ash-parser
    path.pop(); // crates
    path.push("std");
    path.push("README.md");

    assert!(path.exists(), "std/README.md should exist");

    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("# Ash Standard Library"),
        "README should have title"
    );
    assert!(content.contains("Option"), "README should document Option");
    assert!(content.contains("Result"), "README should document Result");
}

#[test]
fn test_stdlib_cargo_toml_exists() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // ash-parser
    path.pop(); // crates
    path.push("std");
    path.push("Cargo.toml");

    assert!(path.exists(), "std/Cargo.toml should exist");

    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("name = \"ash-std\""),
        "Cargo.toml should have correct name"
    );
}

// TASK-494: io module parsing tests
// These tests will fail until the io module and io::path are properly implemented

#[test]
fn test_io_mod_file_exists() {
    let path = stdlib_src_path().join("io/mod.ash");
    assert!(path.exists(), "io/mod.ash should exist");
}

#[test]
fn test_io_path_file_exists() {
    let path = stdlib_src_path().join("io/path.ash");
    assert!(path.exists(), "io/path.ash should exist");
}

#[test]
fn test_io_import_examples_parse_with_canonical_syntax() {
    for source in [
        "use io::path;",
        "use io::path::PathBuf;",
        "use io::{Error, ErrorKind};",
        "use io::path::{PathBuf, from_string};",
    ] {
        let mut input = new_input(source);
        let result = parse_use(&mut input);

        assert!(result.is_ok(), "io import should parse: {source}");
    }
}

#[test]
fn test_io_path_type_definition_parses() {
    let content = read_stdlib_file("io/path.ash");

    // Extract the PathBuf type definition line
    let type_def_line = content
        .lines()
        .find(|l| l.contains("pub type PathBuf"))
        .expect("Should find PathBuf type definition");

    let mut input = new_input(type_def_line);
    let result = parse_type_def(&mut input);

    assert!(
        result.is_ok(),
        "PathBuf type definition should parse: {:?}",
        result
    );

    let type_def = result.unwrap();
    assert_eq!(type_def.name, "PathBuf");
}

#[test]
fn test_io_error_type_definition_parses() {
    let content = read_stdlib_file("io/mod.ash");

    // Extract the Error type definition line (not ErrorKind)
    let type_def_line = content
        .lines()
        .find(|l| l.contains("pub type Error ="))
        .expect("Should find Error type definition");

    let mut input = new_input(type_def_line);
    let result = parse_type_def(&mut input);

    assert!(
        result.is_ok(),
        "Error type definition should parse: {:?}",
        result
    );

    let type_def = result.unwrap();
    assert_eq!(type_def.name, "Error");
}

#[test]
fn test_io_path_public_functions_parse_as_real_fn_definitions() {
    let functions = parse_public_functions("io/path.ash");
    let names = functions
        .iter()
        .map(|function| function.name.as_ref())
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        vec![
            "from_string",
            "join",
            "parent",
            "file_name",
            "extension",
            "is_absolute",
        ]
    );

    let join = functions
        .iter()
        .find(|function| function.name.as_ref() == "join")
        .expect("join function should parse");
    assert!(matches!(join.body, Expr::Block { .. }));

    let parent = functions
        .iter()
        .find(|function| function.name.as_ref() == "parent")
        .expect("parent function should parse");
    assert!(matches!(parent.body, Expr::Block { .. }));
}

#[test]
fn test_io_mod_exports_path() {
    let content = read_stdlib_file("io/mod.ash");

    assert!(
        content.contains("mod path;"),
        "io/mod.ash should declare path module"
    );
    assert!(
        content.contains("pub use path::"),
        "io/mod.ash should re-export from path module"
    );
}

#[test]
fn test_io_mod_exports_error_types() {
    let content = read_stdlib_file("io/mod.ash");

    assert!(
        content.contains("pub type Error"),
        "io/mod.ash should export Error type"
    );
    assert!(
        content.contains("pub type ErrorKind"),
        "io/mod.ash should export ErrorKind type"
    );
    assert!(
        content.contains("pub type Result<T>"),
        "io/mod.ash should export Result<T> type alias"
    );
}

#[test]
fn test_lib_exports_io() {
    let content = read_stdlib_file("lib.ash");

    assert!(
        content.contains("pub use io::"),
        "lib.ash should export from io module"
    );
}

#[test]
fn test_io_path_usage_example_parses() {
    let path_functions = parse_public_functions("io/path.ash");
    let mod_content = read_stdlib_file("io/mod.ash");

    assert!(
        path_functions
            .iter()
            .any(|function| function.name.as_ref() == "from_string")
    );
    assert!(
        path_functions
            .iter()
            .any(|function| function.name.as_ref() == "join")
    );
    assert!(
        mod_content.contains("pub use path::"),
        "io mod should re-export from path"
    );
}

#[test]
fn test_io_path_all_required_functions_exist() {
    let functions = parse_public_functions("io/path.ash");
    let required_functions = [
        "from_string",
        "join",
        "parent",
        "file_name",
        "extension",
        "is_absolute",
    ];

    for func in &required_functions {
        assert!(
            functions
                .iter()
                .any(|function| function.name.as_ref() == *func),
            "io/path.ash should contain {func} function"
        );
    }
}

// TASK-495: io::stdio module parsing tests
// These tests will fail until the io::stdio module is properly implemented

#[test]
fn test_io_stdio_file_exists() {
    let path = stdlib_src_path().join("io/stdio.ash");
    assert!(path.exists(), "io/stdio.ash should exist");
}

#[test]
fn test_io_stdio_read_line_function_parses() {
    let content = read_stdlib_file("io/stdio.ash");

    // Check that read_line function exists and is public
    assert!(
        content.contains("pub fn read_line"),
        "io/stdio.ash should contain read_line function"
    );
}

#[test]
fn test_io_stdio_print_function_parses() {
    let content = read_stdlib_file("io/stdio.ash");

    assert!(
        content.contains("pub fn print"),
        "io/stdio.ash should contain print function"
    );
}

#[test]
fn test_io_stdio_println_function_parses() {
    let content = read_stdlib_file("io/stdio.ash");

    assert!(
        content.contains("pub fn println"),
        "io/stdio.ash should contain println function"
    );
}

#[test]
fn test_io_stdio_capability_parses() {
    let content = read_stdlib_file("io/stdio.ash");

    assert!(
        content.contains("pub capability Stdio"),
        "io/stdio.ash should declare Stdio capability"
    );
}

#[test]
fn test_io_mod_exports_stdio() {
    let content = read_stdlib_file("io/mod.ash");

    assert!(
        content.contains("mod stdio;"),
        "io/mod.ash should declare stdio module"
    );
    assert!(
        content.contains("pub use stdio::"),
        "io/mod.ash should re-export from stdio module"
    );
}

#[test]
fn test_io_stdio_all_required_functions_exist() {
    let content = read_stdlib_file("io/stdio.ash");

    let required_functions = ["read_line", "print", "println"];

    for func in &required_functions {
        assert!(
            content.contains(&format!("pub fn {}", func)),
            "io/stdio.ash should contain {} function",
            func
        );
    }
}

#[test]
fn test_io_stdio_import_examples_parse_with_canonical_syntax() {
    for source in [
        "use io::stdio;",
        "use io::stdio::read_line;",
        "use io::stdio::{print, println};",
        "use io::{read_line, print, println};",
    ] {
        let mut input = new_input(source);
        let result = parse_use(&mut input);

        assert!(result.is_ok(), "io::stdio import should parse: {source}");
    }
}

// TASK-496: io::fs, io::dir, io::meta module parsing tests
// These tests will fail until the filesystem modules are properly implemented

#[test]
fn test_io_fs_file_exists() {
    let path = stdlib_src_path().join("io/fs.ash");
    assert!(path.exists(), "io/fs.ash should exist");
}

#[test]
fn test_io_dir_file_exists() {
    let path = stdlib_src_path().join("io/dir.ash");
    assert!(path.exists(), "io/dir.ash should exist");
}

#[test]
fn test_io_meta_file_exists() {
    let path = stdlib_src_path().join("io/meta.ash");
    assert!(path.exists(), "io/meta.ash should exist");
}

#[test]
fn test_io_fs_capability_parses() {
    let content = read_stdlib_file("io/fs.ash");

    assert!(
        content.contains("pub capability Fs"),
        "io/fs.ash should declare Fs capability"
    );
}

#[test]
fn test_io_dir_capability_parses() {
    let content = read_stdlib_file("io/dir.ash");

    assert!(
        content.contains("pub capability Dir"),
        "io/dir.ash should declare Dir capability"
    );
}

#[test]
fn test_io_meta_capability_parses() {
    let content = read_stdlib_file("io/meta.ash");

    assert!(
        content.contains("pub capability Meta"),
        "io/meta.ash should declare Meta capability"
    );
}

#[test]
fn test_io_fs_import_examples_parse_with_canonical_syntax() {
    for source in [
        "use io::fs;",
        "use io::fs::read;",
        "use io::fs::{read, write};",
        "use io::{read, write_string};",
    ] {
        let mut input = new_input(source);
        let result = parse_use(&mut input);

        assert!(result.is_ok(), "io::fs import should parse: {source}");
    }
}

#[test]
fn test_io_dir_import_examples_parse_with_canonical_syntax() {
    for source in [
        "use io::dir;",
        "use io::dir::create_dir;",
        "use io::dir::{create_dir, remove_dir};",
        "use io::{create_dir_all, read_dir};",
    ] {
        let mut input = new_input(source);
        let result = parse_use(&mut input);

        assert!(result.is_ok(), "io::dir import should parse: {source}");
    }
}

#[test]
fn test_io_meta_import_examples_parse_with_canonical_syntax() {
    for source in [
        "use io::meta;",
        "use io::meta::metadata;",
        "use io::meta::{metadata, is_file};",
        "use io::{is_dir, len, readonly};",
    ] {
        let mut input = new_input(source);
        let result = parse_use(&mut input);

        assert!(result.is_ok(), "io::meta import should parse: {source}");
    }
}

#[test]
fn test_io_fs_read_function_parses() {
    let content = read_stdlib_file("io/fs.ash");
    assert!(
        content.contains("pub fn read"),
        "io/fs.ash should contain read function"
    );
}

#[test]
fn test_io_fs_read_to_string_function_parses() {
    let content = read_stdlib_file("io/fs.ash");
    assert!(
        content.contains("pub fn read_to_string"),
        "io/fs.ash should contain read_to_string function"
    );
}

#[test]
fn test_io_fs_write_function_parses() {
    let content = read_stdlib_file("io/fs.ash");
    assert!(
        content.contains("pub fn write"),
        "io/fs.ash should contain write function"
    );
}

#[test]
fn test_io_fs_write_string_function_parses() {
    let content = read_stdlib_file("io/fs.ash");
    assert!(
        content.contains("pub fn write_string"),
        "io/fs.ash should contain write_string function"
    );
}

#[test]
fn test_io_fs_append_function_parses() {
    let content = read_stdlib_file("io/fs.ash");
    assert!(
        content.contains("pub fn append"),
        "io/fs.ash should contain append function"
    );
}

#[test]
fn test_io_fs_copy_function_parses() {
    let content = read_stdlib_file("io/fs.ash");
    assert!(
        content.contains("pub fn copy"),
        "io/fs.ash should contain copy function"
    );
}

#[test]
fn test_io_fs_rename_function_parses() {
    let content = read_stdlib_file("io/fs.ash");
    assert!(
        content.contains("pub fn rename"),
        "io/fs.ash should contain rename function"
    );
}

#[test]
fn test_io_fs_remove_file_function_parses() {
    let content = read_stdlib_file("io/fs.ash");
    assert!(
        content.contains("pub fn remove_file"),
        "io/fs.ash should contain remove_file function"
    );
}

#[test]
fn test_io_dir_create_dir_function_parses() {
    let content = read_stdlib_file("io/dir.ash");
    assert!(
        content.contains("pub fn create_dir"),
        "io/dir.ash should contain create_dir function"
    );
}

#[test]
fn test_io_dir_create_dir_all_function_parses() {
    let content = read_stdlib_file("io/dir.ash");
    assert!(
        content.contains("pub fn create_dir_all"),
        "io/dir.ash should contain create_dir_all function"
    );
}

#[test]
fn test_io_dir_remove_dir_function_parses() {
    let content = read_stdlib_file("io/dir.ash");
    assert!(
        content.contains("pub fn remove_dir"),
        "io/dir.ash should contain remove_dir function"
    );
}

#[test]
fn test_io_dir_remove_dir_all_function_parses() {
    let content = read_stdlib_file("io/dir.ash");
    assert!(
        content.contains("pub fn remove_dir_all"),
        "io/dir.ash should contain remove_dir_all function"
    );
}

#[test]
fn test_io_dir_read_dir_function_parses() {
    let content = read_stdlib_file("io/dir.ash");
    assert!(
        content.contains("pub fn read_dir"),
        "io/dir.ash should contain read_dir function"
    );
}

#[test]
fn test_io_meta_metadata_function_parses() {
    let content = read_stdlib_file("io/meta.ash");
    assert!(
        content.contains("pub fn metadata"),
        "io/meta.ash should contain metadata function"
    );
}

#[test]
fn test_io_meta_is_file_function_parses() {
    let content = read_stdlib_file("io/meta.ash");
    assert!(
        content.contains("pub fn is_file"),
        "io/meta.ash should contain is_file function"
    );
}

#[test]
fn test_io_meta_is_dir_function_parses() {
    let content = read_stdlib_file("io/meta.ash");
    assert!(
        content.contains("pub fn is_dir"),
        "io/meta.ash should contain is_dir function"
    );
}

#[test]
fn test_io_meta_len_function_parses() {
    let content = read_stdlib_file("io/meta.ash");
    assert!(
        content.contains("pub fn len"),
        "io/meta.ash should contain len function"
    );
}

#[test]
fn test_io_meta_readonly_function_parses() {
    let content = read_stdlib_file("io/meta.ash");
    assert!(
        content.contains("pub fn readonly"),
        "io/meta.ash should contain readonly function"
    );
}

#[test]
fn test_io_mod_exports_fs() {
    let content = read_stdlib_file("io/mod.ash");

    assert!(
        content.contains("mod fs;"),
        "io/mod.ash should declare fs module"
    );
    assert!(
        content.contains("pub use fs::"),
        "io/mod.ash should re-export from fs module"
    );
}

#[test]
fn test_io_mod_exports_dir() {
    let content = read_stdlib_file("io/mod.ash");

    assert!(
        content.contains("mod dir;"),
        "io/mod.ash should declare dir module"
    );
    assert!(
        content.contains("pub use dir::"),
        "io/mod.ash should re-export from dir module"
    );
}

#[test]
fn test_io_mod_exports_meta() {
    let content = read_stdlib_file("io/mod.ash");

    assert!(
        content.contains("mod meta;"),
        "io/mod.ash should declare meta module"
    );
    assert!(
        content.contains("pub use meta::"),
        "io/mod.ash should re-export from meta module"
    );
}

#[test]
fn test_io_fs_all_required_functions_exist() {
    let content = read_stdlib_file("io/fs.ash");

    let required_functions = [
        "read",
        "read_to_string",
        "write",
        "write_string",
        "append",
        "copy",
        "rename",
        "remove_file",
    ];

    for func in &required_functions {
        assert!(
            content.contains(&format!("pub fn {}", func)),
            "io/fs.ash should contain {} function",
            func
        );
    }
}

#[test]
fn test_io_dir_all_required_functions_exist() {
    let content = read_stdlib_file("io/dir.ash");

    let required_functions = [
        "create_dir",
        "create_dir_all",
        "remove_dir",
        "remove_dir_all",
        "read_dir",
    ];

    for func in &required_functions {
        assert!(
            content.contains(&format!("pub fn {}", func)),
            "io/dir.ash should contain {} function",
            func
        );
    }
}

#[test]
fn test_io_meta_all_required_functions_exist() {
    let content = read_stdlib_file("io/meta.ash");

    let required_functions = ["metadata", "is_file", "is_dir", "len", "readonly"];

    for func in &required_functions {
        assert!(
            content.contains(&format!("pub fn {}", func)),
            "io/meta.ash should contain {} function",
            func
        );
    }
}

// TASK-497: io::buf module parsing tests
// These tests will fail until the buffered helpers module is properly implemented

#[test]
fn test_io_buf_file_exists() {
    let path = stdlib_src_path().join("io/buf.ash");
    assert!(path.exists(), "io/buf.ash should exist");
}

#[test]
fn test_io_buf_read_to_end_function_parses() {
    let content = read_stdlib_file("io/buf.ash");
    assert!(
        content.contains("pub fn read_to_end"),
        "io/buf.ash should contain read_to_end function"
    );
}

#[test]
fn test_io_buf_read_to_string_function_parses() {
    let content = read_stdlib_file("io/buf.ash");
    assert!(
        content.contains("pub fn read_to_string"),
        "io/buf.ash should contain read_to_string function"
    );
}

#[test]
fn test_io_buf_write_all_function_parses() {
    let content = read_stdlib_file("io/buf.ash");
    assert!(
        content.contains("pub fn write_all"),
        "io/buf.ash should contain write_all function"
    );
}

#[test]
fn test_io_buf_lines_function_parses() {
    let content = read_stdlib_file("io/buf.ash");
    assert!(
        content.contains("pub fn lines"),
        "io/buf.ash should contain lines function"
    );
}

#[test]
fn test_io_mod_exports_buf() {
    let content = read_stdlib_file("io/mod.ash");

    assert!(
        content.contains("mod buf;"),
        "io/mod.ash should declare buf module"
    );
    assert!(
        content.contains("pub use buf::"),
        "io/mod.ash should re-export from buf module"
    );
}

#[test]
fn test_io_buf_all_required_functions_exist() {
    let content = read_stdlib_file("io/buf.ash");

    let required_functions = ["read_to_end", "read_to_string", "write_all", "lines"];

    for func in &required_functions {
        assert!(
            content.contains(&format!("pub fn {}", func)),
            "io/buf.ash should contain {} function",
            func
        );
    }
}
