use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("ash-cli lives under crates/")
        .to_path_buf()
}

fn read_workspace_file(relative: &str) -> String {
    let path = workspace_root().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "missing TASK-930 compatibility evidence at {}: {error}",
            path.display()
        )
    })
}

#[test]
fn ooda_examples_are_library_or_template_calls_not_primitive_ir() {
    for artifact in [
        "crates/ash-core/src/amir.rs",
        "crates/ash-core/src/runtime_kernel.rs",
    ] {
        let text = read_workspace_file(artifact);
        let lower = text.to_ascii_lowercase();
        assert!(
            !lower.contains("ooda"),
            "{artifact} must not expose OODA as a privileged alpha artifact root"
        );
    }

    let ooda = read_workspace_file("std/src/ooda.ash");
    assert!(
        ooda.contains("library/template compatibility"),
        "std OODA surface must identify itself as compatibility, not primitive IR"
    );
    assert!(
        !ooda.contains("builtin fn"),
        "OODA compatibility helpers must not introduce primitive builtins"
    );
    for helper in ["observe", "orient", "decide", "act"] {
        assert!(
            ooda.contains(&format!("pub fn {helper}")),
            "missing ordinary ooda::{helper} helper"
        );
    }

    let lib = read_workspace_file("std/src/lib.ash");
    assert!(
        lib.contains("pub mod ooda;"),
        "stdlib root must expose OODA as an ordinary module"
    );
    assert!(
        lib.contains("pub use ooda::{"),
        "stdlib root must re-export OODA compatibility helpers"
    );

    let example_notes = read_workspace_file("examples/01-basics/README.md");
    assert!(
        example_notes.contains("ooda::observe")
            && example_notes.contains("ooda::orient")
            && example_notes.contains("ooda::decide")
            && example_notes.contains("ooda::act"),
        "historical OODA example notes should point to ordinary library/template calls"
    );
}

#[test]
fn ooda_lint_points_to_visible_tower_algebra() {
    let lint_readme = read_workspace_file("crates/ash-lint/README.md");
    assert!(
        lint_readme.contains("visible tower algebra"),
        "OODA lint documentation must point users toward visible tower algebra"
    );
    assert!(
        lint_readme.contains("library/template compatibility"),
        "OODA lint documentation must describe OODA as compatibility guidance"
    );

    let spec = read_workspace_file("docs/spec/SPEC-041-ASH-LINT-LIBRARY.md");
    assert!(
        spec.contains("visible tower algebra"),
        "SPEC-041 OODA lint contract must cite the visible tower algebra replacement"
    );
    assert!(
        spec.contains("not primitive alpha execution semantics"),
        "SPEC-041 must not treat OODA lint aliases as primitive alpha semantics"
    );
    assert!(
        spec.contains("ooda-missing-decide") && spec.contains("ooda-missing-orient"),
        "legacy OODA lint aliases must remain documented for compatibility"
    );
}
