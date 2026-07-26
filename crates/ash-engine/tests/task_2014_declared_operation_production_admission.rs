//! TASK-2014 RED coverage for a declaration-resolved provider Raise.
//!
//! The first production path accepted only the built-in `time::sleep` fact.
//! This contract requires the same sealed checked Core/CPS path for one
//! ordinary declaration-resolved operation; rows and generic execution remain
//! non-authoritative.

use ash_core::{
    Effect, Expr, Value,
    capability::{
        CapabilityError, CapabilityProvider, ProviderAuthoringMetadata, ProviderOperationMetadata,
    },
    cps::{Atom, ContRef, Term},
    semantic_summary::SourceOrigin,
};
use ash_engine::{
    Engine, ProductionCheckedCpsOutcome, checked_cps_admission::FrameInstallationInstructionV1,
    standard_profiles::StandardProviderProfile,
};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

const LITERAL_DELAY_SOURCE: &str = r"
interface Clock<T> { sleep(Int) -> Null }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = null }
fn main() -> Null { TestClock::sleep(7) }
";

const LEXICAL_DELAY_SOURCE: &str = r"
interface Clock<T> { sleep(Int) -> Null }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = null }
fn main() -> Null {
    let delay = 7;
    TestClock::sleep(delay)
}
";

const TIME_SLEEP_SOURCE: &str = "fn main() -> Null { time::sleep(0) }";
const CLOSED_ADMISSION_ERROR: &str =
    "checked Core/CPS admission rejected: no validated production typed lowering is available";

#[derive(Debug)]
struct RecordingTestClockProvider {
    required_row: &'static str,
    calls: Arc<Mutex<Vec<Vec<Value>>>>,
}

#[async_trait]
impl CapabilityProvider for RecordingTestClockProvider {
    fn name(&self) -> &'static str {
        "test-clock-host"
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    fn provider_metadata(&self) -> ProviderAuthoringMetadata {
        ProviderAuthoringMetadata::new(self.name()).with_operation(
            ProviderOperationMetadata::new("sleep", Effect::Operational)
                .with_required_row(self.required_row)
                .with_sandbox_policy("test.clock.sleep")
                .with_provenance_policy("test.clock.sleep.redacted"),
        )
    }

    async fn observe(
        &self,
        _constraints: &[ash_core::Constraint],
    ) -> Result<Value, CapabilityError> {
        Err(CapabilityError::NotAvailable(
            "test clock does not support observation".to_string(),
        ))
    }

    async fn execute(&self, action: &str, args: &[Value]) -> Result<Value, CapabilityError> {
        if action != "sleep" {
            return Err(CapabilityError::NotAvailable(format!(
                "unexpected test clock action '{action}'"
            )));
        }
        self.calls
            .lock()
            .expect("test clock call log is not poisoned")
            .push(args.to_vec());
        Ok(Value::Null)
    }
}

fn checked_entry(engine: &Engine, source: &str) -> ash_engine::Entry {
    let mut entry = engine
        .parse(source)
        .expect("declared TestClock fixture parses");
    engine
        .check(&mut entry)
        .expect("declared TestClock fixture type-checks");
    entry
}

fn recording_engine(required_row: &'static str) -> (Engine, Arc<Mutex<Vec<Vec<Value>>>>) {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let engine = Engine::new()
        .with_custom_provider(
            "test-clock-host",
            Arc::new(RecordingTestClockProvider {
                required_row,
                calls: Arc::clone(&calls),
            }),
        )
        .build()
        .expect("test clock engine builds");
    (engine, calls)
}

fn register_exact_binding(engine: &Engine, entry: &ash_engine::Entry) {
    engine
        .register_declared_operation_provider_binding(
            entry
                .declared_concrete_operation
                .as_ref()
                .expect("checked entry retains its resolved TestClock operation"),
            "test-clock-host",
            "sleep",
        )
        .expect("the exact declaration-resolved provider binding registers");
}

fn assert_exact_declared_raise(engine: &Engine, entry: &ash_engine::Entry) {
    let Term::Raise {
        op,
        args,
        resume,
        row,
    } = engine
        .lower_entry_to_checked_cps(entry)
        .expect("checked TestClock entry has a CPS Raise inspection artifact")
    else {
        panic!("declared TestClock operation must lower to a CPS Raise");
    };
    assert_eq!(op.item.namespace, "TestClock");
    assert_eq!(op.item.name, "sleep");
    assert_eq!(op.arg_types, ["Int"]);
    assert_eq!(op.result_type, "Null");
    assert_eq!(args, vec![Atom::Int(7)]);
    assert!(matches!(resume, ContRef::Label(label) if label == "__answer"));
    assert_eq!(row.items.len(), 1);
    assert_eq!(row.items[0].namespace, "TestClock");
    assert_eq!(row.items[0].name, "sleep");
}

fn tamper_legacy_delay_to_99(entry: &mut ash_engine::Entry) {
    match &mut entry.core {
        Expr::Call { arguments, .. } => {
            let [Expr::Literal(Value::Int(delay))] = arguments.as_mut_slice() else {
                panic!("literal declared-operation fixture retains one integer argument");
            };
            *delay = 99;
        }
        Expr::Let { expr, .. } => {
            let Expr::Literal(Value::Int(delay)) = expr.as_mut() else {
                panic!("lexical declared-operation fixture retains an integer delay binding");
            };
            *delay = 99;
        }
        other => {
            panic!("fixture must retain a literal call or lexical delay spine, found {other:?}")
        }
    }
}

#[tokio::test]
async fn declared_test_clock_literal_and_lexical_delays_execute_only_through_sealed_checked_cps() {
    for (name, source) in [
        ("literal delay", LITERAL_DELAY_SOURCE),
        ("lexical delay", LEXICAL_DELAY_SOURCE),
    ] {
        let (engine, calls) = recording_engine("TestClock.sleep");
        let mut entry = checked_entry(&engine, source);
        assert_exact_declared_raise(&engine, &entry);
        register_exact_binding(&engine, &entry);

        let direct_error = engine.execute(&entry).await.expect_err(&format!(
            "generic execution must not directly dispatch {name}"
        ));
        assert!(
            matches!(
                direct_error,
                ash_interp::ExecError::ExecutionFailed(ref message) if message == CLOSED_ADMISSION_ERROR
            ),
            "{name} must retain the Path-B closed-admission error rather than a direct-evaluator fallback: {direct_error}"
        );
        assert!(
            calls.lock().expect("call log remains available").is_empty(),
            "generic execution must not dispatch {name} before sealed production admission"
        );

        let admission = engine.admit_production_checked_cps(&mut entry).expect(
            "checked declaration-resolved TestClock::sleep must mint a sealed production token",
        );
        let [
            FrameInstallationInstructionV1::Provider {
                operation,
                provider_binding,
            },
        ] = admission.frame_installation_summary()
        else {
            panic!(
                "declared TestClock admission must retain exactly one separately authorized Provider instruction"
            );
        };
        assert_eq!(operation.impl_type(), "TestClock");
        assert_eq!(operation.interface(), "Clock");
        assert_eq!(operation.operation(), "sleep");
        assert_eq!(operation.parameter_types(), ["Int"]);
        assert_eq!(operation.result_type(), "Null");
        assert_eq!(provider_binding.operation(), operation);
        assert_eq!(provider_binding.provider_name(), "test-clock-host");
        assert_eq!(provider_binding.provider_operation(), "sleep");

        let (control, _cancellation) = engine
            .new_production_run_control(&admission, None)
            .expect("the issuing Engine creates execution control only after admission");
        let outcome = engine
            .execute_production_checked_cps(&admission, control)
            .await
            .expect("the Engine-private checked-CPS driver dispatches the sealed provider frame");
        assert!(matches!(
            outcome,
            ProductionCheckedCpsOutcome::Return(Value::Null)
        ));
        assert_eq!(
            calls.lock().expect("call log remains available").as_slice(),
            [vec![Value::Int(7)]],
            "the sealed driver must dispatch the exact checked literal/local value"
        );
    }
}

#[tokio::test]
async fn declared_test_clock_missing_or_mismatched_binding_rejects_before_dispatch() {
    let (missing_engine, missing_calls) = recording_engine("TestClock.sleep");
    let mut missing_entry = checked_entry(&missing_engine, LITERAL_DELAY_SOURCE);
    assert!(
        missing_engine
            .admit_production_checked_cps(&mut missing_entry)
            .is_err(),
        "a declaration-resolved Raise without its exact provider binding must reject"
    );
    assert!(
        missing_calls
            .lock()
            .expect("call log remains available")
            .is_empty(),
        "missing binding must reject before provider dispatch"
    );

    let (mismatch_engine, mismatch_calls) = recording_engine("OtherClock.sleep");
    let mut mismatch_entry = checked_entry(&mismatch_engine, LITERAL_DELAY_SOURCE);
    assert!(
        mismatch_engine
            .register_declared_operation_provider_binding(
                mismatch_entry
                    .declared_concrete_operation
                    .as_ref()
                    .expect("checked entry retains TestClock identity"),
                "test-clock-host",
                "sleep",
            )
            .is_err(),
        "a provider binding with a different declared operation row must reject"
    );
    assert!(
        mismatch_engine
            .admit_production_checked_cps(&mut mismatch_entry)
            .is_err(),
        "a rejected mismatched binding must not leave production authority behind"
    );
    assert!(
        mismatch_calls
            .lock()
            .expect("call log remains available")
            .is_empty(),
        "mismatched binding must reject before provider dispatch"
    );
}

#[tokio::test]
async fn declared_test_clock_mutated_legacy_delay_rejects_before_provider_dispatch() {
    for (name, source) in [
        ("literal delay", LITERAL_DELAY_SOURCE),
        ("lexical delay", LEXICAL_DELAY_SOURCE),
    ] {
        let (engine, calls) = recording_engine("TestClock.sleep");
        let mut entry = checked_entry(&engine, source);
        register_exact_binding(&engine, &entry);
        tamper_legacy_delay_to_99(&mut entry);

        assert!(
            engine.admit_production_checked_cps(&mut entry).is_err(),
            "a public legacy Core {name} mutation must not change the sealed provider argument"
        );
        assert!(
            calls.lock().expect("call log remains available").is_empty(),
            "a public legacy Core {name} mutation must reject before provider dispatch"
        );
    }
}

#[tokio::test]
async fn declared_test_clock_precheck_legacy_delay_mutation_rejects_before_provider_dispatch() {
    for (name, source) in [
        ("literal delay", LITERAL_DELAY_SOURCE),
        ("lexical delay", LEXICAL_DELAY_SOURCE),
    ] {
        let (engine, calls) = recording_engine("TestClock.sleep");
        let canonical_entry = checked_entry(&engine, source);
        register_exact_binding(&engine, &canonical_entry);

        let mut entry = engine
            .parse(source)
            .expect("a fresh declaration-resolved fixture parses before checking");
        assert!(
            entry.declared_concrete_operation.is_none(),
            "the fresh {name} fixture has no public checked-operation sidecar yet"
        );
        tamper_legacy_delay_to_99(&mut entry);

        assert!(
            engine.admit_production_checked_cps(&mut entry).is_err(),
            "a pre-check public legacy Core {name} mutation must not become a sealed provider argument"
        );
        assert!(
            calls.lock().expect("call log remains available").is_empty(),
            "a pre-check public legacy Core {name} mutation must reject before provider dispatch"
        );
    }
}

#[tokio::test]
async fn declared_test_clock_forged_anchor_or_operation_rejects_before_dispatch() {
    let (anchor_engine, anchor_calls) = recording_engine("TestClock.sleep");
    let mut anchor_entry = checked_entry(&anchor_engine, LITERAL_DELAY_SOURCE);
    register_exact_binding(&anchor_engine, &anchor_entry);
    anchor_entry.lowering_sidecars.entry_body_origin.origin = SourceOrigin::Synthetic {
        reason: "forged declared-operation source origin".to_string(),
    };
    assert!(
        anchor_engine
            .admit_production_checked_cps(&mut anchor_entry)
            .is_err(),
        "a public source-anchor mutation must not mint a declaration-resolved production token"
    );
    assert!(
        anchor_calls
            .lock()
            .expect("call log remains available")
            .is_empty(),
        "forged anchor must reject before provider dispatch"
    );

    let (operation_engine, operation_calls) = recording_engine("TestClock.sleep");
    let mut operation_entry = checked_entry(&operation_engine, LITERAL_DELAY_SOURCE);
    register_exact_binding(&operation_engine, &operation_entry);
    operation_entry
        .declared_concrete_operation
        .as_mut()
        .expect("checked entry retains TestClock identity")
        .operation = "wake".to_string();
    assert!(
        operation_engine
            .admit_production_checked_cps(&mut operation_entry)
            .is_err(),
        "a public operation-identity mutation must reject rather than retarget provider dispatch"
    );
    assert!(
        operation_calls
            .lock()
            .expect("call log remains available")
            .is_empty(),
        "forged operation identity must reject before provider dispatch"
    );
}

#[tokio::test(start_paused = true)]
async fn built_in_time_sleep_exact_route_remains_compatible() {
    let engine = Engine::new().build().expect("engine builds");
    engine
        .install_standard_profile(StandardProviderProfile::application_default(
            "task-2014-declared-operation-time-compatibility",
            std::iter::empty::<&std::path::Path>(),
            std::iter::empty::<&str>(),
        ))
        .await
        .expect("time profile installs");
    engine
        .register_time_sleep_provider_binding()
        .expect("exact built-in time binding registers");
    let mut entry = checked_entry(&engine, TIME_SLEEP_SOURCE);
    let admission = engine
        .admit_production_checked_cps(&mut entry)
        .expect("the original exact time::sleep route remains admissible");
    let (control, _cancellation) = engine
        .new_production_run_control(&admission, None)
        .expect("time route control is Engine-issued");
    let outcome = engine
        .execute_production_checked_cps(&admission, control)
        .await
        .expect("the original exact time route remains executable");
    assert!(matches!(
        outcome,
        ProductionCheckedCpsOutcome::Return(Value::Null)
    ));
}
