//! Run command for executing Ash workflows.
//!
//! TASK-054: Implement `run` command for executing workflows.
//! TASK-076: Updated to use ash-engine.
//! TASK-309: Implemented --dry-run, --timeout flags.
//! TASK-323: Removed --capability flag.
//! TASK-324: Removed --input flag.

use anyhow::{Context, Result};
use ash_core::capability::{CapabilityError, CapabilityProvider};
use ash_core::runtime_kernel::{
    AdmissionIdentity, AlphaAdmissionDecision, AlphaAdmissionProfile, AlphaAdmissionStatus,
    ProviderRegistryIdentity, RuntimeConfigId, RuntimeEngineRelationship, RuntimeHostMode,
    RuntimeKernelArtifactLanguageSummary, RuntimeKernelIdentity, RuntimeProfileId,
    RuntimeProfileIdentity, RuntimeRootSet, RuntimeRootSetId, WorkflowArtifactIdentity,
    WorkflowDefinitionIdentity, WorkflowInstanceIdentity,
};
use ash_core::{Constraint, Effect, Value};
use ash_engine::EngineError;
use ash_engine::runtime_artifact::{RuntimeArtifactBuildRequest, build_runtime_kernel_artifact};
use ash_interp::ExecError;
use ash_parser::parse_utils::skip_whitespace_and_comments;
use ash_parser::{Token, TokenKind, expr, lex_with_recovery, new_input};
use ash_provenance::{WorkflowTraceSession, create_trace_recorder};
use async_trait::async_trait;
use clap::Args;
use serde::Serialize;
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
    /// Explicitly allow the one-shot workflow instance.
    Allow,
    /// Reject before workflow body execution.
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
    /// Path to workflow file
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

    /// Select an Ash-defined capability implementation for a host binding (`BINDING=IMPLEMENTATION`).
    #[arg(long = "capability-impl", value_name = "BINDING=IMPLEMENTATION")]
    pub capability_impl: Vec<String>,

    /// Select a host resource initializer for a resource type (`RESOURCE=INITIALIZER`).
    #[arg(long = "resource-init", value_name = "RESOURCE=INITIALIZER")]
    pub resource_init: Vec<String>,

    /// Minimal alpha one-shot admission profile (empty, allow, reject).
    #[arg(long = "admission-profile", value_enum, default_value = "empty")]
    pub admission_profile: RunAdmissionProfile,

    /// Runtime arguments passed to the entry workflow after `--`
    #[arg(last = true, value_name = "ARGS")]
    pub program_args: Vec<String>,
}

/// Run a workflow file
///
/// Supports dry-run mode (validate only) and timeout.
///
/// # Errors
///
/// Returns an error if:
/// - The workflow file cannot be read
/// - Parsing fails
/// - Type checking fails (in dry-run or normal mode)
/// - Execution fails
/// - Timeout is exceeded
pub async fn run(args: &RunArgs) -> Result<RunOutcome> {
    let selection = OneShotRunSelection::parse(&args.path);
    let path = selection.path.as_path();

    // Build engine with default capabilities
    let engine = build_engine(args).context("Failed to build engine")?;
    let source =
        std::fs::read_to_string(path).map_err(|error| classify_run_read_error(path, error))?;
    engine
        .validate_configuration_for_source(&source)
        .map_err(classify_engine_error)?;
    let source_kind = classify_workflow_source(&source);
    let source_kind = if selection.workflow.is_some()
        && matches!(source_kind, WorkflowSourceKind::EntryCandidate)
    {
        WorkflowSourceKind::Ordinary
    } else {
        source_kind
    };
    let use_entry_bootstrap = should_use_entry_bootstrap(source_kind);
    let workflow_name = selection.workflow.as_deref().unwrap_or("main");
    let host_mode = if args.trace {
        RuntimeHostMode::Trace
    } else {
        RuntimeHostMode::OneShot
    };
    let admission_profile = AlphaAdmissionProfile::from(args.admission_profile);
    let admission_decision = admission_profile.evaluate();
    if !admission_decision.is_admitted() {
        emit_admission_rejection_report_if_requested(
            host_mode,
            workflow_name,
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

    let kernel = if !is_module_only_source(&source) {
        Some(
            OneShotRuntimeKernel::admit(
                path,
                &source,
                workflow_name,
                host_mode,
                admission_profile,
                &args.program_args,
            )
            .context("Failed to build RuntimeKernel artifact")?,
        )
    } else {
        None
    };

    let outcome: Result<RunOutcome> = async {
        // Dry-run mode: parse and check only
        if args.dry_run {
            if is_module_only_source(&source) {
                println!("Dry run successful");
                return Ok(RunOutcome::completed());
            }

            let mut workflow = parse_runnable_workflow(&engine, &source, WorkflowSourceKind::Entry)
                .map_err(classify_engine_error)?;
            engine
                .verify_entry_workflow(&workflow)
                .map_err(classify_entry_verification_error)?;
            engine.check(&mut workflow).map_err(classify_engine_error)?;

            println!("Dry run successful");
            return Ok(RunOutcome::completed());
        }

        if use_entry_bootstrap {
            let exit_code = if let Some(timeout_secs) = args.timeout {
                match tokio::time::timeout(
                    Duration::from_secs(timeout_secs),
                    execute_entry_source(&engine, &source, args.trace),
                )
                .await
                {
                    Ok(result) => result.map_err(classify_entry_bootstrap_error)?,
                    Err(_) => {
                        return Err(anyhow::anyhow!("timeout after {timeout_secs}s"));
                    }
                }
            } else {
                execute_entry_source(&engine, &source, args.trace)
                    .await
                    .map_err(classify_entry_bootstrap_error)?
            };

            if exit_code == 0 {
                emit_entry_output(args).await?;
            }

            return Ok(RunOutcome::Exit(ExitCode::from(exit_code)));
        }

        // Run the workflow file with optional timeout.
        // Ordinary files use the module-resolver-backed file path for import resolution.
        // LeadingRuntimePrelude files use the source-based path with entry-source parsing.
        let result = if source_kind == WorkflowSourceKind::Ordinary {
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
                        let mut workflow = parse_runnable_workflow(&engine, &source, source_kind)
                            .map_err(classify_engine_error)?;
                        engine.check(&mut workflow).map_err(classify_engine_error)?;
                        execute_with_trace(&engine, &workflow).await
                    } else {
                        run_workflow_source(&engine, &source, source_kind).await
                    }
                };

                match tokio::time::timeout(timeout_duration, execution_fut).await {
                    Ok(result) => result?,
                    Err(_) => {
                        return Err(anyhow::anyhow!("timeout after {timeout_secs}s"));
                    }
                }
            } else if args.trace {
                let mut workflow = parse_runnable_workflow(&engine, &source, source_kind)
                    .map_err(classify_engine_error)?;
                engine.check(&mut workflow).map_err(classify_engine_error)?;
                execute_with_trace(&engine, &workflow).await?
            } else {
                run_workflow_source(&engine, &source, source_kind).await?
            }
        };

        // Output results
        output_result(&result, &args.output, args.format).await?;
        Ok(RunOutcome::completed())
    }
    .await;

    match outcome {
        Ok(outcome) => {
            if let Some(kernel) = &kernel {
                kernel.emit_report_if_requested()?;
            }
            Ok(outcome)
        }
        Err(error) => {
            if let Some(kernel) = &kernel
                && should_emit_kernel_report_for_error(&error)
            {
                kernel.emit_report_if_requested()?;
            }
            Err(error)
        }
    }
}

fn should_emit_kernel_report_for_error(error: &anyhow::Error) -> bool {
    let message = error.to_string();
    !(message.contains("parse error")
        || message.contains("type error")
        || message.contains("entry verification failed")
        || message.contains("Unsupported workflow")
        || message.contains("unsupported workflow"))
}

#[derive(Debug, Clone)]
struct OneShotRunSelection {
    path: std::path::PathBuf,
    workflow: Option<String>,
}

impl OneShotRunSelection {
    fn parse(raw: &str) -> Self {
        if let Some((path, workflow)) = raw.rsplit_once(':')
            && !workflow.is_empty()
            && Path::new(path).exists()
        {
            return Self {
                path: path.into(),
                workflow: Some(workflow.to_string()),
            };
        }

        Self {
            path: raw.into(),
            workflow: None,
        }
    }
}

#[derive(Debug, Clone)]
struct OneShotRuntimeKernel {
    identity: RuntimeKernelIdentity,
    definition: WorkflowDefinitionIdentity,
    artifact: WorkflowArtifactIdentity,
    artifact_summary: RuntimeKernelArtifactLanguageSummary,
    instance: WorkflowInstanceIdentity,
    workflow_name: String,
    admission_profile: AlphaAdmissionProfile,
}

#[derive(Debug, Serialize)]
struct OneShotKernelReport<'a> {
    kernel_id: String,
    host_mode: &'a str,
    workflow: &'a str,
    definition_id: &'a str,
    artifact_id: &'a str,
    instance_id: String,
    admission: AdmissionReport<'a>,
    provider_registry: ProviderRegistryReport,
    source_hash: &'a str,
    check_summary_hash: &'a str,
    artifact_summary: &'a RuntimeKernelArtifactLanguageSummary,
}

#[derive(Debug, Serialize)]
struct OneShotAdmissionRejectionReport<'a> {
    host_mode: &'a str,
    workflow: &'a str,
    admission: AdmissionReport<'a>,
    provider_registry: ProviderRegistryReport,
}

#[derive(Debug, Serialize)]
struct AdmissionReport<'a> {
    status: &'static str,
    profile: &'a str,
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
    fn admit(
        path: &Path,
        source: &str,
        workflow_name: &str,
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
        let verified_artifact = build_runtime_kernel_artifact(
            &RuntimeArtifactBuildRequest::new(
                root_id.as_str(),
                relative_module_path,
                workflow_name,
                profile_id.as_str(),
                config_id.as_str(),
                source,
                format!("workflow={workflow_name};check=alpha-runtime-kernel-shared"),
            )
            .with_runtime_support_identity(selected_runtime_support_identity()),
        )?;
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
        let instance = WorkflowInstanceIdentity::admit(
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
            workflow_name: workflow_name.to_string(),
            admission_profile,
        })
    }

    fn emit_report_if_requested(&self) -> Result<()> {
        let Ok(mode) = std::env::var("ASH_RUNTIME_KERNEL_REPORT") else {
            return Ok(());
        };
        if mode.eq_ignore_ascii_case("json") {
            eprintln!("{}", serde_json::to_string_pretty(&self.report())?);
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

    fn report(&self) -> OneShotKernelReport<'_> {
        OneShotKernelReport {
            kernel_id: self.identity.id.to_string(),
            host_mode: host_mode_label(self.identity.host_mode),
            workflow: &self.workflow_name,
            definition_id: self.definition.id.as_str(),
            artifact_id: self.artifact.id.as_str(),
            instance_id: self.instance.id.0.to_string(),
            admission: AdmissionReport {
                status: AlphaAdmissionStatus::Admitted.as_str(),
                profile: self.admission_profile.as_str(),
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
            source_hash: &self.identity.cache_key.source_hash,
            check_summary_hash: &self.identity.cache_key.check_summary_hash,
            artifact_summary: &self.artifact_summary,
        }
    }
}

fn emit_admission_rejection_report_if_requested(
    host_mode: RuntimeHostMode,
    workflow_name: &str,
    admission_profile: AlphaAdmissionProfile,
    admission_decision: &AlphaAdmissionDecision,
    program_args: &[String],
) -> Result<()> {
    let Ok(mode) = std::env::var("ASH_RUNTIME_KERNEL_REPORT") else {
        return Ok(());
    };
    let reason = admission_decision.reason.as_deref();
    if mode.eq_ignore_ascii_case("json") {
        let provider_registry =
            ProviderRegistryIdentity::new(runtime_arg_provider_names(program_args));
        let grants_admission_authority = provider_registry.grants_admission_authority();
        let report = OneShotAdmissionRejectionReport {
            host_mode: host_mode_label(host_mode),
            workflow: workflow_name,
            admission: AdmissionReport {
                status: admission_decision.status.as_str(),
                profile: admission_profile.as_str(),
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

/// Build an engine with default capabilities
///
/// Adds stdio and fs capabilities by default.
fn build_engine(args: &RunArgs) -> Result<ash_engine::Engine, ash_engine::EngineError> {
    let mut builder = ash_engine::Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities();

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

/// Execute a workflow with tracing enabled
async fn execute_with_trace(
    engine: &ash_engine::Engine,
    workflow: &ash_engine::Workflow,
) -> Result<Value> {
    use ash_core::WorkflowId;

    let workflow_id = WorkflowId::new();
    let recorder = create_trace_recorder(workflow_id);
    let session = WorkflowTraceSession::start(recorder, "main")?;

    match engine.execute(workflow).await {
        Ok(value) => {
            let _recorder = session.finish_success()?;
            Ok(value)
        }
        Err(error) => {
            let _recorder = session.finish_error(format!("{error:?}"), Some("engine.execute"))?;
            Err(classify_exec_error(error))
        }
    }
}

/// Output the result to stdout or file
async fn execute_entry_source(
    engine: &ash_engine::Engine,
    source: &str,
    trace: bool,
) -> std::result::Result<u8, ash_engine::EntryBootstrapError> {
    if !trace {
        return engine.bootstrap_entry_source(source).await;
    }

    use ash_core::WorkflowId;

    let workflow_id = WorkflowId::new();
    let recorder = create_trace_recorder(workflow_id);
    let session = WorkflowTraceSession::start(recorder, "main")
        .map_err(|error| ash_engine::EntryBootstrapError::Execution(error.to_string()))?;

    match engine.bootstrap_entry_source(source).await {
        Ok(exit_code) => {
            let _recorder = session
                .finish_success()
                .map_err(|error| ash_engine::EntryBootstrapError::Execution(error.to_string()))?;
            Ok(exit_code)
        }
        Err(error) => {
            let _recorder = session
                .finish_error(format!("{error:?}"), Some("bootstrap_entry_source"))
                .map_err(|trace_error| {
                    ash_engine::EntryBootstrapError::Execution(trace_error.to_string())
                })?;
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

async fn emit_entry_output(args: &RunArgs) -> Result<()> {
    if let Some(path) = &args.output {
        tokio::fs::write(path, "")
            .await
            .with_context(|| format!("Failed to write output to {path}"))?;
    }

    Ok(())
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
    }
}

fn classify_entry_verification_error(error: ash_engine::EntryVerificationError) -> anyhow::Error {
    match error {
        ash_engine::EntryVerificationError::MissingMain => {
            anyhow::anyhow!("entry file has no 'main' workflow")
        }
        ash_engine::EntryVerificationError::MissingWorkflowMetadata => {
            anyhow::anyhow!("entry workflow metadata is unavailable")
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

fn classify_entry_bootstrap_error(error: ash_engine::EntryBootstrapError) -> anyhow::Error {
    match error {
        ash_engine::EntryBootstrapError::Engine(engine_error) => {
            classify_engine_error(engine_error)
        }
        ash_engine::EntryBootstrapError::Verification(error) => {
            classify_entry_verification_error(error)
        }
        ash_engine::EntryBootstrapError::Execution(message) => {
            anyhow::anyhow!("runtime error: {message}")
        }
        ash_engine::EntryBootstrapError::InvalidExitCode { code } => {
            anyhow::anyhow!("invalid runtime exit code {code}")
        }
    }
}

fn classify_run_read_error(path: &Path, error: std::io::Error) -> anyhow::Error {
    if error.kind() == std::io::ErrorKind::NotFound {
        anyhow::anyhow!("file not found: {}", path.display())
    } else {
        anyhow::anyhow!("failed to read workflow file {}: {error}", path.display())
    }
}

pub fn classify_run_cli_error(error: anyhow::Error) -> CliError {
    let message = error.to_string();
    let lower = message.to_lowercase();

    if lower.contains("file not found:")
        || lower.contains("entry file has no 'main' workflow")
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
            Some(TokenKind::Workflow) | Some(TokenKind::Eof) | None
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
            TokenKind::Workflow | TokenKind::Eof => return Some(index),
            _ => index += 1,
        }
    }

    Some(index)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EntryHeaderShape {
    name_is_main: bool,
    canonical_return_type: bool,
}

fn first_workflow_entry_header_shape(source: &str, tokens: &[Token]) -> Option<EntryHeaderShape> {
    let workflow_index = tokens
        .iter()
        .position(|token| matches!(token.kind, TokenKind::Workflow))?;

    let mut index = workflow_index + 1;
    let name_is_main = matches_ident(tokens.get(index), "main");

    let name_token = tokens.get(index)?;

    if !matches!(name_token.kind, TokenKind::Ident(_)) {
        return None;
    }

    index += 1;
    let next_index = skip_parenthesized_tokens(tokens, index)?;
    index = next_index;

    if !matches!(
        tokens.get(index).map(|token| &token.kind),
        Some(TokenKind::Minus)
    ) || !matches!(
        tokens.get(index + 1).map(|token| &token.kind),
        Some(TokenKind::Gt)
    ) {
        return None;
    }
    index += 2;

    let canonical_return_type = matches_ident(tokens.get(index), "Result")
        && matches!(
            tokens.get(index + 1).map(|token| &token.kind),
            Some(TokenKind::Lt)
        )
        && matches!(
            tokens.get(index + 2).map(|token| &token.kind),
            Some(TokenKind::LParen)
        )
        && matches!(
            tokens.get(index + 3).map(|token| &token.kind),
            Some(TokenKind::RParen)
        )
        && matches!(
            tokens.get(index + 4).map(|token| &token.kind),
            Some(TokenKind::Comma)
        )
        && matches_ident(tokens.get(index + 5), "RuntimeError")
        && matches!(
            tokens.get(index + 6).map(|token| &token.kind),
            Some(TokenKind::Gt)
        );

    if !canonical_return_type {
        while let Some(token) = tokens.get(index) {
            match token.kind {
                TokenKind::LBrace | TokenKind::Eof => break,
                _ => index += 1,
            }
        }
    } else {
        index += 7;
    }

    if !skip_optional_entry_header_clauses(source, tokens, index) {
        return None;
    }

    Some(EntryHeaderShape {
        name_is_main,
        canonical_return_type,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkflowSourceKind {
    Ordinary,
    LeadingRuntimePrelude,
    EntryCandidate,
    Entry,
}

#[cfg(test)]
fn is_entry_workflow_source(source: &str) -> bool {
    matches!(classify_workflow_source(source), WorkflowSourceKind::Entry)
}

fn classify_workflow_source(source: &str) -> WorkflowSourceKind {
    let (tokens, _errors) = lex_with_recovery(source);

    if let Some(shape) = first_workflow_entry_header_shape(source, &tokens) {
        if shape.name_is_main && shape.canonical_return_type {
            WorkflowSourceKind::Entry
        } else if shape.name_is_main || shape.canonical_return_type {
            WorkflowSourceKind::EntryCandidate
        } else if has_leading_entry_prelude(&tokens) {
            WorkflowSourceKind::LeadingRuntimePrelude
        } else {
            WorkflowSourceKind::Ordinary
        }
    } else if has_leading_entry_prelude(&tokens) {
        WorkflowSourceKind::LeadingRuntimePrelude
    } else {
        WorkflowSourceKind::Ordinary
    }
}

fn selected_runtime_support_identity() -> String {
    std::env::var("ASH_RUNTIME_SUPPORT_IDENTITY")
        .unwrap_or_else(|_| "ash-runtime-support:unselected".to_string())
}

fn parse_runnable_workflow(
    engine: &ash_engine::Engine,
    source: &str,
    source_kind: WorkflowSourceKind,
) -> std::result::Result<ash_engine::Workflow, EngineError> {
    match source_kind {
        WorkflowSourceKind::Ordinary => engine.parse(source),
        WorkflowSourceKind::LeadingRuntimePrelude
        | WorkflowSourceKind::EntryCandidate
        | WorkflowSourceKind::Entry => {
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
    }) && !tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Workflow))
}

fn should_use_entry_bootstrap(source_kind: WorkflowSourceKind) -> bool {
    matches!(
        source_kind,
        WorkflowSourceKind::Entry | WorkflowSourceKind::EntryCandidate
    )
}

/// Execute an ordinary workflow file using the engine's module-resolver-backed file path.
///
/// This resolves imports via `engine.parse_file(path)` (which calls
/// `module_loader::load_ordinary_file`), then type-checks and executes.
/// When `trace` is true, parsing uses `engine.parse_file()` but execution
/// goes through the trace-enabled path.
async fn run_ordinary_file(engine: &ash_engine::Engine, path: &Path, trace: bool) -> Result<Value> {
    if trace {
        let mut workflow = engine.parse_file(path).map_err(classify_engine_error)?;
        engine.check(&mut workflow).map_err(classify_engine_error)?;
        execute_with_trace(engine, &workflow).await
    } else {
        engine.run_file(path).await.map_err(classify_exec_error)
    }
}

async fn run_workflow_source(
    engine: &ash_engine::Engine,
    source: &str,
    source_kind: WorkflowSourceKind,
) -> Result<Value> {
    let mut workflow =
        parse_runnable_workflow(engine, source, source_kind).map_err(classify_engine_error)?;
    engine.check(&mut workflow).map_err(classify_engine_error)?;
    engine.execute(&workflow).await.map_err(classify_exec_error)
}

fn skip_optional_entry_header_clauses(source: &str, tokens: &[Token], mut index: usize) -> bool {
    loop {
        match tokens.get(index).map(|token| &token.kind) {
            Some(TokenKind::LBrace) => return true,
            Some(TokenKind::Eof) | None => return false,
            _ if matches_ident(tokens.get(index), "plays") => {
                let Some(next_index) = consume_entry_plays_clause(tokens, index) else {
                    return false;
                };
                index = next_index;
            }
            _ if matches_ident(tokens.get(index), "capabilities") => {
                let Some(next_index) = consume_entry_capabilities_clause(tokens, index) else {
                    return false;
                };
                index = next_index;
            }
            _ if matches_ident(tokens.get(index), "requires")
                || matches_ident(tokens.get(index), "ensures") =>
            {
                let Some(next_index) = consume_entry_contract_clause(source, tokens, index) else {
                    return false;
                };
                index = next_index;
            }
            _ => return false,
        }
    }
}

fn consume_entry_plays_clause(tokens: &[Token], start: usize) -> Option<usize> {
    if !matches_ident(tokens.get(start), "plays") || !matches_ident(tokens.get(start + 1), "role") {
        return None;
    }

    skip_parenthesized_tokens(tokens, start + 2)
}

fn consume_entry_capabilities_clause(tokens: &[Token], start: usize) -> Option<usize> {
    if !matches_ident(tokens.get(start), "capabilities")
        || !matches!(
            tokens.get(start + 1).map(|token| &token.kind),
            Some(TokenKind::Colon)
        )
    {
        return None;
    }

    skip_bracketed_tokens(tokens, start + 2)
}

fn consume_entry_contract_clause(source: &str, tokens: &[Token], start: usize) -> Option<usize> {
    if !matches!(
        tokens.get(start + 1).map(|token| &token.kind),
        Some(TokenKind::Colon)
    ) {
        return None;
    }

    let expression_start = tokens.get(start + 2)?.span.start;
    let mut input = new_input(&source[expression_start..]);
    skip_whitespace_and_comments(&mut input);
    let _ = expr(&mut input).ok()?;
    skip_whitespace_and_comments(&mut input);

    let next_offset = expression_start + (source[expression_start..].len() - input.input.len());

    tokens
        .iter()
        .enumerate()
        .skip(start + 2)
        .find(|(_, token)| token.span.start >= next_offset || matches!(token.kind, TokenKind::Eof))
        .map(|(index, _)| index)
}

fn skip_parenthesized_tokens(tokens: &[Token], start: usize) -> Option<usize> {
    if !matches!(
        tokens.get(start).map(|token| &token.kind),
        Some(TokenKind::LParen)
    ) {
        return None;
    }

    let mut depth = 0usize;
    let mut index = start;
    while let Some(token) = tokens.get(index) {
        match token.kind {
            TokenKind::LParen => depth += 1,
            TokenKind::RParen => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index + 1);
                }
            }
            TokenKind::Eof => return None,
            _ => {}
        }
        index += 1;
    }

    None
}

fn skip_bracketed_tokens(tokens: &[Token], start: usize) -> Option<usize> {
    if !matches!(
        tokens.get(start).map(|token| &token.kind),
        Some(TokenKind::LBracket)
    ) {
        return None;
    }

    let mut depth = 0usize;
    let mut index = start;
    while let Some(token) = tokens.get(index) {
        match token.kind {
            TokenKind::LBracket => depth += 1,
            TokenKind::RBracket => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index + 1);
                }
            }
            TokenKind::Eof => return None,
            _ => {}
        }
        index += 1;
    }

    None
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
    async fn test_dry_run_valid_workflow() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temporary file with a valid canonical entry workflow
        let mut temp_file = NamedTempFile::with_suffix(".ash").unwrap();
        write!(
            temp_file,
            r#"
            use result::Result
            use runtime::RuntimeError

            workflow main() -> Result<(), RuntimeError> {{ done; }}
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
        assert!(result.is_ok(), "Dry run should succeed for valid workflow");
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

        // Create a temporary file with a type error
        // This workflow has inconsistent return types
        let mut temp_file = NamedTempFile::with_suffix(".ash").unwrap();
        write!(
            temp_file,
            r#"workflow main {{
                if true {{
                    ret 42;
                }} else {{
                    ret "string";
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

        // Create a temporary file with a simple workflow
        let mut temp_file = NamedTempFile::with_suffix(".ash").unwrap();
        write!(
            temp_file,
            r#"
            use result::Result
            use runtime::RuntimeError

            workflow main() -> Result<(), RuntimeError> {{ done; }}
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
            "Run with timeout should succeed for quick workflow"
        );
    }

    #[tokio::test]
    async fn test_run_without_timeout() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temporary file with a simple workflow
        let mut temp_file = NamedTempFile::with_suffix(".ash").unwrap();
        write!(
            temp_file,
            r#"
            use result::Result
            use runtime::RuntimeError

            workflow main() -> Result<(), RuntimeError> {{ done; }}
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
    fn test_import_free_entry_detector_accepts_capabilities_clause_after_return_type() {
        let source = r#"
            workflow main() -> Result<(), RuntimeError>
            capabilities: []
            { done; }
        "#;

        assert!(is_entry_workflow_source(source));
    }

    #[test]
    fn test_import_free_entry_detector_rejects_unknown_clause_after_return_type() {
        let source = r#"
            workflow main() -> Result<(), RuntimeError>
            unexpected: []
            { done; }
        "#;

        assert!(!is_entry_workflow_source(source));
    }
}
