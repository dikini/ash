//! User-local Ash toolchain and deployment manager.

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, ExitCode};

use anyhow::{Context, Result, anyhow, bail};
use chrono::Utc;
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

const TOOLCHAIN_ARCHIVE_SCHEMA_VERSION: u32 = 1;

/// User-local XDG-compatible path set for Ash installs.
#[derive(Debug, Clone)]
pub struct AshgrovePaths {
    home: PathBuf,
    data_home: PathBuf,
    config_home: PathBuf,
    cache_home: PathBuf,
    state_home: PathBuf,
}

impl AshgrovePaths {
    /// Build paths from explicit roots, applying XDG defaults for missing roots.
    #[must_use]
    pub fn from_roots(
        home: PathBuf,
        data_home: Option<PathBuf>,
        config_home: Option<PathBuf>,
        cache_home: Option<PathBuf>,
        state_home: Option<PathBuf>,
    ) -> Self {
        Self {
            data_home: data_home.unwrap_or_else(|| home.join(".local/share")),
            config_home: config_home.unwrap_or_else(|| home.join(".config")),
            cache_home: cache_home.unwrap_or_else(|| home.join(".cache")),
            state_home: state_home.unwrap_or_else(|| home.join(".local/state")),
            home,
        }
    }

    /// Build paths from the current process environment.
    ///
    /// # Errors
    ///
    /// Returns an error when `HOME` is not available.
    pub fn from_env() -> Result<Self> {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| anyhow!("HOME is required for user-local ashgrove paths"))?;
        Ok(Self::from_roots(
            home,
            std::env::var_os("XDG_DATA_HOME").map(PathBuf::from),
            std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
            std::env::var_os("XDG_CACHE_HOME").map(PathBuf::from),
            std::env::var_os("XDG_STATE_HOME").map(PathBuf::from),
        ))
    }

    /// Stable launcher directory.
    #[must_use]
    pub fn launcher_bin(&self) -> PathBuf {
        self.home.join(".local/bin")
    }

    /// Installed immutable toolchain directory root.
    #[must_use]
    pub fn toolchains_dir(&self) -> PathBuf {
        self.data_home.join("ash/toolchains")
    }

    /// Ash configuration directory.
    #[must_use]
    pub fn config_dir(&self) -> PathBuf {
        self.config_home.join("ash")
    }

    /// Ash cache directory.
    #[must_use]
    pub fn cache_dir(&self) -> PathBuf {
        self.cache_home.join("ash")
    }

    /// Ash state directory.
    #[must_use]
    pub fn state_dir(&self) -> PathBuf {
        self.state_home.join("ash")
    }

    fn toolchain_dir(&self, id: &ToolchainId) -> PathBuf {
        self.toolchains_dir().join(id.as_str())
    }
}

/// Validated first-slice toolchain id.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct ToolchainId(String);

impl ToolchainId {
    /// Parse a toolchain id, rejecting path-like or empty values.
    ///
    /// # Errors
    ///
    /// Returns an error if `value` is empty or contains path separators,
    /// traversal, or control characters.
    pub fn parse(value: &str) -> Result<Self> {
        if value.is_empty()
            || value.contains('/')
            || value.contains('\\')
            || value.contains("..")
            || value.chars().any(char::is_control)
        {
            bail!("invalid toolchain id '{value}'");
        }
        if !value.starts_with("ash-") {
            bail!("toolchain id '{value}' must start with 'ash-'");
        }
        Ok(Self(value.to_string()))
    }

    /// Borrow the validated id.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for ToolchainId {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(serde::de::Error::custom)
    }
}

/// User selector metadata with preserved reserved fields.
#[derive(Debug, Clone)]
pub struct SelectorMetadata {
    value: toml::Value,
    default: Option<ToolchainId>,
    projects: BTreeMap<String, ToolchainId>,
}

impl SelectorMetadata {
    /// Create empty selector metadata.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            value: toml::Value::Table(toml::map::Map::new()),
            default: None,
            projects: BTreeMap::new(),
        }
    }

    /// Read selector metadata from a TOML file.
    ///
    /// # Errors
    ///
    /// Returns an error when reading, parsing, or toolchain-id validation fails.
    pub fn read_from_path(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path).context("read selector metadata")?;
        Self::from_toml_str(&text)
    }

    /// Parse selector metadata from TOML.
    ///
    /// # Errors
    ///
    /// Returns an error when parsing or toolchain-id validation fails.
    pub fn from_toml_str(text: &str) -> Result<Self> {
        let value: toml::Value = toml::from_str(text).context("parse selector metadata")?;
        let default = value
            .get("default")
            .and_then(toml::Value::as_str)
            .map(ToolchainId::parse)
            .transpose()?;
        let projects = value
            .get("projects")
            .and_then(toml::Value::as_table)
            .map(|table| {
                table
                    .iter()
                    .map(|(path, id)| {
                        let id = id
                            .as_str()
                            .ok_or_else(|| {
                                anyhow!("project selector for '{path}' must be a string")
                            })
                            .and_then(ToolchainId::parse)?;
                        Ok((path.clone(), id))
                    })
                    .collect::<Result<BTreeMap<_, _>>>()
            })
            .transpose()?
            .unwrap_or_default();
        Ok(Self {
            value,
            default,
            projects,
        })
    }

    /// Write selector metadata to a TOML file while preserving reserved fields.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization or writing fails.
    pub fn write_to_path(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("create selector parent")?;
        }
        fs::write(path, self.to_toml_string()?).context("write selector metadata")
    }

    /// Render selector metadata to TOML.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization fails.
    pub fn to_toml_string(&self) -> Result<String> {
        let mut value = self.value.clone();
        let table = value
            .as_table_mut()
            .ok_or_else(|| anyhow!("selector metadata must be a TOML table"))?;
        match &self.default {
            Some(id) => {
                table.insert(
                    "default".to_string(),
                    toml::Value::String(id.as_str().to_string()),
                );
            }
            None => {
                table.remove("default");
            }
        }
        let mut projects = toml::map::Map::new();
        for (path, id) in &self.projects {
            projects.insert(path.clone(), toml::Value::String(id.as_str().to_string()));
        }
        if !projects.is_empty() {
            table.insert("projects".to_string(), toml::Value::Table(projects));
        }
        toml::to_string(&value).context("serialize selector metadata")
    }

    /// Set the user default toolchain.
    pub fn set_default(&mut self, id: ToolchainId) {
        self.default = Some(id);
    }

    /// Record a known project root and selected toolchain.
    pub fn record_project_toolchain(&mut self, project: &Path, id: ToolchainId) {
        self.projects.insert(project.display().to_string(), id);
    }

    /// Borrow the default selector.
    #[must_use]
    pub fn default(&self) -> Option<&ToolchainId> {
        self.default.as_ref()
    }

    /// Borrow a known project selector.
    #[must_use]
    pub fn project_toolchain(&self, project: &Path) -> Option<&ToolchainId> {
        self.projects.get(&project.display().to_string())
    }

    fn project_pins(&self) -> impl Iterator<Item = &ToolchainId> {
        self.projects.values()
    }
}

/// Request used by a stable launcher shim to resolve a versioned tool binary.
#[derive(Debug, Clone)]
pub struct LauncherDispatchRequest {
    tool_name: String,
    explicit_toolchain: Option<ToolchainId>,
    project: Option<PathBuf>,
}

impl LauncherDispatchRequest {
    /// Create a dispatch request for a bundled standard tool such as `ash` or `ashgrove`.
    #[must_use]
    pub fn new(tool_name: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            explicit_toolchain: None,
            project: None,
        }
    }

    /// Select a toolchain explicitly before project and user-default selectors are considered.
    #[must_use]
    pub fn with_explicit_toolchain(mut self, id: ToolchainId) -> Self {
        self.explicit_toolchain = Some(id);
        self
    }

    /// Set the project root whose `ash.toml` may contain a `[toolchain] ash = "..."`
    /// selector.
    #[must_use]
    pub fn with_project(mut self, project: impl AsRef<Path>) -> Self {
        self.project = Some(project.as_ref().to_path_buf());
        self
    }

    /// Borrow the requested tool name.
    #[must_use]
    pub fn tool_name(&self) -> &str {
        &self.tool_name
    }
}

/// Selector source used to choose a launcher dispatch target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LauncherSelectionSource {
    /// An explicit launcher override selected the toolchain.
    ExplicitOverride,
    /// The project `ash.toml` selected the toolchain.
    ProjectPin,
    /// The user default selector selected the toolchain.
    UserDefault,
}

impl LauncherSelectionSource {
    fn diagnostic_label(self) -> &'static str {
        match self {
            Self::ExplicitOverride => "explicit override",
            Self::ProjectPin => "project pin",
            Self::UserDefault => "user default",
        }
    }
}

/// Resolved launcher dispatch target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherDispatch {
    toolchain_id: ToolchainId,
    tool_name: String,
    tool_path: PathBuf,
    selection_source: LauncherSelectionSource,
}

impl LauncherDispatch {
    /// Borrow the selected toolchain id.
    #[must_use]
    pub fn toolchain_id(&self) -> &ToolchainId {
        &self.toolchain_id
    }

    /// Borrow the requested standard tool name.
    #[must_use]
    pub fn tool_name(&self) -> &str {
        &self.tool_name
    }

    /// Borrow the selected versioned binary path.
    #[must_use]
    pub fn tool_path(&self) -> &Path {
        &self.tool_path
    }

    /// Borrow the selector source that selected the toolchain.
    #[must_use]
    pub fn selection_source(&self) -> LauncherSelectionSource {
        self.selection_source
    }
}

/// Install stable user-local launcher shims for bundled Ash tools.
///
/// The installed shims call `dispatcher` with ashgrove's hidden launcher
/// dispatch command, which resolves the selected immutable toolchain before
/// executing the versioned tool binary.
///
/// # Errors
///
/// Returns an error when the dispatcher path is not a file or the launcher
/// directory/script files cannot be created.
pub fn install_launcher_shims(paths: &AshgrovePaths, dispatcher: &Path) -> Result<()> {
    if !dispatcher.is_file() {
        bail!(
            "launcher dispatcher is not a file: {}",
            dispatcher.display()
        );
    }
    fs::create_dir_all(paths.launcher_bin()).context("create launcher bin directory")?;
    for tool in ["ash", "ashgrove"] {
        write_launcher_shim(paths, dispatcher, tool)?;
    }
    Ok(())
}

fn install_launcher_shims_from_current_exe(paths: &AshgrovePaths) -> Result<()> {
    let current_exe = std::env::current_exe().context("resolve current ashgrove executable")?;
    fs::create_dir_all(paths.launcher_bin()).context("create launcher bin directory")?;
    let dispatcher = paths.launcher_bin().join(".ashgrove-dispatcher");
    install_executable_copy(&current_exe, &dispatcher)?;
    install_launcher_shims(paths, &dispatcher)
}

fn install_executable_copy(source: &Path, target: &Path) -> Result<()> {
    let bytes = fs::read(source)
        .with_context(|| format!("read launcher dispatcher source {}", source.display()))?;
    write_executable_file_atomically(target, ".ashgrove-dispatcher.tmp-", &bytes)
        .with_context(|| format!("publish stable launcher dispatcher {}", target.display()))
}

fn write_launcher_shim(paths: &AshgrovePaths, dispatcher: &Path, tool: &str) -> Result<()> {
    validate_launcher_tool_name(tool)?;
    let shim_path = paths.launcher_bin().join(tool);
    let script = launcher_shim_script(dispatcher, tool);
    write_executable_file_atomically(&shim_path, &format!(".{tool}.tmp-"), script.as_bytes())
        .with_context(|| format!("publish launcher {}", shim_path.display()))
}

fn write_executable_file_atomically(
    target: &Path,
    temp_prefix: &str,
    contents: &[u8],
) -> Result<()> {
    let parent = target.parent().ok_or_else(|| {
        anyhow!(
            "launcher target must have a parent directory: {}",
            target.display()
        )
    })?;
    let mut temp = tempfile::Builder::new()
        .prefix(temp_prefix)
        .tempfile_in(parent)
        .with_context(|| format!("create temporary launcher file in {}", parent.display()))?;
    temp.as_file_mut()
        .write_all(contents)
        .with_context(|| format!("write temporary launcher file {}", temp.path().display()))?;
    temp.as_file_mut()
        .sync_all()
        .with_context(|| format!("sync temporary launcher file {}", temp.path().display()))?;
    make_launcher_executable(temp.path())?;
    let persisted = temp
        .persist(target)
        .map_err(|error| error.error)
        .with_context(|| format!("publish launcher file {}", target.display()))?;
    drop(persisted);
    Ok(())
}

fn launcher_shim_script(dispatcher: &Path, tool: &str) -> String {
    format!(
        "#!/bin/sh\nexec {} __launcher-dispatch --tool {} -- \"$@\"\n",
        shell_quote_path(dispatcher),
        shell_quote(tool),
    )
}

fn shell_quote_path(path: &Path) -> String {
    shell_quote(&path.display().to_string())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(unix)]
fn make_launcher_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)
        .with_context(|| format!("read launcher metadata {}", path.display()))?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
        .with_context(|| format!("make launcher executable {}", path.display()))
}

#[cfg(not(unix))]
fn make_launcher_executable(_path: &Path) -> Result<()> {
    Ok(())
}

/// Resolve stable launcher dispatch without creating or executing a shim.
///
/// Resolution order matches SPEC-073: explicit override, project pin, then user
/// default. The returned target is validated against installed toolchain
/// metadata and install records.
///
/// # Errors
///
/// Returns an error when the selector metadata is invalid, no suitable selector
/// exists, the selected toolchain is not installed, or the requested tool is not
/// part of the selected toolchain manifest.
pub fn resolve_launcher_dispatch(
    paths: &AshgrovePaths,
    request: LauncherDispatchRequest,
) -> Result<LauncherDispatch> {
    validate_launcher_tool_name(request.tool_name())?;
    let (id, selection_source) = select_launcher_toolchain(paths, &request)?;
    ensure_dispatch_toolchain_installed(paths, &id, selection_source)?;
    let manifest = installed_toolchain_manifest(paths, &id).with_context(|| {
        format!(
            "validate selected toolchain '{}' from {}",
            id.as_str(),
            selection_source.diagnostic_label()
        )
    })?;
    verify_install_record_any_source(&paths.toolchain_dir(&id), &id).with_context(|| {
        format!(
            "validate selected toolchain '{}' from {}",
            id.as_str(),
            selection_source.diagnostic_label()
        )
    })?;
    let tool = manifest.required_tool(request.tool_name()).ok_or_else(|| {
        anyhow!(
            "selected toolchain '{}' from {} does not provide required tool '{}'",
            id.as_str(),
            selection_source.diagnostic_label(),
            request.tool_name()
        )
    })?;
    let toolchain_root = paths.toolchain_dir(&id);
    let tool_path = validate_contained_standard_tool_path(&toolchain_root, tool.path())?;
    Ok(LauncherDispatch {
        toolchain_id: id,
        tool_name: request.tool_name,
        tool_path,
        selection_source,
    })
}

fn validate_launcher_tool_name(tool_name: &str) -> Result<()> {
    if tool_name.is_empty()
        || tool_name.contains('/')
        || tool_name.contains('\\')
        || tool_name.contains("..")
        || tool_name.chars().any(char::is_control)
    {
        bail!("invalid launcher tool name '{tool_name}'");
    }
    Ok(())
}

fn validate_contained_standard_tool_path(root: &Path, path: &str) -> Result<PathBuf> {
    let rel = validate_relative_toolchain_path(path, "standard tool path")?;
    let tool_path = root.join(rel);
    let metadata = fs::symlink_metadata(&tool_path).with_context(|| {
        format!(
            "validate standard tool path {} inside toolchain root {}",
            tool_path.display(),
            root.display()
        )
    })?;
    if metadata.file_type().is_symlink() {
        bail!(
            "standard tool path must stay inside the toolchain root: {}",
            tool_path.display()
        );
    }
    if !metadata.is_file() {
        bail!(
            "standard tool path must be a file inside the toolchain root: {}",
            tool_path.display()
        );
    }
    let canonical_root = root.canonicalize().with_context(|| {
        format!(
            "canonicalize selected toolchain root {} for standard tool path validation",
            root.display()
        )
    })?;
    let canonical_tool = tool_path.canonicalize().with_context(|| {
        format!(
            "canonicalize standard tool path {} inside toolchain root {}",
            tool_path.display(),
            root.display()
        )
    })?;
    if !canonical_tool.starts_with(&canonical_root) {
        bail!(
            "standard tool path must stay inside the toolchain root: {}",
            tool_path.display()
        );
    }
    Ok(tool_path)
}

fn select_launcher_toolchain(
    paths: &AshgrovePaths,
    request: &LauncherDispatchRequest,
) -> Result<(ToolchainId, LauncherSelectionSource)> {
    if let Some(id) = &request.explicit_toolchain {
        return Ok((id.clone(), LauncherSelectionSource::ExplicitOverride));
    }
    if let Some(project) = &request.project
        && let Some(pin) = project_toolchain_pin(project)?
    {
        return Ok((
            ToolchainId::parse(&pin)?,
            LauncherSelectionSource::ProjectPin,
        ));
    }
    if let Some(default) = read_default(paths)? {
        return Ok((default, LauncherSelectionSource::UserDefault));
    }
    bail!(
        "no suitable Ash toolchain is installed; add [toolchain] ash to ash.toml or run `ashgrove default <toolchain-id>`"
    );
}

fn ensure_dispatch_toolchain_installed(
    paths: &AshgrovePaths,
    id: &ToolchainId,
    selection_source: LauncherSelectionSource,
) -> Result<()> {
    let toolchain_dir = paths.toolchain_dir(id);
    let metadata = match fs::symlink_metadata(&toolchain_dir) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            bail!(
                "selected toolchain '{}' from {} is not installed; install it or change the selector",
                id.as_str(),
                selection_source.diagnostic_label()
            );
        }
        Err(error) => {
            return Err(error).with_context(|| {
                format!(
                    "inspect selected toolchain '{}' from {}",
                    id.as_str(),
                    selection_source.diagnostic_label()
                )
            });
        }
    };
    if metadata.file_type().is_symlink() {
        bail!(
            "selected toolchain '{}' from {} is a symlink; install a real directory under {}",
            id.as_str(),
            selection_source.diagnostic_label(),
            paths.toolchains_dir().display()
        );
    }
    if !metadata.is_dir() {
        bail!(
            "selected toolchain '{}' from {} is not installed; install it or change the selector",
            id.as_str(),
            selection_source.diagnostic_label()
        );
    }
    Ok(())
}

/// Typed first-slice manifest for an immutable Ash toolchain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainManifest {
    toolchain_id: ToolchainId,
    version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    archive_schema_version: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    target_triple: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    stdlib: Option<StdlibMetadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    runtime_support: Option<RuntimeSupportMetadata>,
    #[serde(default)]
    standard_tools: Vec<StandardToolMetadata>,
    #[serde(flatten)]
    extra: BTreeMap<String, toml::Value>,
}

impl ToolchainManifest {
    /// Create a manifest with the required identity and version.
    #[must_use]
    pub fn new(id: ToolchainId, version: impl Into<String>) -> Self {
        Self {
            toolchain_id: id,
            version: version.into(),
            archive_schema_version: None,
            target_triple: None,
            source_kind: None,
            stdlib: None,
            runtime_support: None,
            standard_tools: Vec::new(),
            extra: BTreeMap::new(),
        }
    }

    /// Create the smallest manifest used by first-slice staging tests.
    #[must_use]
    pub fn minimal(
        id: ToolchainId,
        version: impl Into<String>,
        target: impl Into<String>,
        source_kind: impl Into<String>,
    ) -> Self {
        let version = version.into();
        Self::new(id, version.clone())
            .with_target_triple(target)
            .with_source_kind(source_kind)
            .with_stdlib(StdlibMetadata::new("", "lib/ash/std"))
            .with_runtime_support(RuntimeSupportMetadata::required(
                version,
                "lib/ash/std/src/runtime",
            ))
            .with_tool(StandardToolMetadata::required("ash", "bin/ash"))
            .with_tool(StandardToolMetadata::required("ashgrove", "bin/ashgrove"))
    }

    /// Add the target triple.
    #[must_use]
    pub fn with_target_triple(mut self, target: impl Into<String>) -> Self {
        self.target_triple = Some(target.into());
        self
    }

    /// Add the source kind.
    #[must_use]
    pub fn with_source_kind(mut self, source_kind: impl Into<String>) -> Self {
        self.source_kind = Some(source_kind.into());
        self
    }

    /// Add the release archive schema version.
    #[must_use]
    pub fn with_archive_schema_version(mut self, version: u32) -> Self {
        self.archive_schema_version = Some(version);
        self
    }

    /// Add stdlib metadata.
    #[must_use]
    pub fn with_stdlib(mut self, stdlib: StdlibMetadata) -> Self {
        self.stdlib = Some(stdlib);
        self
    }

    /// Add runtime-support payload metadata.
    #[must_use]
    pub fn with_runtime_support(mut self, runtime_support: RuntimeSupportMetadata) -> Self {
        self.runtime_support = Some(runtime_support);
        self
    }

    /// Add a standard tool entry.
    #[must_use]
    pub fn with_tool(mut self, tool: StandardToolMetadata) -> Self {
        self.standard_tools.push(tool);
        self
    }

    /// Parse from TOML while preserving unknown future-compatible fields.
    ///
    /// # Errors
    ///
    /// Returns an error when TOML parsing or toolchain-id validation fails.
    pub fn from_toml_str(text: &str) -> Result<Self> {
        let manifest: Self = toml::from_str(text).context("parse toolchain manifest")?;
        if manifest.stdlib.is_none() {
            bail!("toolchain manifest missing stdlib metadata");
        }
        if manifest.runtime_support.is_none() {
            bail!("toolchain manifest missing runtime support metadata");
        }
        Ok(manifest)
    }

    /// Render TOML, including preserved unknown fields.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization fails.
    pub fn to_toml_string(&self) -> Result<String> {
        toml::to_string(self).context("serialize toolchain manifest")
    }

    /// Write TOML to `path`.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization or writing fails.
    pub fn write_to(&self, path: &Path) -> Result<()> {
        fs::write(path, self.to_toml_string()?).context("write toolchain manifest")
    }

    /// Borrow the manifest toolchain id.
    #[must_use]
    pub fn toolchain_id(&self) -> &ToolchainId {
        &self.toolchain_id
    }

    /// Borrow stdlib metadata.
    #[must_use]
    pub fn stdlib(&self) -> &StdlibMetadata {
        self.stdlib.as_ref().expect("stdlib metadata present")
    }

    /// Borrow runtime-support payload metadata.
    #[must_use]
    pub fn runtime_support(&self) -> &RuntimeSupportMetadata {
        self.runtime_support
            .as_ref()
            .expect("runtime support metadata present")
    }

    /// Iterate required tools.
    pub fn required_tools(&self) -> impl Iterator<Item = &StandardToolMetadata> {
        self.standard_tools.iter().filter(|tool| tool.required)
    }

    /// Find a required standard tool by name.
    #[must_use]
    pub fn required_tool(&self, name: &str) -> Option<&StandardToolMetadata> {
        self.required_tools().find(|tool| tool.name() == name)
    }

    /// Validate this manifest belongs to the expected toolchain id.
    ///
    /// # Errors
    ///
    /// Returns an error when the manifest id differs from `id`.
    pub fn validate_for_toolchain(&self, id: &ToolchainId) -> Result<()> {
        if &self.toolchain_id != id {
            bail!("manifest toolchain id does not match {}", id.as_str());
        }
        let expected_version = id
            .as_str()
            .strip_prefix("ash-")
            .and_then(|suffix| suffix.split('+').next())
            .unwrap_or_default();
        if self.version != expected_version {
            bail!(
                "manifest version '{}' does not match toolchain id {}",
                self.version,
                id.as_str()
            );
        }
        Ok(())
    }

    fn validate_archive_schema_version(&self) -> Result<()> {
        match self.archive_schema_version {
            Some(TOOLCHAIN_ARCHIVE_SCHEMA_VERSION) => Ok(()),
            Some(version) => bail!(
                "unsupported archive schema version {version}; expected {TOOLCHAIN_ARCHIVE_SCHEMA_VERSION}"
            ),
            None => bail!(
                "archive schema version is required; expected {TOOLCHAIN_ARCHIVE_SCHEMA_VERSION}"
            ),
        }
    }
}

/// Metadata for the bundled stdlib in a toolchain manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StdlibMetadata {
    version: String,
    path: String,
}

impl StdlibMetadata {
    /// Create stdlib metadata.
    #[must_use]
    pub fn new(version: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            path: path.into(),
        }
    }

    /// Borrow the stdlib version.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Borrow the stdlib path relative to the toolchain root.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
}

/// Metadata for the bundled runtime-support payload in a toolchain manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSupportMetadata {
    identity: String,
    path: String,
    required: bool,
}

impl RuntimeSupportMetadata {
    /// Create required runtime-support payload metadata for an Ash version.
    #[must_use]
    pub fn required(version: impl AsRef<str>, path: impl Into<String>) -> Self {
        Self {
            identity: format!("ash-runtime-support:{}", version.as_ref()),
            path: path.into(),
            required: true,
        }
    }

    /// Borrow the runtime-support identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }

    /// Borrow the runtime-support path relative to the toolchain root.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Whether this runtime-support payload is required for the toolchain.
    #[must_use]
    pub const fn is_required(&self) -> bool {
        self.required
    }
}

/// Metadata for a public standard tool bundled in a toolchain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardToolMetadata {
    name: String,
    path: String,
    required: bool,
}

impl StandardToolMetadata {
    /// Create a required standard-tool entry.
    #[must_use]
    pub fn required(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            required: true,
        }
    }

    /// Borrow the tool name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Borrow the tool path relative to the toolchain root.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
}

/// Typed first-slice install record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallRecord {
    toolchain_id: ToolchainId,
    source_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    archive_schema_version: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_rev: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    build_profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    target_triple: Option<String>,
    #[serde(default)]
    reproducible: bool,
    #[serde(flatten)]
    extra: BTreeMap<String, toml::Value>,
}

impl InstallRecord {
    /// Create a source-install record.
    #[must_use]
    pub fn source_install(id: ToolchainId) -> Self {
        Self::minimal(id, "source")
    }

    /// Create a minimal install record.
    #[must_use]
    pub fn minimal(id: ToolchainId, source_kind: impl Into<String>) -> Self {
        Self {
            toolchain_id: id,
            source_kind: source_kind.into(),
            archive_schema_version: None,
            source_rev: None,
            build_profile: None,
            target_triple: None,
            reproducible: false,
            extra: BTreeMap::new(),
        }
    }

    /// Record source revision.
    #[must_use]
    pub fn with_source_rev(mut self, rev: impl Into<String>) -> Self {
        self.source_rev = Some(rev.into());
        self
    }

    /// Record build profile.
    #[must_use]
    pub fn with_build_profile(mut self, profile: impl Into<String>) -> Self {
        self.build_profile = Some(profile.into());
        self
    }

    /// Record target triple.
    #[must_use]
    pub fn with_target_triple(mut self, target: impl Into<String>) -> Self {
        self.target_triple = Some(target.into());
        self
    }

    /// Record reproducibility state.
    #[must_use]
    pub fn with_reproducible(mut self, reproducible: bool) -> Self {
        self.reproducible = reproducible;
        self
    }

    /// Record the release archive schema version.
    #[must_use]
    pub fn with_archive_schema_version(mut self, version: u32) -> Self {
        self.archive_schema_version = Some(version);
        self
    }

    /// Parse from TOML while preserving unknown future-compatible fields.
    ///
    /// # Errors
    ///
    /// Returns an error when TOML parsing or toolchain-id validation fails.
    pub fn from_toml_str(text: &str) -> Result<Self> {
        toml::from_str(text).context("parse install record")
    }

    /// Render TOML, including preserved unknown fields.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization fails.
    pub fn to_toml_string(&self) -> Result<String> {
        toml::to_string(self).context("serialize install record")
    }

    /// Write TOML to `path`.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization or writing fails.
    pub fn write_to(&self, path: &Path) -> Result<()> {
        fs::write(path, self.to_toml_string()?).context("write install record")
    }

    /// Borrow the record toolchain id.
    #[must_use]
    pub fn toolchain_id(&self) -> &ToolchainId {
        &self.toolchain_id
    }

    /// Whether the install is reproducible.
    #[must_use]
    pub fn is_reproducible(&self) -> bool {
        self.reproducible
    }

    /// Borrow the source revision.
    #[must_use]
    pub fn source_rev(&self) -> Option<&str> {
        self.source_rev.as_deref()
    }

    /// Borrow the source kind.
    #[must_use]
    pub fn source_kind(&self) -> &str {
        &self.source_kind
    }

    /// Validate this record belongs to the expected toolchain id.
    ///
    /// # Errors
    ///
    /// Returns an error when the record id differs from `id`.
    pub fn validate_for_toolchain(&self, id: &ToolchainId) -> Result<()> {
        if &self.toolchain_id != id {
            bail!("install record toolchain id does not match {}", id.as_str());
        }
        Ok(())
    }

    fn validate_archive_schema_version(&self) -> Result<()> {
        match self.archive_schema_version {
            Some(TOOLCHAIN_ARCHIVE_SCHEMA_VERSION) => Ok(()),
            Some(version) => bail!(
                "unsupported archive schema version {version}; expected {TOOLCHAIN_ARCHIVE_SCHEMA_VERSION}"
            ),
            None => bail!(
                "archive schema version is required; expected {TOOLCHAIN_ARCHIVE_SCHEMA_VERSION}"
            ),
        }
    }
}

/// Publish result for a staged toolchain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublishOutcome {
    /// A new immutable toolchain directory was published.
    Published,
    /// A toolchain with identical manifest metadata was already installed.
    AlreadyInstalled,
}

/// Collision status for a proposed toolchain install.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionStatus {
    /// No installed toolchain directory currently exists for the id.
    Absent,
    /// The installed metadata matches the proposed metadata.
    Identical,
    /// The id already exists with different metadata.
    Conflict,
}

/// Temporary staging directory for atomic toolchain publication.
pub struct ToolchainStage {
    paths: AshgrovePaths,
    id: ToolchainId,
    dir: tempfile::TempDir,
}

impl ToolchainStage {
    /// Create a staging directory next to the final toolchain directory.
    ///
    /// # Errors
    ///
    /// Returns an error when the staging parent or temporary directory cannot be created.
    pub fn create(paths: &AshgrovePaths, id: ToolchainId) -> Result<Self> {
        let staging_root = paths.toolchains_dir().join(".staging");
        fs::create_dir_all(&staging_root).context("create toolchain staging root")?;
        let dir = tempfile::tempdir_in(staging_root).context("create toolchain staging dir")?;
        Ok(Self {
            paths: paths.clone(),
            id,
            dir,
        })
    }

    /// Borrow the staging path.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Copy a prepared toolchain payload into staging.
    ///
    /// # Errors
    ///
    /// Returns an error when copying fails.
    pub fn copy_toolchain_payload(&self, source: &Path) -> Result<()> {
        copy_dir(source, self.path())
    }

    /// Atomically publish the staged toolchain.
    ///
    /// # Errors
    ///
    /// Returns an error when shape validation, collision checks, or publish fails.
    pub fn publish(self) -> Result<PublishOutcome> {
        verify_toolchain_shape(self.path(), &self.id)?;
        match classify_toolchain_collision(&self.paths, &self.id, self.path())? {
            CollisionStatus::Absent => {}
            CollisionStatus::Identical => return Ok(PublishOutcome::AlreadyInstalled),
            CollisionStatus::Conflict => {
                bail!(
                    "metadata collision for installed toolchain '{}'",
                    self.id.as_str()
                );
            }
        }
        fs::create_dir_all(self.paths.toolchains_dir()).context("create toolchains dir")?;
        fs::rename(self.path(), self.paths.toolchain_dir(&self.id))
            .context("publish staged toolchain")?;
        Ok(PublishOutcome::Published)
    }
}

/// Classify whether a proposed toolchain collides with an installed id.
///
/// # Errors
///
/// Returns an error when metadata cannot be read.
pub fn classify_toolchain_collision(
    paths: &AshgrovePaths,
    id: &ToolchainId,
    proposed_root: &Path,
) -> Result<CollisionStatus> {
    let installed = paths.toolchain_dir(id);
    if !installed.exists() {
        return Ok(CollisionStatus::Absent);
    }
    if installed_metadata_matches(&installed, proposed_root)? {
        Ok(CollisionStatus::Identical)
    } else {
        Ok(CollisionStatus::Conflict)
    }
}

fn installed_metadata_matches(installed: &Path, proposed_root: &Path) -> Result<bool> {
    let installed_manifest =
        fs::read_to_string(installed.join("manifest.toml")).context("read installed manifest")?;
    let proposed_manifest = fs::read_to_string(proposed_root.join("manifest.toml"))
        .context("read proposed manifest")?;
    if normalize_ws(&installed_manifest) != normalize_ws(&proposed_manifest) {
        return Ok(false);
    }

    let installed_record = fs::read_to_string(installed.join("install-record.toml"))
        .context("read installed install record")?;
    let proposed_record = fs::read_to_string(proposed_root.join("install-record.toml"))
        .context("read proposed install record")?;
    Ok(normalize_install_record(&installed_record)? == normalize_install_record(&proposed_record)?)
}

fn normalize_install_record(text: &str) -> Result<toml::Value> {
    let mut value = toml::from_str::<toml::Value>(text).context("parse install record")?;
    if let Some(table) = value.as_table_mut() {
        table.remove("installed_at");
        table.remove("tarball_path");
    }
    Ok(value)
}

/// Stage the stdlib package manifest into `lib/ash/std/ash.toml`.
///
/// # Errors
///
/// Returns an error when the stdlib root is missing or the manifest cannot be written.
pub fn stage_stdlib_metadata(toolchain_root: &Path, metadata: &StdlibMetadata) -> Result<()> {
    let metadata_path = Path::new(&metadata.path);
    if metadata_path.is_absolute()
        || metadata_path
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::Prefix(_)))
    {
        bail!(
            "stdlib metadata path must stay inside the toolchain root: {}",
            metadata.path
        );
    }
    let std_root = toolchain_root.join(&metadata.path);
    if !std_root.join("src").is_dir() {
        bail!("stdlib source root is missing at {}", std_root.display());
    }
    fs::create_dir_all(&std_root).context("create stdlib root")?;
    fs::write(
        std_root.join("ash.toml"),
        format!(
            "[package]\nname = \"std\"\nversion = \"{}\"\n",
            metadata.version()
        ),
    )
    .context("write stdlib package metadata")
}

fn stage_source_stdlib_metadata(
    source: &Path,
    toolchain_root: &Path,
    metadata: &StdlibMetadata,
) -> Result<()> {
    let metadata_path = validate_relative_toolchain_path(&metadata.path, "stdlib metadata path")?;
    let std_root = toolchain_root.join(metadata_path);
    if !std_root.join("src").is_dir() {
        bail!("stdlib source root is missing at {}", std_root.display());
    }
    let source_manifest = source.join("std/Cargo.toml");
    let manifest_text = fs::read_to_string(&source_manifest)
        .with_context(|| format!("read stdlib metadata {}", source_manifest.display()))?;
    let manifest: toml::Value =
        toml::from_str(&manifest_text).context("parse source stdlib metadata")?;
    let package = manifest
        .get("package")
        .and_then(toml::Value::as_table)
        .with_context(|| {
            format!(
                "source stdlib metadata missing [package] at {}",
                source_manifest.display()
            )
        })?;
    if package
        .get("name")
        .and_then(toml::Value::as_str)
        .is_none_or(|name| name.trim().is_empty())
    {
        bail!(
            "source stdlib metadata missing [package].name at {}",
            source_manifest.display()
        );
    }
    if !package_version_is_present(package.get("version")) {
        bail!(
            "source stdlib metadata missing [package].version at {}",
            source_manifest.display()
        );
    }
    fs::write(std_root.join("ash.toml"), manifest_text).context("write stdlib package metadata")
}

fn package_version_is_present(value: Option<&toml::Value>) -> bool {
    match value {
        Some(toml::Value::String(version)) => !version.trim().is_empty(),
        Some(toml::Value::Table(table)) => table
            .get("workspace")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false),
        _ => false,
    }
}

#[derive(Debug, Parser)]
#[command(name = "ashgrove")]
#[command(about = "Ash user-local toolchain and deployment manager")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Install a toolchain from source or tarball.
    Install(InstallArgs),
    /// Install a new toolchain and optionally switch the default.
    Update(UpdateArgs),
    /// Set the user default toolchain.
    Default { toolchain_id: String },
    /// List installed toolchains.
    List,
    /// Print the selected toolchain.
    Current {
        /// Project root used for project toolchain selection.
        #[arg(long)]
        project: Option<PathBuf>,
    },
    /// Remove an installed toolchain.
    Remove {
        toolchain_id: String,
        #[arg(long)]
        force: bool,
    },
    /// Plan or perform conservative cleanup.
    Cleanup(CleanupArgs),
    /// Fetch git dependencies recorded in ash.toml.
    Fetch(ProjectArgs),
    /// Resolve or check ash.lock.
    Lock(LockArgs),
    /// Materialize locked dependencies for offline deployment.
    Vendor(VendorArgs),
    /// Internal stable launcher dispatch entry point.
    #[command(name = "__launcher-dispatch", hide = true)]
    LauncherDispatch(LauncherDispatchArgs),
}

#[derive(Debug, Args)]
struct InstallArgs {
    /// Rejected until a release index policy exists.
    bare_version: Option<String>,
    #[arg(long = "from", value_enum)]
    source: Option<InstallSource>,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    url: Option<String>,
    #[arg(long)]
    version: Option<String>,
    #[arg(long)]
    rev: Option<String>,
    #[arg(long)]
    digest: Option<String>,
    #[arg(long)]
    allow_dirty_source: bool,
    #[arg(long)]
    allow_unidentified_source: bool,
    #[arg(long)]
    switch: bool,
}

#[derive(Debug, Args)]
struct UpdateArgs {
    /// Rejected until a release index policy exists.
    bare_version: Option<String>,
    #[arg(long)]
    to: Option<String>,
    #[arg(long = "from", value_enum)]
    source: Option<InstallSource>,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    url: Option<String>,
    #[arg(long)]
    digest: Option<String>,
    #[arg(long)]
    switch: bool,
    #[arg(long)]
    allow_dirty_source: bool,
    #[arg(long)]
    allow_unidentified_source: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum InstallSource {
    Source,
    Tarball,
}

#[derive(Debug, Args)]
struct CleanupArgs {
    #[arg(long)]
    project: Option<PathBuf>,
    #[arg(long)]
    dry_run: bool,
    #[arg(long)]
    cache: bool,
    #[arg(long)]
    orphans: bool,
    #[arg(long)]
    old_toolchains: bool,
}

#[derive(Debug, Args)]
struct ProjectArgs {
    #[arg(long)]
    project: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct LockArgs {
    #[arg(long)]
    project: Option<PathBuf>,
    #[arg(long)]
    check: bool,
}

#[derive(Debug, Args)]
struct VendorArgs {
    #[arg(long)]
    project: Option<PathBuf>,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long)]
    check: bool,
}

#[derive(Debug, Args)]
struct LauncherDispatchArgs {
    #[arg(long)]
    tool: String,
    #[arg(long)]
    toolchain: Option<String>,
    #[arg(long)]
    project: Option<PathBuf>,
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<OsString>,
}

/// CLI entry point.
#[must_use]
pub fn main() -> ExitCode {
    match run_cli() {
        Ok(exit_code) => exit_code,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run_cli() -> Result<ExitCode> {
    let cli = Cli::parse();
    let paths = AshgrovePaths::from_env()?;
    match cli.command {
        Commands::Install(args) => install(&paths, args),
        Commands::Update(args) => update(&paths, args),
        Commands::Default { toolchain_id } => {
            set_default(&paths, &ToolchainId::parse(&toolchain_id)?)
        }
        Commands::List => list_toolchains(&paths),
        Commands::Current { project } => current(&paths, project.as_deref()),
        Commands::Remove {
            toolchain_id,
            force,
        } => remove_toolchain(&paths, &ToolchainId::parse(&toolchain_id)?, force),
        Commands::Cleanup(args) => cleanup(&paths, args),
        Commands::Fetch(args) => fetch(
            &paths,
            args.project.as_deref().unwrap_or_else(|| Path::new(".")),
        ),
        Commands::Lock(args) => lock(
            args.project.as_deref().unwrap_or_else(|| Path::new(".")),
            args.check,
        ),
        Commands::Vendor(args) => vendor(
            &paths,
            args.project.as_deref().unwrap_or_else(|| Path::new(".")),
            args.output.as_deref(),
            args.check,
        ),
        Commands::LauncherDispatch(args) => return run_launcher_dispatch(&paths, args),
    }?;
    Ok(ExitCode::SUCCESS)
}

fn run_launcher_dispatch(paths: &AshgrovePaths, args: LauncherDispatchArgs) -> Result<ExitCode> {
    let mut request = LauncherDispatchRequest::new(args.tool);
    let explicit = args
        .toolchain
        .or_else(|| std::env::var("ASH_TOOLCHAIN").ok())
        .map(|id| ToolchainId::parse(&id))
        .transpose()?;
    if let Some(id) = explicit {
        request = request.with_explicit_toolchain(id);
    }
    let project = match args.project {
        Some(project) => project,
        None => std::env::current_dir().context("read current directory for launcher dispatch")?,
    };
    request = request.with_project(project);
    let dispatch = resolve_launcher_dispatch(paths, request)?;
    let stdlib_root = selected_stdlib_source_root(paths, dispatch.toolchain_id())?;
    let runtime_support_identity =
        selected_runtime_support_identity(paths, dispatch.toolchain_id())?;
    let mut command = Command::new(dispatch.tool_path());
    command
        .args(args.args)
        .env(
            "ASHGROVE_RUNNING_TOOLCHAIN",
            dispatch.toolchain_id().as_str(),
        )
        .env("ASH_STDLIB_ROOT", stdlib_root)
        .env("ASH_RUNTIME_SUPPORT_IDENTITY", runtime_support_identity);
    exec_or_status(command, dispatch.tool_path())
}

fn selected_runtime_support_identity(paths: &AshgrovePaths, id: &ToolchainId) -> Result<String> {
    let manifest = installed_toolchain_manifest(paths, id)?;
    Ok(manifest.runtime_support().identity().to_string())
}

fn selected_stdlib_source_root(paths: &AshgrovePaths, id: &ToolchainId) -> Result<PathBuf> {
    let manifest = installed_toolchain_manifest(paths, id)?;
    let stdlib_path =
        validate_relative_toolchain_path(manifest.stdlib().path(), "stdlib metadata path")?;
    let root = paths.toolchain_dir(id).join(stdlib_path).join("src");
    if !root.is_dir() {
        bail!(
            "selected toolchain '{}' stdlib source root is missing at {}",
            id.as_str(),
            root.display()
        );
    }
    Ok(root)
}

#[cfg(unix)]
fn exec_or_status(mut command: Command, tool_path: &Path) -> Result<ExitCode> {
    use std::os::unix::process::CommandExt;

    Err(command.exec())
        .with_context(|| format!("execute selected launcher tool {}", tool_path.display()))
}

#[cfg(not(unix))]
fn exec_or_status(mut command: Command, tool_path: &Path) -> Result<ExitCode> {
    let status = command
        .status()
        .with_context(|| format!("execute selected launcher tool {}", tool_path.display()))?;
    if let Some(code) = status.code() {
        return Ok(ExitCode::from(code as u8));
    }
    Ok(ExitCode::FAILURE)
}

fn install(paths: &AshgrovePaths, args: InstallArgs) -> Result<()> {
    if args.bare_version.is_some() {
        bail!("bare version install requires a release index policy, which is not available yet");
    }
    match args.source.context("--from is required")? {
        InstallSource::Source => {
            let source = args.path.context("--path is required for source install")?;
            install_from_source(
                paths,
                &source,
                args.allow_dirty_source,
                args.allow_unidentified_source,
                args.switch,
                None,
            )
            .map(|_| ())
        }
        InstallSource::Tarball => {
            if let Some(url) = args.url {
                return install_from_tarball_url(
                    paths,
                    &url,
                    args.switch,
                    None,
                    args.digest.as_deref(),
                )
                .map(|_| ());
            }
            let path = args
                .path
                .context("--path is required for tarball install")?;
            install_from_tarball(
                paths,
                &path,
                args.switch,
                None,
                args.digest.as_deref(),
                None,
            )
            .map(|_| ())
        }
    }
}

fn update(paths: &AshgrovePaths, args: UpdateArgs) -> Result<()> {
    if args.bare_version.is_some() {
        bail!("bare version update requires a release index policy, which is not available yet");
    }
    let to = args.to.context("--to is required for update")?;
    let source = args.source.context("--from is required for update")?;
    let id = ToolchainId::parse(&to)?;
    match source {
        InstallSource::Source => {
            let source = args.path.context("--path is required for source update")?;
            install_from_source(
                paths,
                &source,
                args.allow_dirty_source,
                args.allow_unidentified_source,
                args.switch,
                Some(&id),
            )
            .map(|_| ())
        }
        InstallSource::Tarball => {
            if let Some(url) = args.url {
                return install_from_tarball_url(
                    paths,
                    &url,
                    args.switch,
                    Some(&id),
                    args.digest.as_deref(),
                )
                .map(|_| ());
            }
            let path = args.path.context("--path is required for tarball update")?;
            install_from_tarball(
                paths,
                &path,
                args.switch,
                Some(&id),
                args.digest.as_deref(),
                None,
            )
            .map(|_| ())
        }
    }
}

fn install_from_source(
    paths: &AshgrovePaths,
    source: &Path,
    allow_dirty: bool,
    allow_unidentified: bool,
    switch: bool,
    expected_id: Option<&ToolchainId>,
) -> Result<ToolchainId> {
    if source.join(".dirty").exists() && !allow_dirty {
        bail!(
            "dirty source rejected; pass --allow-dirty-source to record a non-reproducible install"
        );
    }
    if is_source_root(source) {
        return install_from_source_root(
            paths,
            source,
            allow_dirty,
            allow_unidentified,
            switch,
            expected_id,
        );
    }
    let release_source =
        SourceArchiveReleaseMetadata::read_from_source(source, allow_unidentified)?;
    let source_archive_digest = source_tree_digest(source)?;
    let source_rev = release_source
        .as_ref()
        .map(|metadata| metadata.origin_commit.as_str());
    let id = read_toolchain_id(source)?;
    verify_expected_source_id(expected_id, &id)?;
    let source_url = read_optional_trimmed(source.join(".source-url"))?;
    let dirty_source_digest = if allow_dirty {
        Some(source_archive_digest.as_str())
    } else {
        None
    };
    let stage = ToolchainStage::create(paths, id.clone())?;
    stage.copy_toolchain_payload(source)?;
    write_source_install_record(
        &stage.path().join("install-record.toml"),
        SourceInstallRecordInput {
            id: &id,
            source_path: source,
            source_rev,
            source_url: source_url.as_deref(),
            source_origin_commit: source_rev,
            source_archive_digest: Some(source_archive_digest.as_str()),
            dirty_source_digest,
            allow_dirty,
            allow_unidentified,
        },
    )?;
    stage.publish()?;
    install_launcher_shims_from_current_exe(paths)?;
    if switch || read_default(paths)?.is_none() {
        set_default(paths, &id)?;
    }
    Ok(id)
}

#[derive(Debug, Deserialize)]
struct SourceArchiveReleaseMetadata {
    schema_version: u32,
    origin_commit: String,
}

impl SourceArchiveReleaseMetadata {
    fn read_optional_from_source(source: &Path) -> Result<Option<Self>> {
        let path = source.join("release-source.toml");
        if !path.is_file() {
            return Ok(None);
        }
        Self::read_from_source(source, false)
    }

    fn read_from_source(source: &Path, allow_unidentified: bool) -> Result<Option<Self>> {
        let path = source.join("release-source.toml");
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if allow_unidentified {
                    return Ok(None);
                }
                bail!(
                    "release-source metadata is required for source archives; pass --allow-unidentified-source to record a non-reproducible install"
                );
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("read release-source metadata {}", path.display()));
            }
        };
        let metadata: Self = toml::from_str(&text).context("parse release-source metadata")?;
        metadata.validate()?;
        if let Some(legacy_rev) = read_optional_trimmed(source.join(".source-rev"))?
            && legacy_rev != metadata.origin_commit
        {
            bail!("release-source origin_commit does not match legacy source revision");
        }
        Ok(Some(metadata))
    }

    fn validate(&self) -> Result<()> {
        if self.schema_version != TOOLCHAIN_ARCHIVE_SCHEMA_VERSION {
            bail!(
                "unsupported release-source schema version {}; expected {TOOLCHAIN_ARCHIVE_SCHEMA_VERSION}",
                self.schema_version
            );
        }
        if !(7..=64).contains(&self.origin_commit.len())
            || !self.origin_commit.chars().all(|ch| ch.is_ascii_hexdigit())
        {
            bail!("release-source origin_commit must be a git commit hash");
        }
        Ok(())
    }
}

fn is_source_root(source: &Path) -> bool {
    source.join("Cargo.toml").is_file() && source.join("std/src").is_dir()
}

fn install_from_source_root(
    paths: &AshgrovePaths,
    source: &Path,
    allow_dirty: bool,
    allow_unidentified: bool,
    switch: bool,
    expected_id: Option<&ToolchainId>,
) -> Result<ToolchainId> {
    let metadata = SourceRootMetadata::inspect(source)?;
    let release_source = SourceArchiveReleaseMetadata::read_optional_from_source(source)?;
    let source_rev = release_source
        .as_ref()
        .map(|metadata| metadata.origin_commit.as_str())
        .or(metadata.rev.as_deref());
    if let (Some(release_source), Some(git_rev)) = (&release_source, metadata.rev.as_deref())
        && release_source.origin_commit != git_rev
    {
        bail!("release-source origin_commit does not match source root git revision");
    }
    if metadata.dirty && !allow_dirty {
        bail!(
            "dirty source rejected; pass --allow-dirty-source to record a non-reproducible install"
        );
    }
    if source_rev.is_none() && !allow_unidentified {
        bail!("unidentified source rejected; pass --allow-unidentified-source to record it");
    }
    let version = source_package_version(source)?;
    let source_digest = source_tree_digest(source)?;
    let source_archive_digest = release_source.as_ref().map(|_| source_digest.as_str());
    let dirty_digest = if metadata.dirty {
        Some(source_digest.as_str())
    } else {
        None
    };
    let id = source_toolchain_id(&version, source, source_rev, dirty_digest)?;
    verify_expected_source_id(expected_id, &id)?;
    let stage = ToolchainStage::create(paths, id.clone())?;
    stage_source_root_toolchain(paths, source, &stage, &id, &version, &source_digest)?;
    write_source_install_record(
        &stage.path().join("install-record.toml"),
        SourceInstallRecordInput {
            id: &id,
            source_path: source,
            source_rev,
            source_url: metadata.url.as_deref(),
            source_origin_commit: release_source
                .as_ref()
                .map(|metadata| metadata.origin_commit.as_str()),
            source_archive_digest,
            dirty_source_digest: dirty_digest,
            allow_dirty,
            allow_unidentified,
        },
    )?;
    stage.publish()?;
    install_launcher_shims_from_current_exe(paths)?;
    if switch || read_default(paths)?.is_none() {
        set_default(paths, &id)?;
    }
    Ok(id)
}

fn verify_expected_source_id(
    expected_id: Option<&ToolchainId>,
    actual_id: &ToolchainId,
) -> Result<()> {
    if let Some(expected_id) = expected_id
        && expected_id != actual_id
    {
        bail!(
            "update --to {} does not match source toolchain {}",
            expected_id.as_str(),
            actual_id.as_str()
        );
    }
    Ok(())
}

#[derive(Debug)]
struct SourceRootMetadata {
    rev: Option<String>,
    url: Option<String>,
    dirty: bool,
}

impl SourceRootMetadata {
    fn inspect(source: &Path) -> Result<Self> {
        let git_like = source.join(".git").exists() || git_is_inside_work_tree(source)?;
        let rev = git_output_optional(source, &["rev-parse", "HEAD"])?;
        if rev.is_none() && git_like {
            bail!(
                "git revision failed for git source root {}; cannot determine source identity",
                source.display()
            );
        }
        let dirty = match git_status_porcelain(source, git_like || rev.is_some())? {
            Some(status) => !status.trim().is_empty(),
            None => source.join(".dirty").exists(),
        };
        let url = git_output_optional(source, &["config", "--get", "remote.origin.url"])?
            .or(read_optional_trimmed(source.join(".source-url"))?);
        Ok(Self {
            rev: rev.map(|value| value.trim().to_string()),
            url: url.map(|value| value.trim().to_string()),
            dirty,
        })
    }
}

fn git_is_inside_work_tree(source: &Path) -> Result<bool> {
    Ok(
        git_output_optional(source, &["rev-parse", "--is-inside-work-tree"])?.as_deref()
            == Some("true"),
    )
}

fn source_package_version(source: &Path) -> Result<String> {
    let text = fs::read_to_string(source.join("Cargo.toml")).context("read source Cargo.toml")?;
    let value: toml::Value = toml::from_str(&text).context("parse source Cargo.toml")?;
    let version = value
        .get("workspace")
        .and_then(|workspace| workspace.get("package"))
        .and_then(|package| package.get("version"))
        .or_else(|| {
            value
                .get("package")
                .and_then(|package| package.get("version"))
        })
        .and_then(toml::Value::as_str)
        .context("source Cargo.toml missing package version")?;
    Ok(version.to_string())
}

fn source_toolchain_id(
    version: &str,
    source: &Path,
    rev: Option<&str>,
    dirty_digest: Option<&str>,
) -> Result<ToolchainId> {
    let suffix = match rev {
        Some(rev) => {
            let mut suffix = rev.chars().take(12).collect::<String>();
            if let Some(digest) = dirty_digest {
                suffix.push_str(".dirty");
                suffix.extend(digest.chars().take(12));
            }
            suffix
        }
        None => {
            let digest = Sha256::digest(source.display().to_string().as_bytes());
            let mut value = String::from("unidentified");
            for byte in &digest[..6] {
                value.push_str(&format!("{byte:02x}"));
            }
            value
        }
    };
    ToolchainId::parse(&format!("ash-{version}+source.{suffix}"))
}

fn stage_source_root_toolchain(
    paths: &AshgrovePaths,
    source: &Path,
    stage: &ToolchainStage,
    id: &ToolchainId,
    version: &str,
    expected_source_digest: &str,
) -> Result<()> {
    build_source_binaries(paths, source, id)?;
    let post_build_digest = source_tree_digest(source)?;
    if post_build_digest != expected_source_digest {
        bail!(
            "source cargo build dirtied source root {}; aborting before publish",
            source.display()
        );
    }
    fs::create_dir_all(stage.path().join("bin")).context("create source stage bin")?;
    let build_dir = source_build_dir(paths, id).join(build_profile());
    install_executable_copy(
        &build_dir.join(executable_name("ash")),
        &stage.path().join("bin").join(executable_name("ash")),
    )
    .context("stage source-built ash binary")?;
    install_executable_copy(
        &build_dir.join(executable_name("ashgrove")),
        &stage.path().join("bin").join(executable_name("ashgrove")),
    )
    .context("stage source-built ashgrove binary")?;

    copy_dir(
        &source.join("std/src"),
        &stage.path().join("lib/ash/std/src"),
    )?;
    let stdlib = StdlibMetadata::new(version, "lib/ash/std");
    stage_source_stdlib_metadata(source, stage.path(), &stdlib)?;
    ToolchainManifest::minimal(id.clone(), version, target_triple(), "source")
        .write_to(&stage.path().join("manifest.toml"))?;
    Ok(())
}

fn build_source_binaries(paths: &AshgrovePaths, source: &Path, id: &ToolchainId) -> Result<()> {
    let target_dir = source_build_dir(paths, id);
    fs::create_dir_all(&target_dir).context("create source build target dir")?;
    let build_source_root = paths.cache_dir().join("source-build-roots");
    fs::create_dir_all(&build_source_root).context("create source build root cache")?;
    let build_source =
        tempfile::tempdir_in(&build_source_root).context("create isolated source build dir")?;
    copy_source_tree_for_build(source, build_source.path())?;

    let mut command = Command::new("cargo");
    command.args([
        "build",
        "--package",
        "ash-cli",
        "--bin",
        "ash",
        "--package",
        "ashgrove",
        "--bin",
        "ashgrove",
    ]);
    if source.join("Cargo.lock").is_file() {
        command.arg("--locked");
    }
    let status = command
        .current_dir(build_source.path())
        .env("CARGO_TARGET_DIR", &target_dir)
        .status()
        .with_context(|| format!("run cargo build in source root {}", source.display()))?;
    if !status.success() {
        bail!("source cargo build failed for {}", source.display());
    }
    Ok(())
}

fn copy_source_tree_for_build(source: &Path, dest: &Path) -> Result<()> {
    for entry in walkdir::WalkDir::new(source) {
        let entry = entry.context("walk source build input")?;
        let rel = entry.path().strip_prefix(source).context("strip prefix")?;
        if rel.as_os_str().is_empty() || source_digest_skip_path(rel) {
            continue;
        }
        let target = dest.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).with_context(|| format!("create {}", target.display()))?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).context("create parent")?;
            }
            fs::copy(entry.path(), &target)
                .with_context(|| format!("copy {}", entry.path().display()))?;
        }
    }
    Ok(())
}

fn source_tree_digest(source: &Path) -> Result<String> {
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(source) {
        let entry = entry.context("walk source digest input")?;
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry.path().strip_prefix(source).context("strip prefix")?;
        if source_digest_skip_path(rel) {
            continue;
        }
        files.push(rel.to_path_buf());
    }
    files.sort();

    let mut hasher = Sha256::new();
    for rel in files {
        hasher.update(rel.to_string_lossy().as_bytes());
        hasher.update([0]);
        let mut file = fs::File::open(source.join(&rel))
            .with_context(|| format!("open source digest input {}", rel.display()))?;
        let mut buf = [0u8; 8192];
        loop {
            let n = file
                .read(&mut buf)
                .with_context(|| format!("read source digest input {}", rel.display()))?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        hasher.update([0]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn source_digest_skip_path(rel: &Path) -> bool {
    matches!(
        rel.components().next(),
        Some(Component::Normal(name)) if name == OsStr::new(".git") || name == OsStr::new("target")
    )
}

fn source_build_dir(paths: &AshgrovePaths, id: &ToolchainId) -> PathBuf {
    paths.cache_dir().join("builds").join(id.as_str())
}

fn executable_name(name: &str) -> String {
    format!("{name}{}", std::env::consts::EXE_SUFFIX)
}

fn git_output_optional(source: &Path, args: &[&str]) -> Result<Option<String>> {
    let output = Command::new("git")
        .args(args)
        .current_dir(source)
        .output()
        .with_context(|| format!("run git {} in {}", args.join(" "), source.display()))?;
    if !output.status.success() {
        return Ok(None);
    }
    Ok(Some(
        String::from_utf8_lossy(&output.stdout).trim().to_string(),
    ))
}

fn git_status_porcelain(source: &Path, fail_closed: bool) -> Result<Option<String>> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(source)
        .output()
        .with_context(|| format!("run git status --porcelain in {}", source.display()))?;
    if output.status.success() {
        return Ok(Some(
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
        ));
    }
    if fail_closed {
        bail!(
            "git status failed for identified source root {}: {}",
            source.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(None)
}

struct SourceInstallRecordInput<'a> {
    id: &'a ToolchainId,
    source_path: &'a Path,
    source_rev: Option<&'a str>,
    source_url: Option<&'a str>,
    source_origin_commit: Option<&'a str>,
    source_archive_digest: Option<&'a str>,
    dirty_source_digest: Option<&'a str>,
    allow_dirty: bool,
    allow_unidentified: bool,
}

fn write_source_install_record(path: &Path, input: SourceInstallRecordInput<'_>) -> Result<()> {
    let mut table = toml::map::Map::new();
    table.insert(
        "toolchain_id".to_string(),
        toml::Value::String(input.id.as_str().to_string()),
    );
    table.insert(
        "source_kind".to_string(),
        toml::Value::String("source".to_string()),
    );
    table.insert(
        "source_path".to_string(),
        toml::Value::String(input.source_path.display().to_string()),
    );
    if let Some(source_url) = input.source_url {
        table.insert(
            "source_url".to_string(),
            toml::Value::String(source_url.to_string()),
        );
    }
    if let Some(source_rev) = input.source_rev {
        table.insert(
            "source_rev".to_string(),
            toml::Value::String(source_rev.to_string()),
        );
    }
    if let Some(source_origin_commit) = input.source_origin_commit {
        table.insert(
            "source_origin_commit".to_string(),
            toml::Value::String(source_origin_commit.to_string()),
        );
    }
    if let Some(source_archive_digest) = input.source_archive_digest {
        table.insert(
            "source_archive_digest".to_string(),
            toml::Value::String(format!("sha256:{source_archive_digest}")),
        );
    }
    if let Some(dirty_source_digest) = input.dirty_source_digest {
        table.insert(
            "dirty_source_digest".to_string(),
            toml::Value::String(format!("sha256:{dirty_source_digest}")),
        );
    }
    table.insert(
        "build_profile".to_string(),
        toml::Value::String(build_profile().to_string()),
    );
    table.insert(
        "target_triple".to_string(),
        toml::Value::String(target_triple()),
    );
    table.insert(
        "allow_dirty_source".to_string(),
        toml::Value::Boolean(input.allow_dirty),
    );
    table.insert(
        "allow_unidentified_source".to_string(),
        toml::Value::Boolean(input.allow_unidentified),
    );
    table.insert(
        "reproducible".to_string(),
        toml::Value::Boolean(
            (input.source_rev.is_some() || input.source_origin_commit.is_some())
                && !input.allow_dirty
                && !input.allow_unidentified,
        ),
    );
    table.insert(
        "installed_at".to_string(),
        toml::Value::String(Utc::now().to_rfc3339()),
    );
    fs::write(path, toml::to_string(&toml::Value::Table(table))?)
        .context("write source install record")
}

fn read_optional_trimmed(path: PathBuf) -> Result<Option<String>> {
    match fs::read_to_string(&path) {
        Ok(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(trimmed.to_string()))
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("read {}", path.display())),
    }
}

fn build_profile() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    }
}

fn target_triple() -> String {
    let arch = std::env::consts::ARCH;
    match std::env::consts::OS {
        "linux" => format!("{arch}-unknown-linux-{}", target_env_suffix()),
        "macos" => format!("{arch}-apple-darwin"),
        "windows" => format!("{arch}-pc-windows-{}", target_env_suffix()),
        os => format!("{arch}-unknown-{os}"),
    }
}

fn target_env_suffix() -> &'static str {
    if cfg!(target_env = "musl") {
        "musl"
    } else if cfg!(target_env = "msvc") {
        "msvc"
    } else if cfg!(target_env = "gnu") {
        "gnu"
    } else {
        "unknown"
    }
}

fn install_from_tarball(
    paths: &AshgrovePaths,
    archive: &Path,
    switch: bool,
    expected_id: Option<&ToolchainId>,
    expected_digest: Option<&str>,
    tarball_url: Option<&str>,
) -> Result<ToolchainId> {
    let digest = file_digest(archive)?;
    verify_expected_tarball_digest(expected_digest, &digest)?;
    let temp = tempfile::tempdir().context("create extraction staging dir")?;
    let file = fs::File::open(archive)
        .with_context(|| format!("failed to open tarball {}", archive.display()))?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive_reader = tar::Archive::new(decoder);
    archive_reader
        .entries()
        .context("read archive entries")?
        .try_for_each(|entry| -> Result<()> {
            let mut entry = entry.context("read archive entry")?;
            validate_archive_entry(&entry)?;
            entry
                .unpack_in(temp.path())
                .context("unpack safe archive entry")?;
            Ok(())
        })?;

    let mut roots = fs::read_dir(temp.path())
        .context("read extraction root")?
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        bail!("tarball must contain exactly one toolchain root directory");
    }
    let root = roots.remove(0).path();
    let id = root
        .file_name()
        .and_then(OsStr::to_str)
        .context("toolchain root must be utf8")?;
    let id = ToolchainId::parse(id)?;
    if let Some(expected_id) = expected_id
        && &id != expected_id
    {
        bail!(
            "update --to {} does not match tarball toolchain {}",
            expected_id.as_str(),
            id.as_str()
        );
    }
    let manifest = verify_toolchain_shape(&root, &id)?;
    manifest.validate_archive_schema_version()?;
    verify_install_record_shape(&root, &id)?;
    verify_required_binaries_executable(&root, &id)?;
    verify_required_manifest_tools(&root, &manifest)?;
    write_tarball_install_record(
        &root.join("install-record.toml"),
        &id,
        archive,
        &digest,
        tarball_url,
    )?;
    let stage = ToolchainStage::create(paths, id.clone())?;
    stage.copy_toolchain_payload(&root)?;
    stage.publish()?;
    install_launcher_shims_from_current_exe(paths)?;
    if switch || read_default(paths)?.is_none() {
        set_default(paths, &id)?;
    }
    Ok(id)
}

fn install_from_tarball_url(
    paths: &AshgrovePaths,
    url: &str,
    switch: bool,
    expected_id: Option<&ToolchainId>,
    expected_digest: Option<&str>,
) -> Result<ToolchainId> {
    if expected_digest.is_none() {
        bail!(
            "tarball URL install requires authenticated download policy evidence: explicit sha256 digest or signed release-index evidence"
        );
    }
    let archive = authenticated_tarball_url_path(url)?;
    install_from_tarball(
        paths,
        &archive,
        switch,
        expected_id,
        expected_digest,
        Some(url),
    )
}

fn authenticated_tarball_url_path(url: &str) -> Result<PathBuf> {
    let Some(path) = url.strip_prefix("file://") else {
        bail!(
            "authenticated download policy supports file:// tarball URLs with explicit sha256 digest; no best-effort network lookup for {url}"
        );
    };
    if path.trim().is_empty() {
        bail!("tarball URL must include a file path");
    }
    Ok(PathBuf::from(path))
}

fn verify_expected_tarball_digest(
    expected_digest: Option<&str>,
    actual_digest: &str,
) -> Result<()> {
    if let Some(expected_digest) = expected_digest {
        let Some(expected_hex) = expected_digest.strip_prefix("sha256:") else {
            bail!("tarball digest must use sha256:<hex> format");
        };
        if expected_hex.len() != 64 || !expected_hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
            bail!("tarball digest must use sha256:<64-hex> format");
        }
        let expected = expected_hex.to_ascii_lowercase();
        if expected != actual_digest {
            bail!(
                "tarball digest mismatch: expected sha256:{expected}, got sha256:{actual_digest}"
            );
        }
    }
    Ok(())
}

fn validate_archive_entry(entry: &tar::Entry<'_, impl Read>) -> Result<()> {
    let header = entry.header();
    if header.entry_type().is_symlink()
        || header.entry_type().is_hard_link()
        || header.entry_type().is_block_special()
        || header.entry_type().is_character_special()
        || header.entry_type().is_fifo()
    {
        bail!("unsafe archive entry type");
    }
    let mode = header.mode().unwrap_or(0);
    if mode & 0o6000 != 0 {
        bail!("unsafe archive entry mode");
    }
    let path = entry.path().context("archive path")?;
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)))
    {
        bail!("unsafe archive entry '{}'", path.display());
    }
    Ok(())
}

fn verify_toolchain_shape(root: &Path, id: &ToolchainId) -> Result<ToolchainManifest> {
    for rel in [
        "bin/ash",
        "bin/ashgrove",
        "lib/ash/std/src",
        "manifest.toml",
        "install-record.toml",
    ] {
        let path = root.join(rel);
        if !path.exists() {
            bail!("toolchain '{}' missing required path {rel}", id.as_str());
        }
    }
    if !root.join("lib/ash/std/ash.toml").is_file() {
        bail!("stdlib manifest is missing at lib/ash/std/ash.toml");
    }
    let manifest_text = fs::read_to_string(root.join("manifest.toml")).context("read manifest")?;
    let manifest = ToolchainManifest::from_toml_str(&manifest_text)?;
    manifest.validate_for_toolchain(id)?;
    verify_stdlib_manifest(root, manifest.stdlib())?;
    verify_runtime_support_payload(root, manifest.runtime_support())?;
    Ok(manifest)
}

fn verify_install_record_shape(root: &Path, id: &ToolchainId) -> Result<InstallRecord> {
    let record_text =
        fs::read_to_string(root.join("install-record.toml")).context("read install record")?;
    let record = InstallRecord::from_toml_str(&record_text)?;
    record.validate_for_toolchain(id)?;
    if record.source_kind() != "tarball" {
        bail!(
            "install record source_kind must be tarball for {}",
            id.as_str()
        );
    }
    record.validate_archive_schema_version()?;
    Ok(record)
}

fn verify_stdlib_manifest(root: &Path, metadata: &StdlibMetadata) -> Result<()> {
    let stdlib_path = validate_relative_toolchain_path(metadata.path(), "stdlib metadata path")?;
    let manifest = root.join(stdlib_path).join("ash.toml");
    if !manifest.is_file() {
        bail!("stdlib manifest is missing at {}", manifest.display());
    }
    Ok(())
}

fn verify_runtime_support_payload(root: &Path, metadata: &RuntimeSupportMetadata) -> Result<()> {
    if metadata.identity().trim().is_empty() {
        bail!("runtime support metadata identity is required");
    }
    if !metadata.is_required() {
        bail!("runtime support metadata must mark the payload required");
    }
    let payload_path = validate_relative_toolchain_path(metadata.path(), "runtime support path")?;
    let payload = root.join(payload_path);
    if !payload.is_dir() {
        bail!(
            "runtime support payload is missing at {}",
            payload.display()
        );
    }
    Ok(())
}

fn verify_required_manifest_tools(root: &Path, manifest: &ToolchainManifest) -> Result<()> {
    for name in ["ash", "ashgrove"] {
        if manifest.required_tool(name).is_none() {
            bail!("toolchain manifest missing required tool {name}");
        }
    }
    for tool in manifest.required_tools() {
        let rel = validate_relative_toolchain_path(tool.path(), "standard tool path")?;
        let path = root.join(rel);
        if !path.is_file() {
            bail!(
                "toolchain manifest required tool {} is missing at {}",
                tool.name(),
                path.display()
            );
        }
    }
    Ok(())
}

fn validate_relative_toolchain_path<'a>(path: &'a str, label: &str) -> Result<&'a Path> {
    let path = Path::new(path);
    if path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::Prefix(_)))
    {
        bail!(
            "{label} must stay inside the toolchain root: {}",
            path.display()
        );
    }
    Ok(path)
}

fn write_tarball_install_record(
    path: &Path,
    id: &ToolchainId,
    archive: &Path,
    digest: &str,
    tarball_url: Option<&str>,
) -> Result<()> {
    let mut table = toml::map::Map::new();
    table.insert(
        "toolchain_id".to_string(),
        toml::Value::String(id.as_str().to_string()),
    );
    table.insert(
        "source_kind".to_string(),
        toml::Value::String("tarball".to_string()),
    );
    table.insert(
        "archive_schema_version".to_string(),
        toml::Value::Integer(i64::from(TOOLCHAIN_ARCHIVE_SCHEMA_VERSION)),
    );
    table.insert(
        "tarball_path".to_string(),
        toml::Value::String(archive.display().to_string()),
    );
    table.insert(
        "tarball_digest".to_string(),
        toml::Value::String(format!("sha256:{digest}")),
    );
    if let Some(tarball_url) = tarball_url {
        table.insert(
            "tarball_url".to_string(),
            toml::Value::String(tarball_url.to_string()),
        );
        table.insert(
            "tarball_authentication".to_string(),
            toml::Value::String("explicit-digest".to_string()),
        );
    }
    table.insert(
        "installed_at".to_string(),
        toml::Value::String(Utc::now().to_rfc3339()),
    );
    fs::write(path, toml::to_string(&toml::Value::Table(table))?)
        .context("write tarball install record")
}

#[cfg(unix)]
fn verify_required_binaries_executable(root: &Path, id: &ToolchainId) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    for rel in ["bin/ash", "bin/ashgrove"] {
        let path = root.join(rel);
        let mode = fs::metadata(&path)
            .with_context(|| format!("read required binary metadata {}", path.display()))?
            .permissions()
            .mode();
        if mode & 0o111 == 0 {
            bail!(
                "toolchain '{}' required binary {rel} is not executable",
                id.as_str()
            );
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn verify_required_binaries_executable(_root: &Path, _id: &ToolchainId) -> Result<()> {
    Ok(())
}

fn read_toolchain_id(root: &Path) -> Result<ToolchainId> {
    let manifest = fs::read_to_string(root.join("manifest.toml")).context("read manifest.toml")?;
    let value: toml::Value = toml::from_str(&manifest).context("parse manifest.toml")?;
    let id = value
        .get("toolchain_id")
        .and_then(toml::Value::as_str)
        .context("manifest.toml missing toolchain_id")?;
    ToolchainId::parse(id)
}

fn copy_dir(source: &Path, dest: &Path) -> Result<()> {
    for entry in walkdir::WalkDir::new(source) {
        let entry = entry.context("walk source")?;
        let rel = entry.path().strip_prefix(source).context("strip prefix")?;
        if rel.as_os_str().is_empty() {
            continue;
        }
        let target = dest.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).with_context(|| format!("create {}", target.display()))?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).context("create parent")?;
            }
            fs::copy(entry.path(), &target)
                .with_context(|| format!("copy {}", entry.path().display()))?;
        }
    }
    Ok(())
}

fn set_default(paths: &AshgrovePaths, id: &ToolchainId) -> Result<()> {
    if !paths.toolchain_dir(id).is_dir() {
        let package_version = id
            .as_str()
            .strip_prefix("ash-")
            .and_then(|suffix| suffix.split('+').next())
            .unwrap_or_default();
        if !package_version.is_empty()
            && installed_toolchain_ids(paths)?.iter().any(|installed| {
                installed
                    .as_str()
                    .starts_with(&format!("ash-{package_version}+"))
            })
        {
            bail!(
                "exact toolchain id required when installed toolchains share package version {package_version}"
            );
        }
        bail!("toolchain '{}' is not installed", id.as_str());
    }
    require_installed_toolchain(paths, id)?;
    fs::create_dir_all(paths.config_dir()).context("create config dir")?;
    let selector_path = paths.config_dir().join("toolchains.toml");
    let mut selector = if selector_path.exists() {
        SelectorMetadata::read_from_path(&selector_path)?
    } else {
        SelectorMetadata::empty()
    };
    selector.set_default(id.clone());
    selector.write_to_path(&selector_path)?;
    println!("{}", id.as_str());
    Ok(())
}

fn read_selector(paths: &AshgrovePaths) -> Result<SelectorMetadata> {
    let path = paths.config_dir().join("toolchains.toml");
    if !path.exists() {
        return Ok(SelectorMetadata::empty());
    }
    SelectorMetadata::read_from_path(&path)
}

fn read_default(paths: &AshgrovePaths) -> Result<Option<ToolchainId>> {
    Ok(read_selector(paths)?.default().cloned())
}

fn list_toolchains(paths: &AshgrovePaths) -> Result<()> {
    if !paths.toolchains_dir().exists() {
        return Ok(());
    }
    let default = read_default(paths)?;
    let mut ids = installed_toolchain_ids(paths)?;
    ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    for id in ids {
        if default.as_ref() == Some(&id) {
            println!("{} (default)", id.as_str());
        } else {
            println!("{}", id.as_str());
        }
    }
    Ok(())
}

fn current(paths: &AshgrovePaths, project: Option<&Path>) -> Result<()> {
    if let Some(project) = project
        && let Some(pin) = project_toolchain_pin(project)?
    {
        let id = ToolchainId::parse(&pin)?;
        require_installed_toolchain(paths, &id)?;
        println!("{}", id.as_str());
        return Ok(());
    }
    let Some(default) = read_default(paths)? else {
        bail!("no default Ash toolchain is configured");
    };
    require_installed_toolchain(paths, &default)?;
    println!("{}", default.as_str());
    Ok(())
}

fn installed_toolchain_ids(paths: &AshgrovePaths) -> Result<Vec<ToolchainId>> {
    if !paths.toolchains_dir().exists() {
        return Ok(Vec::new());
    }
    let mut ids = Vec::new();
    for entry in fs::read_dir(paths.toolchains_dir()).context("read toolchains dir")? {
        let entry = entry.context("read toolchain dir entry")?;
        if !entry
            .file_type()
            .with_context(|| format!("read file type {}", entry.path().display()))?
            .is_dir()
        {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(ToOwned::to_owned) else {
            continue;
        };
        let Ok(id) = ToolchainId::parse(&name) else {
            continue;
        };
        if installed_toolchain_manifest(paths, &id).is_ok()
            && verify_install_record_any_source(&entry.path(), &id).is_ok()
        {
            ids.push(id);
        }
    }
    Ok(ids)
}

fn installed_toolchain_manifest(
    paths: &AshgrovePaths,
    id: &ToolchainId,
) -> Result<ToolchainManifest> {
    let root = paths.toolchain_dir(id);
    let manifest_text = fs::read_to_string(root.join("manifest.toml"))
        .with_context(|| format!("read installed manifest for {}", id.as_str()))?;
    let manifest = ToolchainManifest::from_toml_str(&manifest_text)?;
    manifest.validate_for_toolchain(id)?;
    verify_stdlib_manifest(&root, manifest.stdlib())?;
    verify_runtime_support_payload(&root, manifest.runtime_support())?;
    verify_required_manifest_tools(&root, &manifest)?;
    Ok(manifest)
}

fn verify_install_record_any_source(root: &Path, id: &ToolchainId) -> Result<InstallRecord> {
    let record_text =
        fs::read_to_string(root.join("install-record.toml")).context("read install record")?;
    let record = InstallRecord::from_toml_str(&record_text)?;
    record.validate_for_toolchain(id)?;
    Ok(record)
}

fn require_installed_toolchain(paths: &AshgrovePaths, id: &ToolchainId) -> Result<()> {
    if !paths.toolchain_dir(id).is_dir() {
        bail!("toolchain '{}' is not installed", id.as_str());
    }
    installed_toolchain_manifest(paths, id)?;
    verify_install_record_any_source(&paths.toolchain_dir(id), id)?;
    Ok(())
}

fn project_toolchain_pin(project: &Path) -> Result<Option<String>> {
    let manifest = project.join("ash.toml");
    if !manifest.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(manifest).context("read ash.toml")?;
    let value: toml::Value = toml::from_str(&text).context("parse ash.toml")?;
    Ok(value
        .get("toolchain")
        .and_then(|table| table.get("ash"))
        .and_then(toml::Value::as_str)
        .map(ToOwned::to_owned))
}

fn remove_toolchain(paths: &AshgrovePaths, id: &ToolchainId, force: bool) -> Result<()> {
    if std::env::var("ASHGROVE_RUNNING_TOOLCHAIN").ok().as_deref() == Some(id.as_str()) {
        bail!(
            "refusing to remove running manager toolchain '{}'",
            id.as_str()
        );
    }
    if live_daemon_uses_toolchain(paths, id)? {
        bail!("refusing to remove live daemon toolchain '{}'", id.as_str());
    }
    let overrides_default = read_default(paths)?.as_ref() == Some(id);
    let overrides_current_project = current_project_uses_toolchain(id)?;
    if overrides_default && !force {
        bail!("refusing to remove default toolchain '{}'", id.as_str());
    }
    if overrides_current_project && !force {
        bail!(
            "refusing to remove current project toolchain '{}'",
            id.as_str()
        );
    }
    if force && (overrides_default || overrides_current_project) {
        confirm_remove_force(id)?;
    }
    let dir = paths.toolchain_dir(id);
    if dir.exists() {
        fs::remove_dir_all(dir).context("remove toolchain")?;
    }
    Ok(())
}

fn current_project_uses_toolchain(id: &ToolchainId) -> Result<bool> {
    let cwd = std::env::current_dir().context("read current directory")?;
    Ok(project_toolchain_pin(&cwd)?.as_deref() == Some(id.as_str()))
}

fn live_daemon_uses_toolchain(paths: &AshgrovePaths, id: &ToolchainId) -> Result<bool> {
    let daemon_dir = paths.state_dir().join("daemon");
    if !daemon_dir.exists() {
        return Ok(false);
    }
    for entry in fs::read_dir(&daemon_dir).context("read daemon state dir")? {
        let entry = entry.context("read daemon state entry")?;
        let metadata = entry
            .file_type()
            .with_context(|| format!("read daemon state type {}", entry.path().display()))?;
        if !metadata.is_file() {
            continue;
        }
        if entry.path().extension().and_then(OsStr::to_str) != Some("toml") {
            continue;
        }
        let text = fs::read_to_string(entry.path()).context("read daemon state")?;
        let value: toml::Value = toml::from_str(&text).context("parse daemon state")?;
        if value.get("toolchain_id").and_then(toml::Value::as_str) == Some(id.as_str()) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn confirm_remove_force(id: &ToolchainId) -> Result<()> {
    eprintln!(
        "confirmation required: type 'yes' or '{}' to remove protected toolchain '{}'",
        id.as_str(),
        id.as_str()
    );
    let answer = read_stdin_confirmation()?;
    if answer == "yes" || answer == id.as_str() {
        return Ok(());
    }
    bail!(
        "confirmation required to remove protected toolchain '{}'",
        id.as_str()
    );
}

fn confirm_cleanup_old_toolchains(candidates: &[ToolchainId]) -> Result<()> {
    eprintln!(
        "confirmation required: type 'yes' to remove {} old toolchain(s)",
        candidates.len()
    );
    let answer = read_stdin_confirmation()?;
    if answer == "yes" {
        return Ok(());
    }
    bail!("confirmation required to remove old toolchains");
}

fn read_stdin_confirmation() -> Result<String> {
    let mut input = String::new();
    let bytes = std::io::stdin()
        .read_line(&mut input)
        .context("read confirmation")?;
    if bytes == 0 {
        return Ok(String::new());
    }
    if let Some(stripped) = input.strip_suffix('\n') {
        input.truncate(stripped.len());
        if let Some(stripped) = input.strip_suffix('\r') {
            input.truncate(stripped.len());
        }
    }
    Ok(input)
}

fn cleanup(paths: &AshgrovePaths, args: CleanupArgs) -> Result<()> {
    let has_cleanup_flags = args.cache || args.orphans || args.old_toolchains;
    let project_pin = args
        .project
        .as_deref()
        .map(project_toolchain_pin)
        .transpose()?
        .flatten()
        .map(|pin| ToolchainId::parse(&pin))
        .transpose()?;

    if args.dry_run && args.project.is_some() && !has_cleanup_flags {
        cleanup_project_dry_run_plan(project_pin.as_ref());
        return Ok(());
    }

    if args.old_toolchains && !args.dry_run {
        cleanup_old_toolchains(paths, project_pin.as_ref(), args.dry_run)?;
    }
    if args.cache {
        cleanup_cache(paths, args.dry_run)?;
    }
    if args.orphans {
        cleanup_orphan_toolchain_dirs(paths, args.dry_run)?;
    }
    if args.old_toolchains && args.dry_run {
        cleanup_old_toolchains(paths, project_pin.as_ref(), args.dry_run)?;
    }

    if args.dry_run || has_cleanup_flags {
        return Ok(());
    }
    bail!("cleanup requires at least one of --cache, --orphans, or --old-toolchains");
}

fn cleanup_project_dry_run_plan(project_pin: Option<&ToolchainId>) {
    match project_pin {
        Some(id) => println!("protected project {}", id.as_str()),
        None => println!("protected project none"),
    }
    println!("no destructive cleanup will occur");
}

fn cleanup_cache(paths: &AshgrovePaths, dry_run: bool) -> Result<()> {
    let cache_dir = paths.cache_dir();
    if !cache_dir.exists() {
        return Ok(());
    }
    for name in ["downloads", "git", "builds", "module-cache"] {
        let path = cache_dir.join(name);
        if !path.exists() {
            continue;
        }
        if dry_run {
            println!("would remove cache {}", path.display());
        } else {
            remove_path(&path).with_context(|| format!("remove cache {}", path.display()))?;
            println!("removed cache {}", path.display());
        }
    }
    Ok(())
}

fn cleanup_orphan_toolchain_dirs(paths: &AshgrovePaths, dry_run: bool) -> Result<()> {
    if !paths.toolchains_dir().exists() {
        return Ok(());
    }
    for entry in fs::read_dir(paths.toolchains_dir()).context("read toolchains dir")? {
        let entry = entry.context("read toolchain dir entry")?;
        if !entry
            .file_type()
            .with_context(|| format!("read file type {}", entry.path().display()))?
            .is_dir()
        {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(ToOwned::to_owned) else {
            continue;
        };
        let Ok(id) = ToolchainId::parse(&name) else {
            continue;
        };
        if installed_toolchain_manifest(paths, &id).is_ok()
            && verify_install_record_any_source(&entry.path(), &id).is_ok()
        {
            continue;
        }
        if dry_run {
            println!("would remove orphan {}", entry.path().display());
        } else {
            fs::remove_dir_all(entry.path()).context("remove orphan toolchain")?;
            println!("removed orphan {}", id.as_str());
        }
    }
    Ok(())
}

fn cleanup_old_toolchains(
    paths: &AshgrovePaths,
    project_pin: Option<&ToolchainId>,
    dry_run: bool,
) -> Result<()> {
    let mut ids = installed_toolchain_ids(paths)?;
    ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    let mut deletion_candidates = Vec::new();
    for id in ids {
        if let Some(reason) = cleanup_protection_reason(paths, &id, project_pin)? {
            println!("protected {reason} {}", id.as_str());
            continue;
        }
        let dir = paths.toolchain_dir(&id);
        if dry_run {
            println!("would remove {}", dir.display());
        } else {
            deletion_candidates.push(id);
        }
    }
    if dry_run || deletion_candidates.is_empty() {
        return Ok(());
    }
    confirm_cleanup_old_toolchains(&deletion_candidates)?;
    for id in deletion_candidates {
        let dir = paths.toolchain_dir(&id);
        fs::remove_dir_all(&dir).context("remove old toolchain")?;
        println!("removed {}", id.as_str());
    }
    Ok(())
}

fn cleanup_protection_reason(
    paths: &AshgrovePaths,
    id: &ToolchainId,
    project_pin: Option<&ToolchainId>,
) -> Result<Option<&'static str>> {
    if std::env::var("ASHGROVE_RUNNING_TOOLCHAIN").ok().as_deref() == Some(id.as_str()) {
        return Ok(Some("running manager"));
    }
    if live_daemon_uses_toolchain(paths, id)? {
        return Ok(Some("live daemon"));
    }
    if read_default(paths)?.as_ref() == Some(id) {
        return Ok(Some("default"));
    }
    if project_pin == Some(id) || current_project_uses_toolchain(id)? {
        return Ok(Some("project"));
    }
    if read_selector(paths)?.project_pins().any(|pin| pin == id) {
        return Ok(Some("project"));
    }
    Ok(None)
}

fn remove_path(path: &Path) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn lock(project: &Path, check: bool) -> Result<()> {
    reject_legacy_conflict(project)?;
    let manifest = Manifest::read(project)?;
    let lock_path = project.join("ash.lock");
    let preserved_trust = if lock_path.exists() {
        Some(read_lock_trust(&lock_path)?).flatten()
    } else {
        None
    };
    let expected = manifest.lock_text(project, preserved_trust)?;
    if check {
        let current = fs::read_to_string(&lock_path).context("read ash.lock")?;
        if normalize_ws(&current) != normalize_ws(&expected) {
            bail!("lockfile drift detected");
        }
        return Ok(());
    }
    fs::write(lock_path, expected).context("write ash.lock")?;
    Ok(())
}

fn fetch(paths: &AshgrovePaths, project: &Path) -> Result<()> {
    let lock_path = project.join("ash.lock");
    if !lock_path.exists() {
        lock(project, false)?;
    }
    let lock = read_lock(project)?;
    materialize_locked_packages(paths, project, &lock)
}

fn vendor(paths: &AshgrovePaths, project: &Path, output: Option<&Path>, check: bool) -> Result<()> {
    let lock = read_lock(project)?;
    let out = output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| project.join("vendor/ash"));
    if check {
        for package in &lock.package {
            validate_package_name(&package.name)?;
            validate_commit(&package.commit)?;
            let name = &package.name;
            let provenance_path = out.join(name).join("provenance.toml");
            if !provenance_path.is_file() {
                bail!("vendor check failed for package '{name}'");
            }
            let provenance_text = fs::read_to_string(&provenance_path)
                .with_context(|| format!("read vendor provenance for package '{name}'"))?;
            let provenance: LockedPackage = toml::from_str(&provenance_text)
                .with_context(|| format!("parse vendor provenance for package '{name}'"))?;
            if &provenance != package {
                bail!("vendor provenance does not match lockfile for package '{name}'");
            }
            let source = locked_package_root(paths, package);
            if !source.is_dir() {
                bail!(
                    "locked package '{}' has not been materialized; run ashgrove fetch first",
                    package.name
                );
            }
            compare_vendor_content(&source, &out.join(name)).with_context(|| {
                format!("vendor content does not match lockfile for package '{name}'")
            })?;
        }
        let expected_packages = lock
            .package
            .iter()
            .map(|package| package.name.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        if out.exists() {
            for entry in fs::read_dir(&out).context("read vendor root")? {
                let entry = entry.context("read vendor root entry")?;
                let file_type = entry.file_type().with_context(|| {
                    format!("read vendor entry type {}", entry.path().display())
                })?;
                let name = entry
                    .file_name()
                    .into_string()
                    .map_err(|_| anyhow!("vendor package name is not utf8"))?;
                if !file_type.is_dir() || !expected_packages.contains(name.as_str()) {
                    bail!("vendor contains unexpected package '{name}'");
                }
            }
        }
        return Ok(());
    }
    for package in &lock.package {
        validate_package_name(&package.name)?;
        validate_commit(&package.commit)?;
        let name = &package.name;
        let dest = out.join(name);
        let source = locked_package_root(paths, package);
        if !source.is_dir() {
            bail!(
                "locked package '{}' has not been materialized; run ashgrove fetch first",
                package.name
            );
        }
        if dest.exists() {
            fs::remove_dir_all(&dest).context("replace vendor package")?;
        }
        fs::create_dir_all(&dest).context("create vendor package")?;
        copy_package_content(&source, &dest)?;
        fs::write(
            dest.join("provenance.toml"),
            toml::to_string(package).context("serialize provenance")?,
        )
        .context("write provenance")?;
    }
    Ok(())
}

fn read_lock(project: &Path) -> Result<LockFile> {
    let lock_path = project.join("ash.lock");
    let text = fs::read_to_string(&lock_path).context("read ash.lock")?;
    toml::from_str(&text).context("parse ash.lock")
}

fn read_lock_trust(lock_path: &Path) -> Result<Option<toml::Value>> {
    let text = fs::read_to_string(lock_path).context("read ash.lock")?;
    let value: toml::Value = toml::from_str(&text).context("parse ash.lock")?;
    Ok(value.get("trust").cloned())
}

fn materialize_locked_packages(
    paths: &AshgrovePaths,
    _project: &Path,
    lock: &LockFile,
) -> Result<()> {
    for package in &lock.package {
        validate_package_name(&package.name)?;
        validate_commit(&package.commit)?;
        materialize_locked_package(paths, package)?;
    }
    Ok(())
}

fn materialize_locked_package(paths: &AshgrovePaths, package: &LockedPackage) -> Result<()> {
    let repo = locked_package_repo(paths, package);
    let checkout = locked_package_root(paths, package);
    if !repo.exists() {
        fs::create_dir_all(repo.parent().context("repo parent")?).context("create repo cache")?;
        run_git_command(
            Path::new("."),
            &["clone", "--mirror", &package.git, repo_str(&repo)?],
            &format!("clone git dependency '{}'", package.name),
        )?;
    } else {
        run_git_command(
            &repo,
            &["fetch", "--tags", "--prune"],
            &format!("fetch git dependency '{}'", package.name),
        )?;
    }

    if checkout.exists() {
        ensure_checkout_commit(&checkout, &package.commit)?;
        return Ok(());
    }

    fs::create_dir_all(checkout.parent().context("checkout parent")?)
        .context("create checkout cache")?;
    let temp = tempfile::tempdir_in(checkout.parent().context("checkout parent")?)
        .context("create checkout staging dir")?;
    run_git_command(
        Path::new("."),
        &["clone", repo_str(&repo)?, path_str(temp.path())?],
        &format!("checkout git dependency '{}'", package.name),
    )?;
    run_git_command(
        temp.path(),
        &["checkout", "--detach", &package.commit],
        &format!("checkout locked commit for '{}'", package.name),
    )?;
    ensure_checkout_commit(temp.path(), &package.commit)?;
    fs::rename(temp.path(), &checkout).context("publish checkout cache")?;
    Ok(())
}

fn locked_package_repo(paths: &AshgrovePaths, package: &LockedPackage) -> PathBuf {
    paths.cache_dir().join("git/repos").join(format!(
        "{}-{}.git",
        package.name,
        git_url_digest(&package.git)
    ))
}

fn locked_package_root(paths: &AshgrovePaths, package: &LockedPackage) -> PathBuf {
    paths
        .cache_dir()
        .join("git/checkouts")
        .join(format!("{}-{}", package.name, git_url_digest(&package.git)))
        .join(&package.commit)
}

fn git_url_digest(url: &str) -> String {
    let digest = Sha256::digest(url.as_bytes());
    let mut out = String::new();
    for byte in &digest[..8] {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn ensure_checkout_commit(checkout: &Path, commit: &str) -> Result<()> {
    let output = git_output(checkout, &["rev-parse", "HEAD"], "read checkout HEAD")?;
    if output.trim() != commit {
        bail!("cached checkout is not at locked commit {commit}");
    }
    Ok(())
}

fn run_git_command(cwd: &Path, args: &[&str], context: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["-c", "commit.gpgsign=false", "-c", "tag.gpgSign=false"])
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| context.to_string())?;
    if !output.status.success() {
        bail!("{context}: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

fn git_output(cwd: &Path, args: &[&str], context: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["-c", "commit.gpgsign=false", "-c", "tag.gpgSign=false"])
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| context.to_string())?;
    if !output.status.success() {
        bail!("{context}: {}", String::from_utf8_lossy(&output.stderr));
    }
    String::from_utf8(output.stdout).context("git output utf8")
}

fn repo_str(path: &Path) -> Result<&str> {
    path.to_str().context("repo path utf8")
}

fn path_str(path: &Path) -> Result<&str> {
    path.to_str().context("path utf8")
}

fn copy_package_content(source: &Path, dest: &Path) -> Result<()> {
    for entry in walkdir::WalkDir::new(source) {
        let entry = entry.context("walk package source")?;
        let rel = entry
            .path()
            .strip_prefix(source)
            .context("strip package root")?;
        if rel.as_os_str().is_empty() || rel.components().any(|part| part.as_os_str() == ".git") {
            continue;
        }
        let target = dest.join(rel);
        if entry.file_type().is_symlink() {
            bail!("refusing to vendor symlink {}", entry.path().display());
        }
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).with_context(|| format!("create {}", target.display()))?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).context("create parent")?;
            }
            fs::copy(entry.path(), &target)
                .with_context(|| format!("copy {}", entry.path().display()))?;
        }
    }
    Ok(())
}

fn compare_vendor_content(source: &Path, vendor: &Path) -> Result<()> {
    let source_files = collect_package_files(source, None)?;
    let vendor_files = collect_package_files(vendor, Some(Path::new("provenance.toml")))?;
    if source_files != vendor_files {
        bail!("vendor content differs from locked checkout");
    }
    Ok(())
}

fn collect_package_files(
    root: &Path,
    skip_exact: Option<&Path>,
) -> Result<std::collections::BTreeMap<PathBuf, Vec<u8>>> {
    let mut files = std::collections::BTreeMap::new();
    for entry in walkdir::WalkDir::new(root) {
        let entry = entry.context("walk package content")?;
        let rel = entry
            .path()
            .strip_prefix(root)
            .context("strip package root")?;
        if rel.as_os_str().is_empty()
            || rel.components().any(|part| part.as_os_str() == ".git")
            || skip_exact == Some(rel)
        {
            continue;
        }
        if entry.file_type().is_symlink() {
            bail!("refusing symlink {}", entry.path().display());
        }
        if entry.file_type().is_file() {
            files.insert(
                rel.to_path_buf(),
                fs::read(entry.path())
                    .with_context(|| format!("read {}", entry.path().display()))?,
            );
        } else if !entry.file_type().is_dir() {
            bail!(
                "refusing unsupported package entry {}",
                entry.path().display()
            );
        }
    }
    Ok(files)
}

#[derive(Debug, Deserialize, Serialize)]
struct LockFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    trust: Option<toml::Value>,
    #[serde(default)]
    package: Vec<LockedPackage>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
struct LockedPackage {
    name: String,
    git: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    commit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_path: Option<String>,
}

fn reject_legacy_conflict(project: &Path) -> Result<()> {
    let legacy = project.join(".ash.toml");
    if !legacy.exists() {
        return Ok(());
    }
    let legacy_text = fs::read_to_string(legacy).context("read .ash.toml")?;
    if legacy_text.contains("[package]")
        || legacy_text.contains("[dependencies")
        || legacy_text.contains("[toolchain]")
    {
        bail!("legacy .ash.toml conflicts with canonical ash.toml package metadata");
    }
    Ok(())
}

#[derive(Debug)]
struct Manifest {
    dependencies: Vec<Dependency>,
}

impl Manifest {
    fn read(project: &Path) -> Result<Self> {
        let text = fs::read_to_string(project.join("ash.toml")).context("read ash.toml")?;
        let value: toml::Value = toml::from_str(&text).context("parse ash.toml")?;
        let dependencies = value
            .get("dependencies")
            .and_then(toml::Value::as_table)
            .map(|table| {
                table
                    .iter()
                    .map(|(name, value)| Dependency::from_value(name, value))
                    .collect::<Result<Vec<_>>>()
            })
            .transpose()?
            .unwrap_or_default();
        Ok(Self { dependencies })
    }

    fn lock_text(&self, project: &Path, trust: Option<toml::Value>) -> Result<String> {
        let mut package = Vec::with_capacity(self.dependencies.len());
        for dep in &self.dependencies {
            let commit = dep.resolve_commit(project)?;
            let rev = dep.rev.as_ref().map(|_| commit.clone());
            package.push(LockedPackage {
                name: dep.name.clone(),
                git: dep.git.clone(),
                tag: dep.tag.clone(),
                rev,
                commit,
                source_path: dep.local_path().map(|path| path.display().to_string()),
            });
        }
        toml::to_string(&LockFile { trust, package }).context("serialize ash.lock")
    }
}

#[derive(Debug)]
struct Dependency {
    name: String,
    git: String,
    tag: Option<String>,
    rev: Option<String>,
}

impl Dependency {
    fn from_value(name: &str, value: &toml::Value) -> Result<Self> {
        validate_package_name(name)?;
        let git = value
            .get("git")
            .and_then(toml::Value::as_str)
            .context("git dependency missing git URL")?
            .to_string();
        let tag = value
            .get("tag")
            .and_then(toml::Value::as_str)
            .map(ToOwned::to_owned);
        let rev = value
            .get("rev")
            .and_then(toml::Value::as_str)
            .map(ToOwned::to_owned);
        if tag.is_none() && rev.is_none() {
            bail!("unpinned git dependency '{name}' must specify tag or rev");
        }
        Ok(Self {
            name: name.to_string(),
            git,
            tag,
            rev,
        })
    }

    fn resolve_commit(&self, project: &Path) -> Result<String> {
        let reference = self.rev.as_ref().or(self.tag.as_ref()).expect("validated");
        let path = self
            .local_path()
            .unwrap_or_else(|| project.join(".ash/cache/git").join(&self.name));
        let output = Command::new("git")
            .args([
                "-C",
                path.to_str().context("git path utf8")?,
                "rev-parse",
                reference,
            ])
            .output()
            .with_context(|| format!("resolve git dependency '{}'", self.name))?;
        if !output.status.success() {
            bail!(
                "failed to resolve git dependency '{}': {}",
                self.name,
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(String::from_utf8(output.stdout)
            .context("git output utf8")?
            .trim()
            .to_string())
    }

    fn local_path(&self) -> Option<PathBuf> {
        self.git.strip_prefix("file://").map(PathBuf::from)
    }
}

fn validate_package_name(name: &str) -> Result<()> {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        bail!("invalid package name '{name}'");
    };
    if !first.is_ascii_alphanumeric()
        || !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        bail!("invalid package name '{name}'");
    }
    Ok(())
}

fn validate_commit(commit: &str) -> Result<()> {
    if commit.len() != 40 || !commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("locked git commit must be a full 40-character commit hash");
    }
    Ok(())
}

fn file_digest(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path).context("open digest input")?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf).context("read digest input")?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn normalize_ws(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
