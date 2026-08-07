//! Local daemon control plane for the alpha RuntimeKernel host mode.

use anyhow::{Context, Result, anyhow, bail};
use ash_core::module_graph::{ModuleArtifactOrigin, ModuleKey};
use ash_core::runtime::{
    ProcessId, ServiceHealthStatus, ServiceId, ServiceLifecycleState, ServiceRuntimeRecord,
    ServiceShutdownMode,
};
use ash_core::runtime_kernel::{
    AdmissionIdentity, AlphaAdmissionProfile, AlphaAdmissionStatus, ApplicationAdmissionProfile,
    ApplicationArtifactIdentity, ApplicationBoundaryBindingManifest, ApplicationBoundaryBindings,
    ApplicationDefinitionIdentity, ApplicationInstanceIdentity, ApplicationRuntimeReport,
    ApplicationTerminalOutcome, ApplicationTraceBundle, ArtifactVersion, ProviderRegistryIdentity,
    RUNTIME_KERNEL_ARTIFACT_VERSION, RuntimeArtifactCacheKey, RuntimeConfigId,
    RuntimeEngineRelationship, RuntimeHostMode, RuntimeKernelArtifactLanguageSummary,
    RuntimeKernelIdentity, RuntimeProfileId, RuntimeProfileIdentity, RuntimeRootSet,
    RuntimeRootSetId, RuntimeTcirCarrierScope,
};
use ash_core::semantic_summary::{SourceAnchor, SourceOrigin};
use ash_core::{Expr, Span};
use ash_engine::{
    AdmittedProgramRequest, CanonicalTerminalEnvelopeV1, Engine,
    SubmittedDescriptorPreExecutionRejection,
    runtime_artifact::{RuntimeArtifactBuildRequest, build_runtime_kernel_artifact},
};
use ash_parser::CanonicalModuleGraphResolver;
use clap::{Args, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;
use walkdir::WalkDir;

/// Submit an Engine-issued admitted-program request for the daemon client.
///
/// The daemon retains transport and lifecycle responsibilities; semantic
/// dispatch and terminal normalization remain exclusively Engine-owned.
pub fn submit_admitted_program(
    engine: &Engine,
    request: &AdmittedProgramRequest,
) -> std::pin::Pin<
    Box<
        dyn std::future::Future<
                Output = std::result::Result<CanonicalTerminalEnvelopeV1, ash_engine::EngineError>,
            > + Send,
    >,
> {
    engine.execute_admitted_program(request)
}

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
    /// Start an entry instance record.
    Start(DaemonApplicationStartArgs),
    /// Start an entry instance record and execute it immediately.
    StartExecute(DaemonApplicationStartArgs),
    /// Report one entry instance status.
    Status(DaemonStatusArgs),
    /// Cancel a non-terminal entry instance record.
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
pub struct DaemonApplicationStartArgs {
    /// Unix-domain socket path for local control.
    #[arg(long, value_name = "PATH")]
    pub socket: PathBuf,
    /// Runtime argument passed to the entry instance record.
    #[arg(long = "arg", value_name = "VALUE")]
    pub start_args: Vec<String>,
    /// Runtime config identity recorded for this entry instance.
    #[arg(long = "config-id", value_name = "NAME", default_value = "default")]
    pub config_id: String,
    /// Minimal alpha daemon admission profile (empty, allow, reject).
    #[arg(long = "admission-profile", value_enum, default_value = "empty")]
    pub admission_profile: DaemonAdmissionProfile,
    /// Entry name to admit.
    #[arg(value_name = "ENTRY")]
    pub entry: String,
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
    /// Entry instance id.
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
    /// Entry instance id.
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
                application: args.entry.clone(),
                args: args.start_args.clone(),
                config_id: args.config_id.clone(),
                admission_profile: args.admission_profile,
                execute: false,
            },
            "entry instance admitted",
        ),
        DaemonCommand::StartExecute(args) => client_request(
            &args.socket,
            args.format,
            DaemonRequest::Start {
                application: args.entry.clone(),
                args: args.start_args.clone(),
                config_id: args.config_id.clone(),
                admission_profile: args.admission_profile,
                execute: true,
            },
            "entry instance executed",
        ),
        DaemonCommand::Status(args) => client_request(
            &args.socket,
            args.format,
            DaemonRequest::Status {
                instance_id: args.instance.clone(),
            },
            "entry instance status",
        ),
        DaemonCommand::Cancel(args) => client_request(
            &args.socket,
            args.format,
            DaemonRequest::Cancel {
                instance_id: args.instance_id.clone(),
            },
            "entry instance cancellation",
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
    /// Explicitly allow the entry instance record.
    Allow,
    /// Reject before the entry instance is admitted or recorded.
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
        #[serde(rename = "application")]
        application: String,
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
    /// Execute the selected submitted-program descriptor through this
    /// daemon's local Engine. The raw JSON is decoded at the command
    /// boundary so malformed descriptors receive the canonical fail-closed
    /// terminal rather than a transport-level fallback.
    ExecuteAdmittedDescriptor {
        descriptor: Value,
    },
}

const TASK_2035_SHARED_DESCRIPTOR_VERSION: u8 = 1;
const TASK_2035_SHARED_SOURCE_ID: &str = "task-2035-shared-int-42-v1";
const TASK_2035_SHARED_SOURCE_DIGEST: &str =
    "sha256:ed4088d136e54744d258b170222ad3b2a064feda91b78b0a248f2ccfb9b7684c";
const TASK_2035_SHARED_SOURCE: &str = "fn main() -> Int { 42 }\n";
const TASK_2035_SHARED_ENTRY: &str = "main";

/// The selected TASK-2035 submitted-program descriptor. This is transport
/// data only: after validation, the daemon parses, checks, admits, and mints
/// a fresh opaque request in its own Engine.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SubmittedProgramDescriptor {
    version: u8,
    source_identity: String,
    source_digest: String,
    source: String,
    entry: String,
    inputs: Vec<Value>,
    bindings: BTreeMap<String, Value>,
    run_control: SubmittedRunControl,
    host_configuration: SubmittedHostConfiguration,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SubmittedRunControl {
    deadline_millis: Option<u64>,
    cancellation: SubmittedCancellation,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SubmittedCancellation {
    NotCancelled,
    Cancelled,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum SubmittedHostConfiguration {
    None(()),
    Admission(SubmittedAdmissionHostConfiguration),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SubmittedAdmissionHostConfiguration {
    admission_profile: SubmittedAdmissionProfile,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SubmittedAdmissionProfile {
    Reject,
}

/// One closed control record for the selected submitted-program descriptor.
///
/// This is constructed only after every transport field has matched the
/// declared TASK-2035 contract. Execution consumes this record rather than
/// reinterpreting unvalidated transport controls.
#[derive(Debug)]
enum ValidatedSubmittedProgramRecord {
    Normal { source: String },
    ZeroDeadline { source: String },
    PreCancelled { source: String },
    HostRejected,
}

impl SubmittedProgramDescriptor {
    fn has_selected_task_2035_source_contract(&self) -> bool {
        self.version == TASK_2035_SHARED_DESCRIPTOR_VERSION
            && self.source_identity == TASK_2035_SHARED_SOURCE_ID
            && self.source_digest == TASK_2035_SHARED_SOURCE_DIGEST
            && self.source == TASK_2035_SHARED_SOURCE
            && source_digest(&self.source) == TASK_2035_SHARED_SOURCE_DIGEST
            && self.entry == TASK_2035_SHARED_ENTRY
            && self.inputs.is_empty()
            && self.bindings.is_empty()
    }

    fn into_selected_task_2035_record(self) -> Option<ValidatedSubmittedProgramRecord> {
        if !self.has_selected_task_2035_source_contract() {
            return None;
        }

        match (
            self.run_control.deadline_millis,
            self.run_control.cancellation,
            self.host_configuration,
        ) {
            (None, SubmittedCancellation::NotCancelled, SubmittedHostConfiguration::None(())) => {
                Some(ValidatedSubmittedProgramRecord::Normal {
                    source: self.source,
                })
            }
            (
                Some(0),
                SubmittedCancellation::NotCancelled,
                SubmittedHostConfiguration::None(()),
            ) => Some(ValidatedSubmittedProgramRecord::ZeroDeadline {
                source: self.source,
            }),
            (None, SubmittedCancellation::Cancelled, SubmittedHostConfiguration::None(())) => {
                Some(ValidatedSubmittedProgramRecord::PreCancelled {
                    source: self.source,
                })
            }
            (
                None,
                SubmittedCancellation::NotCancelled,
                SubmittedHostConfiguration::Admission(SubmittedAdmissionHostConfiguration {
                    admission_profile: SubmittedAdmissionProfile::Reject,
                }),
            ) => Some(ValidatedSubmittedProgramRecord::HostRejected),
            _ => None,
        }
    }
}

fn source_digest(source: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

fn descriptor_terminal_response(terminal: &CanonicalTerminalEnvelopeV1) -> Value {
    json!({
        "ok": true,
        "terminal": crate::value_convert::canonical_terminal_envelope_to_json(terminal),
    })
}

fn invalid_descriptor_terminal_response() -> Value {
    descriptor_terminal_response(
        &SubmittedDescriptorPreExecutionRejection::InvalidDescriptor.canonical_terminal_envelope(),
    )
}

fn admit_daemon_source(
    engine: &ash_engine::Engine,
    path: &Path,
    source: &str,
    entry_name: &str,
) -> Result<ash_engine::AdmittedProgram, ash_engine::EngineError> {
    match engine.canonical_module_closure_from_source(path, source, entry_name)? {
        Some(closure) => engine.admit_linked_module_closure(closure),
        None => {
            let mut entry = engine.parse_file_source(path, source)?;
            engine.admit_program(&mut entry)
        }
    }
}

fn execute_descriptor_with_local_engine(
    descriptor: ValidatedSubmittedProgramRecord,
    descriptor_path: PathBuf,
) -> Result<Value> {
    let (source, timeout, pre_cancelled) = match descriptor {
        ValidatedSubmittedProgramRecord::HostRejected => {
            return Ok(descriptor_terminal_response(
                &SubmittedDescriptorPreExecutionRejection::HostAdmissionRejected
                    .canonical_terminal_envelope(),
            ));
        }
        ValidatedSubmittedProgramRecord::Normal { source } => (source, None, false),
        ValidatedSubmittedProgramRecord::ZeroDeadline { source } => {
            (source, Some(Duration::ZERO), false)
        }
        ValidatedSubmittedProgramRecord::PreCancelled { source } => (source, None, true),
    };
    let engine = Engine::new()
        .build()
        .context("failed to build daemon descriptor Engine")?;
    let mut entry = engine
        .parse_file_source(&descriptor_path, &source)
        .context("failed to parse submitted descriptor source")?;
    let program = match engine.admit_program(&mut entry) {
        Ok(program) => program,
        Err(error) => {
            if let Some(terminal) = error.canonical_terminal_envelope() {
                return Ok(descriptor_terminal_response(&terminal));
            }
            return Err(anyhow!(
                "failed to admit submitted descriptor source: {error}"
            ));
        }
    };
    let (request, cancellation) = engine
        .new_admitted_program_request(&program, timeout)
        .context("failed to mint daemon-local admitted-program request")?;

    if pre_cancelled {
        cancellation.cancel();
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to build daemon descriptor execution runtime")?;
    let terminal = runtime
        .block_on(submit_admitted_program(&engine, &request))
        .context("daemon-local admitted-program execution failed")?;
    Ok(descriptor_terminal_response(&terminal))
}

fn default_config_id() -> String {
    "default".to_string()
}

#[derive(Debug, Clone, Serialize)]
struct DefinitionRecord {
    #[serde(rename = "application")]
    application: String,
    relative_module_path: String,
    #[serde(skip_serializing)]
    definition_id: String,
    #[serde(skip_serializing)]
    canonical_module_route: bool,
    artifact_id: String,
    artifact_version: String,
    source_hash: String,
    check_summary_hash: String,
    artifact_summary: RuntimeKernelArtifactLanguageSummary,
}

#[derive(Debug, Clone, Serialize)]
struct InstanceRecord {
    instance_id: String,
    #[serde(rename = "application")]
    application: String,
    status: InstanceStatus,
    args: Vec<String>,
    config_id: String,
    admission: InstanceAdmissionRecord,
    definition_id: String,
    artifact_id: String,
    artifact_version: String,
    source_hash: String,
    artifact_summary: RuntimeKernelArtifactLanguageSummary,
    application_report: ApplicationRuntimeReport,
    service_lifecycle: ServiceRuntimeRecord,
    #[serde(skip_serializing_if = "Option::is_none")]
    class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    report: Option<InstanceExecutionReport>,
}

#[derive(Debug, Clone, Serialize)]
struct InstanceAdmissionRecord {
    status: String,
    profile: String,
    profile_boundary: ApplicationAdmissionProfile,
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
    boundary: String,
    kind: String,
    host_failure: bool,
    entity: String,
    message: String,
    payload_type: Option<String>,
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

    /// Execute the selected descriptor through a daemon-local Engine.
    ///
    /// The descriptor identifies exact submitted bytes and controls; it is
    /// never an opaque request transport. The request below is minted only
    /// after this daemon's Engine has parsed, checked, lowered, and admitted
    /// the submitted source.
    fn execute_admitted_descriptor(&self, descriptor: Value) -> Result<Value> {
        let descriptor = match serde_json::from_value::<SubmittedProgramDescriptor>(descriptor) {
            Ok(descriptor) => match descriptor.into_selected_task_2035_record() {
                Some(record) => record,
                None => return Ok(invalid_descriptor_terminal_response()),
            },
            Err(_) => return Ok(invalid_descriptor_terminal_response()),
        };
        let descriptor_path = self.root.join("task-2035-shared-int-42-v1.ash");
        // The socket loop runs under the CLI's Tokio runtime and Engine CPS
        // admissions can be thread-local. Keep local Engine construction,
        // request minting, and execution together on the daemon worker.
        std::thread::scope(|scope| {
            let handle = scope
                .spawn(move || execute_descriptor_with_local_engine(descriptor, descriptor_path));
            handle
                .join()
                .map_err(|_| anyhow!("daemon descriptor execution worker panicked"))?
        })
    }

    fn start(
        &mut self,
        entry: &str,
        args: &[String],
        config_id: &str,
        admission_profile: DaemonAdmissionProfile,
    ) -> Result<Value> {
        if config_id != self.config_id.as_str() {
            bail!(
                "non-default daemon config_id '{config_id}' is unsupported in the alpha daemon; restart the daemon with that default config or use config_id '{}'",
                self.config_id.as_str()
            );
        }

        let definition = self
            .definitions
            .iter()
            .find(|definition| definition.application == entry)
            .ok_or_else(|| anyhow!("entry definition not indexed: {entry}"))?;

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

        let profile_boundary = ApplicationAdmissionProfile::runtime_boundary(
            admission_profile.as_str(),
            "daemon:start.admission_profile",
            false,
        )?;
        let boundary_bindings = daemon_start_boundary_bindings(args)?;
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
        let definition_identity = ApplicationDefinitionIdentity::new(
            self.root_id.clone(),
            definition.relative_module_path.clone(),
            definition.application.clone(),
            self.profile_id.clone(),
            self.config_id.clone(),
            definition.source_hash.clone(),
        );
        let artifact_identity = ApplicationArtifactIdentity::new(
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
            profile_boundary: profile_boundary.clone(),
            reason: None,
            capability_grants: admission.capability_grants.len(),
            resource_grants: admission.resource_grants.len(),
        };
        let mut artifact_summary = definition.artifact_summary.clone();
        artifact_summary.invocation_packet.admission_profile = profile_boundary;
        artifact_summary.invocation_packet.boundary_bindings = boundary_bindings;
        let application_report = application_report_from_artifact_summary(
            &artifact_summary,
            ApplicationTerminalOutcome::admitted(),
        );
        let service_lifecycle = daemon_service_lifecycle(&definition.application);
        let instance = ApplicationInstanceIdentity::admit(
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
            application: definition.application.clone(),
            status: InstanceStatus::Admitted,
            args: args.to_vec(),
            config_id: config_id.to_string(),
            admission: admission_record.clone(),
            definition_id: definition.definition_id.clone(),
            artifact_id: definition.artifact_id.clone(),
            artifact_version: definition.artifact_version.clone(),
            source_hash: definition.source_hash.clone(),
            artifact_summary: artifact_summary.clone(),
            application_report: application_report.clone(),
            service_lifecycle: service_lifecycle.clone(),
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
            "application": entry,
            "args": args,
            "config_id": config_id,
            "admission": admission_record,
            "definition_id": definition.definition_id,
            "artifact_id": definition.artifact_id,
            "artifact_version": definition.artifact_version,
            "artifact_summary": artifact_summary,
            "application_report": application_report,
            "service_lifecycle": service_lifecycle,
            "provider_registry": provider_registry_json(&self.provider_registry),
        }))
    }

    fn start_and_execute(
        &mut self,
        entry: &str,
        args: &[String],
        config_id: &str,
        admission_profile: DaemonAdmissionProfile,
    ) -> Result<Value> {
        let start = self.start(entry, args, config_id, admission_profile)?;
        let instance_id = start["instance_id"]
            .as_str()
            .ok_or_else(|| anyhow!("daemon start response missing instance id"))?
            .to_string();

        let outcome = self.execute_instance(entry, &instance_id);
        let instance = self
            .instances
            .get_mut(&instance_id)
            .ok_or_else(|| anyhow!("entry instance not found after start: {instance_id}"))?;
        match outcome {
            Ok(()) => {
                instance.status = InstanceStatus::Succeeded;
                instance.class = Some("application_succeeded".to_string());
                instance.application_report = application_report_from_artifact_summary(
                    &instance.artifact_summary,
                    ApplicationTerminalOutcome::succeeded(),
                );
                instance.service_lifecycle = daemon_shutdown_service_lifecycle(
                    &instance.service_lifecycle,
                    ServiceShutdownMode::Graceful,
                    "daemon start-execute completed",
                );
                instance.report = Some(InstanceExecutionReport {
                    status: "succeeded".to_string(),
                    failure: None,
                });
            }
            Err(failure) => {
                instance.status = InstanceStatus::Failed;
                instance.class = Some(failure.class().to_string());
                let failure_reason = failure.report.message.clone();
                instance.application_report = application_report_from_artifact_summary(
                    &instance.artifact_summary,
                    ApplicationTerminalOutcome::failed(failure_reason),
                );
                instance.service_lifecycle = daemon_shutdown_service_lifecycle(
                    &instance.service_lifecycle,
                    ServiceShutdownMode::Forced,
                    "daemon start-execute failed",
                );
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
        entry: &str,
        instance_id: &str,
    ) -> std::result::Result<(), Box<InstanceExecutionFailure>> {
        let definition = self
            .definitions
            .iter()
            .find(|definition| definition.application == entry)
            .ok_or_else(|| {
                Box::new(InstanceExecutionFailure::application_request(format!(
                    "entry definition not indexed: {entry}"
                )))
            })?;
        let instance = self.instances.get(instance_id).ok_or_else(|| {
            Box::new(InstanceExecutionFailure::application_request(format!(
                "entry instance not found for execution: {instance_id}"
            )))
        })?;
        if definition.source_hash != instance.source_hash {
            return Err(Box::new(InstanceExecutionFailure::application_request(
                format!(
                    "admitted artifact drift: indexed definition source hash {} no longer matches admitted source hash {}",
                    definition.source_hash, instance.source_hash
                ),
            )));
        }
        let path = self.root.join(&definition.relative_module_path);
        let current_source = std::fs::read_to_string(&path).map_err(|error| {
            Box::new(InstanceExecutionFailure::application_request(format!(
                "failed to read daemon entry for admitted artifact drift check: {error}"
            )))
        })?;
        let engine = ash_engine::Engine::new().build().map_err(|error| {
            Box::new(InstanceExecutionFailure::application_request(format!(
                "failed to build daemon entry checker for admitted artifact drift check: {error}"
            )))
        })?;
        let current_source_hash = if definition.canonical_module_route {
            let closure = engine
                .canonical_module_closure_from_source(&path, &current_source, &instance.application)
                .map_err(|error| {
                    Box::new(InstanceExecutionFailure::application_request(format!(
                        "failed to rebuild canonical daemon closure for admitted artifact drift check: {error}"
                    )))
                })?
                .ok_or_else(|| {
                    Box::new(InstanceExecutionFailure::application_request(
                        "canonical daemon entry no longer has a canonical module route".to_string(),
                    ))
                })?;
            canonical_definition_record(
                &self.root_id,
                &self.profile_id,
                &RuntimeConfigId::new(instance.config_id.clone()),
                &definition.relative_module_path,
                &current_source,
                &closure,
            )
            .map(|record| record.source_hash)
            .map_err(|error| {
                Box::new(InstanceExecutionFailure::application_request(format!(
                    "failed to rebuild canonical daemon metadata for admitted artifact drift check: {error}"
                )))
            })?
        } else {
            let mut entry = engine.parse_file(&path).map_err(|error| {
                Box::new(InstanceExecutionFailure::application_request(format!(
                    "failed to parse daemon entry for admitted artifact drift check: {error}"
                )))
            })?;
            let checked_function = engine
                .check_entry_artifact(
                    &mut entry,
                    format!(
                        "callable:{}::{}",
                        definition.relative_module_path, instance.application
                    ),
                    SourceAnchor::new(
                        SourceOrigin::File(definition.relative_module_path.clone()),
                        Some(Span {
                            start: 0,
                            end: current_source.len(),
                        }),
                        format!("checked-function:{}", instance.application),
                    ),
                )
                .map_err(|error| {
                    Box::new(InstanceExecutionFailure::application_request(format!(
                        "failed to check daemon entry for admitted artifact drift check: {error}"
                    )))
                })?;
            let artifact_request = daemon_entry_artifact_request(
                self.root_id.as_str(),
                definition.relative_module_path.clone(),
                instance.application.clone(),
                self.profile_id.as_str(),
                instance.config_id.as_str(),
                checked_function,
                current_source.clone(),
            )
            .map_err(|error| {
                Box::new(InstanceExecutionFailure::application_request(format!(
                    "failed to prepare daemon entry artifact drift check: {error}"
                )))
            })?;
            build_runtime_kernel_artifact(&artifact_request)
                .map(|artifact| artifact.source_hash)
                .map_err(|error| {
                    Box::new(InstanceExecutionFailure::application_request(format!(
                        "failed to rebuild daemon entry artifact for admitted artifact drift check: {error}"
                    )))
                })?
        };
        if current_source_hash != instance.source_hash {
            return Err(Box::new(InstanceExecutionFailure::application_request(
                format!(
                    "admitted artifact drift: live source hash {current_source_hash} no longer matches admitted source hash {}",
                    instance.source_hash
                ),
            )));
        }
        let execution_source = current_source;
        let execution_path = path;
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
                    let program = admit_daemon_source(
                        &engine,
                        &execution_path,
                        &execution_source,
                        &instance.application,
                    )
                    .map_err(|error| {
                        Box::new(InstanceExecutionFailure::application_request(format!(
                            "failed to admit daemon entry for shared Engine execution: {error}"
                        )))
                    })?;
                    let (request, _cancellation) = engine
                        .new_admitted_program_request(&program, None)
                        .map_err(|error| {
                            Box::new(InstanceExecutionFailure::application_request(format!(
                                "failed to create daemon admitted-program request: {error}"
                            )))
                        })?;
                    match submit_admitted_program(&engine, &request)
                        .await
                        .map_err(|error| {
                            Box::new(InstanceExecutionFailure::application_request(format!(
                                "shared Engine execution failed: {error}"
                            )))
                        })? {
                        CanonicalTerminalEnvelopeV1::Returned(_) => Ok(()),
                        terminal => Err(Box::new(InstanceExecutionFailure::application_request(
                            format!("shared Engine terminal envelope: {terminal:?}"),
                        ))),
                    }
                })
            });
            handle.join().map_err(|_| {
                Box::new(InstanceExecutionFailure::host(
                    "daemon entry execution worker panicked".to_string(),
                ))
            })?
        })
    }

    fn status(&self, instance_id: &str) -> Result<Value> {
        let instance = self
            .instances
            .get(instance_id)
            .ok_or_else(|| anyhow!("entry instance not found: {instance_id}"))?;
        Ok(json!({
            "ok": true,
            "host_mode": "Daemon",
            "instance_id": instance.instance_id,
            "application": instance.application,
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
            "application_report": instance.application_report,
            "service_lifecycle": instance.service_lifecycle,
        }))
    }

    fn cancel(&mut self, instance_id: &str) -> Result<Value> {
        let instance = self
            .instances
            .get_mut(instance_id)
            .ok_or_else(|| anyhow!("entry instance not found: {instance_id}"))?;
        let class = if instance.status.is_terminal() {
            "already_terminal"
        } else {
            instance.status = InstanceStatus::Cancelled;
            instance.application_report = application_report_from_artifact_summary(
                &instance.artifact_summary,
                ApplicationTerminalOutcome::cancelled("daemon instance cancelled"),
            );
            instance.service_lifecycle = daemon_shutdown_service_lifecycle(
                &instance.service_lifecycle,
                ServiceShutdownMode::Graceful,
                "daemon instance cancelled",
            );
            "cancelled"
        };
        Ok(json!({
            "ok": true,
            "host_mode": "Daemon",
            "instance_id": instance.instance_id,
            "status": instance.status,
            "class": class,
            "service_lifecycle": instance.service_lifecycle,
        }))
    }

    fn reload(&mut self) -> Result<Value> {
        let staged =
            index_definitions(&self.root, &self.root_id, &self.profile_id, &self.config_id)?;
        let count = staged.len();
        self.definitions = staged;
        let mut service_lifecycle = None;
        for instance in self.instances.values_mut() {
            if !instance.service_lifecycle.terminal {
                instance.service_lifecycle =
                    daemon_reload_service_lifecycle(&instance.service_lifecycle, "daemon:reload");
                service_lifecycle = Some(instance.service_lifecycle.clone());
            }
        }
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
            "service_lifecycle": service_lifecycle,
        }))
    }
}

struct InstanceExecutionFailure {
    report: InstanceFailureReport,
}

impl InstanceExecutionFailure {
    fn application_request(message: String) -> Self {
        Self {
            report: InstanceFailureReport {
                boundary: "Application".to_string(),
                kind: "application_execution_failure".to_string(),
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
                boundary: "DaemonHost".to_string(),
                kind: "daemon_execution_host_failure".to_string(),
                host_failure: true,
                entity: "daemon_host".to_string(),
                message,
                payload_type: None,
            },
        }
    }

    fn class(&self) -> &str {
        match self.report.kind.as_str() {
            "child_proc_failure" => "application_child_failure",
            "application_failure" => "application_failure",
            "effect_failure" => "application_child_failure",
            "daemon_execution_host_failure" => "daemon_host_failure",
            _ => "application_failure",
        }
    }
}

fn instance_status_json(instance: &InstanceRecord) -> Value {
    json!({
        "ok": true,
        "host_mode": "Daemon",
        "instance_id": instance.instance_id,
        "application": instance.application,
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
        "application_report": instance.application_report,
        "service_lifecycle": instance.service_lifecycle,
    })
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
            application,
            args,
            config_id,
            admission_profile,
            execute,
        } => {
            if execute {
                state.start_and_execute(&application, &args, &config_id, admission_profile)
            } else {
                state.start(&application, &args, &config_id, admission_profile)
            }
        }
        DaemonRequest::Status { instance_id } => state.status(&instance_id),
        DaemonRequest::Cancel { instance_id } => state.cancel(&instance_id),
        DaemonRequest::Reload => state.reload(),
        DaemonRequest::ExecuteAdmittedDescriptor { descriptor } => {
            state.execute_admitted_descriptor(descriptor)
        }
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
    let canonical_child_paths = canonical_child_module_paths(&paths)?;

    let mut definitions = Vec::new();
    for path in paths {
        if canonical_child_paths.contains(&canonical_path_for_index(&path)) {
            continue;
        }
        let source = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let relative_module_path = path
            .strip_prefix(root)
            .unwrap_or(path.as_path())
            .to_string_lossy()
            .replace('\\', "/");
        if let Some(closure) = engine
            .canonical_module_closure_from_source(&path, &source, "main")
            .map_err(|error| anyhow!("parse/check/index failure in {}: {error}", path.display()))?
        {
            definitions.push(canonical_definition_record(
                root_id,
                profile_id,
                config_id,
                &relative_module_path,
                &source,
                &closure,
            )?);
            continue;
        }
        let mut checked_application = engine.parse_file(&path).map_err(|error| {
            anyhow!("parse/check/index failure in {}: {}", path.display(), error)
        })?;
        engine
            .verify_entry_definition(&checked_application)
            .map_err(|error| {
                anyhow!("parse/check/index failure in {}: {}", path.display(), error)
            })?;
        let entry_name = "main".to_string();
        let checked_function = engine
            .check_entry_artifact(
                &mut checked_application,
                format!("callable:{relative_module_path}::{entry_name}"),
                SourceAnchor::new(
                    SourceOrigin::File(relative_module_path.clone()),
                    Some(Span {
                        start: 0,
                        end: source.len(),
                    }),
                    format!("checked-function:{entry_name}"),
                ),
            )
            .map_err(|error| {
                anyhow!("parse/check/index failure in {}: {}", path.display(), error)
            })?;
        let verified_artifact = build_runtime_kernel_artifact(&daemon_entry_artifact_request(
            root_id.as_str(),
            relative_module_path.clone(),
            entry_name.clone(),
            profile_id.as_str(),
            config_id.as_str(),
            checked_function,
            source.clone(),
        )?)?;
        let artifact_summary =
            RuntimeKernelArtifactLanguageSummary::from_verified_artifact(&verified_artifact);
        definitions.push(DefinitionRecord {
            application: entry_name,
            relative_module_path,
            definition_id: verified_artifact.definition.id.as_str().to_string(),
            canonical_module_route: false,
            artifact_id: verified_artifact.artifact.id.as_str().to_string(),
            artifact_version: verified_artifact.artifact_version.as_str().to_string(),
            source_hash: verified_artifact.source_hash,
            check_summary_hash: verified_artifact.check_summary_hash,
            artifact_summary,
        });
    }

    Ok(definitions)
}

fn canonical_definition_record(
    root_id: &RuntimeRootSetId,
    profile_id: &RuntimeProfileId,
    config_id: &RuntimeConfigId,
    relative_module_path: &str,
    source: &str,
    closure: &ash_engine::LinkedModuleClosure,
) -> Result<DefinitionRecord> {
    let root_module = closure
        .modules()
        .iter()
        .find(|module| module.interface().artifact().key() == closure.root())
        .ok_or_else(|| anyhow!("canonical daemon closure has no root module"))?;
    let entry_name = root_module
        .entry_name()
        .filter(|name| !name.is_empty())
        .ok_or_else(|| anyhow!("canonical daemon closure has no selected main entry"))?;
    let core = root_module
        .core()
        .ok_or_else(|| anyhow!("canonical daemon closure has no root Core artifact"))?;
    let checked = core.checked_core_program();
    let core_fingerprint = serde_json::to_vec(&(checked.expr(), checked.ty(), checked.row()))
        .context("failed to fingerprint canonical daemon root Core")?;
    let mut hasher = Sha256::new();
    hasher.update(&core_fingerprint);
    let body_fingerprint = format!("{:x}", hasher.finalize());
    let checked_function = ash_core::runtime_kernel::CheckedFunctionArtifact {
        function_identity: format!("callable:{relative_module_path}::{entry_name}"),
        effect_row: checked.row().clone(),
        // This carrier exists only for daemon reporting/index identity. The
        // canonical Core/CPS closure remains the sole execution input.
        body: Expr::Variable {
            name: format!("__canonical_module_body_{body_fingerprint}"),
            span: Span {
                start: 0,
                end: source.len(),
            },
        },
        source_anchor: root_module.source_anchor().clone(),
        result_type: checked.ty().clone(),
    };
    let artifact_request = daemon_entry_artifact_request(
        root_id.as_str(),
        relative_module_path.to_owned(),
        entry_name.to_owned(),
        profile_id.as_str(),
        config_id.as_str(),
        checked_function,
        source.to_owned(),
    )?
    .with_tcir_carrier_scope(RuntimeTcirCarrierScope::CanonicalModuleClosureMetadata);
    let verified_artifact = build_runtime_kernel_artifact(&artifact_request)
        .context("failed to build canonical daemon metadata artifact")?;
    let artifact_summary =
        RuntimeKernelArtifactLanguageSummary::from_verified_artifact(&verified_artifact);
    Ok(DefinitionRecord {
        application: entry_name.to_owned(),
        relative_module_path: relative_module_path.to_owned(),
        definition_id: verified_artifact.definition.id.as_str().to_string(),
        canonical_module_route: true,
        artifact_id: verified_artifact.artifact.id.as_str().to_string(),
        artifact_version: verified_artifact.artifact_version.as_str().to_string(),
        source_hash: verified_artifact.source_hash,
        check_summary_hash: verified_artifact.check_summary_hash,
        artifact_summary,
    })
}

fn canonical_child_module_paths(paths: &[PathBuf]) -> Result<std::collections::BTreeSet<PathBuf>> {
    let mut child_paths = std::collections::BTreeSet::new();
    for path in paths {
        let source = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let Ok(parsed) = ash_parser::parse_surface_file_with_path(&source, Some(path)) else {
            continue;
        };
        if parsed.module_decls.is_empty() {
            continue;
        }
        let crate_name = parsed
            .crate_metadata
            .as_ref()
            .map(|metadata| metadata.crate_name.to_string())
            .or_else(|| {
                path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(ToOwned::to_owned)
            })
            .unwrap_or_else(|| "app".to_owned());
        let root_key = ModuleKey::root(crate_name)
            .map_err(|error| anyhow!("invalid canonical daemon crate identity: {error}"))?;
        let graph = CanonicalModuleGraphResolver::new()
            .resolve_root(root_key.clone(), path)
            .map_err(|error| anyhow!("canonical daemon module graph failure: {error}"))?;
        for (module, unit) in graph.module_units() {
            if module == &root_key {
                continue;
            }
            if let ModuleArtifactOrigin::File(child_path) = unit.artifact().origin() {
                child_paths.insert(canonical_path_for_index(Path::new(child_path)));
            }
        }
    }
    Ok(child_paths)
}

fn canonical_path_for_index(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn daemon_entry_artifact_request(
    root_id: impl Into<String>,
    relative_module_path: impl Into<String>,
    entry_name: impl Into<String>,
    profile_id: impl Into<String>,
    config_id: impl Into<String>,
    checked_function: ash_core::runtime_kernel::CheckedFunctionArtifact,
    source: impl Into<String>,
) -> Result<RuntimeArtifactBuildRequest> {
    let relative_module_path = relative_module_path.into();
    let entry_name = entry_name.into();
    Ok(RuntimeArtifactBuildRequest::new_application_entrypoint(
        root_id,
        relative_module_path.clone(),
        entry_name.clone(),
        format!("callable:{relative_module_path}::{entry_name}"),
        format!("runtime-target:application-entry:{entry_name}"),
        profile_id,
        config_id,
        checked_function,
        source,
        format!(
            "entrypoint={entry_name};callable={relative_module_path}::{entry_name};check=application-runtime-kernel-shared"
        ),
    )?
    .with_runtime_support_identity(selected_runtime_support_identity()))
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
    let euid = current_effective_uid();
    let socket_parent = args
        .socket
        .parent()
        .ok_or_else(|| anyhow!("invalid socket path {}: no parent", args.socket.display()))?;
    for (label, path) in [
        ("root", args.root.as_path()),
        ("socket parent", socket_parent),
        ("state dir", args.state_dir.as_path()),
        ("cache dir", args.cache_dir.as_path()),
        ("log dir", args.log_dir.as_path()),
    ] {
        validate_same_user_control_dir(label, path, euid)?;
    }
    Ok(())
}

#[cfg(unix)]
fn validate_same_user_control_dir(label: &str, path: &Path, euid: u32) -> Result<()> {
    let link_metadata = fs::symlink_metadata(path)
        .with_context(|| format!("invalid {label} {}: not accessible", path.display()))?;
    if link_metadata.file_type().is_symlink() {
        bail!(
            "invalid {label} {}: symbolic links are not allowed for local-control directories",
            path.display()
        );
    }
    let metadata = fs::metadata(path)
        .with_context(|| format!("invalid {label} {}: not accessible", path.display()))?;
    if !metadata.is_dir() {
        bail!("invalid {label} {}: must be a directory", path.display());
    }
    if metadata.uid() != euid {
        bail!(
            "invalid {label} {}: owner uid {} does not match current effective user uid {euid}",
            path.display(),
            metadata.uid()
        );
    }
    let mode = metadata.mode() & 0o777;
    if mode & 0o022 != 0 {
        bail!(
            "invalid {label} {}: directory is group/world-writable (mode {mode:o})",
            path.display()
        );
    }
    Ok(())
}

#[cfg(unix)]
fn current_effective_uid() -> u32 {
    unsafe extern "C" {
        fn geteuid() -> u32;
    }

    // SAFETY: POSIX `geteuid` has no arguments, does not require caller-owned
    // memory, and is safe to call for querying the process effective UID.
    unsafe { geteuid() }
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

fn daemon_start_boundary_bindings(
    args: &[String],
) -> Result<
    ApplicationBoundaryBindings,
    ash_core::runtime_kernel::ApplicationBoundaryBindingDiagnostic,
> {
    ApplicationBoundaryBindings::from_manifest(
        "daemon:start.boundary",
        ApplicationBoundaryBindingManifest {
            providers: args
                .iter()
                .enumerate()
                .map(|(index, _)| format!("Args:{index}"))
                .collect(),
            grants_authority: false,
            ..ApplicationBoundaryBindingManifest::default()
        },
    )
}

fn daemon_service_lifecycle(entry: &str) -> ServiceRuntimeRecord {
    let id = ServiceId::new();
    ServiceRuntimeRecord {
        id,
        name: entry.to_string(),
        process_id: ProcessId::new(),
        lifecycle: ServiceLifecycleState::Running,
        health: ServiceHealthStatus::Healthy,
        reload_generation: 0,
        last_reload: None,
        shutdown_mode: None,
        terminal_reason: None,
        terminal: false,
        retained: true,
        report_identity: Some(format!("service-report:{id:?}")),
    }
}

fn daemon_reload_service_lifecycle(
    lifecycle: &ServiceRuntimeRecord,
    reload_identity: &str,
) -> ServiceRuntimeRecord {
    let mut lifecycle = lifecycle.clone();
    lifecycle.lifecycle = ServiceLifecycleState::Running;
    lifecycle.health = ServiceHealthStatus::Healthy;
    lifecycle.reload_generation = lifecycle.reload_generation.saturating_add(1);
    lifecycle.last_reload = Some(reload_identity.to_string());
    lifecycle
}

fn daemon_shutdown_service_lifecycle(
    lifecycle: &ServiceRuntimeRecord,
    mode: ServiceShutdownMode,
    reason: &str,
) -> ServiceRuntimeRecord {
    let mut lifecycle = lifecycle.clone();
    lifecycle.lifecycle = ServiceLifecycleState::Terminated;
    lifecycle.health = ServiceHealthStatus::Unavailable;
    lifecycle.shutdown_mode = Some(mode);
    lifecycle.terminal_reason = Some(reason.to_string());
    lifecycle.terminal = true;
    lifecycle.retained = true;
    lifecycle
}

fn application_report_from_artifact_summary(
    artifact_summary: &RuntimeKernelArtifactLanguageSummary,
    terminal_outcome: ApplicationTerminalOutcome,
) -> ApplicationRuntimeReport {
    let trace_bundle = ApplicationTraceBundle::from_invocation_packet(
        &artifact_summary.invocation_packet,
        Vec::new(),
        Vec::new(),
    );
    ApplicationRuntimeReport::new(
        &artifact_summary.invocation_packet,
        terminal_outcome,
        trace_bundle,
    )
}

fn selected_runtime_support_identity() -> String {
    std::env::var("ASH_RUNTIME_SUPPORT_IDENTITY")
        .unwrap_or_else(|_| "ash-runtime-support:unselected".to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::Value;

    #[test]
    fn task_2064_daemon_index_discovers_canonical_child_module_files() {
        let fixture = tempfile::tempdir().expect("create daemon module fixture");
        fs::write(
            fixture.path().join("main.ash"),
            "pub mod api; fn main() -> Int { 42 }",
        )
        .expect("write daemon module root");
        fs::write(
            fixture.path().join("api.ash"),
            "pub fn serve() -> Int { 2 }",
        )
        .expect("write daemon module child");
        let child_paths = canonical_child_module_paths(&[
            fixture.path().join("main.ash"),
            fixture.path().join("api.ash"),
        ])
        .expect("discover canonical daemon child paths");
        assert!(child_paths.contains(&canonical_path_for_index(&fixture.path().join("api.ash"))));
    }

    #[test]
    fn task_2064_daemon_index_accepts_canonical_module_root() {
        let fixture = tempfile::tempdir().expect("create daemon index fixture");
        let root = fixture.path().join("main.ash");
        fs::write(
            &root,
            "crate app; pub mod api; use crate::api::serve as remote; fn main() -> Int { remote() }",
        )
        .expect("write daemon canonical root");
        fs::write(
            fixture.path().join("api.ash"),
            "pub fn serve() -> Int { 42 }",
        )
        .expect("write daemon canonical child");

        let root_id = RuntimeRootSetId::new(fixture.path().display().to_string());
        let definitions = index_definitions(
            fixture.path(),
            &root_id,
            &RuntimeProfileId::new("default"),
            &RuntimeConfigId::new("default"),
        )
        .expect("canonical daemon root must be indexable");

        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].application, "main");
        assert_eq!(definitions[0].relative_module_path, "main.ash");
        assert_eq!(
            definitions[0].artifact_summary.tcir.carrier_scope,
            RuntimeTcirCarrierScope::CanonicalModuleClosureMetadata
        );
    }

    #[test]
    fn task_2064_daemon_execution_uses_canonical_module_route() {
        let fixture = tempfile::tempdir().expect("create daemon execution fixture");
        let root = fixture.path().join("main.ash");
        let source = "crate app; pub mod api; use crate::api::serve as remote; fn main() -> Int { remote() }";
        fs::write(&root, source).expect("write daemon execution root");
        fs::write(
            fixture.path().join("api.ash"),
            "pub fn serve() -> Int { 42 }",
        )
        .expect("write daemon execution child");

        let engine = ash_engine::Engine::new()
            .build()
            .expect("build daemon execution Engine");
        let admitted = admit_daemon_source(&engine, &root, source, "main")
            .expect("daemon source must reach the canonical linked admission");
        let (request, _cancellation) = engine
            .new_admitted_program_request(&admitted, None)
            .expect("daemon Engine mints one admitted request");
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build daemon test runtime");
        let terminal = runtime
            .block_on(submit_admitted_program(&engine, &request))
            .expect("daemon adapter executes canonical request");

        assert_eq!(
            terminal,
            CanonicalTerminalEnvelopeV1::returned(Value::Int(42))
        );
    }

    #[test]
    fn task_2064_daemon_rejects_parseable_unsupported_callable_without_fallback() {
        let fixture = tempfile::tempdir().expect("create unsupported daemon fixture");
        let root = fixture.path().join("main.ash");
        let source = "fn main() -> Int { match 1 { 1 => 1, _ => 0 } }";
        fs::write(&root, source).expect("write unsupported daemon source");

        let engine = ash_engine::Engine::new()
            .build()
            .expect("build unsupported daemon Engine");
        let result = admit_daemon_source(&engine, &root, source, "main");

        assert!(
            result.is_err(),
            "daemon admission must not reinterpret an unsupported canonical callable through the legacy route"
        );
    }

    #[test]
    fn task_2064_daemon_rejects_parseable_invalid_import_without_fallback() {
        let fixture = tempfile::tempdir().expect("create invalid-import daemon fixture");
        let root = fixture.path().join("main.ash");
        let source = "use crate::missing::serve; fn main() -> Int { 42 }";
        fs::write(&root, source).expect("write invalid-import daemon source");

        let engine = ash_engine::Engine::new()
            .build()
            .expect("build invalid-import daemon Engine");
        let result = admit_daemon_source(&engine, &root, source, "main");

        assert!(
            result.is_err(),
            "daemon admission must not reinterpret a canonical invalid import through the legacy route"
        );
    }

    #[test]
    fn task_2064_daemon_execution_uses_canonical_inline_module_route() {
        let fixture = tempfile::tempdir().expect("create daemon inline fixture");
        let root = fixture.path().join("main.ash");
        let source = "crate app; pub mod api { pub fn serve() -> Int { 42 } } use crate::api::serve as remote; fn main() -> Int { remote() }";
        fs::write(&root, source).expect("write daemon inline root");

        let engine = ash_engine::Engine::new()
            .build()
            .expect("build daemon inline execution Engine");
        let admitted = admit_daemon_source(&engine, &root, source, "main")
            .expect("daemon inline source must reach canonical linked admission");
        let (request, _cancellation) = engine
            .new_admitted_program_request(&admitted, None)
            .expect("daemon Engine mints one inline admitted request");
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build daemon inline test runtime");
        let terminal = runtime
            .block_on(submit_admitted_program(&engine, &request))
            .expect("daemon adapter executes inline canonical request");

        assert_eq!(
            terminal,
            CanonicalTerminalEnvelopeV1::returned(Value::Int(42))
        );
    }

    #[test]
    fn task_2064_daemon_execution_uses_canonical_ordinary_root_route() {
        let fixture = tempfile::tempdir().expect("create daemon ordinary-root fixture");
        let root = fixture.path().join("main.ash");
        let source = "crate app; fn main() -> Int { 42 }";
        fs::write(&root, source).expect("write daemon ordinary-root source");

        let engine = ash_engine::Engine::new()
            .build()
            .expect("build daemon ordinary-root Engine");
        let admitted = admit_daemon_source(&engine, &root, source, "main")
            .expect("daemon ordinary root must reach canonical linked admission");
        let (request, _cancellation) = engine
            .new_admitted_program_request(&admitted, None)
            .expect("daemon Engine mints one ordinary-root admitted request");
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build daemon ordinary-root test runtime");
        let terminal = runtime
            .block_on(submit_admitted_program(&engine, &request))
            .expect("daemon adapter executes ordinary canonical request");

        assert_eq!(
            terminal,
            CanonicalTerminalEnvelopeV1::returned(Value::Int(42))
        );
    }

    #[test]
    fn task_2064_daemon_execution_uses_canonical_modulo_route() {
        let fixture = tempfile::tempdir().expect("create daemon modulo fixture");
        let root = fixture.path().join("main.ash");
        let source = "crate app; fn main() -> Int { 7 % 3 }";
        fs::write(&root, source).expect("write daemon modulo source");

        let engine = ash_engine::Engine::new()
            .build()
            .expect("build daemon modulo Engine");
        let admitted = admit_daemon_source(&engine, &root, source, "main")
            .expect("daemon modulo root must reach canonical linked admission");
        let (request, _cancellation) = engine
            .new_admitted_program_request(&admitted, None)
            .expect("daemon Engine mints one modulo admitted request");
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build daemon modulo test runtime");
        let terminal = runtime
            .block_on(submit_admitted_program(&engine, &request))
            .expect("daemon adapter executes modulo canonical request");

        assert_eq!(
            terminal,
            CanonicalTerminalEnvelopeV1::returned(Value::Int(1))
        );
    }

    #[test]
    fn task_2064_daemon_execution_uses_canonical_record_field_call_route() {
        let fixture = tempfile::tempdir().expect("create daemon record-field-call fixture");
        let root = fixture.path().join("main.ash");
        let source = "crate app; fn helper() -> Int { 41 } fn main() -> Int { let person = { age: helper() }; person.age }";
        fs::write(&root, source).expect("write daemon record-field-call source");

        let engine = ash_engine::Engine::new()
            .build()
            .expect("build daemon record-field-call Engine");
        let admitted = admit_daemon_source(&engine, &root, source, "main")
            .expect("daemon record-field call root must reach canonical linked admission");
        let (request, _cancellation) = engine
            .new_admitted_program_request(&admitted, None)
            .expect("daemon Engine mints one record-field-call admitted request");
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build daemon record-field-call test runtime");
        let terminal = runtime
            .block_on(submit_admitted_program(&engine, &request))
            .expect("daemon adapter executes record field call canonical request");

        assert_eq!(
            terminal,
            CanonicalTerminalEnvelopeV1::returned(Value::Int(41))
        );
    }

    #[test]
    fn task_2064_daemon_execution_uses_canonical_nested_record_field_call_route() {
        let fixture = tempfile::tempdir().expect("create daemon nested record-field-call fixture");
        let root = fixture.path().join("main.ash");
        let source = "crate app; fn helper() -> Int { 41 } fn main() -> Int { let person = { inner: { age: helper() } }; person.inner.age }";
        fs::write(&root, source).expect("write daemon nested record-field-call source");

        let engine = ash_engine::Engine::new()
            .build()
            .expect("build daemon nested record-field-call Engine");
        let admitted = admit_daemon_source(&engine, &root, source, "main")
            .expect("daemon nested record field call root must reach canonical linked admission");
        let (request, _cancellation) = engine
            .new_admitted_program_request(&admitted, None)
            .expect("daemon Engine mints one nested record-field-call admitted request");
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build daemon nested record-field-call test runtime");
        let terminal = runtime
            .block_on(submit_admitted_program(&engine, &request))
            .expect("daemon adapter executes nested record field call canonical request");

        assert_eq!(
            terminal,
            CanonicalTerminalEnvelopeV1::returned(Value::Int(41))
        );
    }

    #[test]
    fn task_2064_daemon_execution_uses_canonical_record_field_expression_call_route() {
        let fixture =
            tempfile::tempdir().expect("create daemon record-field-expression-call fixture");
        let root = fixture.path().join("main.ash");
        let source = "crate app; fn helper() -> Int { 40 } fn main() -> Int { let person = { age: helper() + 1 }; person.age }";
        fs::write(&root, source).expect("write daemon record-field-expression-call source");

        let engine = ash_engine::Engine::new()
            .build()
            .expect("build daemon record-field-expression-call Engine");
        let admitted = admit_daemon_source(&engine, &root, source, "main")
            .expect("daemon record field expression root must reach canonical linked admission");
        let (request, _cancellation) = engine
            .new_admitted_program_request(&admitted, None)
            .expect("daemon Engine mints one record-field-expression admitted request");
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build daemon record-field-expression test runtime");
        let terminal = runtime
            .block_on(submit_admitted_program(&engine, &request))
            .expect("daemon adapter executes record field expression canonical request");

        assert_eq!(
            terminal,
            CanonicalTerminalEnvelopeV1::returned(Value::Int(41))
        );
    }

    #[test]
    fn task_2064_daemon_keeps_role_policy_stubs_out_of_callable_route() {
        let fixture = tempfile::tempdir().expect("create daemon role-policy fixture");
        let root = fixture.path().join("main.ash");
        let source = "crate app; pub mod api { pub role reviewer { capabilities: [], obligations: [] } pub policy Access { marker: Int } pub fn serve() -> Int { 42 } } use crate::api::serve as remote; fn main() -> Int { remote() }";
        fs::write(&root, source).expect("write daemon role-policy root");

        let engine = ash_engine::Engine::new()
            .build()
            .expect("build daemon role-policy Engine");
        let admitted = admit_daemon_source(&engine, &root, source, "main")
            .expect("daemon role/policy stubs must not block canonical admission");
        let (request, _cancellation) = engine
            .new_admitted_program_request(&admitted, None)
            .expect("daemon Engine mints one role-policy admitted request");
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build daemon role-policy test runtime");
        let terminal = runtime
            .block_on(submit_admitted_program(&engine, &request))
            .expect("daemon adapter executes canonical callable route");

        assert_eq!(
            terminal,
            CanonicalTerminalEnvelopeV1::returned(Value::Int(42))
        );
    }

    #[test]
    fn task_2064_daemon_allows_metadata_only_role_policy_child_module() {
        let fixture = tempfile::tempdir().expect("create daemon metadata-only child fixture");
        let root = fixture.path().join("main.ash");
        let source = "crate app; pub mod api; fn main() -> Int { 42 }";
        fs::write(&root, source).expect("write daemon metadata-only child root");
        fs::write(
            fixture.path().join("api.ash"),
            "pub role reviewer { capabilities: [], obligations: [] } pub policy Access { marker: Int }",
        )
        .expect("write daemon metadata-only role-policy child");

        let engine = ash_engine::Engine::new()
            .build()
            .expect("build daemon metadata-only child Engine");
        let admitted = admit_daemon_source(&engine, &root, source, "main")
            .expect("metadata-only role/policy child must not block canonical admission");
        let (request, _cancellation) = engine
            .new_admitted_program_request(&admitted, None)
            .expect("daemon Engine mints one metadata-only child request");
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build daemon metadata-only child runtime");
        let terminal = runtime
            .block_on(submit_admitted_program(&engine, &request))
            .expect("daemon adapter executes root with metadata-only child");

        assert_eq!(
            terminal,
            CanonicalTerminalEnvelopeV1::returned(Value::Int(42))
        );
    }

    #[test]
    fn task_2064_daemon_allows_handler_only_child_module() {
        let fixture = tempfile::tempdir().expect("create daemon handler-only child fixture");
        let root = fixture.path().join("main.ash");
        let source = "crate app; pub mod api; fn main() -> Int { 42 }";
        fs::write(&root, source).expect("write daemon handler-only child root");
        fs::write(
            fixture.path().join("api.ash"),
            r#"interface Clock<T> { sleep(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = milliseconds }
handler trap_sleep(comp: () -> { TestClock::sleep } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => resume(ms),
        done(value) => value,
    }
}"#,
        )
        .expect("write daemon handler-only child");

        let engine = ash_engine::Engine::new()
            .build()
            .expect("build daemon handler-only child Engine");
        let admitted = admit_daemon_source(&engine, &root, source, "main")
            .expect("handler-only child must not block daemon canonical admission");
        let (request, _cancellation) = engine
            .new_admitted_program_request(&admitted, None)
            .expect("daemon Engine mints one handler-only child request");
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build daemon handler-only child runtime");
        let terminal = runtime
            .block_on(submit_admitted_program(&engine, &request))
            .expect("daemon adapter executes root with handler-only child");

        assert_eq!(
            terminal,
            CanonicalTerminalEnvelopeV1::returned(Value::Int(42))
        );
    }
}
