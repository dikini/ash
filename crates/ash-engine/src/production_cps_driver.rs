//! Private provider-frame construction for production checked-CPS execution.
//!
//! This module accepts only an Engine-issued production admission token.  It
//! intentionally constructs no frames from effect rows, public V1 evidence, or
//! caller-provided instructions. The driver is deliberately limited to the
//! one sealed `time::sleep` CPS producer admitted by TASK-2014.

use crate::{
    Engine, EngineError, ForwardSleepProductionAdmission,
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
/// It carries no admission, provider, row, or frame authority. Direct driver
/// use starts its absolute deadline when the Engine creates it. The shared
/// admitted-program seam derives a fresh deadline from its retained timeout
/// for each submission while retaining the same cancellation channel and
/// admission token.
#[derive(Clone)]
pub struct ProductionRunControl {
    deadline: Option<tokio::time::Instant>,
    timeout: Option<Duration>,
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
    /// Creates a control envelope for an admitted route that has no provider
    /// driver. The control is non-authorizing: it cannot satisfy either
    /// provider driver's private admission-token checks.
    pub(crate) fn new_unbound(
        timeout: Option<Duration>,
    ) -> Result<(Self, ProductionCancellation), EngineError> {
        Self::new_with_admission_token(std::sync::Arc::new(()), timeout)
    }

    pub(crate) fn new(
        admission: &CheckedCpsProductionAdmission,
        timeout: Option<Duration>,
    ) -> Result<(Self, ProductionCancellation), EngineError> {
        Self::new_with_admission_token(admission.run_control_token(), timeout)
    }

    pub(crate) fn new_with_admission_token(
        admission_token: std::sync::Arc<()>,
        timeout: Option<Duration>,
    ) -> Result<(Self, ProductionCancellation), EngineError> {
        let (cancellation_sender, cancellation) = watch::channel(false);
        let deadline = Self::deadline_from_timeout(timeout)?;
        Ok((
            Self {
                deadline,
                timeout,
                cancellation,
                admission_token,
            },
            ProductionCancellation {
                cancellation: cancellation_sender,
            },
        ))
    }

    /// Creates one fresh submission control while preserving the sealed
    /// admission token and shared cancellation state.
    ///
    /// Reusable admitted-program requests retain timeout configuration rather
    /// than one stale deadline. Cancellation intentionally remains sticky:
    /// once requested through its original handle, every later submission
    /// observes the same channel.
    pub(crate) fn fresh_submission(&self) -> Result<Self, EngineError> {
        Ok(Self {
            deadline: Self::deadline_from_timeout(self.timeout)?,
            timeout: self.timeout,
            cancellation: self.cancellation.clone(),
            admission_token: self.admission_token.clone(),
        })
    }

    fn deadline_from_timeout(
        timeout: Option<Duration>,
    ) -> Result<Option<tokio::time::Instant>, EngineError> {
        let now = tokio::time::Instant::now();
        timeout
            .map(|duration| {
                now.checked_add(duration).ok_or_else(|| {
                    EngineError::Type(
                        "production run-control deadline exceeds tokio Instant range".to_string(),
                    )
                })
            })
            .transpose()
    }

    /// Verifies the private per-admission seal before the driver constructs a
    /// frame or observes a provider.
    #[must_use]
    pub(crate) fn is_for_admission(&self, admission: &CheckedCpsProductionAdmission) -> bool {
        admission.has_run_control_token(&self.admission_token)
    }

    pub(crate) fn is_for_forward_sleep_admission(
        &self,
        admission: &ForwardSleepProductionAdmission,
    ) -> bool {
        admission.has_run_control_token(&self.admission_token)
    }

    pub(crate) fn terminal_outcome(&self) -> Option<ProductionCheckedCpsOutcome> {
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

/// Prepare TASK-2026's sealed handler/provider composition. The code
/// deliberately replays one or two explicit provider instructions followed by
/// the source-handler instruction and reverse-scans them for each checked
/// Raise; no effect row participates in frame construction or lookup.
#[allow(
    clippy::too_many_lines,
    reason = "the sealed two-Raise validation remains intentionally linear so every authority check is visible at this boundary"
)]
pub fn prepare_production_forward_sleep(
    engine: &Engine,
    admission: &ForwardSleepProductionAdmission,
) -> Result<PreparedProductionForwardSleep, EngineError> {
    if !admission.is_issued_by(&engine.production_forward_sleep_execution_token) {
        return Err(closed_driver_error(
            "forward_sleep token was not issued by this Engine",
        ));
    }
    let Some((source_handler, provider_instructions)) =
        admission.frame_installation_summary().split_last()
    else {
        return Err(closed_driver_error(
            "forward_sleep token lacks its exact ordered frame instructions",
        ));
    };
    let FrameInstallationInstructionV1::SourceHandler {
        operation: sleep,
        handler_name,
        core_handle,
    } = source_handler
    else {
        return Err(closed_driver_error(
            "forward_sleep token lacks its inner SourceHandler instruction",
        ));
    };
    let providers = admission.resolved_wake_providers();
    if !(1..=2).contains(&provider_instructions.len())
        || provider_instructions.len() != providers.len()
        || provider_instructions
            .iter()
            .zip(providers)
            .any(|(instruction, provider)| {
                !matches!(instruction, FrameInstallationInstructionV1::Provider { operation, provider_binding }
                    if operation == admission.wake_operation()
                        && provider_binding == provider.binding())
            })
        || sleep != admission.sleep_operation()
        || handler_name != "forward_sleep"
        || !core_handle.path().is_empty()
    {
        return Err(closed_driver_error(
            "forward_sleep frame instructions no longer match sealed authority",
        ));
    }

    ash_interp::cps::validate::validate_cps_program(admission.executable()).map_err(|error| {
        EngineError::Type(format!(
            "forward_sleep production driver received invalid CPS: {error}"
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
            "forward_sleep token lacks terminal answer continuation",
        ));
    };
    let CpsTerm::Handle {
        clause,
        body: handled_body,
        cont,
        ..
    } = body.as_ref()
    else {
        return Err(closed_driver_error(
            "forward_sleep driver requires its root checked Handle",
        ));
    };
    if !is_terminal_answer_continuation(name, param, cont_body, cont) {
        return Err(closed_driver_error(
            "forward_sleep Handle does not target its terminal answer continuation",
        ));
    }
    let CpsTerm::Raise {
        op: sleep_raise,
        args: sleep_args,
        resume: sleep_resume,
        ..
    } = handled_body.as_ref()
    else {
        return Err(closed_driver_error(
            "forward_sleep handled body is not its sealed sleep Raise",
        ));
    };
    if !exact_identity_matches_cps(admission.sleep_operation(), sleep_raise)
        || !matches!(sleep_args.as_slice(), [CpsAtom::Int(0)])
        || !is_terminal_answer_continuation(name, param, cont_body, sleep_resume)
    {
        return Err(closed_driver_error(
            "forward_sleep sleep Raise differs from its sealed checked form",
        ));
    }
    // The ordered vector is outer Provider then inner SourceHandler. Reverse
    // lookup must find the handler for sleep before the outer provider.
    if !matches!(
        forward_sleep_reverse_lookup(admission.frame_installation_summary(), sleep_raise),
        ForwardSleepFrameMatch::SourceHandler
    ) {
        return Err(closed_driver_error(
            "forward_sleep sleep Raise did not select its inner source handler",
        ));
    }
    let CpsTerm::Raise {
        op: wake_raise,
        args: wake_args,
        resume: wake_resume,
        ..
    } = clause.body.as_ref()
    else {
        return Err(closed_driver_error(
            "forward_sleep clause is not its sealed wake Raise",
        ));
    };
    let [parameter] = clause.params.as_slice() else {
        return Err(closed_driver_error(
            "forward_sleep handler clause has an unexpected parameter shape",
        ));
    };
    let ForwardSleepFrameMatch::Provider { instruction_index } =
        forward_sleep_reverse_lookup(admission.frame_installation_summary(), wake_raise)
    else {
        return Err(closed_driver_error(
            "forward_sleep wake Raise did not select an authorized provider frame",
        ));
    };
    if !exact_identity_matches_cps(admission.wake_operation(), wake_raise)
        || !matches!(wake_args.as_slice(), [CpsAtom::Var(value)] if value == parameter)
        || !matches!(wake_resume, ContRef::Var(resume) if resume == &clause.resume)
    {
        return Err(closed_driver_error(
            "forward_sleep wake Raise differs from its sealed checked form",
        ));
    }
    let inner_provider = providers.get(instruction_index).ok_or_else(|| {
        closed_driver_error(
            "forward_sleep selected provider index is outside the sealed frame chain",
        )
    })?;
    Ok(PreparedProductionForwardSleep {
        provider: std::sync::Arc::clone(inner_provider.provider()),
        action: inner_provider.binding().provider_operation().to_string(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ForwardSleepFrameMatch {
    Provider { instruction_index: usize },
    SourceHandler,
    Missing,
}

fn forward_sleep_reverse_lookup(
    instructions: &[FrameInstallationInstructionV1],
    operation: &ash_core::cps::EffectOp,
) -> ForwardSleepFrameMatch {
    instructions
        .iter()
        .enumerate()
        .rev()
        .find_map(|(instruction_index, instruction)| match instruction {
            FrameInstallationInstructionV1::Provider {
                operation: candidate,
                ..
            } if exact_identity_matches_cps(candidate, operation) => {
                Some(ForwardSleepFrameMatch::Provider { instruction_index })
            }
            FrameInstallationInstructionV1::SourceHandler {
                operation: candidate,
                ..
            } if exact_identity_matches_cps(candidate, operation) => {
                Some(ForwardSleepFrameMatch::SourceHandler)
            }
            _ => None,
        })
        .unwrap_or(ForwardSleepFrameMatch::Missing)
}

/// Send-safe data crossing only the admitted `wake` provider await.
pub struct PreparedProductionForwardSleep {
    provider: std::sync::Arc<dyn CapabilityProvider>,
    action: String,
}

impl PreparedProductionForwardSleep {
    pub async fn execute(
        self,
        control: ProductionRunControl,
    ) -> Result<ProductionCheckedCpsOutcome, EngineError> {
        if let Some(outcome) = control.terminal_outcome() {
            return Ok(outcome);
        }
        let args = [EngineValue::Int(0)];
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
        match provider_result {
            EngineValue::Int(value) => {
                Ok(ProductionCheckedCpsOutcome::Return(EngineValue::Int(value)))
            }
            _ => Err(closed_driver_error(
                "forward_sleep wake provider returned a value outside the sealed Int result type",
            )),
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

fn exact_identity_matches_cps(
    identity: &OperationIdentityV1,
    operation: &ash_core::cps::EffectOp,
) -> bool {
    operation.item.namespace == "cap"
        && operation.item.name == format!("{}.{}", identity.impl_type(), identity.operation())
        && operation.arg_types == identity.parameter_types()
        && operation.result_type == identity.result_type()
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
    use super::{
        ForwardSleepFrameMatch, ProductionProviderFrameChain, forward_sleep_reverse_lookup,
    };
    use crate::{
        Engine,
        checked_cps_admission::{
            CheckedCpsProductionAdmission, FrameInstallationInstructionV1, OperationIdentityV1,
            ProviderBindingV1,
        },
        standard_profiles::StandardProviderProfile,
    };
    use ash_core::cps::{EffectItem, EffectItemKind, EffectOp};

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

    #[test]
    fn forward_sleep_lookup_returns_the_innermost_matching_provider_instruction_index() {
        let wake = OperationIdentityV1::new("TestClock", "Clock", "wake", ["Int"], "Int");
        let instructions = vec![
            FrameInstallationInstructionV1::Provider {
                operation: wake.clone(),
                provider_binding: ProviderBindingV1::new(
                    wake.clone(),
                    "task-2014-2005-outer-wake",
                    "wake",
                ),
            },
            FrameInstallationInstructionV1::Provider {
                operation: wake.clone(),
                provider_binding: ProviderBindingV1::new(wake, "task-2014-2005-inner-wake", "wake"),
            },
        ];
        let operation = EffectOp {
            item: EffectItem {
                namespace: "cap".to_string(),
                name: "TestClock.wake".to_string(),
                kind: EffectItemKind::Capability,
            },
            arg_types: vec!["Int".to_string()],
            result_type: "Int".to_string(),
        };

        assert_eq!(
            forward_sleep_reverse_lookup(&instructions, &operation),
            ForwardSleepFrameMatch::Provider {
                instruction_index: 1,
            },
            "TASK-1993 requires the last matching provider instruction to win"
        );
    }
}
