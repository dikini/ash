//! TASK-2017 RED coverage for one normal symbolic `PosixFs::read` operation.
//!
//! The provider below is deliberately controlled: it never touches the host
//! filesystem and records the one typed argument it receives.  The tests use
//! ordinary source declarations and the public parse/check/admission/execute
//! path; CPS inspection remains explicitly private.

use ash_core::{
    Effect, Value,
    capability::{
        CapabilityError, CapabilityProvider, ProviderAuthoringMetadata, ProviderOperationMetadata,
    },
    core_ash::{CoreRowItem, CoreType},
    cps::{Atom, Term},
};
use ash_engine::{ApplicationAdmissionOutcome, ApplicationAdmissionRequest, Engine};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

const LITERAL_READ_SOURCE: &str = r#"
interface Fs<T> { read(String) -> String }
newtype PosixFs = PosixFs(String);
impl Fs<PosixFs> { read(path) = path }
fn main() -> String { PosixFs::read("fixture-path") }
"#;

const LOCAL_READ_SOURCE: &str = r#"
interface Fs<T> { read(String) -> String }
newtype PosixFs = PosixFs(String);
impl Fs<PosixFs> { read(path) = path }
fn main() -> String {
    let path = "fixture-path";
    PosixFs::read(path)
}
"#;

const WRONG_ARGUMENT_TYPE_SOURCE: &str = r"
interface Fs<T> { read(String) -> String }
newtype PosixFs = PosixFs(String);
impl Fs<PosixFs> { read(path) = path }
fn main() -> String { PosixFs::read(0) }
";

const UNKNOWN_OPERATION_SOURCE: &str = r#"
interface Fs<T> { read(String) -> String }
newtype PosixFs = PosixFs(String);
impl Fs<PosixFs> { read(path) = path }
fn main() -> String { PosixFs::missing("fixture-path") }
"#;
const CLOSED_ADMISSION_ERROR: &str =
    "checked Core/CPS admission rejected: no validated production typed lowering is available";

#[derive(Debug)]
struct RecordingPosixFsProvider {
    calls: Arc<Mutex<Vec<Vec<Value>>>>,
}

#[derive(Debug)]
struct MetadataPosixFsProvider {
    operation: &'static str,
    required_row: &'static str,
}

#[async_trait]
impl CapabilityProvider for MetadataPosixFsProvider {
    fn name(&self) -> &'static str {
        "fixture-posixfs-metadata"
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    fn provider_metadata(&self) -> ProviderAuthoringMetadata {
        ProviderAuthoringMetadata::new(self.name()).with_operation(
            ProviderOperationMetadata::new(self.operation, Effect::Operational)
                .with_required_row(self.required_row)
                .with_sandbox_policy("fixture.posixfs.read")
                .with_provenance_policy("fixture.posixfs.read"),
        )
    }

    async fn observe(
        &self,
        _constraints: &[ash_core::Constraint],
    ) -> Result<Value, CapabilityError> {
        Err(CapabilityError::NotAvailable(
            "metadata-only fixture does not support observation".to_string(),
        ))
    }

    async fn execute(&self, _action: &str, _args: &[Value]) -> Result<Value, CapabilityError> {
        Err(CapabilityError::NotAvailable(
            "metadata-only fixture must not dispatch".to_string(),
        ))
    }
}

#[async_trait]
impl CapabilityProvider for RecordingPosixFsProvider {
    fn name(&self) -> &'static str {
        "fixture-posixfs"
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    fn provider_metadata(&self) -> ProviderAuthoringMetadata {
        ProviderAuthoringMetadata::new(self.name()).with_operation(
            ProviderOperationMetadata::new("read", Effect::Operational)
                .with_required_row("PosixFs.read")
                .with_sandbox_policy("fixture.posixfs.read")
                .with_provenance_policy("fixture.posixfs.read"),
        )
    }

    async fn observe(
        &self,
        _constraints: &[ash_core::Constraint],
    ) -> Result<Value, CapabilityError> {
        Err(CapabilityError::NotAvailable(
            "fixture provider does not support observation".to_string(),
        ))
    }

    async fn execute(&self, action: &str, args: &[Value]) -> Result<Value, CapabilityError> {
        if action != "read" {
            return Err(CapabilityError::NotAvailable(format!(
                "unexpected fixture operation '{action}'"
            )));
        }
        if args != [Value::String("fixture-path".to_string())] {
            return Err(CapabilityError::InvalidArgument(format!(
                "expected exactly one fixture String argument, received {args:?}"
            )));
        }
        self.calls
            .lock()
            .expect("recording provider mutex is not poisoned")
            .push(args.to_vec());
        Ok(Value::String("fixture-contents".to_string()))
    }
}

fn request(entry: &ash_engine::Entry) -> ApplicationAdmissionRequest {
    ApplicationAdmissionRequest {
        entry_name: "main".to_string(),
        body: entry.core.clone(),
        application_id: None,
        run_id: None,
        active_role: None,
        admitted_role: None,
        required_capabilities: Vec::new(),
        requires: Vec::new(),
        ensures: Vec::new(),
    }
}

fn checked_entry(engine: &Engine, source: &str) -> ash_engine::Entry {
    let mut entry = engine.parse(source).expect("PosixFs fixture parses");
    engine
        .check(&mut entry)
        .expect("PosixFs::read resolves from local declarations");
    entry
}

fn register_fixture_binding(engine: &Engine, entry: &ash_engine::Entry) {
    engine
        .register_declared_operation_provider_binding(
            entry
                .declared_concrete_operation
                .as_ref()
                .expect("checked entry retains its declared PosixFs operation"),
            "fixture-posixfs",
            "read",
        )
        .expect("exact PosixFs::read binding registers from provider metadata");
}

fn assert_declared_posixfs_read(entry: &ash_engine::Entry) {
    let operation = entry
        .declared_concrete_operation
        .as_ref()
        .expect("normal checking retains a declaration-backed operation");
    assert_eq!(operation.impl_type, "PosixFs");
    assert_eq!(operation.interface, "Fs");
    assert_eq!(operation.operation, "read");
    assert_eq!(
        operation
            .params
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        ["String"]
    );
    assert_eq!(operation.result_type.to_string(), "String");

    let CoreType::Function { row, .. } = entry
        .core_callable_types
        .get("main")
        .expect("checked entry has main's Core callable type")
    else {
        panic!("main must retain a Core function type");
    };
    let matching = row
        .items
        .iter()
        .filter(|item| {
            matches!(
                item,
                CoreRowItem::Operation { path, operation }
                    if path == &["PosixFs".to_string()] && operation == "read"
            )
        })
        .count();
    assert_eq!(
        matching, 1,
        "PosixFs::read must have exactly one non-granting row item: {row:?}"
    );
}

#[test]
fn task_2017_literal_read_resolves_from_one_local_nominal_declaration() {
    let engine = Engine::new().build().expect("engine builds");
    let entry = checked_entry(&engine, LITERAL_READ_SOURCE);
    assert_declared_posixfs_read(&entry);
}

#[test]
fn task_2017_checked_local_string_resolves_to_the_same_declared_identity_and_row() {
    let engine = Engine::new().build().expect("engine builds");
    let entry = checked_entry(&engine, LOCAL_READ_SOURCE);
    assert_declared_posixfs_read(&entry);
}

#[test]
fn task_2017_non_string_argument_fails_normal_checking_before_admission() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(WRONG_ARGUMENT_TYPE_SOURCE)
        .expect("wrong-argument fixture parses");

    let error = engine
        .check(&mut entry)
        .expect_err("PosixFs::read accepts only its declared String argument");
    assert!(
        error
            .to_string()
            .contains("PosixFs::read: argument type mismatch"),
        "unexpected PosixFs::read argument diagnostic: {error}"
    );
    assert!(
        entry.declared_concrete_operation.is_none(),
        "a failed normal check must not create an admission-ready declared operation"
    );
}

#[test]
fn task_2017_unknown_posixfs_operation_fails_declaration_resolution() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(UNKNOWN_OPERATION_SOURCE)
        .expect("unknown-operation fixture parses");

    let error = engine
        .check(&mut entry)
        .expect_err("PosixFs::missing is not declared by the local Fs implementation");
    assert!(
        error.to_string().contains("concrete impl 'PosixFs'")
            && error.to_string().contains("operation 'missing'"),
        "unknown operations must retain their resolved impl and operation identity in diagnostics: {error}"
    );
    assert!(
        entry.declared_concrete_operation.is_none(),
        "an unresolved operation must not reach provider admission"
    );
}

#[test]
fn task_2017_binding_rejects_provider_operation_name_mismatch_for_posixfs_read() {
    let engine = Engine::new()
        .with_custom_provider(
            "fixture-posixfs-metadata",
            Arc::new(MetadataPosixFsProvider {
                operation: "missing",
                required_row: "PosixFs.read",
            }),
        )
        .build()
        .expect("engine builds");
    let entry = checked_entry(&engine, LITERAL_READ_SOURCE);

    let error = engine
        .register_declared_operation_provider_binding(
            entry
                .declared_concrete_operation
                .as_ref()
                .expect("checked PosixFs::read retains its declaration"),
            "fixture-posixfs-metadata",
            "read",
        )
        .expect_err("a PosixFs binding cannot select an undeclared provider operation");
    assert!(
        error.to_string().contains("provider operation 'read'"),
        "unexpected PosixFs provider-operation mismatch: {error}"
    );
}

#[test]
fn task_2017_binding_rejects_required_row_mismatch_for_posixfs_read() {
    let engine = Engine::new()
        .with_custom_provider(
            "fixture-posixfs-metadata",
            Arc::new(MetadataPosixFsProvider {
                operation: "read",
                required_row: "Fs.read",
            }),
        )
        .build()
        .expect("engine builds");
    let entry = checked_entry(&engine, LITERAL_READ_SOURCE);

    let error = engine
        .register_declared_operation_provider_binding(
            entry
                .declared_concrete_operation
                .as_ref()
                .expect("checked PosixFs::read retains its declaration"),
            "fixture-posixfs-metadata",
            "read",
        )
        .expect_err("a PosixFs binding requires its exact concrete operation row");
    assert!(
        error.to_string().contains("PosixFs.read"),
        "provider required-row matching must not accept interface identity: {error}"
    );
}

#[tokio::test]
async fn task_2017_exact_binding_reaches_closed_admission_without_dispatch() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let engine = Engine::new()
        .with_custom_provider(
            "fixture-posixfs",
            Arc::new(RecordingPosixFsProvider {
                calls: Arc::clone(&calls),
            }),
        )
        .build()
        .expect("engine builds");
    let entry = checked_entry(&engine, LOCAL_READ_SOURCE);
    register_fixture_binding(&engine, &entry);

    let ApplicationAdmissionOutcome::Rejected { failure, .. } = engine
        .admit_application_with_explicit_rows(request(&entry), &entry)
        .await
    else {
        panic!(
            "all generic source admission must remain closed after the exact binding discharges its row"
        );
    };
    assert_eq!(
        failure.kind,
        ash_core::runtime::ApplicationFailureKind::AdmissionFailure,
        "the exact binding must pass row discharge before the closed production admission boundary"
    );
    let error = engine.execute(&entry).await.expect_err(
        "generic source execution must not dispatch PosixFs::read before typed lowering",
    );
    assert!(
        matches!(
            error,
            ash_interp::ExecError::ExecutionFailed(message) if message == CLOSED_ADMISSION_ERROR
        ),
        "bound PosixFs::read must expose the exact checked Core/CPS closed-admission error"
    );
    assert!(
        calls
            .lock()
            .expect("recording provider mutex is not poisoned")
            .is_empty(),
        "closed generic execution must not dispatch the string path to the provider"
    );
}

#[tokio::test]
async fn task_2017_missing_binding_rejects_before_the_controlled_provider_executes() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let engine = Engine::new()
        .with_custom_provider(
            "fixture-posixfs",
            Arc::new(RecordingPosixFsProvider {
                calls: Arc::clone(&calls),
            }),
        )
        .build()
        .expect("engine builds");
    let entry = checked_entry(&engine, LITERAL_READ_SOURCE);

    let admission = engine
        .admit_application_with_explicit_rows(request(&entry), &entry)
        .await;
    assert!(
        matches!(admission, ApplicationAdmissionOutcome::Rejected { .. }),
        "a PosixFs::read row must not select a same-spelled provider without an exact binding: {admission:?}"
    );
    let error = engine
        .execute(&entry)
        .await
        .expect_err("generic source execution without a binding must reject at admission");
    assert!(
        matches!(
            error,
            ash_interp::ExecError::ExecutionFailed(message) if message == CLOSED_ADMISSION_ERROR
        ),
        "unbound PosixFs::read must expose the exact checked Core/CPS closed-admission error"
    );
    assert!(
        calls
            .lock()
            .expect("recording provider mutex is not poisoned")
            .is_empty(),
        "missing binding must reject before provider dispatch"
    );
}

#[test]
fn task_2017_checked_cps_inspection_preserves_the_string_atom_for_literal_and_local_paths() {
    for source in [LITERAL_READ_SOURCE, LOCAL_READ_SOURCE] {
        let engine = Engine::new().build().expect("engine builds");
        let entry = checked_entry(&engine, source);
        let Term::Raise { op, args, row, .. } = engine
            .lower_entry_to_checked_cps(&entry)
            .expect("PosixFs::read has a private checked CPS inspection artifact")
        else {
            panic!("declared PosixFs::read must inspect as CPS Raise");
        };
        assert_eq!(op.item.namespace, "PosixFs");
        assert_eq!(op.item.name, "read");
        assert_eq!(op.arg_types, ["String"]);
        assert_eq!(op.result_type, "String");
        assert_eq!(args, vec![Atom::String("fixture-path".to_string())]);
        assert_eq!(row.items.len(), 1);
        assert_eq!(row.items[0].namespace, "PosixFs");
        assert_eq!(row.items[0].name, "read");
    }
}
