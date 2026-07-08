//! Phase 200 LSP migration polish regression tests.

use ash_lint::LintConfig;
use ash_lsp_core::diagnostics::compute_diagnostics;
use ash_lsp_core::symbols::document_symbols;
use lsp_types::NumberOrString;
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn strip_import_lines(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("use "))
        .collect::<Vec<_>>()
        .join("\n")
}

const fn diagnostic_code(diagnostic: &lsp_types::Diagnostic) -> Option<&str> {
    match diagnostic.code.as_ref() {
        Some(NumberOrString::String(code)) => Some(code.as_str()),
        _ => None,
    }
}

#[test]
fn reserved_callable_arrow_surfaces_lsp_migration_diagnostic() {
    let diagnostics = compute_diagnostics(
        "fn f(x: [Int => Bool]) -> Bool { true }\nfn main() -> Bool { true }\n",
        &LintConfig::default(),
    );

    assert_eq!(diagnostics.len(), 1, "diagnostics={diagnostics:?}");
    let diagnostic = &diagnostics[0];
    assert_eq!(
        diagnostic_code(diagnostic),
        Some("DeprecatedSyntaxMigration")
    );
    assert!(
        diagnostic
            .message
            .contains("removed callable arrow syntax is not accepted"),
        "{diagnostic:?}"
    );
    assert_eq!(diagnostic.range.start.line, 0);
    assert_eq!(diagnostic.range.start.character, 13);
}

#[test]
fn phase199_current_examples_keep_lsp_document_symbols() {
    for relative in [
        "examples/10-testing-helpers/testing_helpers.ash",
        "examples/11-process-channel-helpers/process_channel_helpers.ash",
    ] {
        let path = repo_root().join(relative);
        let source = fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!("read {}: {error}", path.display());
        });
        let parser_only_source = strip_import_lines(&source);
        let module = ash_parser::parse_surface_file(&parser_only_source)
            .unwrap_or_else(|errors| panic!("parse {relative}: {errors:?}"));
        let symbols = document_symbols(&module);

        assert!(
            symbols.iter().any(|symbol| symbol.name == "main"),
            "{relative} should expose target main function symbol; symbols={symbols:?}"
        );
    }
}
