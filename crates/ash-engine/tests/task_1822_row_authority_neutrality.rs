//! TASK-1822 row requirements remain metadata, not runtime authority.

use ash_core::capability::{
    CapabilityError, CapabilityProvider, ProviderAuthoringMetadata, ProviderOperationMetadata,
};
use ash_core::runtime::{ApplicationFailureKind, ApplicationReportStatus};
use ash_core::{Constraint, Effect, Role, Value};
use ash_engine::{
    ApplicationAdmissionOutcome, ApplicationAdmissionRequest, ApplicationContractRequirement,
    Engine,
};
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

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
    role tenant.admin,
    policy deployment.approve,
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

fn admitted_role(name: &str) -> Role {
    Role {
        name: name.to_string(),
        authority: vec![],
        obligations: vec![],
    }
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
async fn row_roles_and_capabilities_do_not_satisfy_application_admission() {
    let engine = Engine::new().build().expect("engine builds");
    let application = engine
        .parse(row_authority_source())
        .expect("row-bearing source parses");

    let role_outcome = engine
        .admit_application(ApplicationAdmissionRequest {
            entry_name: "row_role_neutrality".into(),
            body: application.core.clone(),
            application_id: None,
            run_id: None,
            active_role: None,
            admitted_role: None,
            required_capabilities: vec![],
            requires: vec![ApplicationContractRequirement::Role("tenant.admin".into())],
            ensures: vec![],
        })
        .await;

    match role_outcome {
        ApplicationAdmissionOutcome::Rejected { failure, report } => {
            assert_eq!(failure.kind, ApplicationFailureKind::RoleAdmissionFailure);
            assert_eq!(report.status, ApplicationReportStatus::Failed);
            assert_eq!(report.admission.active_role, None);
            assert!(report.admission.admitted_capabilities.is_empty());
        }
        ApplicationAdmissionOutcome::Admitted { .. } => {
            panic!("role row requirement must not admit role authority")
        }
    }

    let capability_outcome = engine
        .admit_application(ApplicationAdmissionRequest {
            entry_name: "row_operation_neutrality".into(),
            body: application.core.clone(),
            application_id: None,
            run_id: None,
            active_role: Some("tenant.admin".into()),
            admitted_role: Some(admitted_role("tenant.admin")),
            required_capabilities: vec![],
            requires: vec![ApplicationContractRequirement::Capability(
                "hostfs.read".into(),
            )],
            ensures: vec![],
        })
        .await;

    match capability_outcome {
        ApplicationAdmissionOutcome::Rejected { failure, report } => {
            assert_eq!(
                failure.kind,
                ApplicationFailureKind::CapabilityAdmissionFailure
            );
            assert_eq!(report.status, ApplicationReportStatus::Failed);
            assert!(
                report.admission.admitted_capability_bindings.is_empty(),
                "operation rows must not admit RuntimeKernel capability bindings"
            );
        }
        ApplicationAdmissionOutcome::Admitted { .. } => {
            panic!("operation row requirement must not admit capability authority")
        }
    }
}

#[tokio::test]
async fn row_requirements_do_not_call_host_hooks_during_parse_check_or_execute() {
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
    let value = engine
        .execute(&application)
        .await
        .expect("application body executes independently of row metadata");

    assert_eq!(value, Value::String("ok".into()));
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
async fn imported_row_requirements_do_not_call_host_hooks_during_parse_check_or_execute() {
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
    let value = engine
        .execute(&application)
        .await
        .expect("application body executes independently of imported row metadata");

    assert_eq!(value, Value::String("ok".into()));
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
