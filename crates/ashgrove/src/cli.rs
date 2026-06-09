use super::*;

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
    release_index: Option<PathBuf>,
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
    release_index: Option<PathBuf>,
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
pub struct CleanupArgs {
    #[arg(long)]
    pub project: Option<PathBuf>,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub cache: bool,
    #[arg(long)]
    pub orphans: bool,
    #[arg(long)]
    pub old_toolchains: bool,
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
            if let Some(release_index) = args.release_index.as_deref() {
                verify_signed_release_index(release_index)?;
            }
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
            if let Some(release_index) = args.release_index.as_deref() {
                verify_signed_release_index(release_index)?;
            }
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
