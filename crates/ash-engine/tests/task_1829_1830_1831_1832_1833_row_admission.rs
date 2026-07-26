//! Phase 179 explicit row admission runtime wiring tests.
//!
//! Covers TASK-1829 through TASK-1833: operation, resource, role, and policy
//! row admission checks, imported-callable parity, and authority-neutrality
//! regressions.

use ash_core::capability::{
    CapabilityError, CapabilityProvider, ProviderAuthoringMetadata, ProviderOperationMetadata,
};
use ash_core::runtime::{ApplicationFailureKind, ApplicationReportStatus};
use ash_core::{Constraint, Effect, Role, Value};
use ash_engine::row_admission::{RowAdmissionCheck, RowAdmissionRequirement};
use ash_engine::{ApplicationAdmissionOutcome, ApplicationAdmissionRequest, Engine};
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

const ROW_AUTHORITY_SOURCE: &str = r#"
fn guarded(path: String) -> String where row {
    posixfs.read,
    resource vault write,
    role tenant.admin,
    policy pii.redact,
    process spawn,
    fail HostFailure,
    evidence signed,
    group handler
} { path }

fn main() -> String { "ok" }
"#;

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
                .with_required_row("posixfs.read")
                .with_sandbox_policy("host.posixfs.read")
                .with_provenance_policy("host.posixfs.read.redacted"),
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

fn base_request(application: &ash_engine::Entry) -> ApplicationAdmissionRequest {
    ApplicationAdmissionRequest {
        entry_name: "row_admission".into(),
        body: application.core.clone(),
        application_id: None,
        run_id: None,
        active_role: None,
        admitted_role: None,
        required_capabilities: vec![],
        requires: vec![],
        ensures: vec![],
    }
}

fn application_with_inline_operation_row() -> ash_engine::Entry {
    Engine::new()
        .build()
        .expect("engine builds")
        .parse(
            "fn read(path: String) -> {posixfs.read} String { path }\nfn main() -> String { \"ok\" }\n",
        )
        .expect("application parses")
}

#[tokio::test]
async fn operation_row_rejects_when_provider_missing() {
    let engine = Engine::new().build().expect("engine builds");
    let application = application_with_inline_operation_row();
    let request = base_request(&application);

    let outcome = engine
        .admit_application_with_explicit_rows(request, &application)
        .await;

    match outcome {
        ApplicationAdmissionOutcome::Rejected { failure, report } => {
            assert_eq!(
                failure.kind,
                ApplicationFailureKind::CapabilityAdmissionFailure
            );
            assert_eq!(report.status, ApplicationReportStatus::Failed);
            assert!(
                failure
                    .evidence
                    .notes
                    .iter()
                    .any(|note| note.contains("posixfs.read")),
                "diagnostic should name the missing capability: {failure:?}"
            );
        }
        ApplicationAdmissionOutcome::Admitted { .. } => {
            panic!("operation row must reject when provider is missing")
        }
    }
}

#[tokio::test]
async fn operation_row_with_registered_provider_reaches_closed_application_admission() {
    let observe_calls = Arc::new(AtomicUsize::new(0));
    let execute_calls = Arc::new(AtomicUsize::new(0));
    let provider = CountingProvider {
        name: "posixfs",
        observe_calls: Arc::clone(&observe_calls),
        execute_calls: Arc::clone(&execute_calls),
    };
    let engine = Engine::new()
        .with_custom_provider("posixfs", Arc::new(provider))
        .build()
        .expect("engine builds");
    let application = application_with_inline_operation_row();
    let request = base_request(&application);

    assert!(engine.has_provider("posixfs"));

    let outcome = engine
        .admit_application_with_explicit_rows(request, &application)
        .await;

    match outcome {
        ApplicationAdmissionOutcome::Rejected { failure, report } => {
            assert_eq!(failure.kind, ApplicationFailureKind::AdmissionFailure);
            assert_eq!(report.status, ApplicationReportStatus::Failed);
        }
        ApplicationAdmissionOutcome::Admitted { .. } => {
            panic!("registered operation row must still use closed checked-Core/CPS admission")
        }
    }

    assert_eq!(
        observe_calls.load(Ordering::SeqCst),
        0,
        "row admission must not call host observe"
    );
    assert_eq!(
        execute_calls.load(Ordering::SeqCst),
        0,
        "row admission must not call host execute"
    );
}

fn application_with_resource_row() -> ash_engine::Entry {
    Engine::new()
        .build()
        .expect("engine builds")
        .parse(
            "fn store(key: String) -> String where row { resource vault write } { key }\nfn main() -> String { \"ok\" }\n",
        )
        .expect("application parses")
}

#[tokio::test]
async fn resource_row_rejects_when_initializer_missing() {
    let engine = Engine::new().build().expect("engine builds");
    let application = application_with_resource_row();
    let request = base_request(&application);

    let outcome = engine
        .admit_application_with_explicit_rows(request, &application)
        .await;

    match outcome {
        ApplicationAdmissionOutcome::Rejected { failure, report } => {
            assert_eq!(
                failure.kind,
                ApplicationFailureKind::CapabilityAdmissionFailure
            );
            assert_eq!(report.status, ApplicationReportStatus::Failed);
            assert!(
                failure
                    .evidence
                    .notes
                    .iter()
                    .any(|note| note.contains("vault")),
                "diagnostic should name the missing resource: {failure:?}"
            );
        }
        ApplicationAdmissionOutcome::Admitted { .. } => {
            panic!("resource row must reject when initializer is missing")
        }
    }
}

#[tokio::test]
async fn resource_row_with_selected_initializer_reaches_closed_application_admission() {
    let engine = Engine::new()
        .with_resource_initializer("vault", "memory")
        .build()
        .expect("engine builds");
    let application = application_with_resource_row();
    let request = base_request(&application);

    assert_eq!(
        engine.resource_initializer_selection("vault"),
        Some("memory")
    );

    let outcome = engine
        .admit_application_with_explicit_rows(request, &application)
        .await;

    match outcome {
        ApplicationAdmissionOutcome::Rejected { failure, report } => {
            assert_eq!(failure.kind, ApplicationFailureKind::AdmissionFailure);
            assert_eq!(report.status, ApplicationReportStatus::Failed);
        }
        ApplicationAdmissionOutcome::Admitted { .. } => {
            panic!("selected resource row must still use closed checked-Core/CPS admission")
        }
    }
}

fn application_with_role_row() -> ash_engine::Entry {
    Engine::new()
        .build()
        .expect("engine builds")
        .parse(
            "fn admin() -> String where row { role tenant.admin } { \"ok\" }\nfn main() -> String { \"ok\" }\n",
        )
        .expect("application parses")
}

#[tokio::test]
async fn role_row_rejects_when_role_missing() {
    let engine = Engine::new().build().expect("engine builds");
    let application = application_with_role_row();
    let request = base_request(&application);

    let outcome = engine
        .admit_application_with_explicit_rows(request, &application)
        .await;

    match outcome {
        ApplicationAdmissionOutcome::Rejected { failure, report } => {
            assert_eq!(failure.kind, ApplicationFailureKind::RoleAdmissionFailure);
            assert_eq!(report.status, ApplicationReportStatus::Failed);
        }
        ApplicationAdmissionOutcome::Admitted { .. } => {
            panic!("role row must reject when role is missing")
        }
    }
}

#[tokio::test]
async fn role_row_with_admitted_role_reaches_closed_application_admission() {
    let engine = Engine::new().build().expect("engine builds");
    let application = application_with_role_row();
    let mut request = base_request(&application);
    request.admitted_role = Some(admitted_role("tenant.admin"));
    request.active_role = Some("tenant.admin".into());

    let outcome = engine
        .admit_application_with_explicit_rows(request, &application)
        .await;

    match outcome {
        ApplicationAdmissionOutcome::Rejected { failure, report } => {
            assert_eq!(failure.kind, ApplicationFailureKind::AdmissionFailure);
            assert_eq!(report.status, ApplicationReportStatus::Failed);
        }
        ApplicationAdmissionOutcome::Admitted { .. } => {
            panic!("admitted role row must still use closed checked-Core/CPS admission")
        }
    }
}

fn application_with_policy_row() -> ash_engine::Entry {
    Engine::new()
        .build()
        .expect("engine builds")
        .parse(
            "fn policy_guard() -> String where row { policy pii.redact } { \"ok\" }\nfn main() -> String { \"ok\" }\n",
        )
        .expect("application parses")
}

#[tokio::test]
async fn policy_row_fails_closed_as_unsupported() {
    let engine = Engine::new().build().expect("engine builds");
    let application = application_with_policy_row();
    let request = base_request(&application);

    let outcome = engine
        .admit_application_with_explicit_rows(request, &application)
        .await;

    match outcome {
        ApplicationAdmissionOutcome::Rejected { failure, report } => {
            assert_eq!(failure.kind, ApplicationFailureKind::RequiresViolation);
            assert_eq!(report.status, ApplicationReportStatus::Failed);
            assert!(
                failure
                    .evidence
                    .notes
                    .iter()
                    .any(|note| note.contains("pii.redact")),
                "diagnostic should name the unsupported policy: {failure:?}"
            );
        }
        ApplicationAdmissionOutcome::Admitted { .. } => {
            panic!("policy row must fail closed as unsupported")
        }
    }
}

fn imported_application(module_source: &str, import_name: &str) -> ash_engine::Entry {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();
    let library = dir.join("library.ash");
    let caller = dir.join("caller.ash");

    write(&library, module_source);
    write(
        &caller,
        &format!("use library::{{{import_name}}}\nfn main() -> String {{ \"ok\" }}\n"),
    );

    Engine::new()
        .build()
        .expect("engine builds")
        .parse_file(&caller)
        .expect("caller with import should parse")
}

#[tokio::test]
async fn imported_operation_row_with_registered_provider_reaches_closed_application_admission() {
    let observe_calls = Arc::new(AtomicUsize::new(0));
    let execute_calls = Arc::new(AtomicUsize::new(0));
    let provider = CountingProvider {
        name: "posixfs",
        observe_calls: Arc::clone(&observe_calls),
        execute_calls: Arc::clone(&execute_calls),
    };
    let engine = Engine::new()
        .with_custom_provider("posixfs", Arc::new(provider))
        .build()
        .expect("engine builds");

    let application = imported_application(
        "pub fn read(path: String) -> {posixfs.read} String { path }\n",
        "read",
    );
    let request = base_request(&application);

    assert!(application.callable_row_requirements.contains_key("read"));

    let outcome = engine
        .admit_application_with_explicit_rows(request, &application)
        .await;

    match outcome {
        ApplicationAdmissionOutcome::Rejected { failure, report } => {
            assert_eq!(failure.kind, ApplicationFailureKind::AdmissionFailure);
            assert_eq!(report.status, ApplicationReportStatus::Failed);
        }
        ApplicationAdmissionOutcome::Admitted { .. } => {
            panic!("registered imported row must still use closed checked-Core/CPS admission")
        }
    }
}

#[tokio::test]
async fn imported_operation_row_rejects_when_provider_missing() {
    let engine = Engine::new().build().expect("engine builds");
    let application = imported_application(
        "pub fn read(path: String) -> {posixfs.read} String { path }\n",
        "read",
    );
    let request = base_request(&application);

    let outcome = engine
        .admit_application_with_explicit_rows(request, &application)
        .await;

    match outcome {
        ApplicationAdmissionOutcome::Rejected { failure, .. } => {
            assert_eq!(
                failure.kind,
                ApplicationFailureKind::CapabilityAdmissionFailure
            );
        }
        ApplicationAdmissionOutcome::Admitted { .. } => {
            panic!("imported operation row must reject when provider missing")
        }
    }
}

#[tokio::test]
async fn row_admission_does_not_install_authority_or_call_host_hooks() {
    let observe_calls = Arc::new(AtomicUsize::new(0));
    let execute_calls = Arc::new(AtomicUsize::new(0));
    let provider = CountingProvider {
        name: "posixfs",
        observe_calls: Arc::clone(&observe_calls),
        execute_calls: Arc::clone(&execute_calls),
    };
    let engine = Engine::new()
        .with_custom_provider("posixfs", Arc::new(provider))
        .build()
        .expect("engine builds");

    let application = Engine::new()
        .build()
        .expect("engine builds")
        .parse(ROW_AUTHORITY_SOURCE)
        .expect("row-bearing source parses");

    let before_provider_count = engine.provider_count();
    let before_resource_count = engine.resource_initializer_selection_count();
    let before_cap_impl_count = engine.capability_implementation_selection_count();

    let request = base_request(&application);
    let _ = engine
        .admit_application_with_explicit_rows(request, &application)
        .await;

    assert_eq!(
        engine.provider_count(),
        before_provider_count,
        "row admission must not register providers"
    );
    assert_eq!(
        engine.resource_initializer_selection_count(),
        before_resource_count,
        "row admission must not select resources"
    );
    assert_eq!(
        engine.capability_implementation_selection_count(),
        before_cap_impl_count,
        "row admission must not select capability implementations"
    );
    assert_eq!(
        observe_calls.load(Ordering::SeqCst),
        0,
        "row admission must not call host observe"
    );
    assert_eq!(
        execute_calls.load(Ordering::SeqCst),
        0,
        "row admission must not call host execute"
    );
}

#[test]
fn row_admission_requirement_derives_from_core_row() {
    use ash_core::core_ash::{CoreRow, CoreRowItem};

    let row = CoreRow::closed(vec![
        CoreRowItem::operation(vec!["posixfs".to_string()], "read"),
        CoreRowItem::Resource {
            path: vec!["vault".to_string()],
            mode: "write".to_string(),
        },
        CoreRowItem::Role {
            path: vec!["tenant".to_string(), "admin".to_string()],
        },
    ]);

    let requirements = RowAdmissionRequirement::from_core_row(&row);
    assert!(requirements.iter().any(|r| matches!(
        r,
        RowAdmissionRequirement::Operation { authority, operation }
            if authority == "posixfs" && operation == "read"
    )));
    assert!(requirements.iter().any(|r| matches!(
        r,
        RowAdmissionRequirement::Resource { resource, mode }
            if resource == "vault" && mode == "write"
    )));
    assert!(requirements.iter().any(|r| matches!(
        r,
        RowAdmissionRequirement::Role { role } if role == "tenant.admin"
    )));
}

#[test]
fn row_admission_check_operation_satisfied_when_provider_present() {
    let engine = Engine::new()
        .with_fs_capabilities()
        .build()
        .expect("engine builds");
    let application = Engine::new()
        .build()
        .expect("engine builds")
        .parse("fn main() -> Int { 0 }")
        .expect("parses");
    let request = base_request(&application);
    let req = RowAdmissionRequirement::Operation {
        authority: "fs".to_string(),
        operation: "read".to_string(),
    };

    assert!(matches!(
        RowAdmissionCheck::check(&engine, &request, &req),
        RowAdmissionCheck::Satisfied
    ));
}
