//! TASK-2002 red fixtures for the canonical ambient-`do` lowering boundary.
//!
//! The first fixture records the currently supported end-to-end path: source
//! origin reaches the parsed module, evidence remains a callable requirement,
//! and ambient `do` lowers as ordinary Core sequencing.  The second fixture is
//! deliberately red until all removed named `do` targets reject through one
//! canonical, user-facing diagnostic rather than parser or legacy-lowering
//! accidents.

use std::path::Path;

use ash_core::Expr as CoreExpr;
use ash_engine::{CallableRowRequirementSource, Engine, EngineError};
use ash_parser::{
    parse_surface_file_with_path,
    surface::{ComputationRowItem, Definition},
};

const REMOVED_DO_TARGET_DIAGNOSTIC: &str =
    "generic do target annotations are removed; use ambient `do { ... }` with row requirements";

fn engine() -> Engine {
    Engine::new().build().expect("engine builds")
}

#[test]
fn task_2002_ambient_do_preserves_source_and_evidence_requirement_through_lowering() {
    let source = r"
fn main() -> Int where row { evidence audit_log } {
    do {
        let value = 41;
        return value + 1
    }
}
";
    let path = Path::new("fixtures/task-2002-ambient-do.ash");

    let parsed = parse_surface_file_with_path(source, Some(path))
        .expect("ambient do source should parse with its file origin");
    assert_eq!(
        parsed.path.as_deref(),
        Some("fixtures/task-2002-ambient-do.ash")
    );
    let Definition::Function(main) = &parsed.definitions[0] else {
        panic!("expected main function");
    };
    assert!(
        main.span.end > main.span.start,
        "main must retain its source span"
    );

    let engine = engine();
    let mut entry = engine
        .parse_file_source(path, source)
        .expect("ambient do should reach the lowering boundary");
    engine
        .check(&mut entry)
        .expect("ambient do row should typecheck as a requirement, not authority");

    let row = entry
        .callable_row_requirements
        .get("main")
        .expect("evidence row sidecar must survive lowering");
    assert_eq!(row.source, CallableRowRequirementSource::WhereRow);
    assert!(matches!(
        row.row.items.as_slice(),
        [ComputationRowItem::Evidence { path, .. }]
            if path.iter().map(AsRef::as_ref).collect::<Vec<_>>() == ["audit_log"]
    ));
    assert!(
        matches!(entry.core, CoreExpr::Let { .. }),
        "ambient do must lower to Core sequencing"
    );
    assert_eq!(
        entry.lowering_sidecars.entry_body_origin.origin,
        ash_core::semantic_summary::SourceOrigin::File(
            "fixtures/task-2002-ambient-do.ash".to_string()
        ),
        "the lowered Core entry must retain its file origin sidecar"
    );
    assert_eq!(
        entry.lowering_sidecars.entry_body_origin.span,
        Some(ash_core::Span {
            start: main.span.start,
            end: main.span.end,
        }),
        "the lowered Core entry must retain the source span of its enclosing callable"
    );
}

#[test]
fn task_2002_named_do_targets_reject_with_one_canonical_diagnostic() {
    let engine = engine();

    for target in ["K", "Act", "Proc", "Workflow"] {
        let source = format!("fn main() {{ do:{target} {{ return 1 }} }}");
        let error = engine
            .parse_file_source(Path::new("fixtures/task-2002-named-do.ash"), &source)
            .expect_err("named do targets are removed from target Ash");
        let EngineError::Parse(diagnostic) = error else {
            panic!("named do target must fail at the source-entry parse boundary");
        };
        assert_eq!(
            diagnostic, REMOVED_DO_TARGET_DIAGNOSTIC,
            "{target} must reject before typechecking or legacy generic-do lowering"
        );
    }
}

#[test]
fn task_2002_local_macro_expands_before_ambient_do_reaches_core_lowering() {
    let source = r"
macro answer() => 42;

fn main() {
    do { return answer!() }
}

";

    let entry = engine()
        .parse_file_source(Path::new("fixtures/task-2002-local-macro.ash"), source)
        .expect("source entry must expand a local macro before ambient-do lowering");

    assert!(
        matches!(entry.core, CoreExpr::Literal(ash_core::Value::Int(42))),
        "the Core entry must contain the expanded value, never a residual macro carrier"
    );
    assert!(
        entry
            .lowering_sidecars
            .expansion_origins
            .iter()
            .any(|origin| matches!(
                origin.origin,
                ash_parser::surface::SurfaceOrigin::MacroExpansion { ref expansion_id, .. }
                    if expansion_id.as_ref() == "answer"
            )),
        "the entry lowering sidecars must retain the successful macro expansion origin"
    );
}

#[test]
fn task_2002_local_notation_expands_before_ambient_do_reaches_core_lowering() {
    let source = r"
infixl 6 <+> = combine;

fn combine(x: Int, y: Int) -> Int {
    x + y
}

fn main() {
    do { return (41 <+>) }
}
";

    let entry = engine()
        .parse_file_source(Path::new("fixtures/task-2002-local-notation.ash"), source)
        .expect("source entry must expand local notation before ambient-do lowering");

    assert!(
        !format!("{:#?}", entry.core).contains("<+>"),
        "the Core entry must contain the elaborated callable, never a residual notation carrier"
    );
    assert!(
        entry
            .lowering_sidecars
            .expansion_origins
            .iter()
            .any(|origin| matches!(
                origin.origin,
                ash_parser::surface::SurfaceOrigin::NotationExpansion { ref target, .. }
                    if target.as_ref() == "combine"
            )),
        "the entry lowering sidecars must retain the successful notation expansion origin"
    );
}

fn assert_retained_discharge_status(
    discharge: &ash_core::core_ash_contract::ContractDischargeRecord,
) {
    assert!(
        matches!(
            discharge.status(),
            ash_core::core_ash_contract::ContractDischargeStatus::Dynamic { .. }
                | ash_core::core_ash_contract::ContractDischargeStatus::Deferred { .. }
        ),
        "sidecars retain the stable, public discharge-status projection"
    );
}

#[test]
fn task_2002_all_local_callable_contracts_survive_entry_lowering_as_non_authorizing_sidecars() {
    let source = r"
fn contractless_helper() -> Int {
    0
}

fn helper(value: Int) -> Int requires: value >= 0 ensures: result >= 0 {
    value
}

fn inline_row_helper(path: Path) -> { evidence audit_log } Int ensures: result >= 0 {
    0
}

fn main() -> Int ensures: result >= 0 {
    helper(41)
}
";

    parse_surface_file_with_path(
        source,
        Some(Path::new("fixtures/task-2002-all-callable-contracts.ash")),
    )
    .expect("inline-row callable contract control should parse");

    let entry = engine()
        .parse_file_source(
            Path::new("fixtures/task-2002-all-callable-contracts.ash"),
            source,
        )
        .expect("accepted local fn contracts should lower before the entry is published");

    let contracts = &entry.lowering_sidecars.callable_contracts;
    assert_eq!(
        contracts.keys().map(String::as_str).collect::<Vec<_>>(),
        ["contractless_helper", "helper", "inline_row_helper", "main"],
        "contract sidecars must be keyed deterministically by every local callable name"
    );

    let contractless = contracts
        .get("contractless_helper")
        .expect("every local fn, including one without a contract, needs a sidecar");
    assert!(contractless.contract.requires.is_empty());
    assert!(contractless.contract.ensures.is_empty());
    assert!(contractless.requires_discharges.is_empty());
    assert!(contractless.ensures_discharges.is_empty());
    assert!(contractless.runtime_postconditions.predicates.is_empty());

    let helper = contracts
        .get("helper")
        .expect("helper contract must survive entry lowering");
    assert_eq!(helper.contract.requires.len(), 1);
    assert_eq!(helper.contract.ensures.len(), 1);
    assert_eq!(helper.requires_discharges.len(), 1);
    assert_eq!(helper.ensures_discharges.len(), 1);
    assert_eq!(helper.runtime_postconditions.predicates.len(), 1);
    assert_retained_discharge_status(&helper.requires_discharges[0]);
    assert_retained_discharge_status(&helper.ensures_discharges[0]);

    let inline_row = contracts
        .get("inline_row_helper")
        .expect("inline-row helper contract must survive entry lowering");
    assert_eq!(inline_row.contract.ensures.len(), 1);
    assert_eq!(inline_row.ensures_discharges.len(), 1);
    assert_eq!(inline_row.runtime_postconditions.predicates.len(), 1);
    assert_retained_discharge_status(&inline_row.ensures_discharges[0]);
    assert_eq!(
        inline_row.result_binder_type.as_ref(),
        Some(&ash_core::core_ash::CoreType::Base("Int".to_string())),
        "inline-row callable contracts must use the callable result type, never the raw function type"
    );

    let main = contracts
        .get("main")
        .expect("main contract must survive entry lowering");
    assert!(main.contract.requires.is_empty());
    assert_eq!(main.contract.ensures.len(), 1);
    assert!(main.requires_discharges.is_empty());
    assert_eq!(main.ensures_discharges.len(), 1);
    assert_eq!(main.runtime_postconditions.predicates.len(), 1);
    assert_retained_discharge_status(&main.ensures_discharges[0]);

    assert_eq!(
        entry
            .callable_row_requirements
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["inline_row_helper"],
        "only the explicitly declared inline row, never a retained contract, may produce a row requirement"
    );
    assert_eq!(
        entry
            .callable_row_requirements
            .get("inline_row_helper")
            .expect("the explicit inline row must retain its existing row summary")
            .source,
        CallableRowRequirementSource::InlineReturn
    );
    assert!(
        entry.declared_concrete_operation.is_none(),
        "retained fn contracts must not select an operation or imply runtime behavior"
    );
}

#[test]
fn task_2002_invalid_helper_contract_rejects_before_entry_sidecars_can_be_published() {
    let source = r"
fn invalid_helper(value: Int) -> Int ensures: value >= 0 {
    value
}

fn main() -> Int {
    invalid_helper(41)
}
";
    let path = Path::new("fixtures/task-2002-invalid-helper-contract.ash");

    parse_surface_file_with_path(source, Some(path))
        .expect("the invalid lowering control must use parser-accepted fn-contract syntax");

    let error = engine()
        .parse_file_source(path, source)
        .expect_err("an invalid local helper contract must reject before an Entry is returned");
    assert!(
        matches!(error, EngineError::Parse(_)),
        "contract lowering failures must use the source-entry parse/lowering boundary: {error}"
    );
}
