//! TASK-777 regression tests for public workflow summary import/export.

use ash_core::workflow_carrier::SourceOrigin;
use ash_engine::module_loader::load_ordinary_file;

#[test]
fn load_ordinary_file_exports_public_workflow_summary_for_imported_workflow() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("flows.ash");
    let caller = dir.path().join("caller.ash");

    std::fs::write(
        &module,
        r"pub workflow guarded() -> Workflow<Int> {
    done
}
",
    )
    .expect("write module");
    std::fs::write(
        &caller,
        r"use flows::{guarded}
workflow main { ret 0 }
",
    )
    .expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("caller imports public workflow");
    let callable = loaded
        .imported_callables
        .get("guarded")
        .expect("guarded callable imported");
    let summary = callable
        .workflow_summary
        .as_ref()
        .expect("pub workflow export carries public workflow summary");

    assert!(
        summary.node_count > 0,
        "summary should expose workflow node shape"
    );
    assert!(
        summary.projection_events.iter().any(|event| {
            matches!(
                event.origin,
                SourceOrigin::ImportedSummary {
                    ref module,
                    ref public_anchor,
                } if module == "flows" && public_anchor == "guarded"
            )
        }),
        "summary events should use public imported-summary origins"
    );
}
