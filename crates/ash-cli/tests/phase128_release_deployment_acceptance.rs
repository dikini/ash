use assert_cmd::Command;
use predicates::prelude::*;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const HELPER_GIT_URL: &str = "https://user:secret@example.invalid/org/helper.git";
const SANITIZED_HELPER_GIT_URL: &str = "https://example.invalid/org/helper.git";

struct XdgFixture {
    data: TempDir,
    config: TempDir,
    cache: TempDir,
    state: TempDir,
    home: TempDir,
}

impl XdgFixture {
    fn new() -> Self {
        Self {
            data: tempfile::tempdir().expect("data"),
            config: tempfile::tempdir().expect("config"),
            cache: tempfile::tempdir().expect("cache"),
            state: tempfile::tempdir().expect("state"),
            home: tempfile::tempdir().expect("home"),
        }
    }

    fn env(&self) -> Vec<(&'static str, String)> {
        vec![
            ("HOME", self.home.path().display().to_string()),
            ("XDG_DATA_HOME", self.data.path().display().to_string()),
            ("XDG_CONFIG_HOME", self.config.path().display().to_string()),
            ("XDG_CACHE_HOME", self.cache.path().display().to_string()),
            ("XDG_STATE_HOME", self.state.path().display().to_string()),
        ]
    }

    fn launcher_bin(&self) -> PathBuf {
        self.home.path().join(".local/bin")
    }
}

struct LockedProject {
    _temp: TempDir,
    _helper_dep: TempDir,
    root: PathBuf,
    main: PathBuf,
    helper_commit: String,
}

#[test]
fn task_985_cli_uses_locked_authenticated_dependency_with_selected_toolchain_runtime_support() {
    let roots = XdgFixture::new();
    let output = tempfile::tempdir().expect("output");
    let toolchain_id = "ash-0.1.0+tarball.cli985";
    let archive = cli_toolchain_tarball(toolchain_id, output.path());

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "install",
            "--from",
            "tarball",
            "--path",
            archive.to_str().expect("utf8 archive"),
            "--switch",
        ])
        .envs(roots.env())
        .assert()
        .success();

    let project = locked_authenticated_project(&roots);

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "lock",
            "--project",
            project.root.to_str().expect("utf8 project"),
        ])
        .envs(roots.env())
        .assert()
        .success();

    let lock_text = fs::read_to_string(project.root.join("ash.lock")).expect("lock");
    assert!(lock_text.contains(SANITIZED_HELPER_GIT_URL));
    assert!(lock_text.contains("authenticated_origin = \"credentials-redacted\""));
    assert!(!lock_text.contains("user"));
    assert!(!lock_text.contains("secret"));

    let checkout = roots
        .cache
        .path()
        .join("ash/git/checkouts")
        .join(format!(
            "helper-{}",
            git_url_digest(SANITIZED_HELPER_GIT_URL)
        ))
        .join(&project.helper_commit);
    clone_git_dep(project._helper_dep.path(), &checkout);

    let check_env_capture = project.root.join("check-runtime-env.txt");
    Command::new(roots.launcher_bin().join("ash"))
        .args(["check", project.main.to_str().expect("utf8 main")])
        .envs(roots.env())
        .env("ASH_TASK985_ENV_CAPTURE", &check_env_capture)
        .env("ASH_RUNTIME_KERNEL_REPORT", "json")
        .assert()
        .success()
        .stdout(predicate::str::contains("[OK]"));
    assert_runtime_support_capture(&check_env_capture);

    let run_env_capture = project.root.join("run-runtime-env.txt");
    Command::new(roots.launcher_bin().join("ash"))
        .args([
            "run",
            &format!("{}:main", project.main.to_str().expect("utf8 main")),
        ])
        .envs(roots.env())
        .env("ASH_TASK985_ENV_CAPTURE", &run_env_capture)
        .env("ASH_RUNTIME_KERNEL_REPORT", "json")
        .assert()
        .success()
        .stdout(predicate::str::contains("HelperToken"));
    assert_runtime_support_capture(&run_env_capture);
}

fn locked_authenticated_project(roots: &XdgFixture) -> LockedProject {
    let temp = tempfile::tempdir().expect("project");
    let helper_dep = tempfile::tempdir().expect("helper git dep");
    let helper_commit = init_git_dep(
        helper_dep.path(),
        "pub type HelperToken = HelperToken { value: Int };\n",
    );
    let root = temp.path().to_path_buf();
    fs::create_dir_all(root.join("src")).expect("src");
    fs::write(
        root.join("ash.toml"),
        format!(
            "[package]\nname = \"task985-cli\"\n\n[dependencies.helper]\ngit = \"{HELPER_GIT_URL}\"\nrev = \"{helper_commit}\"\n",
        ),
    )
    .expect("manifest");
    let main = root.join("src/main.ash");
    fs::write(
        &main,
        "use helper::{HelperToken}\nworkflow main() -> HelperToken { ret HelperToken { value: 7 }; }\n",
    )
    .expect("main");

    let _ = roots;
    LockedProject {
        _temp: temp,
        _helper_dep: helper_dep,
        root,
        main,
        helper_commit,
    }
}

fn cli_toolchain_tarball(id: &str, output: &Path) -> PathBuf {
    let script = workspace_root().join("scripts/package-ash-toolchain.sh");
    let wrapper_dir = tempfile::tempdir().expect("ash wrapper dir");
    let wrapper = wrapper_dir.path().join("ash");
    write_ash_env_capture_wrapper(&wrapper);
    let package = std::process::Command::new(script)
        .args([
            "--toolchain-id",
            id,
            "--output-dir",
            output.to_str().expect("utf8 output"),
        ])
        .env("ASH_PACKAGE_ASH_BIN", &wrapper)
        .env(
            "ASH_PACKAGE_ASHGROVE_BIN",
            assert_cmd::cargo::cargo_bin("ashgrove"),
        )
        .output()
        .expect("run package producer");
    assert!(
        package.status.success(),
        "producer failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&package.stdout),
        String::from_utf8_lossy(&package.stderr)
    );
    let stdout = String::from_utf8(package.stdout).expect("producer stdout utf8");
    stdout
        .lines()
        .find_map(|line| line.strip_prefix("archive="))
        .map(PathBuf::from)
        .expect("archive output")
}

fn write_ash_env_capture_wrapper(path: &Path) {
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    let real_ash = assert_cmd::cargo::cargo_bin("ash");
    fs::write(
        path,
        format!(
            "#!/bin/sh\nif [ -n \"${{ASH_TASK985_ENV_CAPTURE:-}}\" ]; then\n  printf 'runtime=%s\\nstdlib=%s\\n' \"${{ASH_RUNTIME_SUPPORT_IDENTITY:-}}\" \"${{ASH_STDLIB_ROOT:-}}\" > \"$ASH_TASK985_ENV_CAPTURE\"\nfi\nexec \"{}\" \"$@\"\n",
            real_ash.display()
        ),
    )
    .expect("write ash wrapper");
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(path).expect("wrapper metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("wrapper permissions");
    }
}

fn assert_runtime_support_capture(path: &Path) {
    let captured = fs::read_to_string(path).expect("runtime support capture");
    assert!(
        captured.contains("runtime=ash-runtime-support:0.1.0"),
        "selected toolchain runtime support identity missing:\n{captured}"
    );
    assert!(
        captured.contains("stdlib=") && captured.contains("lib/ash/std/src"),
        "selected toolchain stdlib root missing:\n{captured}"
    );
}

fn init_git_dep(root: &Path, module_source: &str) -> String {
    fs::write(root.join("mod.ash"), module_source).expect("git dep module");
    run_git(root, &["init"]);
    run_git(root, &["config", "user.email", "ash@example.invalid"]);
    run_git(root, &["config", "user.name", "Ash Test"]);
    run_git(root, &["add", "."]);
    run_git(root, &["commit", "-m", "initial"]);
    git_output(root, &["rev-parse", "HEAD"]).trim().to_string()
}

fn clone_git_dep(source: &Path, checkout: &Path) {
    run_git(
        Path::new("."),
        &[
            "clone",
            source.to_str().expect("utf8 source"),
            checkout.to_str().expect("utf8 checkout"),
        ],
    );
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

fn git_url_digest(url: &str) -> String {
    let digest = Sha256::digest(url.as_bytes());
    let mut out = String::new();
    for byte in &digest[..8] {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
        .to_path_buf()
}
