#[test]
fn task590_debug_scan_tree_parse_minimal() {
    use ash_parser::input::new_input;
    use ash_parser::parse_module::parse_fn_definition;
    use winnow::prelude::Parser;

    let cases = [
        r#"pub fn scan_tree(root: String) -> FileTree { root }"#,
        r#"pub fn scan_tree(root: String) { FileTree { spec_files: [] } }"#,
        r#"pub fn scan_tree(root: String) -> FileTree { FileTree { spec_files: [] } }"#,
        r#"pub fn scan_tree(root: String) -> FileTree { FileTree { spec_files: [], plan_files: [] } }"#,
    ];
    for (i, snippet) in cases.iter().enumerate() {
        let mut input = new_input(snippet);
        let result = parse_fn_definition.parse_next(&mut input);
        eprintln!(
            "Case {}: {:?} -> {}",
            i,
            snippet,
            if result.is_ok() { "OK" } else { "FAIL" }
        );
        if let Err(ref e) = result {
            eprintln!("  Error: {}", e);
        }
    }
}

#[test]
#[ignore = "TODO(TASK-590)"]
fn task590_debug_collect_ash_file_parse() {
    use ash_parser::input::new_input;
    use ash_parser::parse_module::parse_fn_definition;
    use winnow::prelude::Parser;

    let source_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/plan/PLAN-090-SPEC-PROCESSOR.md");
    let source = std::fs::read_to_string(&source_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", source_path.display()));
    let lines: Vec<&str> = source.lines().collect();
    let mut snippet = String::new();
    let mut in_snippet = false;
    let mut brace_depth = 0usize;
    let mut seen_open = false;

    for line in &lines {
        let trimmed = line.trim_start();
        if !in_snippet && trimmed.starts_with("pub fn ") {
            in_snippet = true;
            snippet.clear();
            brace_depth = 0;
            seen_open = false;
        }
        if in_snippet {
            if !snippet.is_empty() {
                snippet.push('\n');
            }
            snippet.push_str(line);
            for ch in line.chars() {
                match ch {
                    '{' => {
                        brace_depth += 1;
                        seen_open = true;
                    }
                    '}' => {
                        brace_depth -= 1;
                    }
                    _ => {}
                }
            }
            if seen_open && brace_depth == 0 {
                break;
            }
        }
    }

    eprintln!("Extracted snippet:\n{}", snippet);
    eprintln!("Snippet bytes: {:?}", snippet.as_bytes());

    let mut input = new_input(&snippet);
    let result = parse_fn_definition.parse_next(&mut input);
    if let Err(ref e) = result {
        eprintln!("Parse failed: {}", e);
    }
    assert!(
        result.is_ok(),
        "expected parse to succeed, got: {:?}",
        result
    );
}

#[test]
fn task590_debug_multiline_record_constructor() {
    use ash_parser::input::new_input;
    use ash_parser::parse_module::parse_fn_definition;
    use winnow::prelude::Parser;

    let cases = [
        (
            r#"pub fn scan_tree(root: String) -> FileTree { FileTree { spec_files: [] } }"#,
            true,
        ),
        (
            r#"pub fn scan_tree(root: String) -> FileTree {
    FileTree { spec_files: [] }
}"#,
            true,
        ),
        (
            r#"pub fn scan_tree(root: String) -> FileTree {
    FileTree {
        spec_files: []
    }
}"#,
            true,
        ),
    ];
    for (i, (snippet, expected)) in cases.iter().enumerate() {
        let mut input = new_input(snippet);
        let result = parse_fn_definition.parse_next(&mut input);
        eprintln!(
            "Case {}: expected={} actual={}",
            i,
            expected,
            result.is_ok()
        );
        if let Err(ref e) = result {
            eprintln!("  Error: {}", e);
        }
        assert_eq!(result.is_ok(), *expected);
    }
}

// TODO(TASK-590): known failure — parser gap with multiline record constructor + trailing comma.
#[test]
#[ignore = "TODO(TASK-590)"]
fn task590_debug_exact_file_snippet() {
    use ash_parser::input::new_input;
    use ash_parser::parse_module::parse_fn_definition;
    use winnow::prelude::Parser;

    let snippet = r#"pub fn scan_tree(root: String) -> FileTree {
    FileTree {
        spec_files: [],
        plan_files: [],
        example_files: [],
        changelog_files: [],
    }
}"#;
    let mut input = new_input(snippet);
    let result = parse_fn_definition.parse_next(&mut input);
    eprintln!("Result: {}", if result.is_ok() { "OK" } else { "FAIL" });
    if let Err(ref e) = result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok(), "got: {:?}", result);
}

#[test]
fn task590_debug_field_count_isolation() {
    use ash_parser::input::new_input;
    use ash_parser::parse_module::parse_fn_definition;
    use winnow::prelude::Parser;

    let cases = [
        (
            "1 field",
            r#"pub fn scan_tree(root: String) -> FileTree {
    FileTree { spec_files: [] }
}"#,
        ),
        (
            "2 fields inline",
            r#"pub fn scan_tree(root: String) -> FileTree {
    FileTree { spec_files: [], plan_files: [] }
}"#,
        ),
        (
            "2 fields multiline",
            r#"pub fn scan_tree(root: String) -> FileTree {
    FileTree {
        spec_files: [],
        plan_files: []
    }
}"#,
        ),
        (
            "3 fields multiline trailing comma",
            r#"pub fn scan_tree(root: String) -> FileTree {
    FileTree {
        spec_files: [],
        plan_files: [],
        example_files: [],
    }
}"#,
        ),
        (
            "4 fields multiline trailing comma",
            r#"pub fn scan_tree(root: String) -> FileTree {
    FileTree {
        spec_files: [],
        plan_files: [],
        example_files: [],
        changelog_files: [],
    }
}"#,
        ),
    ];
    for (name, snippet) in cases.iter() {
        let mut input = new_input(snippet);
        let result = parse_fn_definition.parse_next(&mut input);
        eprintln!("{}: {}", name, if result.is_ok() { "OK" } else { "FAIL" });
    }
}

#[test]
fn task590_debug_let_then_constructor() {
    use ash_parser::input::new_input;
    use ash_parser::parse_module::parse_fn_definition;
    use winnow::prelude::Parser;

    let cases = [
        r#"pub fn scan_tree(root: String) -> FileTree {
    let all = collect_files([], root);
    FileTree { spec_files: all }
}"#,
        r#"pub fn scan_tree(root: String) -> FileTree {
    let all = [];
    FileTree { spec_files: all }
}"#,
        r#"pub fn scan_tree(root: String) -> FileTree {
    let all = root;
    FileTree { spec_files: [], plan_files: [], example_files: [], changelog_files: [] }
}"#,
    ];
    for (i, snippet) in cases.iter().enumerate() {
        let mut input = new_input(snippet);
        let result = parse_fn_definition.parse_next(&mut input);
        eprintln!("Case {}: {}", i, if result.is_ok() { "OK" } else { "FAIL" });
        if let Err(ref e) = result {
            eprintln!("  Error: {}", e);
        }
    }
}

// TODO(TASK-590): known failure — parser gap with record constructor containing closure arguments.
#[test]
#[ignore = "TODO(TASK-590)"]
fn task590_debug_long_constructor() {
    use ash_parser::input::new_input;
    use ash_parser::parse_module::parse_fn_definition;
    use winnow::prelude::Parser;

    let snippet = r#"pub fn scan_tree(root: String) -> FileTree {
    let all = collect_files([], root);
    FileTree { spec_files: filter(all, fn(p) { starts_with(p, "SPEC-") && ends_with(p, ".md") }), plan_files: filter(all, fn(p) { starts_with(p, "PLAN-") && ends_with(p, ".md") }), example_files: filter(all, fn(p) { ends_with(p, ".ash") }), changelog_files: filter(all, fn(p) { ends_with(p, "CHANGELOG.md") }) }
}"#;
    let mut input = new_input(snippet);
    let result = parse_fn_definition.parse_next(&mut input);
    eprintln!("Result: {}", if result.is_ok() { "OK" } else { "FAIL" });
    if let Err(ref e) = result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok(), "got: {:?}", result);
}

// TODO(TASK-590): known failure — parser gap with let-then-record-constructor pattern.
#[test]
#[ignore = "TODO(TASK-590)"]
fn task590_debug_let_bindings_then_constructor() {
    use ash_parser::input::new_input;
    use ash_parser::parse_module::parse_fn_definition;
    use winnow::prelude::Parser;

    let snippet = r#"pub fn scan_tree(root: String) -> FileTree {
    let all = collect_files([], root);
    let specs = filter(all, fn(p) { starts_with(p, "SPEC-") && ends_with(p, ".md") });
    let plans = filter(all, fn(p) { starts_with(p, "PLAN-") && ends_with(p, ".md") });
    let examples = filter(all, fn(p) { ends_with(p, ".ash") });
    let changelogs = filter(all, fn(p) { ends_with(p, "CHANGELOG.md") });
    FileTree { spec_files: specs, plan_files: plans, example_files: examples, changelog_files: changelogs }
}"#;
    let mut input = new_input(snippet);
    let result = parse_fn_definition.parse_next(&mut input);
    eprintln!("Result: {}", if result.is_ok() { "OK" } else { "FAIL" });
    if let Err(ref e) = result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok(), "got: {:?}", result);
}

#[test]
fn task590_debug_let_closure() {
    use ash_parser::input::new_input;
    use ash_parser::parse_module::parse_fn_definition;
    use winnow::prelude::Parser;

    let cases = [
        (
            "simple let",
            r#"pub fn f() {
    let x = filter([], fn(p) { p });
    x
}"#,
        ),
        (
            "let with string ops",
            r#"pub fn f() {
    let x = filter([], fn(p) { starts_with(p, "a") });
    x
}"#,
        ),
        (
            "let with &&",
            r#"pub fn f() {
    let x = filter([], fn(p) { starts_with(p, "a") && ends_with(p, "b") });
    x
}"#,
        ),
        (
            "two lets",
            r#"pub fn f() {
    let x = filter([], fn(p) { starts_with(p, "a") });
    let y = filter([], fn(p) { ends_with(p, "b") });
    x
}"#,
        ),
    ];
    for (name, snippet) in cases.iter() {
        let mut input = new_input(snippet);
        let result = parse_fn_definition.parse_next(&mut input);
        eprintln!("{}: {}", name, if result.is_ok() { "OK" } else { "FAIL" });
        if let Err(ref e) = result {
            eprintln!("  Error: {}", e);
        }
    }
}

#[test]
fn task590_debug_closure_in_call_arg() {
    use ash_parser::input::new_input;
    use ash_parser::parse_expr::expr;

    let cases = [
        r#"filter([], fn(p) { p })"#,
        r#"filter([], |p| -> p)"#,
        r#"filter([], fn(p) { starts_with(p, "a") })"#,
    ];
    for (i, snippet) in cases.iter().enumerate() {
        let mut input = new_input(snippet);
        let result = expr(&mut input);
        eprintln!("Case {}: {}", i, if result.is_ok() { "OK" } else { "FAIL" });
        if let Err(ref e) = result {
            eprintln!("  Error: {}", e);
        }
    }
}

#[test]
fn task590_debug_pipe_closure_in_call_arg() {
    use ash_parser::input::new_input;
    use ash_parser::parse_expr::expr;

    let cases = [
        r#"filter([], |p| -> p)"#,
        r#"filter([], |p| -> starts_with(p, "a"))"#,
        r#"filter([], |p| -> starts_with(p, "a") && ends_with(p, "b"))"#,
    ];
    for (i, snippet) in cases.iter().enumerate() {
        let mut input = new_input(snippet);
        let result = expr(&mut input);
        eprintln!("Case {}: {}", i, if result.is_ok() { "OK" } else { "FAIL" });
    }
}
