use super::*;

pub(super) fn lock(project: &Path, check: bool) -> Result<()> {
    reject_legacy_conflict(project)?;
    let manifest = Manifest::read(project)?;
    let lock_path = project.join("ash.lock");
    let preserved_metadata = if lock_path.exists() {
        read_lock_reserved_metadata(&lock_path)?
    } else {
        LockReservedMetadata::default()
    };
    let preserved_any = preserved_metadata.has_reserved_metadata();
    let expected = manifest.lock_text(project, preserved_metadata)?;
    if check {
        let current = fs::read_to_string(&lock_path).context("read ash.lock")?;
        enforce_lock_signature_policy(&current)?;
        if normalize_ws(&current) != normalize_ws(&expected) {
            bail!("lockfile drift detected");
        }
        if preserved_any {
            println!("preserved trust metadata; lock signing policy enforced when required");
        }
        return Ok(());
    }
    fs::write(lock_path, expected).context("write ash.lock")?;
    if preserved_any {
        println!(
            "preserved trust metadata; lock signing policy preserved for subsequent enforcement"
        );
    }
    Ok(())
}

pub(super) fn fetch(paths: &AshgrovePaths, project: &Path) -> Result<()> {
    let lock_path = project.join("ash.lock");
    if !lock_path.exists() {
        lock(project, false)?;
    }
    let lock = read_lock(project)?;
    materialize_locked_packages(paths, project, &lock)
}

pub(super) fn vendor(
    paths: &AshgrovePaths,
    project: &Path,
    output: Option<&Path>,
    check: bool,
) -> Result<()> {
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
            package.git_url()?;
            provenance.git_url()?;
            let source = locked_package_root(paths, package)?;
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
        let source = locked_package_root(paths, package)?;
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

pub(super) fn read_lock(project: &Path) -> Result<LockFile> {
    let lock_path = project.join("ash.lock");
    let text = fs::read_to_string(&lock_path).context("read ash.lock")?;
    enforce_lock_signature_policy(&text)?;
    let lock: LockFile = toml::from_str(&text).context("parse ash.lock")?;
    lock.validate()?;
    Ok(lock)
}

fn enforce_lock_signature_policy(lock_text: &str) -> Result<()> {
    let value: toml::Value = toml::from_str(lock_text).context("parse ash.lock trust metadata")?;
    let Some(signing_lock) = value
        .get("signing")
        .and_then(toml::Value::as_table)
        .and_then(|signing| signing.get("lock"))
        .and_then(toml::Value::as_table)
    else {
        return Ok(());
    };
    if !signing_lock
        .get("required")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
    {
        return Ok(());
    }
    let expected = signing_lock
        .get("package_manifest_digest")
        .and_then(toml::Value::as_str)
        .context("required lock signature package_manifest_digest is missing")?;
    let expected = parse_sha256_digest(expected, "lock signature package_manifest_digest")?;
    let packages = value
        .get("package")
        .and_then(toml::Value::as_array)
        .context("required lock signature has no packages to verify")?;
    let matched = packages.iter().any(|package| {
        package
            .get("manifest_digest")
            .and_then(toml::Value::as_str)
            .and_then(|digest| digest.strip_prefix("sha256:"))
            .is_some_and(|digest| digest.eq_ignore_ascii_case(&expected))
    });
    if !matched {
        bail!("lock signature mismatch for required package manifest digest sha256:{expected}");
    }
    Ok(())
}

fn read_lock_reserved_metadata(lock_path: &Path) -> Result<LockReservedMetadata> {
    let text = fs::read_to_string(lock_path).context("read ash.lock")?;
    let value: toml::Value = toml::from_str(&text).context("parse ash.lock")?;
    Ok(LockReservedMetadata {
        trust: value.get("trust").cloned(),
        signing: value.get("signing").cloned(),
    })
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
    let git = package.git_url()?;
    let repo = locked_package_repo(paths, package)?;
    let checkout = locked_package_root(paths, package)?;
    if !repo.exists() {
        fs::create_dir_all(repo.parent().context("repo parent")?).context("create repo cache")?;
        run_git_command(
            Path::new("."),
            &["clone", "--mirror", git, repo_str(&repo)?],
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

pub(super) fn locked_package_repo(
    paths: &AshgrovePaths,
    package: &LockedPackage,
) -> Result<PathBuf> {
    let git = package.git_url()?;
    Ok(paths.cache_dir().join("git/repos").join(format!(
        "{}-{}.git",
        package.name,
        git_url_digest(git)
    )))
}

pub(super) fn locked_package_root(
    paths: &AshgrovePaths,
    package: &LockedPackage,
) -> Result<PathBuf> {
    let git = package.git_url()?;
    Ok(paths
        .cache_dir()
        .join("git/checkouts")
        .join(format!("{}-{}", package.name, git_url_digest(git)))
        .join(&package.commit))
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
pub(super) struct LockFile {
    #[serde(default = "lockfile_schema_version")]
    pub(super) version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) trust: Option<toml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) signing: Option<toml::Value>,
    #[serde(default)]
    pub(super) package: Vec<LockedPackage>,
}

#[derive(Debug, Default)]
struct LockReservedMetadata {
    trust: Option<toml::Value>,
    signing: Option<toml::Value>,
}

impl LockReservedMetadata {
    fn has_reserved_metadata(&self) -> bool {
        self.trust.is_some() || self.signing.is_some()
    }
}

const fn lockfile_schema_version() -> u32 {
    1
}

impl LockFile {
    fn validate(&self) -> Result<()> {
        for package in &self.package {
            package.git_url()?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(super) struct LockedPackage {
    pub(super) name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) git: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) package: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) registry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) rev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    requested: Option<RequestedPackage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolved: Option<ResolvedPackage>,
    pub(super) commit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) manifest_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) source_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) authenticated_origin: Option<String>,
}

impl LockedPackage {
    pub(super) fn git_url(&self) -> Result<&str> {
        match (self.source.as_deref(), self.git.as_deref()) {
            (Some(source), Some(git)) => {
                let source_git = source.strip_prefix("git+").ok_or_else(|| {
                    anyhow!(
                        "hosted registry dependencies are out of scope; ash.lock package source must be git+ URL"
                    )
                })?;
                if source_git != git {
                    bail!("ash.lock package source does not match legacy git URL");
                }
                validate_lock_git_url(source_git)?;
                Ok(source_git)
            }
            (Some(source), None) => {
                let source_git = source.strip_prefix("git+").ok_or_else(|| {
                    anyhow!(
                        "hosted registry dependencies are out of scope; ash.lock package source must be git+ URL"
                    )
                })?;
                validate_lock_git_url(source_git)?;
                Ok(source_git)
            }
            (None, Some(git)) => {
                validate_lock_git_url(git)?;
                Ok(git)
            }
            (None, None) => bail!("ash.lock package missing git"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
struct RequestedPackage {
    #[serde(skip_serializing_if = "Option::is_none")]
    tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
struct ResolvedPackage {
    rev: String,
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

    fn lock_text(&self, project: &Path, reserved: LockReservedMetadata) -> Result<String> {
        let mut package = Vec::with_capacity(self.dependencies.len());
        for dep in &self.dependencies {
            let commit = dep.resolve_commit(project)?;
            let origin = canonical_git_url_for_lock(&dep.git)?;
            let rev = dep.rev.as_ref().map(|_| commit.clone());
            package.push(LockedPackage {
                name: dep.name.clone(),
                git: Some(origin.url.clone()),
                source: Some(format!("git+{}", origin.url)),
                package: dep.package.clone(),
                version: dep.version.clone(),
                registry: dep.registry.clone(),
                kind: dep.kind.clone(),
                license: dep.license.clone(),
                tag: dep.tag.clone(),
                rev,
                requested: Some(RequestedPackage {
                    tag: dep.tag.clone(),
                    rev: dep.rev.clone(),
                }),
                resolved: Some(ResolvedPackage {
                    rev: commit.clone(),
                }),
                commit,
                manifest_digest: dep.registry_metadata_digest()?,
                source_path: dep.local_path().map(|path| path.display().to_string()),
                authenticated_origin: origin.authenticated_origin,
            });
        }
        toml::to_string(&LockFile {
            version: lockfile_schema_version(),
            trust: reserved.trust,
            signing: reserved.signing,
            package,
        })
        .context("serialize ash.lock")
    }
}

#[derive(Debug)]
struct Dependency {
    name: String,
    git: String,
    package: Option<String>,
    version: Option<String>,
    registry: Option<String>,
    kind: Option<String>,
    license: Option<String>,
    tag: Option<String>,
    rev: Option<String>,
}

impl Dependency {
    fn from_value(name: &str, value: &toml::Value) -> Result<Self> {
        validate_package_name(name)?;
        let package = optional_dependency_string(value, "package")?;
        let version = optional_dependency_string(value, "version")?;
        let registry = optional_dependency_string(value, "registry")?;
        let kind = optional_dependency_string(value, "kind")?;
        let license = optional_dependency_string(value, "license")?;
        let git = match value.get("git").and_then(toml::Value::as_str) {
            Some(git) => {
                validate_git_protocol(git)?;
                git.to_string()
            }
            None if package.is_some() || version.is_some() || registry.is_some() => {
                bail!(
                    "hosted registry dependencies are not supported and remain out of scope; dependency '{name}' must use explicit git plus tag or rev"
                );
            }
            None => bail!("git dependency missing git URL"),
        };
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
            package,
            version,
            registry,
            kind,
            license,
            tag,
            rev,
        })
    }

    fn resolve_commit(&self, project: &Path) -> Result<String> {
        let reference = self.rev.as_ref().or(self.tag.as_ref()).expect("validated");
        if self.tag.is_none() && is_full_git_commit(reference) && self.local_path().is_none() {
            return Ok(reference.to_string());
        }
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

    fn registry_metadata_digest(&self) -> Result<Option<String>> {
        if self.package.is_none()
            && self.version.is_none()
            && self.registry.is_none()
            && self.kind.is_none()
            && self.license.is_none()
        {
            return Ok(None);
        }
        let mut table = toml::map::Map::new();
        insert_optional_metadata(&mut table, "package", &self.package);
        insert_optional_metadata(&mut table, "version", &self.version);
        insert_optional_metadata(&mut table, "registry", &self.registry);
        insert_optional_metadata(&mut table, "kind", &self.kind);
        insert_optional_metadata(&mut table, "license", &self.license);
        let text = toml::to_string(&toml::Value::Table(table))
            .context("serialize registry metadata for digest")?;
        Ok(Some(format!("sha256:{}", sha256_hex(text.as_bytes()))))
    }
}

#[derive(Debug)]
pub(super) struct CanonicalGitOrigin {
    pub(super) url: String,
    pub(super) authenticated_origin: Option<String>,
}

pub(super) fn canonical_git_url_for_lock(raw: &str) -> Result<CanonicalGitOrigin> {
    validate_git_protocol(raw)?;
    if let Some(rest) = raw.strip_prefix("https://") {
        let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
        if authority.contains('@') {
            if authority.matches('@').count() != 1 {
                bail!(
                    "ambiguous authenticated git URL rejected before lock serialization; credentials must be percent-encoded or removed"
                );
            }
            let (userinfo, host) = authority
                .split_once('@')
                .context("authenticated git URL missing host delimiter")?;
            if userinfo.is_empty() {
                bail!("authenticated git URL missing credentials");
            }
            if host.is_empty() {
                bail!("authenticated git URL missing host");
            }
            let sanitized = if path.is_empty() {
                format!("https://{host}")
            } else {
                format!("https://{host}/{path}")
            };
            validate_git_protocol(&sanitized)?;
            return Ok(CanonicalGitOrigin {
                url: sanitized,
                authenticated_origin: Some("credentials-redacted".to_string()),
            });
        }
    }
    if credential_bearing_ssh_userinfo(raw).is_some() {
        bail!("credentials-bearing ssh git URL is rejected before lock serialization");
    }
    Ok(CanonicalGitOrigin {
        url: raw.to_string(),
        authenticated_origin: None,
    })
}

fn validate_lock_git_url(url: &str) -> Result<()> {
    validate_git_protocol(url)?;
    if git_url_contains_credentials(url) {
        bail!("ash.lock git URL must not contain credentials or secrets");
    }
    Ok(())
}

pub(super) fn validate_git_protocol(url: &str) -> Result<()> {
    if url.starts_with("file://")
        || url.starts_with("https://")
        || url.starts_with("ssh://")
        || is_scp_like_ssh_url(url)
    {
        return Ok(());
    }
    let scheme = url
        .split_once("://")
        .map_or("unknown", |(scheme, _)| scheme);
    bail!("untrusted git protocol '{scheme}' rejected before fetch");
}

fn git_url_contains_credentials(url: &str) -> bool {
    https_url_userinfo(url).is_some_and(|userinfo| !userinfo.is_empty())
        || credential_bearing_ssh_userinfo(url).is_some()
}

fn https_url_userinfo(url: &str) -> Option<&str> {
    let rest = url.strip_prefix("https://")?;
    let (authority, _) = rest.split_once('/').unwrap_or((rest, ""));
    authority.split_once('@').map(|(userinfo, _)| userinfo)
}

fn credential_bearing_ssh_userinfo(url: &str) -> Option<&str> {
    let rest = url.strip_prefix("ssh://")?;
    let (authority, _) = rest.split_once('/').unwrap_or((rest, ""));
    let (userinfo, host) = authority.split_once('@')?;
    if host.is_empty() || userinfo.is_empty() || !userinfo.contains(':') {
        return None;
    }
    Some(userinfo)
}

fn is_scp_like_ssh_url(url: &str) -> bool {
    let Some((user_host, path)) = url.split_once(':') else {
        return false;
    };
    !path.is_empty()
        && !user_host.contains('/')
        && !user_host.is_empty()
        && user_host.contains('@')
        && !user_host.contains("://")
}

fn is_full_git_commit(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn optional_dependency_string(value: &toml::Value, key: &str) -> Result<Option<String>> {
    value
        .get(key)
        .map(|raw| {
            raw.as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| anyhow!("dependency metadata field '{key}' must be a string"))
        })
        .transpose()
}

fn insert_optional_metadata(
    table: &mut toml::map::Map<String, toml::Value>,
    key: &str,
    value: &Option<String>,
) {
    if let Some(value) = value {
        table.insert(key.to_string(), toml::Value::String(value.clone()));
    }
}

pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::new();
    for byte in digest {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

pub(super) fn validate_package_name(name: &str) -> Result<()> {
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

pub(super) fn validate_commit(commit: &str) -> Result<()> {
    if commit.len() != 40 || !commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("locked git commit must be a full 40-character commit hash");
    }
    Ok(())
}

pub(super) fn file_digest(path: &Path) -> Result<String> {
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

pub(super) fn normalize_ws(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
