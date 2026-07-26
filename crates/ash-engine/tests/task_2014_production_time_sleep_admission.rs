//! TASK-2014 RED coverage for the first sealed production CPS admission.
//!
//! A source row, a registered provider, or public V1 inspection evidence is
//! not executable authority. Only the exact checked `time::sleep(0)` producer
//! plus an explicit Engine-owned `time.sleep` binding may mint this opaque
//! production token.

use ash_core::{
    Effect, Expr, Value,
    capability::{
        CapabilityError, CapabilityProvider, ProviderAuthoringMetadata, ProviderOperationMetadata,
    },
    semantic_summary::SourceOrigin,
};
use ash_engine::{
    Engine, checked_cps_admission::FrameInstallationInstructionV1,
    standard_profiles::StandardProviderProfile,
};
use async_trait::async_trait;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

const SLEEP: &str = "fn main() -> Null { time::sleep(0) }";
const FORGED_SLEEP_SOURCE: &str = "fn main() -> Null { null }";

const HANDLER_RESUME_DONE: &str = r"
interface Clock<T> { sleep(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = milliseconds }
handler resume_sleep(comp: () -> { TestClock::sleep } Int) -> Int {
    on comp {
        TestClock::sleep(milliseconds, resume) => resume(milliseconds),
        done(value) => value,
    }
}
fn main() -> Int { handle TestClock::sleep(0) with resume_sleep }
";

const OPEN_TAIL: &str = r"
interface Clock<T> { sleep(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> {
    sleep(milliseconds) = milliseconds
    derive handler clock;
}
effect alias OpenClock = { TestClock::sleep | rest };
fn main(computation: () -> { OpenClock } Int) -> Int {
    handle computation() with clock
}
";

const NON_TIME_RAISE: &str = r"
interface Clock<T> { sleep(Int) -> Null }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = null }
fn main() -> Null { TestClock::sleep(0) }
";

fn checked_entry(engine: &Engine, source: &str) -> ash_engine::Entry {
    let mut entry = engine.parse(source).expect("TASK-2014 fixture parses");
    engine
        .check(&mut entry)
        .expect("TASK-2014 fixture type-checks before production admission");
    entry
}

async fn install_application_time_profile(engine: &Engine) {
    engine
        .install_standard_profile(StandardProviderProfile::application_default(
            "task-2014-production-time",
            std::iter::empty::<&std::path::Path>(),
            std::iter::empty::<&str>(),
        ))
        .await
        .expect("the standard application profile installs the time provider");
}

#[derive(Debug)]
struct WrongTimeProvider {
    executions: Arc<AtomicUsize>,
}

#[async_trait]
impl CapabilityProvider for WrongTimeProvider {
    fn name(&self) -> &'static str {
        "time"
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    fn provider_metadata(&self) -> ProviderAuthoringMetadata {
        ProviderAuthoringMetadata::new("time").with_operation(
            ProviderOperationMetadata::new("wake", Effect::Operational)
                .with_required_row("time.sleep")
                .with_sandbox_policy("host.time.wake")
                .with_provenance_policy("host.time.wake.redacted"),
        )
    }

    async fn observe(
        &self,
        _constraints: &[ash_core::Constraint],
    ) -> Result<Value, CapabilityError> {
        Err(CapabilityError::NotAvailable(
            "wrong time provider does not observe".to_string(),
        ))
    }

    async fn execute(&self, _action: &str, _args: &[Value]) -> Result<Value, CapabilityError> {
        self.executions.fetch_add(1, Ordering::SeqCst);
        Ok(Value::Null)
    }
}

#[tokio::test]
async fn exact_typed_time_sleep_with_registered_binding_seals_a_production_token() {
    let engine = Engine::new().build().expect("engine builds");
    install_application_time_profile(&engine).await;
    engine
        .register_time_sleep_provider_binding()
        .expect("the exact Engine-owned time.sleep binding registers");
    let mut entry = checked_entry(&engine, SLEEP);
    let expected_anchor = entry.lowering_sidecars.entry_body_origin.clone();

    let admission = engine
        .admit_production_checked_cps(&mut entry)
        .expect("only the exact typed time::sleep producer seals production admission");

    assert_eq!(admission.source_anchor(), &expected_anchor);
    let [
        FrameInstallationInstructionV1::Provider {
            operation,
            provider_binding,
        },
    ] = admission.frame_installation_summary()
    else {
        panic!("production time::sleep admission must retain one Provider instruction");
    };
    assert_eq!(operation.impl_type(), "time");
    assert_eq!(operation.operation(), "sleep");
    assert_eq!(operation.parameter_types(), ["Int"]);
    assert_eq!(operation.result_type(), "Null");
    assert_eq!(provider_binding.operation(), &operation);
    assert_eq!(provider_binding.provider_name(), "time");
    assert_eq!(provider_binding.provider_operation(), "sleep");
    assert!(
        engine.host_boundary_evidence().await.is_empty(),
        "sealing production authority must not execute a provider"
    );
}

#[tokio::test]
async fn forged_legacy_core_sleep_cannot_replace_the_checked_source_operation_fact() {
    let engine = Engine::new().build().expect("engine builds");
    install_application_time_profile(&engine).await;
    engine
        .register_time_sleep_provider_binding()
        .expect("the exact Engine-owned time.sleep binding registers");
    let mut entry = checked_entry(&engine, FORGED_SLEEP_SOURCE);

    // `Entry::core` remains public legacy compatibility data.  Giving it the
    // exact syntactic shape, duration, and result type of the admitted
    // operation must not replace the Engine-retained checked source program,
    // which established only `null` for this entry.
    entry.core = Expr::Call {
        func: "sleep".into(),
        module: Some("time".into()),
        arguments: vec![Expr::Literal(Value::Int(0))],
    };

    assert!(
        engine.admit_production_checked_cps(&mut entry).is_err(),
        "a forged legacy Core call must not mint production authority without an exact checked source operation fact"
    );
    assert!(
        engine.host_boundary_evidence().await.is_empty(),
        "forged admission must reject before provider work"
    );
}

#[tokio::test]
async fn forged_entry_anchor_cannot_be_rechecked_and_sealed_for_production() {
    let engine = Engine::new().build().expect("engine builds");
    install_application_time_profile(&engine).await;
    engine
        .register_time_sleep_provider_binding()
        .expect("the exact Engine-owned time.sleep binding registers");
    let mut entry = checked_entry(&engine, SLEEP);
    let original_anchor = entry.lowering_sidecars.entry_body_origin.clone();

    // This public diagnostic sidecar is mutable. Production authority must
    // remain tied to the exact anchor parsed by the issuing Engine, rather
    // than accepting a later re-check of caller-forged provenance.
    entry.lowering_sidecars.entry_body_origin.origin = SourceOrigin::Synthetic {
        reason: "forged production origin".to_string(),
    };
    entry.lowering_sidecars.entry_body_origin.label = "forged entry callable".to_string();

    assert_ne!(entry.lowering_sidecars.entry_body_origin, original_anchor);
    assert!(
        engine.admit_production_checked_cps(&mut entry).is_err(),
        "production admission must reject an entry whose public source anchor changed after parsing"
    );
    assert!(
        engine.host_boundary_evidence().await.is_empty(),
        "forged provenance must reject before any provider work"
    );
}

#[tokio::test]
async fn no_binding_wrong_binding_row_only_or_foreign_engine_cannot_seal_time_sleep() {
    let no_binding_engine = Engine::new().build().expect("engine builds");
    let mut no_binding = checked_entry(&no_binding_engine, SLEEP);
    assert!(
        no_binding_engine
            .admit_production_checked_cps(&mut no_binding)
            .is_err(),
        "a checked time::sleep raise with no provider binding must reject"
    );
    assert!(
        no_binding_engine.host_boundary_evidence().await.is_empty(),
        "missing binding must reject before provider work"
    );

    let row_only_engine = Engine::new().build().expect("engine builds");
    install_application_time_profile(&row_only_engine).await;
    let mut row_only = checked_entry(&row_only_engine, SLEEP);
    assert!(
        row_only_engine
            .admit_production_checked_cps(&mut row_only)
            .is_err(),
        "a time.sleep row plus a registered provider must not synthesize a production binding"
    );
    assert!(
        row_only_engine.host_boundary_evidence().await.is_empty(),
        "row-only admission must reject before provider work"
    );

    let wrong_executions = Arc::new(AtomicUsize::new(0));
    let wrong_binding_engine = Engine::new()
        .with_custom_provider(
            "time",
            Arc::new(WrongTimeProvider {
                executions: Arc::clone(&wrong_executions),
            }),
        )
        .build()
        .expect("engine with malformed time metadata builds");
    assert!(
        wrong_binding_engine
            .register_time_sleep_provider_binding()
            .is_err(),
        "the exact registrar must reject a time provider without time.sleep metadata"
    );
    let mut wrong_binding = checked_entry(&wrong_binding_engine, SLEEP);
    assert!(
        wrong_binding_engine
            .admit_production_checked_cps(&mut wrong_binding)
            .is_err(),
        "a failed exact registration must not leave a usable production binding"
    );
    assert_eq!(wrong_executions.load(Ordering::SeqCst), 0);
    assert!(
        wrong_binding_engine
            .host_boundary_evidence()
            .await
            .is_empty(),
        "wrong binding rejection must occur before provider work"
    );

    let issuing_engine = Engine::new().build().expect("issuing engine builds");
    install_application_time_profile(&issuing_engine).await;
    issuing_engine
        .register_time_sleep_provider_binding()
        .expect("issuing engine registers its exact binding");
    let mut foreign_entry = checked_entry(&issuing_engine, SLEEP);
    let foreign_engine = Engine::new().build().expect("foreign engine builds");
    assert!(
        foreign_engine
            .admit_production_checked_cps(&mut foreign_entry)
            .is_err(),
        "a distinct Engine must not seal an entry it did not parse and check"
    );
    assert!(
        issuing_engine.host_boundary_evidence().await.is_empty()
            && foreign_engine.host_boundary_evidence().await.is_empty(),
        "foreign admission must reject before provider work"
    );
}

#[tokio::test]
async fn handler_resume_done_open_tail_and_non_time_raises_remain_closed() {
    for (name, source) in [
        ("handler/resume/done", HANDLER_RESUME_DONE),
        ("open tail", OPEN_TAIL),
        ("non-time raise", NON_TIME_RAISE),
    ] {
        let engine = Engine::new().build().expect("engine builds");
        let mut entry = checked_entry(&engine, source);

        assert!(
            engine.admit_production_checked_cps(&mut entry).is_err(),
            "{name} must remain outside the first production time::sleep admission slice"
        );
        assert!(
            engine.host_boundary_evidence().await.is_empty(),
            "{name} must reject before any provider work"
        );
    }
}
