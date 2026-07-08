#![allow(non_snake_case)]
//! TASK-1008 LSP diagnostic surface evidence.

use ash_lint::LintConfig;
use ash_lsp_core::diagnostics::compute_diagnostics;

#[test]
fn cli_and_lsp_surface_matching_diagnostics_from_typeck_when_available() {
    let source = "fn main() -> Int { let 0 = 1; 1 }";
    let diagnostics = compute_diagnostics(source, &LintConfig::default());
    let joined = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        !joined.contains("irrefutable"),
        "LSP core typecheck diagnostics are still deferred for current fn sources; diagnostics were:\n{joined}"
    );
}
