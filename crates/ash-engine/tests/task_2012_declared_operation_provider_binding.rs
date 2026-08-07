//! TASK-2012 RED coverage for typed declared-operation/provider bindings.

use ash_core::capability::{
    CapabilityError, CapabilityProvider, ProviderAuthoringMetadata, ProviderOperationMetadata,
};
use ash_core::{
    Effect, Value,
    core_ash::{CoreRowItem, CoreType},
};
use ash_engine::{ApplicationAdmissionOutcome, ApplicationAdmissionRequest, Engine};
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

const DECLARED_CLOCK_SOURCE: &str = r"
interface Clock<T> { sleep(Int) -> Null }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = null }
fn main() -> Null { TestClock::sleep(0) }
";
const CLOSED_ADMISSION_ERROR: &str =
    "checked Core/CPS admission rejected: no validated production typed lowering is available";

#[derive(Debug)]
struct CountingClockProvider {
    operation: &'static str,
    required_row: &'static str,
    executions: Arc<AtomicUsize>,
}

#[async_trait]
impl CapabilityProvider for CountingClockProvider {
    fn name(&self) -> &'static str {
        "clock-host"
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    fn provider_metadata(&self) -> ProviderAuthoringMetadata {
        ProviderAuthoringMetadata::new(self.name()).with_operation(
            ProviderOperationMetadata::new(self.operation, Effect::Operational)
                .with_required_row(self.required_row)
                .with_sandbox_policy("host.clock.sleep")
                .with_provenance_policy("host.clock.sleep.redacted"),
        )
    }

    async fn observe(
        &self,
        _constraints: &[ash_core::Constraint],
    ) -> Result<Value, CapabilityError> {
        Err(CapabilityError::NotAvailable(
            "clock observation is not supported".to_string(),
        ))
    }

    async fn execute(&self, action: &str, _args: &[Value]) -> Result<Value, CapabilityError> {
        if action != self.operation {
            return Err(CapabilityError::NotAvailable(format!(
                "unexpected clock action '{action}'"
            )));
        }
        self.executions.fetch_add(1, Ordering::SeqCst);
        Ok(Value::Null)
    }
}

fn checked_declared_clock_entry(engine: &Engine) -> ash_engine::Entry {
    checked_declared_clock_entry_from_source(engine, DECLARED_CLOCK_SOURCE)
}

fn checked_declared_clock_entry_from_source(engine: &Engine, source: &str) -> ash_engine::Entry {
    let mut entry = engine.parse(source).expect("declared clock fixture parses");
    engine
        .check(&mut entry)
        .expect("declared clock operation resolves before binding registration");
    entry
}

#[test]
fn task_2015_rechecking_declared_operation_does_not_duplicate_its_requirement_row() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(DECLARED_CLOCK_SOURCE)
        .expect("declared clock fixture parses");

    engine
        .check(&mut entry)
        .expect("initial declared clock operation check succeeds");
    engine
        .check(&mut entry)
        .expect("repeat declared clock operation check succeeds");

    let CoreType::Function { row, .. } = entry
        .core_callable_types
        .get("main")
        .expect("checked entry has a Core callable type")
    else {
        panic!("main must retain a Core function type");
    };
    let matching_rows = row
        .items
        .iter()
        .filter(|item| {
            matches!(
                item,
                CoreRowItem::Operation { path, operation }
                    if path == &["TestClock".to_string()] && operation == "sleep"
            )
        })
        .count();
    assert_eq!(
        matching_rows, 1,
        "a checked declaration-resolved operation must contribute exactly one requirement row: {row:?}"
    );
}

#[derive(Debug)]
struct RecordingClockProvider {
    received_arguments: Arc<Mutex<Vec<Vec<Value>>>>,
}

#[async_trait]
impl CapabilityProvider for RecordingClockProvider {
    fn name(&self) -> &'static str {
        "clock-host"
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    fn provider_metadata(&self) -> ProviderAuthoringMetadata {
        ProviderAuthoringMetadata::new(self.name()).with_operation(
            ProviderOperationMetadata::new("sleep", Effect::Operational)
                .with_required_row("TestClock.sleep")
                .with_sandbox_policy("host.clock.sleep")
                .with_provenance_policy("host.clock.sleep.redacted"),
        )
    }

    async fn observe(
        &self,
        _constraints: &[ash_core::Constraint],
    ) -> Result<Value, CapabilityError> {
        Err(CapabilityError::NotAvailable(
            "clock observation is not supported".to_string(),
        ))
    }

    async fn execute(&self, action: &str, args: &[Value]) -> Result<Value, CapabilityError> {
        if action != "sleep" {
            return Err(CapabilityError::NotAvailable(format!(
                "unexpected clock action '{action}'"
            )));
        }
        self.received_arguments
            .lock()
            .expect("recording clock provider mutex is not poisoned")
            .push(args.to_vec());
        Ok(Value::Null)
    }
}

fn request(entry: &ash_engine::Entry) -> ApplicationAdmissionRequest {
    ApplicationAdmissionRequest {
        entry_name: "main".to_string(),
        body: entry.core.clone(),
        application_id: None,
        run_id: None,
        required_capabilities: Vec::new(),
        requires: Vec::new(),
        ensures: Vec::new(),
    }
}

fn register_sleep_binding(engine: &Engine, entry: &ash_engine::Entry) {
    engine
        .register_declared_operation_provider_binding(
            entry
                .declared_concrete_operation
                .as_ref()
                .expect("checked entry retains its resolved declared operation"),
            "clock-host",
            "sleep",
        )
        .expect("exact declared operation/provider binding registers");
}

#[tokio::test]
async fn task_2012_exact_declared_binding_admits_but_generic_execution_stays_closed() {
    let executions = Arc::new(AtomicUsize::new(0));
    let engine = Engine::new()
        .with_custom_provider(
            "clock-host",
            Arc::new(CountingClockProvider {
                operation: "sleep",
                required_row: "TestClock.sleep",
                executions: Arc::clone(&executions),
            }),
        )
        .build()
        .expect("engine builds");
    let entry = checked_declared_clock_entry(&engine);
    register_sleep_binding(&engine, &entry);

    let error = engine
        .execute(&entry)
        .await
        .expect_err("generic source execution must not dispatch the bound declared operation");
    assert!(
        matches!(
            error,
            ash_runtime::ExecError::ExecutionFailed(message) if message == CLOSED_ADMISSION_ERROR
        ),
        "bound declared operations must expose the exact checked Core/CPS closed-admission error"
    );
    assert_eq!(
        executions.load(Ordering::SeqCst),
        0,
        "closed generic execution must not call the bound provider"
    );
}

#[tokio::test]
async fn task_2012_declared_binding_preserves_typed_local_argument_without_provider_dispatch() {
    let received_arguments = Arc::new(Mutex::new(Vec::new()));
    let engine = Engine::new()
        .with_custom_provider(
            "clock-host",
            Arc::new(RecordingClockProvider {
                received_arguments: Arc::clone(&received_arguments),
            }),
        )
        .build()
        .expect("engine builds");
    let entry = checked_declared_clock_entry_from_source(
        &engine,
        r"
interface Clock<T> { sleep(Int) -> Null }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = null }
fn main() -> Null {
    let delay = 0;
    TestClock::sleep(delay)
}
",
    );
    let operation = entry
        .declared_concrete_operation
        .as_ref()
        .expect("a typed local argument retains the resolved concrete operation");
    assert_eq!(operation.impl_type, "TestClock");
    assert_eq!(operation.operation, "sleep");
    let ash_core::core_ash::CoreType::Function { row, .. } = entry
        .core_callable_types
        .get("main")
        .expect("checked entry has a Core callable type")
    else {
        panic!("main must retain a Core function type");
    };
    assert!(row.items.iter().any(|item| matches!(
        item,
        ash_core::core_ash::CoreRowItem::Operation { path, operation }
            if path == &["TestClock".to_string()] && operation == "sleep"
    )));
    register_sleep_binding(&engine, &entry);

    let error = engine
        .execute(&entry)
        .await
        .expect_err("generic source execution must reject before provider argument dispatch");
    assert!(
        matches!(
            error,
            ash_runtime::ExecError::ExecutionFailed(message) if message == CLOSED_ADMISSION_ERROR
        ),
        "typed-local declared operations must expose the exact checked Core/CPS closed-admission error"
    );
    assert!(
        received_arguments
            .lock()
            .expect("recorded arguments remain available")
            .is_empty(),
        "closed generic execution must not dispatch a typed local argument to the provider"
    );
}

#[test]
fn task_2012_declared_binding_rejects_non_int_typed_local_argument_during_checking() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(
            r#"
interface Clock<T> { sleep(Int) -> Null }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = null }
fn main() -> Null {
    let delay = "zero";
    TestClock::sleep(delay)
}
"#,
        )
        .expect("typed local argument fixture parses");

    let error = engine
        .check(&mut entry)
        .expect_err("a non-Int local argument must fail before admission or dispatch");
    assert!(
        error
            .to_string()
            .contains("TestClock::sleep: argument type mismatch"),
        "unexpected typed-local argument mismatch: {error}"
    );
}

#[tokio::test]
async fn task_2012_unbound_declared_row_rejects_despite_an_unrelated_provider() {
    let engine = Engine::new()
        .with_custom_provider(
            "unrelated-clock",
            Arc::new(CountingClockProvider {
                operation: "sleep",
                required_row: "TestClock.sleep",
                executions: Arc::new(AtomicUsize::new(0)),
            }),
        )
        .build()
        .expect("engine builds");
    let entry = checked_declared_clock_entry(&engine);

    let ApplicationAdmissionOutcome::Rejected { failure, .. } = engine
        .admit_application_with_explicit_rows(request(&entry), &entry)
        .await
    else {
        panic!("a row without an explicit binding must not select a provider");
    };
    assert!(
        failure
            .evidence
            .notes
            .iter()
            .any(|note| note.contains("missing declared-operation binding")),
        "unexpected missing-binding diagnostic: {failure:?}"
    );
}

#[test]
fn task_2012_provider_operation_mismatch_is_rejected_at_binding_registration() {
    let engine = Engine::new()
        .with_custom_provider(
            "clock-host",
            Arc::new(CountingClockProvider {
                operation: "wake",
                required_row: "TestClock.sleep",
                executions: Arc::new(AtomicUsize::new(0)),
            }),
        )
        .build()
        .expect("engine builds");
    let entry = checked_declared_clock_entry(&engine);

    let error = engine
        .register_declared_operation_provider_binding(
            entry
                .declared_concrete_operation
                .as_ref()
                .expect("checked entry retains its resolved declared operation"),
            "clock-host",
            "sleep",
        )
        .expect_err("provider metadata must declare the selected operation");
    assert!(
        error.to_string().contains("provider operation 'sleep'"),
        "unexpected provider-operation mismatch: {error}"
    );
}

#[test]
fn task_2012_provider_row_mismatch_is_rejected_at_binding_registration() {
    let engine = Engine::new()
        .with_custom_provider(
            "clock-host",
            Arc::new(CountingClockProvider {
                operation: "sleep",
                required_row: "OtherClock.sleep",
                executions: Arc::new(AtomicUsize::new(0)),
            }),
        )
        .build()
        .expect("engine builds");
    let entry = checked_declared_clock_entry(&engine);

    let error = engine
        .register_declared_operation_provider_binding(
            entry
                .declared_concrete_operation
                .as_ref()
                .expect("checked entry retains its resolved declared operation"),
            "clock-host",
            "sleep",
        )
        .expect_err("provider row metadata must match the declared operation identity exactly");
    assert!(
        error.to_string().contains("TestClock.sleep"),
        "unexpected provider-row mismatch: {error}"
    );
}

#[test]
fn task_2012_direct_source_invoke_stays_rejected_after_binding_work() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse("fn main() -> Unit { invoke(\"clock-host\", \"sleep\", [0]) }")
        .expect("direct invoke fixture parses");

    let error = engine
        .check(&mut entry)
        .expect_err("direct source invoke remains rejected");
    assert!(
        error
            .to_string()
            .contains("direct source invoke is not admitted"),
        "unexpected invoke rejection: {error}"
    );
}
