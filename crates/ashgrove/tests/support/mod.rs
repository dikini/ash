#![allow(dead_code)]

use std::path::{Path, PathBuf};

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

pub fn source_fixture(id: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("source");
    create_toolchain_shape(dir.path(), id);
    std::fs::write(dir.path().join(".source-rev"), "abcdef1234567890").expect("rev");
    dir
}

pub fn install_fake_toolchain(roots: &XdgFixture, id: &str) {
    let path = roots.toolchain(id);
    create_toolchain_shape(&path, id);
}

pub fn create_toolchain_shape(root: &Path, id: &str) {
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    std::fs::create_dir_all(root.join("bin")).expect("bin");
    std::fs::create_dir_all(root.join("lib/ash/std/src")).expect("std");
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
        root.join("manifest.toml"),
        format!("toolchain_id = \"{id}\"\nversion = \"0.1.0\"\n"),
    )
    .expect("manifest");
    std::fs::write(root.join("install-record.toml"), "source = \"fixture\"\n").expect("record");
}

pub fn toolchain_tarball_fixture(id: &str) -> tempfile::NamedTempFile {
    let source = tempfile::tempdir().expect("source");
    create_toolchain_shape(source.path(), id);
    pack_toolchain_dir(id, source.path())
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

fn run_git(dir: &Path, args: &[&str]) {
    let status = std::process::Command::new("git")
        .args(["-c", "commit.gpgsign=false", "-c", "tag.gpgSign=false"])
        .args(args)
        .current_dir(dir)
        .status()
        .expect("git");
    assert!(status.success(), "git {args:?}");
}
