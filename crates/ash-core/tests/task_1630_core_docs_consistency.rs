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

fn fixture_path(name: &str) -> PathBuf {
    repo_root()
        .join("crates/ash-core/tests/fixtures/core")
        .join(name)
}

#[test]
fn core_text_reference_names_committed_phase_161_fixtures_and_goldens() {
    let reference = read(repo_root().join("docs/reference/core-ash-text-format.md"));

    for fixture in [
        "let_val_jump.core",
        "let_prim_if.core",
        "call_non_tail.core",
        "let_call.core",
        "raise_handle.core",
        "contract_trap.core",
    ] {
        assert!(
            fixture_path(fixture).is_file(),
            "{fixture} should exist as a committed Core fixture"
        );
        assert!(
            fixture_path(&format!("{fixture}.cps.golden")).is_file(),
            "{fixture} should have a committed CPS golden"
        );
        assert!(
            reference.contains(fixture),
            "Core text reference should name fixture {fixture}"
        );
        assert!(
            reference.contains(&format!("{fixture}.cps.golden")),
            "Core text reference should name CPS golden for {fixture}"
        );
    }

    assert!(reference.contains("invalid_duplicate_row.core"));
    assert!(reference.contains("not surface Ash"));
    assert!(reference.contains("fixture/debug format"));
}

#[test]
fn core_lowering_reference_documents_boundaries_without_overclaiming() {
    let reference = read(repo_root().join("docs/reference/core-ash-lowering.md"));

    for required in [
        "Core-to-CPS lowering",
        "validated Core",
        "surface-to-Core lowering is out of scope",
        "typeclass solving is out of scope",
        "user-defined algebraic effects are out of scope",
        "single-clause `handle` with affine or legal multi-shot-pure resume metadata",
        "HandlerClause.resume_row = Known(row)",
        "Core `(let-cont-call name cont-ref atom body)` lowers to CPS",
        "Core Match is out of scope",
        "full type checker is out of scope",
        "ContractViolation is trap metadata",
        "Handle.row is local residual row",
    ] {
        assert!(
            reference.contains(required),
            "Core lowering reference should include boundary phrase: {required}"
        );
    }

    for fixture in [
        "call_non_tail.core",
        "let_call.core",
        "raise_handle.core",
        "contract_trap.core",
    ] {
        assert!(
            reference.contains(fixture),
            "Core lowering reference should tie example text to {fixture}"
        );
    }
}
