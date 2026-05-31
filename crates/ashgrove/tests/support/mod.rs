#![allow(dead_code)]

use std::path::{Path, PathBuf};

use ashgrove::AshgrovePaths;
use sha2::{Digest, Sha256};

pub struct XdgFixture {
    pub data: tempfile::TempDir,
    pub config: tempfile::TempDir,
    pub cache: tempfile::TempDir,
    pub state: tempfile::TempDir,
    pub home: tempfile::TempDir,
}

impl XdgFixture {
    pub fn env(&self) -> Vec<(&'static str, String)> {
        vec![
            ("HOME", self.home.path().display().to_string()),
            ("XDG_DATA_HOME", self.data.path().display().to_string()),
            ("XDG_CONFIG_HOME", self.config.path().display().to_string()),
            ("XDG_CACHE_HOME", self.cache.path().display().to_string()),
            ("XDG_STATE_HOME", self.state.path().display().to_string()),
        ]
    }

    pub fn toolchain(&self, id: &str) -> PathBuf {
        self.data.path().join("ash/toolchains").join(id)
    }
}

pub fn xdg_fixture() -> XdgFixture {
    XdgFixture {
        data: tempfile::tempdir().expect("data"),
        config: tempfile::tempdir().expect("config"),
        cache: tempfile::tempdir().expect("cache"),
        state: tempfile::tempdir().expect("state"),
        home: tempfile::tempdir().expect("home"),
    }
}

pub fn ashgrove_paths(roots: &XdgFixture) -> AshgrovePaths {
    AshgrovePaths::from_roots(
        roots.home.path().to_path_buf(),
        Some(roots.data.path().to_path_buf()),
        Some(roots.config.path().to_path_buf()),
        Some(roots.cache.path().to_path_buf()),
        Some(roots.state.path().to_path_buf()),
    )
}

pub fn source_fixture(id: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("source");
    create_toolchain_shape(dir.path(), id);
    std::fs::write(dir.path().join(".source-rev"), "abcdef1234567890").expect("rev");
    std::fs::write(
        dir.path().join("release-source.toml"),
        "schema_version = 1\norigin_commit = \"abcdef1234567890\"\n\n[attestation]\norigin_commit = \"abcdef1234567890\"\n",
    )
    .expect("release source metadata");
    dir
}

pub struct SourceWorkspaceFixture {
    dir: tempfile::TempDir,
    revision: String,
}

impl SourceWorkspaceFixture {
    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub fn revision(&self) -> &str {
        &self.revision
    }

    pub fn toolchain_id(&self) -> String {
        format!("ash-0.1.0+source.{}", &self.revision[..12])
    }
}

pub fn git_status(dir: &Path) -> String {
    git_output(dir, &["status", "--porcelain"])
}

pub fn source_workspace_fixture() -> SourceWorkspaceFixture {
    let dir = tempfile::tempdir().expect("source workspace");
    create_source_workspace(dir.path());
    run_git(dir.path(), &["init"]);
    run_git(dir.path(), &["config", "user.email", "ash@example.invalid"]);
    run_git(dir.path(), &["config", "user.name", "Ash Test"]);
    run_git(
        dir.path(),
        &["remote", "add", "origin", "https://example.invalid/ash.git"],
    );
    run_git(dir.path(), &["add", "."]);
    run_git(dir.path(), &["commit", "-m", "initial"]);
    let revision = git_output(dir.path(), &["rev-parse", "HEAD"])
        .trim()
        .to_string();
    SourceWorkspaceFixture { dir, revision }
}

pub fn unidentified_source_workspace_fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("source workspace");
    create_source_workspace(dir.path());
    dir
}

pub fn create_source_workspace(root: &Path) {
    std::fs::create_dir_all(root.join("crates/ash-cli/src")).expect("ash-cli src");
    std::fs::create_dir_all(root.join("crates/ashgrove/src")).expect("ashgrove src");
    std::fs::create_dir_all(root.join("std/src/runtime")).expect("std runtime src");
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/ash-cli\", \"crates/ashgrove\"]\nresolver = \"2\"\n\n[workspace.package]\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("workspace manifest");
    std::fs::write(
        root.join("crates/ash-cli/Cargo.toml"),
        "[package]\nname = \"ash-cli\"\nversion.workspace = true\nedition.workspace = true\n\n[[bin]]\nname = \"ash\"\npath = \"src/main.rs\"\n",
    )
    .expect("ash-cli manifest");
    std::fs::write(
        root.join("crates/ash-cli/src/main.rs"),
        "fn main() { println!(\"ash source fixture\"); }\n",
    )
    .expect("ash main");
    std::fs::write(
        root.join("crates/ashgrove/Cargo.toml"),
        "[package]\nname = \"ashgrove\"\nversion.workspace = true\nedition.workspace = true\n\n[[bin]]\nname = \"ashgrove\"\npath = \"src/main.rs\"\n",
    )
    .expect("ashgrove manifest");
    std::fs::write(
        root.join("crates/ashgrove/src/main.rs"),
        "fn main() { println!(\"ashgrove source fixture\"); }\n",
    )
    .expect("ashgrove main");
    std::fs::write(
        root.join("std/Cargo.toml"),
        "[package]\nname = \"ash-stdlib\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("std manifest");
    std::fs::write(
        root.join("std/src/lib.ash"),
        "pub type SourceRootSentinel = SourceRootSentinel;\n",
    )
    .expect("stdlib source");
    std::fs::write(
        root.join("std/src/runtime/args.ash"),
        "pub type SourceRuntimeArgs = SourceRuntimeArgs;\n",
    )
    .expect("runtime support source");
}

pub fn install_fake_toolchain(roots: &XdgFixture, id: &str) {
    let path = roots.toolchain(id);
    create_toolchain_shape(&path, id);
}

pub fn write_tool_script(path: &Path, body: &str) {
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(path, format!("#!/bin/sh\n{body}")).expect("tool script");
    #[cfg(unix)]
    {
        let mut permissions = std::fs::metadata(path).expect("metadata").permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).expect("permissions");
    }
}

pub fn create_toolchain_shape(root: &Path, id: &str) {
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    std::fs::create_dir_all(root.join("bin")).expect("bin");
    std::fs::create_dir_all(root.join("lib/ash/std/src/runtime")).expect("std");
    std::fs::write(root.join("bin/ash"), "#!/bin/sh\n").expect("ash");
    std::fs::write(root.join("bin/ashgrove"), "#!/bin/sh\n").expect("ashgrove");
    #[cfg(unix)]
    {
        for rel in ["bin/ash", "bin/ashgrove"] {
            let path = root.join(rel);
            let mut permissions = std::fs::metadata(&path).expect("metadata").permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(path, permissions).expect("permissions");
        }
    }
    std::fs::write(
        root.join("lib/ash/std/ash.toml"),
        "[package]\nname = \"std\"\n",
    )
    .expect("std manifest");
    std::fs::write(
        root.join("lib/ash/std/src/lib.ash"),
        "pub type StdSentinel = StdSentinel;\n",
    )
    .expect("stdlib");
    std::fs::write(
        root.join("lib/ash/std/src/runtime/args.ash"),
        "pub type RuntimeArgs = RuntimeArgs;\n",
    )
    .expect("runtime support");
    std::fs::write(
        root.join("manifest.toml"),
        format!(
            "toolchain_id = \"{id}\"\nversion = \"0.1.0\"\narchive_schema_version = 1\nsource_kind = \"fixture\"\n[stdlib]\nversion = \"0.1.0\"\npath = \"lib/ash/std\"\n[runtime_support]\nidentity = \"ash-runtime-support:0.1.0\"\npath = \"lib/ash/std/src/runtime\"\nrequired = true\n[[standard_tools]]\nname = \"ash\"\npath = \"bin/ash\"\nrequired = true\n[[standard_tools]]\nname = \"ashgrove\"\npath = \"bin/ashgrove\"\nrequired = true\n"
        ),
    )
    .expect("manifest");
    std::fs::write(
        root.join("install-record.toml"),
        format!("toolchain_id = \"{id}\"\nsource_kind = \"tarball\"\narchive_schema_version = 1\n"),
    )
    .expect("record");
}

pub fn toolchain_tarball_fixture(id: &str) -> tempfile::NamedTempFile {
    let source = tempfile::tempdir().expect("source");
    create_toolchain_shape(source.path(), id);
    pack_toolchain_dir(id, source.path())
}

pub fn produced_toolchain_tarball_in(id: &str, output: &Path) -> std::path::PathBuf {
    let ash = tempfile::NamedTempFile::new().expect("ash");
    let ashgrove = tempfile::NamedTempFile::new().expect("ashgrove");
    write_tool_script(ash.path(), "echo produced ash\n");
    write_tool_script(ashgrove.path(), "echo produced ashgrove\n");

    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root");
    let package = std::process::Command::new(repo_root.join("scripts/package-ash-toolchain.sh"))
        .args([
            "--toolchain-id",
            id,
            "--output-dir",
            output.to_str().expect("utf8 output"),
        ])
        .env("ASH_PACKAGE_ASH_BIN", ash.path())
        .env("ASH_PACKAGE_ASHGROVE_BIN", ashgrove.path())
        .output()
        .expect("run package producer");
    assert!(
        package.status.success(),
        "producer failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&package.stdout),
        String::from_utf8_lossy(&package.stderr)
    );
    let stdout = String::from_utf8(package.stdout).expect("producer stdout utf8");
    let archive = stdout
        .lines()
        .find_map(|line| line.strip_prefix("archive="))
        .map(std::path::PathBuf::from)
        .expect("archive output");
    assert!(archive.is_file(), "archive exists at {}", archive.display());
    archive
}

pub fn toolchain_tarball_fixture_with_mutation(
    id: &str,
    mutate: impl FnOnce(&Path),
) -> tempfile::NamedTempFile {
    let source = tempfile::tempdir().expect("source");
    create_toolchain_shape(source.path(), id);
    mutate(source.path());
    pack_toolchain_dir(id, source.path())
}

pub fn remove_toml_line(path: impl AsRef<Path>, key: &str) {
    let path = path.as_ref();
    let text = std::fs::read_to_string(path).expect("read toml");
    let filtered = text
        .lines()
        .filter(|line| !line.trim_start().starts_with(key))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(path, format!("{filtered}\n")).expect("write toml");
}

pub fn replace_toml_line(path: impl AsRef<Path>, key: &str, replacement: &str) {
    let path = path.as_ref();
    let text = std::fs::read_to_string(path).expect("read toml");
    let replaced = text
        .lines()
        .map(|line| {
            if line.trim_start().starts_with(key) {
                replacement
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(path, format!("{replaced}\n")).expect("write toml");
}

#[cfg(unix)]
pub fn non_executable_toolchain_tarball_fixture(id: &str) -> tempfile::NamedTempFile {
    use std::os::unix::fs::PermissionsExt;

    let source = tempfile::tempdir().expect("source");
    create_toolchain_shape(source.path(), id);
    let mut permissions = std::fs::metadata(source.path().join("bin/ash"))
        .expect("metadata")
        .permissions();
    permissions.set_mode(0o644);
    std::fs::set_permissions(source.path().join("bin/ash"), permissions).expect("permissions");
    pack_toolchain_dir(id, source.path())
}

fn pack_toolchain_dir(id: &str, source: &Path) -> tempfile::NamedTempFile {
    let file = tempfile::NamedTempFile::new().expect("archive");
    let encoder = flate2::write::GzEncoder::new(
        file.reopen().expect("reopen"),
        flate2::Compression::default(),
    );
    let mut builder = tar::Builder::new(encoder);
    builder.append_dir_all(id, source).expect("append");
    builder.finish().expect("finish");
    file
}

pub fn unsafe_tarball_fixture() -> tempfile::NamedTempFile {
    let file = tempfile::NamedTempFile::new().expect("archive");
    let encoder = flate2::write::GzEncoder::new(
        file.reopen().expect("reopen"),
        flate2::Compression::default(),
    );
    let mut builder = tar::Builder::new(encoder);
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(tar::EntryType::Symlink);
    header
        .set_path("ash-0.1.0+test.tarball.unsafe/bin/ash")
        .expect("path");
    header.set_link_name("/tmp/escape").expect("link");
    header.set_size(0);
    header.set_cksum();
    builder.append(&header, &[][..]).expect("append");
    builder.finish().expect("finish");
    file
}

pub fn unsafe_path_tarball_fixture(path: &str) -> tempfile::NamedTempFile {
    let file = tempfile::NamedTempFile::new().expect("archive");
    let encoder = flate2::write::GzEncoder::new(
        file.reopen().expect("reopen"),
        flate2::Compression::default(),
    );
    let mut builder = tar::Builder::new(encoder);
    let bytes = b"escape";
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(tar::EntryType::Regular);
    set_raw_tar_path(&mut header, path);
    header.set_size(bytes.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder.append(&header, &bytes[..]).expect("append");
    builder.finish().expect("finish");
    file
}

fn set_raw_tar_path(header: &mut tar::Header, path: &str) {
    let bytes = path.as_bytes();
    assert!(bytes.len() <= 100, "test tar path too long");
    let header_bytes = header.as_mut_bytes();
    header_bytes[..bytes.len()].copy_from_slice(bytes);
    header_bytes[bytes.len()] = 0;
}

pub fn unsafe_hardlink_tarball_fixture() -> tempfile::NamedTempFile {
    let file = tempfile::NamedTempFile::new().expect("archive");
    let encoder = flate2::write::GzEncoder::new(
        file.reopen().expect("reopen"),
        flate2::Compression::default(),
    );
    let mut builder = tar::Builder::new(encoder);
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(tar::EntryType::Link);
    header
        .set_path("ash-0.1.0+test.tarball.hardlink001/bin/ash")
        .expect("path");
    header.set_link_name("/tmp/escape").expect("link");
    header.set_size(0);
    header.set_cksum();
    builder.append(&header, &[][..]).expect("append");
    builder.finish().expect("finish");
    file
}

pub fn unsafe_mode_tarball_fixture(mode: u32) -> tempfile::NamedTempFile {
    let file = tempfile::NamedTempFile::new().expect("archive");
    let encoder = flate2::write::GzEncoder::new(
        file.reopen().expect("reopen"),
        flate2::Compression::default(),
    );
    let mut builder = tar::Builder::new(encoder);
    let bytes = b"mode";
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(tar::EntryType::Regular);
    header
        .set_path("ash-0.1.0+test.tarball.mode0000001/bin/ash")
        .expect("path");
    header.set_size(bytes.len() as u64);
    header.set_mode(mode);
    header.set_cksum();
    builder.append(&header, &bytes[..]).expect("append");
    builder.finish().expect("finish");
    file
}

pub fn project_fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("project");
    std::fs::create_dir_all(dir.path().join("src")).expect("src");
    std::fs::write(dir.path().join("src/main.ash"), "workflow main { ret 0 }\n").expect("main");
    dir
}

pub struct GitDepFixture {
    dir: tempfile::TempDir,
}

impl GitDepFixture {
    pub fn url(&self) -> String {
        format!("file://{}", self.dir.path().display())
    }

    pub fn commit(&self, rev: &str) -> String {
        git_output(self.dir.path(), &["rev-parse", rev])
            .trim()
            .to_string()
    }

    pub fn force_tag(&self, tag: &str, rev: &str) {
        run_git(self.dir.path(), &["tag", "-f", tag, rev]);
    }
}

pub fn git_dep_fixture() -> GitDepFixture {
    let dir = tempfile::tempdir().expect("git dep");
    std::fs::write(dir.path().join("lib.ash"), "pub type Dep = Dep;\n").expect("dep");
    run_git(dir.path(), &["init"]);
    run_git(dir.path(), &["config", "user.email", "ash@example.invalid"]);
    run_git(dir.path(), &["config", "user.name", "Ash Test"]);
    run_git(dir.path(), &["add", "."]);
    run_git(dir.path(), &["commit", "-m", "initial"]);
    run_git(dir.path(), &["tag", "v1"]);
    std::fs::write(
        dir.path().join("lib.ash"),
        "pub type Dep = Dep;\npub type Dep2 = Dep2;\n",
    )
    .expect("dep2");
    run_git(dir.path(), &["add", "."]);
    run_git(dir.path(), &["commit", "-m", "second"]);
    run_git(dir.path(), &["tag", "v2"]);
    GitDepFixture { dir }
}

pub fn locked_project_fixture() -> tempfile::TempDir {
    let project = project_fixture();
    std::fs::write(
        project.path().join("ash.lock"),
        "[[package]]\nname = \"dep\"\ngit = \"file:///tmp/dep\"\ncommit = \"0123456789abcdef0123456789abcdef01234567\"\nsource_path = \"/tmp/dep\"\n",
    )
    .expect("lock");
    project
}

pub fn git_url_digest(url: &str) -> String {
    let digest = Sha256::digest(url.as_bytes());
    let mut out = String::new();
    for byte in &digest[..8] {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn run_git(dir: &Path, args: &[&str]) {
    let status = std::process::Command::new("git")
        .args(["-c", "commit.gpgsign=false", "-c", "tag.gpgSign=false"])
        .args(args)
        .current_dir(dir)
        .status()
        .expect("git");
    assert!(status.success(), "git {args:?}");
}

fn git_output(dir: &Path, args: &[&str]) -> String {
    let output = std::process::Command::new("git")
        .args(["-c", "commit.gpgsign=false", "-c", "tag.gpgSign=false"])
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git");
    assert!(output.status.success(), "git {args:?}");
    String::from_utf8(output.stdout).expect("git stdout")
}
