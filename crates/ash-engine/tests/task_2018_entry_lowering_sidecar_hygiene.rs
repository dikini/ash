//! TASK-2018: entry lowering must transport expanded-surface hygiene unchanged.

use ash_core::{Expr as CoreExpr, Value};
use ash_engine::{Engine, EngineError};
use ash_parser::{
    parse_surface_file,
    surface::{IdentifierHygieneContext, expand_surface_module},
};

const CLOSED_ADMISSION_ERROR: &str =
    "checked Core/CPS admission rejected: no validated production typed lowering is available";

fn engine() -> Engine {
    Engine::new().build().expect("engine builds")
}

const SECTION_SOURCE: &str = r"
infixl 6 <+> = combine;

fn combine(x: Int, y: Int) -> Int {
    x + y
}

fn section() {
    (<+>)
}

fn main() -> Int {
    42
}
";

#[tokio::test]
async fn task_2018_entry_sidecar_is_the_exact_expanded_hygiene_product() {
    let expanded = expand_surface_module(
        parse_surface_file(SECTION_SOURCE).expect("section-bearing source parses"),
    )
    .expect("section-bearing source expands");
    assert!(expanded.hygiene.iter().any(|item| {
        item.context == IdentifierHygieneContext::Generated
            && item.name.starts_with("$ash_generated_section_")
            && item.expansion_id.is_some()
    }));
    assert!(expanded.hygiene.iter().any(|item| {
        item.context == IdentifierHygieneContext::DefinitionSite
            && item.name.as_ref() == "x"
            && item.expansion_id.is_none()
    }));

    let engine = engine();
    let mut entry = engine
        .parse(SECTION_SOURCE)
        .expect("expanded source entry parses");

    assert_eq!(
        entry.lowering_sidecars.identifier_hygiene, expanded.hygiene,
        "the engine must transport the parser product without filtering, reconstruction, or normalization"
    );
    assert!(
        !format!("{:#?}", entry.core).contains("<+>"),
        "Core must not retain a notation carrier"
    );
    assert!(entry.callable_row_requirements.is_empty());
    assert!(entry.declared_concrete_operation.is_none());

    engine
        .check(&mut entry)
        .expect("hygiene sidecar must not affect checking");
    let error = engine
        .execute(&entry)
        .await
        .expect_err("source without validated typed lowering must reject at admission");
    assert!(
        matches!(error, ash_interp::ExecError::ExecutionFailed(message) if message == CLOSED_ADMISSION_ERROR),
        "hygiene-bearing source must expose the exact canonical closed-admission error"
    );
    assert!(matches!(entry.core, CoreExpr::Literal(Value::Int(42))));
}

#[test]
fn task_2018_plain_expanded_entry_keeps_source_identifier_hygiene() {
    let source = "fn main(value: Int) -> Int { value }";
    let expanded = expand_surface_module(parse_surface_file(source).expect("source parses"))
        .expect("source expands");
    let entry = engine().parse(source).expect("source entry parses");

    assert_eq!(entry.lowering_sidecars.identifier_hygiene, expanded.hygiene);
    assert!(
        entry
            .lowering_sidecars
            .identifier_hygiene
            .iter()
            .any(|item| {
                item.name.as_ref() == "value"
                    && matches!(
                        item.context,
                        IdentifierHygieneContext::DefinitionSite
                            | IdentifierHygieneContext::CallSite
                    )
                    && item.expansion_id.is_none()
            })
    );
}

#[test]
fn task_2018_expansion_rejection_returns_parse_error_before_entry_creation() {
    let source = r"
infixl 6 <+> = combine;
infixl 6 <+> = combine_again;

fn main() -> Int {
    42
}
";

    parse_surface_file(source).expect("duplicate notation source must parse before expansion");

    let error = engine()
        .parse(source)
        .expect_err("expansion rejection must prevent entry construction");
    let EngineError::Parse(message) = error else {
        panic!("expansion rejection must surface as EngineError::Parse: {error}");
    };

    assert_eq!(
        message,
        "expanded-surface validation failed: duplicate notation declaration for `<+>`"
    );
}
