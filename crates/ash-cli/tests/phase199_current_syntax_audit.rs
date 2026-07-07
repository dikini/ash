//! TASK-1943: Phase 199 current-syntax inventory gate.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const AUDIT_PATH: &str = "docs/plan/audits/AUDIT-199-current-syntax-library-template-inventory.md";
const AUDIT_ROOTS: &[&str] = &["std/src", "examples", "tests/std", "tests/workflows"];
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
