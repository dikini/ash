//! TASK-1948: canonical app template corpus conformance.

use assert_cmd::Command;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn ash() -> Command {
    Command::cargo_bin("ash").expect("ash binary exists")
}

#[test]
fn canonical_templates_instantiate_and_check() {
    let root = repo_root();
    let template_root = root.join("templates/apps");
    let expected = [
        "cli-tool",
        "file-pipeline",
        "http-fetch-process",
        "provider-profile-test",
        "supervised-worker",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();

    let discovered = discover_template_ids(&template_root);
    assert_eq!(discovered, expected);

    for id in expected {
        let dir = tempfile::tempdir().expect("tempdir");
        let out = dir.path().join(&id);
        let manifest = template_root.join(&id).join("template.json");

        ash()
            .args(["template", "instantiate"])
            .arg("--manifest")
            .arg(&manifest)
            .arg("--out")
            .arg(&out)
            .arg("--param")
            .arg(format!("app_name={id}"))
            .assert()
            .success();
    }
}

fn discover_template_ids(root: &Path) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for entry in fs::read_dir(root).expect("read templates/apps") {
        let entry = entry.expect("read template dir");
        if entry.path().join("template.json").exists() {
            ids.insert(
                entry
                    .file_name()
                    .into_string()
                    .expect("template id should be utf8"),
            );
        }
    }
    ids
}
