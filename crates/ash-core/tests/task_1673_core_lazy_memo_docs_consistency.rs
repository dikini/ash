//! TASK-1673: Core lazy/memo docs consistency.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("ash-core crate lives under crates/ash-core")
        .to_path_buf()
}

fn read(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

#[test]
fn task_1673_docs_cover_syntax_typechecking_lowering_and_runtime() {
    let reference = read(repo_root().join("docs/reference/core-ash-lazy-memo-modes.md"));

    for required in [
        "lazy/memo mode features",
        "CoreValue::Thunk",
        "CoreExpr::LetMode",
        "CoreExpr::Force",
        "Value::ThunkClosure",
        "PrimOp::ForceThunk",
        "lazy",
        "memo",
        "re-entrancy",
        "ThunkConstructed",
        "MemoCacheFilled",
    ] {
        assert!(
            reference.contains(required),
            "mode-docs reference should mention {required}"
        );
    }
}

#[test]
fn task_1673_doc_tracking_is_reconciled() {
    let spec = read(repo_root().join("docs/spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md"));
    let plan = read(repo_root().join("docs/plan/PLAN-163-CORE-LAZY-MEMO-MODES.md"));
    let task_1673 =
        read(repo_root().join("docs/plan/tasks/TASK-1673-core-lazy-memo-reference-closeout.md"));

    assert!(
        spec.contains("Core Ash representation, typing"),
        "SPEC-101 should still document Core lazy/memo scope"
    );
    assert!(
        spec.contains("thunk") || spec.contains("Thunk"),
        "SPEC-101 should document thunk carriers"
    );

    assert!(
        plan.contains("Reference docs, PLAN-INDEX, task files, and CHANGELOG are reconciled."),
        "PLAN-163 should include the closeout reconciliation requirement"
    );

    assert!(
        task_1673.contains("**Status:** Done"),
        "TASK-1673 should be marked done"
    );
    assert!(
        task_1673.contains("Reference docs explain implemented behavior and non-goals."),
        "TASK-1673 requirements should include documented behavior"
    );

    for required in [
        "task_1671_core_mode_end_to_end",
        "task_1672_core_mode_tracing_docs_consistency",
        "task_1669_core_mode_lowering",
    ] {
        assert!(
            task_1673.contains(required),
            "TASK-1673 should still reference follow-on verification points"
        );
    }
}

#[test]
fn task_1673_reference_docs_are_anchored_in_text_typecheck_and_lowering_pages() {
    let text = read(repo_root().join("docs/reference/core-ash-text-format.md"));
    let typechecking = read(repo_root().join("docs/reference/core-ash-type-checking.md"));
    let lowering = read(repo_root().join("docs/reference/core-ash-lowering.md"));

    for required in [
        "thunk",
        "let-mode",
        "force",
        "CoreValue::Thunk",
        "CoreType::Mode",
        "(let-mode",
        "ForceThunk",
    ] {
        assert!(
            text.contains(required)
                || typechecking.contains(required)
                || lowering.contains(required),
            "mode docs should be anchored in existing reference pages for {required}"
        );
    }
}
