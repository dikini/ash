//! Local daemon control plane for the alpha RuntimeKernel host mode.

use anyhow::{Context, Result, anyhow, bail};
use ash_core::runtime::{FailureEntity, OperationalFailure, ProcessId, TowerLevel};
use ash_core::runtime_kernel::{
    AdmissionIdentity, AlphaAdmissionProfile, AlphaAdmissionStatus, ArtifactVersion,
    ProviderRegistryIdentity, RUNTIME_KERNEL_ARTIFACT_VERSION, RuntimeArtifactCacheKey,
    RuntimeConfigId, RuntimeEngineRelationship, RuntimeHostMode,
    RuntimeKernelArtifactLanguageSummary, RuntimeKernelIdentity, RuntimeProfileId,
    RuntimeProfileIdentity, RuntimeRootSet, RuntimeRootSetId, WorkflowArtifactIdentity,
    WorkflowDefinitionIdentity, WorkflowInstanceIdentity,
};
use ash_core::{Expr, Value as AshValue};
use ash_engine::runtime_artifact::{RuntimeArtifactBuildRequest, build_runtime_kernel_artifact};
use ash_interp::{ChildEnvProjection, Context as AshContext, EvalError, ExecError, RuntimeState};
use clap::{Args, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use walkdir::WalkDir;

#[cfg(unix)]
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};

/// Output format for daemon control commands.
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum DaemonOutputFormat {
    /// Human-readable text format.
    #[default]
    Text,
    /// JSON format.
    Json,
}

/// Arguments for the `daemon` command group.
#[derive(Args, Debug, Clone)]
pub struct DaemonArgs {
    /// Daemon control subcommand.
    #[command(subcommand)]
    pub command: DaemonCommand,
}

/// Local daemon control operations.
#[derive(Subcommand, Debug, Clone)]
pub enum DaemonCommand {
    /// Serve the local daemon control socket.
    Serve(DaemonServeArgs),
    /// List indexed definitions and admitted instances.
    List(DaemonSocketArgs),
    /// Start a workflow instance record.
    Start(DaemonStartArgs),
    /// Report one workflow instance status.
    Status(DaemonStatusArgs),
    /// Cancel a non-terminal workflow instance record.
    Cancel(DaemonCancelArgs),
    /// Reload the daemon definition index transactionally.
    Reload(DaemonSocketArgs),
}

/// Arguments for `ash daemon serve`.
#[derive(Args, Debug, Clone)]
pub struct DaemonServeArgs {
    /// Source root to index.
    #[arg(long, value_name = "DIR")]
    pub root: PathBuf,
    /// Unix-domain socket path for local control.
    #[arg(long, value_name = "PATH")]
    pub socket: PathBuf,
    /// Runtime state directory.
    #[arg(long, value_name = "DIR")]
    pub state_dir: PathBuf,
    /// Runtime cache directory.
    #[arg(long, value_name = "DIR")]
    pub cache_dir: PathBuf,
    /// Runtime log directory.
    #[arg(long, value_name = "DIR")]
    pub log_dir: PathBuf,
    /// Output format.
    #[arg(long, value_enum, default_value = "text")]
    pub format: DaemonOutputFormat,
}

/// Shared socket/format arguments for daemon client commands.
#[derive(Args, Debug, Clone)]
pub struct DaemonSocketArgs {
    /// Unix-domain socket path for local control.
    #[arg(long, value_name = "PATH")]
    pub socket: PathBuf,
    /// Output format.
    #[arg(long, value_enum, default_value = "text")]
    pub format: DaemonOutputFormat,
}

/// Arguments for `ash daemon start`.
#[derive(Args, Debug, Clone)]
pub struct DaemonStartArgs {
    /// Unix-domain socket path for local control.
    #[arg(long, value_name = "PATH")]
    pub socket: PathBuf,
    /// Runtime argument passed to the workflow instance record.
    #[arg(long = "arg", value_name = "VALUE")]
    pub start_args: Vec<String>,
    /// Runtime config identity recorded for this workflow instance.
    #[arg(long = "config-id", value_name = "NAME", default_value = "default")]
    pub config_id: String,
    /// Minimal alpha daemon admission profile (empty, allow, reject).
    #[arg(long = "admission-profile", value_enum, default_value = "empty")]
    pub admission_profile: DaemonAdmissionProfile,
    /// Workflow name to admit.
    #[arg(value_name = "WORKFLOW")]
    pub workflow: String,
    /// Output format.
    #[arg(long, value_enum, default_value = "text")]
    pub format: DaemonOutputFormat,
}

/// Arguments for `ash daemon status`.
#[derive(Args, Debug, Clone)]
pub struct DaemonStatusArgs {
    /// Unix-domain socket path for local control.
    #[arg(long, value_name = "PATH")]
    pub socket: PathBuf,
    /// Workflow instance id.
    #[arg(long, value_name = "ID")]
    pub instance: String,
    /// Output format.
    #[arg(long, value_enum, default_value = "text")]
    pub format: DaemonOutputFormat,
}

/// Arguments for `ash daemon cancel`.
#[derive(Args, Debug, Clone)]
pub struct DaemonCancelArgs {
    /// Unix-domain socket path for local control.
    #[arg(long, value_name = "PATH")]
    pub socket: PathBuf,
    /// Workflow instance id.
    #[arg(value_name = "INSTANCE_ID")]
    pub instance_id: String,
    /// Output format.
    #[arg(long, value_enum, default_value = "text")]
    pub format: DaemonOutputFormat,
}

/// Execute an `ash daemon ...` command.
///
/// # Errors
///
/// Returns an error when daemon serving fails, the local socket cannot be
/// reached, or the daemon rejects a request.
pub async fn daemon(args: &DaemonArgs) -> Result<ExitCode> {
    match &args.command {
        DaemonCommand::Serve(args) => serve(args),
        DaemonCommand::List(args) => client_request(
            &args.socket,
            args.format,
            DaemonRequest::List,
            "indexed definitions and instances",
        ),
        DaemonCommand::Start(args) => client_request(
            &args.socket,
            args.format,
            DaemonRequest::Start {
                workflow: args.workflow.clone(),
                args: args.start_args.clone(),
                config_id: args.config_id.clone(),
                admission_profile: args.admission_profile,
                execute: false,
            },
            "workflow instance admitted",
        ),
        DaemonCommand::Status(args) => client_request(
            &args.socket,
            args.format,
            DaemonRequest::Status {
                instance_id: args.instance.clone(),
            },
            "workflow instance status",
        ),
        DaemonCommand::Cancel(args) => client_request(
            &args.socket,
            args.format,
            DaemonRequest::Cancel {
                instance_id: args.instance_id.clone(),
            },
            "workflow instance cancellation",
        ),
        DaemonCommand::Reload(args) => client_request(
            &args.socket,
            args.format,
            DaemonRequest::Reload,
            "definition index reloaded",
        ),
    }
}

/// Minimal alpha admission profile selection for `ash daemon start`.
#[derive(Debug, Clone, Copy, Default, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DaemonAdmissionProfile {
    /// Preserve current empty-admission daemon behavior.
    #[default]
    Empty,
    /// Explicitly allow the workflow instance record.
    Allow,
    /// Reject before the workflow instance is admitted or recorded.
    Reject,
}

impl DaemonAdmissionProfile {
    fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Allow => "allow",
            Self::Reject => "reject",
        }
    }
}

impl From<DaemonAdmissionProfile> for AlphaAdmissionProfile {
    fn from(profile: DaemonAdmissionProfile) -> Self {
        match profile {
            DaemonAdmissionProfile::Empty => Self::Empty,
            DaemonAdmissionProfile::Allow => Self::Allow,
            DaemonAdmissionProfile::Reject => Self::Reject,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
enum DaemonRequest {
    List,
    Start {
        workflow: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default = "default_config_id")]
        config_id: String,
        #[serde(default)]
        admission_profile: DaemonAdmissionProfile,
        #[serde(default)]
        execute: bool,
    },
    Status {
        instance_id: String,
    },
    Cancel {
        instance_id: String,
    },
    Reload,
}

fn default_config_id() -> String {
    "default".to_string()
}

#[derive(Debug, Clone, Serialize)]
struct DefinitionRecord {
    workflow: String,
    relative_module_path: String,
    definition_id: String,
    artifact_id: String,
    artifact_version: String,
    source_hash: String,
    check_summary_hash: String,
    artifact_summary: RuntimeKernelArtifactLanguageSummary,
}

#[derive(Debug, Clone, Serialize)]
struct InstanceRecord {
    instance_id: String,
    workflow: String,
    status: InstanceStatus,
    args: Vec<String>,
    config_id: String,
    admission: InstanceAdmissionRecord,
    definition_id: String,
    artifact_id: String,
    artifact_version: String,
    source_hash: String,
    artifact_summary: RuntimeKernelArtifactLanguageSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    report: Option<InstanceExecutionReport>,
}

#[derive(Debug, Clone, Serialize)]
struct InstanceAdmissionRecord {
    status: String,
    profile: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    capability_grants: usize,
    resource_grants: usize,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum InstanceStatus {
    Admitted,
    Succeeded,
    Failed,
    Cancelled,
}

impl InstanceStatus {
    fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }
}

#[derive(Debug, Clone, Serialize)]
struct InstanceExecutionReport {
    status: String,
    failure: Option<InstanceFailureReport>,
}

#[derive(Debug, Clone, Serialize)]
struct InstanceFailureReport {
    tower: String,
    kind: String,
    host_failure: bool,
    entity: String,
    message: String,
    payload_type: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DaemonExecutionFailureClass {
    ChildProc,
    Workflow,
    Effect,
    Host,
}

struct DaemonState {
    root: PathBuf,
    root_id: RuntimeRootSetId,
    profile_id: RuntimeProfileId,
    config_id: RuntimeConfigId,
    identity: RuntimeKernelIdentity,
    definitions: Vec<DefinitionRecord>,
    instances: BTreeMap<String, InstanceRecord>,
    provider_registry: ProviderRegistryIdentity,
}

impl DaemonState {
    fn new(args: &DaemonServeArgs) -> Result<Self> {
        validate_root(&args.root)?;
        fs::create_dir_all(&args.state_dir)
            .with_context(|| format!("failed to create state dir {}", args.state_dir.display()))?;
        fs::create_dir_all(&args.cache_dir)
            .with_context(|| format!("failed to create cache dir {}", args.cache_dir.display()))?;
        fs::create_dir_all(&args.log_dir)
            .with_context(|| format!("failed to create log dir {}", args.log_dir.display()))?;
        validate_local_control_paths(args)?;

        let root = args
            .root
            .canonicalize()
            .with_context(|| format!("failed to canonicalize root {}", args.root.display()))?;
        let root_id = RuntimeRootSetId::new(root.display().to_string());
        let profile_id = RuntimeProfileId::new("default");
        let config_id = RuntimeConfigId::new("default");
        let definitions = index_definitions(&root, &root_id, &profile_id, &config_id)?;
        let cache_key = definitions.first().map_or_else(
            || {
                RuntimeArtifactCacheKey::new(
                    root_id.clone(),
                    profile_id.clone(),
                    config_id.clone(),
                    "empty-index",
                    "empty-index",
                    ArtifactVersion::new(RUNTIME_KERNEL_ARTIFACT_VERSION),
                )
            },
            |definition| {
                RuntimeArtifactCacheKey::new(
                    root_id.clone(),
                    profile_id.clone(),
                    config_id.clone(),
                    definition.source_hash.clone(),
                    definition.check_summary_hash.clone(),
                    ArtifactVersion::new(definition.artifact_version.clone()),
                )
            },
        );
        let roots = RuntimeRootSet::new(
            root_id.clone(),
            vec![root.display().to_string()],
            Vec::new(),
            Vec::new(),
            args.state_dir.display().to_string(),
            args.cache_dir.display().to_string(),
            args.log_dir.display().to_string(),
        );
        let identity = RuntimeKernelIdentity::new(
            RuntimeHostMode::Daemon,
            roots,
            cache_key,
            RuntimeEngineRelationship::ExistingAshEngineEmbedded,
        );

        Ok(Self {
            root,
            root_id,
            profile_id,
            config_id,
            identity,
            definitions,
            instances: BTreeMap::new(),
            provider_registry: ProviderRegistryIdentity::new(Vec::new()),
        })
    }

    fn response_list(&self) -> Value {
        json!({
            "ok": true,
            "kernel_id": self.identity.id.to_string(),
            "host_mode": host_mode_label(self.identity.host_mode),
            "definitions": self.definitions,
            "instances": self.instances.values().collect::<Vec<_>>(),
            "provider_registry": provider_registry_json(&self.provider_registry),
        })
    }

    fn start(
        &mut self,
        workflow: &str,
        args: &[String],
        config_id: &str,
        admission_profile: DaemonAdmissionProfile,
    ) -> Result<Value> {
        let definition = self
            .definitions
            .iter()
            .find(|definition| definition.workflow == workflow)
            .ok_or_else(|| anyhow!("workflow definition not indexed: {workflow}"))?;

        let alpha_profile = AlphaAdmissionProfile::from(admission_profile);
        let admission_decision = alpha_profile.evaluate();
        if !admission_decision.is_admitted() {
            bail!(
                "admission rejected: {}",
                admission_decision
                    .reason
                    .as_deref()
                    .unwrap_or("alpha admission profile rejected the daemon start")
            );
        }

        let profile = RuntimeProfileIdentity::new(
            self.profile_id.clone(),
            RuntimeConfigId::new(config_id),
            vec![
                "ash daemon alpha default profile".to_string(),
                format!("ash daemon alpha start config_id={config_id}"),
                format!(
                    "ash daemon alpha admission_profile={}",
                    admission_profile.as_str()
                ),
            ],
        );
        let definition_identity = WorkflowDefinitionIdentity::new(
            self.root_id.clone(),
            definition.relative_module_path.clone(),
            definition.workflow.clone(),
            self.profile_id.clone(),
            self.config_id.clone(),
            definition.source_hash.clone(),
        );
        let artifact_identity = WorkflowArtifactIdentity::new(
            definition_identity.id.clone(),
            RuntimeArtifactCacheKey::new(
                self.root_id.clone(),
                self.profile_id.clone(),
                self.config_id.clone(),
                definition.source_hash.clone(),
                definition.check_summary_hash.clone(),
                ArtifactVersion::new(definition.artifact_version.clone()),
            ),
            ArtifactVersion::new(definition.artifact_version.clone()),
        );
        let admission = AdmissionIdentity::empty();
        let admission_record = InstanceAdmissionRecord {
            status: AlphaAdmissionStatus::Admitted.as_str().to_string(),
            profile: admission_profile.as_str().to_string(),
            reason: None,
            capability_grants: admission.capability_grants.len(),
            resource_grants: admission.resource_grants.len(),
        };
        let instance = WorkflowInstanceIdentity::admit(
            RuntimeHostMode::Daemon,
            definition_identity.id,
            artifact_identity.id,
            profile,
            self.provider_registry.clone(),
            admission,
        );
        let instance_id = instance.id.0.to_string();
        let record = InstanceRecord {
            instance_id: instance_id.clone(),
            workflow: definition.workflow.clone(),
            status: InstanceStatus::Admitted,
            args: args.to_vec(),
            config_id: config_id.to_string(),
            admission: admission_record.clone(),
            definition_id: definition.definition_id.clone(),
            artifact_id: definition.artifact_id.clone(),
            artifact_version: definition.artifact_version.clone(),
            source_hash: definition.source_hash.clone(),
            artifact_summary: definition.artifact_summary.clone(),
            class: None,
            report: None,
        };
        self.instances.insert(instance_id.clone(), record);
        Ok(json!({
            "ok": true,
            "host_mode": "Daemon",
            "status": "admitted",
            "execution": "not_started_alpha_record_only",
            "instance_id": instance_id,
            "workflow": workflow,
            "args": args,
            "config_id": config_id,
            "admission": admission_record,
            "definition_id": definition.definition_id,
            "artifact_id": definition.artifact_id,
            "artifact_version": definition.artifact_version,
            "artifact_summary": definition.artifact_summary,
            "provider_registry": provider_registry_json(&self.provider_registry),
        }))
    }

    fn start_and_execute(
        &mut self,
        workflow: &str,
        args: &[String],
        config_id: &str,
        admission_profile: DaemonAdmissionProfile,
    ) -> Result<Value> {
        let start = self.start(workflow, args, config_id, admission_profile)?;
        let instance_id = start["instance_id"]
            .as_str()
            .ok_or_else(|| anyhow!("daemon start response missing instance id"))?
            .to_string();

        let outcome = self.execute_instance(workflow);
        let instance = self
            .instances
            .get_mut(&instance_id)
            .ok_or_else(|| anyhow!("workflow instance not found after start: {instance_id}"))?;
        match outcome {
            Ok(()) => {
                instance.status = InstanceStatus::Succeeded;
                instance.class = Some("workflow_succeeded".to_string());
                instance.report = Some(InstanceExecutionReport {
                    status: "succeeded".to_string(),
                    failure: None,
                });
            }
            Err(failure) => {
                instance.status = InstanceStatus::Failed;
                instance.class = Some(failure.class().to_string());
                instance.report = Some(InstanceExecutionReport {
                    status: "failed".to_string(),
                    failure: Some(failure.report),
                });
            }
        }

        Ok(instance_status_json(instance))
    }

    fn execute_instance(
        &self,
        workflow: &str,
    ) -> std::result::Result<(), Box<InstanceExecutionFailure>> {
        let definition = self
            .definitions
            .iter()
            .find(|definition| definition.workflow == workflow)
            .ok_or_else(|| {
                Box::new(InstanceExecutionFailure::workflow_request(format!(
                    "workflow definition not indexed: {workflow}"
                )))
            })?;
        let path = self.root.join(&definition.relative_module_path);
        std::thread::scope(|scope| {
            let handle = scope.spawn(|| {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|error| {
                        Box::new(InstanceExecutionFailure::host(format!(
                            "daemon execution runtime build failed: {error}"
                        )))
                    })?;

                runtime.block_on(async {
                    let engine = ash_engine::Engine::new().build().map_err(|error| {
                        Box::new(InstanceExecutionFailure::host(format!(
                            "failed to build daemon execution engine: {error}"
                        )))
                    })?;
                    let mut workflow = engine.parse_file(&path).map_err(|error| {
                        Box::new(InstanceExecutionFailure::workflow_request(format!(
                            "failed to parse daemon workflow for execution: {error}"
                        )))
                    })?;
                    engine.check(&mut workflow).map_err(|error| {
                        Box::new(InstanceExecutionFailure::workflow_request(format!(
                            "failed to check daemon workflow for execution: {error}"
                        )))
                    })?;
                    let value = engine
                        .execute(&workflow)
                        .await
                        .map_err(|error| Box::new(InstanceExecutionFailure::from_exec(error)))?;
                    force_returned_proc_if_present(value)
                        .await
                        .map_err(|error| Box::new(InstanceExecutionFailure::from_eval(error)))?;
                    Ok(())
                })
            });
            handle.join().map_err(|_| {
                Box::new(InstanceExecutionFailure::host(
                    "daemon workflow execution worker panicked".to_string(),
                ))
            })?
        })
    }

    fn status(&self, instance_id: &str) -> Result<Value> {
        let instance = self
            .instances
            .get(instance_id)
            .ok_or_else(|| anyhow!("workflow instance not found: {instance_id}"))?;
        Ok(json!({
            "ok": true,
            "host_mode": "Daemon",
            "instance_id": instance.instance_id,
            "workflow": instance.workflow,
            "status": instance.status,
            "class": instance.class,
            "report": instance.report,
            "args": instance.args,
            "config_id": instance.config_id,
            "admission": instance.admission,
            "definition_id": instance.definition_id,
            "artifact_id": instance.artifact_id,
            "artifact_version": instance.artifact_version,
            "source_hash": instance.source_hash,
            "artifact_summary": instance.artifact_summary,
        }))
    }

    fn cancel(&mut self, instance_id: &str) -> Result<Value> {
        let instance = self
            .instances
            .get_mut(instance_id)
            .ok_or_else(|| anyhow!("workflow instance not found: {instance_id}"))?;
        let class = if instance.status.is_terminal() {
            "already_terminal"
        } else {
            instance.status = InstanceStatus::Cancelled;
            "cancelled"
        };
        Ok(json!({
            "ok": true,
            "host_mode": "Daemon",
            "instance_id": instance.instance_id,
            "status": instance.status,
            "class": class,
        }))
    }

    fn reload(&mut self) -> Result<Value> {
        let staged =
            index_definitions(&self.root, &self.root_id, &self.profile_id, &self.config_id)?;
        let count = staged.len();
        self.definitions = staged;
        if let Some(definition) = self.definitions.first() {
            self.identity.cache_key = RuntimeArtifactCacheKey::new(
                self.root_id.clone(),
                self.profile_id.clone(),
                self.config_id.clone(),
                definition.source_hash.clone(),
                definition.check_summary_hash.clone(),
                ArtifactVersion::new(definition.artifact_version.clone()),
            );
        }
        Ok(json!({
            "ok": true,
            "host_mode": "Daemon",
            "status": "reloaded",
            "definition_count": count,
            "definitions": self.definitions,
        }))
    }
}

struct InstanceExecutionFailure {
    report: InstanceFailureReport,
}

impl InstanceExecutionFailure {
    fn workflow_request(message: String) -> Self {
        Self {
            report: InstanceFailureReport {
                tower: "Workflow".to_string(),
                kind: "workflow_execution_failure".to_string(),
                host_failure: false,
                entity: "daemon_instance".to_string(),
                message,
                payload_type: None,
            },
        }
    }

    fn host(message: String) -> Self {
        Self {
            report: InstanceFailureReport {
                tower: "DaemonHost".to_string(),
                kind: "daemon_execution_host_failure".to_string(),
                host_failure: true,
                entity: "daemon_host".to_string(),
                message,
                payload_type: None,
            },
        }
    }

    fn from_exec(error: ExecError) -> Self {
        match error {
            ExecError::Eval(EvalError::OperationalFailure(failure)) => {
                Self::from_operational_failure(&failure)
            }
            ExecError::Eval(error) => Self::workflow_request(error.to_string()),
            error => Self::host(error.to_string()),
        }
    }

    fn from_eval(error: EvalError) -> Self {
        match error {
            EvalError::OperationalFailure(failure) => Self::from_operational_failure(&failure),
            error => Self::workflow_request(error.to_string()),
        }
    }

    fn from_operational_failure(failure: &OperationalFailure) -> Self {
        let classification = classify_operational_failure(failure);
        let kind = match classification {
            DaemonExecutionFailureClass::ChildProc => "child_proc_failure",
            DaemonExecutionFailureClass::Workflow => "workflow_failure",
            DaemonExecutionFailureClass::Effect => "effect_failure",
            DaemonExecutionFailureClass::Host => "daemon_execution_host_failure",
        };
        let (tower, entity) = report_attribution(failure, classification);
        Self {
            report: InstanceFailureReport {
                tower: tower_label(tower).to_string(),
                kind: kind.to_string(),
                host_failure: classification == DaemonExecutionFailureClass::Host,
                entity,
                message: operational_failure_message(failure),
                payload_type: Some(failure.payload_type.clone()),
            },
        }
    }

    fn class(&self) -> &str {
        match self.report.kind.as_str() {
            "child_proc_failure" => "workflow_child_failure",
            "workflow_failure" => "workflow_failure",
            "effect_failure" => "workflow_child_failure",
            "daemon_execution_host_failure" => "daemon_host_failure",
            _ => "workflow_failure",
        }
    }
}

fn classify_operational_failure(failure: &OperationalFailure) -> DaemonExecutionFailureClass {
    if find_proc_failure(failure).is_some() {
        return DaemonExecutionFailureClass::ChildProc;
    }
    match (failure.tower, failure.entity) {
        (TowerLevel::Workflow, FailureEntity::Workflow(_)) => DaemonExecutionFailureClass::Workflow,
        (TowerLevel::Effectful, FailureEntity::EffectScope(_)) => {
            DaemonExecutionFailureClass::Effect
        }
        _ => DaemonExecutionFailureClass::Workflow,
    }
}

fn find_proc_failure(failure: &OperationalFailure) -> Option<&OperationalFailure> {
    let mut cursor = Some(failure);
    while let Some(current) = cursor {
        if matches!(
            (current.tower, current.entity),
            (TowerLevel::Proc, FailureEntity::Process(_))
        ) {
            return Some(current);
        }
        cursor = current.cause.as_deref();
    }
    None
}

fn report_attribution(
    failure: &OperationalFailure,
    classification: DaemonExecutionFailureClass,
) -> (TowerLevel, String) {
    if classification == DaemonExecutionFailureClass::ChildProc
        && let Some(proc_failure) = find_proc_failure(failure)
    {
        return (proc_failure.tower, format!("{:?}", proc_failure.entity));
    }
    (failure.tower, format!("{:?}", failure.entity))
}

fn operational_failure_message(failure: &OperationalFailure) -> String {
    let mut parts = vec![failure.payload.to_string()];
    let mut cause = failure.cause.as_deref();
    while let Some(next) = cause {
        parts.push(next.payload.to_string());
        cause = next.cause.as_deref();
    }
    parts.join(": ")
}

async fn force_returned_proc_if_present(value: AshValue) -> std::result::Result<(), EvalError> {
    if !matches!(value, AshValue::Closure { .. }) {
        return Ok(());
    }

    let runtime_state = RuntimeState::new();
    let process_id = ProcessId::new();
    runtime_state
        .register_root_process(process_id)
        .await
        .map_err(|error| EvalError::ExecutionFailed(error.to_string()))?;
    let parent_ctx = AshContext::new().with_runtime_state(runtime_state.clone());
    let process_ctx =
        ash_interp::derive_child_env(&parent_ctx, ChildEnvProjection::new(process_id, 0))
            .map_err(|error| EvalError::ExecutionFailed(error.to_string()))?;

    let mut call_ctx = process_ctx;
    call_ctx.set("__daemon_proc".to_string(), value);
    eval_returned_proc(&call_ctx).await.map(|_| ())
}

async fn eval_returned_proc(ctx: &AshContext) -> std::result::Result<AshValue, EvalError> {
    ash_interp::eval_expr_async(
        &Expr::Call {
            func: "__daemon_proc".to_string(),
            module: None,
            arguments: vec![Expr::Literal(AshValue::Null)],
        },
        ctx,
    )
    .await
}

fn instance_status_json(instance: &InstanceRecord) -> Value {
    json!({
        "ok": true,
        "host_mode": "Daemon",
        "instance_id": instance.instance_id,
        "workflow": instance.workflow,
        "status": instance.status,
        "class": instance.class,
        "report": instance.report,
        "args": instance.args,
        "config_id": instance.config_id,
        "admission": instance.admission,
        "definition_id": instance.definition_id,
        "artifact_id": instance.artifact_id,
        "artifact_version": instance.artifact_version,
        "source_hash": instance.source_hash,
        "artifact_summary": instance.artifact_summary,
    })
}

fn tower_label(tower: TowerLevel) -> &'static str {
    match tower {
        TowerLevel::Pure => "Pure",
        TowerLevel::Effectful => "Effectful",
        TowerLevel::Proc => "Proc",
        TowerLevel::Workflow => "Workflow",
    }
}

#[cfg(unix)]
fn serve(args: &DaemonServeArgs) -> Result<ExitCode> {
    let mut state = DaemonState::new(args)?;
    prepare_socket_path(&args.socket)?;
    let listener = UnixListener::bind(&args.socket)
        .with_context(|| format!("failed to bind daemon socket {}", args.socket.display()))?;
    fs::set_permissions(&args.socket, fs::Permissions::from_mode(0o600))
        .with_context(|| format!("failed to chmod daemon socket {}", args.socket.display()))?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(error) = handle_client(stream, &mut state) {
                    tracing::warn!("daemon client handling failed: {error:#}");
                }
            }
            Err(error) => tracing::warn!("daemon socket accept failed: {error}"),
        }
    }
    Ok(ExitCode::SUCCESS)
}

#[cfg(not(unix))]
fn serve(_args: &DaemonServeArgs) -> Result<ExitCode> {
    bail!("ash daemon serve is only implemented for local Unix-domain sockets in this alpha")
}

#[cfg(unix)]
fn handle_client(mut stream: UnixStream, state: &mut DaemonState) -> Result<()> {
    let mut line = String::new();
    BufReader::new(stream.try_clone()?).read_line(&mut line)?;
    let response = match serde_json::from_str::<DaemonRequest>(&line) {
        Ok(request) => match handle_request(request, state) {
            Ok(value) => value,
            Err(error) => json!({
                "ok": false,
                "error": {
                    "class": classify_daemon_error(&error),
                    "message": error.to_string(),
                }
            }),
        },
        Err(error) => json!({
            "ok": false,
            "error": {
                "class": "protocol_error",
                "message": error.to_string(),
            }
        }),
    };
    serde_json::to_writer(&mut stream, &response)?;
    stream.write_all(b"\n")?;
    Ok(())
}

fn handle_request(request: DaemonRequest, state: &mut DaemonState) -> Result<Value> {
    match request {
        DaemonRequest::List => Ok(state.response_list()),
        DaemonRequest::Start {
            workflow,
            args,
            config_id,
            admission_profile,
            execute,
        } => {
            if execute {
                state.start_and_execute(&workflow, &args, &config_id, admission_profile)
            } else {
                state.start(&workflow, &args, &config_id, admission_profile)
            }
        }
        DaemonRequest::Status { instance_id } => state.status(&instance_id),
        DaemonRequest::Cancel { instance_id } => state.cancel(&instance_id),
        DaemonRequest::Reload => state.reload(),
    }
}

#[cfg(unix)]
fn client_request(
    socket: &Path,
    format: DaemonOutputFormat,
    request: DaemonRequest,
    text_message: &str,
) -> Result<ExitCode> {
    let mut stream = UnixStream::connect(socket)
        .with_context(|| format!("failed to connect daemon socket {}", socket.display()))?;
    serde_json::to_writer(&mut stream, &request)?;
    stream.write_all(b"\n")?;

    let mut line = String::new();
    BufReader::new(stream).read_line(&mut line)?;
    let response: Value = serde_json::from_str(&line).context("invalid daemon response json")?;
    if response["ok"].as_bool() == Some(false) {
        bail!(
            "{}",
            response["error"]["message"]
                .as_str()
                .unwrap_or("daemon request failed")
        );
    }

    emit_response(format, &response, text_message)?;
    Ok(ExitCode::SUCCESS)
}

#[cfg(not(unix))]
fn client_request(
    _socket: &Path,
    _format: DaemonOutputFormat,
    _request: DaemonRequest,
    _text_message: &str,
) -> Result<ExitCode> {
    bail!("ash daemon control is only implemented for local Unix-domain sockets in this alpha")
}

fn emit_response(format: DaemonOutputFormat, response: &Value, text_message: &str) -> Result<()> {
    match format {
        DaemonOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(response)?);
        }
        DaemonOutputFormat::Text => {
            println!("{text_message}");
        }
    }
    Ok(())
}

fn index_definitions(
    root: &Path,
    root_id: &RuntimeRootSetId,
    profile_id: &RuntimeProfileId,
    config_id: &RuntimeConfigId,
) -> Result<Vec<DefinitionRecord>> {
    let engine = ash_engine::Engine::new()
        .build()
        .context("failed to build daemon indexing engine")?;
    let mut paths = Vec::new();
    for entry in WalkDir::new(root) {
        let entry = entry.with_context(|| format!("failed to walk root {}", root.display()))?;
        if entry.file_type().is_file()
            && entry
                .path()
                .extension()
                .and_then(|extension| extension.to_str())
                == Some("ash")
        {
            paths.push(entry.into_path());
        }
    }
    paths.sort();

    let mut definitions = Vec::new();
    for path in paths {
        let source = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let module =
            ash_parser::parse_surface_file_with_path(&source, Some(&path)).map_err(|errors| {
                anyhow!(
                    "parse/check/index failure in {}: {}",
                    path.display(),
                    errors
                        .into_iter()
                        .map(|error| error.to_string())
                        .collect::<Vec<_>>()
                        .join("; ")
                )
            })?;
        let Some(workflow) = module.workflow else {
            continue;
        };
        let mut checked_workflow = engine.parse_file(&path).map_err(|error| {
            anyhow!("parse/check/index failure in {}: {}", path.display(), error)
        })?;
        engine.check(&mut checked_workflow).map_err(|error| {
            anyhow!("parse/check/index failure in {}: {}", path.display(), error)
        })?;
        let relative_module_path = path
            .strip_prefix(root)
            .unwrap_or(path.as_path())
            .to_string_lossy()
            .replace('\\', "/");
        let workflow_name = workflow.name.to_string();
        let verified_artifact = build_runtime_kernel_artifact(&RuntimeArtifactBuildRequest::new(
            root_id.as_str(),
            relative_module_path.clone(),
            workflow_name.clone(),
            profile_id.as_str(),
            config_id.as_str(),
            source,
            format!("workflow={workflow_name};check=alpha-runtime-kernel-shared"),
        ))?;
        let artifact_summary =
            RuntimeKernelArtifactLanguageSummary::from_verified_artifact(&verified_artifact);
        definitions.push(DefinitionRecord {
            workflow: workflow_name,
            relative_module_path,
            definition_id: verified_artifact.definition.id.as_str().to_string(),
            artifact_id: verified_artifact.artifact.id.as_str().to_string(),
            artifact_version: verified_artifact.artifact_version.as_str().to_string(),
            source_hash: verified_artifact.source_hash,
            check_summary_hash: verified_artifact.check_summary_hash,
            artifact_summary,
        });
    }

    Ok(definitions)
}

fn validate_root(root: &Path) -> Result<()> {
    let metadata = fs::metadata(root)
        .with_context(|| format!("invalid root {}: not accessible", root.display()))?;
    if !metadata.is_dir() {
        bail!("invalid root {}: root must be a directory", root.display());
    }
    Ok(())
}

#[cfg(unix)]
fn prepare_socket_path(socket: &Path) -> Result<()> {
    let Ok(metadata) = fs::symlink_metadata(socket) else {
        return Ok(());
    };
    if metadata.file_type().is_socket() {
        fs::remove_file(socket)
            .with_context(|| format!("failed to remove stale socket {}", socket.display()))?;
        return Ok(());
    }
    bail!(
        "invalid socket path {}: existing path is not a Unix-domain socket",
        socket.display()
    )
}

#[cfg(unix)]
fn validate_local_control_paths(args: &DaemonServeArgs) -> Result<()> {
    let root_uid = fs::metadata(&args.root)?.uid();
    let socket_parent = args
        .socket
        .parent()
        .ok_or_else(|| anyhow!("invalid socket path {}: no parent", args.socket.display()))?;
    for (label, path) in [
        ("socket parent", socket_parent),
        ("state dir", args.state_dir.as_path()),
        ("cache dir", args.cache_dir.as_path()),
        ("log dir", args.log_dir.as_path()),
    ] {
        let metadata = fs::metadata(path)
            .with_context(|| format!("invalid {label} {}: not accessible", path.display()))?;
        if !metadata.is_dir() {
            bail!("invalid {label} {}: must be a directory", path.display());
        }
        if metadata.uid() != root_uid {
            bail!(
                "invalid {label} {}: ownership does not match root owner",
                path.display()
            );
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_local_control_paths(_args: &DaemonServeArgs) -> Result<()> {
    Ok(())
}

fn provider_registry_json(provider_registry: &ProviderRegistryIdentity) -> Value {
    json!({
        "id": provider_registry.id().to_string(),
        "provider_names": provider_registry.provider_names,
        "grants_admission_authority": provider_registry.grants_admission_authority(),
    })
}

fn classify_daemon_error(error: &anyhow::Error) -> &'static str {
    let message = error.to_string().to_lowercase();
    if message.contains("parse") || message.contains("index") {
        "index_failure"
    } else if message.contains("admission") && message.contains("rejected") {
        "admission_rejected"
    } else if message.contains("not found") {
        "not_found"
    } else {
        "request_failure"
    }
}

fn host_mode_label(host_mode: RuntimeHostMode) -> &'static str {
    match host_mode {
        RuntimeHostMode::Entry => "Entry",
        RuntimeHostMode::OneShot => "OneShot",
        RuntimeHostMode::Trace => "Trace",
        RuntimeHostMode::Daemon => "Daemon",
    }
}
