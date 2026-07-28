//! TASK-2032 RED contracts for the one Engine-owned admitted-program seam.
//!
//! The intended public boundary is deliberately absent while this test is
//! written: `Engine` must mint the opaque program/request artifacts and
//! return TASK-2008's canonical envelope. Neither this test nor a client may
//! construct frames, select a semantic route, or invoke a direct evaluator.

use ash_core::{Expr, Value};
use ash_engine::{
    CanonicalTerminalEnvelopeV1, Engine, ProductionTerminalClassification,
    standard_profiles::StandardProviderProfile,
};
use proptest::prelude::*;
use std::time::Duration;

const RETURN_SOURCE: &str = "fn main() -> Int { 42 }";
const UNADMITTED_SOURCE: &str = r"
fn main() -> Int {
    do {
        let value = 1;
        return - value;
    }
}
";
const SLEEP_SOURCE: &str = "fn main() -> Null { time::sleep(1) }";
const TRAP_SLEEP_SOURCE: &str = r"
interface Clock<T> { sleep(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = milliseconds }
handler trap_sleep(comp: () -> { TestClock::sleep } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => 1 / 0,
        done(value) => value,
    }
}
fn main() -> Int { handle TestClock::sleep(0) with trap_sleep }
";
const WRONG_HANDLER_SOURCE: &str = r"
interface Clock<T> { sleep(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = milliseconds }
handler unknown_sleep(comp: () -> { TestClock::sleep } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => resume(ms),
        done(value) => value,
    }
}
fn main() -> Int { handle TestClock::sleep(0) with unknown_sleep }
";
const TRAP_SLEEP_WITH_EXTRA_HANDLER_SOURCE: &str = r"
interface Clock<T> { sleep(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = milliseconds }
handler trap_sleep(comp: () -> { TestClock::sleep } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => 1 / 0,
        done(value) => value,
    }
}
handler extra_sleep(comp: () -> { TestClock::sleep } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => resume(ms),
        done(value) => value,
    }
}
fn main() -> Int { handle TestClock::sleep(0) with trap_sleep }
";
const DEEP_AFFINE_CLOCK_SOURCE: &str = r"
interface Clock<T> {
    sleep(Int) -> Int
    wake(Int) -> Int
}

type TestClock = SystemClock(Int);
impl Clock<TestClock> {
    sleep(milliseconds) = milliseconds
    wake(milliseconds) = milliseconds
}

handler deep_affine_clock(comp: () -> { TestClock::sleep, TestClock::wake } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => resume(ms),
        TestClock::wake(ms, resume) => resume(ms),
        done(value) => value + 100,
    }
}

fn main() -> Int {
    handle {
        TestClock::sleep(0);
        TestClock::wake(1);
        TestClock::sleep(2);
        7
    } with deep_affine_clock
}
";

fn admit_program_implementation() -> &'static str {
    const ENGINE_SOURCE: &str = include_str!("../src/lib.rs");
    let start = ENGINE_SOURCE
        .find("    pub fn admit_program(&self, entry: &mut Entry)")
        .expect("Engine::admit_program remains a public Engine boundary");
    let body_and_following = &ENGINE_SOURCE[start..];
    let end = body_and_following
        .find("\n    /// Create a reusable Engine-owned execution request")
        .expect("Engine::admit_program ends before request construction");

    &body_and_following[..end]
}

fn function_implementation(function_name: &str) -> String {
    const ENGINE_SOURCE: &str = include_str!("../src/lib.rs");
    let signature = format!("fn {function_name}(");
    let start = ENGINE_SOURCE.find(&signature).unwrap_or_else(|| {
        panic!("shared-admission materializer '{function_name}' remains present")
    });
    let body_start = ENGINE_SOURCE[start..]
        .find('{')
        .map(|offset| start + offset)
        .expect("materializer declaration has a body");

    let mut depth = 0_u32;
    for (offset, character) in ENGINE_SOURCE[body_start..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return ENGINE_SOURCE[start..=body_start + offset].to_string();
                }
            }
            _ => {}
        }
    }

    panic!("materializer '{function_name}' has a balanced implementation body");
}

fn shared_admission_materialization_call_chain() -> Vec<(&'static str, String)> {
    // These are the current route-materialization edges. The traversal starts
    // at the sole public seam and follows only helpers that can construct the
    // selected pure/provider/handler admissions; it deliberately excludes
    // unrelated historical validation and execution APIs.
    const CANDIDATES: &[&str] = &[
        "admit_program",
        "admit_entry_to_checked_cps",
        "lower_entry_to_checked_cps",
        "checked_cps_exact_local_call_core",
        "checked_cps_answer_input_type",
        "admit_production_checked_cps",
        "admit_production_checked_handler",
        "admit_checked_handler_inspection",
        "admit_production_forward_sleep",
        "admit_production_deep_affine_clock",
        "sealed_forward_sleep_operation_facts",
    ];

    let mut seen = std::collections::BTreeSet::new();
    let mut pending = vec!["admit_program"];
    let mut materializers = Vec::new();

    while let Some(function_name) = pending.pop() {
        if !seen.insert(function_name) {
            continue;
        }
        let implementation = function_implementation(function_name);
        for candidate in CANDIDATES {
            if candidate != &function_name && implementation.contains(&format!("{candidate}(")) {
                pending.push(candidate);
            }
        }
        materializers.push((function_name, implementation));
    }

    materializers
}

async fn install_time_sleep_profile(engine: &Engine) {
    engine
        .install_standard_profile(StandardProviderProfile::application_default(
            "task-2032-shared-engine-seam",
            std::iter::empty::<&std::path::Path>(),
            std::iter::empty::<&str>(),
        ))
        .await
        .expect("the bounded time.sleep fixture installs its application provider profile");
    engine
        .register_time_sleep_provider_binding()
        .expect("the Engine seals the checked time.sleep provider binding before admission");
}

#[tokio::test]
async fn admitted_program_request_projects_a_versioned_return_envelope() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine.parse(RETURN_SOURCE).expect("return fixture parses");

    let program = engine
        .admit_program(&mut entry)
        .expect("a selected checked return receives an Engine-issued admitted-program artifact");
    let (request, _cancellation) = engine
        .new_admitted_program_request(&program, None)
        .expect("only the issuing Engine creates a request after admission");
    let envelope = engine
        .execute_admitted_program(&request)
        .await
        .expect("the Engine executes the admitted request through the shared seam");

    assert_eq!(
        envelope,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(42)),
        "the Engine seam owns the versioned normalized return projection"
    );
}

proptest! {
    #[test]
    fn admitted_literal_return_projects_the_same_value_through_the_engine_seam(
        value in 0_i64..=10_000,
    ) {
        let engine = Engine::new().build().expect("engine builds");
        let source = format!("fn main() -> Int {{ {value} }}");
        let mut entry = engine
            .parse(&source)
            .expect("bounded literal-return fixture parses");
        let program = engine
            .admit_program(&mut entry)
            .expect("the Engine admits every bounded literal return");
        let (request, _cancellation) = engine
            .new_admitted_program_request(&program, None)
            .expect("the issuing Engine mints each admitted literal request");
        let terminal = tokio_test::block_on(engine.execute_admitted_program(&request))
            .expect("the Engine executes every admitted literal request");

        prop_assert_eq!(
            terminal,
            CanonicalTerminalEnvelopeV1::returned(Value::Int(value)),
        );
    }
}

#[tokio::test]
async fn admitted_program_and_request_reject_a_foreign_engine_before_dispatch() {
    let issuing_engine = Engine::new().build().expect("issuing engine builds");
    let foreign_engine = Engine::new().build().expect("foreign engine builds");
    let mut entry = issuing_engine
        .parse(RETURN_SOURCE)
        .expect("return fixture parses for the issuing engine");
    let program = issuing_engine
        .admit_program(&mut entry)
        .expect("the issuing Engine mints the bounded return artifact");
    let (request, _cancellation) = issuing_engine
        .new_admitted_program_request(&program, None)
        .expect("the issuing Engine mints the bounded return request");

    let Err(foreign_request_error) = foreign_engine.new_admitted_program_request(&program, None)
    else {
        panic!("a foreign Engine must not mint a request from another Engine's artifact");
    };
    assert!(
        matches!(foreign_request_error, ash_engine::EngineError::Type(_)),
        "foreign request construction fails at the opaque issuer boundary"
    );

    let foreign_execution_error = foreign_engine
        .execute_admitted_program(&request)
        .await
        .expect_err("a foreign Engine must not dispatch another Engine's request");
    assert!(
        matches!(foreign_execution_error, ash_engine::EngineError::Type(_)),
        "foreign execution fails at the opaque issuer boundary"
    );
    assert!(
        issuing_engine.host_boundary_evidence().await.is_empty(),
        "the issuing Engine does not dispatch while its request is rejected by another Engine"
    );
    assert!(
        foreign_engine.host_boundary_evidence().await.is_empty(),
        "the foreign Engine rejects before provider or host dispatch"
    );
}

#[test]
fn missing_admission_rejects_before_a_client_can_submit_a_request() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(UNADMITTED_SOURCE)
        .expect("unsupported-lowering fixture still parses");
    engine
        .check(&mut entry)
        .expect("unsupported-lowering fixture still type-checks");

    let rejection = engine.admit_program(&mut entry).expect_err(
        "a source without validated lowering must not mint an admitted-program artifact",
    );

    assert_eq!(
        rejection.classification(),
        ProductionTerminalClassification::MissingAdmission,
        "a client cannot replace missing admission with direct evaluation"
    );
    assert_eq!(
        rejection
            .canonical_terminal_envelope()
            .expect("a sealed missing-admission rejection has a canonical terminal envelope",),
        CanonicalTerminalEnvelopeV1::admission_rejected(),
        "the shared Engine boundary owns the canonical rejection outcome"
    );
}

#[test]
fn forged_checked_evidence_rejects_before_engine_frame_dispatch() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine.parse(SLEEP_SOURCE).expect("sleep fixture parses");
    engine
        .check(&mut entry)
        .expect("sleep fixture type-checks before its public legacy field is forged");
    entry.core = Expr::Literal(Value::Null);

    let rejection = engine
        .admit_program(&mut entry)
        .expect_err("forged checked evidence must not mint an admitted-program artifact");

    assert_eq!(
        rejection.classification(),
        ProductionTerminalClassification::InvalidCheckedCoreCps,
        "provenance failure is not an authorization source for a provider or frame"
    );
    assert_eq!(
        rejection
            .canonical_terminal_envelope()
            .expect("a sealed invalid-artifact rejection has a canonical terminal envelope",),
        CanonicalTerminalEnvelopeV1::invalid_checked_artifact(),
        "the failure is a stable pre-execution terminal projection"
    );
}

#[test]
fn forged_checked_sidecar_rejects_before_engine_frame_dispatch() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine.parse(SLEEP_SOURCE).expect("sleep fixture parses");
    engine
        .check(&mut entry)
        .expect("sleep fixture type-checks before its public source sidecar is forged");
    entry.lowering_sidecars.entry_body_origin.label =
        "forged task-2032 admitted-program source sidecar".to_string();

    let rejection = engine
        .admit_program(&mut entry)
        .expect_err("forged checked sidecars must not mint an admitted-program artifact");

    assert_eq!(
        rejection.classification(),
        ProductionTerminalClassification::InvalidCheckedCoreCps,
        "public source sidecars cannot retarget a sealed provider or frame route"
    );
    assert_eq!(
        rejection
            .canonical_terminal_envelope()
            .expect("a sealed invalid-artifact rejection has a canonical terminal envelope"),
        CanonicalTerminalEnvelopeV1::invalid_checked_artifact(),
        "the provenance failure remains a pre-execution terminal projection"
    );
}

#[tokio::test]
async fn post_check_public_artifact_mutation_rejects_before_provider_or_frame_dispatch() {
    for mutation in ["legacy Core", "source sidecar"] {
        let engine = Engine::new().build().expect("engine builds");
        install_time_sleep_profile(&engine).await;
        let mut entry = engine.parse(SLEEP_SOURCE).expect("sleep fixture parses");
        engine
            .check(&mut entry)
            .expect("the initial source artifact type-checks before mutation");

        match mutation {
            "legacy Core" => entry.core = Expr::Literal(Value::Null),
            "source sidecar" => {
                entry.lowering_sidecars.entry_body_origin.label =
                    "forged post-check source provenance".to_string();
            }
            _ => unreachable!("the mutation matrix is closed"),
        }

        let rejection = engine
            .admit_program(&mut entry)
            .expect_err("a mutation after checking must not be refreshed into an admitted route");

        assert_eq!(
            rejection.classification(),
            ProductionTerminalClassification::InvalidCheckedCoreCps,
            "{mutation} mutation is invalid checked evidence, not an authorization source"
        );
        assert!(
            engine.host_boundary_evidence().await.is_empty(),
            "{mutation} mutation must reject before provider dispatch or handler-frame execution"
        );
    }
}

#[tokio::test]
async fn typed_wrong_or_extra_handler_facts_reject_without_an_admitted_fallback() {
    for (name, source) in [
        ("wrong selected handler", WRONG_HANDLER_SOURCE),
        (
            "extra checked handler alongside the selected trap handler",
            TRAP_SLEEP_WITH_EXTRA_HANDLER_SOURCE,
        ),
    ] {
        let engine = Engine::new().build().expect("engine builds");
        let mut entry = engine
            .parse(source)
            .unwrap_or_else(|error| panic!("{name} fixture parses: {error}"));
        engine.check(&mut entry).unwrap_or_else(|error| {
            panic!("{name} fixture produces typed handler facts before admission: {error}")
        });

        assert!(
            engine.admit_program(&mut entry).is_err(),
            "{name} must reject at the shared admission seam rather than ignore a fact or select a fallback"
        );
        assert!(
            engine.host_boundary_evidence().await.is_empty(),
            "{name} must reject before host dispatch"
        );
    }
}

#[test]
fn shared_admission_selects_only_private_checked_route_facts() {
    let admission = admit_program_implementation();

    assert!(
        !admission.contains("matches_legacy_call"),
        "Engine::admit_program must not classify a production route from a legacy Core call shape"
    );
    assert!(
        !admission.contains(".contains_key(SEALED_"),
        "Engine::admit_program must not classify a production route from a handler name"
    );
    assert!(
        !admission.contains("entry.core"),
        "Engine::admit_program must not inspect public legacy Core as a source route selector"
    );
    assert!(
        !admission.contains("is_exact_pure_helper_projection_program"),
        "Engine::admit_program must not derive terminal projection metadata from a raw Surface helper shape"
    );
    assert!(
        !admission.contains("get_surface_program"),
        "Engine::admit_program must not select terminal projection metadata from any raw Surface program"
    );
}

#[test]
fn shared_admission_materialization_never_reselects_from_surface_or_legacy_artifacts() {
    let materializers = shared_admission_materialization_call_chain();
    let top_level = materializers
        .iter()
        .find(|(name, _)| *name == "admit_program")
        .expect("the materialization traversal begins at Engine::admit_program");
    let route_selection = top_level
        .1
        .find("let route = match")
        .expect("Engine::admit_program selects the private checked route exactly once");
    let initial_check = top_level
        .1
        .find("self.check(entry)")
        .expect("Engine::admit_program checks before it selects the sealed route");
    assert!(
        initial_check < route_selection,
        "checking must complete before the shared seam selects its private route fact"
    );
    assert!(
        !top_level.1[route_selection..].contains("self.check("),
        "the shared seam must not re-check after it has selected a sealed route"
    );

    let forbidden_reselectors = [
        "checked_handlers",
        "checked_handler_applications",
        "get_surface_program",
        "entry.core",
        "parsed_legacy_core",
        "entry.lowering_sidecars",
        "matches_legacy_call",
        "TIME_SLEEP_OPERATION",
        "is_exact_",
        "checked_cps_is_exact_local_call",
        "lower_checked_handler_application_to_core",
    ];
    let mut violations = Vec::new();

    for (function_name, implementation) in materializers {
        if function_name != "admit_program" && implementation.contains("self.check(") {
            violations.push(format!(
                "{function_name} performs a second check after route selection"
            ));
        }
        for forbidden in forbidden_reselectors {
            if implementation.contains(forbidden) {
                violations.push(format!(
                    "{function_name} reselects from forbidden '{forbidden}' data"
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "checking must seal complete typed route evidence/Core/CPS; admission may materialize only from that sealed evidence plus Engine host bindings:\n{}",
        violations.join("\n")
    );
}

#[tokio::test]
async fn admitted_handler_body_trap_is_a_canonical_terminal_envelope() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(TRAP_SLEEP_SOURCE)
        .expect("handler-body trap fixture parses");

    let program = engine
        .admit_program(&mut entry)
        .expect("validated handler evidence mints only an Engine-issued artifact");
    let (request, _cancellation) = engine
        .new_admitted_program_request(&program, None)
        .expect("the issuing Engine creates one request");
    let envelope = engine
        .execute_admitted_program(&request)
        .await
        .expect("a language-level trap is projected as a canonical result, not a client error");

    assert_eq!(
        envelope,
        CanonicalTerminalEnvelopeV1::trapped("division by zero"),
        "the handler-body trap has one Engine-owned terminal projection"
    );
}

#[tokio::test]
async fn admitted_deep_affine_clock_projects_its_checked_cps_return_through_the_shared_seam() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(DEEP_AFFINE_CLOCK_SOURCE)
        .expect("the exact deep-affine source fixture parses");
    engine
        .check(&mut entry)
        .expect("the exact deep-affine source fixture has checked handler facts before admission");

    let program = engine
        .admit_program(&mut entry)
        .expect("the Engine converts only the sealed deep-affine admission into an opaque program");
    let (request, _cancellation) = engine
        .new_admitted_program_request(&program, None)
        .expect("the issuing Engine creates a shared request after deep-affine admission");
    let envelope = engine
        .execute_admitted_program(&request)
        .await
        .expect("the deep-affine handler executes only through the shared Engine seam");

    assert_eq!(
        envelope,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(107)),
        "the checked deep-affine handler retains its current bounded CPS result without direct evaluation"
    );
}

#[tokio::test(start_paused = true)]
async fn admitted_program_timeout_is_projected_by_the_engine_seam() {
    let engine = Engine::new().build().expect("engine builds");
    install_time_sleep_profile(&engine).await;
    let mut entry = engine.parse(SLEEP_SOURCE).expect("sleep fixture parses");
    let program = engine
        .admit_program(&mut entry)
        .expect("the checked sleep artifact is admitted before its execution deadline starts");
    let (request, _cancellation) = engine
        .new_admitted_program_request(&program, Some(Duration::ZERO))
        .expect("a post-admission zero deadline is representable");

    let envelope = engine
        .execute_admitted_program(&request)
        .await
        .expect("timeout is a canonical terminal result rather than a client-local race");

    assert_eq!(envelope, CanonicalTerminalEnvelopeV1::timed_out());
}

#[tokio::test(start_paused = true)]
async fn admitted_program_cancellation_is_projected_by_the_engine_seam() {
    let engine = Engine::new().build().expect("engine builds");
    install_time_sleep_profile(&engine).await;
    let mut entry = engine.parse(SLEEP_SOURCE).expect("sleep fixture parses");
    let program = engine
        .admit_program(&mut entry)
        .expect("the checked sleep artifact is admitted before cancellation is available");
    let (request, cancellation) = engine
        .new_admitted_program_request(&program, None)
        .expect("the issuing Engine creates a cancellation pair only after admission");
    cancellation.cancel();

    let envelope = engine
        .execute_admitted_program(&request)
        .await
        .expect("cancellation is a canonical terminal result rather than a client-local error");

    assert_eq!(envelope, CanonicalTerminalEnvelopeV1::cancelled());
}
