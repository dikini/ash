use std::path::{Path, PathBuf};

use ash_core::core_ash_lower::lower_core_program;
use ash_core::core_ash_text::parse_core_file;
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};
use ash_core::sexp::{string_to_term, term_to_string};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("ash-core crate lives under crates/ash-core")
        .to_path_buf()
}

fn fixture_dir() -> PathBuf {
    repo_root().join("crates/ash-core/tests/fixtures/core")
}

fn fixture_path(name: &str) -> PathBuf {
    fixture_dir().join(name)
}

fn lower_fixture(name: &str) -> String {
    let core = parse_core_file(fixture_path(name)).expect("Core fixture should parse");
    let valid =
        validate_core_program(RawCoreProgram::new(core)).expect("Core fixture should validate");
    let cps = lower_core_program(valid).expect("Core fixture should lower");
    let text = term_to_string(&cps).expect("lowered CPS should serialize");
    let reparsed = string_to_term(&text).expect("serialized CPS golden should parse");
    assert_eq!(reparsed, cps, "CPS golden serialization should round trip");
    format!("{text}\n")
}

#[test]
fn core_fixtures_lower_to_stable_cps_golden_files() {
    for fixture in [
        "let_val_jump.core",
        "let_prim_if.core",
        "call_non_tail.core",
        "let_call.core",
        "raise_handle.core",
        "contract_trap.core",
    ] {
        let actual = lower_fixture(fixture);
        let golden_path = fixture_path(&format!("{fixture}.cps.golden"));
        let expected = std::fs::read_to_string(&golden_path).unwrap_or_else(|error| {
            panic!(
                "failed to read golden {}: {error}\nactual:\n{actual}",
                golden_path.display()
            )
        });

        assert_eq!(actual, expected, "golden mismatch for {fixture}");
    }
}

#[test]
fn invalid_core_fixture_fails_validation_before_lowering() {
    let core =
        parse_core_file(fixture_path("invalid_duplicate_row.core")).expect("fixture should parse");
    let error = validate_core_program(RawCoreProgram::new(core)).unwrap_err();

    assert!(
        error.to_string().contains("duplicate row item"),
        "unexpected validation error: {error}"
    );
}
