//! TASK-963 coverage for stdlib/reference callable syntax migration.

use std::path::{Path, PathBuf};

use ash_parser::{
    Definition, input::new_input, parse_module::parse_fn_definition,
    parse_utils::skip_whitespace_and_comments,
};
use winnow::Parser;

fn repo_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

fn read(relative: &str) -> String {
    std::fs::read_to_string(repo_path(relative)).expect("fixture should be readable")
}

fn source_files(root: &str) -> Vec<PathBuf> {
    fn walk(path: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(path).expect("directory should be readable") {
            let path = entry.expect("entry should be readable").path();
            if path.is_dir() {
                walk(&path, out);
            } else if matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("ash" | "md")
            ) {
                out.push(path);
            }
        }
    }

    let mut out = Vec::new();
    walk(&repo_path(root), &mut out);
    out.sort();
    out
}

fn relative_display(path: &Path) -> String {
    let root = repo_path("");
    path.strip_prefix(&root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn has_removed_form_marker(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    ["removed", "migration", "historical", "reserved"]
        .iter()
        .any(|marker| lower.contains(marker))
}

fn has_removed_callable_fn_spelling(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut offset = 0;

    while let Some(found) = line[offset..].find("Fn") {
        let start = offset + found;
        let before_is_word =
            start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_');
        if before_is_word {
            offset = start + 2;
            continue;
        }

        let mut cursor = start + 2;
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor < bytes.len() && bytes[cursor] == b'(' {
            return true;
        }
        offset = start + 2;
    }

    false
}

fn has_bare_unary_callback_arrow(line: &str) -> bool {
    ["f:", "predicate:"]
        .iter()
        .filter_map(|name| line.find(name).map(|idx| idx + name.len()))
        .any(|start| {
            let ty = line[start..].trim_start();
            !ty.starts_with('(') && ty.contains(" -> ")
        })
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

fn assert_public_functions_parse(relative: &str) {
    let source = read(relative);
    let functions = extract_public_fn_sources(&source);
    assert!(
        !functions.is_empty(),
        "{relative} should contain public functions for callable migration coverage"
    );

    for function_source in functions {
        let mut input = new_input(&function_source);
        skip_whitespace_and_comments(&mut input);
        match parse_fn_definition.parse_next(&mut input) {
            Ok(Definition::Function(_)) => {}
            Ok(other) => panic!("{relative} expected function definition, got {other:?}"),
            Err(error) => {
                panic!("{relative} function should parse after callable migration: {error}")
            }
        }
    }
}

#[test]
fn stdlib_callable_signatures_parse_with_preferred_syntax() {
    for relative in [
        "std/src/list.ash",
        "std/src/option.ash",
        "std/src/result.ash",
    ] {
        assert_public_functions_parse(relative);
    }
}

#[test]
fn stdlib_contains_no_removed_fn_callback_signatures() {
    let mut violations = Vec::new();
    for path in source_files("std/src") {
        let text = std::fs::read_to_string(&path).expect("file should be readable");
        for (line_no, line) in text.lines().enumerate() {
            if has_removed_callable_fn_spelling(line) {
                violations.push(format!(
                    "{}:{}: {line}",
                    relative_display(&path),
                    line_no + 1
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "stdlib removed callable signatures remain:\n{}",
        violations.join("\n")
    );
}

#[test]
fn stdlib_callback_signatures_do_not_use_bare_unary_arrow_domains() {
    let mut violations = Vec::new();
    for path in source_files("std/src") {
        let text = std::fs::read_to_string(&path).expect("file should be readable");
        for (line_no, line) in text.lines().enumerate() {
            if has_bare_unary_callback_arrow(line) {
                violations.push(format!(
                    "{}:{}: {line}",
                    relative_display(&path),
                    line_no + 1
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "stdlib callback signatures should parenthesize unary callable domains:\n{}",
        violations.join("\n")
    );
}

#[test]
fn reference_current_examples_prefer_callable_arrow_syntax() {
    let mut violations = Vec::new();
    for path in source_files("reference") {
        let text = std::fs::read_to_string(&path).expect("file should be readable");
        for (line_no, line) in text.lines().enumerate() {
            if has_removed_callable_fn_spelling(line) && !has_removed_form_marker(line) {
                violations.push(format!(
                    "{}:{}: {line}",
                    relative_display(&path),
                    line_no + 1
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "unlabelled reference removed callable syntax remains:\n{}",
        violations.join("\n")
    );
}

#[test]
fn reference_current_examples_prefer_pure_closure_arrow() {
    let mut violations = Vec::new();
    for path in source_files("reference") {
        let text = std::fs::read_to_string(&path).expect("file should be readable");
        for (line_no, line) in text.lines().enumerate() {
            if line.contains("|") && line.contains("=>") && !has_removed_form_marker(line) {
                violations.push(format!(
                    "{}:{}: {line}",
                    relative_display(&path),
                    line_no + 1
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "unlabelled reference removed closure syntax remains:\n{}",
        violations.join("\n")
    );
}

#[test]
fn removed_callable_examples_are_labeled_historical_or_reserved() {
    let calls = read("reference/language/functions/calls-and-values.md");
    assert!(
        calls.contains("historical `Fn(<params>) -> <return>` spelling is removed syntax"),
        "removed callable spelling must be explicitly labeled historical"
    );

    let local = read("reference/language/functions/local-and-anonymous.md");
    assert!(
        local.contains("`|args| =>`") && local.contains("reserved and rejected"),
        "removed/future closure fat arrow must be labeled reserved, not taught as pure syntax"
    );
}
