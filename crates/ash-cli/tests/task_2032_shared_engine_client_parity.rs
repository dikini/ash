//! TASK-2032 RED parity contract for the in-process CLI and daemon adapters.
//!
//! The adapters named here do not exist yet. When implemented, neither one
//! may parse, lower, route by source/handler spelling, construct a frame, or
//! call a direct evaluator. They submit the exact same opaque Engine request
//! and return only the normalized terminal envelope. This test does not claim
//! the daemon service transport accepts every source fixture or exposes V1
//! terminal envelopes.

use ash_cli::commands::{
    daemon::submit_admitted_program as submit_daemon_admitted_program,
    run::submit_admitted_program as submit_cli_admitted_program,
};
use ash_core::{
    Effect, Value,
    capability::{
        CapabilityError, CapabilityProvider, ProviderAuthoringMetadata, ProviderOperationMetadata,
    },
};
use ash_engine::{CanonicalTerminalEnvelopeV1, Engine, standard_profiles::StandardProviderProfile};
use async_trait::async_trait;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;

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

#[derive(Debug)]
struct WakeProvider {
    execution_count: Arc<AtomicUsize>,
}

#[async_trait]
impl CapabilityProvider for WakeProvider {
    fn name(&self) -> &'static str {
        "task-2032-forward-wake"
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    fn provider_metadata(&self) -> ProviderAuthoringMetadata {
        ProviderAuthoringMetadata::new(self.name()).with_operation(
            ProviderOperationMetadata::new("wake", Effect::Operational)
                .with_required_row("TestClock.wake")
                .with_sandbox_policy("task-2032.forward.wake")
                .with_provenance_policy("task-2032.forward.wake.redacted"),
        )
    }

    async fn observe(
        &self,
        _constraints: &[ash_core::Constraint],
    ) -> Result<Value, CapabilityError> {
        Err(CapabilityError::NotAvailable(
            "the TASK-2032 forward parity provider exposes wake only".to_string(),
        ))
    }

    async fn execute(&self, action: &str, args: &[Value]) -> Result<Value, CapabilityError> {
        assert_eq!(action, "wake");
        assert_eq!(args, [Value::Int(0)]);
        self.execution_count.fetch_add(1, Ordering::Relaxed);
        Ok(Value::Int(73))
    }
}

async fn install_time_sleep_profile(engine: &Engine) {
    engine
        .install_standard_profile(StandardProviderProfile::application_default(
            "task-2032-client-parity",
            std::iter::empty::<&std::path::Path>(),
            std::iter::empty::<&str>(),
        ))
        .await
        .expect("the shared-parity fixture installs its application provider profile");
    engine
        .register_time_sleep_provider_binding()
        .expect("the Engine seals the time::sleep provider binding before admission");
}

#[tokio::test]
async fn client_adapters_normalize_the_same_admitted_request_equally() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse("fn main() -> Int { 42 }")
        .expect("selected shared-seam return fixture parses");
    let program = engine
        .admit_program(&mut entry)
        .expect("only Engine admission creates the opaque shared-client artifact");
    let (request, _cancellation) = engine
        .new_admitted_program_request(&program, None)
        .expect("one issuing Engine creates the request shared by both clients");

    let cli_terminal = submit_cli_admitted_program(&engine, &request)
        .await
        .expect("the CLI adapter submits the Engine-issued request");
    let daemon_terminal = submit_daemon_admitted_program(&engine, &request)
        .await
        .expect("the daemon adapter submits the same Engine-issued request");

    assert_eq!(
        cli_terminal, daemon_terminal,
        "the normalized terminal envelope must be independent of client formatting and lifecycle"
    );
    assert_eq!(
        cli_terminal,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(42)),
        "both clients preserve the Engine-owned canonical terminal projection"
    );
}

#[tokio::test(start_paused = true)]
async fn client_adapters_reuse_the_same_timeout_request_with_a_fresh_submission_deadline() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse("fn main() -> Int { 42 }")
        .expect("selected shared-seam return fixture parses");
    let program = engine
        .admit_program(&mut entry)
        .expect("the Engine admits the opaque reusable pure artifact");
    let (request, _cancellation) = engine
        .new_admitted_program_request(&program, Some(Duration::from_secs(1)))
        .expect("one issuing Engine creates the reusable timed request");

    let cli_terminal = submit_cli_admitted_program(&engine, &request)
        .await
        .expect("the CLI adapter submits the first timed request");
    assert_eq!(
        cli_terminal,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(42)),
    );

    tokio::time::advance(Duration::from_secs(1)).await;

    let daemon_terminal = submit_daemon_admitted_program(&engine, &request)
        .await
        .expect("the daemon adapter submits the same reusable timed request");
    assert_eq!(
        daemon_terminal,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(42)),
        "a reusable request binds its deadline to each client submission, not request construction",
    );
}

#[tokio::test]
async fn client_adapters_normalize_the_same_admitted_timeout_equally() {
    let engine = Engine::new().build().expect("engine builds");
    install_time_sleep_profile(&engine).await;
    let mut entry = engine
        .parse(SLEEP_SOURCE)
        .expect("selected shared-seam timeout fixture parses");
    let program = engine
        .admit_program(&mut entry)
        .expect("the Engine admits the sealed time::sleep artifact once");
    let (request, _cancellation) = engine
        .new_admitted_program_request(&program, Some(Duration::ZERO))
        .expect("one Engine-issued request binds the shared zero deadline");

    let cli_terminal = submit_cli_admitted_program(&engine, &request)
        .await
        .expect("the CLI adapter submits the Engine-owned timeout request");
    let daemon_terminal = submit_daemon_admitted_program(&engine, &request)
        .await
        .expect("the daemon adapter submits the identical timeout request");

    assert_eq!(cli_terminal, daemon_terminal);
    assert_eq!(cli_terminal, CanonicalTerminalEnvelopeV1::timed_out());
}

#[tokio::test]
async fn client_adapters_normalize_the_same_admitted_cancellation_equally() {
    let engine = Engine::new().build().expect("engine builds");
    install_time_sleep_profile(&engine).await;
    let mut entry = engine
        .parse(SLEEP_SOURCE)
        .expect("selected shared-seam cancellation fixture parses");
    let program = engine
        .admit_program(&mut entry)
        .expect("the Engine admits the sealed time::sleep artifact once");
    let (request, cancellation) = engine
        .new_admitted_program_request(&program, None)
        .expect("one Engine-issued request binds the shared cancellation control");
    cancellation.cancel();

    let cli_terminal = submit_cli_admitted_program(&engine, &request)
        .await
        .expect("the CLI adapter submits the Engine-owned cancelled request");
    let daemon_terminal = submit_daemon_admitted_program(&engine, &request)
        .await
        .expect("the daemon adapter submits the identical cancelled request");

    assert_eq!(cli_terminal, daemon_terminal);
    assert_eq!(cli_terminal, CanonicalTerminalEnvelopeV1::cancelled());
}

#[tokio::test]
async fn client_adapters_normalize_the_same_admitted_handler_trap_equally() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(TRAP_SLEEP_SOURCE)
        .expect("selected shared-seam handler-trap fixture parses");
    let program = engine
        .admit_program(&mut entry)
        .expect("the Engine admits the sealed handler trap once");
    let (request, _cancellation) = engine
        .new_admitted_program_request(&program, None)
        .expect("one Engine-issued request preserves handler authority");

    let cli_terminal = submit_cli_admitted_program(&engine, &request)
        .await
        .expect("the CLI adapter submits the handler request");
    let daemon_terminal = submit_daemon_admitted_program(&engine, &request)
        .await
        .expect("the daemon adapter submits the identical handler request");

    assert_eq!(cli_terminal, daemon_terminal);
    assert_eq!(
        cli_terminal,
        CanonicalTerminalEnvelopeV1::trapped("division by zero")
    );
}

#[tokio::test]
async fn client_adapters_normalize_the_same_admitted_deep_handler_equally() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(DEEP_AFFINE_CLOCK_SOURCE)
        .expect("selected shared-seam deep-handler fixture parses");
    let program = engine
        .admit_program(&mut entry)
        .expect("the Engine admits the sealed deep handler once");
    let (request, _cancellation) = engine
        .new_admitted_program_request(&program, None)
        .expect("one Engine-issued request preserves deep-frame authority");

    let cli_terminal = submit_cli_admitted_program(&engine, &request)
        .await
        .expect("the CLI adapter submits the deep-handler request");
    let daemon_terminal = submit_daemon_admitted_program(&engine, &request)
        .await
        .expect("the daemon adapter submits the identical deep-handler request");

    assert_eq!(cli_terminal, daemon_terminal);
    assert_eq!(
        cli_terminal,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(107))
    );
}

#[tokio::test]
async fn client_adapters_normalize_the_same_admitted_forward_handler_equally() {
    let execution_count = Arc::new(AtomicUsize::new(0));
    let engine = Engine::new()
        .with_custom_provider(
            "task-2032-forward-wake",
            Arc::new(WakeProvider {
                execution_count: Arc::clone(&execution_count),
            }),
        )
        .build()
        .expect("forward-handler parity engine builds");
    let mut entry = engine
        .parse(FORWARD_SLEEP_SOURCE)
        .expect("selected shared-seam forward-handler fixture parses");
    engine
        .check(&mut entry)
        .expect("the sealed forward-provider binding requires checked source facts");
    engine
        .register_sealed_forward_sleep_wake_provider_binding(
            &entry,
            "task-2032-forward-wake",
            "wake",
        )
        .expect("the Engine seals the forward wake provider binding before admission");
    let program = engine
        .admit_program(&mut entry)
        .expect("the Engine admits the sealed forward handler once");
    let (request, _cancellation) = engine
        .new_admitted_program_request(&program, None)
        .expect("one Engine-issued request preserves forward-frame authority");

    let cli_terminal = submit_cli_admitted_program(&engine, &request)
        .await
        .expect("the CLI adapter submits the forward-handler request");
    let daemon_terminal = submit_daemon_admitted_program(&engine, &request)
        .await
        .expect("the daemon adapter submits the identical forward-handler request");

    assert_eq!(cli_terminal, daemon_terminal);
    assert_eq!(
        cli_terminal,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(73))
    );
    assert_eq!(
        execution_count.load(Ordering::Relaxed),
        2,
        "both client submissions dispatch the sealed forward provider exactly once"
    );
}
