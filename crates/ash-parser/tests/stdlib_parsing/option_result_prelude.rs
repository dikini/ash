use super::support::*;

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
            "pure",
            "apply",
            "and_then",
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
        SurfaceType::Fn(ref params, _, ref ret)
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
            "pure",
            "apply",
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
        assert!(matches!(function.params[1].ty, SurfaceType::Fn(_, _, _)));
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
            contains_public_callable(&content, func),
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
            contains_public_callable(&content, func),
            "result.ash should contain {} function",
            func
        );
    }
}
