//! TASK-1943: Phase 199 current-syntax inventory gate.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const AUDIT_PATH: &str = "docs/plan/audits/AUDIT-199-current-syntax-library-template-inventory.md";
const ACTIVE_AUDIT_PATH: &str = "docs/plan/audits/AUDIT-201-deprecated-functionality-removal.md";
const AUDIT_ROOTS: &[&str] = &["std/src", "examples", "tests/std", "tests/applications"];
const ALLOWED_CLASSES: &[&str] = &[
    "current executable",
    "current reference",
    "historical/reference-only",
    "removed from productive path",
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn collect_ash_files(repo: &Path) -> BTreeSet<String> {
    fn visit(base: &Path, dir: &Path, out: &mut BTreeSet<String>) {
        if !dir.exists() {
            return;
        }

        for entry in std::fs::read_dir(dir).expect("read audit root") {
            let entry = entry.expect("read audit entry");
            let path = entry.path();
            if path.is_dir() {
                visit(base, &path, out);
            } else if path.extension().is_some_and(|ext| ext == "ash") {
                out.insert(
                    path.strip_prefix(base)
                        .expect("audit file should be under repository root")
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }

    let mut files = BTreeSet::new();
    for root in AUDIT_ROOTS {
        visit(repo, &repo.join(root), &mut files);
    }
    files
}

fn collect_ash_files_under(repo: &Path, root: &str) -> impl Iterator<Item = String> {
    fn visit(base: &Path, dir: &Path, out: &mut Vec<String>) {
        if !dir.exists() {
            return;
        }

        for entry in std::fs::read_dir(dir).expect("read audit root") {
            let entry = entry.expect("read audit entry");
            let path = entry.path();
            if path.is_dir() {
                visit(base, &path, out);
            } else if path.extension().is_some_and(|ext| ext == "ash") {
                out.push(
                    path.strip_prefix(base)
                        .expect("audit file should be under repository root")
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }

    let mut files = Vec::new();
    visit(repo, &repo.join(root), &mut files);
    files.into_iter()
}

#[derive(Debug)]
struct AuditRow {
    classification: String,
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
        if cells.len() < 3 || cells[0] == "Path" || cells[0].starts_with("---") {
            continue;
        }

        let Some(path) = cells[0]
            .strip_prefix('`')
            .and_then(|cell| cell.strip_suffix('`'))
        else {
            continue;
        };

        rows.insert(
            path.to_owned(),
            AuditRow {
                classification: cells[1].to_owned(),
                gate_or_reason: cells[2].to_owned(),
            },
        );
    }

    rows
}

#[test]
fn phase199_audit_classifies_all_productive_candidate_ash_files() {
    let repo = repo_root();
    let audit_path = repo.join(AUDIT_PATH);
    let audit = std::fs::read_to_string(&audit_path).unwrap_or_else(|error| {
        panic!(
            "missing Phase 199 audit at {}: {error}",
            audit_path.display()
        )
    });

    if audit.contains("**Status:** Superseded by Phase 201") {
        assert!(
            audit.contains("AUDIT-201-deprecated-functionality-removal.md"),
            "superseded Phase 199 audit must link to the active Phase 201 audit"
        );
        for root in AUDIT_ROOTS {
            let root_has_ash_files = collect_ash_files_under(&repo, root).next().is_some();
            if root_has_ash_files {
                assert!(
                    audit.contains(root),
                    "superseded Phase 199 audit must preserve current authority root {root}"
                );
            }
        }

        let active_audit_path = repo.join(ACTIVE_AUDIT_PATH);
        let active_audit = std::fs::read_to_string(&active_audit_path).unwrap_or_else(|error| {
            panic!(
                "missing active Phase 201 audit at {}: {error}",
                active_audit_path.display()
            )
        });
        assert!(
            active_audit.contains("**Status:** Active"),
            "Phase 201 audit must be marked active"
        );
        assert!(
            active_audit.contains("phase201_deprecated_functionality_removal_gate"),
            "Phase 201 audit must identify the active deprecated-form gate"
        );
        return;
    }

    let discovered = collect_ash_files(&repo);
    let rows = parse_audit_rows(&audit);
    let classified: BTreeSet<_> = rows.keys().cloned().collect();

    assert_eq!(
        classified, discovered,
        "Phase 199 audit must classify every .ash file under {:?}",
        AUDIT_ROOTS
    );

    for (path, row) in rows {
        assert!(
            ALLOWED_CLASSES.contains(&row.classification.as_str()),
            "{path} has unsupported classification {:?}",
            row.classification
        );
        assert!(
            !row.gate_or_reason.trim().is_empty() && row.gate_or_reason.trim() != "-",
            "{path} must carry a check gate or exclusion reason"
        );
        if row.classification == "current executable" || row.classification == "current reference" {
            assert!(
                row.gate_or_reason.contains("ash check")
                    || row.gate_or_reason.contains("cargo test")
                    || row.gate_or_reason.contains("artifact assertion"),
                "{path} is productive/current but lacks an executable or artifact gate: {}",
                row.gate_or_reason
            );
        }
    }
}
