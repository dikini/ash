//! User-local Ash toolchain and deployment manager.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::fmt::Write as _;
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, ExitCode};

use anyhow::{Context, Result, anyhow, bail};
use chrono::Utc;
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

const TOOLCHAIN_ARCHIVE_SCHEMA_VERSION: u32 = 1;
const DISPATCHER_LIFECYCLE_FILE: &str = ".ashgrove-dispatcher.toml";
const SOURCE_ROOT_PAYLOAD_DIGEST_POLICY: &str = "source-root-v2-gitignore-local-state";

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

    fn project_roots(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.projects.keys().map(PathBuf::from)
    }
}

mod launcher;
pub use launcher::{
    LauncherDispatch, LauncherDispatchRequest, LauncherSelectionSource, install_launcher_shims,
    resolve_launcher_dispatch,
};
use launcher::{
    install_executable_copy, install_launcher_shims_from_current_exe,
    install_packaged_launcher_shims, stable_dispatcher_manager_toolchain,
};

mod manifest;
use manifest::stage_source_stdlib_metadata;
pub use manifest::{
    CollisionStatus, InstallRecord, PublishOutcome, RuntimeSupportMetadata, StandardToolMetadata,
    StdlibMetadata, ToolchainManifest, ToolchainStage, classify_toolchain_collision,
    rewrite_project_manifest_preserving_trust_metadata, stage_stdlib_metadata,
};

mod cli;
use cli::CleanupArgs;
pub use cli::main;

mod source;
use source::install_from_source;

mod tarball;
use tarball::{
    install_from_tarball, install_from_tarball_url, parse_sha256_digest,
    validate_relative_toolchain_path, verify_required_manifest_tools,
    verify_runtime_support_payload, verify_signed_release_index, verify_stdlib_manifest,
    verify_toolchain_shape,
};

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

mod selector;
use selector::{
    cleanup, current, installed_toolchain_ids, installed_toolchain_manifest, list_toolchains,
    project_toolchain_pin, read_default, remove_toolchain, require_installed_toolchain,
    verify_install_record_any_source,
};

mod lockfile;
use lockfile::{
    LockFile, LockedPackage, canonical_git_url_for_lock, fetch, file_digest, lock,
    locked_package_repo, locked_package_root, normalize_ws, read_lock, validate_commit,
    validate_package_name, vendor,
};
