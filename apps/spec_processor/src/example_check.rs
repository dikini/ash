//! Example syntax conformance checker.
//!
//! Validates every `.ash` file found in the repository by parsing and type-checking
//! it through the Ash engine. Files that fail either step produce an `ExampleFailure`
//! finding at tier Error.

use std::path::Path;

use crate::finding::SpecFinding;

/// Parse and type-check every `.ash` file in `example_files`.
///
/// Each path is interpreted as relative to `repo_root`.  Files that cannot be
/// parsed or that contain type errors produce a single `SpecFinding` with
/// category `"ExampleFailure"` and tier `Error`.
///
/// # Errors
///
/// Returns `Err` if the Ash engine fails to build (should only happen under
/// extreme memory/resource pressure).
pub fn check_examples(
    example_files: &[String],
    repo_root: &Path,
) -> Result<Vec<SpecFinding>, ash_engine::EngineError> {
    let engine = ash_engine::Engine::new().build()?;
    let mut findings = Vec::new();

    for relative in example_files {
        let full_path = repo_root.join(relative);
        let check_result = engine
            .parse_file(&full_path)
            .and_then(|mut w| engine.check(&mut w));

        if let Err(err) = check_result {
            findings.push(
                SpecFinding::error("ExampleFailure", format!("{err}")).with_file(relative.as_str()),
            );
        }
    }

    Ok(findings)
}
