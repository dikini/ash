//! Standard provider/admission profiles for productive Ash applications.
//!
//! Profiles are runtime configuration bundles. They register providers, sandbox policies, and
//! explicit capability bindings, but the profile identity itself never grants authority.

use crate::providers::{
    FsConfig, FsProvider, HttpConfig, HttpProvider, LoggingProvider, TimeProvider,
};
use ash_core::runtime::HostSandboxPolicy;
use ash_core::{CapabilityBinding, CapabilityBindingId, CapabilityInterfaceId};
use ash_runtime::{ExecError, ExecResult, RuntimeState};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Supported standard provider/profile families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StandardProfileKind {
    /// Filesystem read-only profile.
    ReadOnlyFilesystem,
    /// Filesystem read-write profile.
    ReadWriteFilesystem,
    /// HTTP profile constrained by host/method policy metadata.
    SandboxedHttp,
    /// Deterministic test profile with fixed clock inputs.
    DeterministicTest,
    /// Logging-only profile.
    LoggingOnly,
    /// Application default profile.
    ApplicationDefault,
}

/// Result of installing one standard profile into a runtime state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledStandardProfile {
    /// Profile name selected at the runtime boundary.
    pub name: String,
    /// Profile family.
    pub kind: StandardProfileKind,
    /// Capability bindings admitted by this profile.
    pub binding_ids: Vec<CapabilityBindingId>,
    /// Sandbox policy identities registered by this profile.
    pub sandbox_policies: Vec<String>,
    /// Profiles must not grant authority by name.
    pub grants_authority: bool,
}

/// One standard provider profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardProviderProfile {
    name: String,
    kind: StandardProfileKind,
    provider_bindings: Vec<StandardProviderBinding>,
    sandbox_policies: Vec<HostSandboxPolicy>,
    fixed_epoch_millis: Option<u64>,
    allowed_paths: Vec<PathBuf>,
    allowed_hosts: Vec<String>,
    grants_authority: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StandardProviderBinding {
    binding_name: String,
    provider_name: String,
    rows: Vec<String>,
}

impl StandardProviderProfile {
    /// Create a read-only filesystem profile constrained to the provided path prefixes.
    pub fn read_only_filesystem<'a>(
        name: impl Into<String>,
        allowed_paths: impl IntoIterator<Item = &'a Path>,
    ) -> Self {
        let allowed_paths = allowed_paths
            .into_iter()
            .map(Path::to_path_buf)
            .collect::<Vec<_>>();
        Self {
            name: name.into(),
            kind: StandardProfileKind::ReadOnlyFilesystem,
            provider_bindings: vec![StandardProviderBinding::new(
                "fs",
                "fs",
                ["fs.exists", "fs.read", "fs.metadata", "fs.read_dir"],
            )],
            sandbox_policies: fs_policies(false, &allowed_paths),
            fixed_epoch_millis: None,
            allowed_paths,
            allowed_hosts: Vec::new(),
            grants_authority: false,
        }
    }

    /// Create a read-write filesystem profile constrained to the provided path prefixes.
    pub fn read_write_filesystem<'a>(
        name: impl Into<String>,
        allowed_paths: impl IntoIterator<Item = &'a Path>,
    ) -> Self {
        let allowed_paths = allowed_paths
            .into_iter()
            .map(Path::to_path_buf)
            .collect::<Vec<_>>();
        Self {
            name: name.into(),
            kind: StandardProfileKind::ReadWriteFilesystem,
            provider_bindings: vec![StandardProviderBinding::new(
                "fs",
                "fs",
                [
                    "fs.exists",
                    "fs.read",
                    "fs.metadata",
                    "fs.read_dir",
                    "fs.write",
                    "fs.append",
                    "fs.copy",
                    "fs.rename",
                    "fs.remove_file",
                    "fs.create_dir",
                    "fs.create_dir_all",
                    "fs.remove_dir",
                    "fs.remove_dir_all",
                ],
            )],
            sandbox_policies: fs_policies(false, &allowed_paths),
            fixed_epoch_millis: None,
            allowed_paths,
            allowed_hosts: Vec::new(),
            grants_authority: false,
        }
    }

    /// Create a sandboxed HTTP profile for an explicit host allow-list.
    pub fn sandboxed_http(
        name: impl Into<String>,
        allowed_hosts: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let allowed_hosts = allowed_hosts
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();
        Self {
            name: name.into(),
            kind: StandardProfileKind::SandboxedHttp,
            provider_bindings: vec![StandardProviderBinding::new(
                "http",
                "http",
                [
                    "http.get",
                    "http.head",
                    "http.post",
                    "http.put",
                    "http.delete",
                ],
            )],
            sandbox_policies: ["get", "head", "post", "put", "delete"]
                .into_iter()
                .map(|operation| {
                    allowed_hosts.iter().fold(
                        HostSandboxPolicy::allow_all(format!("host.http.{operation}")),
                        |policy, host| policy.with_allowed_host(host.clone()),
                    )
                })
                .collect(),
            fixed_epoch_millis: None,
            allowed_paths: Vec::new(),
            allowed_hosts,
            grants_authority: false,
        }
    }

    /// Create a deterministic test profile with a fixed clock.
    #[must_use]
    pub fn deterministic_test(name: impl Into<String>, fixed_epoch_millis: u64) -> Self {
        Self {
            name: name.into(),
            kind: StandardProfileKind::DeterministicTest,
            provider_bindings: vec![StandardProviderBinding::new(
                "time",
                "time",
                [
                    "time.now",
                    "time.now_iso",
                    "time.epoch_millis",
                    "time.sleep",
                ],
            )],
            sandbox_policies: vec![
                HostSandboxPolicy::allow_all("host.time.now"),
                HostSandboxPolicy::deny_all(
                    "host.time.sleep",
                    "deterministic test profile denies wall-clock sleep",
                ),
            ],
            fixed_epoch_millis: Some(fixed_epoch_millis),
            allowed_paths: Vec::new(),
            allowed_hosts: Vec::new(),
            grants_authority: false,
        }
    }

    /// Create a logging-only profile. It installs a deny-by-default log policy so denied attempts
    /// still produce redacted host-boundary evidence.
    #[must_use]
    pub fn logging_only(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: StandardProfileKind::LoggingOnly,
            provider_bindings: vec![StandardProviderBinding::new(
                "logging",
                "logging",
                [
                    "logging.debug",
                    "logging.info",
                    "logging.warn",
                    "logging.error",
                ],
            )],
            sandbox_policies: vec![HostSandboxPolicy::deny_all(
                "host.logging.write",
                "logging profile is evidence-only by default",
            )],
            fixed_epoch_millis: None,
            allowed_paths: Vec::new(),
            allowed_hosts: Vec::new(),
            grants_authority: false,
        }
    }

    /// Create an application-default profile over explicit standard provider rows.
    ///
    /// The default application bundle is still authority-neutral: it registers standard providers,
    /// sandbox policies, and row admissions, but does not grant authority by profile name.
    pub fn application_default<'a>(
        name: impl Into<String>,
        allowed_paths: impl IntoIterator<Item = &'a Path>,
        allowed_hosts: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let allowed_paths = allowed_paths
            .into_iter()
            .map(Path::to_path_buf)
            .collect::<Vec<_>>();
        let allowed_hosts = allowed_hosts
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();
        let mut sandbox_policies = fs_policies(false, &allowed_paths);
        sandbox_policies.extend(["get", "head", "post", "put", "delete"].into_iter().map(
            |operation| {
                allowed_hosts.iter().fold(
                    HostSandboxPolicy::allow_all(format!("host.http.{operation}")),
                    |policy, host| policy.with_allowed_host(host.clone()),
                )
            },
        ));
        sandbox_policies.push(HostSandboxPolicy::allow_all("host.time.now"));
        sandbox_policies.push(HostSandboxPolicy::allow_all("host.time.sleep"));
        sandbox_policies.push(HostSandboxPolicy::allow_all("host.logging.write"));

        Self {
            name: name.into(),
            kind: StandardProfileKind::ApplicationDefault,
            provider_bindings: vec![
                StandardProviderBinding::new(
                    "fs",
                    "fs",
                    ["fs.exists", "fs.read", "fs.metadata", "fs.read_dir"],
                ),
                StandardProviderBinding::new(
                    "http",
                    "http",
                    [
                        "http.get",
                        "http.head",
                        "http.post",
                        "http.put",
                        "http.delete",
                    ],
                ),
                StandardProviderBinding::new(
                    "time",
                    "time",
                    [
                        "time.now",
                        "time.now_iso",
                        "time.epoch_millis",
                        "time.sleep",
                    ],
                ),
                StandardProviderBinding::new(
                    "logging",
                    "logging",
                    [
                        "logging.debug",
                        "logging.info",
                        "logging.warn",
                        "logging.error",
                    ],
                ),
            ],
            sandbox_policies,
            fixed_epoch_millis: None,
            allowed_paths,
            allowed_hosts,
            grants_authority: false,
        }
    }

    /// Escape hatch for constructing invalid profile metadata in validation tests.
    #[must_use]
    pub const fn with_authority_grant_for_test(mut self, grants_authority: bool) -> Self {
        self.grants_authority = grants_authority;
        self
    }

    /// Escape hatch for constructing stale or incompatible profile rows in validation tests.
    #[must_use]
    pub fn with_provider_rows_for_test<const N: usize>(
        mut self,
        provider_name: &str,
        rows: [&str; N],
    ) -> Self {
        if let Some(binding) = self
            .provider_bindings
            .iter_mut()
            .find(|binding| binding.provider_name == provider_name)
        {
            binding.rows = rows.into_iter().map(str::to_string).collect();
        }
        self
    }

    /// Return the profile family.
    #[must_use]
    pub const fn kind(&self) -> StandardProfileKind {
        self.kind
    }

    /// Return whether profile metadata claims direct authority.
    #[must_use]
    pub const fn grants_authority(&self) -> bool {
        self.grants_authority
    }

    /// Return declared rows for one provider.
    #[must_use]
    pub fn provider_rows(&self, provider_name: &str) -> Option<Vec<String>> {
        self.provider_bindings
            .iter()
            .find(|binding| binding.provider_name == provider_name)
            .map(|binding| binding.rows.clone())
    }

    /// Return sandbox policy identities declared by this profile.
    #[must_use]
    pub fn sandbox_policies(&self) -> Vec<String> {
        self.sandbox_policies
            .iter()
            .map(|policy| policy.identity.clone())
            .collect()
    }

    /// Install this profile into a runtime state.
    ///
    /// # Errors
    ///
    /// Returns runtime validation errors when a provider, sandbox policy, or capability binding is
    /// malformed or incompatible with registered provider metadata.
    pub async fn install(&self, runtime: &RuntimeState) -> ExecResult<InstalledStandardProfile> {
        if self.name.trim().is_empty() {
            return Err(ExecError::InvalidRuntimeState(
                "standard profile is missing name".to_string(),
            ));
        }
        if self.grants_authority {
            return Err(ExecError::InvalidRuntimeState(format!(
                "standard profile '{}' must not grant authority",
                self.name
            )));
        }

        self.install_providers(runtime);

        let mut sandbox_policies = Vec::with_capacity(self.sandbox_policies.len());
        for policy in &self.sandbox_policies {
            let registered = runtime.register_host_sandbox_policy(policy.clone()).await?;
            sandbox_policies.push(registered.identity);
        }

        let mut binding_ids = Vec::with_capacity(self.provider_bindings.len());
        for binding in &self.provider_bindings {
            let id = CapabilityBindingId::new();
            let admitted = runtime
                .admit_capability_binding(CapabilityBinding::host_provider(
                    id,
                    binding.binding_name.clone(),
                    CapabilityInterfaceId::new(&binding.binding_name),
                    binding.provider_name.clone(),
                    binding.rows.clone(),
                ))
                .await?;
            binding_ids.push(admitted);
        }

        Ok(InstalledStandardProfile {
            name: self.name.clone(),
            kind: self.kind,
            binding_ids,
            sandbox_policies,
            grants_authority: self.grants_authority,
        })
    }

    fn install_providers(&self, runtime: &RuntimeState) {
        match self.kind {
            StandardProfileKind::ReadOnlyFilesystem => {
                runtime.register_provider(
                    "fs",
                    Arc::new(FsProvider::with_config(FsConfig {
                        allowed_paths: self.allowed_paths.clone(),
                        read_only: true,
                        base_dir: None,
                    })),
                );
            }
            StandardProfileKind::ReadWriteFilesystem => {
                runtime.register_provider(
                    "fs",
                    Arc::new(FsProvider::with_config(FsConfig {
                        allowed_paths: self.allowed_paths.clone(),
                        read_only: false,
                        base_dir: None,
                    })),
                );
            }
            StandardProfileKind::SandboxedHttp => {
                runtime.register_provider(
                    "http",
                    Arc::new(HttpProvider::with_config(
                        HttpConfig::new().with_allowed_hosts(self.allowed_hosts.clone()),
                    )),
                );
            }
            StandardProfileKind::DeterministicTest => {
                if let Some(fixed_epoch_millis) = self.fixed_epoch_millis {
                    runtime.register_provider(
                        "time",
                        Arc::new(TimeProvider::mock(fixed_epoch_millis)),
                    );
                }
            }
            StandardProfileKind::LoggingOnly => {
                runtime.register_provider("logging", Arc::new(LoggingProvider::new()));
            }
            StandardProfileKind::ApplicationDefault => {
                runtime.register_provider(
                    "fs",
                    Arc::new(FsProvider::with_config(FsConfig {
                        allowed_paths: self.allowed_paths.clone(),
                        read_only: true,
                        base_dir: None,
                    })),
                );
                runtime.register_provider(
                    "http",
                    Arc::new(HttpProvider::with_config(
                        HttpConfig::new().with_allowed_hosts(self.allowed_hosts.clone()),
                    )),
                );
                runtime.register_provider("time", Arc::new(TimeProvider::new()));
                runtime.register_provider("logging", Arc::new(LoggingProvider::new()));
            }
        }
    }
}

impl StandardProviderBinding {
    fn new<const N: usize>(
        binding_name: impl Into<String>,
        provider_name: impl Into<String>,
        rows: [&str; N],
    ) -> Self {
        Self {
            binding_name: binding_name.into(),
            provider_name: provider_name.into(),
            rows: rows.into_iter().map(str::to_string).collect(),
        }
    }
}

fn fs_policies(deny_all: bool, allowed_paths: &[PathBuf]) -> Vec<HostSandboxPolicy> {
    let operations = [
        "exists",
        "read_file",
        "read_to_string",
        "metadata",
        "read_dir",
        "write_file",
        "write",
        "write_string",
        "append",
        "copy",
        "rename",
        "remove_file",
        "create_dir",
        "create_dir_all",
        "remove_dir",
        "remove_dir_all",
    ];
    operations
        .into_iter()
        .map(|operation| {
            let identity = format!("host.fs.{operation}");
            if deny_all {
                HostSandboxPolicy::deny_all(identity, "filesystem profile denied host execution")
            } else {
                allowed_paths
                    .iter()
                    .fold(HostSandboxPolicy::allow_all(identity), |policy, path| {
                        policy.with_allowed_path(path.display().to_string())
                    })
            }
        })
        .collect()
}
