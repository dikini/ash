use super::*;

pub fn read_selector(paths: &AshgrovePaths) -> Result<SelectorMetadata> {
    let path = paths.config_dir().join("toolchains.toml");
    if !path.exists() {
        return Ok(SelectorMetadata::empty());
    }
    SelectorMetadata::read_from_path(&path)
}

pub fn read_default(paths: &AshgrovePaths) -> Result<Option<ToolchainId>> {
    Ok(read_selector(paths)?.default().cloned())
}

pub fn list_toolchains(paths: &AshgrovePaths) -> Result<()> {
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

pub fn current(paths: &AshgrovePaths, project: Option<&Path>) -> Result<()> {
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

pub fn installed_toolchain_ids(paths: &AshgrovePaths) -> Result<Vec<ToolchainId>> {
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

pub fn installed_toolchain_manifest(
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

pub fn verify_install_record_any_source(root: &Path, id: &ToolchainId) -> Result<InstallRecord> {
    let record_text =
        fs::read_to_string(root.join("install-record.toml")).context("read install record")?;
    let record = InstallRecord::from_toml_str(&record_text)?;
    record.validate_for_toolchain(id)?;
    Ok(record)
}

pub fn require_installed_toolchain(paths: &AshgrovePaths, id: &ToolchainId) -> Result<()> {
    if !paths.toolchain_dir(id).is_dir() {
        bail!("toolchain '{}' is not installed", id.as_str());
    }
    installed_toolchain_manifest(paths, id)?;
    verify_install_record_any_source(&paths.toolchain_dir(id), id)?;
    Ok(())
}

pub fn project_toolchain_pin(project: &Path) -> Result<Option<String>> {
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

pub fn remove_toolchain(paths: &AshgrovePaths, id: &ToolchainId, force: bool) -> Result<()> {
    if std::env::var("ASHGROVE_RUNNING_TOOLCHAIN").ok().as_deref() == Some(id.as_str()) {
        bail!(
            "refusing to remove running manager toolchain '{}'",
            id.as_str()
        );
    }
    if stable_dispatcher_manager_toolchain(paths)?.as_ref() == Some(id) {
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

pub fn cleanup(paths: &AshgrovePaths, args: CleanupArgs) -> Result<()> {
    let has_cleanup_flags = args.cache || args.orphans || args.old_toolchains;
    let reachability = CleanupReachability::collect(paths, args.project.as_deref())?;
    let project_pin = args
        .project
        .as_deref()
        .and_then(|project| reachability.project_toolchain(project));

    if args.dry_run && args.project.is_some() && !has_cleanup_flags {
        cleanup_project_dry_run_plan(project_pin);
        return Ok(());
    }

    if args.old_toolchains && !args.dry_run {
        cleanup_old_toolchains(paths, &reachability, args.dry_run)?;
    }
    if args.cache {
        cleanup_cache(paths, &reachability, args.dry_run)?;
    }
    if args.orphans {
        cleanup_orphan_toolchain_dirs(paths, args.dry_run)?;
    }
    if args.old_toolchains && args.dry_run {
        cleanup_old_toolchains(paths, &reachability, args.dry_run)?;
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

#[derive(Debug, Default)]
struct CleanupReachability {
    project_toolchains: BTreeMap<PathBuf, ToolchainId>,
    package_roots: BTreeSet<PathBuf>,
    package_repos: BTreeSet<PathBuf>,
}

impl CleanupReachability {
    fn collect(paths: &AshgrovePaths, supplied_project: Option<&Path>) -> Result<Self> {
        let mut reachability = Self::default();
        let mut projects = BTreeSet::new();
        if let Some(project) = supplied_project {
            projects.insert(project.to_path_buf());
        }
        projects.insert(std::env::current_dir().context("read current directory")?);
        let selector = read_selector(paths)?;
        projects.extend(selector.project_roots());

        for project in projects {
            if !project.exists() {
                continue;
            }
            if let Some(pin) = project_toolchain_pin(&project)? {
                reachability
                    .project_toolchains
                    .insert(project.clone(), ToolchainId::parse(&pin)?);
            }
            if project.join("ash.lock").is_file() {
                let lock = read_lock(&project)
                    .with_context(|| format!("read cleanup lockfile {}", project.display()))?;
                reachability.record_lock(paths, &lock)?;
            }
            reachability.record_vendor_provenance(paths, &project)?;
        }
        Ok(reachability)
    }

    fn record_lock(&mut self, paths: &AshgrovePaths, lock: &LockFile) -> Result<()> {
        for package in &lock.package {
            validate_package_name(&package.name)?;
            validate_commit(&package.commit)?;
            self.package_roots
                .insert(locked_package_root(paths, package)?);
            self.package_repos
                .insert(locked_package_repo(paths, package)?);
        }
        Ok(())
    }

    fn record_vendor_provenance(&mut self, paths: &AshgrovePaths, project: &Path) -> Result<()> {
        let vendor_root = project.join("vendor/ash");
        if !vendor_root.is_dir() {
            return Ok(());
        }
        for entry in fs::read_dir(&vendor_root)
            .with_context(|| format!("read vendor root {}", vendor_root.display()))?
        {
            let entry = entry.context("read vendor package entry")?;
            if !entry
                .file_type()
                .with_context(|| format!("read vendor package type {}", entry.path().display()))?
                .is_dir()
            {
                continue;
            }
            let provenance_path = entry.path().join("provenance.toml");
            if !provenance_path.is_file() {
                continue;
            }
            let text = fs::read_to_string(&provenance_path)
                .with_context(|| format!("read vendor provenance {}", provenance_path.display()))?;
            let package: LockedPackage = toml::from_str(&text).with_context(|| {
                format!("parse vendor provenance {}", provenance_path.display())
            })?;
            package.git_url()?;
            validate_package_name(&package.name)?;
            validate_commit(&package.commit)?;
            self.package_roots
                .insert(locked_package_root(paths, &package)?);
            self.package_repos
                .insert(locked_package_repo(paths, &package)?);
        }
        Ok(())
    }

    fn project_toolchain(&self, project: &Path) -> Option<&ToolchainId> {
        self.project_toolchains.get(project)
    }

    fn protects_toolchain(&self, id: &ToolchainId) -> bool {
        self.project_toolchains.values().any(|pin| pin == id)
    }

    fn is_reachable_cache_path(&self, path: &Path) -> bool {
        self.package_roots.contains(path) || self.package_repos.contains(path)
    }
}

fn cleanup_cache(
    paths: &AshgrovePaths,
    reachability: &CleanupReachability,
    dry_run: bool,
) -> Result<()> {
    let cache_dir = paths.cache_dir();
    if !cache_dir.exists() {
        return Ok(());
    }
    cleanup_cache_tree(&cache_dir.join("git/repos"), reachability, dry_run)?;
    cleanup_cache_tree(&cache_dir.join("git/checkouts"), reachability, dry_run)?;
    for name in ["downloads", "builds", "module-cache"] {
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

fn cleanup_cache_tree(
    root: &Path,
    reachability: &CleanupReachability,
    dry_run: bool,
) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root).with_context(|| format!("read cache {}", root.display()))? {
        let entry = entry.context("read cache entry")?;
        let path = entry.path();
        if root.ends_with("checkouts") && entry.file_type()?.is_dir() {
            for checkout in fs::read_dir(&path)
                .with_context(|| format!("read cache checkout namespace {}", path.display()))?
            {
                let checkout = checkout.context("read cache checkout entry")?;
                report_or_remove_cache_entry(&checkout.path(), reachability, dry_run)?;
            }
            if !dry_run {
                remove_empty_dir(&path)?;
            }
        } else {
            report_or_remove_cache_entry(&path, reachability, dry_run)?;
        }
    }
    Ok(())
}

fn report_or_remove_cache_entry(
    path: &Path,
    reachability: &CleanupReachability,
    dry_run: bool,
) -> Result<()> {
    if reachability.is_reachable_cache_path(path) {
        println!("reachable cache {}", path.display());
        return Ok(());
    }
    if dry_run {
        println!("would remove cache {}", path.display());
    } else {
        remove_path(path).with_context(|| format!("remove cache {}", path.display()))?;
        println!("removed cache {}", path.display());
    }
    Ok(())
}

fn remove_empty_dir(path: &Path) -> Result<()> {
    if path.is_dir()
        && fs::read_dir(path)
            .with_context(|| format!("read cache dir {}", path.display()))?
            .next()
            .is_none()
    {
        fs::remove_dir(path)
            .with_context(|| format!("remove empty cache dir {}", path.display()))?;
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
    reachability: &CleanupReachability,
    dry_run: bool,
) -> Result<()> {
    let mut ids = installed_toolchain_ids(paths)?;
    ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    let mut deletion_candidates = Vec::new();
    for id in ids {
        if let Some(reason) = cleanup_protection_reason(paths, &id, reachability)? {
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
    reachability: &CleanupReachability,
) -> Result<Option<&'static str>> {
    if std::env::var("ASHGROVE_RUNNING_TOOLCHAIN").ok().as_deref() == Some(id.as_str()) {
        return Ok(Some("running manager"));
    }
    if stable_dispatcher_manager_toolchain(paths)?.as_ref() == Some(id) {
        return Ok(Some("running manager"));
    }
    if live_daemon_uses_toolchain(paths, id)? {
        return Ok(Some("live daemon"));
    }
    if read_default(paths)?.as_ref() == Some(id) {
        return Ok(Some("default"));
    }
    if reachability.protects_toolchain(id) || current_project_uses_toolchain(id)? {
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
