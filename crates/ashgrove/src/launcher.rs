use std::io::Write as _;

use super::*;

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

pub fn install_launcher_shims_from_current_exe(paths: &AshgrovePaths) -> Result<()> {
    let current_exe = std::env::current_exe().context("resolve current ashgrove executable")?;
    fs::create_dir_all(paths.launcher_bin()).context("create launcher bin directory")?;
    let dispatcher = paths.launcher_bin().join(".ashgrove-dispatcher");
    install_executable_copy(&current_exe, &dispatcher)?;
    install_launcher_shims(paths, &dispatcher)
}

pub fn install_packaged_launcher_shims(
    paths: &AshgrovePaths,
    id: &ToolchainId,
    manifest: &ToolchainManifest,
) -> Result<()> {
    let root = paths.toolchain_dir(id);
    let manager = manifest
        .required_tool("ashgrove")
        .context("packaged toolchain manifest missing ashgrove manager tool")?;
    let manager_path = validate_contained_standard_tool_path(&root, manager.path())?;
    fs::create_dir_all(paths.launcher_bin()).context("create launcher bin directory")?;
    let dispatcher = paths.launcher_bin().join(".ashgrove-dispatcher");
    install_executable_copy(&manager_path, &dispatcher)?;
    DispatcherLifecycleMetadata::new(id.clone())
        .write_to_path(&paths.launcher_bin().join(DISPATCHER_LIFECYCLE_FILE))?;
    install_launcher_shims(paths, &dispatcher)
}

pub fn stable_dispatcher_manager_toolchain(paths: &AshgrovePaths) -> Result<Option<ToolchainId>> {
    let lifecycle = paths.launcher_bin().join(DISPATCHER_LIFECYCLE_FILE);
    if !lifecycle.exists() {
        return Ok(None);
    }
    Ok(Some(
        DispatcherLifecycleMetadata::read_from_path(&lifecycle)?.manager_toolchain_id,
    ))
}

pub fn install_executable_copy(source: &Path, target: &Path) -> Result<()> {
    let bytes = fs::read(source)
        .with_context(|| format!("read launcher dispatcher source {}", source.display()))?;
    write_file_atomically(target, ".ashgrove-dispatcher.tmp-", &bytes, true)
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
    write_file_atomically(target, temp_prefix, contents, true)
}

fn write_file_atomically(
    target: &Path,
    temp_prefix: &str,
    contents: &[u8],
    executable: bool,
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
    if executable {
        make_launcher_executable(temp.path())?;
    }
    let persisted = temp
        .persist(target)
        .map_err(|error| error.error)
        .with_context(|| format!("publish launcher file {}", target.display()))?;
    drop(persisted);
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DispatcherLifecycleMetadata {
    manager_toolchain_id: ToolchainId,
}

impl DispatcherLifecycleMetadata {
    fn new(manager_toolchain_id: ToolchainId) -> Self {
        Self {
            manager_toolchain_id,
        }
    }

    fn read_from_path(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("read dispatcher lifecycle metadata {}", path.display()))?;
        toml::from_str(&text).context("parse dispatcher lifecycle metadata")
    }

    fn write_to_path(&self, path: &Path) -> Result<()> {
        let text = toml::to_string(self).context("serialize dispatcher lifecycle metadata")?;
        write_file_atomically(
            path,
            ".ashgrove-dispatcher.toml.tmp-",
            text.as_bytes(),
            false,
        )
        .with_context(|| format!("publish dispatcher lifecycle metadata {}", path.display()))
    }
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
