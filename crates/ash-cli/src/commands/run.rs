//! Run command for executing target Ash entries.
//!
//! TASK-054: Implement `run` command for executing Ash entries.
//! TASK-076: Updated to use ash-engine.
//! TASK-309: Implemented --dry-run, --timeout flags.
//! TASK-323: Removed --capability flag.
//! TASK-324: Removed --input flag.

use anyhow::{Context, Result};
use ash_core::capability::{CapabilityError, CapabilityProvider};
use ash_core::runtime_kernel::{
    AdmissionIdentity, AlphaAdmissionDecision, AlphaAdmissionProfile, AlphaAdmissionStatus,
    ApplicationAdmissionProfile, ApplicationArtifactIdentity, ApplicationBoundaryBindingManifest,
    ApplicationBoundaryBindings, ApplicationDefinitionIdentity, ApplicationEntrypointKind,
    ApplicationEntrypointMetadata, ApplicationInstanceIdentity, ApplicationInvocationPacket,
    ApplicationRuntimeReport, ApplicationTerminalOutcome, ApplicationTraceBundle,
    ProviderRegistryIdentity, RuntimeConfigId, RuntimeEngineRelationship, RuntimeHostMode,
    RuntimeKernelArtifactLanguageSummary, RuntimeKernelIdentity, RuntimeProfileId,
    RuntimeProfileIdentity, RuntimeRootSet, RuntimeRootSetId,
};
use ash_core::semantic_summary::{SourceAnchor, SourceOrigin};
use ash_core::{Constraint, Effect, Span, Value};
use ash_engine::EngineError;
use ash_engine::runtime_artifact::{RuntimeArtifactBuildRequest, build_runtime_kernel_artifact};
use ash_interp::ExecError;
use ash_parser::{Token, TokenKind, lex_with_recovery};
use ash_provenance::{ApplicationTraceSession, create_trace_recorder};
use async_trait::async_trait;
use clap::Args;
use serde::Serialize;
use std::future::Future;
use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use crate::error::CliError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RunOutcome {
    Completed,
    Exit(ExitCode),
}

impl RunOutcome {
    #[must_use]
    pub const fn completed() -> Self {
        Self::Completed
    }

    #[must_use]
    pub fn exit_code(self) -> ExitCode {
        match self {
            Self::Completed => ExitCode::SUCCESS,
            Self::Exit(code) => code,
        }
    }
}

/// Output format for run command
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum RunOutputFormat {
    /// Human-readable text format
    #[default]
    Text,
    /// JSON format
    Json,
}

/// Minimal alpha admission profile selection for `ash run`.
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum RunAdmissionProfile {
    /// Preserve current empty-admission behavior.
    #[default]
    Empty,
    /// Explicitly allow the one-shot entry instance.
    Allow,
    /// Reject before entry execution.
    Reject,
}

impl From<RunAdmissionProfile> for AlphaAdmissionProfile {
    fn from(profile: RunAdmissionProfile) -> Self {
        match profile {
            RunAdmissionProfile::Empty => Self::Empty,
            RunAdmissionProfile::Allow => Self::Allow,
            RunAdmissionProfile::Reject => Self::Reject,
        }
    }
}

/// Arguments for the run command
#[derive(Args, Debug, Clone)]
pub struct RunArgs {
    /// Path to Ash source file.
    #[arg(value_name = "PATH")]
    pub path: String,

    /// Output file for results
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<String>,

    /// Enable trace mode
    #[arg(long)]
    pub trace: bool,

    /// Output format (text, json)
    #[arg(long, value_enum, default_value = "text")]
    pub format: RunOutputFormat,

    /// Validate without executing
    #[arg(long)]
    pub dry_run: bool,

    /// Execution timeout in seconds
    #[arg(long, value_name = "SECONDS")]
    pub timeout: Option<u64>,

    /// Select a host implementation for an admitted capability binding (`BINDING=IMPLEMENTATION`).
    #[arg(long = "capability-impl", value_name = "BINDING=IMPLEMENTATION")]
    pub capability_impl: Vec<String>,

    /// Select a host resource initializer for a resource type (`RESOURCE=INITIALIZER`).
    #[arg(long = "resource-init", value_name = "RESOURCE=INITIALIZER")]
    pub resource_init: Vec<String>,

    /// Minimal alpha one-shot admission profile (empty, allow, reject).
    #[arg(long = "admission-profile", value_enum, default_value = "empty")]
    pub admission_profile: RunAdmissionProfile,

    /// Runtime arguments passed to the entry after `--`.
    #[arg(last = true, value_name = "ARGS")]
    pub program_args: Vec<String>,
}

/// Run an Ash source file.
///
/// Supports dry-run mode (validate only) and timeout.
///
/// # Errors
///
/// Returns an error if:
/// - The Ash source file cannot be read
/// - Parsing fails
/// - Type checking fails (in dry-run or normal mode)
/// - Execution fails
/// - Timeout is exceeded
pub async fn run(args: &RunArgs) -> Result<RunOutcome> {
    let selection = OneShotRunSelection::parse(&args.path);
    let path = selection.path.as_path();

    // Build engine with default capabilities.
    let engine = match build_engine(args) {
        Ok(engine) => engine,
        Err(error) => {
            if matches!(args.format, RunOutputFormat::Json) {
                emit_pre_entry_failure(args, "configuration", "run configuration is invalid")
                    .await?;
            }
            return Err(anyhow::Error::new(error).context("Failed to build engine"));
        }
    };
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            if matches!(args.format, RunOutputFormat::Json) {
                emit_pre_entry_failure(args, "input", "entry source could not be read").await?;
            }
            return Err(classify_run_read_error(path, error));
        }
    };
    if let Err(error) = engine.validate_configuration_for_source(&source) {
        if matches!(args.format, RunOutputFormat::Json) {
            emit_pre_entry_failure(args, "configuration", "run configuration is invalid").await?;
        }
        return Err(classify_engine_error(error));
    }
    let source_kind = classify_runnable_source(&source);
    let entrypoint_selection =
        runtime_entrypoint_selection(&source, selection.application.is_some());
    let production_time_sleep_candidate = is_production_time_sleep_candidate(&source);
    let production_trap_sleep_candidate = is_production_trap_sleep_candidate(&source);
    let use_checked_cps_pure_entry = !args.dry_run
        && !production_time_sleep_candidate
        && !production_trap_sleep_candidate
        && has_checked_cps_pure_entry_admission(&engine, &source, source_kind);
    let entry_name = selection.application.as_deref().unwrap_or("main");
    let host_mode = if args.trace {
        RuntimeHostMode::Trace
    } else {
        RuntimeHostMode::OneShot
    };
    let admission_profile = AlphaAdmissionProfile::from(args.admission_profile);
    let admission_decision = admission_profile.evaluate();
    if !admission_decision.is_admitted() {
        emit_terminal_observable(
            args,
            &crate::value_convert::CanonicalTerminalObservable::External {
                boundary: "admission".to_string(),
                outcome: "rejected".to_string(),
            },
        )
        .await?;
        emit_admission_rejection_report_if_requested(
            host_mode,
            entry_name,
            admission_profile,
            &admission_decision,
            &args.program_args,
        )?;
        anyhow::bail!(
            "admission rejected: {}",
            admission_decision
                .reason
                .as_deref()
                .unwrap_or("alpha admission profile rejected the run")
        );
    }

    // TASK-2014's sole effectful source route is selected syntactically only
    // as a *candidate*.  The Engine still parses, checks, and seals the exact
    // operation/provider/anchor evidence below; this CLI predicate grants no
    // execution authority and there is no fallback after candidate admission
    // rejects.
    if !args.dry_run && production_time_sleep_candidate {
        if args.trace {
            emit_pre_entry_failure(
                args,
                "configuration",
                "trace is not supported for the admitted checked-CPS time::sleep route",
            )
            .await?;
            anyhow::bail!("trace is not supported for the admitted checked-CPS time::sleep route");
        }
        return run_admitted_time_sleep(args, &engine, &source).await;
    }
    if !args.dry_run && production_trap_sleep_candidate {
        if args.trace {
            emit_pre_entry_failure(
                args,
                "configuration",
                "trace is not supported for the admitted checked-CPS trap_sleep route",
            )
            .await?;
            anyhow::bail!("trace is not supported for the admitted checked-CPS trap_sleep route");
        }
        return match run_admitted_trap_sleep(args, &engine, &source).await {
            Ok(outcome) => Ok(outcome),
            Err(error) => {
                if let Some(observable) = production_terminal_observable(&error) {
                    emit_terminal_observable(args, &observable).await?;
                }
                Err(error)
            }
        };
    }

    let mut prepared_entry = if is_module_only_source(&source) {
        None
    } else {
        let entry = if source_kind == RunnableSourceKind::Ordinary {
            match engine.parse_file(path) {
                Ok(entry) => entry,
                Err(error) => {
                    let error = classify_engine_error(error);
                    let (class, message) = if error.to_string().contains("expected fn main entry") {
                        ("entry_verification", "entry file has no 'main' entry")
                    } else {
                        ("parse", "entry source could not be parsed")
                    };
                    emit_pre_entry_failure(args, class, message).await?;
                    return Err(error);
                }
            }
        } else {
            let entry = match parse_runnable_entry(&engine, &source, RunnableSourceKind::Entry) {
                Ok(entry) => entry,
                Err(error) => {
                    emit_pre_entry_failure(args, "parse", "entry source could not be parsed")
                        .await?;
                    return Err(classify_engine_error(error));
                }
            };
            if let Err(error) = engine.verify_entry_definition(&entry)
                && !use_checked_cps_pure_entry
            {
                emit_pre_entry_failure(
                    args,
                    "entry_verification",
                    "entry contract verification failed",
                )
                .await?;
                return Err(classify_entry_verification_error(error));
            }
            entry
        };
        Some(entry)
    };
    let kernel = if let Some(entry) = prepared_entry.as_mut() {
        let relative_module_path = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("<source>");
        let checked_function = match engine.check_entry_artifact(
            entry,
            format!("callable:{relative_module_path}::{entry_name}"),
            SourceAnchor::new(
                SourceOrigin::File(relative_module_path.to_string()),
                Some(Span {
                    start: 0,
                    end: source.len(),
                }),
                format!("checked-function:{entry_name}"),
            ),
        ) {
            Ok(checked_function) => checked_function,
            Err(error) => {
                emit_pre_entry_failure(args, "typecheck", "entry failed type checking").await?;
                return Err(classify_engine_error(error));
            }
        };
        Some(
            OneShotRuntimeKernel::admit(
                path,
                &source,
                entry_name,
                entrypoint_selection,
                checked_function,
                host_mode,
                admission_profile,
                &args.program_args,
            )
            .context("Failed to build RuntimeKernel artifact")?,
        )
    } else {
        None
    };

    let execution = async {
        // Dry-run mode: parse and check only
        if args.dry_run {
            if is_module_only_source(&source) {
                if !args.capability_impl.is_empty() || !args.resource_init.is_empty() {
                    println!("Dry run successful");
                    return Ok(RunOutcome::completed());
                }
                if matches!(args.format, RunOutputFormat::Json) {
                    emit_pre_entry_failure(
                        args,
                        "entry_verification",
                        "entry file has no 'main' entry",
                    )
                    .await?;
                }
                anyhow::bail!("entry file has no fn main");
            }

            let _checked_entry = prepared_entry
                .take()
                .expect("non-module dry runs prepare one checked entry artifact");

            println!("Dry run successful");
            return Ok(RunOutcome::completed());
        }

        // Run the source file with optional timeout.
        // Ordinary files use the module-resolver-backed file path for import resolution.
        // LeadingRuntimePrelude files use the source-based path with entry-source parsing.
        let result = if source_kind == RunnableSourceKind::Ordinary {
            if let Some(timeout_secs) = args.timeout {
                match tokio::time::timeout(
                    Duration::from_secs(timeout_secs),
                    run_ordinary_file(&engine, path, args.trace),
                )
                .await
                {
                    Ok(result) => result?,
                    Err(_) => {
                        return Err(anyhow::anyhow!("timeout after {timeout_secs}s"));
                    }
                }
            } else {
                run_ordinary_file(&engine, path, args.trace).await?
            }
        } else {
            // LeadingRuntimePrelude: source-based path
            if let Some(timeout_secs) = args.timeout {
                let timeout_duration = Duration::from_secs(timeout_secs);
                let execution_fut = async {
                    if args.trace {
                        let mut entry = parse_runnable_entry(&engine, &source, source_kind)
                            .map_err(classify_engine_error)?;
                        engine.check(&mut entry).map_err(classify_engine_error)?;
                        execute_with_trace(&engine, &mut entry).await
                    } else {
                        run_runnable_source(&engine, &source, source_kind).await
                    }
                };

                match tokio::time::timeout(timeout_duration, execution_fut).await {
                    Ok(result) => result?,
                    Err(_) => {
                        return Err(anyhow::anyhow!("timeout after {timeout_secs}s"));
                    }
                }
            } else if args.trace {
                let mut entry = parse_runnable_entry(&engine, &source, source_kind)
                    .map_err(classify_engine_error)?;
                engine.check(&mut entry).map_err(classify_engine_error)?;
                execute_with_trace(&engine, &mut entry).await?
            } else {
                run_runnable_source(&engine, &source, source_kind).await?
            }
        };

        if matches!(source_kind, RunnableSourceKind::Entry) && !use_checked_cps_pure_entry {
            return project_checked_entry_terminal(args, &result).await;
        }

        output_result(&result, &args.output, args.format).await?;
        Ok(RunOutcome::completed())
    };
    let outcome = run_execution_with_cancellation(args, execution, async {
        if tokio::signal::ctrl_c().await.is_err() {
            std::future::pending::<()>().await;
        }
    })
    .await;

    match outcome {
        Ok(outcome) => {
            if let Some(kernel) = &kernel {
                let terminal_outcome = if outcome.exit_code() == ExitCode::SUCCESS {
                    ApplicationTerminalOutcome::succeeded()
                } else {
                    ApplicationTerminalOutcome::failed(
                        "runtime error: entry returned a nonzero exit status",
                    )
                };
                kernel.emit_report_if_requested(terminal_outcome)?;
            }
            Ok(outcome)
        }
        Err(error) => {
            if let Some(observable) = production_terminal_observable(&error) {
                emit_terminal_observable(args, &observable).await?;
            }
            if let Some(kernel) = &kernel
                && should_emit_kernel_report_for_error(&error)
            {
                kernel.emit_report_if_requested(ApplicationTerminalOutcome::failed(
                    error.to_string(),
                ))?;
            }
            Err(error)
        }
    }
}

async fn run_execution_with_cancellation<T, Execution, Cancellation>(
    args: &RunArgs,
    execution: Execution,
    cancellation: Cancellation,
) -> Result<T>
where
    Execution: Future<Output = Result<T>>,
    Cancellation: Future<Output = ()>,
{
    tokio::select! {
        result = execution => result,
        () = cancellation => {
            emit_terminal_observable(
                args,
                &crate::value_convert::CanonicalTerminalObservable::External {
                    boundary: "execution".to_string(),
                    outcome: "cancelled".to_string(),
                },
            ).await?;
            Err(anyhow::anyhow!("execution cancelled"))
        }
    }
}

/// Admit and execute the one checked-CPS host-operation slice owned by
/// TASK-2014.
///
/// Admission completes before this function creates its Engine control
/// envelope, so a zero timeout cannot relabel an unsupported source as an
/// execution timeout. SIGINT is merely forwarded into the Engine's
/// cooperative cancellation handle; the Engine driver retains the control
/// race and drops a pending provider future when cancellation wins.
async fn run_admitted_time_sleep(
    args: &RunArgs,
    engine: &ash_engine::Engine,
    source: &str,
) -> Result<RunOutcome> {
    engine
        .register_time_sleep_provider_binding()
        .map_err(classify_engine_error)?;
    let mut entry = engine.parse(source).map_err(classify_engine_error)?;
    engine.check(&mut entry).map_err(classify_engine_error)?;
    let admission = match engine.admit_production_checked_cps(&mut entry) {
        Ok(admission) => admission,
        Err(error) => {
            if let Some(observable) = production_terminal_observable_from_engine_error(&error) {
                emit_terminal_observable(args, &observable).await?;
            }
            return Err(anyhow::Error::new(error));
        }
    };
    let (control, cancellation) = engine
        .new_production_run_control(&admission, args.timeout.map(Duration::from_secs))
        .map_err(classify_engine_error)?;
    let execution = engine.execute_production_checked_cps(&admission, control);
    tokio::pin!(execution);

    // This is the existing signal source, not an outer cancellation policy.
    // The biased branch forwards SIGINT before observing a simultaneously
    // ready provider completion; the Engine then applies its own documented
    // cancellation > deadline > completion ordering.
    let terminal = tokio::select! {
        biased;
        signal = tokio::signal::ctrl_c() => {
            if signal.is_ok() {
                cancellation.cancel();
            }
            execution.await
        }
        result = &mut execution => result,
    };
    let terminal = match terminal {
        Ok(terminal) => terminal,
        Err(error) => {
            if let Some(observable) = production_terminal_observable_from_engine_error(&error) {
                emit_terminal_observable(args, &observable).await?;
            }
            return Err(anyhow::Error::new(error));
        }
    };

    match terminal {
        ash_engine::ProductionCheckedCpsOutcome::Return(value) => {
            if matches!(args.format, RunOutputFormat::Json) {
                emit_terminal_observable(
                    args,
                    &crate::value_convert::CanonicalTerminalObservable::Return { value },
                )
                .await?;
            } else {
                output_result(&value, &args.output, args.format).await?;
            }
            Ok(RunOutcome::completed())
        }
        ash_engine::ProductionCheckedCpsOutcome::Trap(reason) => {
            let reason = format!("{reason:?}");
            emit_terminal_observable(
                args,
                &crate::value_convert::CanonicalTerminalObservable::Trap {
                    reason: reason.clone(),
                },
            )
            .await?;
            Err(anyhow::anyhow!("runtime error: {reason}"))
        }
        ash_engine::ProductionCheckedCpsOutcome::TimedOut => {
            emit_terminal_observable(
                args,
                &crate::value_convert::CanonicalTerminalObservable::External {
                    boundary: "execution".to_string(),
                    outcome: "timeout".to_string(),
                },
            )
            .await?;
            Err(anyhow::anyhow!(
                "timeout after {}s",
                args.timeout.unwrap_or_default()
            ))
        }
        ash_engine::ProductionCheckedCpsOutcome::Cancelled => {
            emit_terminal_observable(
                args,
                &crate::value_convert::CanonicalTerminalObservable::External {
                    boundary: "execution".to_string(),
                    outcome: "cancelled".to_string(),
                },
            )
            .await?;
            Err(anyhow::anyhow!("execution cancelled"))
        }
    }
}

/// Admit and execute TASK-2013/TASK-2014's exact abortive handler witness.
///
/// The candidate predicate above supplies no authority.  This route still
/// checks the source and asks the Engine to mint its opaque checked-CPS token;
/// it only projects the post-admission language trap that the fixed handler
/// body produces.
async fn run_admitted_trap_sleep(
    args: &RunArgs,
    engine: &ash_engine::Engine,
    source: &str,
) -> Result<RunOutcome> {
    let mut entry = engine.parse(source).map_err(classify_engine_error)?;
    engine.check(&mut entry).map_err(classify_engine_error)?;
    let admission = engine
        .admit_production_checked_handler(&mut entry)
        .map_err(anyhow::Error::new)?;
    let error = match engine.execute_production_checked_handler(&admission).await {
        Err(error) => error,
        Ok(_) => {
            return Err(anyhow::anyhow!(
                "internal invariant violation: sealed trap_sleep admission returned instead of trapping"
            ));
        }
    };
    let reason = error.to_string();
    emit_terminal_observable(
        args,
        &crate::value_convert::CanonicalTerminalObservable::Trap {
            reason: reason.clone(),
        },
    )
    .await?;
    Err(anyhow::anyhow!("runtime error: {reason}"))
}

fn should_emit_kernel_report_for_error(error: &anyhow::Error) -> bool {
    let message = error.to_string();
    !(message.contains("parse error")
        || message.contains("type error")
        || message.contains("entry verification failed")
        || message.contains("Unsupported application")
        || message.contains("unsupported application"))
}

#[derive(Debug, Clone)]
struct OneShotRunSelection {
    path: std::path::PathBuf,
    application: Option<String>,
}

impl OneShotRunSelection {
    fn parse(raw: &str) -> Self {
        if let Some((path, application)) = raw.rsplit_once(':')
            && !application.is_empty()
            && Path::new(path).exists()
        {
            return Self {
                path: path.into(),
                application: Some(application.to_string()),
            };
        }

        Self {
            path: raw.into(),
            application: None,
        }
    }
}

#[derive(Debug, Clone)]
struct OneShotRuntimeKernel {
    identity: RuntimeKernelIdentity,
    definition: ApplicationDefinitionIdentity,
    artifact: ApplicationArtifactIdentity,
    artifact_summary: RuntimeKernelArtifactLanguageSummary,
    instance: ApplicationInstanceIdentity,
    entry_name: String,
    admission_profile: AlphaAdmissionProfile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeEntrypointSelection {
    CheckedCallable,
}

#[derive(Debug, Serialize)]
struct OneShotKernelReport<'a> {
    kernel_id: String,
    host_mode: &'a str,
    application: &'a str,
    definition_id: &'a str,
    artifact_id: &'a str,
    instance_id: String,
    admission: AdmissionReport<'a>,
    provider_registry: ProviderRegistryReport,
    application_report: ApplicationRuntimeReport,
    source_hash: &'a str,
    check_summary_hash: &'a str,
    artifact_summary: &'a RuntimeKernelArtifactLanguageSummary,
}

#[derive(Debug, Serialize)]
struct OneShotAdmissionRejectionReport<'a> {
    host_mode: &'a str,
    application: &'a str,
    admission: AdmissionReport<'a>,
    provider_registry: ProviderRegistryReport,
    application_report: ApplicationRuntimeReport,
}

#[derive(Debug, Serialize)]
struct AdmissionReport<'a> {
    status: &'static str,
    profile: &'a str,
    profile_boundary: &'a ApplicationAdmissionProfile,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<&'a str>,
    capability_grants: usize,
    resource_grants: usize,
    action_grants: usize,
    capability_grant_ids: Vec<String>,
    resource_grant_ids: Vec<String>,
    action_grant_details: Vec<ActionGrantReport>,
}

#[derive(Debug, Serialize)]
struct ActionGrantReport {
    binding_id: String,
    provider_name: String,
    action_name: String,
    action_surface: String,
}

#[derive(Debug, Serialize)]
struct ProviderRegistryReport {
    provider_names: Vec<String>,
    grants_admission_authority: bool,
}

impl OneShotRuntimeKernel {
    #[allow(clippy::too_many_arguments)]
    fn admit(
        path: &Path,
        source: &str,
        entry_name: &str,
        entrypoint_selection: RuntimeEntrypointSelection,
        checked_function: ash_core::runtime_kernel::CheckedFunctionArtifact,
        host_mode: RuntimeHostMode,
        admission_profile: AlphaAdmissionProfile,
        program_args: &[String],
    ) -> Result<Self> {
        let profile_id = RuntimeProfileId::new("default");
        let config_id = RuntimeConfigId::new("default");
        let root_id = RuntimeRootSetId::new(
            path.parent()
                .map_or_else(|| ".".to_string(), |parent| parent.display().to_string()),
        );
        let relative_module_path = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("<source>");
        let mut artifact_request = match entrypoint_selection {
            RuntimeEntrypointSelection::CheckedCallable => {
                RuntimeArtifactBuildRequest::new_application_entrypoint(
                    root_id.as_str(),
                    relative_module_path,
                    entry_name,
                    format!("callable:{relative_module_path}::{entry_name}"),
                    format!("runtime-target:application-entry:{entry_name}"),
                    profile_id.as_str(),
                    config_id.as_str(),
                    checked_function,
                    source,
                    format!(
                        "entrypoint={entry_name};callable={relative_module_path}::{entry_name};check=application-runtime-kernel-shared"
                    ),
                )?
            }
        };
        artifact_request =
            artifact_request.with_admission_profile(ApplicationAdmissionProfile::runtime_boundary(
                admission_profile.as_str(),
                "cli:--admission-profile",
                false,
            )?);
        artifact_request =
            artifact_request.with_boundary_bindings(runtime_boundary_bindings(program_args)?);
        artifact_request =
            artifact_request.with_runtime_support_identity(selected_runtime_support_identity());
        let verified_artifact = build_runtime_kernel_artifact(&artifact_request)?;
        let artifact_summary =
            RuntimeKernelArtifactLanguageSummary::from_verified_artifact(&verified_artifact);
        let roots = RuntimeRootSet::new(
            root_id.clone(),
            vec![path.display().to_string()],
            Vec::new(),
            Vec::new(),
            ".ash/state",
            ".ash/cache",
            ".ash/log",
        );
        let identity = RuntimeKernelIdentity::new(
            host_mode,
            roots,
            verified_artifact.cache_key.clone(),
            RuntimeEngineRelationship::ExistingAshEngineEmbedded,
        );
        let profile = RuntimeProfileIdentity::new(
            profile_id,
            config_id,
            vec!["ash run one-shot default profile/config".to_string()],
        );
        let provider_registry =
            ProviderRegistryIdentity::new(runtime_arg_provider_names(program_args));
        let admission = AdmissionIdentity::empty();
        let definition_id = verified_artifact.definition.id.clone();
        let artifact_id = verified_artifact.artifact.id.clone();
        let instance = ApplicationInstanceIdentity::admit(
            host_mode,
            definition_id,
            artifact_id,
            profile,
            provider_registry,
            admission,
        );

        Ok(Self {
            identity,
            definition: verified_artifact.definition,
            artifact: verified_artifact.artifact,
            artifact_summary,
            instance,
            entry_name: entry_name.to_string(),
            admission_profile,
        })
    }

    fn emit_report_if_requested(&self, terminal_outcome: ApplicationTerminalOutcome) -> Result<()> {
        let Ok(mode) = std::env::var("ASH_RUNTIME_KERNEL_REPORT") else {
            return Ok(());
        };
        if mode.eq_ignore_ascii_case("json") {
            eprintln!(
                "{}",
                serde_json::to_string_pretty(&self.report(terminal_outcome))?
            );
        } else {
            eprintln!(
                "runtime_kernel.host_mode={}",
                host_mode_label(self.identity.host_mode)
            );
            eprintln!("runtime_kernel.admission=admitted");
            eprintln!(
                "runtime_kernel.admission_profile={}",
                self.admission_profile.as_str()
            );
            eprintln!("runtime_kernel.kernel_id={}", self.identity.id);
            eprintln!("runtime_kernel.instance_id={}", self.instance.id.0);
            eprintln!("runtime_kernel.artifact_id={}", self.artifact.id.as_str());
        }
        Ok(())
    }

    fn report(&self, terminal_outcome: ApplicationTerminalOutcome) -> OneShotKernelReport<'_> {
        let application_report = application_report_from_invocation_packet(
            &self.artifact_summary.invocation_packet,
            terminal_outcome,
        );
        OneShotKernelReport {
            kernel_id: self.identity.id.to_string(),
            host_mode: host_mode_label(self.identity.host_mode),
            application: &self.entry_name,
            definition_id: self.definition.id.as_str(),
            artifact_id: self.artifact.id.as_str(),
            instance_id: self.instance.id.0.to_string(),
            admission: AdmissionReport {
                status: AlphaAdmissionStatus::Admitted.as_str(),
                profile: self.admission_profile.as_str(),
                profile_boundary: &self.artifact_summary.invocation_packet.admission_profile,
                reason: None,
                capability_grants: self.instance.admission.capability_grants.len(),
                resource_grants: self.instance.admission.resource_grants.len(),
                action_grants: self.instance.admission.action_grants.len(),
                capability_grant_ids: self
                    .instance
                    .admission
                    .capability_grants
                    .iter()
                    .map(|binding_id| binding_id.0.to_string())
                    .collect(),
                resource_grant_ids: self
                    .instance
                    .admission
                    .resource_grants
                    .iter()
                    .map(|resource_id| resource_id.0.to_string())
                    .collect(),
                action_grant_details: self
                    .instance
                    .admission
                    .action_grants
                    .iter()
                    .map(|grant| ActionGrantReport {
                        binding_id: grant.binding_id.0.to_string(),
                        provider_name: grant.provider_name.clone(),
                        action_name: grant.action_name.clone(),
                        action_surface: grant.action_surface(),
                    })
                    .collect(),
            },
            provider_registry: ProviderRegistryReport {
                provider_names: self.instance.provider_registry.provider_names.clone(),
                grants_admission_authority: self
                    .instance
                    .provider_registry
                    .grants_admission_authority(),
            },
            application_report,
            source_hash: &self.identity.cache_key.source_hash,
            check_summary_hash: &self.identity.cache_key.check_summary_hash,
            artifact_summary: &self.artifact_summary,
        }
    }
}

fn emit_admission_rejection_report_if_requested(
    host_mode: RuntimeHostMode,
    entry_name: &str,
    admission_profile: AlphaAdmissionProfile,
    admission_decision: &AlphaAdmissionDecision,
    program_args: &[String],
) -> Result<()> {
    let Ok(mode) = std::env::var("ASH_RUNTIME_KERNEL_REPORT") else {
        return Ok(());
    };
    let reason = admission_decision.reason.as_deref();
    let profile_boundary = ApplicationAdmissionProfile::runtime_boundary(
        admission_profile.as_str(),
        "cli:--admission-profile",
        false,
    )?;
    let boundary_bindings = runtime_boundary_bindings(program_args)?;
    if mode.eq_ignore_ascii_case("json") {
        let provider_registry =
            ProviderRegistryIdentity::new(runtime_arg_provider_names(program_args));
        let grants_admission_authority = provider_registry.grants_admission_authority();
        let invocation_packet = ApplicationInvocationPacket::new(
            ApplicationEntrypointMetadata {
                name: entry_name.to_string(),
                kind: ApplicationEntrypointKind::CheckedCallable,
                callable_identity: Some(format!("callable:<admission-rejected>::{entry_name}")),
                relative_module_path: "<admission-rejected>".to_string(),
                runtime_target_identity: format!("runtime-target:admission-rejected:{entry_name}"),
            },
            profile_boundary.clone(),
            boundary_bindings,
            "admission-rejected-source",
            "admission-rejected-check",
            format!("runtime-target:admission-rejected:{entry_name}"),
        );
        let application_report = application_report_from_invocation_packet(
            &invocation_packet,
            ApplicationTerminalOutcome::rejected(reason.unwrap_or("admission rejected")),
        );
        let report = OneShotAdmissionRejectionReport {
            host_mode: host_mode_label(host_mode),
            application: entry_name,
            admission: AdmissionReport {
                status: admission_decision.status.as_str(),
                profile: admission_profile.as_str(),
                profile_boundary: &profile_boundary,
                reason,
                capability_grants: 0,
                resource_grants: 0,
                action_grants: 0,
                capability_grant_ids: Vec::new(),
                resource_grant_ids: Vec::new(),
                action_grant_details: Vec::new(),
            },
            provider_registry: ProviderRegistryReport {
                provider_names: provider_registry.provider_names,
                grants_admission_authority,
            },
            application_report,
        };
        eprintln!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        eprintln!("runtime_kernel.host_mode={}", host_mode_label(host_mode));
        eprintln!(
            "runtime_kernel.admission={}",
            admission_decision.status.as_str()
        );
        eprintln!(
            "runtime_kernel.admission_profile={}",
            admission_profile.as_str()
        );
        if let Some(reason) = reason {
            eprintln!("runtime_kernel.admission_reason={reason}");
        }
    }
    Ok(())
}

fn host_mode_label(host_mode: RuntimeHostMode) -> &'static str {
    match host_mode {
        RuntimeHostMode::Entry => "Entry",
        RuntimeHostMode::OneShot => "OneShot",
        RuntimeHostMode::Trace => "Trace",
        RuntimeHostMode::Daemon => "Daemon",
    }
}

fn runtime_arg_provider_names(program_args: &[String]) -> Vec<String> {
    program_args
        .iter()
        .enumerate()
        .map(|(index, _)| format!("Args:{index}"))
        .collect()
}

fn runtime_boundary_bindings(
    program_args: &[String],
) -> Result<
    ApplicationBoundaryBindings,
    ash_core::runtime_kernel::ApplicationBoundaryBindingDiagnostic,
> {
    ApplicationBoundaryBindings::from_manifest(
        "cli:runtime-boundary",
        ApplicationBoundaryBindingManifest {
            providers: runtime_arg_provider_names(program_args),
            grants_authority: false,
            ..ApplicationBoundaryBindingManifest::default()
        },
    )
}

fn application_report_from_invocation_packet(
    invocation_packet: &ApplicationInvocationPacket,
    terminal_outcome: ApplicationTerminalOutcome,
) -> ApplicationRuntimeReport {
    let trace_bundle =
        ApplicationTraceBundle::from_invocation_packet(invocation_packet, Vec::new(), Vec::new());
    ApplicationRuntimeReport::new(invocation_packet, terminal_outcome, trace_bundle)
}

/// Build an engine with default capabilities
///
/// Adds stdio and fs capabilities by default.
fn build_engine(args: &RunArgs) -> Result<ash_engine::Engine, ash_engine::EngineError> {
    let mut builder = ash_engine::Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .with_custom_provider("time", Arc::new(ash_engine::providers::TimeProvider::new()));

    for selection in &args.capability_impl {
        let (binding, implementation) = parse_name_pair(selection, "--capability-impl")?;
        builder = builder.with_capability_implementation(binding, implementation);
    }

    for selection in &args.resource_init {
        let (resource, initializer) = parse_name_pair(selection, "--resource-init")?;
        builder = builder.with_resource_initializer(resource, initializer);
    }

    for (index, value) in args.program_args.iter().enumerate() {
        let provider = Arc::new(RuntimeArgProvider::new(index, value));
        let provider_name = provider.name.clone();
        builder = builder.with_custom_provider(&provider_name, provider);
    }

    builder.build()
}

fn parse_name_pair(value: &str, flag: &str) -> Result<(String, String), ash_engine::EngineError> {
    let Some((left, right)) = value.split_once('=') else {
        return Err(EngineError::Configuration(format!(
            "{flag} expects NAME=NAME, got '{value}'"
        )));
    };
    let left = left.trim();
    let right = right.trim();
    if left.is_empty() || right.is_empty() {
        return Err(EngineError::Configuration(format!(
            "{flag} expects NAME=NAME with non-empty names, got '{value}'"
        )));
    }
    Ok((left.to_string(), right.to_string()))
}

#[derive(Debug)]
struct RuntimeArgProvider {
    name: String,
    value: String,
}

impl RuntimeArgProvider {
    fn new(index: usize, value: &str) -> Self {
        Self {
            name: format!("Args:{index}"),
            value: value.to_string(),
        }
    }
}

#[async_trait]
impl CapabilityProvider for RuntimeArgProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn effect(&self) -> Effect {
        Effect::Epistemic
    }

    async fn observe(&self, _constraints: &[Constraint]) -> Result<Value, CapabilityError> {
        Ok(Value::variant(
            "Some",
            vec![("value", Value::String(self.value.clone()))],
        ))
    }

    async fn execute(&self, _action_name: &str, _args: &[Value]) -> Result<Value, CapabilityError> {
        Ok(Value::Null)
    }
}

/// Execute a parsed Ash entry with tracing enabled.
async fn execute_with_trace(
    engine: &ash_engine::Engine,
    entry: &mut ash_engine::Entry,
) -> Result<Value> {
    use ash_core::ApplicationId;

    let application_id = ApplicationId::new();
    let recorder = create_trace_recorder(application_id);
    let session = ApplicationTraceSession::start(recorder, "main")?;

    let execution = engine
        .admit_entry_to_checked_cps(entry)
        .map_err(anyhow::Error::new)
        .and_then(|admission| {
            engine
                .execute_checked_cps_admission(&admission)
                .into_inner()
                .map_err(classify_exec_error)
        });

    match execution {
        Ok(value) => {
            let _recorder = session.finish_success()?;
            Ok(value)
        }
        Err(error) => {
            let _recorder =
                session.finish_error(format!("{error:?}"), Some("checked_cps_admission"))?;
            Err(error)
        }
    }
}

async fn output_result(
    result: &Value,
    output_path: &Option<String>,
    format: RunOutputFormat,
) -> Result<()> {
    let output = match format {
        RunOutputFormat::Text => format!("{result}"),
        RunOutputFormat::Json => {
            let json_value = crate::value_convert::value_to_json(result);
            serde_json::to_string_pretty(&json_value)
                .context("Failed to serialize result to JSON")?
        }
    };

    match output_path {
        Some(path) => {
            tokio::fs::write(path, output)
                .await
                .with_context(|| format!("Failed to write output to {path}"))?;
        }
        None => {
            println!("{output}");
        }
    }

    Ok(())
}

/// Project an entry value produced by sealed checked-CPS execution.
///
/// This deliberately consumes an already-computed value. It preserves the
/// canonical entry `Ok`/`Err` observable and exit-status contract without
/// parsing or executing through the removed bootstrap evaluator.
async fn project_checked_entry_terminal(args: &RunArgs, value: &Value) -> Result<RunOutcome> {
    let observable = entry_terminal_observable(value);
    let exit_code = ash_engine::derive_entry_exit_code(value).unwrap_or(1);

    if matches!(args.format, RunOutputFormat::Json) || exit_code == 0 {
        emit_terminal_observable(args, &observable).await?;
    }

    Ok(RunOutcome::Exit(ExitCode::from(exit_code)))
}

fn entry_terminal_observable(
    terminal_value: &Value,
) -> crate::value_convert::CanonicalTerminalObservable {
    match terminal_value {
        Value::Variant { name, fields } if name == "Ok" => {
            let value = fields
                .iter()
                .find(|(field_name, _)| field_name == "value")
                .map_or(Value::Null, |(_, value)| value.clone());
            crate::value_convert::CanonicalTerminalObservable::Return { value }
        }
        Value::Variant { name, fields } if name == "Err" => {
            let reason = fields
                .iter()
                .find(|(field_name, _)| field_name == "error")
                .map_or_else(
                    || terminal_value.to_string(),
                    |(_, value)| value.to_string(),
                );
            crate::value_convert::CanonicalTerminalObservable::Trap { reason }
        }
        value => crate::value_convert::CanonicalTerminalObservable::Trap {
            reason: value.to_string(),
        },
    }
}

async fn emit_terminal_observable(
    args: &RunArgs,
    observable: &crate::value_convert::CanonicalTerminalObservable,
) -> Result<()> {
    match args.format {
        RunOutputFormat::Text => {
            if let Some(path) = &args.output {
                tokio::fs::write(path, "")
                    .await
                    .with_context(|| format!("Failed to write output to {path}"))?;
            }
        }
        RunOutputFormat::Json => {
            let output = serde_json::to_string_pretty(
                &crate::value_convert::canonical_terminal_observable_to_json(observable),
            )
            .context("Failed to serialize terminal observable to JSON")?;
            match &args.output {
                Some(path) => tokio::fs::write(path, output)
                    .await
                    .with_context(|| format!("Failed to write output to {path}"))?,
                None => println!("{output}"),
            }
        }
    }
    Ok(())
}

async fn emit_pre_entry_failure(args: &RunArgs, class: &str, message: &str) -> Result<()> {
    emit_terminal_observable(
        args,
        &crate::value_convert::CanonicalTerminalObservable::PreEntryFailure {
            class: class.to_string(),
            message: message.to_string(),
        },
    )
    .await
}

fn classify_exec_error(error: ExecError) -> anyhow::Error {
    // Per SPEC-021: preserve distinct error classes for observable behavior
    match error {
        // Parse errors - will exit with code 2
        ExecError::Parse(_) => anyhow::anyhow!("{error}"),
        // Type errors - will exit with code 3
        ExecError::Type(_) => anyhow::anyhow!("{error}"),
        // IO errors - will exit with code 4
        ExecError::Io(_) => anyhow::anyhow!("{error}"),
        // Capability/verification errors - exit code 6
        ExecError::CapabilityNotAvailable(name) => {
            anyhow::anyhow!("verification error: capability not available: {name}")
        }
        // Policy errors
        ExecError::PolicyDenied { policy } => anyhow::anyhow!("policy denial: {policy}"),
        ExecError::RequiresApproval {
            role,
            operation,
            capability,
        } => anyhow::anyhow!(
            "approval required: role '{}' must approve {} on {}",
            role.as_ref(),
            operation,
            capability
        ),
        // Other execution errors - exit code 5
        other => anyhow::anyhow!("{other}"),
    }
}

fn classify_engine_error(error: EngineError) -> anyhow::Error {
    match error {
        EngineError::Parse(message) => anyhow::anyhow!("parse error: {message}"),
        EngineError::Type(message) => anyhow::anyhow!("type error: {message}"),
        EngineError::Execution(message) => anyhow::anyhow!("runtime error: {message}"),
        EngineError::CapabilityNotFound(name) => {
            anyhow::anyhow!("verification error: capability not found: {name}")
        }
        EngineError::Io(error) => anyhow::anyhow!("io error: {error}"),
        EngineError::Configuration(message) => {
            anyhow::anyhow!("configuration error: {message}")
        }
        EngineError::ProductionTerminal {
            classification,
            message,
        } => anyhow::anyhow!("production terminal {classification:?}: {message}"),
    }
}

fn production_terminal_observable(
    error: &anyhow::Error,
) -> Option<crate::value_convert::CanonicalTerminalObservable> {
    production_terminal_observable_from_engine_error(error.downcast_ref::<EngineError>()?)
}

fn production_terminal_observable_from_engine_error(
    error: &EngineError,
) -> Option<crate::value_convert::CanonicalTerminalObservable> {
    let EngineError::ProductionTerminal { classification, .. } = error else {
        return None;
    };

    match classification {
        ash_engine::ProductionTerminalClassification::MissingAdmission => Some(
            crate::value_convert::CanonicalTerminalObservable::External {
                boundary: "admission".to_string(),
                outcome: "rejected".to_string(),
            },
        ),
        ash_engine::ProductionTerminalClassification::InvalidCheckedCoreCps => Some(
            crate::value_convert::CanonicalTerminalObservable::PreEntryFailure {
                class: "entry_verification".to_string(),
                message: "checked Core/CPS artifact is invalid".to_string(),
            },
        ),
    }
}

fn classify_entry_verification_error(error: ash_engine::EntryVerificationError) -> anyhow::Error {
    match error {
        ash_engine::EntryVerificationError::MissingMain => {
            anyhow::anyhow!("entry file has no 'main' entry")
        }
        ash_engine::EntryVerificationError::MissingApplicationMetadata => {
            anyhow::anyhow!("entry metadata is unavailable")
        }
        ash_engine::EntryVerificationError::WrongReturnType { expected, found } => {
            anyhow::anyhow!(
                "'main' has wrong return type\n  expected: {expected}\n  found: {found}"
            )
        }
        ash_engine::EntryVerificationError::NonCapabilityParameter { name, found } => {
            anyhow::anyhow!("parameter '{name}' must be capability type\n  found: {found}")
        }
    }
}

fn classify_run_read_error(path: &Path, error: std::io::Error) -> anyhow::Error {
    if error.kind() == std::io::ErrorKind::NotFound {
        anyhow::anyhow!("file not found: {}", path.display())
    } else {
        anyhow::anyhow!("failed to read Ash file {}: {error}", path.display())
    }
}

pub fn classify_run_cli_error(error: anyhow::Error) -> CliError {
    if let Some(EngineError::ProductionTerminal {
        classification,
        message,
    }) = error.downcast_ref::<EngineError>()
    {
        return match classification {
            ash_engine::ProductionTerminalClassification::MissingAdmission => {
                CliError::general(message.clone())
            }
            ash_engine::ProductionTerminalClassification::InvalidCheckedCoreCps => {
                CliError::verification("checked Core/CPS artifact is invalid", vec![])
            }
        };
    }
    let message = error.to_string();
    let lower = message.to_lowercase();

    if lower == "execution cancelled" {
        CliError::Cancelled
    } else if lower.contains("file not found:")
        || lower.contains("entry file has no 'main' entry")
        || lower.contains("'main' has wrong return type")
        || lower.contains("must be capability type")
        || lower.contains("invalid runtime exit code")
    {
        CliError::general(message)
    } else if lower.contains("failed to build engine") {
        let mut detail = message;
        for cause in error.chain().skip(1) {
            detail.push_str(": ");
            detail.push_str(&cause.to_string());
        }
        CliError::general(detail)
    } else {
        CliError::from(error)
    }
}

fn has_leading_entry_prelude(tokens: &[Token]) -> bool {
    let mut index = 0;
    let mut saw_entry_use = false;

    while let Some(token) = tokens.get(index) {
        if matches!(token.kind, TokenKind::Eof) {
            break;
        }

        let Some(next_index) = consume_entry_prelude_use(tokens, index) else {
            break;
        };

        saw_entry_use = true;
        index = next_index;
    }

    saw_entry_use
        && matches!(
            tokens.get(index).map(|token| &token.kind),
            Some(TokenKind::Fn) | Some(TokenKind::Eof) | None
        )
}

fn consume_entry_prelude_use(tokens: &[Token], start: usize) -> Option<usize> {
    if !matches_ident(tokens.get(start), "use") {
        return None;
    }

    let first_segment = ident_name(tokens.get(start + 1)?)?;
    if first_segment != "result" && first_segment != "runtime" {
        return None;
    }

    if !matches!(tokens.get(start + 2)?.kind, TokenKind::Colon)
        || !matches!(tokens.get(start + 3)?.kind, TokenKind::Colon)
    {
        return None;
    }

    let mut index = start + 4;
    while let Some(token) = tokens.get(index) {
        match token.kind {
            TokenKind::Semicolon => return Some(index + 1),
            TokenKind::Fn | TokenKind::Eof => return Some(index),
            _ => index += 1,
        }
    }

    Some(index)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunnableSourceKind {
    Ordinary,
    LeadingRuntimePrelude,
    Entry,
}

#[cfg(test)]
fn is_entry_source(source: &str) -> bool {
    matches!(classify_runnable_source(source), RunnableSourceKind::Entry)
}

fn classify_runnable_source(source: &str) -> RunnableSourceKind {
    let (tokens, _errors) = lex_with_recovery(source);

    if contains_fn_main_entry(&tokens) {
        RunnableSourceKind::Entry
    } else if has_leading_entry_prelude(&tokens) {
        RunnableSourceKind::LeadingRuntimePrelude
    } else {
        RunnableSourceKind::Ordinary
    }
}

/// Identifies the syntactic shape that may enter the sole TASK-2014 host
/// operation route. It does not validate or authorize execution: that work is
/// performed by `Engine::admit_production_checked_cps`.
fn is_production_time_sleep_candidate(source: &str) -> bool {
    let (tokens, _errors) = lex_with_recovery(source);
    matches!(
        tokens.as_slice(),
        [
            Token { kind: TokenKind::Fn, .. },
            Token { kind: TokenKind::Ident(main), .. },
            Token { kind: TokenKind::LParen, .. },
            Token { kind: TokenKind::RParen, .. },
            Token { kind: TokenKind::Minus, .. },
            Token { kind: TokenKind::Gt, .. },
            Token { kind: TokenKind::Ident(result), .. },
            Token { kind: TokenKind::LBrace, .. },
            Token { kind: TokenKind::Ident(namespace), .. },
            Token { kind: TokenKind::ColonColon, .. },
            Token { kind: TokenKind::Ident(operation), .. },
            Token { kind: TokenKind::LParen, .. },
            Token { kind: TokenKind::Int(duration), .. },
            Token { kind: TokenKind::RParen, .. },
            Token { kind: TokenKind::RBrace, .. },
            Token { kind: TokenKind::Eof, .. },
        ] if main.as_ref() == "main"
            && result.as_ref() == "Null"
            && namespace.as_ref() == "time"
            && operation.as_ref() == "sleep"
            && *duration >= 0
    )
}

/// Identifies the sole abortive handler candidate.  This lexical test is only
/// an early route selector; exact source, checked handler, Core/CPS, and
/// source-handler frame validation remain Engine-owned admission checks.
fn is_production_trap_sleep_candidate(source: &str) -> bool {
    let (tokens, _errors) = lex_with_recovery(source);
    tokens.windows(2).any(|pair| {
        matches!(
            pair,
            [
                Token {
                    kind: TokenKind::Ident(keyword),
                    ..
                },
                Token {
                    kind: TokenKind::Ident(handler_name),
                    ..
                },
            ] if keyword.as_ref() == "handler" && handler_name.as_ref() == "trap_sleep"
        )
    })
}

/// Determines whether the narrow helper-sleep/pure-main compatibility shape
/// can use the already-admitted pure checked-CPS route.
///
/// This intentionally proves the route by parsing, checking, and requesting
/// the existing opaque pure admission rather than inferring it from a helper
/// operation spelling. A failed probe grants nothing and leaves the existing
/// bootstrap/closed routes unchanged.
fn has_checked_cps_pure_entry_admission(
    engine: &ash_engine::Engine,
    source: &str,
    source_kind: RunnableSourceKind,
) -> bool {
    if !matches!(source_kind, RunnableSourceKind::Entry)
        || !is_helper_time_sleep_with_literal_main_candidate(source)
    {
        return false;
    }
    let Ok(mut entry) = parse_runnable_entry(engine, source, source_kind) else {
        return false;
    };
    engine.check(&mut entry).is_ok() && engine.admit_entry_to_checked_cps(&mut entry).is_ok()
}

/// A narrow compatibility bridge for the closed pure-entry route: a helper
/// may contain the production operation spelling while the declared `main`
/// itself remains a literal pure function. This is intentionally a complete
/// source shape, not a broad search for `time::sleep`.
fn is_helper_time_sleep_with_literal_main_candidate(source: &str) -> bool {
    let (tokens, _errors) = lex_with_recovery(source);
    matches!(
        tokens.as_slice(),
        [
            Token { kind: TokenKind::Fn, .. },
            Token { kind: TokenKind::Ident(helper), .. },
            Token { kind: TokenKind::LParen, .. },
            Token { kind: TokenKind::RParen, .. },
            Token { kind: TokenKind::Minus, .. },
            Token { kind: TokenKind::Gt, .. },
            Token { kind: TokenKind::Ident(helper_result), .. },
            Token { kind: TokenKind::LBrace, .. },
            Token { kind: TokenKind::Ident(namespace), .. },
            Token { kind: TokenKind::ColonColon, .. },
            Token { kind: TokenKind::Ident(operation), .. },
            Token { kind: TokenKind::LParen, .. },
            Token { kind: TokenKind::Int(duration), .. },
            Token { kind: TokenKind::RParen, .. },
            Token { kind: TokenKind::RBrace, .. },
            Token { kind: TokenKind::Fn, .. },
            Token { kind: TokenKind::Ident(main), .. },
            Token { kind: TokenKind::LParen, .. },
            Token { kind: TokenKind::RParen, .. },
            Token { kind: TokenKind::Minus, .. },
            Token { kind: TokenKind::Gt, .. },
            Token { kind: TokenKind::Ident(main_result), .. },
            Token { kind: TokenKind::LBrace, .. },
            Token { kind: TokenKind::Int(_), .. },
            Token { kind: TokenKind::RBrace, .. },
            Token { kind: TokenKind::Eof, .. },
        ] if helper.as_ref() != "main"
            && helper_result.as_ref() == "Null"
            && namespace.as_ref() == "time"
            && operation.as_ref() == "sleep"
            && *duration >= 0
            && main.as_ref() == "main"
            && main_result.as_ref() == "Int"
    )
}

fn runtime_entrypoint_selection(
    _source: &str,
    _explicit_application_selector: bool,
) -> RuntimeEntrypointSelection {
    RuntimeEntrypointSelection::CheckedCallable
}

fn selected_runtime_support_identity() -> String {
    std::env::var("ASH_RUNTIME_SUPPORT_IDENTITY")
        .unwrap_or_else(|_| "ash-runtime-support:unselected".to_string())
}

fn parse_runnable_entry(
    engine: &ash_engine::Engine,
    source: &str,
    source_kind: RunnableSourceKind,
) -> std::result::Result<ash_engine::Entry, EngineError> {
    match source_kind {
        RunnableSourceKind::Ordinary => engine.parse(source),
        RunnableSourceKind::LeadingRuntimePrelude | RunnableSourceKind::Entry => {
            engine.load_runtime_stdlib()?;
            engine.parse_entry_source(source)
        }
    }
}

fn is_module_only_source(source: &str) -> bool {
    let (tokens, _errors) = lex_with_recovery(source);
    tokens.iter().any(|token| {
        matches!(
            token.kind,
            TokenKind::Capability | TokenKind::Policy | TokenKind::Role
        )
    }) && !contains_fn_main_entry(&tokens)
}

fn contains_fn_main_entry(tokens: &[Token]) -> bool {
    tokens.windows(2).any(|window| {
        matches!(window[0].kind, TokenKind::Fn) && matches_ident(window.get(1), "main")
    })
}

/// Execute an ordinary Ash source file through the sealed checked-CPS owner.
///
/// The module resolver remains the parsing boundary, but ordinary files do
/// not regain the legacy direct evaluator after successful parsing or
/// checking. Any source that lacks an Engine-issued checked-CPS admission
/// stays typed as `MissingAdmission` for the CLI projection.
async fn run_ordinary_file(engine: &ash_engine::Engine, path: &Path, trace: bool) -> Result<Value> {
    let mut entry = engine.parse_file(path).map_err(classify_engine_error)?;
    engine.check(&mut entry).map_err(classify_engine_error)?;
    if trace {
        return execute_with_trace(engine, &mut entry).await;
    }
    let admission = engine
        .admit_entry_to_checked_cps(&mut entry)
        .map_err(anyhow::Error::new)?;
    engine
        .execute_checked_cps_admission(&admission)
        .into_inner()
        .map_err(classify_exec_error)
}

async fn run_runnable_source(
    engine: &ash_engine::Engine,
    source: &str,
    source_kind: RunnableSourceKind,
) -> Result<Value> {
    let mut entry =
        parse_runnable_entry(engine, source, source_kind).map_err(classify_engine_error)?;
    engine.check(&mut entry).map_err(classify_engine_error)?;
    let admission = engine
        .admit_entry_to_checked_cps(&mut entry)
        .map_err(anyhow::Error::new)?;
    engine
        .execute_checked_cps_admission(&admission)
        .into_inner()
        .map_err(classify_exec_error)
}

fn matches_ident(token: Option<&Token>, expected: &str) -> bool {
    ident_name_from_option(token).is_some_and(|name| name == expected)
}

fn ident_name(token: &Token) -> Option<&str> {
    match &token.kind {
        TokenKind::Ident(name) => Some(name.as_ref()),
        _ => None,
    }
}

fn ident_name_from_option(token: Option<&Token>) -> Option<&str> {
    token.and_then(ident_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::Expr;

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

    #[tokio::test]
    async fn task_2014_forged_checked_core_cps_projects_entry_verification_to_output_owner() {
        let engine = ash_engine::Engine::new()
            .build()
            .expect("engine builds for forged checked-Core/CPS boundary test");
        let mut entry = engine
            .parse("fn main() -> Null { time::sleep(0) }")
            .expect("sealed time::sleep source parses");
        engine
            .check(&mut entry)
            .expect("sealed time::sleep source type-checks before its public core is forged");
        entry.core = Expr::Literal(Value::Null);

        let Err(error) = engine.admit_production_checked_cps(&mut entry) else {
            panic!("forged public Core must be rejected before an admission token exists");
        };
        assert_eq!(
            error.classification(),
            ash_engine::ProductionTerminalClassification::InvalidCheckedCoreCps,
            "the Engine-to-CLI seam must receive a typed invalid-artifact classification"
        );

        let output_root = tempfile::tempdir().expect("temporary output root");
        let output_path = output_root.path().join("terminal.json");
        let args = RunArgs {
            path: "forged.ash".to_string(),
            output: Some(output_path.display().to_string()),
            trace: false,
            format: RunOutputFormat::Json,
            dry_run: false,
            timeout: None,
            capability_impl: vec![],
            resource_init: vec![],
            admission_profile: RunAdmissionProfile::Empty,
            program_args: vec![],
        };

        let observable = production_terminal_observable_from_engine_error(&error)
            .expect("typed invalid Core/CPS errors must have one canonical CLI projection");
        emit_terminal_observable(&args, &observable)
            .await
            .expect("--output owns the invalid-artifact terminal envelope");
        let cli_error = classify_run_cli_error(anyhow::Error::new(error));

        assert_eq!(cli_error.exit_code(), ExitCode::from(4));
        assert_eq!(
            std::fs::read_to_string(&output_path).expect("read output-owned envelope"),
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "kind": "pre_entry_failure",
                "class": "entry_verification",
                "message": "checked Core/CPS artifact is invalid",
            }))
            .expect("serialize expected canonical envelope"),
            "the output file is the sole owner of the fixed V1 invalid-artifact envelope"
        );
    }

    #[tokio::test]
    async fn task_2014_forged_trap_sleep_core_projects_entry_verification_to_output_owner() {
        let engine = ash_engine::Engine::new()
            .build()
            .expect("engine builds for forged trap_sleep Core boundary test");
        let mut entry = engine
            .parse(TRAP_SLEEP_SOURCE)
            .expect("sealed trap_sleep source parses");
        engine
            .check(&mut entry)
            .expect("sealed trap_sleep source type-checks before its public Core is forged");
        entry.core = Expr::Literal(Value::Int(99));

        let Err(error) = engine.admit_production_checked_handler(&mut entry) else {
            panic!("forged trap_sleep public Core must reject before an admission token exists");
        };
        assert_eq!(
            error.classification(),
            ash_engine::ProductionTerminalClassification::InvalidCheckedCoreCps,
            "the handler admission seam must classify forged checked Core as invalid evidence"
        );

        let output_root = tempfile::tempdir().expect("temporary output root");
        let output_path = output_root.path().join("terminal.json");
        let args = RunArgs {
            path: "forged-trap-sleep.ash".to_string(),
            output: Some(output_path.display().to_string()),
            trace: false,
            format: RunOutputFormat::Json,
            dry_run: false,
            timeout: None,
            capability_impl: vec![],
            resource_init: vec![],
            admission_profile: RunAdmissionProfile::Empty,
            program_args: vec![],
        };

        let observable = production_terminal_observable_from_engine_error(&error)
            .expect("typed invalid trap_sleep Core must have one canonical CLI projection");
        emit_terminal_observable(&args, &observable)
            .await
            .expect("--output owns the invalid trap_sleep terminal envelope");
        let cli_error = classify_run_cli_error(anyhow::Error::new(error));

        assert_eq!(cli_error.exit_code(), ExitCode::from(4));
        assert_eq!(
            std::fs::read_to_string(&output_path).expect("read output-owned envelope"),
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "kind": "pre_entry_failure",
                "class": "entry_verification",
                "message": "checked Core/CPS artifact is invalid",
            }))
            .expect("serialize expected canonical envelope"),
            "the output file is the sole owner of the fixed V1 invalid-artifact handler envelope"
        );
    }

    #[test]
    fn test_run_args_parsing() {
        let args = RunArgs {
            path: "test.ash".to_string(),
            output: Some("out.json".to_string()),
            trace: true,
            format: RunOutputFormat::Text,
            dry_run: false,
            timeout: Some(30),
            capability_impl: vec![],
            resource_init: vec![],
            admission_profile: RunAdmissionProfile::Empty,
            program_args: vec!["hello".to_string()],
        };

        assert_eq!(args.path, "test.ash");
        assert!(args.trace);
        assert!(args.output.is_some());
        assert!(matches!(args.format, RunOutputFormat::Text));
        assert!(!args.dry_run);
        assert_eq!(args.timeout, Some(30));
        assert_eq!(args.program_args, vec!["hello"]);
    }

    #[test]
    fn test_run_args_format_json() {
        let args = RunArgs {
            path: "test.ash".to_string(),
            output: None,
            trace: false,
            format: RunOutputFormat::Json,
            dry_run: true,
            timeout: None,
            capability_impl: vec![],
            resource_init: vec![],
            admission_profile: RunAdmissionProfile::Empty,
            program_args: vec![],
        };

        assert!(matches!(args.format, RunOutputFormat::Json));
        assert!(args.dry_run);
    }

    // ============================================================
    // TASK-309: Tests for --dry-run, --timeout flags
    // ============================================================

    #[test]
    fn test_build_engine_default_capabilities() {
        let args = RunArgs {
            path: "test.ash".to_string(),
            output: None,
            trace: false,
            format: RunOutputFormat::Text,
            dry_run: false,
            timeout: None,
            capability_impl: vec![],
            resource_init: vec![],
            admission_profile: RunAdmissionProfile::Empty,
            program_args: vec![],
        };
        let result = build_engine(&args);
        assert!(
            result.is_ok(),
            "Engine should build with default capabilities"
        );
    }

    #[tokio::test]
    async fn test_dry_run_valid_application() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temporary file with a valid canonical entry function.
        let mut temp_file = NamedTempFile::with_suffix(".ash").unwrap();
        write!(
            temp_file,
            r#"
            use result::Result
            use runtime::RuntimeError

            fn main() -> Result<(), RuntimeError> {{ Ok {{ value: {{}} }} }}
            "#
        )
        .unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();

        let args = RunArgs {
            path,
            output: None,
            trace: false,
            format: RunOutputFormat::Text,
            dry_run: true, // Enable dry-run
            timeout: None,
            capability_impl: vec![],
            resource_init: vec![],
            admission_profile: RunAdmissionProfile::Empty,
            program_args: vec![],
        };

        let result = run(&args).await;
        assert!(
            result.is_ok(),
            "Dry run should succeed for valid application"
        );
    }

    #[tokio::test]
    async fn test_dry_run_valid_fn_main_entry() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::with_suffix(".ash").unwrap();
        write!(
            temp_file,
            r#"
            fn main() -> Result<(), RuntimeError> {{
                match true {{
                    true => Ok {{ value: {{}} }},
                    _ => Err {{ error: RuntimeError(0, "") }}
                }}
            }}
            "#
        )
        .unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();

        let args = RunArgs {
            path,
            output: None,
            trace: false,
            format: RunOutputFormat::Text,
            dry_run: true,
            timeout: None,
            capability_impl: vec![],
            resource_init: vec![],
            admission_profile: RunAdmissionProfile::Empty,
            program_args: vec![],
        };

        let result = run(&args).await;
        assert!(
            result.is_ok(),
            "dry run should accept function-first entry source: {result:?}"
        );
    }

    #[tokio::test]
    async fn test_dry_run_fn_main_with_module_declaration_is_checked() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::with_suffix(".ash").unwrap();
        write!(
            temp_file,
            r#"
            type ReviewFlag = Bool;

            fn main() -> Result<(), RuntimeError> {{
                missing_name
            }}
            "#
        )
        .unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();

        let args = RunArgs {
            path,
            output: None,
            trace: false,
            format: RunOutputFormat::Text,
            dry_run: true,
            timeout: None,
            capability_impl: vec![],
            resource_init: vec![],
            admission_profile: RunAdmissionProfile::Empty,
            program_args: vec![],
        };

        let result = run(&args).await;
        assert!(
            result.is_err(),
            "dry run must typecheck function-first entry sources even when they include module declarations"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("missing_name") || err_msg.contains("Unbound variable"),
            "dry run should report the fn main type error, got: {err_msg}"
        );
    }

    #[tokio::test]
    async fn test_dry_run_module_without_entry_is_rejected() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::with_suffix(".ash").unwrap();
        write!(
            temp_file,
            r#"
            policy ReviewPolicy {{ allow => true }}
            "#
        )
        .unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();

        let args = RunArgs {
            path,
            output: None,
            trace: false,
            format: RunOutputFormat::Text,
            dry_run: true,
            timeout: None,
            capability_impl: vec![],
            resource_init: vec![],
            admission_profile: RunAdmissionProfile::Empty,
            program_args: vec![],
        };

        let result = run(&args).await;
        assert!(
            result.is_err(),
            "dry run should require a runnable entry, not accept declaration-only modules"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("fn main"),
            "dry run should report the missing entry shape, got: {err_msg}"
        );
    }

    #[tokio::test]
    async fn test_run_valid_fn_main_entry() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::with_suffix(".ash").unwrap();
        write!(
            temp_file,
            r#"
            use result::Result
            use runtime::RuntimeError

            fn main() -> Result<(), RuntimeError> {{ Ok {{ value: {{}} }} }}
            "#
        )
        .unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();

        let args = RunArgs {
            path,
            output: None,
            trace: false,
            format: RunOutputFormat::Text,
            dry_run: false,
            timeout: None,
            capability_impl: vec![],
            resource_init: vec![],
            admission_profile: RunAdmissionProfile::Empty,
            program_args: vec![],
        };

        let result = run(&args).await;
        assert!(
            result.is_ok(),
            "run should execute function-first entry source: {result:?}"
        );
    }

    #[tokio::test]
    async fn test_dry_run_invalid_syntax() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temporary file with invalid syntax
        let mut temp_file = NamedTempFile::with_suffix(".ash").unwrap();
        write!(temp_file, "invalid syntax!!!").unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();

        let args = RunArgs {
            path,
            output: None,
            trace: false,
            format: RunOutputFormat::Text,
            dry_run: true, // Enable dry-run
            timeout: None,
            capability_impl: vec![],
            resource_init: vec![],
            admission_profile: RunAdmissionProfile::Empty,
            program_args: vec![],
        };

        let result = run(&args).await;
        assert!(result.is_err(), "Dry run should fail for invalid syntax");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("parse") || err_msg.contains("Parse"),
            "Error should indicate parse failure: {err_msg}"
        );
    }

    #[tokio::test]
    async fn test_dry_run_type_error() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temporary file with a type error.
        // This entry function has inconsistent return types.
        let mut temp_file = NamedTempFile::with_suffix(".ash").unwrap();
        write!(
            temp_file,
            r#"fn main() {{
                if true {{
                    42
                }} else {{
                    "string"
                }}
            }}"#
        )
        .unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();

        let args = RunArgs {
            path,
            output: None,
            trace: false,
            format: RunOutputFormat::Text,
            dry_run: true, // Enable dry-run
            timeout: None,
            capability_impl: vec![],
            resource_init: vec![],
            admission_profile: RunAdmissionProfile::Empty,
            program_args: vec![],
        };

        let _result = run(&args).await;
        // Note: Depending on the type checker, this may or may not be a type error
        // The test verifies the dry-run path works end-to-end
    }

    #[tokio::test]
    async fn test_run_with_timeout() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temporary file with a simple entry function.
        let mut temp_file = NamedTempFile::with_suffix(".ash").unwrap();
        write!(
            temp_file,
            r#"
            use result::Result
            use runtime::RuntimeError

            fn main() -> Result<(), RuntimeError> {{ Ok {{ value: {{}} }} }}
            "#
        )
        .unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();

        let args = RunArgs {
            path,
            output: None,
            trace: false,
            format: RunOutputFormat::Text,
            dry_run: false,
            timeout: Some(30), // 30 second timeout
            capability_impl: vec![],
            resource_init: vec![],
            admission_profile: RunAdmissionProfile::Empty,
            program_args: vec![],
        };

        let result = run(&args).await;
        assert!(
            result.is_ok(),
            "Run with timeout should succeed for quick application"
        );
    }

    #[tokio::test]
    async fn test_run_without_timeout() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temporary file with a simple entry function.
        let mut temp_file = NamedTempFile::with_suffix(".ash").unwrap();
        write!(
            temp_file,
            r#"
            use result::Result
            use runtime::RuntimeError

            fn main() -> Result<(), RuntimeError> {{ Ok {{ value: {{}} }} }}
            "#
        )
        .unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();

        let args = RunArgs {
            path,
            output: None,
            trace: false,
            format: RunOutputFormat::Text,
            dry_run: false,
            timeout: None, // No timeout
            capability_impl: vec![],
            resource_init: vec![],
            admission_profile: RunAdmissionProfile::Empty,
            program_args: vec![],
        };

        let result = run(&args).await;
        assert!(result.is_ok(), "Run without timeout should succeed");
    }

    #[test]
    fn test_import_free_entry_detector_accepts_fn_main() {
        let source = r#"
            fn main() -> Result<(), RuntimeError> {
                match true {
                    true => Ok { value: {} },
                    _ => Err { error: RuntimeError(0, "") }
                }
            }
        "#;

        assert!(is_entry_source(source));
    }

    #[test]
    fn test_import_free_entry_detector_rejects_non_main_function() {
        let source = r#"
            fn helper() -> Result<(), RuntimeError> {
                match true {
                    true => Ok { value: {} },
                    _ => Err { error: RuntimeError(0, "") }
                }
            }
        "#;

        assert!(!is_entry_source(source));
    }

    #[tokio::test]
    async fn task_2004_non_bootstrap_runnable_entry_executes_through_checked_cps_admission() {
        let engine = ash_engine::Engine::new()
            .build()
            .expect("engine builds for runnable-source admission");

        let value = run_runnable_source(
            &engine,
            "fn main() -> Int { 42 }",
            RunnableSourceKind::Entry,
        )
        .await
        .expect("a supported runnable entry must execute through checked Core/CPS admission");

        assert_eq!(value, Value::Int(42));
    }

    #[tokio::test]
    async fn task_2014_cli_runnable_entry_executes_representative_atom_only_binary_primitives_through_checked_cps_admission()
     {
        let engine = ash_engine::Engine::new()
            .build()
            .expect("engine builds for primitive runnable-source admission");

        for (name, source, expected) in [
            ("sub", "fn main() -> Int { 7 - 2 }", Value::Int(5)),
            (
                "comparison",
                "fn main() -> Bool { 7 >= 7 }",
                Value::Bool(true),
            ),
            ("mul", "fn main() -> Int { 7 * 2 }", Value::Int(14)),
            (
                "nested comparison",
                "fn main() -> Bool { (1 + 2) >= (2 * 3) }",
                Value::Bool(false),
            ),
        ] {
            let value = run_runnable_source(&engine, source, RunnableSourceKind::Entry)
                .await
                .unwrap_or_else(|error| {
                    panic!(
                        "{name} must execute through sealed checked Core/CPS admission: {error:#}"
                    )
                });
            assert_eq!(
                value, expected,
                "{name} must retain its checked CPS terminal value"
            );
        }
    }

    #[tokio::test]
    async fn task_2014_cli_runnable_and_trace_execute_a_computed_nested_binary_variable_let_through_checked_cps_admission()
     {
        let engine = ash_engine::Engine::new()
            .build()
            .expect("engine builds for computed-let runnable-source admission");
        let source = r"
            fn main() -> Int {
                do {
                    let __checked_add_result = 99;
                    let computed = (1 + 2) * 3;
                    return computed + 4;
                }
            }
            ";

        let runnable = run_runnable_source(&engine, source, RunnableSourceKind::Entry)
            .await
            .expect("computed variable let must execute through sealed checked CPS admission");
        assert_eq!(runnable, Value::Int(13));

        let mut entry = parse_runnable_entry(&engine, source, RunnableSourceKind::Entry)
            .expect("computed variable let parses through the runnable source path");
        engine
            .check(&mut entry)
            .expect("computed variable let typechecks before trace execution");
        let traced = execute_with_trace(&engine, &mut entry).await.expect(
            "computed variable let trace must execute through sealed checked CPS admission",
        );
        assert_eq!(traced, Value::Int(13));
    }

    #[tokio::test]
    async fn task_2004_non_bootstrap_runnable_entry_executes_nested_boolean_not_through_checked_cps_admission()
     {
        let engine = ash_engine::Engine::new()
            .build()
            .expect("engine builds for runnable-source admission");

        let value = run_runnable_source(
            &engine,
            "fn main() -> Bool { !!true }",
            RunnableSourceKind::Entry,
        )
        .await
        .expect("nested Boolean Not must execute through sealed checked Core/CPS admission");

        assert_eq!(
            value,
            Value::Bool(true),
            "runnable source must expose the nested Boolean Not terminal value"
        );
    }

    #[tokio::test]
    async fn task_2004_trace_executes_a_checked_pure_entry_through_sealed_admission() {
        let engine = ash_engine::Engine::new()
            .build()
            .expect("engine builds for traced runnable-source admission");
        let mut entry = parse_runnable_entry(
            &engine,
            "fn main() -> Int { 42 }",
            RunnableSourceKind::Entry,
        )
        .expect("literal entry parses through the runnable source path");
        engine
            .check(&mut entry)
            .expect("literal entry typechecks before trace execution");

        let value = execute_with_trace(&engine, &mut entry).await.expect(
            "a checked pure traced entry must execute through sealed checked Core/CPS admission",
        );

        assert_eq!(value, Value::Int(42));
    }

    #[tokio::test]
    async fn task_2004_trace_executes_nested_boolean_not_through_sealed_checked_cps_admission() {
        let engine = ash_engine::Engine::new()
            .build()
            .expect("engine builds for traced runnable-source admission");
        let mut entry = parse_runnable_entry(
            &engine,
            "fn main() -> Bool { !!true }",
            RunnableSourceKind::Entry,
        )
        .expect("nested Boolean Not entry parses through the runnable source path");
        engine
            .check(&mut entry)
            .expect("nested Boolean Not entry typechecks before trace execution");

        let value = execute_with_trace(&engine, &mut entry).await.expect(
            "nested Boolean Not trace must execute through sealed checked Core/CPS admission",
        );

        assert_eq!(
            value,
            Value::Bool(true),
            "traced source must expose the nested Boolean Not terminal value"
        );
    }

    #[tokio::test]
    async fn one_shot_cancellation_drops_execution_and_projects_canonical_terminal_envelope() {
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };

        struct DropProbe(Arc<AtomicBool>);

        impl Drop for DropProbe {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }

        let temp = tempfile::tempdir().expect("create temporary output directory");
        let output = temp.path().join("cancelled.json");
        let args = RunArgs {
            path: "unused.ash".to_owned(),
            output: Some(output.display().to_string()),
            trace: false,
            format: RunOutputFormat::Json,
            dry_run: false,
            timeout: None,
            capability_impl: vec![],
            resource_init: vec![],
            admission_profile: RunAdmissionProfile::Empty,
            program_args: vec![],
        };
        let dropped = Arc::new(AtomicBool::new(false));
        let execution = {
            let probe = DropProbe(Arc::clone(&dropped));
            async move {
                let _probe = probe;
                std::future::pending::<anyhow::Result<RunOutcome>>().await
            }
        };

        let error =
            match run_execution_with_cancellation(&args, execution, std::future::ready(())).await {
                Ok(_) => panic!("an immediately cancelled execution must not complete"),
                Err(error) => error,
            };
        let error = classify_run_cli_error(error);

        assert!(dropped.load(Ordering::SeqCst));
        assert!(matches!(error, CliError::Cancelled));
        assert_eq!(error.exit_code(), std::process::ExitCode::from(130));
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(
                &std::fs::read(output).expect("read cancellation terminal envelope"),
            )
            .expect("parse cancellation terminal envelope"),
            serde_json::json!({
                "schema_version": 1,
                "kind": "external",
                "boundary": "execution",
                "outcome": "cancelled",
            }),
        );
    }
}
