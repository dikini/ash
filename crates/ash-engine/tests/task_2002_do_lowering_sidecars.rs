//! TASK-2002 red fixtures for the canonical ambient-`do` lowering boundary.
//!
//! The first fixture records the currently supported end-to-end path: source
//! origin reaches the parsed module, evidence remains a callable requirement,
//! and ambient `do` lowers as ordinary Core sequencing.  The second fixture is
//! deliberately red until all removed named `do` targets reject through one
//! canonical, user-facing diagnostic rather than parser or legacy-lowering
//! accidents.

use std::path::Path;

use ash_core::{
    Expr as CoreExpr, core_ash::CoreSourceSpan, core_ash_contract::PredicateEnvironment,
};
use ash_engine::{CallableRowRequirementSource, Engine, EngineError};
use ash_parser::{
    lower::lower_fn_contract_for_function,
    parse_surface_file, parse_surface_file_with_path,
    surface::{ComputationRowItem, Definition, FnDef, Requirement, Spanned},
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

fn function_named<'a>(module: &'a ash_parser::surface::ModuleFile, name: &str) -> &'a FnDef {
    module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == name => Some(function),
            _ => None,
        })
        .unwrap_or_else(|| panic!("expected local function `{name}`"))
}

fn contract_clause_spans(function: &FnDef) -> Vec<ash_parser::Span> {
    let Some(contract) = &function.contract else {
        return Vec::new();
    };

    contract
        .requires
        .iter()
        .map(|requirement| match requirement {
            Requirement::Arithmetic { expr } => expr.span(),
            Requirement::HasCapability { .. } => {
                panic!("the source-accurate contract-span fixture only uses arithmetic clauses")
            }
        })
        .chain(contract.ensures.iter().map(|clause| clause.span))
        .collect()
}

fn assert_discharge_spans(
    discharges: impl IntoIterator<Item = ash_core::core_ash_contract::ContractDischargeRecord>,
    expected_clause_spans: &[ash_parser::Span],
    expected_file: Option<&str>,
) {
    let discharges = discharges.into_iter().collect::<Vec<_>>();
    assert_eq!(
        discharges.len(),
        expected_clause_spans.len(),
        "every source contract clause must produce exactly one retained discharge"
    );

    for (discharge, expected_span) in discharges.iter().zip(expected_clause_spans) {
        assert_eq!(
            discharge.source_span(),
            &CoreSourceSpan {
                file: expected_file.map(str::to_owned),
                start: expected_span.start,
                end: expected_span.end,
            },
            "retained contract discharge must point to its originating source clause"
        );
    }
}

fn predicate_binder_span(env: &PredicateEnvironment, binder_name: &str) -> CoreSourceSpan {
    env.binders()
        .iter()
        .find(|binder| binder.id().local() == binder_name)
        .unwrap_or_else(|| panic!("predicate environment must retain `{binder_name}`"))
        .source_span()
        .clone()
}

fn declaration_name_span(source: &str, name: &str) -> ash_parser::Span {
    let start = source
        .find(&format!("{name}:"))
        .unwrap_or_else(|| panic!("fixture must declare parameter `{name}`"));
    ash_parser::Span {
        start,
        end: start + name.len(),
        line: 0,
        column: 0,
    }
}

fn assert_predicate_binder_spans(
    lowered: &ash_parser::lower::LoweredFnContract,
    function: &FnDef,
    source: &str,
    expected_file: Option<&str>,
) {
    let expected_parameter_spans = function
        .params
        .iter()
        .map(|param| {
            (
                param.name.as_ref(),
                declaration_name_span(source, param.name.as_ref()),
            )
        })
        .collect::<Vec<_>>();

    for environment in [
        &lowered.requires_predicate_environment,
        &lowered.ensures_predicate_environment,
    ] {
        for &(name, span) in &expected_parameter_spans {
            assert_eq!(
                predicate_binder_span(environment, name),
                CoreSourceSpan {
                    file: expected_file.map(str::to_owned),
                    start: span.start,
                    end: span.end,
                },
                "parameter predicate binders must use their exact declaration-name span"
            );
        }
    }

    assert_eq!(
        predicate_binder_span(&lowered.ensures_predicate_environment, "result"),
        CoreSourceSpan {
            file: expected_file.map(str::to_owned),
            start: function.span.start,
            end: function.span.end,
        },
        "the synthetic result binder must use the stable enclosing callable anchor"
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
fn task_2002_file_backed_all_local_contract_discharges_keep_clause_spans_and_module_path() {
    let source = r"
fn helper(value: Int) -> Int requires: value >= 0 ensures: result >= 0 {
    value
}

fn main() -> Int ensures: result >= 0 {
    helper(41)
}
";
    let path = Path::new("fixtures/task-2002-contract-source-spans.ash");
    let parsed = parse_surface_file_with_path(source, Some(path))
        .expect("the file-backed all-local contract fixture should parse");
    let entry = engine()
        .parse_file_source(path, source)
        .expect("valid all-local contracts should lower before entry publication");

    for name in ["helper", "main"] {
        let function = function_named(&parsed, name);
        let lowered = entry
            .lowering_sidecars
            .callable_contracts
            .get(name)
            .unwrap_or_else(|| panic!("{name} must retain its all-local contract sidecar"));
        assert_discharge_spans(
            lowered.discharges(),
            &contract_clause_spans(function),
            Some("fixtures/task-2002-contract-source-spans.ash"),
        );
    }
}

#[test]
fn task_2002_file_backed_contract_discharges_keep_original_offsets_after_leading_imports() {
    let source = "use dependency::{first};\nuse dependency::{second};\n\nfn helper(value: Int) -> Int requires: value >= 0 ensures: result >= 0 {\n    value\n}\n\nfn main() -> Int ensures: result >= 0 {\n    helper(41)\n}\n";
    let directory = tempfile::tempdir().expect("temporary import fixture directory exists");
    let dependency_path = directory.path().join("dependency.ash");
    std::fs::write(
        &dependency_path,
        "pub fn first() -> Int { 1 }\npub fn second() -> Int { 2 }\n",
    )
    .expect("import fixture dependency is written");
    let entry_path = directory.path().join("entry.ash");

    // The ordinary-file loader consumes imports before lowering local callables.
    // Blank only those prelude lines for the expectation parse so every source
    // offset and the originating file path remain identical to the input file.
    let source_with_imports_masked = source
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("use ") || line.trim_start().starts_with("pub use ") {
                " ".repeat(line.len())
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let parsed = parse_surface_file_with_path(&source_with_imports_masked, Some(&entry_path))
        .expect("the offset-preserving local source should parse");
    let entry = engine()
        .parse_file_source(&entry_path, source)
        .expect("file-backed source with leading imports should lower");
    let expected_file = entry_path.to_string_lossy();

    for name in ["helper", "main"] {
        let function = function_named(&parsed, name);
        let lowered = entry
            .lowering_sidecars
            .callable_contracts
            .get(name)
            .unwrap_or_else(|| panic!("{name} must retain its all-local contract sidecar"));
        assert_discharge_spans(
            lowered.discharges(),
            &contract_clause_spans(function),
            Some(expected_file.as_ref()),
        );
    }
}

#[test]
fn task_2002_runtime_entry_imports_preserve_in_memory_contract_clause_coordinates() {
    let source = "use time::{sleep};\n\nfn helper(value: Int) -> Int requires: value >= 0 ensures: result >= 0 {\n    value\n}\n\nfn main() -> Int ensures: result >= 0 {\n    helper(41)\n}\n";
    let source_with_import_masked = source
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("use ") || line.trim_start().starts_with("pub use ") {
                " ".repeat(line.len())
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let parsed = parse_surface_file(&source_with_import_masked)
        .expect("the newline-masked direct entry source should retain local contracts");
    let engine = engine();
    engine
        .load_runtime_stdlib()
        .expect("the accepted time runtime import must be registered");
    let entry = engine
        .parse_entry_source(source)
        .expect("the registered time import and local contracts should lower as an entry");

    for name in ["helper", "main"] {
        let function = function_named(&parsed, name);
        let lowered = entry
            .lowering_sidecars
            .callable_contracts
            .get(name)
            .unwrap_or_else(|| panic!("{name} must retain its direct-entry contract sidecar"));
        assert_discharge_spans(lowered.discharges(), &contract_clause_spans(function), None);
    }
}

#[test]
fn task_2002_runtime_entry_file_imports_keep_contract_offsets_and_file_path() {
    let source = "use time::{sleep};\n\nfn helper(value: Int) -> Int requires: value >= 0 ensures: result >= 0 {\n    value\n}\n\nfn main() -> Int ensures: result >= 0 {\n    helper(41)\n}";
    let source_with_import_masked = source
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("use ") || line.trim_start().starts_with("pub use ") {
                " ".repeat(line.len())
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let directory = tempfile::tempdir().expect("temporary runtime entry directory exists");
    let entry_path = directory.path().join("runtime-entry.ash");
    std::fs::write(&entry_path, source).expect("runtime entry fixture is written");
    let canonical_path = entry_path
        .canonicalize()
        .expect("written runtime entry has a canonical path")
        .to_string_lossy()
        .into_owned();
    let parsed = parse_surface_file_with_path(&source_with_import_masked, Some(&entry_path))
        .expect("the coordinate-preserving runtime entry source should parse");
    let engine = engine();
    engine
        .load_runtime_stdlib()
        .expect("the accepted time runtime import must be registered");

    let entry = engine
        .parse_entry_file(&entry_path)
        .expect("file-backed runtime entry imports and local contracts should lower");

    assert_eq!(
        entry.lowering_sidecars.entry_body_origin.origin,
        ash_core::semantic_summary::SourceOrigin::File(canonical_path.clone()),
        "the runtime entry file path must remain the enclosing sidecar origin"
    );
    for name in ["helper", "main"] {
        let function = function_named(&parsed, name);
        let lowered = entry
            .lowering_sidecars
            .callable_contracts
            .get(name)
            .unwrap_or_else(|| {
                panic!("{name} must retain its file-backed runtime-entry contract sidecar")
            });
        assert_discharge_spans(
            lowered.discharges(),
            &contract_clause_spans(function),
            Some(&canonical_path),
        );
    }
}

#[test]
fn task_2002_runtime_entry_imports_reject_before_contract_sidecars_can_be_published() {
    let source_with_contract = "fn main() -> Int ensures: result >= 0 { 0 }";
    let source = format!("use PosixFs::{{read}};\n\n{source_with_contract}");
    let error = engine()
        .parse_entry_source(&source)
        .expect_err("unsupported runtime imports must reject before Entry publication");
    assert!(
        matches!(error, EngineError::Parse(_)),
        "runtime import rejection must use the source-entry parse boundary: {error}"
    );
}

#[test]
fn task_2002_in_memory_contract_lowering_keeps_exact_clause_offsets_without_file() {
    let source = "fn helper(value: Int) -> Int requires: value >= 0 ensures: result >= 0 { value }";
    let parsed =
        parse_surface_file(source).expect("the in-memory all-local contract fixture should parse");
    let helper = function_named(&parsed, "helper");
    let lowered = lower_fn_contract_for_function(helper)
        .expect("valid in-memory helper contract should lower");

    assert_discharge_spans(lowered.discharges(), &contract_clause_spans(helper), None);
}

#[test]
fn task_2002_direct_contract_predicate_binders_keep_exact_name_spans_without_file() {
    let source = "fn helper(alpha: Int, beta: Int) -> Int requires: alpha >= 0 ensures: result >= 0 { alpha }";
    let parsed = parse_surface_file(source)
        .expect("the direct predicate-binder provenance fixture should parse");
    let helper = function_named(&parsed, "helper");
    let lowered = lower_fn_contract_for_function(helper)
        .expect("the direct predicate-binder provenance fixture should lower");

    assert_predicate_binder_spans(&lowered, helper, source, None);
}

#[test]
fn task_2002_file_contract_predicate_binders_keep_exact_name_spans_and_canonical_path() {
    let source = "fn helper(alpha: Int, beta: Int) -> Int requires: alpha >= 0 ensures: result >= 0 { alpha }\n\nfn main() -> Int { helper(0, 0) }";
    let directory = tempfile::tempdir().expect("temporary provenance fixture directory exists");
    let entry_path = directory.path().join("predicate-binder-spans.ash");
    std::fs::write(&entry_path, source).expect("file-backed provenance fixture is written");
    let canonical_path = entry_path
        .canonicalize()
        .expect("written provenance fixture has a canonical path")
        .to_string_lossy()
        .into_owned();
    let parsed = parse_surface_file_with_path(source, Some(&entry_path))
        .expect("the file-backed predicate-binder provenance fixture should parse");
    let helper = function_named(&parsed, "helper");
    let entry = engine()
        .parse_entry_file(&entry_path)
        .expect("the file-backed predicate-binder provenance fixture should lower");
    let lowered = entry
        .lowering_sidecars
        .callable_contracts
        .get("helper")
        .expect("the helper must retain its lowered contract sidecar");

    assert_predicate_binder_spans(lowered, helper, source, Some(&canonical_path));
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
