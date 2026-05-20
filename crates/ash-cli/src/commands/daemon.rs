//! Local daemon control plane for the alpha RuntimeKernel host mode.

use anyhow::{Context, Result, anyhow, bail};
use ash_core::runtime_kernel::{
    AdmissionIdentity, ArtifactVersion, ProviderRegistryIdentity, RuntimeArtifactCacheKey,
    RuntimeConfigId, RuntimeEngineRelationship, RuntimeHostMode, RuntimeKernelIdentity,
    RuntimeProfileId, RuntimeProfileIdentity, RuntimeRootSet, RuntimeRootSetId,
    WorkflowArtifactIdentity, WorkflowDefinitionIdentity, WorkflowInstanceIdentity,
};
use ash_engine::Engine;
use ash_provenance::Hash as ProvenanceHash;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
enum DaemonRequest {
    List,
    Start { workflow: String },
    Status { instance_id: String },
    Cancel { instance_id: String },
    Reload,
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
}

#[derive(Debug, Clone, Serialize)]
struct InstanceRecord {
    instance_id: String,
    workflow: String,
    status: InstanceStatus,
    definition_id: String,
    artifact_id: String,
    artifact_version: String,
    source_hash: String,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum InstanceStatus {
    Admitted,
    Cancelled,
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
                    ArtifactVersion::new("source-check-summary-v1"),
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

    fn start(&mut self, workflow: &str) -> Result<Value> {
        let definition = self
            .definitions
            .iter()
            .find(|definition| definition.workflow == workflow)
            .ok_or_else(|| anyhow!("workflow definition not indexed: {workflow}"))?;

        let profile = RuntimeProfileIdentity::new(
            self.profile_id.clone(),
            self.config_id.clone(),
            vec!["ash daemon alpha default profile/config".to_string()],
        );
        let instance = WorkflowInstanceIdentity::admit(
            RuntimeHostMode::Daemon,
            WorkflowDefinitionIdentity::new(
                self.root_id.clone(),
                definition.relative_module_path.clone(),
                definition.workflow.clone(),
                self.profile_id.clone(),
                self.config_id.clone(),
                definition.source_hash.clone(),
            )
            .id,
            WorkflowArtifactIdentity::new(
                WorkflowDefinitionIdentity::new(
                    self.root_id.clone(),
                    definition.relative_module_path.clone(),
                    definition.workflow.clone(),
                    self.profile_id.clone(),
                    self.config_id.clone(),
                    definition.source_hash.clone(),
                )
                .id,
                RuntimeArtifactCacheKey::new(
                    self.root_id.clone(),
                    self.profile_id.clone(),
                    self.config_id.clone(),
                    definition.source_hash.clone(),
                    definition.check_summary_hash.clone(),
                    ArtifactVersion::new(definition.artifact_version.clone()),
                ),
                ArtifactVersion::new(definition.artifact_version.clone()),
            )
            .id,
            profile,
            self.provider_registry.clone(),
            AdmissionIdentity::empty(),
        );
        let instance_id = instance.id.0.to_string();
        let record = InstanceRecord {
            instance_id: instance_id.clone(),
            workflow: definition.workflow.clone(),
            status: InstanceStatus::Admitted,
            definition_id: definition.definition_id.clone(),
            artifact_id: definition.artifact_id.clone(),
            artifact_version: definition.artifact_version.clone(),
            source_hash: definition.source_hash.clone(),
        };
        self.instances.insert(instance_id.clone(), record);
        Ok(json!({
            "ok": true,
            "host_mode": "Daemon",
            "status": "admitted",
            "execution": "not_started_alpha_record_only",
            "instance_id": instance_id,
            "workflow": workflow,
            "definition_id": definition.definition_id,
            "artifact_id": definition.artifact_id,
            "artifact_version": definition.artifact_version,
            "provider_registry": provider_registry_json(&self.provider_registry),
        }))
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
            "definition_id": instance.definition_id,
            "artifact_id": instance.artifact_id,
            "artifact_version": instance.artifact_version,
            "source_hash": instance.source_hash,
        }))
    }

    fn cancel(&mut self, instance_id: &str) -> Result<Value> {
        let instance = self
            .instances
            .get_mut(instance_id)
            .ok_or_else(|| anyhow!("workflow instance not found: {instance_id}"))?;
        let class = if instance.status == InstanceStatus::Cancelled {
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
        DaemonRequest::Start { workflow } => state.start(&workflow),
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
        let engine = Engine::new()
            .build()
            .context("parse/check/index failure: failed to build engine")?;
        let mut checked_workflow = engine
            .parse_file(&path)
            .with_context(|| format!("parse/check/index failure in {}", path.display()))?;
        engine
            .check(&mut checked_workflow)
            .with_context(|| format!("parse/check/index failure in {}", path.display()))?;
        let relative_module_path = path
            .strip_prefix(root)
            .unwrap_or(path.as_path())
            .to_string_lossy()
            .replace('\\', "/");
        let workflow_name = workflow.name.to_string();
        let source_hash = stable_digest(&[&source, &relative_module_path, &workflow_name]);
        let check_summary_hash = stable_digest(&[
            &source_hash,
            &workflow_name,
            "parse-surface-file-source-summary",
        ]);
        let artifact_version = "source-check-summary-v1".to_string();
        let definition = WorkflowDefinitionIdentity::new(
            root_id.clone(),
            relative_module_path.clone(),
            workflow_name.clone(),
            profile_id.clone(),
            config_id.clone(),
            source_hash.clone(),
        );
        let cache_key = RuntimeArtifactCacheKey::new(
            root_id.clone(),
            profile_id.clone(),
            config_id.clone(),
            source_hash.clone(),
            check_summary_hash.clone(),
            ArtifactVersion::new(artifact_version.clone()),
        );
        let artifact = WorkflowArtifactIdentity::new(
            definition.id.clone(),
            cache_key,
            ArtifactVersion::new(artifact_version.clone()),
        );
        definitions.push(DefinitionRecord {
            workflow: workflow_name,
            relative_module_path,
            definition_id: definition.id.as_str().to_string(),
            artifact_id: artifact.id.as_str().to_string(),
            artifact_version,
            source_hash,
            check_summary_hash,
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
    } else if message.contains("not found") {
        "not_found"
    } else {
        "request_failure"
    }
}

fn stable_digest(parts: &[&str]) -> String {
    let mut bytes = Vec::new();
    for part in parts {
        bytes.extend_from_slice(part.len().to_string().as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(part.as_bytes());
        bytes.push(0xff);
    }
    format!("sha256:{}", ProvenanceHash::from_bytes(&bytes))
}

fn host_mode_label(host_mode: RuntimeHostMode) -> &'static str {
    match host_mode {
        RuntimeHostMode::Entry => "Entry",
        RuntimeHostMode::OneShot => "OneShot",
        RuntimeHostMode::Trace => "Trace",
        RuntimeHostMode::Daemon => "Daemon",
    }
}
