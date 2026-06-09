use super::support::*;

#[test]
fn test_io_path_join_preserves_absolute_law_parses() {
    let source = read_stdlib_file("io/path.ash");
    let module = ash_parser::parse_surface_file(&source)
        .unwrap_or_else(|errors| panic!("io/path.ash should parse: {errors:?}\n{source}"));

    let law = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Law(law) if law.name.as_ref() == "join_preserves_absolute" => Some(law),
            _ => None,
        })
        .expect("io/path.ash should declare module law join_preserves_absolute");

    assert_eq!(law.params.len(), 2);
    assert_eq!(law.params[0].name.as_ref(), "base");
    assert!(
        matches!(&law.params[0].ty, SurfaceType::Name(name) if name.as_ref() == "PathBuf"),
        "base parameter should have PathBuf type, got {:?}",
        law.params[0].ty
    );
    assert_eq!(law.params[1].name.as_ref(), "child");
    assert!(
        matches!(&law.params[1].ty, SurfaceType::Name(name) if name.as_ref() == "String"),
        "child parameter should have String type, got {:?}",
        law.params[1].ty
    );
}

#[test]
fn test_io_path_file_exists() {
    let path = stdlib_src_path().join("io/path.ash");
    assert!(path.exists(), "io/path.ash should exist");
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
            "preserves_absolute_after_join",
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
