use super::*;

pub(super) fn install_from_tarball(
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
    enforce_toolchain_release_signature(&manifest, &id, archive, &digest)?;
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
    install_packaged_launcher_shims(paths, &id, &manifest)?;
    if switch || read_default(paths)?.is_none() {
        set_default(paths, &id)?;
    }
    Ok(id)
}

pub(super) fn install_from_tarball_url(
    paths: &AshgrovePaths,
    url: &str,
    switch: bool,
    expected_id: Option<&ToolchainId>,
    expected_digest: Option<&str>,
) -> Result<ToolchainId> {
    if expected_digest.is_none() {
        bail!(
            "tarball URL install requires authenticated download policy evidence: explicit sha256 digest"
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

pub(super) fn verify_signed_release_index(path: &Path) -> Result<()> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("read release index {}", path.display()))?;
    let value: toml::Value = toml::from_str(&text).context("parse release index")?;
    let signed = value
        .get("signing")
        .and_then(toml::Value::as_table)
        .and_then(|signing| signing.get("signature"))
        .and_then(toml::Value::as_str)
        .is_some_and(|signature| !signature.trim().is_empty());
    if !signed {
        bail!("unsigned release index rejected before publish");
    }
    bail!(
        "release-index signature binding is not implemented; explicit --digest remains required for URL install/update"
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

fn enforce_toolchain_release_signature(
    manifest: &ToolchainManifest,
    id: &ToolchainId,
    archive: &Path,
    archive_digest: &str,
) -> Result<()> {
    let Some(trust) = manifest.extra.get("trust") else {
        return Ok(());
    };
    let Some(release) = trust.get("release").and_then(toml::Value::as_table) else {
        return Ok(());
    };
    if !release
        .get("signature_required")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
    {
        return Ok(());
    }
    let evidence = ReleaseSignatureEvidence::read_for_archive(archive)?;
    if evidence.schema_version != TOOLCHAIN_ARCHIVE_SCHEMA_VERSION {
        bail!(
            "unsupported toolchain release signature schema version {}; expected {TOOLCHAIN_ARCHIVE_SCHEMA_VERSION}",
            evidence.schema_version
        );
    }
    if evidence.toolchain_id != *id {
        bail!(
            "toolchain release signature toolchain_id mismatch: expected {}, got {}",
            id.as_str(),
            evidence.toolchain_id.as_str()
        );
    }
    let expected = parse_sha256_digest(
        &evidence.tarball_digest,
        "toolchain release signature tarball_digest",
    )?;
    if expected != archive_digest {
        bail!(
            "toolchain release signature mismatch: expected sha256:{expected}, got sha256:{archive_digest}"
        );
    }
    if evidence.signature.trim().is_empty() {
        bail!("toolchain release signature evidence is empty");
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct ReleaseSignatureEvidence {
    schema_version: u32,
    toolchain_id: ToolchainId,
    tarball_digest: String,
    signature: String,
}

impl ReleaseSignatureEvidence {
    fn read_for_archive(archive: &Path) -> Result<Self> {
        let path = release_signature_sidecar_path(archive);
        let text = fs::read_to_string(&path).with_context(|| {
            format!(
                "required toolchain release signature evidence is missing at {}",
                path.display()
            )
        })?;
        toml::from_str(&text).context("parse toolchain release signature evidence")
    }
}

fn release_signature_sidecar_path(archive: &Path) -> PathBuf {
    PathBuf::from(format!("{}.release-signature.toml", archive.display()))
}

fn verify_expected_tarball_digest(
    expected_digest: Option<&str>,
    actual_digest: &str,
) -> Result<()> {
    if let Some(expected_digest) = expected_digest {
        let expected = parse_sha256_digest(expected_digest, "tarball digest")?;
        if expected != actual_digest {
            bail!(
                "tarball digest mismatch: expected sha256:{expected}, got sha256:{actual_digest}"
            );
        }
    }
    Ok(())
}

pub(super) fn parse_sha256_digest(value: &str, label: &str) -> Result<String> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        bail!("{label} must use sha256:<hex> format");
    };
    if hex.len() != 64 || !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        bail!("{label} must use sha256:<64-hex> format");
    }
    Ok(hex.to_ascii_lowercase())
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

pub(super) fn verify_toolchain_shape(root: &Path, id: &ToolchainId) -> Result<ToolchainManifest> {
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

pub(super) fn verify_stdlib_manifest(root: &Path, metadata: &StdlibMetadata) -> Result<()> {
    let stdlib_path = validate_relative_toolchain_path(metadata.path(), "stdlib metadata path")?;
    let manifest = root.join(stdlib_path).join("ash.toml");
    if !manifest.is_file() {
        bail!("stdlib manifest is missing at {}", manifest.display());
    }
    Ok(())
}

pub(super) fn verify_runtime_support_payload(
    root: &Path,
    metadata: &RuntimeSupportMetadata,
) -> Result<()> {
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

pub(super) fn verify_required_manifest_tools(
    root: &Path,
    manifest: &ToolchainManifest,
) -> Result<()> {
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

pub(super) fn validate_relative_toolchain_path<'a>(path: &'a str, label: &str) -> Result<&'a Path> {
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
