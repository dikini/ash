//! TASK-2000 inventory gate for residual `Act`/`Proc` machinery.
//!
//! The decision task must not silently lose a reachable tower wrapper during a
//! rename or a broad source edit.  This gate deliberately inventories every
//! Rust source/test file that currently mentions either legacy carrier name.
//! Each record classifies its role; entries marked `public-*` are the decision
//! surface, while `runtime-internal`, `test-consumer`, and
//! `negative-regression` entries explain why their references are retained.

use std::{collections::BTreeSet, fs, path::Path};

use serde::Deserialize;

const INVENTORY: &str = include_str!("fixtures/task_2000_tower_machinery_inventory.json");

#[derive(Debug, Deserialize)]
struct Inventory {
    version: u32,
    records: Vec<Record>,
}

#[derive(Debug, Deserialize)]
struct Record {
    path: String,
    classification: String,
    anchors: Vec<String>,
}

#[test]
fn residual_act_proc_references_are_completely_classified() {
    let inventory: Inventory =
        serde_json::from_str(INVENTORY).expect("inventory must be valid JSON");
    assert_eq!(
        inventory.version, 1,
        "bump the gate deliberately when its schema changes"
    );

    let allowed = BTreeSet::from([
        "public-constructor",
        "public-builtin",
        "public-diagnostic",
        "lowering",
        "runtime-internal",
        "test-consumer",
        "negative-regression",
    ]);
    let mut listed = BTreeSet::new();
    for record in &inventory.records {
        assert!(
            allowed.contains(record.classification.as_str()),
            "{} has an unknown classification {:?}",
            record.path,
            record.classification
        );
        assert!(
            listed.insert(record.path.clone()),
            "duplicate inventory record: {}",
            record.path
        );

        let source = fs::read_to_string(workspace_path(&record.path))
            .unwrap_or_else(|error| panic!("inventory path {} must exist: {error}", record.path));
        assert!(
            record.anchors.iter().all(|anchor| source.contains(anchor)),
            "{} no longer contains every recorded reachability anchor {:?}",
            record.path,
            record.anchors
        );
    }

    let discovered = tower_reference_files(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crates root"),
    );
    assert_eq!(
        listed, discovered,
        "every Rust source/test file mentioning Act or Proc must be classified before TASK-2000 can decide its disposition"
    );

    assert!(
        inventory.records.iter().any(|record| record.path
            == "crates/ash-engine/tests/task_2000_tower_public_surface_rejection.rs"),
        "the public-admission deletion regression must remain an explicit decision subject"
    );
}

fn workspace_path(relative: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .join(relative)
}

fn tower_reference_files(crates_root: &Path) -> BTreeSet<String> {
    // Kept as a generated-at-test-time source inventory.  The explicit static
    // mapping makes a changed reference fail loudly rather than being silently
    // folded into a new category by heuristic classification.
    let workspace = crates_root.parent().expect("workspace root");
    let mut files = BTreeSet::new();
    for entry in walkdir::WalkDir::new(crates_root) {
        let entry = entry.expect("workspace paths must be readable");
        if !entry.file_type().is_file() || entry.path().extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(workspace)
            .expect("workspace-relative source");
        let relative = relative.to_string_lossy();
        if relative == "crates/ash-cli/tests/task_2000_tower_machinery_inventory.rs" {
            continue;
        }
        let source = fs::read_to_string(entry.path()).expect("Rust source must be readable");
        if contains_tower_token(&source) {
            files.insert(relative.into_owned());
        }
    }
    files
}

#[test]
fn qualified_lowercase_tower_module_paths_are_discovered() {
    assert!(contains_tower_token("let _ = proc::admit;"));
    assert!(contains_tower_token("pub use act::Act;"));
    assert!(!contains_tower_token("let predicate = contract::check;"));
}

fn contains_tower_token(source: &str) -> bool {
    contains_qualified_module_path(source, "act::")
        || contains_qualified_module_path(source, "proc::")
        || source
            .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
            .any(|token| matches!(token, "Act" | "Proc"))
}

fn contains_qualified_module_path(source: &str, qualified_path: &str) -> bool {
    source.match_indices(qualified_path).any(|(offset, _)| {
        source[..offset]
            .chars()
            .next_back()
            .is_none_or(|character| !character.is_ascii_alphanumeric() && character != '_')
    })
}
