//! TASK-1822 row requirements remain metadata, not runtime authority.

use ash_core::capability::{
    CapabilityError, CapabilityProvider, ProviderAuthoringMetadata, ProviderOperationMetadata,
};
use ash_core::{Constraint, Effect, Value};
use ash_engine::Engine;
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

const CLOSED_ADMISSION_ERROR: &str =
    "checked Core/CPS admission rejected: no validated production typed lowering is available";

#[derive(Debug)]
struct CountingProvider {
    name: &'static str,
    observe_calls: Arc<AtomicUsize>,
    execute_calls: Arc<AtomicUsize>,
}

#[async_trait]
impl CapabilityProvider for CountingProvider {
    fn name(&self) -> &str {
        self.name
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    fn provider_metadata(&self) -> ProviderAuthoringMetadata {
        ProviderAuthoringMetadata::new(self.name()).with_operation(
            ProviderOperationMetadata::new("read", self.effect())
                .with_required_row("hostfs.read")
                .with_sandbox_policy("host.hostfs.read")
                .with_provenance_policy("host.hostfs.read.redacted"),
        )
    }

    async fn observe(&self, _constraints: &[Constraint]) -> Result<Value, CapabilityError> {
        self.observe_calls.fetch_add(1, Ordering::SeqCst);
        Ok(Value::String("observed".into()))
    }

    async fn execute(&self, _action_name: &str, _args: &[Value]) -> Result<Value, CapabilityError> {
        self.execute_calls.fetch_add(1, Ordering::SeqCst);
        Ok(Value::String("executed".into()))
    }
}

const fn row_authority_source() -> &'static str {
    r#"
fn guarded(path: String) -> String where row {
    hostfs.read,
    resource vault write,
    process spawn,
    fail HostFailure,
    evidence signed,
    group handler
} { path }

fn main() -> String { "ok" }
"#
}

fn write(path: &Path, source: &str) {
    std::fs::write(path, source)
        .unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
}

#[test]
fn row_requirements_do_not_register_providers_resources_or_runtime_modules() {
    let engine = Engine::new().build().expect("engine builds");

    let application = engine
        .parse(row_authority_source())
        .expect("row-bearing source parses");

    assert!(
        application.core_callable_types.contains_key("guarded"),
        "test fixture must exercise Core callable row metadata"
    );
    assert!(
        !engine.has_provider("hostfs"),
        "operation rows must not create provider authority"
    );
    assert_eq!(
        engine.resource_initializer_selection_count(),
        0,
        "resource rows must not create resource initializer ownership"
    );
    assert_eq!(
        engine.capability_implementation_selection_count(),
        0,
        "operation rows must not create implementation authority selections"
    );
    assert!(
        !engine.has_registered_runtime_module("handler"),
        "group/handler-looking rows must not register runtime modules"
    );
}

#[test]
fn imported_row_requirements_do_not_register_providers_resources_or_runtime_modules() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();
    let library = dir.join("library.ash");
    let caller = dir.join("caller.ash");
    write(
        &library,
        &row_authority_source().replace("fn guarded", "pub fn guarded"),
    );
    write(
        &caller,
        "use library::{guarded}\nfn main() -> String { \"ok\" }\n",
    );

    let engine = Engine::new().build().expect("engine builds");
    let application = engine
        .parse_file(&caller)
        .expect("caller importing row-bearing callable parses");

    assert!(
        application.core_callable_types.contains_key("guarded"),
        "import path must exercise Core callable row metadata"
    );
    assert!(
        application
            .callable_row_requirements
            .contains_key("guarded"),
        "import path must carry row requirements as metadata"
    );
    assert!(
        !engine.has_provider("hostfs"),
        "imported operation rows must not create provider authority"
    );
    assert_eq!(
        engine.resource_initializer_selection_count(),
        0,
        "imported resource rows must not create resource initializer ownership"
    );
    assert_eq!(
        engine.capability_implementation_selection_count(),
        0,
        "imported operation rows must not create implementation authority selections"
    );
    assert!(
        !engine.has_registered_runtime_module("handler"),
        "imported group/handler-looking rows must not register runtime modules"
    );
}

#[tokio::test]
async fn row_requirements_do_not_call_host_hooks_during_parse_check_or_closed_execution() {
    let observe_calls = Arc::new(AtomicUsize::new(0));
    let execute_calls = Arc::new(AtomicUsize::new(0));
    let provider = CountingProvider {
        name: "hostfs",
        observe_calls: Arc::clone(&observe_calls),
        execute_calls: Arc::clone(&execute_calls),
    };
    let engine = Engine::new()
        .with_custom_provider("hostfs", Arc::new(provider))
        .build()
        .expect("engine builds");

    let mut application = engine
        .parse(row_authority_source())
        .expect("row-bearing source parses");
    engine
        .check(&mut application)
        .expect("row-bearing source checks without invoking provider");
    let error = engine.execute(&application).await.expect_err(
        "generic source execution must reject before row metadata can dispatch host hooks",
    );
    assert!(
        matches!(
            error,
            ash_runtime::ExecError::ExecutionFailed(message) if message == CLOSED_ADMISSION_ERROR
        ),
        "row-bearing source must expose the exact checked Core/CPS closed-admission error"
    );
    assert_eq!(
        observe_calls.load(Ordering::SeqCst),
        0,
        "operation rows must not call host observe hooks"
    );
    assert_eq!(
        execute_calls.load(Ordering::SeqCst),
        0,
        "operation rows must not call host execute hooks"
    );
}

#[tokio::test]
async fn imported_row_requirements_do_not_call_host_hooks_during_parse_check_or_closed_execution() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();
    let library = dir.join("library.ash");
    let caller = dir.join("caller.ash");
    write(
        &library,
        &row_authority_source().replace("fn guarded", "pub fn guarded"),
    );
    write(
        &caller,
        "use library::{guarded}\nfn main() -> String { \"ok\" }\n",
    );

    let observe_calls = Arc::new(AtomicUsize::new(0));
    let execute_calls = Arc::new(AtomicUsize::new(0));
    let provider = CountingProvider {
        name: "hostfs",
        observe_calls: Arc::clone(&observe_calls),
        execute_calls: Arc::clone(&execute_calls),
    };
    let engine = Engine::new()
        .with_custom_provider("hostfs", Arc::new(provider))
        .build()
        .expect("engine builds");

    let mut application = engine
        .parse_file(&caller)
        .expect("caller importing row-bearing callable parses");
    engine
        .check(&mut application)
        .expect("imported row-bearing callable checks without invoking provider");
    let error = engine
        .execute(&application)
        .await
        .expect_err("generic imported source execution must reject before host hooks can run");
    assert!(
        matches!(
            error,
            ash_runtime::ExecError::ExecutionFailed(message) if message == CLOSED_ADMISSION_ERROR
        ),
        "imported row-bearing source must expose the exact checked Core/CPS closed-admission error"
    );
    assert_eq!(
        observe_calls.load(Ordering::SeqCst),
        0,
        "imported operation rows must not call host observe hooks"
    );
    assert_eq!(
        execute_calls.load(Ordering::SeqCst),
        0,
        "imported operation rows must not call host execute hooks"
    );
}
