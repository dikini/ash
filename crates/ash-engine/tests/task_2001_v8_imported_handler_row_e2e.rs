//! TASK-2001 RED: a real public import carries V8 structural row facts into
//! typed-handler normalization.
//!
//! This deliberately uses the ordinary file loader.  It must not be satisfied
//! by hand-built JSON or a synthetic `TypeEnv` summary.

use ash_core::semantic_summary::{StructuralEffectRowItemSummary, SummaryVersion};
use ash_engine::module_loader::load_ordinary_file;
use ash_parser::surface::{ComputationRow, Definition, Program, ProgramEntry};
use ash_typeck::normalize_handler_row_with_imported_summaries_for_test;

const CLOCK_PREFIX: &str = r"
interface Clock<T> { sleep(Int) -> Null }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = null }
";

fn write_file(path: &std::path::Path, source: &str) {
    std::fs::write(path, source).expect("write Ash fixture");
}

fn parse_program(source: &str) -> Program {
    let module = ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("TASK-2001 caller source should parse: {errors:?}"));
    let entry = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == "main" => {
                Some(ProgramEntry {
                    function: function.name.clone(),
                    span: function.span,
                })
            }
            _ => None,
        })
        .expect("fixture must define main");
    Program {
        definitions: module.definitions,
        entry,
    }
}

fn row_named<'a>(program: &'a Program, name: &str) -> &'a ComputationRow {
    program
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::EffectAlias(alias) if alias.name.as_ref() == name => Some(&alias.row),
            Definition::EffectGroup(group) if group.name.as_ref() == name => Some(&group.row),
            _ => None,
        })
        .unwrap_or_else(|| panic!("fixture must define effect row {name}"))
}

#[test]
fn public_file_import_emits_v8_structural_row_and_normalizes_it_for_a_typed_handler() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        &format!(
            "{CLOCK_PREFIX}\
             pub effect alias ClockAudit = {{ TestClock::sleep, evidence response.proved, | rest }};\n"
        ),
    );
    let caller_source = format!(
        "use provider::{{ClockAudit}}\n\
         {CLOCK_PREFIX}\
         effect alias Boundary = {{ ClockAudit }};\n\
         fn main() -> Null {{ null }}\n"
    );
    write_file(&caller, &caller_source);

    let loaded =
        load_ordinary_file(&caller).expect("ordinary public import must load a semantic summary");
    let (version, row) = loaded
        .imported_semantic_summaries
        .iter()
        .find_map(|summary| {
            summary
                .exported_effect_rows
                .iter()
                .find(|row| row.exported_name == "ClockAudit")
                .map(|row| (summary.version, row))
        })
        .expect("selected public effect row must be transported from the provider file");

    assert_eq!(
        version,
        SummaryVersion::STRUCTURAL_EFFECT_ROW_PROVIDER_BINDINGS_V8,
        "a source-loaded provider row for typed handlers must use V8, not text-only V7"
    );
    assert!(matches!(
        row.row_items[0].structural(),
        Some(StructuralEffectRowItemSummary::Operation {
            impl_type,
            interface,
            operation,
        }) if impl_type == "TestClock" && interface == "Clock" && operation == "sleep"
    ));
    assert!(matches!(
        row.row_items[1].structural(),
        Some(StructuralEffectRowItemSummary::Evidence { path })
            if path.as_slice() == ["response".to_string(), "proved".to_string()]
    ));
    assert!(matches!(
        row.row_items[2].structural(),
        Some(StructuralEffectRowItemSummary::Tail { variable }) if variable == "rest"
    ));

    // `parse_surface_file` owns the declaration grammar, while the ordinary
    // loader above owns file-level imports.  Keep the real loader-produced
    // summary and feed the parser seam the same import-prefix-stripped source
    // representation used by file-checking fixtures.
    let parser_source = caller_source
        .strip_prefix("use provider::{ClockAudit}\n")
        .expect("fixture must begin with the loader-resolved import prefix");
    let program = parse_program(parser_source);
    let normalized = normalize_handler_row_with_imported_summaries_for_test(
        &program,
        row_named(&program, "Boundary"),
        &loaded.imported_semantic_summaries,
    )
    .expect("V8 structural imported content must normalize without reparsing row text");

    assert_eq!(
        normalized
            .items
            .iter()
            .map(ash_typeck::NormalizedHandlerRowItem::canonical_key)
            .collect::<Vec<_>>(),
        [
            "operation:TestClock::Clock::sleep".to_string(),
            "evidence:response.proved".to_string(),
        ]
    );
    assert_eq!(normalized.tail.as_deref(), Some("rest"));
    assert!(normalized.items.iter().all(|item| !item.grants_authority()));
}
