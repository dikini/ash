//! TASK-1949: tutorial docs tied to executable gates.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn productive_apps_tutorial_links_to_current_gated_examples() {
    let path = repo_root().join("docs/tutorials/phase199-productive-apps.md");
    let tutorial = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("read Phase 199 tutorial {}: {err}", path.display()));

    for required in [
        "templates/apps/README.md",
        "examples/10-testing-helpers/testing_helpers.ash",
        "examples/11-process-channel-helpers/process_channel_helpers.ash",
        "ash template instantiate",
        "phase199_canonical_templates",
        "phase199_testing_helpers",
        "phase199_process_channel_helpers",
    ] {
        assert!(
            tutorial.contains(required),
            "Phase 199 tutorial should reference {required}"
        );
    }

    for stale in [
        "observe ",
        " with ",
        "act ",
        "Proc<",
        "Workflow<",
        "ambient authority",
    ] {
        assert!(
            !tutorial.contains(stale),
            "productive tutorial should not contain stale or ambient-authority wording: {stale}"
        );
    }
}
