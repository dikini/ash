use super::*;

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
    pub(crate) extra: BTreeMap<String, toml::Value>,
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

    pub fn validate_archive_schema_version(&self) -> Result<()> {
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

    pub fn validate_archive_schema_version(&self) -> Result<()> {
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

/// Rewrite a project `ash.toml` without interpreting reserved trust/signing metadata.
///
/// This helper exists for read-modify-write callers that need to normalize or rewrite the
/// project manifest while preserving future-compatible trust metadata as opaque TOML data.
///
/// # Errors
///
/// Returns an error when `ash.toml` cannot be read, parsed, serialized, or written.
pub fn rewrite_project_manifest_preserving_trust_metadata(project: impl AsRef<Path>) -> Result<()> {
    let path = project.as_ref().join("ash.toml");
    let text = fs::read_to_string(&path).context("read ash.toml")?;
    let value: toml::Value = toml::from_str(&text).context("parse ash.toml")?;
    let rewritten = toml::to_string(&value).context("serialize ash.toml")?;
    fs::write(path, rewritten).context("write ash.toml")
}

pub fn stage_source_stdlib_metadata(
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
