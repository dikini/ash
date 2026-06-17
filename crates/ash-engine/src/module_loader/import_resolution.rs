//! Import path, dependency-root, lockfile, and stdlib-root resolution for module loading.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::MODULE_ROOT_OVERRIDE;
use crate::error::EngineError;

/// Resolve a module path from segments and search roots.
///
/// # Errors
/// Returns an error if a locked vendor root or package root check fails.
pub fn resolve_module_path(
    module_segments: &[String],
    search_roots: &[SearchRoot],
) -> Result<Option<PathBuf>, EngineError> {
    for root in search_roots {
        if is_locked_vendor_root(&root.path)
            && !locked_vendor_root_allows(&root.path, module_segments)?
        {
            continue;
        }
        if is_locked_vendor_package_root(&root.path) {
            if !locked_vendor_package_root_allows(&root.path, module_segments)? {
                continue;
            }
            if let Some(package_relative_segments) = module_segments.get(1..)
                && let Some(path) = resolve_in_root(root.path.as_path(), package_relative_segments)
            {
                return Ok(Some(path));
            }
            continue;
        }
        if root.kind == SearchRootKind::LockedCache
            && let Some(package_name) = locked_cache_package_name(&root.path)
        {
            if module_segments.first().map(String::as_str) != Some(package_name.as_str()) {
                continue;
            }
            if let Some(package_relative_segments) = module_segments.get(1..)
                && let Some(path) = resolve_in_root(root.path.as_path(), package_relative_segments)
            {
                return Ok(Some(path));
            }
            continue;
        }
        if let Some(path) = resolve_in_root(root.path.as_path(), module_segments) {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

/// A search root for module resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchRoot {
    /// The path to the search root.
    pub path: PathBuf,
    /// The kind of search root.
    pub kind: SearchRootKind,
}

impl SearchRoot {
    /// Create an ordinary search root.
    #[must_use]
    pub const fn ordinary(path: PathBuf) -> Self {
        Self {
            path,
            kind: SearchRootKind::Ordinary,
        }
    }

    /// Create a locked cache search root.
    #[must_use]
    pub const fn locked_cache(path: PathBuf) -> Self {
        Self {
            path,
            kind: SearchRootKind::LockedCache,
        }
    }
}

/// The kind of search root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchRootKind {
    /// An ordinary search root.
    Ordinary,
    /// A locked cache search root.
    LockedCache,
}

fn is_locked_vendor_root(root: &Path) -> bool {
    root.file_name().and_then(|name| name.to_str()) == Some("ash")
        && root
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            == Some("vendor")
}

fn is_locked_vendor_package_root(root: &Path) -> bool {
    root.parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        == Some("ash")
        && root
            .parent()
            .and_then(Path::parent)
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            == Some("vendor")
}

fn locked_cache_package_name(root: &Path) -> Option<String> {
    root.parent()
        .and_then(Path::parent)
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .filter(|name| *name == "checkouts")?;
    let package_digest = root.parent()?.file_name()?.to_str()?;
    let (package_name, digest) = package_digest.rsplit_once('-')?;
    if digest.len() == 16 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Some(package_name.to_string())
    } else {
        None
    }
}

fn locked_vendor_package_root_allows(
    root: &Path,
    module_segments: &[String],
) -> Result<bool, EngineError> {
    let Some(first) = module_segments.first() else {
        return Ok(false);
    };
    let Some(package_name) = root.file_name().and_then(|name| name.to_str()) else {
        return Ok(false);
    };
    if first != package_name {
        return Ok(false);
    }
    let Some(project_root) = root.parent().and_then(Path::parent).and_then(Path::parent) else {
        return Ok(false);
    };
    locked_project_allows_package(project_root, first)
}

fn locked_vendor_root_allows(root: &Path, module_segments: &[String]) -> Result<bool, EngineError> {
    let Some(first) = module_segments.first() else {
        return Ok(false);
    };
    let Some(project_root) = root.parent().and_then(Path::parent) else {
        return Ok(false);
    };
    locked_project_allows_package(project_root, first)
}

fn locked_project_allows_package(
    project_root: &Path,
    package_name: &str,
) -> Result<bool, EngineError> {
    let lock = read_project_lock(project_root)?;
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| EngineError::Configuration("ash.lock missing package entries".into()))?;

    let mut allowed = false;
    for package in packages {
        let name = locked_package_name(package)?;
        let _git = locked_package_git(package)?;
        let _commit = locked_package_commit(package)?;
        if name == package_name {
            allowed = true;
        }
    }
    Ok(allowed)
}

fn normalize_import_resolution(
    module_segments: &[String],
    importing_dir: &Path,
    crate_root: Option<&Path>,
    absolute_roots: &[SearchRoot],
) -> (Vec<String>, Vec<SearchRoot>) {
    let Some(first) = module_segments.first().map(String::as_str) else {
        return (Vec::new(), absolute_roots.to_vec());
    };

    match first {
        "self" => (
            module_segments[1..].to_vec(),
            vec![SearchRoot::ordinary(importing_dir.to_path_buf())],
        ),
        "super" => {
            let mut root = importing_dir.to_path_buf();
            let mut roots = vec![SearchRoot::ordinary(root.clone())];
            let mut index = 0usize;
            while module_segments
                .get(index)
                .is_some_and(|segment| segment == "super")
            {
                root.pop();
                roots.push(SearchRoot::ordinary(root.clone()));
                index += 1;
            }
            (module_segments[index..].to_vec(), roots)
        }
        "crate" => (
            module_segments[1..].to_vec(),
            crate_import_roots(importing_dir, crate_root),
        ),
        _ => (module_segments.to_vec(), absolute_roots.to_vec()),
    }
}

/// Compute import resolution roots for a module import.
///
/// # Errors
/// Returns an error if the module segments are empty or invalid.
pub fn import_resolution_roots(
    module_segments: &[String],
    importing_dir: &Path,
    crate_root: Option<&Path>,
) -> Result<(Vec<String>, Vec<SearchRoot>), EngineError> {
    let absolute_roots = if import_uses_local_roots(module_segments) {
        Vec::new()
    } else {
        search_roots(importing_dir)?
    };
    Ok(normalize_import_resolution(
        module_segments,
        importing_dir,
        crate_root,
        &absolute_roots,
    ))
}

fn import_uses_local_roots(module_segments: &[String]) -> bool {
    matches!(
        module_segments.first().map(String::as_str),
        Some("self" | "super" | "crate")
    )
}

fn crate_import_roots(importing_dir: &Path, crate_root: Option<&Path>) -> Vec<SearchRoot> {
    let mut roots = Vec::new();
    let mut current = Some(importing_dir);
    while let Some(path) = current {
        roots.push(SearchRoot::ordinary(path.to_path_buf()));
        current = path.parent();
    }

    match crate_root {
        Some(root) if !roots.iter().any(|candidate| candidate.path == root) => {
            roots.push(SearchRoot::ordinary(root.to_path_buf()));
        }
        _ => {}
    }

    roots
}

/// Discover the crate root for a given importing directory.
#[must_use]
pub fn discover_crate_root(importing_dir: &Path) -> Option<PathBuf> {
    let mut current = importing_dir;
    let mut best = None;

    loop {
        if is_ash_module_root(current, importing_dir) {
            best = Some(current.to_path_buf());
        }

        let Some(parent) = current.parent() else {
            break;
        };
        current = parent;
    }

    best.or_else(|| fallback_std_module_root(importing_dir))
}

fn fallback_std_module_root(importing_dir: &Path) -> Option<PathBuf> {
    let std_root = builtin_stdlib_root().canonicalize().ok()?;
    let importing_dir = importing_dir.canonicalize().ok()?;
    if importing_dir.starts_with(&std_root) {
        Some(std_root)
    } else {
        None
    }
}

fn is_ash_module_root(path: &Path, importing_dir: &Path) -> bool {
    if path.join("mod.ash").is_file() {
        return true;
    }

    path != importing_dir && contains_ash_files(path)
}

fn contains_ash_files(path: &Path) -> bool {
    std::fs::read_dir(path).is_ok_and(|entries| {
        entries.flatten().any(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "ash")
        })
    })
}

pub(super) fn search_roots(root: &Path) -> Result<Vec<SearchRoot>, EngineError> {
    let mut roots = vec![SearchRoot::ordinary(root.to_path_buf())];
    MODULE_ROOT_OVERRIDE.with(|slot| {
        if let Some(override_roots) = slot.borrow().as_ref() {
            roots.extend(
                override_roots
                    .dependency_roots
                    .iter()
                    .cloned()
                    .map(SearchRoot::ordinary),
            );
        }
    });
    if let Some(value) = std::env::var_os("ASH_DEP_ROOTS") {
        roots.extend(std::env::split_paths(&value).map(SearchRoot::ordinary));
    }
    if let Some(value) = std::env::var_os("ASH_DEPENDENCY_ROOTS") {
        roots.extend(std::env::split_paths(&value).map(SearchRoot::ordinary));
    }
    if let Some(value) = std::env::var_os("ASH_LIBRARY_PATH") {
        roots.extend(std::env::split_paths(&value).map(SearchRoot::ordinary));
    }
    roots.push(SearchRoot::ordinary(builtin_stdlib_root()));
    roots.extend(discover_locked_project_roots(root)?);
    Ok(roots)
}

fn discover_locked_project_roots(importing_dir: &Path) -> Result<Vec<SearchRoot>, EngineError> {
    let Some(project_root) = discover_ash_project_root(importing_dir) else {
        return Ok(Vec::new());
    };

    let vendor_root = project_root.join("vendor/ash");
    if !vendor_root.is_dir() && !project_root.join("ash.lock").is_file() {
        return Ok(Vec::new());
    }

    let lock = read_project_lock(&project_root)?;
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| EngineError::Configuration("ash.lock missing package entries".into()))?;

    if vendor_root.is_dir() {
        discover_locked_vendor_roots(&vendor_root, packages)
    } else {
        discover_locked_cache_roots(packages)
    }
}

fn read_project_lock(project_root: &Path) -> Result<toml::Value, EngineError> {
    let lock_text = std::fs::read_to_string(project_root.join("ash.lock")).map_err(|error| {
        EngineError::Configuration(format!(
            "failed to read ash.lock for project '{}': {error}",
            project_root.display()
        ))
    })?;
    enforce_lock_signature_policy(&lock_text)?;
    toml::from_str(&lock_text).map_err(|error| {
        EngineError::Configuration(format!(
            "failed to parse ash.lock for project '{}': {error}",
            project_root.display()
        ))
    })
}

fn discover_locked_vendor_roots(
    vendor_root: &Path,
    packages: &[toml::Value],
) -> Result<Vec<SearchRoot>, EngineError> {
    let mut roots = Vec::with_capacity(packages.len() + 1);
    roots.push(SearchRoot::ordinary(vendor_root.to_path_buf()));
    for package in packages {
        let name = locked_package_name(package)?;
        let _git = locked_package_git(package)?;
        let _commit = locked_package_commit(package)?;
        let package_root = vendor_root.join(name);
        if !package_root.is_dir() {
            return Err(EngineError::Configuration(format!(
                "locked package '{name}' is missing from vendor root '{}'",
                vendor_root.display()
            )));
        }
    }
    Ok(roots)
}

fn discover_locked_cache_roots(packages: &[toml::Value]) -> Result<Vec<SearchRoot>, EngineError> {
    let mut roots = Vec::with_capacity(packages.len());
    let cache_home = xdg_cache_home()?;
    for package in packages {
        let name = locked_package_name(package)?;
        let git = locked_package_git(package)?;
        let commit = locked_package_commit(package)?;
        let package_root = cache_home
            .join("ash/git/checkouts")
            .join(format!("{}-{}", name, git_url_digest(git)))
            .join(commit);
        if !package_root.is_dir() {
            return Err(EngineError::Configuration(format!(
                "locked package '{name}' is missing from fetched cache '{}'",
                package_root.display()
            )));
        }
        ensure_fetched_checkout_commit(&package_root, commit)?;
        roots.push(SearchRoot::locked_cache(package_root));
    }
    Ok(roots)
}

fn ensure_fetched_checkout_commit(checkout: &Path, commit: &str) -> Result<(), EngineError> {
    if !checkout.join(".git").exists() {
        return Err(EngineError::Configuration(format!(
            "locked package fetched cache '{}' is not a git checkout",
            checkout.display()
        )));
    }
    let output = std::process::Command::new("git")
        .args(["-C"])
        .arg(checkout)
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|error| {
            EngineError::Configuration(format!(
                "failed to inspect fetched cache checkout '{}': {error}",
                checkout.display()
            ))
        })?;
    if !output.status.success() {
        return Err(EngineError::Configuration(format!(
            "failed to inspect fetched cache checkout '{}': {}",
            checkout.display(),
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    let head = String::from_utf8(output.stdout).map_err(|error| {
        EngineError::Configuration(format!(
            "fetched cache checkout '{}' reported non-utf8 HEAD: {error}",
            checkout.display()
        ))
    })?;
    if head.trim() != commit {
        return Err(EngineError::Configuration(format!(
            "fetched cache checkout '{}' is not at locked commit {commit}",
            checkout.display()
        )));
    }
    Ok(())
}

fn locked_package_name(package: &toml::Value) -> Result<&str, EngineError> {
    let name = package
        .get("name")
        .and_then(toml::Value::as_str)
        .ok_or_else(|| EngineError::Configuration("ash.lock package missing name".into()))?;
    validate_locked_package_name(name)?;
    Ok(name)
}

fn locked_package_git(package: &toml::Value) -> Result<&str, EngineError> {
    let git = package.get("git").and_then(toml::Value::as_str);
    let source = package.get("source").and_then(toml::Value::as_str);
    match (source, git) {
        (Some(source), Some(git)) => {
            let source_git = locked_package_source_git(source)?;
            if source_git != git {
                return Err(EngineError::Configuration(
                    "ash.lock package source does not match legacy git URL".into(),
                ));
            }
            validate_locked_git_url(source_git)?;
            Ok(source_git)
        }
        (Some(source), None) => {
            let git = locked_package_source_git(source)?;
            validate_locked_git_url(git)?;
            Ok(git)
        }
        (None, Some(git)) => {
            validate_locked_git_url(git)?;
            Ok(git)
        }
        (None, None) => Err(EngineError::Configuration(
            "ash.lock package missing git".into(),
        )),
    }
}

fn locked_package_source_git(source: &str) -> Result<&str, EngineError> {
    source.strip_prefix("git+").ok_or_else(|| {
        EngineError::Configuration(
            "hosted registry dependencies are out of scope; ash.lock package source must be git+ URL"
                .into(),
        )
    })
}

fn locked_package_commit(package: &toml::Value) -> Result<&str, EngineError> {
    let commit = if let Some(commit) = package.get("commit").and_then(toml::Value::as_str) {
        commit
    } else {
        package
            .get("resolved")
            .and_then(toml::Value::as_table)
            .and_then(|resolved| resolved.get("rev"))
            .and_then(toml::Value::as_str)
            .ok_or_else(|| EngineError::Configuration("ash.lock package missing commit".into()))?
    };
    validate_locked_commit(commit)?;
    Ok(commit)
}

fn enforce_lock_signature_policy(lock_text: &str) -> Result<(), EngineError> {
    let value: toml::Value = toml::from_str(lock_text).map_err(|error| {
        EngineError::Configuration(format!("failed to parse ash.lock trust metadata: {error}"))
    })?;
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
        .ok_or_else(|| {
            EngineError::Configuration(
                "required lock signature package_manifest_digest is missing".into(),
            )
        })
        .and_then(|digest| parse_sha256_digest(digest, "lock signature package_manifest_digest"))?;
    let packages = value
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| {
            EngineError::Configuration("required lock signature has no packages to verify".into())
        })?;
    let matched = packages.iter().any(|package| {
        package
            .get("manifest_digest")
            .and_then(toml::Value::as_str)
            .and_then(|digest| digest.strip_prefix("sha256:"))
            .is_some_and(|digest| digest.eq_ignore_ascii_case(&expected))
    });
    if !matched {
        return Err(EngineError::Configuration(format!(
            "lock signature mismatch for required package manifest digest sha256:{expected}"
        )));
    }
    Ok(())
}

fn parse_sha256_digest(value: &str, label: &str) -> Result<String, EngineError> {
    let digest = value.strip_prefix("sha256:").ok_or_else(|| {
        EngineError::Configuration(format!("{label} must use sha256:<64-hex> format"))
    })?;
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(EngineError::Configuration(format!(
            "{label} must use sha256:<64-hex> format"
        )));
    }
    Ok(digest.to_ascii_lowercase())
}

fn validate_locked_git_url(url: &str) -> Result<(), EngineError> {
    validate_locked_git_protocol(url)?;
    if locked_git_url_contains_credentials(url) {
        return Err(EngineError::Configuration(
            "ash.lock git URL must not contain credentials or secrets".into(),
        ));
    }
    Ok(())
}

fn validate_locked_git_protocol(url: &str) -> Result<(), EngineError> {
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
    Err(EngineError::Configuration(format!(
        "untrusted git protocol '{scheme}' rejected before lock use"
    )))
}

fn locked_git_url_contains_credentials(url: &str) -> bool {
    https_locked_url_userinfo(url).is_some_and(|userinfo| !userinfo.is_empty())
        || credential_bearing_ssh_userinfo(url).is_some()
}

fn https_locked_url_userinfo(url: &str) -> Option<&str> {
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

fn xdg_cache_home() -> Result<PathBuf, EngineError> {
    if let Some(root) = std::env::var_os("XDG_CACHE_HOME") {
        return Ok(PathBuf::from(root));
    }
    let home = std::env::var_os("HOME").ok_or_else(|| {
        EngineError::Configuration("HOME is required for XDG cache lookup".into())
    })?;
    Ok(PathBuf::from(home).join(".cache"))
}

fn discover_ash_project_root(importing_dir: &Path) -> Option<PathBuf> {
    let mut current = Some(importing_dir);
    while let Some(path) = current {
        if path.join("ash.toml").is_file() {
            return Some(path.to_path_buf());
        }
        current = path.parent();
    }
    None
}

fn validate_locked_package_name(name: &str) -> Result<(), EngineError> {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return Err(EngineError::Configuration("invalid package name ''".into()));
    };
    if !first.is_ascii_alphanumeric()
        || !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err(EngineError::Configuration(format!(
            "invalid package name '{name}'"
        )));
    }
    Ok(())
}

fn validate_locked_commit(commit: &str) -> Result<(), EngineError> {
    if commit.len() == 40 && commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(EngineError::Configuration(
            "locked git commit must be a full 40-character commit hash".into(),
        ))
    }
}

fn git_url_digest(url: &str) -> String {
    let digest = Sha256::digest(url.as_bytes());
    let mut out = String::new();
    for byte in &digest[..8] {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

fn resolve_in_root(root: &Path, module_segments: &[String]) -> Option<PathBuf> {
    let joined = module_segments
        .iter()
        .fold(root.to_path_buf(), |mut path, segment| {
            path.push(segment);
            path
        });

    let file_candidate = joined.with_extension("ash");
    if file_candidate.is_file() {
        return Some(file_candidate);
    }

    let mod_candidate = joined.join("mod.ash");
    if mod_candidate.is_file() {
        return Some(mod_candidate);
    }

    None
}

pub(super) fn builtin_stdlib_root() -> PathBuf {
    if let Some(root) = MODULE_ROOT_OVERRIDE.with(|slot| {
        slot.borrow()
            .as_ref()
            .and_then(|override_roots| override_roots.stdlib_root.clone())
    }) {
        return root;
    }
    if let Some(root) = std::env::var_os("ASH_STDLIB_ROOT") {
        return PathBuf::from(root);
    }
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../std/src")
}
