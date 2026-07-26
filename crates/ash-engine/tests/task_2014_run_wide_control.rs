//! TASK-2014 RED coverage for one Engine-created cooperative run-control
//! envelope at a real production `time::sleep` provider await.

use ash_core::{
    Constraint, Effect, Value,
    capability::{
        CapabilityError, CapabilityProvider, ProviderAuthoringMetadata, ProviderOperationMetadata,
    },
};
use ash_engine::{Engine, ProductionCheckedCpsOutcome};
use async_trait::async_trait;
use std::{
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll},
    time::Duration,
};
use tokio::sync::Notify;

const SLEEP: &str = "fn main() -> Null { time::sleep(1) }";

#[derive(Debug)]
struct PendingTimeProvider {
    calls: Arc<AtomicUsize>,
    dropped_awaits: Arc<AtomicUsize>,
    started: Arc<Notify>,
    completes_immediately: bool,
}

impl PendingTimeProvider {
    fn new() -> Self {
        Self::with_completion(false)
    }

    fn immediate() -> Self {
        Self::with_completion(true)
    }

    fn with_completion(completes_immediately: bool) -> Self {
        Self {
            calls: Arc::new(AtomicUsize::new(0)),
            dropped_awaits: Arc::new(AtomicUsize::new(0)),
            started: Arc::new(Notify::new()),
            completes_immediately,
        }
    }

    fn metadata() -> ProviderAuthoringMetadata {
        ProviderAuthoringMetadata::new("time").with_operation(
            ProviderOperationMetadata::new("sleep", Effect::Operational)
                .with_required_row("time.sleep")
                .with_sandbox_policy("host.time.sleep.test")
                .with_provenance_policy("host.time.sleep.test.redacted"),
        )
    }
}

#[derive(Debug)]
struct PendingProviderAwait {
    dropped_awaits: Arc<AtomicUsize>,
}

impl Future for PendingProviderAwait {
    type Output = Result<Value, CapabilityError>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}

impl Drop for PendingProviderAwait {
    fn drop(&mut self) {
        self.dropped_awaits.fetch_add(1, Ordering::SeqCst);
    }
}

#[async_trait]
impl CapabilityProvider for PendingTimeProvider {
    fn name(&self) -> &'static str {
        "time"
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    fn provider_metadata(&self) -> ProviderAuthoringMetadata {
        Self::metadata()
    }

    async fn observe(&self, _constraints: &[Constraint]) -> Result<Value, CapabilityError> {
        Err(CapabilityError::NotAvailable(
            "the TASK-2014 pending provider exposes sleep only".to_string(),
        ))
    }

    async fn execute(&self, action: &str, args: &[Value]) -> Result<Value, CapabilityError> {
        assert_eq!(action, "sleep", "the sealed operation chooses time.sleep");
        assert_eq!(
            args,
            [Value::Int(1)],
            "checked CPS preserves the literal argument"
        );
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.completes_immediately {
            return Ok(Value::Null);
        }
        self.started.notify_one();
        PendingProviderAwait {
            dropped_awaits: Arc::clone(&self.dropped_awaits),
        }
        .await
    }
}

fn engine_with_pending_time_provider(provider: Arc<PendingTimeProvider>) -> Engine {
    Engine::new()
        .with_custom_provider("time", provider)
        .build()
        .expect("engine builds with the test-only time provider")
}

fn register_pending_time_binding(engine: &Engine) {
    engine
        .register_time_sleep_provider_binding()
        .expect("the Engine validates the registry-resolved time.sleep provider before admission");
}

async fn wait_until_provider_is_awaited<F>(
    execution: &mut Pin<Box<F>>,
    provider: &PendingTimeProvider,
) where
    F: Future,
{
    tokio::select! {
        _ = &mut *execution => panic!("pending provider execution completed before a control decision"),
        () = provider.started.notified() => {}
    }
}

#[tokio::test(start_paused = true)]
async fn deadline_while_awaiting_the_sealed_provider_returns_timeout_and_drops_the_await() {
    let provider = Arc::new(PendingTimeProvider::new());
    let engine = engine_with_pending_time_provider(Arc::clone(&provider));
    register_pending_time_binding(&engine);
    let mut entry = engine.parse(SLEEP).expect("fixture parses");
    let admission = engine
        .admit_production_checked_cps(&mut entry)
        .expect("the exact checked source program mints the opaque token");

    // The absolute deadline is created after admission and is not supplied by
    // the provider, row, or opaque token.
    let (control, _cancellation) = engine
        .new_production_run_control(&admission, Some(Duration::from_millis(1)))
        .expect("only the issuing Engine creates post-admission run control");
    let mut execution = Box::pin(engine.execute_production_checked_cps(&admission, control));
    wait_until_provider_is_awaited(&mut execution, &provider).await;

    tokio::time::advance(Duration::from_millis(1)).await;
    let outcome = execution
        .await
        .expect("control is an execution result, not an admission failure");

    assert!(matches!(outcome, ProductionCheckedCpsOutcome::TimedOut));
    assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        provider.dropped_awaits.load(Ordering::SeqCst),
        1,
        "timeout drops the in-flight host future rather than evaluating a later CPS reduction"
    );
}

#[tokio::test(start_paused = true)]
async fn cancellation_wins_over_an_expired_deadline_and_drops_the_provider_await() {
    let provider = Arc::new(PendingTimeProvider::new());
    let engine = engine_with_pending_time_provider(Arc::clone(&provider));
    register_pending_time_binding(&engine);
    let mut entry = engine.parse(SLEEP).expect("fixture parses");
    let admission = engine
        .admit_production_checked_cps(&mut entry)
        .expect("the exact checked source program mints the opaque token");

    let (control, cancellation) = engine
        .new_production_run_control(&admission, Some(Duration::from_millis(1)))
        .expect("only the issuing Engine creates post-admission run control");
    let mut execution = Box::pin(engine.execute_production_checked_cps(&admission, control));
    wait_until_provider_is_awaited(&mut execution, &provider).await;

    // Make cancellation and deadline expiry observable before allowing the
    // driver to decide. The specified priority is cancellation > timeout >
    // completion, and no continuation may run after that decision.
    cancellation.cancel();
    tokio::time::advance(Duration::from_millis(1)).await;
    let outcome = execution
        .await
        .expect("control is an execution result, not an admission failure");

    assert!(matches!(outcome, ProductionCheckedCpsOutcome::Cancelled));
    assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        provider.dropped_awaits.load(Ordering::SeqCst),
        1,
        "cancellation drops the in-flight future and prevents a later CPS reduction"
    );
}

#[tokio::test(start_paused = true)]
async fn control_for_one_admission_cannot_execute_a_later_same_engine_admission() {
    // Complete a wrongly dispatched call immediately, so an implementation
    // that neglects admission identity fails the assertion instead of hanging.
    let provider = Arc::new(PendingTimeProvider::immediate());
    let engine = engine_with_pending_time_provider(Arc::clone(&provider));
    register_pending_time_binding(&engine);

    let mut first_entry = engine.parse(SLEEP).expect("first fixture parses");
    let first_admission = engine
        .admit_production_checked_cps(&mut first_entry)
        .expect("first checked source program mints the opaque token");
    let (control, _cancellation) = engine
        .new_production_run_control(&first_admission, None)
        .expect("the Engine creates control only for the first admitted token");

    // Admission B occurs after A's control creation. Sharing an Engine issuer
    // is insufficient: the control is bound to A's exact sealed admission.
    let mut second_entry = engine.parse(SLEEP).expect("second fixture parses");
    let second_admission = engine
        .admit_production_checked_cps(&mut second_entry)
        .expect("second checked source program mints a distinct opaque token");
    let result = engine
        .execute_production_checked_cps(&second_admission, control)
        .await;

    assert!(
        result.is_err(),
        "a control created for admission A must reject admission B before execution"
    );
    assert_eq!(
        provider.calls.load(Ordering::SeqCst),
        0,
        "a mismatched control must reject before provider dispatch"
    );
}
