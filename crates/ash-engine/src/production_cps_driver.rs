//! Private provider-frame construction for production checked-CPS execution.
//!
//! This module accepts only an Engine-issued production admission token.  It
//! intentionally constructs no frames from effect rows, public V1 evidence, or
//! caller-provided instructions. The driver is deliberately limited to the
//! one sealed `time::sleep` CPS producer admitted by TASK-2014.

use crate::{
    Engine, EngineError,
    checked_cps_admission::{
        CheckedCpsProductionAdmission, FrameInstallationInstructionV1, OperationIdentityV1,
        ProviderBindingV1, ResolvedProviderBinding,
    },
};
use ash_core::{
    Value as EngineValue,
    capability::CapabilityProvider,
    cps::{Atom as CpsAtom, ContRef, Term as CpsTerm, TrapReason, Value as CpsValue},
};
use std::time::Duration;
use tokio::sync::watch;

/// Authority-neutral terminal observation from one production checked-CPS run.
///
/// The Engine creates the input token and control envelope; this outcome
/// exposes neither CPS nor provider/frame authority to callers.
#[derive(Debug, Clone, PartialEq)]
pub enum ProductionCheckedCpsOutcome {
    /// The sealed checked CPS answer continuation returned a host value.
    Return(EngineValue),
    /// A checked CPS terminal trap was reached.
    Trap(TrapReason),
    /// The absolute execution-phase deadline expired.
    TimedOut,
    /// Cooperative cancellation was observed.
    Cancelled,
}

/// Opaque Engine-created cooperative control envelope for one production run.
///
/// It carries no admission, provider, row, or frame authority. Its absolute
/// deadline starts when the Engine creates it, after callers have successfully
/// obtained a production admission token.
pub struct ProductionRunControl {
    deadline: Option<tokio::time::Instant>,
    cancellation: watch::Receiver<bool>,
    admission_token: std::sync::Arc<()>,
}

/// Opaque cooperative cancellation handle paired with a run control.
#[derive(Clone)]
pub struct ProductionCancellation {
    cancellation: watch::Sender<bool>,
}

impl ProductionCancellation {
    /// Requests cooperative cancellation. In-flight provider futures are
    /// dropped by the Engine driver; host rollback is not implied.
    pub fn cancel(&self) {
        self.cancellation.send_replace(true);
    }
}

impl ProductionRunControl {
    pub(crate) fn new(
        admission: &CheckedCpsProductionAdmission,
        timeout: Option<Duration>,
    ) -> Result<(Self, ProductionCancellation), EngineError> {
        let (cancellation_sender, cancellation) = watch::channel(false);
        let now = tokio::time::Instant::now();
        let deadline = timeout
            .map(|duration| {
                now.checked_add(duration).ok_or_else(|| {
                    EngineError::Type(
                        "production run-control deadline exceeds tokio Instant range".to_string(),
                    )
                })
            })
            .transpose()?;
        Ok((
            Self {
                deadline,
                cancellation,
                admission_token: admission.run_control_token(),
            },
            ProductionCancellation {
                cancellation: cancellation_sender,
            },
        ))
    }

    /// Verifies the private per-admission seal before the driver constructs a
    /// frame or observes a provider.
    #[must_use]
    pub(crate) fn is_for_admission(&self, admission: &CheckedCpsProductionAdmission) -> bool {
        admission.has_run_control_token(&self.admission_token)
    }

    fn terminal_outcome(&self) -> Option<ProductionCheckedCpsOutcome> {
        if *self.cancellation.borrow() {
            Some(ProductionCheckedCpsOutcome::Cancelled)
        } else if self
            .deadline
            .is_some_and(|deadline| deadline <= tokio::time::Instant::now())
        {
            Some(ProductionCheckedCpsOutcome::TimedOut)
        } else {
            None
        }
    }

    async fn cancelled(&self) {
        let mut cancellation = self.cancellation.clone();
        loop {
            if *cancellation.borrow() {
                return;
            }
            if cancellation.changed().await.is_err() {
                std::future::pending::<()>().await;
            }
        }
    }
}

/// One provider frame constructed from an Engine-sealed instruction and its
/// exact registry-resolved provider object.
struct ProductionProviderFrame {
    operation: OperationIdentityV1,
    resolved_binding: ResolvedProviderBinding,
}

impl ProductionProviderFrame {
    /// Returns the exact operation discharged by this frame.
    #[must_use]
    const fn operation(&self) -> &OperationIdentityV1 {
        &self.operation
    }

    /// Returns the exact admitted provider-binding identity for this frame.
    #[must_use]
    const fn provider_binding(&self) -> &ProviderBindingV1 {
        self.resolved_binding.binding()
    }

    /// Returns the exact provider object sealed into the production token.
    #[must_use]
    fn provider(&self) -> &std::sync::Arc<dyn CapabilityProvider> {
        self.resolved_binding.provider()
    }
}

/// Ordered provider frames eligible for production checked-CPS dispatch.
///
/// Frames retain their separately authorized installation order. Lookup scans
/// that order backwards, preserving TASK-1993's innermost-first rule.
#[derive(Default)]
struct ProductionProviderFrameChain {
    frames: Vec<ProductionProviderFrame>,
}

/// Prepares the one sealed provider-backed production CPS slice without
/// retaining its non-thread-safe CPS evidence across a provider await.
///
/// The token's issuer seal and exact instruction-to-provider binding are
/// revalidated while constructing the private frame. Unsupported CPS shapes
/// fail closed instead of falling back to either interpreter evaluator.
pub fn prepare_production_checked_cps(
    engine: &Engine,
    admission: &CheckedCpsProductionAdmission,
) -> Result<PreparedProductionTimeSleep, EngineError> {
    let frames = ProductionProviderFrameChain::from_engine_admission(engine, admission)?;
    ash_interp::cps::validate::validate_cps_program(admission.executable()).map_err(|error| {
        EngineError::Type(format!(
            "production checked-CPS driver received invalid CPS: {error}"
        ))
    })?;

    let CpsTerm::LetCont {
        name,
        param,
        cont_body,
        body,
        ..
    } = admission.executable()
    else {
        return Err(closed_driver_error(
            "production token lacks its terminal answer continuation",
        ));
    };

    let CpsTerm::Raise {
        op, args, resume, ..
    } = body.as_ref()
    else {
        return Err(closed_driver_error(
            "production driver admits only its sealed provider Raise reduction",
        ));
    };
    let Some(frame) = frames.find_cps_operation_frame(op) else {
        return Err(closed_driver_error(
            "production provider Raise has no exact sealed provider frame",
        ));
    };
    if !exact_frame_operation_matches_cps(frame, op) {
        return Err(closed_driver_error(
            "production provider Raise does not match its sealed frame operation",
        ));
    }
    let [CpsAtom::Int(duration)] = args.as_slice() else {
        return Err(closed_driver_error(
            "production time.sleep Raise must retain its sealed integer literal",
        ));
    };

    if !is_terminal_answer_continuation(name, param, cont_body, resume) {
        return Err(closed_driver_error(
            "production provider Raise does not resume the sealed terminal answer continuation",
        ));
    }

    Ok(PreparedProductionTimeSleep {
        provider: std::sync::Arc::clone(frame.provider()),
        action: frame.provider_binding().provider_operation().to_string(),
        duration: *duration,
    })
}

/// Send-safe execution data extracted from an opaque checked-CPS token before
/// the driver crosses an async provider boundary.
pub struct PreparedProductionTimeSleep {
    provider: std::sync::Arc<dyn CapabilityProvider>,
    action: String,
    duration: i64,
}

impl PreparedProductionTimeSleep {
    pub async fn execute(
        self,
        control: ProductionRunControl,
    ) -> Result<ProductionCheckedCpsOutcome, EngineError> {
        if let Some(outcome) = control.terminal_outcome() {
            return Ok(outcome);
        }

        let args = [EngineValue::Int(self.duration)];
        let provider_result =
            match race_provider_execution(&control, self.provider.execute(&self.action, &args))
                .await?
            {
                ProviderAwaitResult::Completed(result) => result,
                ProviderAwaitResult::Control(outcome) => return Ok(outcome),
            };

        if let Some(outcome) = control.terminal_outcome() {
            return Ok(outcome);
        }

        if matches!(provider_result, EngineValue::Null) {
            Ok(ProductionCheckedCpsOutcome::Return(EngineValue::Null))
        } else {
            Err(closed_driver_error(
                "production time.sleep provider returned a value outside the sealed Null result type",
            ))
        }
    }
}

fn closed_driver_error(reason: &str) -> EngineError {
    EngineError::Type(format!("production checked-CPS driver rejected: {reason}"))
}

fn exact_frame_operation_matches_cps(
    frame: &ProductionProviderFrame,
    operation: &ash_core::cps::EffectOp,
) -> bool {
    operation.item.namespace == "cap"
        && operation.item.name
            == format!(
                "{}.{}",
                frame.operation().impl_type(),
                frame.operation().operation()
            )
        && operation.arg_types == frame.operation().parameter_types()
        && operation.result_type == frame.operation().result_type()
}

fn is_terminal_answer_continuation(
    name: &str,
    param: &str,
    body: &CpsTerm,
    resume: &ContRef,
) -> bool {
    matches!(
        (body, resume),
        (
            CpsTerm::Return {
                value: CpsValue::Atom(CpsAtom::Var(returned)),
            },
            ContRef::Label(target),
        ) if target == name && returned == param
    )
}

enum ProviderAwaitResult {
    Completed(EngineValue),
    Control(ProductionCheckedCpsOutcome),
}

async fn race_provider_execution(
    control: &ProductionRunControl,
    execution: impl std::future::Future<
        Output = Result<EngineValue, ash_core::capability::CapabilityError>,
    >,
) -> Result<ProviderAwaitResult, EngineError> {
    if let Some(outcome) = control.terminal_outcome() {
        return Ok(ProviderAwaitResult::Control(outcome));
    }

    let provider_result = if let Some(deadline) = control.deadline {
        tokio::select! {
            biased;
            () = control.cancelled() => return Ok(ProviderAwaitResult::Control(ProductionCheckedCpsOutcome::Cancelled)),
            () = tokio::time::sleep_until(deadline) => return Ok(ProviderAwaitResult::Control(ProductionCheckedCpsOutcome::TimedOut)),
            result = execution => result,
        }
    } else {
        tokio::select! {
            biased;
            () = control.cancelled() => return Ok(ProviderAwaitResult::Control(ProductionCheckedCpsOutcome::Cancelled)),
            result = execution => result,
        }
    };
    provider_result
        .map(ProviderAwaitResult::Completed)
        .map_err(|error| {
            EngineError::Execution(format!("sealed production provider failed: {error}"))
        })
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "TASK-2014 Task 3 consumes the Engine-private chain in the async production driver"
    )
)]
impl ProductionProviderFrameChain {
    /// Builds provider frames from an Engine-issued production admission.
    ///
    /// The issuer seal, explicit frame instruction, concrete operation
    /// identity, and exact registry-resolved binding must all agree before a
    /// frame exists. Effect rows do not participate in this construction.
    fn from_engine_admission(
        engine: &Engine,
        admission: &CheckedCpsProductionAdmission,
    ) -> Result<Self, EngineError> {
        if !admission.is_issued_by(&engine.production_checked_cps_execution_token) {
            return Err(EngineError::Type(
                "production provider-frame construction requires an admission issued by this Engine"
                    .to_string(),
            ));
        }

        let mut remaining_bindings = admission
            .resolved_provider_bindings()
            .iter()
            .cloned()
            .map(Some)
            .collect::<Vec<_>>();
        let mut frames = Vec::with_capacity(admission.frame_installation_summary().len());

        for instruction in admission.frame_installation_summary() {
            let FrameInstallationInstructionV1::Provider {
                operation,
                provider_binding,
            } = instruction
            else {
                return Err(EngineError::Type(
                    "production provider admission contains a non-provider frame instruction"
                        .to_string(),
                ));
            };

            let Some(binding_index) = remaining_bindings.iter().position(|candidate| {
                candidate.as_ref().is_some_and(|candidate| {
                    candidate.binding() == provider_binding
                        && candidate.binding().operation() == operation
                })
            }) else {
                return Err(EngineError::Type(
                    "production provider-frame instruction lacks its exact resolved binding"
                        .to_string(),
                ));
            };
            let Some(resolved_binding) = remaining_bindings[binding_index].take() else {
                return Err(EngineError::Type(
                    "production provider-frame binding was consumed during construction"
                        .to_string(),
                ));
            };

            frames.push(ProductionProviderFrame {
                operation: operation.clone(),
                resolved_binding,
            });
        }

        Ok(Self { frames })
    }

    /// Returns the innermost explicitly authorized provider frame for an
    /// exact operation identity.
    #[must_use]
    fn find_operation_frame(
        &self,
        operation: &OperationIdentityV1,
    ) -> Option<&ProductionProviderFrame> {
        self.frames
            .iter()
            .rev()
            .find(|frame| frame.operation() == operation)
    }

    /// Returns the innermost authorized frame whose sealed concrete identity
    /// exactly matches a checked CPS effect operation. This compares against
    /// the frame's sealed identity directly; it never rebuilds an operation
    /// identity from a row or CPS spelling.
    #[must_use]
    fn find_cps_operation_frame(
        &self,
        operation: &ash_core::cps::EffectOp,
    ) -> Option<&ProductionProviderFrame> {
        self.frames
            .iter()
            .rev()
            .find(|frame| exact_frame_operation_matches_cps(frame, operation))
    }

    /// Returns the number of explicitly constructed provider frames.
    #[must_use]
    const fn len(&self) -> usize {
        self.frames.len()
    }
}

#[cfg(test)]
mod tests {
    use super::ProductionProviderFrameChain;
    use crate::{
        Engine,
        checked_cps_admission::{CheckedCpsProductionAdmission, FrameInstallationInstructionV1},
        standard_profiles::StandardProviderProfile,
    };

    const SLEEP: &str = "fn main() -> Null { time::sleep(0) }";

    async fn install_application_time_profile(engine: &Engine) {
        engine
            .install_standard_profile(StandardProviderProfile::application_default(
                "task-2014-production-frame-order",
                std::iter::empty::<&std::path::Path>(),
                std::iter::empty::<&str>(),
            ))
            .await
            .expect("the standard application profile installs the time provider");
    }

    async fn engine_issued_time_sleep_admission(engine: &Engine) -> CheckedCpsProductionAdmission {
        install_application_time_profile(engine).await;
        engine
            .register_time_sleep_provider_binding()
            .expect("the exact Engine-owned time.sleep binding registers");
        let mut entry = engine.parse(SLEEP).expect("fixture parses");
        engine.check(&mut entry).expect("fixture type-checks");
        engine
            .admit_production_checked_cps(&mut entry)
            .expect("the exact checked time::sleep source seals a production token")
    }

    #[tokio::test]
    async fn only_an_engine_issued_admission_constructs_the_exact_explicit_provider_frame() {
        let engine = Engine::new().build().expect("engine builds");
        let admission = engine_issued_time_sleep_admission(&engine).await;
        let [
            FrameInstallationInstructionV1::Provider {
                operation,
                provider_binding,
            },
        ] = admission.frame_installation_summary()
        else {
            panic!("the narrow production slice seals one explicit Provider instruction");
        };

        let frames = ProductionProviderFrameChain::from_engine_admission(&engine, &admission)
            .expect("only the issuing Engine may turn its sealed token into frames");

        assert_eq!(frames.len(), 1, "rows cannot add implicit provider frames");
        let selected = frames
            .find_operation_frame(operation)
            .expect("the explicit instruction installs its exact operation frame");
        assert_eq!(
            selected.operation(),
            operation,
            "frame lookup must preserve namespace, name, parameter types, and result type"
        );
        assert_eq!(
            selected.provider_binding(),
            provider_binding,
            "the frame must retain the registry-resolved binding sealed in the token"
        );
    }

    #[tokio::test]
    async fn a_foreign_engine_cannot_construct_frames_from_another_engines_admission() {
        let issuing_engine = Engine::new().build().expect("issuing engine builds");
        let admission = engine_issued_time_sleep_admission(&issuing_engine).await;
        let foreign_engine = Engine::new().build().expect("foreign engine builds");

        assert!(
            ProductionProviderFrameChain::from_engine_admission(&foreign_engine, &admission)
                .is_err(),
            "the private carrier must reject a token whose issuer seal does not match the Engine"
        );
    }
}
