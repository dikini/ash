//! TASK-1957: productive docs teach current syntax and quarantine old forms.

use std::path::{Path, PathBuf};

const PRODUCTIVE_DOCS: &[&str] = &["docs/README.md", "docs/TUTORIAL.md", "docs/tutorials"];
const REFERENCE_DOCS: &[&str] = &["docs/reference/phase-199-app-template-manifest-schema.md"];
const SPEC_INDEX: &str = "docs/spec/README.md";
const CURRENT_SYNTAX_ANCHORS: &[&str] = &[
    "templates/apps/README.md",
    "examples/10-testing-helpers/testing_helpers.ash",
    "examples/11-process-channel-helpers/process_channel_helpers.ash",
    "ash template instantiate",
    "fn main",
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn markdown_files(root: &Path, relative: &str) -> Vec<PathBuf> {
    let path = root.join(relative);
    if path.is_file() {
        return vec![path];
    }

    let mut files = Vec::new();
    if !path.exists() {
        return files;
    }

    for entry in std::fs::read_dir(&path)
        .unwrap_or_else(|error| panic!("read docs path {}: {error}", path.display()))
    {
        let entry = entry.expect("read docs entry");
        let entry_path = entry.path();
        if entry_path.is_dir() {
            let nested = entry_path
                .strip_prefix(root)
                .expect("docs file should be under repository root")
                .to_string_lossy()
                .replace('\\', "/");
            files.extend(markdown_files(root, &nested));
        } else if entry_path
            .extension()
            .is_some_and(|extension| extension == "md")
        {
            files.push(entry_path);
        }
    }

    files
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .expect("docs file should be under repository root")
        .to_string_lossy()
        .replace('\\', "/")
}

fn contains_token_followed_by_with(line: &str, token: &str) -> bool {
    let mut rest = line;
    while let Some(index) = rest.find(token) {
        let before = rest[..index].chars().next_back();
        let after = rest[index + token.len()..].chars().next();
        let boundary_before = before.is_none_or(|ch| !is_ident_char(ch));
        let boundary_after = after.is_some_and(char::is_whitespace);
        if boundary_before && boundary_after && rest[index + token.len()..].contains(" with") {
            return true;
        }
        rest = &rest[index + token.len()..];
    }
    false
}

fn is_ident_char(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

fn stale_productive_pattern(line: &str) -> Option<&'static str> {
    let lower = line.to_ascii_lowercase();
    if contains_token_followed_by_with(line, "observe") {
        Some("observe-with")
    } else if contains_token_followed_by_with(line, "act") {
        Some("act-with")
    } else if line.contains(&["Pr", "oc<"].concat()) {
        Some("proc-carrier")
    } else if line.contains(&["A", "ct<"].concat()) {
        Some("act-carrier")
    } else if line.contains(&["Work", "flow<"].concat()) {
        Some("application-carrier")
    } else if lower.contains("removed application") {
        Some("removed-application")
    } else if lower.contains("removed syntax") || lower.contains("old syntax") {
        Some("removed-syntax")
    } else if lower.contains("ambient authority") || lower.contains("direct provider") {
        Some("authority-bypass-wording")
    } else if line.trim_start().starts_with("if ") && line.contains(" {") {
        Some("stale-if-without-then")
    } else if line.trim_start().starts_with("for ") && line.contains(" in ") && line.contains(" {")
    {
        Some("stale-for-in-loop")
    } else if line.trim_start().starts_with("decide ") && line.contains(" else") {
        Some("stale-decide-else")
    } else {
        None
    }
}

fn is_reference_labeled(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("historical")
        || lower.contains("reference")
        || lower.contains("migration")
        || lower.contains("compatibility")
        || lower.contains("superseded")
        || lower.contains("removed")
}

#[test]
fn productive_docs_do_not_teach_unlabeled_old_forms() {
    let repo = repo_root();
    let mut failures = Vec::new();

    for root in PRODUCTIVE_DOCS {
        for path in markdown_files(&repo, root) {
            let relative = relative_path(&repo, &path);
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
            for (line_index, line) in source.lines().enumerate() {
                if let Some(pattern) = stale_productive_pattern(line) {
                    failures.push(format!("{relative}:{}:{pattern}: {line}", line_index + 1));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "productive docs must teach current syntax only:\n{}",
        failures.join("\n")
    );
}

#[test]
fn tutorial_points_to_current_productive_app_path() {
    let repo = repo_root();
    let path = repo.join("docs/TUTORIAL.md");
    let tutorial = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read tutorial {}: {error}", path.display()));

    for anchor in CURRENT_SYNTAX_ANCHORS {
        assert!(
            tutorial.contains(anchor),
            "docs/TUTORIAL.md should point readers at current productive syntax anchor {anchor}"
        );
    }
}

#[test]
fn reference_and_spec_old_form_mentions_are_labeled() {
    let repo = repo_root();
    let mut failures = Vec::new();

    for root in REFERENCE_DOCS.iter().copied().chain([SPEC_INDEX]) {
        for path in markdown_files(&repo, root) {
            let relative = relative_path(&repo, &path);
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
            for (line_index, line) in source.lines().enumerate() {
                if stale_productive_pattern(line).is_some() && !is_reference_labeled(line) {
                    failures.push(format!("{}:{}: {}", relative, line_index + 1, line));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "reference/spec index old-form mentions must be labeled as historical, reference, migration, compatibility, superseded, or removed:\n{}",
        failures.join("\n")
    );
}
