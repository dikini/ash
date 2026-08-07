//! TASK-2013 RED contract for structural handler-computation row normalization.
//!
//! These tests deliberately exercise the source row grammar rather than a
//! stringly summary projection.  The normalization seam is test-only: it must
//! resolve concrete operations through the declaration environment, expand
//! aliases/groups structurally, preserve every non-operation item and source
//! anchor, and fail closed at an imported privacy boundary.

use ash_core::ast::{Span, Visibility};
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    EffectRowClosureMetadata, EffectRowExportClassification, EffectRowExportId,
    EffectRowExportSummary, EffectRowItemSummary, ModuleIdentity, ModuleSemanticSummary,
    ModuleSourceOrigin, SourceAnchor, SourceOrigin, SummaryVersion,
};
use ash_parser::surface::{ComputationRow, Definition, Program, ProgramEntry};
use ash_typeck::{
    NormalizedHandlerRowItem, normalize_handler_row_for_test,
    normalize_handler_row_with_imported_summaries_for_test,
};
use proptest::prelude::*;

const CLOCK_PREFIX: &str = r#"
interface Clock<T> { sleep(Int) -> Null }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = null }
"#;

fn parse_program(source: &str) -> Program {
    let module = ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("TASK-2013 source should parse: {errors:?}"));
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
        .expect("fixture must define fn main");
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
        .unwrap_or_else(|| panic!("fixture must define row {name}"))
}

fn all_row_forms_program() -> Program {
    parse_program(&format!(
        "{CLOCK_PREFIX}\
         effect alias Base = {{\n\
           TestClock::sleep,\n\
           resource read filesystem,\n\
           fail NetworkError,\n\
           evidence response.proved,\n\
           | rest\n\
         }};\n\
         effect alias Transport = {{ channel write audit_events, process spawn }};\n\
         effect group AllEffects = {{ Base, group Transport }};\n\
         fn main() -> Null {{ null }}"
    ))
}

fn item_keys(items: &[NormalizedHandlerRowItem]) -> Vec<String> {
    items
        .iter()
        .map(NormalizedHandlerRowItem::canonical_key)
        .collect()
}

#[test]
fn task_2013_normalizes_every_parseable_row_family_without_authority() {
    let program = all_row_forms_program();
    let normalized = normalize_handler_row_for_test(&program, row_named(&program, "AllEffects"))
        .expect("all source row families must normalize structurally");

    assert_eq!(
        item_keys(&normalized.items),
        vec![
            "operation:TestClock::Clock::sleep".to_string(),
            "resource:read:filesystem".to_string(),
            "channel:write:audit_events".to_string(),
            "process:spawn".to_string(),
            "fail:NetworkError".to_string(),
            "evidence:response.proved".to_string(),
        ],
        "aliases and groups must expand structurally while every non-operation item remains"
    );
    assert_eq!(normalized.tail.as_deref(), Some("rest"));
    assert!(
        normalized.items.iter().all(|item| !item.grants_authority()),
        "a normalized requirement row must not synthesize provider/capability authority"
    );

    let operation = normalized
        .items
        .iter()
        .find(|item| item.canonical_key() == "operation:TestClock::Clock::sleep")
        .expect("declared concrete operation must be retained by canonical identity");
    assert_eq!(operation.source_provenance().len(), 1);
    assert!(
        operation.source_provenance()[0]
            .expansion_path()
            .ends_with("Base"),
        "the alias expansion path must remain available for diagnostics"
    );
    assert!(
        normalized
            .tail_provenance()
            .expect("open tail must retain a source anchor")
            .expansion_path()
            .ends_with("Base")
    );
}

#[test]
fn task_2013_alias_and_group_expansion_is_idempotent_and_keeps_each_source_anchor() {
    let program = parse_program(&format!(
        "{CLOCK_PREFIX}\
         effect alias Sleep = {{ TestClock::sleep }};\n\
         effect group Repeated = {{ Sleep, group Sleep }};\n\
         fn main() -> Null {{ null }}"
    ));

    let once = normalize_handler_row_for_test(&program, row_named(&program, "Repeated"))
        .expect("repeated compatible aliases/groups should normalize");
    let twice = normalize_handler_row_for_test(&program, row_named(&program, "Repeated"))
        .expect("normalization must be deterministic and immutable");

    assert_eq!(once, twice);
    assert_eq!(
        item_keys(&once.items),
        ["operation:TestClock::Clock::sleep"]
    );
    assert_eq!(once.items[0].source_provenance().len(), 2);
}

#[test]
fn task_2013_alias_and_group_cycles_fail_closed_before_any_normalized_fact_publishes() {
    let program = parse_program(&format!(
        "{CLOCK_PREFIX}\
         effect alias A = {{ group B }};\n\
         effect group B = {{ A }};\n\
         fn main() -> Null {{ null }}"
    ));

    let error = normalize_handler_row_for_test(&program, row_named(&program, "A"))
        .expect_err("a row expansion cycle must not become an empty or partial row");
    assert_eq!(
        error.to_string(),
        "cyclic handler-computation row expansion: A -> B -> A"
    );
}

#[test]
fn task_2013_unknown_concrete_operation_fails_closed_at_its_row_anchor() {
    let program = parse_program(&format!(
        "{CLOCK_PREFIX}\
         effect alias Broken = {{ TestClock::wake }};\n\
         fn main() -> Null {{ null }}"
    ));

    let error = normalize_handler_row_for_test(&program, row_named(&program, "Broken"))
        .expect_err("an unknown concrete operation must never normalize as an opaque string");
    assert!(
        error
            .to_string()
            .contains("concrete impl 'TestClock' has no operation 'wake'"),
        "unexpected unknown-operation diagnostic: {error}"
    );
}

fn module_identity() -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(2013)),
        ModuleId(2013),
        vec!["task2013".into(), "row_boundary".into()],
        ModuleSourceOrigin::Synthetic {
            reason: "TASK-2013 imported row fixture".into(),
        },
    )
}

fn opaque_imported_row(secret: &str) -> ModuleSemanticSummary {
    let module = module_identity();
    let mut row = EffectRowExportSummary::new(
        EffectRowExportId::new(module.clone(), "External"),
        "External",
        Visibility::Public,
        EffectRowExportClassification::TransparentAlias,
        vec![EffectRowItemSummary::new(secret)],
        SourceAnchor::new(
            SourceOrigin::Synthetic {
                reason: secret.into(),
            },
            Some(Span { start: 0, end: 1 }),
            secret,
        ),
    );
    row.closure_metadata = Some(EffectRowClosureMetadata {
        sanitizer_schema_version: 1,
        public_closure_digest: "sha256:opaque".into(),
    });
    row.mark_opaque_inaccessible_dependency();
    ModuleSemanticSummary::new(module)
        .with_version(SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7)
        .with_exported_effect_row(row)
}

fn transparent_v7_imported_row() -> ModuleSemanticSummary {
    let module = module_identity();
    let mut row = EffectRowExportSummary::new(
        EffectRowExportId::new(module.clone(), "External"),
        "External",
        Visibility::Public,
        EffectRowExportClassification::TransparentAlias,
        vec![EffectRowItemSummary::new("TestClock::sleep")],
        SourceAnchor::new(
            SourceOrigin::Synthetic {
                reason: "TASK-2001 legacy V7 fixture".into(),
            },
            Some(Span { start: 0, end: 1 }),
            "legacy V7 External row",
        ),
    );
    row.closure_metadata = Some(EffectRowClosureMetadata {
        sanitizer_schema_version: 1,
        public_closure_digest: "sha256:legacy-v7-external".into(),
    });
    ModuleSemanticSummary::new(module)
        .with_version(SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7)
        .with_exported_effect_row(row)
}

#[test]
fn task_2001_legacy_v7_row_is_ineligible_for_typed_handler_normalization() {
    let program = parse_program(&format!(
        "{CLOCK_PREFIX}\
         effect alias Boundary = {{ External }};\n\
         fn main() -> Null {{ null }}"
    ));

    let error = normalize_handler_row_with_imported_summaries_for_test(
        &program,
        row_named(&program, "Boundary"),
        &[transparent_v7_imported_row()],
    )
    .expect_err("legacy text-only V7 rows must not normalize into typed handler facts");

    assert_eq!(
        error.to_string(),
        "malformed imported-effect-row-summary: legacy V7 provider/binding row is ineligible for typed-handler normalization; require V8 structural content"
    );
    assert!(
        !error.to_string().contains("TestClock::sleep"),
        "the V7 rejection must not treat row text as a semantic identity"
    );
}

#[test]
fn task_2013_malformed_or_private_imported_rows_fail_closed_without_private_identifier_leakage() {
    let secret = "TASK2013_PRIVATE_ROW_SHOULD_NOT_LEAK";
    let program = parse_program(&format!(
        "{CLOCK_PREFIX}\
         effect alias Boundary = {{ External }};\n\
         fn main() -> Null {{ null }}"
    ));

    let error = normalize_handler_row_with_imported_summaries_for_test(
        &program,
        row_named(&program, "Boundary"),
        &[opaque_imported_row(secret)],
    )
    .expect_err("opaque imported summaries must not contribute a handler row fact");

    assert_eq!(
        error.to_string(),
        "malformed imported-effect-row-summary: provider-binding effect-row closure is inaccessible at public boundary"
    );
    assert!(!error.to_string().contains(secret));
}

#[test]
fn task_2013_distinct_open_tails_reject_without_losing_the_first_tail() {
    let program = parse_program(&format!(
        "{CLOCK_PREFIX}\
         effect alias Left = {{ TestClock::sleep | left }};\n\
         effect alias Right = {{ resource filesystem | right }};\n\
         effect group Conflict = {{ Left, group Right }};\n\
         fn main() -> Null {{ null }}"
    ));

    let error = normalize_handler_row_for_test(&program, row_named(&program, "Conflict"))
        .expect_err("distinct tails cannot be silently merged or dropped");
    assert_eq!(
        error.to_string(),
        "conflicting handler-computation row tails: left vs right"
    );
}

#[test]
fn task_2013_compatible_open_tails_from_direct_alias_routes_keep_every_anchor() {
    // The surface parser permits only one tail per literal row.  Two aliases
    // therefore provide the direct compatible-tail route available in the
    // grammar: both literal anchors reach one caller row without a group hop.
    let program = parse_program(&format!(
        "{CLOCK_PREFIX}\
         effect alias Left = {{ TestClock::sleep | rest }};\n\
         effect alias Right = {{ resource filesystem | rest }};\n\
         effect alias Combined = {{ Left, Right }};\n\
         fn main() -> Null {{ null }}"
    ));

    let once = normalize_handler_row_for_test(&program, row_named(&program, "Combined"))
        .expect("equal tails must deduplicate semantically");
    let twice = normalize_handler_row_for_test(&program, row_named(&program, "Combined"))
        .expect("normalization must preserve compatible tail provenance deterministically");

    assert_eq!(once.tail.as_deref(), Some("rest"));
    assert_eq!(once.tail_provenances(), twice.tail_provenances());
    assert_eq!(
        once.tail_provenances()
            .iter()
            .map(|provenance| provenance.expansion_path())
            .collect::<Vec<_>>(),
        ["Combined -> Left", "Combined -> Right"],
        "each compatible tail occurrence must retain its caller-visible route"
    );
    let spans = once
        .tail_provenances()
        .iter()
        .map(|provenance| provenance.source_span())
        .collect::<Vec<_>>();
    assert_eq!(spans.len(), 2);
    assert!(
        spans[0].start < spans[1].start,
        "the two direct tail anchors must remain source-distinct and ordered"
    );
}

#[test]
fn task_2013_compatible_open_tails_through_alias_and_group_keep_every_anchor() {
    let program = parse_program(&format!(
        "{CLOCK_PREFIX}\
         effect alias AliasLeaf = {{ TestClock::sleep | rest }};\n\
         effect group GroupLeaf = {{ resource filesystem | rest }};\n\
         effect alias AliasRoute = {{ AliasLeaf }};\n\
         effect group GroupRoute = {{ group GroupLeaf }};\n\
         effect group Combined = {{ AliasRoute, group GroupRoute }};\n\
         fn main() -> Null {{ null }}"
    ));

    let normalized = normalize_handler_row_for_test(&program, row_named(&program, "Combined"))
        .expect("compatible alias/group tails must normalize to one semantic tail");

    assert_eq!(normalized.tail.as_deref(), Some("rest"));
    assert_eq!(
        normalized
            .tail_provenances()
            .iter()
            .map(|provenance| provenance.expansion_path())
            .collect::<Vec<_>>(),
        [
            "Combined -> AliasRoute -> AliasLeaf",
            "Combined -> GroupRoute -> GroupLeaf",
        ],
        "nested alias/group expansion must retain every compatible-tail route in traversal order"
    );
    let spans = normalized
        .tail_provenances()
        .iter()
        .map(|provenance| provenance.source_span())
        .collect::<Vec<_>>();
    assert_eq!(spans.len(), 2);
    assert!(
        spans[0].start < spans[1].start,
        "the nested routes must retain their distinct leaf tail anchors"
    );
}

proptest! {
    #[test]
    fn task_2013_normalization_is_deterministic_for_permuted_distinct_supported_items(
        order in prop::collection::vec(0usize..3, 3)
            .prop_filter("generated order must be a permutation", |order| {
                let mut distinct = order.clone();
                distinct.sort_unstable();
                distinct == [0, 1, 2]
            })
    ) {
        let spellings = [
            "TestClock::sleep",
            "resource filesystem",
            "evidence response.proved",
        ];
        let row = order
            .iter()
            .map(|index| spellings[*index])
            .collect::<Vec<_>>()
            .join(", ");
        let program = parse_program(&format!(
            "{CLOCK_PREFIX} effect alias Permuted = {{ {row} }}; fn main() -> Null {{ null }}"
        ));

        let normalized = normalize_handler_row_for_test(&program, row_named(&program, "Permuted"))
            .expect("every generated permutation contains only supported distinct items");
        prop_assert_eq!(
            item_keys(&normalized.items),
            vec![
                "operation:TestClock::Clock::sleep".to_string(),
                "resource:filesystem".to_string(),
                "evidence:response.proved".to_string(),
            ]
        );
        prop_assert!(normalized.tail.is_none());
        prop_assert!(normalized.items.iter().all(|item| !item.grants_authority()));
    }
}
