use super::*;

pub(super) fn install_from_source(
    paths: &AshgrovePaths,
    source: &Path,
    allow_dirty: bool,
    allow_unidentified: bool,
    switch: bool,
    expected_id: Option<&ToolchainId>,
) -> Result<ToolchainId> {
    let input_kind = classify_source_input(source)?;
    if input_kind.uses_live_source_root_policy() {
        return install_from_source_root(
            paths,
            source,
            input_kind,
            allow_dirty,
            allow_unidentified,
            switch,
            expected_id,
        );
    }
    if source.join(".dirty").exists() && !allow_dirty {
        bail!(
            "dirty source rejected; pass --allow-dirty-source to record a non-reproducible install"
        );
    }
    if input_kind == SourceInputKind::SourceShapedArchive {
        return install_from_source_shaped_archive(
            paths,
            source,
            allow_dirty,
            allow_unidentified,
            switch,
            expected_id,
        );
    }
    let release_source =
        SourceArchiveReleaseMetadata::read_from_source_archive(source, allow_unidentified)?;
    let source_archive_digest = source_archive_tree_digest(source)?;
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
            source_payload_digest_policy: None,
            source_payload_digest: None,
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
    #[serde(default)]
    attestation: Option<SourceArchiveAttestation>,
}

#[derive(Debug, Deserialize)]
struct SourceArchiveAttestation {
    #[serde(default)]
    required: bool,
    origin_commit: Option<String>,
}

impl SourceArchiveReleaseMetadata {
    fn read_optional_from_source(source: &Path, require_attestation: bool) -> Result<Option<Self>> {
        let path = source.join("release-source.toml");
        if !path.is_file() {
            return Ok(None);
        }
        Self::read_from_source(source, false, require_attestation)
    }

    fn read_from_source_archive(source: &Path, allow_unidentified: bool) -> Result<Option<Self>> {
        Self::read_from_source(source, allow_unidentified, true)
    }

    fn read_from_source(
        source: &Path,
        allow_unidentified: bool,
        require_attestation: bool,
    ) -> Result<Option<Self>> {
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
        metadata.validate(require_attestation)?;
        if let Some(legacy_rev) = read_optional_trimmed(source.join(".source-rev"))?
            && legacy_rev != metadata.origin_commit
        {
            bail!("release-source origin_commit does not match legacy source revision");
        }
        Ok(Some(metadata))
    }

    fn validate(&self, require_attestation: bool) -> Result<()> {
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
        let Some(attestation) = &self.attestation else {
            if require_attestation {
                bail!("source archive attestation evidence is required");
            }
            return Ok(());
        };
        if require_attestation || attestation.required {
            let attested = attestation
                .origin_commit
                .as_deref()
                .context("source archive attestation origin_commit is required")?;
            if attested != self.origin_commit {
                bail!(
                    "source archive attestation origin_commit mismatch: expected {}, got {attested}",
                    self.origin_commit
                );
            }
        }
        Ok(())
    }
}

fn is_source_root(source: &Path) -> bool {
    source.join("Cargo.toml").is_file() && source.join("std/src").is_dir()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceInputKind {
    LiveGitSourceRoot,
    LiveNonGitSourceRoot,
    SourceShapedArchive,
    SourceArchive,
}

impl SourceInputKind {
    fn uses_live_source_root_policy(self) -> bool {
        matches!(self, Self::LiveGitSourceRoot | Self::LiveNonGitSourceRoot)
    }

    fn payload_membership(self) -> Option<SourceRootPayloadMembership> {
        match self {
            Self::LiveGitSourceRoot => Some(SourceRootPayloadMembership::Git),
            Self::LiveNonGitSourceRoot => Some(SourceRootPayloadMembership::NonGit),
            Self::SourceShapedArchive | Self::SourceArchive => None,
        }
    }
}

fn classify_source_input(source: &Path) -> Result<SourceInputKind> {
    if !is_source_root(source) {
        return Ok(SourceInputKind::SourceArchive);
    }
    if source.join("release-source.toml").is_file() {
        return Ok(SourceInputKind::SourceShapedArchive);
    }
    if has_git_marker_at(source) || git_is_inside_work_tree(source, has_git_marker(source))? {
        return Ok(SourceInputKind::LiveGitSourceRoot);
    }
    Ok(SourceInputKind::LiveNonGitSourceRoot)
}

fn install_from_source_shaped_archive(
    paths: &AshgrovePaths,
    source: &Path,
    allow_dirty: bool,
    allow_unidentified: bool,
    switch: bool,
    expected_id: Option<&ToolchainId>,
) -> Result<ToolchainId> {
    let release_source =
        SourceArchiveReleaseMetadata::read_from_source_archive(source, allow_unidentified)?;
    let source_archive_digest = source_archive_tree_digest(source)?;
    let source_rev = release_source
        .as_ref()
        .map(|metadata| metadata.origin_commit.as_str());
    let source_url = read_optional_trimmed(source.join(".source-url"))?;
    let dirty_source_digest = if allow_dirty {
        Some(source_archive_digest.as_str())
    } else {
        None
    };
    let version = source_package_version(source)?;
    let id = source_toolchain_id(&version, source, source_rev, dirty_source_digest)?;
    verify_expected_source_id(expected_id, &id)?;
    let stage = ToolchainStage::create(paths, id.clone())?;
    let build_payload = SourceRootBuildPayload::SourceArchive {
        digest: source_archive_digest.clone(),
    };
    stage_source_root_toolchain(paths, source, &stage, &id, &version, &build_payload)?;
    write_source_install_record(
        &stage.path().join("install-record.toml"),
        SourceInstallRecordInput {
            id: &id,
            source_path: source,
            source_rev,
            source_url: source_url.as_deref(),
            source_origin_commit: source_rev,
            source_archive_digest: Some(source_archive_digest.as_str()),
            source_payload_digest_policy: None,
            source_payload_digest: None,
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

fn install_from_source_root(
    paths: &AshgrovePaths,
    source: &Path,
    input_kind: SourceInputKind,
    allow_dirty: bool,
    allow_unidentified: bool,
    switch: bool,
    expected_id: Option<&ToolchainId>,
) -> Result<ToolchainId> {
    let metadata = SourceRootMetadata::inspect(source)?;
    let release_source =
        SourceArchiveReleaseMetadata::read_optional_from_source(source, metadata.rev.is_none())?;
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
    let build_payload = SourceRootBuildPayload::inspect(source, input_kind)?;
    let source_digest = build_payload.digest();
    let source_archive_digest = if input_kind == SourceInputKind::SourceShapedArchive {
        release_source.as_ref().map(|_| source_digest)
    } else {
        None
    };
    let source_payload_digest = if input_kind.uses_live_source_root_policy() {
        Some(source_digest)
    } else {
        None
    };
    let dirty_digest = if metadata.dirty {
        Some(source_digest)
    } else {
        None
    };
    let id = source_toolchain_id(&version, source, source_rev, dirty_digest)?;
    verify_expected_source_id(expected_id, &id)?;
    let stage = ToolchainStage::create(paths, id.clone())?;
    stage_source_root_toolchain(paths, source, &stage, &id, &version, &build_payload)?;
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
            source_payload_digest_policy: source_payload_digest
                .map(|_| SOURCE_ROOT_PAYLOAD_DIGEST_POLICY),
            source_payload_digest,
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
        let git_like =
            has_git_marker_at(source) || git_is_inside_work_tree(source, has_git_marker(source))?;
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
            url: url
                .map(|value| canonical_git_url_for_lock(value.trim()).map(|origin| origin.url))
                .transpose()?,
            dirty,
        })
    }
}

fn git_is_inside_work_tree(source: &Path, fail_closed: bool) -> Result<bool> {
    let output = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(source)
        .output()
        .with_context(|| {
            format!(
                "run git rev-parse --is-inside-work-tree in {}",
                source.display()
            )
        })?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim() == "true");
    }
    if fail_closed {
        bail!(
            "git work tree detection failed for git-like source root {}: {}",
            source.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(false)
}

fn has_git_marker(source: &Path) -> bool {
    source.ancestors().any(has_resolvable_git_marker_at)
}

fn has_git_marker_at(path: &Path) -> bool {
    fs::symlink_metadata(path.join(".git")).is_ok()
}

fn has_resolvable_git_marker_at(path: &Path) -> bool {
    let marker = path.join(".git");
    if marker.is_file() {
        return true;
    }
    marker.is_dir() && marker.join("HEAD").exists()
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
    build_payload: &SourceRootBuildPayload,
) -> Result<()> {
    build_source_binaries(paths, source, id, build_payload)?;
    let post_build_digest = build_payload.post_build_digest(source)?;
    if post_build_digest != build_payload.digest() {
        match build_payload {
            SourceRootBuildPayload::LiveRoot { .. } => {
                bail!(
                    "source-payload-changed: source payload changed during build for {}; ignored local state is excluded from live source-root payload checks; aborting before publish",
                    source.display()
                );
            }
            SourceRootBuildPayload::SourceArchive { .. } => {
                bail!(
                    "source-payload-changed: source archive payload changed during build for {}; aborting before publish",
                    source.display()
                );
            }
        }
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

fn build_source_binaries(
    paths: &AshgrovePaths,
    source: &Path,
    id: &ToolchainId,
    build_payload: &SourceRootBuildPayload,
) -> Result<()> {
    let target_dir = source_build_dir(paths, id);
    fs::create_dir_all(&target_dir).context("create source build target dir")?;
    let build_source_root = paths.cache_dir().join("source-build-roots");
    fs::create_dir_all(&build_source_root).context("create source build root cache")?;
    let build_source =
        tempfile::tempdir_in(&build_source_root).context("create isolated source build dir")?;
    build_payload.copy_for_build(source, build_source.path())?;

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
    if build_source.path().join("Cargo.lock").is_file() {
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

#[derive(Debug, Clone, Copy)]
enum SourceRootPayloadMembership {
    Git,
    NonGit,
}

#[derive(Debug)]
enum SourceRootBuildPayload {
    LiveRoot {
        membership: SourceRootPayloadMembership,
        files: Vec<PathBuf>,
        digest: String,
    },
    SourceArchive {
        digest: String,
    },
}

impl SourceRootBuildPayload {
    fn inspect(source: &Path, input_kind: SourceInputKind) -> Result<Self> {
        if let Some(membership) = input_kind.payload_membership() {
            let files = source_root_payload_files(source, membership)?;
            let digest = digest_source_files(source, &files)?;
            return Ok(Self::LiveRoot {
                membership,
                files,
                digest,
            });
        }
        Ok(Self::SourceArchive {
            digest: source_archive_tree_digest(source)?,
        })
    }

    fn digest(&self) -> &str {
        match self {
            Self::LiveRoot { digest, .. } | Self::SourceArchive { digest } => digest,
        }
    }

    fn copy_for_build(&self, source: &Path, dest: &Path) -> Result<()> {
        match self {
            Self::LiveRoot { files, .. } => {
                copy_source_payload_files_for_build(source, dest, files)
            }
            Self::SourceArchive { .. } => copy_source_archive_tree_for_build(source, dest),
        }
    }

    fn post_build_digest(&self, source: &Path) -> Result<String> {
        match self {
            Self::LiveRoot { membership, .. } => source_root_payload_digest(source, *membership),
            Self::SourceArchive { .. } => source_archive_tree_digest(source),
        }
    }
}

fn source_root_payload_digest(
    source: &Path,
    membership: SourceRootPayloadMembership,
) -> Result<String> {
    let files = source_root_payload_files(source, membership)?;
    digest_source_files(source, &files)
}

fn source_root_payload_files(
    source: &Path,
    membership: SourceRootPayloadMembership,
) -> Result<Vec<PathBuf>> {
    match membership {
        SourceRootPayloadMembership::Git => git_source_root_payload_files(source),
        SourceRootPayloadMembership::NonGit => non_git_source_root_payload_files(source),
    }
}

fn git_source_root_payload_files(source: &Path) -> Result<Vec<PathBuf>> {
    let output = Command::new("git")
        .args([
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ])
        .current_dir(source)
        .output()
        .with_context(|| {
            format!(
                "run git source payload membership query in {}",
                source.display()
            )
        })?;
    if !output.status.success() {
        bail!(
            "source payload membership failed for git source root {}: {}",
            source.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let mut files = Vec::new();
    for raw in output.stdout.split(|byte| *byte == 0) {
        if raw.is_empty() {
            continue;
        }
        let rel = path_buf_from_git_bytes(raw)?;
        validate_source_payload_relative_path(&rel)?;
        if source.join(&rel).is_file() {
            files.push(rel);
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn path_buf_from_git_bytes(raw: &[u8]) -> Result<PathBuf> {
    Ok(PathBuf::from(
        std::str::from_utf8(raw).context("decode git source payload path as utf8")?,
    ))
}

fn validate_source_payload_relative_path(rel: &Path) -> Result<()> {
    if rel.as_os_str().is_empty() || rel.is_absolute() {
        bail!("invalid source payload path {}", rel.display());
    }
    for component in rel.components() {
        if !matches!(component, Component::Normal(_)) {
            bail!("invalid source payload path {}", rel.display());
        }
    }
    Ok(())
}

fn non_git_source_root_payload_files(source: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(source) {
        let entry = entry.context("walk non-git source payload input")?;
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry.path().strip_prefix(source).context("strip prefix")?;
        if non_git_source_root_ignore_path(rel) {
            continue;
        }
        files.push(rel.to_path_buf());
    }
    files.sort();
    Ok(files)
}

fn non_git_source_root_ignore_path(rel: &Path) -> bool {
    first_component_is(rel, ".git")
        || first_component_is(rel, ".agents")
        || first_component_is(rel, ".worktrees")
        || first_component_is(rel, ".codex")
        || rel_starts_with(rel, &["tools", "agent-pipeline", ".agents"])
        || rel.components().any(|component| {
            matches!(component, Component::Normal(name) if name == OsStr::new("target"))
        })
}

fn first_component_is(rel: &Path, expected: &str) -> bool {
    matches!(
        rel.components().next(),
        Some(Component::Normal(name)) if name == OsStr::new(expected)
    )
}

fn rel_starts_with(rel: &Path, expected: &[&str]) -> bool {
    let mut components = rel.components();
    for expected_component in expected {
        match components.next() {
            Some(Component::Normal(actual)) if actual == OsStr::new(expected_component) => {}
            _ => return false,
        }
    }
    true
}

fn copy_source_payload_files_for_build(
    source: &Path,
    dest: &Path,
    files: &[PathBuf],
) -> Result<()> {
    for rel in files {
        let target = dest.join(rel);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).context("create source payload parent")?;
        }
        fs::copy(source.join(rel), &target)
            .with_context(|| format!("copy source payload file {}", rel.display()))?;
    }
    Ok(())
}

fn copy_source_archive_tree_for_build(source: &Path, dest: &Path) -> Result<()> {
    for entry in walkdir::WalkDir::new(source) {
        let entry = entry.context("walk source build input")?;
        let rel = entry.path().strip_prefix(source).context("strip prefix")?;
        if rel.as_os_str().is_empty() || source_archive_digest_skip_path(rel) {
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

fn source_archive_tree_digest(source: &Path) -> Result<String> {
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(source) {
        let entry = entry.context("walk source digest input")?;
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry.path().strip_prefix(source).context("strip prefix")?;
        if source_archive_digest_skip_path(rel) {
            continue;
        }
        files.push(rel.to_path_buf());
    }
    files.sort();

    digest_source_files(source, &files)
}

fn digest_source_files(source: &Path, files: &[PathBuf]) -> Result<String> {
    let mut hasher = Sha256::new();
    for rel in files {
        hasher.update(rel.to_string_lossy().as_bytes());
        hasher.update([0]);
        let mut file = fs::File::open(source.join(rel))
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

fn source_archive_digest_skip_path(rel: &Path) -> bool {
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
    source_payload_digest_policy: Option<&'a str>,
    source_payload_digest: Option<&'a str>,
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
    if let Some(source_payload_digest_policy) = input.source_payload_digest_policy {
        table.insert(
            "source_payload_digest_policy".to_string(),
            toml::Value::String(source_payload_digest_policy.to_string()),
        );
    }
    if let Some(source_payload_digest) = input.source_payload_digest {
        table.insert(
            "source_payload_digest".to_string(),
            toml::Value::String(format!("sha256:{source_payload_digest}")),
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
