//! TASK-2039 controls for REPL submission through the Engine-owned executor.
//!
//! The declared corpus contains only the two TASK-2035 REPL source controls.
//! It deliberately does not synthesize, mutate, or otherwise generate source.

use std::path::Path;

use ash_core::Value;
use ash_engine::{CanonicalTerminalEnvelopeV1, Engine};
use ash_repl::{
    InputDetector, InputStatus, Repl, ReplError, ast_display, help_text, infer_type_display,
};
use proptest::prelude::*;

const REPL_ROUTE_INT_ID: &str = "TASK-2035-REPL-ROUTE-001";
const REPL_ROUTE_INT_SOURCE: &str = "fn main() -> Int { 42 }\n";
const REPL_ROUTE_BOOL_ID: &str = "TASK-2035-REPL-ROUTE-002";
const REPL_ROUTE_BOOL_SOURCE: &str = "fn main() -> Bool { 1 == 1 }\n";
const SHARED_ROUTE_ID: &str = "TASK-2035-SHARED-ROUTE-001";
const UNADMITTED_REPL_SOURCE: &str = "fn main() { 42 }\n";

const DECLARED_REPL_CORPUS: [(&str, &str); 2] = [
    (REPL_ROUTE_INT_ID, REPL_ROUTE_INT_SOURCE),
    (REPL_ROUTE_BOOL_ID, REPL_ROUTE_BOOL_SOURCE),
];

fn expected_terminal(route_id: &str) -> CanonicalTerminalEnvelopeV1 {
    match route_id {
        REPL_ROUTE_INT_ID | SHARED_ROUTE_ID => {
            CanonicalTerminalEnvelopeV1::returned(Value::Int(42))
        }
        REPL_ROUTE_BOOL_ID => CanonicalTerminalEnvelopeV1::returned(Value::Bool(true)),
        _ => panic!("test helper accepts only declared TASK-2035 REPL identities"),
    }
}

fn declared_route_source(route_id: &str) -> &'static str {
    match route_id {
        REPL_ROUTE_INT_ID => REPL_ROUTE_INT_SOURCE,
        REPL_ROUTE_BOOL_ID => REPL_ROUTE_BOOL_SOURCE,
        _ => panic!("test helper accepts only declared TASK-2035 REPL identities"),
    }
}

async fn engine_terminal_for_declared_source(source: &str) -> CanonicalTerminalEnvelopeV1 {
    let engine = Engine::new().build().expect("test Engine builds");
    let mut entry = engine
        .parse_file_source(Path::new("task-2035-repl-route.ash"), source)
        .expect("declared source parses through Engine");
    let execution = {
        let admitted = engine
            .admit_program(&mut entry)
            .expect("declared source admits through Engine");
        let (request, _cancellation) = engine
            .new_admitted_program_request(&admitted, None)
            .expect("Engine mints an admitted request with empty run control");
        engine.execute_admitted_program(&request)
    };

    execution
        .await
        .expect("Engine terminalizes the admitted request")
}

#[tokio::test]
async fn selected_repl_source_submissions_observe_their_engine_terminals() {
    for (route_id, source) in DECLARED_REPL_CORPUS {
        assert_eq!(
            engine_terminal_for_declared_source(source).await,
            expected_terminal(route_id),
            "{route_id} must remain a valid admitted Engine control"
        );

        let mut repl = Repl::new(true).expect("REPL initializes without history");
        let value = repl
            .eval(source)
            .await
            .expect("normal REPL submission reaches the admitted Engine route");

        assert_eq!(
            CanonicalTerminalEnvelopeV1::returned(value),
            expected_terminal(route_id),
            "{route_id} must render its Engine terminal without another evaluator"
        );
    }
}

#[tokio::test]
async fn unadmitted_repl_source_rejects_without_a_local_fallback() {
    let mut repl = Repl::new(true).expect("REPL initializes without history");
    let error = repl
        .eval(UNADMITTED_REPL_SOURCE)
        .await
        .expect_err("an unannotated entry lacks a checked admission artifact");

    let ReplError::Engine(message) = error else {
        panic!("expected a canonical Engine rejection, got {error:?}");
    };
    assert!(
        message.contains("admission rejected"),
        "rejection must not become a local value: {message}"
    );
}

#[tokio::test]
async fn completed_multiline_source_observes_the_engine_terminal() {
    let mut detector = InputDetector::new();
    assert!(matches!(
        detector.check("fn main() -> Int {"),
        InputStatus::Incomplete(_)
    ));
    assert!(matches!(
        detector.check(REPL_ROUTE_INT_SOURCE),
        InputStatus::Complete
    ));

    let mut repl = Repl::new(true).expect("REPL initializes without history");
    let value = repl
        .eval(REPL_ROUTE_INT_SOURCE)
        .await
        .expect("completed multiline input submits the declared source to Engine");
    assert_eq!(
        CanonicalTerminalEnvelopeV1::returned(value),
        expected_terminal(REPL_ROUTE_INT_ID)
    );
}

#[test]
fn inspection_commands_retain_no_evaluation_route() {
    assert!(help_text().contains(":help"));
    assert_eq!(infer_type_display("42").expect("type inspection"), "Int");
    assert!(
        ast_display("1 + 2")
            .expect("AST inspection")
            .contains("Binary")
    );

    let repl_source = include_str!("../src/lib.rs");
    let command_region = repl_source
        .split("fn handle_command")
        .nth(1)
        .expect("REPL command handler exists")
        .split("fn print_help")
        .next()
        .expect("REPL command handler ends before help renderer");
    assert!(
        !command_region.contains("self.eval") && !command_region.contains("self.engine"),
        "inspection commands must not select an execution route"
    );
}

#[test]
fn stored_entry_execution_submits_only_admitted_engine_requests() {
    let session_source = include_str!("../src/session.rs");
    let repl_source = include_str!("../src/lib.rs");

    assert!(
        session_source.contains("execute_admitted_program"),
        "stored-entry execution must submit an Engine-issued request"
    );
    assert!(
        repl_source.contains("execute_admitted_program"),
        "normal REPL evaluation must preserve the Engine terminal envelope"
    );
    assert!(
        !session_source.contains(".execute("),
        "stored entries must not call the retired direct Engine execution helper"
    );
    assert!(
        !repl_source.contains("self.engine.run("),
        "normal REPL evaluation must not discard the canonical terminal envelope"
    );
    assert!(
        !repl_source.contains("format!(\"fn main()"),
        "normal REPL evaluation must not synthesize a source wrapper"
    );
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 8,
        failure_persistence: None,
        .. ProptestConfig::default()
    })]

    #[test]
    fn declared_corpus_property_preserves_engine_terminal_observations(
        route_id in prop_oneof![Just(REPL_ROUTE_INT_ID), Just(REPL_ROUTE_BOOL_ID)],
    ) {
        let source = declared_route_source(route_id);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("property-test runtime builds");
        let expected = runtime.block_on(engine_terminal_for_declared_source(source));
        prop_assert_eq!(&expected, &expected_terminal(route_id));

        let rendered = runtime.block_on(async {
            let mut repl = Repl::new(true).expect("REPL initializes without history");
            repl.eval(source).await
        });
        match rendered {
            Ok(value) => prop_assert_eq!(CanonicalTerminalEnvelopeV1::returned(value), expected),
            Err(error) => prop_assert!(false, "declared source is evaluated through Engine: {error}"),
        }
    }
}

#[tokio::test]
async fn shared_route_matches_the_engine_normalized_terminal_envelope() {
    let engine_terminal = engine_terminal_for_declared_source(REPL_ROUTE_INT_SOURCE).await;
    assert_eq!(engine_terminal, expected_terminal(SHARED_ROUTE_ID));

    let mut repl = Repl::new(true).expect("REPL initializes without history");
    let repl_value = repl
        .eval(REPL_ROUTE_INT_SOURCE)
        .await
        .expect("shared source reaches the REPL Engine route");
    assert_eq!(
        CanonicalTerminalEnvelopeV1::returned(repl_value),
        engine_terminal
    );
}
