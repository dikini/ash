//! TASK-1952: Phase 200 legacy/deprecated form inventory gate.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const AUDIT_PATH: &str = "docs/plan/audits/AUDIT-200-legacy-deprecated-form-inventory.md";
const SEARCH_ROOTS: &[&str] = &[
    "crates/ash-cli/tests",
    "crates/ash-lsp-core/tests",
    "docs/TUTORIAL.md",
    "docs/reference",
    "docs/spec",
    "docs/tutorials",
    "examples",
    "std",
    "templates",
    "tests/std",
    "tests/workflows",
];
const ALLOWED_CLASSES: &[&str] = &[
    "removed",
    "migrated",
    "compatibility-only",
    "historical/reference-only",
    "retained-with-migration-diagnostic",
];
const PRODUCTIVE_CLASSES: &[&str] = &["removed", "migrated", "retained-with-migration-diagnostic"];
const PATTERNS: &[(&str, &str)] = &[
    ("legacy-workflow", "legacy workflow"),
    ("old-syntax", "old syntax"),
    ("deprecated-syntax", "deprecated syntax"),
    ("observe-with", "observe "),
    ("act-with", "act "),
    ("proc-carrier", "Proc<"),
    ("act-carrier", "Act<"),
    ("workflow-carrier", "Workflow<"),
    ("direct-provider", "direct provider"),
    ("ambient-authority", "ambient authority"),
    ("formatter-legacy", "formatter"),
    ("lsp-legacy", "LSP"),
    ("example-compatibility", "compatibility"),
    ("template-workflow", "template"),
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn should_scan(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("ash" | "md" | "rs" | "json")
    )
}

fn collect_hits(repo: &Path) -> BTreeSet<String> {
    fn visit(base: &Path, path: &Path, out: &mut BTreeSet<String>) {
        if !path.exists() {
            return;
        }

        if path.is_file() {
            scan_file(base, path, out);
            return;
        }

        for entry in std::fs::read_dir(path).expect("read Phase 200 audit root") {
            let entry = entry.expect("read Phase 200 audit entry");
            let path = entry.path();
            if path.is_dir() {
                visit(base, &path, out);
            } else if should_scan(&path) {
                scan_file(base, &path, out);
            }
        }
    }

    let mut hits = BTreeSet::new();
    for root in SEARCH_ROOTS {
        visit(repo, &repo.join(root), &mut hits);
    }
    hits
}

fn scan_file(base: &Path, path: &Path, out: &mut BTreeSet<String>) {
    if !should_scan(path) {
        return;
    }

    let rel_path = path
        .strip_prefix(base)
        .expect("audit file should be under repository root")
        .to_string_lossy()
        .replace('\\', "/");
    if rel_path == AUDIT_PATH || rel_path.ends_with("phase200_legacy_deprecated_form_audit.rs") {
        return;
    }

    let source = std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    for (pattern_id, needle) in PATTERNS {
        if source
            .lines()
            .any(|line| pattern_matches(pattern_id, needle, line))
        {
            out.insert(format!("{rel_path}:{pattern_id}"));
        }
    }
}

fn pattern_matches(pattern_id: &str, needle: &str, line: &str) -> bool {
    match pattern_id {
        "observe-with" => contains_token_followed_by_with(line, "observe"),
        "act-with" => contains_token_followed_by_with(line, "act"),
        "direct-provider" => {
            line.to_ascii_lowercase().contains("capability")
                && line.to_ascii_lowercase().contains("direct provider")
        }
        "formatter-legacy" => {
            line.to_ascii_lowercase().contains("formatter")
                && line.to_ascii_lowercase().contains("legacy")
        }
        "lsp-legacy" => {
            line.to_ascii_lowercase().contains("lsp")
                && line.to_ascii_lowercase().contains("legacy")
        }
        "example-compatibility" => {
            line.to_ascii_lowercase().contains("example")
                && line.to_ascii_lowercase().contains("compatibility")
        }
        "template-workflow" => {
            line.to_ascii_lowercase().contains("template")
                && line.to_ascii_lowercase().contains("workflow")
        }
        _ => line.contains(needle),
    }
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
fn phase200_audit_classifies_legacy_and_deprecated_form_hits() {
    let repo = repo_root();
    let audit_path = repo.join(AUDIT_PATH);
    let audit = std::fs::read_to_string(&audit_path).unwrap_or_else(|error| {
        panic!(
            "missing Phase 200 legacy/deprecated form audit at {}: {error}",
            audit_path.display()
        )
    });

    let discovered = collect_hits(&repo);
    let rows = parse_audit_rows(&audit);
    let classified: BTreeSet<_> = rows.keys().cloned().collect();

    assert_eq!(
        classified, discovered,
        "Phase 200 audit must classify every legacy/deprecated form hit under {:?}",
        SEARCH_ROOTS
    );

    for (hit, row) in rows {
        assert!(
            ALLOWED_CLASSES.contains(&row.classification.as_str()),
            "{hit} has unsupported classification {:?}",
            row.classification
        );
        assert!(
            row.owner_task.starts_with("TASK-19"),
            "{hit} must carry follow-up task ownership, got {}",
            row.owner_task
        );
        assert!(
            !row.gate_or_reason.trim().is_empty() && row.gate_or_reason.trim() != "-",
            "{hit} must carry a check gate or exclusion reason"
        );
        if PRODUCTIVE_CLASSES.contains(&row.classification.as_str()) {
            assert!(
                row.gate_or_reason.contains("cargo test")
                    || row.gate_or_reason.contains("ash check")
                    || row.gate_or_reason.contains("docs gate")
                    || row.gate_or_reason.contains("audit gate"),
                "{hit} is productive/remediated but lacks a concrete gate: {}",
                row.gate_or_reason
            );
        }
    }
}
