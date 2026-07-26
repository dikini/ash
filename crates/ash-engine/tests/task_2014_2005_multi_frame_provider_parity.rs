//! TASK-2014/TASK-2005 correspondence control for real ordered provider frames.
//!
//! The exact checked `forward_sleep` source is the smallest Engine-admitted
//! value-returning operation route. Two separately authorized `wake` providers
//! must survive source/Core/anchor-bound admission in outer-to-inner order so
//! the inner result is observable without treating provider internals as a
//! semantic result. This is Engine-private TASK-1993 correspondence evidence,
//! not a differential direct-runtime/Core-CPS parity claim.

use ash_core::{
    Effect, Value,
    capability::{
        CapabilityError, CapabilityProvider, ProviderAuthoringMetadata, ProviderOperationMetadata,
    },
};
use ash_engine::{
    Engine, ProductionCheckedCpsOutcome, checked_cps_admission::FrameInstallationInstructionV1,
};
use async_trait::async_trait;
use std::sync::Arc;

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
    name: &'static str,
    result: i64,
}

#[async_trait]
impl CapabilityProvider for WakeProvider {
    fn name(&self) -> &'static str {
        self.name
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    fn provider_metadata(&self) -> ProviderAuthoringMetadata {
        ProviderAuthoringMetadata::new(self.name).with_operation(
            ProviderOperationMetadata::new("wake", Effect::Operational)
                .with_required_row("TestClock.wake")
                .with_sandbox_policy("task-2014-2005.multi-frame.wake")
                .with_provenance_policy("task-2014-2005.multi-frame.wake.redacted"),
        )
    }

    async fn observe(
        &self,
        _constraints: &[ash_core::Constraint],
    ) -> Result<Value, CapabilityError> {
        Err(CapabilityError::NotAvailable(
            "the multi-frame parity provider exposes wake only".to_string(),
        ))
    }

    async fn execute(&self, action: &str, args: &[Value]) -> Result<Value, CapabilityError> {
        assert_eq!(action, "wake");
        assert_eq!(args, [Value::Int(0)]);
        Ok(Value::Int(self.result))
    }
}

#[tokio::test]
async fn admitted_forward_sleep_uses_the_inner_of_two_authorized_wake_provider_frames() {
    let engine = Engine::new()
        .with_custom_provider(
            "task-2014-2005-outer-wake",
            Arc::new(WakeProvider {
                name: "task-2014-2005-outer-wake",
                result: 11,
            }),
        )
        .with_custom_provider(
            "task-2014-2005-inner-wake",
            Arc::new(WakeProvider {
                name: "task-2014-2005-inner-wake",
                result: 73,
            }),
        )
        .build()
        .expect("the two-provider test Engine builds");
    let mut entry = engine
        .parse(FORWARD_SLEEP_SOURCE)
        .expect("the exact checked source fixture parses");
    engine
        .check(&mut entry)
        .expect("the exact source fixture typechecks before admission");
    let expected_anchor = entry.lowering_sidecars.entry_body_origin.clone();

    engine
        .register_sealed_forward_sleep_wake_provider_binding(
            &entry,
            "task-2014-2005-outer-wake",
            "wake",
        )
        .expect("the outer provider has an exact checked wake binding");
    engine
        .register_sealed_forward_sleep_wake_provider_binding(
            &entry,
            "task-2014-2005-inner-wake",
            "wake",
        )
        .expect("the inner provider has a separately authorized checked wake binding");

    let admission = engine
        .admit_production_forward_sleep(&mut entry)
        .expect("the exact source/Core/anchor-bound entry admits two ordered provider frames");
    assert_eq!(
        entry.lowering_sidecars.entry_body_origin, expected_anchor,
        "admission must retain the parsed entry anchor rather than reconstructing one from a row or provider"
    );
    let [
        FrameInstallationInstructionV1::Provider {
            operation: outer_operation,
            provider_binding: outer_binding,
        },
        FrameInstallationInstructionV1::Provider {
            operation: inner_operation,
            provider_binding: inner_binding,
        },
        FrameInstallationInstructionV1::SourceHandler {
            operation: sleep_operation,
            handler_name,
            core_handle,
        },
    ] = admission.frame_installation_summary()
    else {
        panic!(
            "admission must seal outer Provider(wake), inner Provider(wake), then SourceHandler(sleep)"
        );
    };
    assert_eq!(outer_operation, inner_operation);
    assert_eq!(outer_operation.impl_type(), "TestClock");
    assert_eq!(outer_operation.interface(), "Clock");
    assert_eq!(outer_operation.operation(), "wake");
    assert_eq!(outer_operation.parameter_types(), ["Int"]);
    assert_eq!(outer_operation.result_type(), "Int");
    assert_eq!(outer_binding.operation(), outer_operation);
    assert_eq!(inner_binding.operation(), inner_operation);
    assert_eq!(outer_binding.provider_name(), "task-2014-2005-outer-wake");
    assert_eq!(inner_binding.provider_name(), "task-2014-2005-inner-wake");
    assert_eq!(sleep_operation.operation(), "sleep");
    assert_eq!(handler_name, "forward_sleep");
    assert!(core_handle.path().is_empty());

    let (control, _cancellation) = engine
        .new_forward_sleep_run_control(&admission, None)
        .expect("the issuing Engine binds the run control to the sealed multi-frame admission");
    assert_eq!(
        engine
            .execute_production_forward_sleep(&admission, control)
            .await
            .expect("the real production driver dispatches the sealed inner provider"),
        ProductionCheckedCpsOutcome::Return(Value::Int(73)),
        "TASK-1993 requires reverse lookup to choose the inner authorized wake frame"
    );
}

#[tokio::test]
async fn duplicate_exact_wake_registration_keeps_one_authorized_provider_instruction() {
    let engine = Engine::new()
        .with_custom_provider(
            "task-2014-2005-duplicate-wake",
            Arc::new(WakeProvider {
                name: "task-2014-2005-duplicate-wake",
                result: 73,
            }),
        )
        .build()
        .expect("the duplicate-registration test Engine builds");
    let mut entry = engine
        .parse(FORWARD_SLEEP_SOURCE)
        .expect("the exact source fixture parses");
    engine
        .check(&mut entry)
        .expect("the exact source fixture typechecks");

    for _ in 0..2 {
        engine
            .register_sealed_forward_sleep_wake_provider_binding(
                &entry,
                "task-2014-2005-duplicate-wake",
                "wake",
            )
            .expect("repeating an exact binding is idempotent");
    }

    let admission = engine
        .admit_production_forward_sleep(&mut entry)
        .expect("one exact idempotent binding still admits");
    assert!(matches!(
        admission.frame_installation_summary(),
        [
            FrameInstallationInstructionV1::Provider { provider_binding, .. },
            FrameInstallationInstructionV1::SourceHandler { .. },
        ] if provider_binding.provider_name() == "task-2014-2005-duplicate-wake"
    ));
}

#[tokio::test]
async fn third_distinct_wake_registration_rejects_before_admission_or_dispatch() {
    let engine = Engine::new()
        .with_custom_provider(
            "task-2014-2005-first-wake",
            Arc::new(WakeProvider {
                name: "task-2014-2005-first-wake",
                result: 11,
            }),
        )
        .with_custom_provider(
            "task-2014-2005-second-wake",
            Arc::new(WakeProvider {
                name: "task-2014-2005-second-wake",
                result: 22,
            }),
        )
        .with_custom_provider(
            "task-2014-2005-third-wake",
            Arc::new(WakeProvider {
                name: "task-2014-2005-third-wake",
                result: 73,
            }),
        )
        .build()
        .expect("the bounded-registration test Engine builds");
    let mut entry = engine
        .parse(FORWARD_SLEEP_SOURCE)
        .expect("the exact source fixture parses");
    engine
        .check(&mut entry)
        .expect("the exact source fixture typechecks");

    for provider in ["task-2014-2005-first-wake", "task-2014-2005-second-wake"] {
        engine
            .register_sealed_forward_sleep_wake_provider_binding(&entry, provider, "wake")
            .expect("the first two distinct bindings are separately authorized");
    }
    let error = engine
        .register_sealed_forward_sleep_wake_provider_binding(
            &entry,
            "task-2014-2005-third-wake",
            "wake",
        )
        .expect_err("the exact sealed route admits no third provider frame");
    assert!(
        error
            .to_string()
            .contains("at most two ordered wake provider bindings"),
        "the bound must reject before an admission artifact or provider dispatch exists: {error}"
    );
    assert!(
        engine.host_boundary_evidence().await.is_empty(),
        "registration rejection must not dispatch a provider"
    );
}
