//! Phase 200 LSP migration polish regression tests.

use ash_lint::LintConfig;
use ash_lsp_core::completion::completions;
use ash_lsp_core::db::{AshLspDatabase, SourceFile, build_symbol_index};
use ash_lsp_core::diagnostics::compute_diagnostics;
use ash_lsp_core::goto::goto_definition;
use ash_lsp_core::hover::hover_at;
use ash_lsp_core::symbols::document_symbols;
use lsp_types::{CompletionResponse, GotoDefinitionResponse, HoverContents, NumberOrString, Uri};
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

#[test]
fn remaining_definition_forms_support_lsp_navigation_and_indexing() {
    let source = concat!(
        "fn helper() -> Int { 1 }\n",
        "interface Sensor { read() -> Int }\n",
        "fn main() -> Int { helper() }\n",
    );
    let module = ash_parser::parse_surface_file(source).expect("parse remaining definitions");
    let uri: Uri = "file:///remaining-definitions.ash"
        .parse()
        .expect("valid test URI");

    let db = AshLspDatabase::new();
    let file = SourceFile::new(&db, uri.to_string(), source.to_string(), 1);
    let index = build_symbol_index(&db, file);
    let indexed_names = index
        .document_symbols
        .iter()
        .map(|symbol| symbol.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(indexed_names, ["helper", "Sensor", "main"]);

    let CompletionResponse::Array(completion_items) = completions(&module, source, 2, 0) else {
        panic!("expected array completion response");
    };
    let completion_names = completion_items
        .iter()
        .map(|item| item.label.as_str())
        .collect::<Vec<_>>();
    assert!(completion_names.contains(&"helper"));
    assert!(completion_names.contains(&"Sensor"));
    assert!(completion_names.contains(&"main"));
    assert!(!completion_names.contains(&"proxy"));

    let helper_call_column = u32::try_from(
        source
            .lines()
            .nth(2)
            .expect("main line")
            .find("helper")
            .expect("helper call"),
    )
    .expect("test column fits in u32");
    let Some(GotoDefinitionResponse::Scalar(location)) =
        goto_definition(&module, source, &uri, 2, helper_call_column)
    else {
        panic!("helper call should resolve to its function definition");
    };
    assert_eq!(location.range.start.line, 0);

    let Some(hover) = hover_at(&module, source, 0, 3) else {
        panic!("helper definition should have hover information");
    };
    let HoverContents::Markup(markup) = hover.contents else {
        panic!("expected markdown hover information");
    };
    assert!(markup.value.contains("fn helper() -> Int"));

    let document_symbols = document_symbols(&module);
    let document_symbol_names = document_symbols
        .iter()
        .map(|symbol| symbol.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(document_symbol_names, ["helper", "Sensor", "main"]);
}
