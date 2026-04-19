//! TASK-595 / TASK-613 limitation regression test
//!
//! Documents the current honest boundary: `use regex::{find}` fails at module
//! load time because `regex.ash` uses `act execute` inside `fn` bodies, which
//! the parser cannot handle at expression position. This test codifies that
//! known limitation so that any future fix that makes Ash-language regex
//! imports work will be detected (the test will fail and need updating).

use std::io::Write;

/// Verifies that importing `regex::{find}` from Ash source currently fails.
///
/// This is NOT a feature test — it documents the current honest limitation.
/// When the parser gains support for `act execute` in `fn` body expression
/// position, this test should be replaced with a positive end-to-end test.
#[test]
fn regex_import_fails_at_module_boundary() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let ash_file = tmp_dir.path().join("test_regex_import.ash");
    let mut f = std::fs::File::create(&ash_file).expect("create temp ash file");
    writeln!(
        f,
        "use regex::{{find}}
workflow test_regex() -> String {{
    done;
}}"
    )
    .expect("write temp ash file");

    let result = ash_engine::module_loader::load_ordinary_file(&ash_file);
    assert!(
        result.is_err(),
        "Expected regex import to fail, but it succeeded. \
         If this test fails, the Ash-language regex surface is now working — \
         replace this limitation test with a positive end-to-end test."
    );

    let err_msg = result.unwrap_err().to_string();
    // The specific current failure is "item 'find' not found in module 'regex'"
    // because regex.ash's pub fn bodies cannot be parsed. Accept either this
    // exact pattern or the module-not-found fallback (if the file stops being
    // discovered). Do NOT accept generic parse errors — we want the specific
    // module-loader boundary failure.
    let item_not_found = err_msg.contains("item 'find' not found in module 'regex'");
    let module_not_found = err_msg.contains("module 'regex' not found");

    assert!(
        item_not_found || module_not_found,
        "Unexpected error message: {err_msg}. \
         Expected the specific module-loader boundary failure for regex import."
    );
}
