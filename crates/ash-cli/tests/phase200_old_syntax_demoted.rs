//! TASK-1958: old syntax is removed from productive paths or explicitly demoted.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const AUDIT_PATH: &str = "docs/plan/audits/AUDIT-200-legacy-deprecated-form-inventory.md";
const PRODUCTIVE_ROOTS: &[&str] = &[
    "docs/TUTORIAL.md",
    "docs/tutorials",
    "examples/10-testing-helpers",
    "examples/11-process-channel-helpers",
    "templates/apps",
];
const ALLOWED_COMPATIBILITY_PREFIXES: &[&str] = &[
    "crates/ash-cli/tests/",
    "crates/ash-lsp-core/tests/",
    "std/src/",
    "tests/std/",
    "tests/workflows/",
];
const STALE_REASON_MARKERS: &[&str] = &[
    "requires review",
    "pending final",
    "until migrated or removed",
    "until docs refresh",
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn should_scan(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("ash" | "md" | "rs" | "json")
    )
}

fn walk_files(root: &Path, relative: &str) -> Vec<PathBuf> {
    let path = root.join(relative);
    if path.is_file() {
        return vec![path];
    }

    let mut files = Vec::new();
    if !path.exists() {
        return files;
    }

    for entry in std::fs::read_dir(&path)
        .unwrap_or_else(|error| panic!("read productive root {}: {error}", path.display()))
    {
        let entry = entry.expect("read productive entry");
        let entry_path = entry.path();
        if entry_path.is_dir() {
            let nested = entry_path
                .strip_prefix(root)
                .expect("productive file should be under repository root")
                .to_string_lossy()
                .replace('\\', "/");
            files.extend(walk_files(root, &nested));
        } else if should_scan(&entry_path) {
            files.push(entry_path);
        }
    }

    files
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .expect("productive file should be under repository root")
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

fn old_form_pattern(line: &str) -> Option<&'static str> {
    let lower = line.to_ascii_lowercase();
    if contains_token_followed_by_with(line, "observe") {
        Some("observe-with")
    } else if contains_token_followed_by_with(line, "act") {
        Some("act-with")
    } else if line.contains("Proc<") {
        Some("proc-carrier")
    } else if line.contains("Act<") {
        Some("act-carrier")
    } else if line.contains("Workflow<") {
        Some("workflow-carrier")
    } else if lower.contains("legacy workflow") {
        Some("legacy-workflow")
    } else if lower.contains("deprecated syntax") || lower.contains("old syntax") {
        Some("deprecated-syntax")
    } else if lower.contains("ambient authority") || lower.contains("direct provider") {
        Some("authority-bypass-wording")
    } else {
        None
    }
}

#[derive(Debug)]
struct AuditRow {
    classification: String,
    owner_task: String,
    gate_or_reason: String,
}

fn parse_audit_rows(markdown: &str) -> BTreeMap<String, AuditRow> {
    let mut rows = BTreeMap::new();

    for line in markdown.lines() {
        let cells: Vec<_> = line
            .trim()
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        if cells.len() < 5 || cells[0] == "Hit" || cells[0].starts_with("---") {
            continue;
        }

        let Some(hit) = cells[0]
            .strip_prefix('`')
            .and_then(|cell| cell.strip_suffix('`'))
        else {
            continue;
        };

        rows.insert(
            hit.to_owned(),
            AuditRow {
                classification: cells[2].to_owned(),
                owner_task: cells[3].to_owned(),
                gate_or_reason: cells[4].to_owned(),
            },
        );
    }

    rows
}

#[test]
fn productive_roots_fail_closed_against_old_forms() {
    let repo = repo_root();
    let mut failures = Vec::new();

    for root in PRODUCTIVE_ROOTS {
        for path in walk_files(&repo, root) {
            let relative = relative_path(&repo, &path);
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
            for (line_index, line) in source.lines().enumerate() {
                if let Some(pattern) = old_form_pattern(line) {
                    failures.push(format!("{relative}:{}:{pattern}: {line}", line_index + 1));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "productive roots must not contain old/deprecated forms:\n{}",
        failures.join("\n")
    );
}

#[test]
fn audit_has_no_unresolved_demote_or_remove_language() {
    let repo = repo_root();
    let path = repo.join(AUDIT_PATH);
    let audit = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read audit {}: {error}", path.display()));
    let rows = parse_audit_rows(&audit);

    let mut failures = Vec::new();
    for (hit, row) in rows {
        let lower_reason = row.gate_or_reason.to_ascii_lowercase();
        for marker in STALE_REASON_MARKERS {
            if lower_reason.contains(marker) {
                failures.push(format!("{hit}: stale audit reason contains {marker:?}"));
            }
        }

        if row.classification == "compatibility-only" {
            let path = hit.split(':').next().expect("hit path");
            let allowed_prefix = ALLOWED_COMPATIBILITY_PREFIXES
                .iter()
                .any(|prefix| path.starts_with(prefix));
            if !allowed_prefix {
                failures.push(format!(
                    "{hit}: compatibility-only retained outside compatibility fixture roots"
                ));
            }
            if !row.owner_task.starts_with("TASK-19") {
                failures.push(format!("{hit}: missing task ownership"));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "old syntax audit must be fully demoted or removed:\n{}",
        failures.join("\n")
    );
}
