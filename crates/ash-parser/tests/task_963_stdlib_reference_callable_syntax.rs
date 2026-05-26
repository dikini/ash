//! TASK-963 coverage for stdlib/reference callable syntax migration.

use std::path::{Path, PathBuf};

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

fn has_allowed_marker(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    [
        "compatibility",
        "legacy",
        "migration",
        "historical",
        "reserved",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn has_legacy_callable_fn_spelling(line: &str) -> bool {
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

#[test]
fn stdlib_callable_signatures_parse_with_preferred_syntax() {
    for relative in [
        "std/src/act.ash",
        "std/src/list.ash",
        "std/src/option.ash",
        "std/src/proc.ash",
        "std/src/result.ash",
        "std/src/workflow.ash",
    ] {
        let source = read(relative);
        ash_parser::parse_surface_file(&source).unwrap_or_else(|err| {
            panic!("{relative} should parse after callable migration: {err:?}")
        });
    }
}

#[test]
fn stdlib_contains_no_legacy_fn_callback_signatures() {
    let mut violations = Vec::new();
    for path in source_files("std/src") {
        let text = std::fs::read_to_string(&path).expect("file should be readable");
        for (line_no, line) in text.lines().enumerate() {
            if has_legacy_callable_fn_spelling(line) {
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
        "stdlib legacy callable signatures remain:\n{}",
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
            if has_legacy_callable_fn_spelling(line) && !has_allowed_marker(line) {
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
        "unlabelled reference legacy callable syntax remains:\n{}",
        violations.join("\n")
    );
}

#[test]
fn reference_current_examples_prefer_pure_closure_arrow() {
    let mut violations = Vec::new();
    for path in source_files("reference") {
        let text = std::fs::read_to_string(&path).expect("file should be readable");
        for (line_no, line) in text.lines().enumerate() {
            if line.contains("|") && line.contains("=>") && !has_allowed_marker(line) {
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
        "unlabelled reference legacy closure syntax remains:\n{}",
        violations.join("\n")
    );
}

#[test]
fn legacy_callable_examples_are_labeled_compatibility() {
    let calls = read("reference/language/functions/calls-and-values.md");
    assert!(
        calls.contains("legacy `Fn(<params>) -> <return>` spelling is compatibility syntax"),
        "legacy callable spelling must be explicitly labeled compatibility"
    );

    let local = read("reference/language/functions/local-and-anonymous.md");
    assert!(
        local.contains("`|args| =>`") && local.contains("reserved and rejected"),
        "legacy/future closure fat arrow must be labeled reserved, not taught as pure syntax"
    );
}
