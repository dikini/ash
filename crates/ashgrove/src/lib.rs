//! User-local Ash toolchain and deployment manager.

use std::ffi::OsStr;
use std::fs;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, ExitCode};

use anyhow::{Context, Result, anyhow, bail};
use chrono::Utc;
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    #[arg(long)]
    to: String,
    #[arg(long = "from", value_enum)]
    source: InstallSource,
    #[arg(long)]
    path: Option<PathBuf>,
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
    Existing,
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

/// CLI entry point.
#[must_use]
pub fn main() -> ExitCode {
    match run_cli() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run_cli() -> Result<()> {
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
        Commands::Fetch(args) => fetch(args.project.as_deref().unwrap_or_else(|| Path::new("."))),
        Commands::Lock(args) => lock(
            args.project.as_deref().unwrap_or_else(|| Path::new(".")),
            args.check,
        ),
        Commands::Vendor(args) => vendor(
            args.project.as_deref().unwrap_or_else(|| Path::new(".")),
            args.output.as_deref(),
            args.check,
        ),
    }
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
            )
        }
        InstallSource::Tarball => {
            if let Some(url) = args.url {
                bail!(
                    "tarball URL install is reserved until authenticated download policy exists: {url}"
                );
            }
            let path = args
                .path
                .context("--path is required for tarball install")?;
            install_from_tarball(paths, &path, args.switch)
        }
        InstallSource::Existing => bail!("install --from existing is only valid for update tests"),
    }
}

fn update(paths: &AshgrovePaths, args: UpdateArgs) -> Result<()> {
    let id = ToolchainId::parse(&args.to)?;
    match args.source {
        InstallSource::Existing => {
            if !paths.toolchain_dir(&id).is_dir() {
                bail!("toolchain '{}' is not installed", id.as_str());
            }
            if args.switch {
                set_default(paths, &id)?;
            }
            Ok(())
        }
        InstallSource::Source => {
            let source = args.path.context("--path is required for source update")?;
            install_from_source(
                paths,
                &source,
                args.allow_dirty_source,
                args.allow_unidentified_source,
                args.switch,
            )
        }
        InstallSource::Tarball => {
            let path = args.path.context("--path is required for tarball update")?;
            install_from_tarball(paths, &path, args.switch)
        }
    }
}

fn install_from_source(
    paths: &AshgrovePaths,
    source: &Path,
    allow_dirty: bool,
    allow_unidentified: bool,
    switch: bool,
) -> Result<()> {
    if source.join(".dirty").exists() && !allow_dirty {
        bail!(
            "dirty source rejected; pass --allow-dirty-source to record a non-reproducible install"
        );
    }
    if !source.join(".source-rev").exists() && !allow_unidentified {
        bail!("unidentified source rejected; pass --allow-unidentified-source to record it");
    }
    let id = read_toolchain_id(source)?;
    publish_shape(paths, source, &id)?;
    let record = paths.toolchain_dir(&id).join("install-record.toml");
    append_record(
        &record,
        &format!(
            "source_kind = \"source\"\nallow_dirty_source = {}\nallow_unidentified_source = {}\ninstalled_at = \"{}\"\n",
            allow_dirty,
            allow_unidentified,
            Utc::now().to_rfc3339()
        ),
    )?;
    if switch || read_default(paths)?.is_none() {
        set_default(paths, &id)?;
    }
    Ok(())
}

fn install_from_tarball(paths: &AshgrovePaths, archive: &Path, switch: bool) -> Result<()> {
    let digest = file_digest(archive)?;
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
    verify_toolchain_shape(&root, &id)?;
    verify_required_binaries_executable(&root, &id)?;
    publish_shape(paths, &root, &id)?;
    append_record(
        &paths.toolchain_dir(&id).join("install-record.toml"),
        &format!(
            "source_kind = \"tarball\"\ntarball_digest = \"sha256:{digest}\"\ninstalled_at = \"{}\"\n",
            Utc::now().to_rfc3339()
        ),
    )?;
    if switch || read_default(paths)?.is_none() {
        set_default(paths, &id)?;
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

fn publish_shape(paths: &AshgrovePaths, source: &Path, id: &ToolchainId) -> Result<()> {
    verify_toolchain_shape(source, id)?;
    let dest = paths.toolchain_dir(id);
    if dest.exists() {
        verify_toolchain_shape(&dest, id)?;
        return Ok(());
    }
    fs::create_dir_all(paths.toolchains_dir()).context("create toolchains dir")?;
    copy_dir(source, &dest)?;
    Ok(())
}

fn verify_toolchain_shape(root: &Path, id: &ToolchainId) -> Result<()> {
    for rel in [
        "bin/ash",
        "bin/ashgrove",
        "lib/ash/std/ash.toml",
        "lib/ash/std/src",
        "manifest.toml",
        "install-record.toml",
    ] {
        let path = root.join(rel);
        if !path.exists() {
            bail!("toolchain '{}' missing required path {rel}", id.as_str());
        }
    }
    let manifest = fs::read_to_string(root.join("manifest.toml")).context("read manifest")?;
    if !manifest.contains(&format!("toolchain_id = \"{}\"", id.as_str())) {
        bail!("manifest toolchain id does not match {}", id.as_str());
    }
    Ok(())
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
        bail!("toolchain '{}' is not installed", id.as_str());
    }
    fs::create_dir_all(paths.config_dir()).context("create config dir")?;
    fs::write(
        paths.config_dir().join("toolchains.toml"),
        format!("default = \"{}\"\n", id.as_str()),
    )
    .context("write selector")?;
    println!("{}", id.as_str());
    Ok(())
}

fn read_default(paths: &AshgrovePaths) -> Result<Option<String>> {
    let path = paths.config_dir().join("toolchains.toml");
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(path).context("read selector")?;
    let value: toml::Value = toml::from_str(&text).context("parse selector")?;
    Ok(value
        .get("default")
        .and_then(toml::Value::as_str)
        .map(ToOwned::to_owned))
}

fn list_toolchains(paths: &AshgrovePaths) -> Result<()> {
    if !paths.toolchains_dir().exists() {
        return Ok(());
    }
    let mut ids = fs::read_dir(paths.toolchains_dir())?
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect::<Vec<_>>();
    ids.sort();
    for id in ids {
        println!("{id}");
    }
    Ok(())
}

fn current(paths: &AshgrovePaths, project: Option<&Path>) -> Result<()> {
    if let Some(project) = project
        && let Some(pin) = project_toolchain_pin(project)?
    {
        println!("{pin}");
        return Ok(());
    }
    let Some(default) = read_default(paths)? else {
        bail!("no default Ash toolchain is configured");
    };
    println!("{default}");
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
    if read_default(paths)?.as_deref() == Some(id.as_str()) && !force {
        bail!("refusing to remove default toolchain '{}'", id.as_str());
    }
    if current_project_uses_toolchain(id)? && !force {
        bail!(
            "refusing to remove current project toolchain '{}'",
            id.as_str()
        );
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

fn cleanup(paths: &AshgrovePaths, args: CleanupArgs) -> Result<()> {
    let _ = (&args.project, args.cache, args.orphans);
    if args.dry_run {
        if args.old_toolchains && paths.toolchains_dir().exists() {
            for entry in fs::read_dir(paths.toolchains_dir())?.flatten() {
                if entry.path().is_dir() {
                    println!("would remove {}", entry.path().display());
                }
            }
        }
        return Ok(());
    }
    bail!("cleanup without --dry-run is not implemented for this conservative first slice");
}

fn lock(project: &Path, check: bool) -> Result<()> {
    reject_legacy_conflict(project)?;
    let manifest = Manifest::read(project)?;
    let expected = manifest.lock_text(project)?;
    let lock_path = project.join("ash.lock");
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

fn fetch(project: &Path) -> Result<()> {
    lock(project, false)
}

fn vendor(project: &Path, output: Option<&Path>, check: bool) -> Result<()> {
    let lock_path = project.join("ash.lock");
    let text = fs::read_to_string(&lock_path).context("read ash.lock")?;
    let lock: LockFile = toml::from_str(&text).context("parse ash.lock")?;
    let out = output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| project.join("vendor/ash"));
    if check {
        for package in &lock.package {
            validate_package_name(&package.name)?;
            let name = &package.name;
            if !out.join(name).join("provenance.toml").is_file() {
                bail!("vendor check failed for package '{name}'");
            }
        }
        return Ok(());
    }
    for package in &lock.package {
        validate_package_name(&package.name)?;
        let name = &package.name;
        let dest = out.join(name);
        fs::create_dir_all(&dest).context("create vendor package")?;
        fs::write(
            dest.join("provenance.toml"),
            toml::to_string(package).context("serialize provenance")?,
        )
        .context("write provenance")?;
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
struct LockFile {
    #[serde(default)]
    package: Vec<LockedPackage>,
}

#[derive(Debug, Deserialize, Serialize)]
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

    fn lock_text(&self, project: &Path) -> Result<String> {
        let mut package = Vec::with_capacity(self.dependencies.len());
        for dep in &self.dependencies {
            let commit = dep.resolve_commit(project)?;
            package.push(LockedPackage {
                name: dep.name.clone(),
                git: dep.git.clone(),
                tag: dep.tag.clone(),
                rev: dep.rev.clone(),
                commit,
                source_path: dep.local_path().map(|path| path.display().to_string()),
            });
        }
        toml::to_string(&LockFile { package }).context("serialize ash.lock")
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

fn append_record(path: &Path, text: &str) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .context("open install record")?;
    file.write_all(text.as_bytes())
        .context("append install record")?;
    Ok(())
}

fn normalize_ws(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
