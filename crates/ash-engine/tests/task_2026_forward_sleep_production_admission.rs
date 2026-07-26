//! TASK-2026 RED contract for the sealed `forward_sleep` handler-provider slice.
//!
//! This test deliberately names a separate, opaque Engine admission route. It
//! must not be satisfied by the generic `execute`, public V1, inspection, or
//! single-provider production paths.

use ash_core::{
    Effect, Expr, Value,
    capability::{
        CapabilityError, CapabilityProvider, ProviderAuthoringMetadata, ProviderOperationMetadata,
    },
};
use ash_engine::{
    Engine, ProductionCheckedCpsOutcome, checked_cps_admission::FrameInstallationInstructionV1,
};
use async_trait::async_trait;
use std::{
    collections::HashMap,
    fs,
    future::Future,
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll},
    time::Duration,
};
use tokio::sync::Notify;

const FORWARD_SLEEP_SOURCE: &str = r"
interface Clock<T> {
    sleep(Int) -> Int
    wake(Int) -> Int
}

type TestClock = SystemClock(Int);
impl Clock<TestClock> {
    sleep(milliseconds) = milliseconds
    wake(milliseconds) = milliseconds
}

handler forward_sleep(comp: () -> { TestClock::sleep } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => TestClock::wake(ms),
        done(value) => value,
    }
}

fn main() -> Int { handle TestClock::sleep(0) with forward_sleep }
";

const FORWARD_SLEEP_WITH_EXTRA_LOCAL: &str = r"
interface Clock<T> {
    sleep(Int) -> Int
    wake(Int) -> Int
}

type TestClock = SystemClock(Int);
impl Clock<TestClock> {
    sleep(milliseconds) = milliseconds
    wake(milliseconds) = milliseconds
}

handler forward_sleep(comp: () -> { TestClock::sleep } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => TestClock::wake(ms),
        done(value) => value,
    }
}

fn helper() -> Int { 1 }
fn main() -> Int { handle TestClock::sleep(0) with forward_sleep }
";

const CLOSED_ADMISSION_ERROR: &str = "application execution failed: checked Core/CPS admission rejected: no validated production typed lowering is available";

#[derive(Debug)]
struct RecordingWakeProvider {
    required_row: &'static str,
    extra_required_row: Option<&'static str>,
    operation: &'static str,
    returned_value: i64,
    calls: Arc<Mutex<Vec<Vec<Value>>>>,
}

#[derive(Debug)]
struct PendingWakeProvider {
    calls: Arc<AtomicUsize>,
    dropped_awaits: Arc<AtomicUsize>,
    started: Arc<Notify>,
}

#[derive(Debug)]
struct PendingWakeAwait {
    dropped_awaits: Arc<AtomicUsize>,
}

impl Future for PendingWakeAwait {
    type Output = Result<Value, CapabilityError>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}

impl Drop for PendingWakeAwait {
    fn drop(&mut self) {
        self.dropped_awaits.fetch_add(1, Ordering::SeqCst);
    }
}

#[async_trait]
impl CapabilityProvider for PendingWakeProvider {
    fn name(&self) -> &'static str {
        "task-2026-pending-clock"
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    fn provider_metadata(&self) -> ProviderAuthoringMetadata {
        ProviderAuthoringMetadata::new(self.name()).with_operation(
            ProviderOperationMetadata::new("wake", Effect::Operational)
                .with_required_row("TestClock.wake")
                .with_sandbox_policy("task-2026.pending-clock.wake")
                .with_provenance_policy("task-2026.pending-clock.wake.redacted"),
        )
    }

    async fn observe(
        &self,
        _constraints: &[ash_core::Constraint],
    ) -> Result<Value, CapabilityError> {
        Err(CapabilityError::NotAvailable(
            "the TASK-2026 pending provider exposes wake only".to_string(),
        ))
    }

    async fn execute(&self, action: &str, args: &[Value]) -> Result<Value, CapabilityError> {
        assert_eq!(action, "wake", "the sealed provider binding chooses wake");
        assert_eq!(
            args,
            [Value::Int(0)],
            "the handler forwards the sleep binder"
        );
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.started.notify_one();
        PendingWakeAwait {
            dropped_awaits: Arc::clone(&self.dropped_awaits),
        }
        .await
    }
}

#[async_trait]
impl CapabilityProvider for RecordingWakeProvider {
    fn name(&self) -> &'static str {
        "task-2026-test-clock"
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    fn provider_metadata(&self) -> ProviderAuthoringMetadata {
        let operation = ProviderOperationMetadata::new(self.operation, Effect::Operational)
            .with_required_row(self.required_row)
            .with_sandbox_policy("task-2026.test-clock.wake")
            .with_provenance_policy("task-2026.test-clock.wake.redacted");
        let operation = if let Some(extra_row) = self.extra_required_row {
            operation.with_required_row(extra_row)
        } else {
            operation
        };
        ProviderAuthoringMetadata::new(self.name()).with_operation(operation)
    }

    async fn observe(
        &self,
        _constraints: &[ash_core::Constraint],
    ) -> Result<Value, CapabilityError> {
        Err(CapabilityError::NotAvailable(
            "the TASK-2026 wake test double does not support observation".to_string(),
        ))
    }

    async fn execute(&self, action: &str, args: &[Value]) -> Result<Value, CapabilityError> {
        if action != "wake" {
            return Err(CapabilityError::NotAvailable(format!(
                "unexpected TASK-2026 test-clock action '{action}'"
            )));
        }
        let [Value::Int(_milliseconds)] = args else {
            return Err(CapabilityError::InvalidArgument(
                "TASK-2026 wake requires one Int argument".to_string(),
            ));
        };
        self.calls
            .lock()
            .expect("wake call log is not poisoned")
            .push(args.to_vec());
        Ok(Value::Int(self.returned_value))
    }
}

fn checked_entry(engine: &Engine) -> ash_engine::Entry {
    let mut entry = engine
        .parse(FORWARD_SLEEP_SOURCE)
        .expect("the canonical forward_sleep fixture parses");
    engine
        .check(&mut entry)
        .expect("the canonical forward_sleep fixture checks");
    entry
}

fn recording_engine(required_row: &'static str) -> (Engine, Arc<Mutex<Vec<Vec<Value>>>>) {
    recording_engine_with_result(required_row, 0)
}

fn recording_engine_with_result(
    required_row: &'static str,
    returned_value: i64,
) -> (Engine, Arc<Mutex<Vec<Vec<Value>>>>) {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let engine = Engine::new()
        .with_custom_provider(
            "task-2026-test-clock",
            Arc::new(RecordingWakeProvider {
                required_row,
                extra_required_row: None,
                operation: "wake",
                returned_value,
                calls: Arc::clone(&calls),
            }),
        )
        .build()
        .expect("TASK-2026 test engine builds");
    (engine, calls)
}

fn register_exact_wake_binding(engine: &Engine, entry: &ash_engine::Entry) {
    engine
        .register_sealed_forward_sleep_wake_provider_binding(entry, "task-2026-test-clock", "wake")
        .expect("the exact checked forward_sleep wake binding registers");
}

fn pending_wake_engine(provider: Arc<PendingWakeProvider>) -> Engine {
    Engine::new()
        .with_custom_provider("task-2026-pending-clock", provider)
        .build()
        .expect("TASK-2026 pending-wake engine builds")
}

async fn wait_until_wake_is_awaited<F>(execution: &mut Pin<Box<F>>, provider: &PendingWakeProvider)
where
    F: Future,
{
    tokio::select! {
        _ = &mut *execution => panic!("pending wake execution completed before a control decision"),
        () = provider.started.notified() => {}
    }
}

#[tokio::test]
async fn forward_sleep_admission_installs_provider_outer_then_source_handler_inner() {
    let (engine, calls) = recording_engine("TestClock.wake");
    let mut entry = checked_entry(&engine);

    let direct_error = engine
        .execute(&entry)
        .await
        .expect_err("generic execution must remain closed before sealed admission");
    assert_eq!(direct_error.to_string(), CLOSED_ADMISSION_ERROR);
    let input_error = engine
        .execute_with_input(&entry, HashMap::new())
        .await
        .expect_err("generic input execution must remain closed before sealed admission");
    assert_eq!(input_error.to_string(), CLOSED_ADMISSION_ERROR);
    assert!(
        calls
            .lock()
            .expect("wake call log remains available")
            .is_empty(),
        "generic paths must not dispatch wake"
    );

    register_exact_wake_binding(&engine, &entry);
    let admission = engine
        .admit_production_forward_sleep(&mut entry)
        .expect("only the exact checked forward_sleep source mints its opaque production token");
    let [
        FrameInstallationInstructionV1::Provider {
            operation: wake,
            provider_binding,
        },
        FrameInstallationInstructionV1::SourceHandler {
            operation: sleep,
            handler_name,
            core_handle,
        },
    ] = admission.frame_installation_summary()
    else {
        panic!(
            "TASK-2026 must seal exactly Provider(TestClock::wake) outer then SourceHandler(TestClock::sleep) inner"
        );
    };
    assert_eq!(wake.impl_type(), "TestClock");
    assert_eq!(wake.interface(), "Clock");
    assert_eq!(wake.operation(), "wake");
    assert_eq!(wake.parameter_types(), ["Int"]);
    assert_eq!(wake.result_type(), "Int");
    assert_eq!(wake, provider_binding.operation());
    assert_eq!(provider_binding.provider_name(), "task-2026-test-clock");
    assert_eq!(provider_binding.provider_operation(), "wake");
    assert_eq!(sleep.impl_type(), "TestClock");
    assert_eq!(sleep.interface(), "Clock");
    assert_eq!(sleep.operation(), "sleep");
    assert_eq!(sleep.parameter_types(), ["Int"]);
    assert_eq!(sleep.result_type(), "Int");
    assert_eq!(handler_name, "forward_sleep");
    assert!(core_handle.path().is_empty());

    let (control, _cancellation) = engine
        .new_forward_sleep_run_control(&admission, None)
        .expect("the issuing Engine creates control only for its sealed admission");
    let outcome = engine
        .execute_production_forward_sleep(&admission, control)
        .await
        .expect("the checked-CPS driver must dispatch sleep then wake through sealed frames");
    assert_eq!(outcome, ProductionCheckedCpsOutcome::Return(Value::Int(0)));
    assert_eq!(
        calls
            .lock()
            .expect("wake call log remains available")
            .as_slice(),
        [vec![Value::Int(0)]],
        "the inner source handler must forward its binder to the outer wake provider exactly once"
    );

    let run_entry = checked_entry(&engine);
    register_exact_wake_binding(&engine, &run_entry);
    assert_eq!(
        engine
            .run(FORWARD_SLEEP_SOURCE)
            .await
            .expect("run must route the exact sealed forward_sleep source through checked CPS"),
        Value::Int(0),
    );
}

#[tokio::test]
async fn forward_sleep_file_run_uses_the_same_sealed_checked_cps_route() {
    let (engine, _calls) = recording_engine("TestClock.wake");
    let binding_entry = checked_entry(&engine);
    register_exact_wake_binding(&engine, &binding_entry);
    let directory = tempfile::tempdir().expect("temporary source directory");
    let path = directory.path().join("forward-sleep.ash");
    fs::write(&path, FORWARD_SLEEP_SOURCE).expect("fixture writes");
    assert_eq!(
        engine
            .run_file(&path)
            .await
            .expect("file run must use the sealed forward_sleep route"),
        Value::Int(0),
    );
}

#[tokio::test]
async fn forward_sleep_done_clause_returns_the_wake_provider_int_result() {
    let (engine, calls) = recording_engine_with_result("TestClock.wake", 73);
    let mut entry = checked_entry(&engine);
    register_exact_wake_binding(&engine, &entry);
    let admission = engine
        .admit_production_forward_sleep(&mut entry)
        .expect("the exact checked forward_sleep source admits");
    let (control, _cancellation) = engine
        .new_forward_sleep_run_control(&admission, None)
        .expect("the issuing Engine creates the control envelope");
    assert_eq!(
        engine
            .execute_production_forward_sleep(&admission, control)
            .await
            .expect("the sealed driver accepts an Int wake result"),
        ProductionCheckedCpsOutcome::Return(Value::Int(73)),
        "done(value) => value must preserve the provider's checked Int result"
    );
    assert_eq!(
        calls
            .lock()
            .expect("wake call log remains available")
            .as_slice(),
        [vec![Value::Int(0)]],
        "the source sleep argument and provider result remain distinct checked values"
    );
}

#[tokio::test(start_paused = true)]
async fn forward_sleep_timeout_drops_the_pending_wake_await() {
    let provider = Arc::new(PendingWakeProvider {
        calls: Arc::new(AtomicUsize::new(0)),
        dropped_awaits: Arc::new(AtomicUsize::new(0)),
        started: Arc::new(Notify::new()),
    });
    let engine = pending_wake_engine(Arc::clone(&provider));
    let mut entry = checked_entry(&engine);
    engine
        .register_sealed_forward_sleep_wake_provider_binding(
            &entry,
            "task-2026-pending-clock",
            "wake",
        )
        .expect("the exact pending wake provider binding registers");
    let admission = engine
        .admit_production_forward_sleep(&mut entry)
        .expect("the exact checked forward_sleep source admits");
    let (control, _cancellation) = engine
        .new_forward_sleep_run_control(&admission, Some(Duration::from_millis(1)))
        .expect("only the issuer creates the forward_sleep control envelope");
    let mut execution = Box::pin(engine.execute_production_forward_sleep(&admission, control));
    wait_until_wake_is_awaited(&mut execution, &provider).await;

    tokio::time::advance(Duration::from_millis(1)).await;
    assert!(matches!(
        execution
            .await
            .expect("timeout is a terminal execution outcome, not an admission failure"),
        ProductionCheckedCpsOutcome::TimedOut
    ));
    assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        provider.dropped_awaits.load(Ordering::SeqCst),
        1,
        "timeout must cooperatively drop the in-flight wake await"
    );
}

#[tokio::test(start_paused = true)]
async fn forward_sleep_cancellation_wins_over_expired_timeout_and_drops_wake_await() {
    let provider = Arc::new(PendingWakeProvider {
        calls: Arc::new(AtomicUsize::new(0)),
        dropped_awaits: Arc::new(AtomicUsize::new(0)),
        started: Arc::new(Notify::new()),
    });
    let engine = pending_wake_engine(Arc::clone(&provider));
    let mut entry = checked_entry(&engine);
    engine
        .register_sealed_forward_sleep_wake_provider_binding(
            &entry,
            "task-2026-pending-clock",
            "wake",
        )
        .expect("the exact pending wake provider binding registers");
    let admission = engine
        .admit_production_forward_sleep(&mut entry)
        .expect("the exact checked forward_sleep source admits");
    let (control, cancellation) = engine
        .new_forward_sleep_run_control(&admission, Some(Duration::from_millis(1)))
        .expect("only the issuer creates the forward_sleep control envelope");
    let mut execution = Box::pin(engine.execute_production_forward_sleep(&admission, control));
    wait_until_wake_is_awaited(&mut execution, &provider).await;

    cancellation.cancel();
    tokio::time::advance(Duration::from_millis(1)).await;
    assert!(matches!(
        execution
            .await
            .expect("cancellation is a terminal execution outcome, not an admission failure"),
        ProductionCheckedCpsOutcome::Cancelled
    ));
    assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        provider.dropped_awaits.load(Ordering::SeqCst),
        1,
        "cancellation must win the simultaneous deadline and drop the wake await"
    );
}

#[test]
fn forward_sleep_missing_or_mismatched_wake_bindings_reject_before_dispatch() {
    let (missing_engine, missing_calls) = recording_engine("TestClock.wake");
    let mut missing_entry = checked_entry(&missing_engine);
    assert!(
        missing_engine
            .admit_production_forward_sleep(&mut missing_entry)
            .is_err(),
        "the checked local residual wake row cannot synthesize a provider frame"
    );
    assert!(
        missing_calls
            .lock()
            .expect("wake call log remains available")
            .is_empty(),
        "missing binding must reject before provider dispatch"
    );

    let (mismatch_engine, mismatch_calls) = recording_engine("OtherClock.wake");
    let mismatch_entry = checked_entry(&mismatch_engine);
    assert!(
        mismatch_engine
            .register_sealed_forward_sleep_wake_provider_binding(
                &mismatch_entry,
                "task-2026-test-clock",
                "wake",
            )
            .is_err(),
        "a provider that names a different declared operation must not bind wake"
    );
    let mut mismatch_entry = mismatch_entry;
    assert!(
        mismatch_engine
            .admit_production_forward_sleep(&mut mismatch_entry)
            .is_err(),
        "a rejected binding must not leave forward_sleep production authority behind"
    );
    assert!(
        mismatch_calls
            .lock()
            .expect("wake call log remains available")
            .is_empty(),
        "mismatched binding must reject before provider dispatch"
    );
}

#[test]
fn forward_sleep_binding_rejects_extra_rows_and_non_wake_provider_operations() {
    let extra_rows_calls = Arc::new(Mutex::new(Vec::new()));
    let extra_rows_engine = Engine::new()
        .with_custom_provider(
            "task-2026-test-clock",
            Arc::new(RecordingWakeProvider {
                required_row: "TestClock.wake",
                extra_required_row: Some("TestClock.sleep"),
                operation: "wake",
                returned_value: 0,
                calls: Arc::clone(&extra_rows_calls),
            }),
        )
        .build()
        .expect("extra-row test engine builds");
    let extra_rows_entry = checked_entry(&extra_rows_engine);
    assert!(
        extra_rows_engine
            .register_sealed_forward_sleep_wake_provider_binding(
                &extra_rows_entry,
                "task-2026-test-clock",
                "wake",
            )
            .is_err(),
        "a wake binding may authorize exactly its residual wake row, not an additional frame"
    );

    let mislabeled_calls = Arc::new(Mutex::new(Vec::new()));
    let mislabeled_engine = Engine::new()
        .with_custom_provider(
            "task-2026-test-clock",
            Arc::new(RecordingWakeProvider {
                required_row: "TestClock.wake",
                extra_required_row: None,
                operation: "sleep",
                returned_value: 0,
                calls: Arc::clone(&mislabeled_calls),
            }),
        )
        .build()
        .expect("mislabeled-operation test engine builds");
    let mislabeled_entry = checked_entry(&mislabeled_engine);
    assert!(
        mislabeled_engine
            .register_sealed_forward_sleep_wake_provider_binding(
                &mislabeled_entry,
                "task-2026-test-clock",
                "wake",
            )
            .is_err(),
        "the public requested action cannot rename a provider-declared sleep operation into wake"
    );
    assert!(
        extra_rows_calls
            .lock()
            .expect("extra-row call log remains available")
            .is_empty()
            && mislabeled_calls
                .lock()
                .expect("mislabeled call log remains available")
                .is_empty(),
        "hostile metadata must fail before dispatch"
    );
}

#[test]
fn forward_sleep_admission_rejects_a_program_with_an_extra_local_declaration() {
    let (engine, calls) = recording_engine("TestClock.wake");
    let mut entry = engine
        .parse(FORWARD_SLEEP_WITH_EXTRA_LOCAL)
        .expect("the extra-local fixture parses");
    engine
        .check(&mut entry)
        .expect("the extra-local fixture type checks before its sealed-shape rejection");
    register_exact_wake_binding(&engine, &entry);
    assert!(
        engine.admit_production_forward_sleep(&mut entry).is_err(),
        "the exact production route has closed admission and cannot silently accept extra locals"
    );
    assert!(
        calls
            .lock()
            .expect("wake call log remains available")
            .is_empty(),
        "source-shape rejection must occur before provider dispatch"
    );
}

#[test]
fn forward_sleep_admission_rejects_foreign_and_mutated_public_provenance() {
    let (issuing_engine, calls) = recording_engine("TestClock.wake");
    let mut issued_entry = checked_entry(&issuing_engine);
    register_exact_wake_binding(&issuing_engine, &issued_entry);
    let admission = issuing_engine
        .admit_production_forward_sleep(&mut issued_entry)
        .expect("the issuing Engine admits its exact entry");
    let foreign_engine = Engine::new().build().expect("foreign Engine builds");
    assert!(
        foreign_engine
            .new_forward_sleep_run_control(&admission, None)
            .is_err(),
        "a foreign Engine cannot create control for a forward_sleep admission"
    );

    let mut forged_anchor = checked_entry(&issuing_engine);
    register_exact_wake_binding(&issuing_engine, &forged_anchor);
    forged_anchor.lowering_sidecars.entry_body_origin.label = "forged TASK-2026 anchor".to_string();
    assert!(
        issuing_engine
            .admit_production_forward_sleep(&mut forged_anchor)
            .is_err(),
        "a public source-anchor mutation must reject before constructing frames"
    );

    let mut forged_core = checked_entry(&issuing_engine);
    register_exact_wake_binding(&issuing_engine, &forged_core);
    forged_core.core = Expr::Literal(Value::Int(99));
    assert!(
        issuing_engine
            .admit_production_forward_sleep(&mut forged_core)
            .is_err(),
        "a public legacy Core mutation must reject before constructing frames"
    );
    assert!(
        calls
            .lock()
            .expect("wake call log remains available")
            .is_empty(),
        "all rejected provenance paths must fail before provider dispatch"
    );
}
