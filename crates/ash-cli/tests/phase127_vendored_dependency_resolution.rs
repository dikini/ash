use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const HELPER_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const HELPER_GIT_URL: &str = "file:///tmp/helper";
const HELPER_GIT_DIGEST: &str = "520d384526df63a4";
const OPTION_COMMIT: &str = "fedcba9876543210fedcba9876543210fedcba98";
const OPTION_GIT_URL: &str = "file:///tmp/option";
const OPTION_GIT_DIGEST: &str = "89d73728824c6295";

struct VendoredProject {
    _temp: TempDir,
    root: PathBuf,
    main: PathBuf,
}

struct FetchedProject {
    _temp: TempDir,
    _cache: TempDir,
    _helper_dep: TempDir,
    root: PathBuf,
    cache_root: PathBuf,
    helper_commit: String,
    main: PathBuf,
}

impl FetchedProject {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("temp project");
        let cache = tempfile::tempdir().expect("xdg cache");
        let helper_dep = tempfile::tempdir().expect("helper git dep");
        let helper_commit = init_git_dep(
            helper_dep.path(),
            "pub type HelperToken = HelperToken { value: Int };\n",
        );
        let root = temp.path().to_path_buf();
        let src = root.join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(
            root.join("ash.toml"),
            format!(
                "[package]\nname = \"app\"\n\n[dependencies.helper]\ngit = \"{HELPER_GIT_URL}\"\nrev = \"{helper_commit}\"\n",
            ),
        )
        .expect("manifest");
        fs::write(
            root.join("ash.lock"),
            format!(
                "[[package]]\nname = \"helper\"\ngit = \"{HELPER_GIT_URL}\"\ncommit = \"{helper_commit}\"\n",
            ),
        )
        .expect("lock");
        let checkout = cache
            .path()
            .join("ash/git/checkouts")
            .join(format!("helper-{HELPER_GIT_DIGEST}"))
            .join(&helper_commit);
        clone_git_dep(helper_dep.path(), &checkout);
        let main = src.join("main.ash");
        fs::write(
            &main,
            "use helper::{HelperToken}\nworkflow main() -> HelperToken { ret HelperToken { value: 7 }; }\n",
        )
        .expect("main");

        Self {
            _temp: temp,
            cache_root: cache.path().to_path_buf(),
            _cache: cache,
            _helper_dep: helper_dep,
            root,
            helper_commit,
            main,
        }
    }

    fn main_path(&self) -> &Path {
        &self.main
    }
}

impl VendoredProject {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("temp project");
        let root = temp.path().to_path_buf();
        let src = root.join("src");
        let helper = root.join("vendor/ash/helper");
        fs::create_dir_all(&src).expect("src");
        fs::create_dir_all(&helper).expect("helper vendor dir");
        fs::write(
            root.join("ash.toml"),
            format!(
                "[package]\nname = \"app\"\n\n[dependencies.helper]\ngit = \"{HELPER_GIT_URL}\"\nrev = \"{HELPER_COMMIT}\"\n",
            ),
        )
        .expect("manifest");
        fs::write(
            root.join("ash.lock"),
            format!(
                "[[package]]\nname = \"helper\"\ngit = \"{HELPER_GIT_URL}\"\ncommit = \"{HELPER_COMMIT}\"\n",
            ),
        )
        .expect("lock");
        fs::write(
            helper.join("mod.ash"),
            "pub type HelperToken = HelperToken { value: Int };\n",
        )
        .expect("helper module");
        let main = src.join("main.ash");
        fs::write(
            &main,
            "use helper::{HelperToken}\nworkflow main() -> HelperToken { ret HelperToken { value: 7 }; }\n",
        )
        .expect("main");

        Self {
            _temp: temp,
            root,
            main,
        }
    }

    fn main_path(&self) -> &Path {
        &self.main
    }
}

fn ash_command() -> Command {
    let mut command = Command::cargo_bin("ash").expect("ash binary");
    command
        .arg("--color")
        .arg("never")
        .env_remove("ASH_DEPENDENCY_ROOTS")
        .env_remove("ASH_DEP_ROOTS")
        .env_remove("ASH_LIBRARY_PATH")
        .env_remove("ASH_STDLIB_ROOT");
    command
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
            source.to_str().expect("utf8"),
            checkout.to_str().expect("utf8"),
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

#[test]
fn check_discovers_locked_vendored_dependency_without_dependency_root_env() {
    let project = VendoredProject::new();

    let mut command = ash_command();
    command.args(["check", project.main_path().to_str().expect("utf8")]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("[OK]"))
        .stdout(predicate::str::contains("main.ash"));
}

#[test]
fn run_discovers_locked_vendored_dependency_without_dependency_root_env() {
    let project = VendoredProject::new();

    let mut command = ash_command();
    command.args([
        "run",
        &format!("{}:main", project.main_path().to_str().expect("utf8")),
    ]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("HelperToken"));
}

#[test]
fn check_discovers_locked_fetched_cache_dependency_without_dependency_root_env() {
    let project = FetchedProject::new();

    let mut command = ash_command();
    command
        .env("XDG_CACHE_HOME", &project.cache_root)
        .args(["check", project.main_path().to_str().expect("utf8")]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("[OK]"))
        .stdout(predicate::str::contains("main.ash"));
}

#[test]
fn run_discovers_locked_fetched_cache_dependency_without_dependency_root_env() {
    let project = FetchedProject::new();

    let mut command = ash_command();
    command.env("XDG_CACHE_HOME", &project.cache_root).args([
        "run",
        &format!("{}:main", project.main_path().to_str().expect("utf8")),
    ]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("HelperToken"));
}

#[test]
fn missing_fetched_cache_checkout_fails_closed_without_source_fallback() {
    let project = FetchedProject::new();
    fs::remove_dir_all(
        project
            .cache_root
            .join("ash/git/checkouts")
            .join(format!("helper-{HELPER_GIT_DIGEST}"))
            .join(&project.helper_commit),
    )
    .expect("remove checkout");

    let mut command = ash_command();
    command
        .env("XDG_CACHE_HOME", &project.cache_root)
        .args(["check", project.main_path().to_str().expect("utf8")]);

    command.assert().failure().stderr(predicate::str::contains(
        "locked package 'helper' is missing from fetched cache",
    ));
}

#[test]
fn mismatched_fetched_cache_checkout_fails_closed_without_source_fallback() {
    let project = FetchedProject::new();
    let mismatched_commit = "1111111111111111111111111111111111111111";
    fs::write(
        project.root.join("ash.lock"),
        format!(
            "[[package]]\nname = \"helper\"\ngit = \"{HELPER_GIT_URL}\"\ncommit = \"{mismatched_commit}\"\n",
        ),
    )
    .expect("mismatched lock");
    let mismatched_checkout = project
        .cache_root
        .join("ash/git/checkouts")
        .join(format!("helper-{HELPER_GIT_DIGEST}"))
        .join(mismatched_commit);
    let real_checkout = project
        .cache_root
        .join("ash/git/checkouts")
        .join(format!("helper-{HELPER_GIT_DIGEST}"))
        .join(&project.helper_commit);
    clone_git_dep(&real_checkout, &mismatched_checkout);

    let mut command = ash_command();
    command
        .env("XDG_CACHE_HOME", &project.cache_root)
        .args(["check", project.main_path().to_str().expect("utf8")]);

    command.assert().failure().stderr(predicate::str::contains(
        "is not at locked commit 1111111111111111111111111111111111111111",
    ));
}

#[test]
fn cli_uses_explicit_stdlib_root_when_vendor_dependency_has_stdlib_module_name() {
    let stdlib = tempfile::tempdir().expect("explicit stdlib");
    fs::write(
        stdlib.path().join("option.ash"),
        "pub type SelectedOption = SelectedOption;\n",
    )
    .expect("selected stdlib option module");

    let project = VendoredProject::new();
    let vendored_option = project.root.join("vendor/ash/option");
    fs::create_dir_all(&vendored_option).expect("option vendor dir");
    fs::write(
        vendored_option.join("mod.ash"),
        "pub type VendoredOption = VendoredOption;\n",
    )
    .expect("vendored option module");
    fs::write(
        project.root.join("ash.toml"),
        format!(
            "[package]\nname = \"app\"\n\n[dependencies.helper]\ngit = \"{HELPER_GIT_URL}\"\nrev = \"{HELPER_COMMIT}\"\n\n[dependencies.option]\ngit = \"{OPTION_GIT_URL}\"\nrev = \"{OPTION_COMMIT}\"\n",
        ),
    )
    .expect("manifest");
    fs::write(
        project.root.join("ash.lock"),
        format!(
            "[[package]]\nname = \"helper\"\ngit = \"{HELPER_GIT_URL}\"\ncommit = \"{HELPER_COMMIT}\"\n\n[[package]]\nname = \"option\"\ngit = \"{OPTION_GIT_URL}\"\ncommit = \"{OPTION_COMMIT}\"\n",
        ),
    )
    .expect("lock");
    fs::write(
        project.root.join("src/main.ash"),
        "use helper::{HelperToken}\nuse option::{SelectedOption}\nworkflow main() -> HelperToken { ret HelperToken { value: 7 }; }\n",
    )
    .expect("main");

    let mut check = ash_command();
    check
        .env("ASH_STDLIB_ROOT", stdlib.path())
        .args(["check", project.main_path().to_str().expect("utf8")]);
    check
        .assert()
        .success()
        .stdout(predicate::str::contains("[OK]"));

    let mut run = ash_command();
    run.env("ASH_STDLIB_ROOT", stdlib.path()).args([
        "run",
        &format!("{}:main", project.main_path().to_str().expect("utf8")),
    ]);
    run.assert()
        .success()
        .stdout(predicate::str::contains("HelperToken"));
}

#[test]
fn cli_uses_explicit_stdlib_root_when_fetched_dependency_has_stdlib_module_name() {
    let stdlib = tempfile::tempdir().expect("explicit stdlib");
    fs::write(
        stdlib.path().join("option.ash"),
        "pub type SelectedOption = SelectedOption;\n",
    )
    .expect("selected stdlib option module");

    let project = FetchedProject::new();
    let option_dep = tempfile::tempdir().expect("option git dep");
    let option_commit = init_git_dep(
        option_dep.path(),
        "pub type FetchedOption = FetchedOption;\n",
    );
    fs::write(
        project.root.join("ash.toml"),
        format!(
            "[package]\nname = \"app\"\n\n[dependencies.helper]\ngit = \"{HELPER_GIT_URL}\"\nrev = \"{}\"\n\n[dependencies.option]\ngit = \"{OPTION_GIT_URL}\"\nrev = \"{option_commit}\"\n",
            project.helper_commit
        ),
    )
    .expect("manifest");
    fs::write(
        project.root.join("ash.lock"),
        format!(
            "[[package]]\nname = \"helper\"\ngit = \"{HELPER_GIT_URL}\"\ncommit = \"{}\"\n\n[[package]]\nname = \"option\"\ngit = \"{OPTION_GIT_URL}\"\ncommit = \"{option_commit}\"\n",
            project.helper_commit
        ),
    )
    .expect("lock");
    let option_checkout = project
        .cache_root
        .join("ash/git/checkouts")
        .join(format!("option-{OPTION_GIT_DIGEST}"))
        .join(&option_commit);
    clone_git_dep(option_dep.path(), &option_checkout);
    fs::write(
        project.root.join("src/main.ash"),
        "use helper::{HelperToken}\nuse option::{SelectedOption}\nworkflow main() -> HelperToken { ret HelperToken { value: 7 }; }\n",
    )
    .expect("main");

    let mut check = ash_command();
    check
        .env("ASH_STDLIB_ROOT", stdlib.path())
        .env("XDG_CACHE_HOME", &project.cache_root)
        .args(["check", project.main_path().to_str().expect("utf8")]);
    check
        .assert()
        .success()
        .stdout(predicate::str::contains("[OK]"));
}

#[test]
fn malformed_lock_package_name_fails_closed_without_resolving_vendor_escape() {
    let project = malformed_lock_project();

    let mut command = ash_command();
    command.args(["check", project.main_path().to_str().expect("utf8")]);

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid package name"));
}

#[test]
fn run_fails_closed_on_malformed_lock_package_name() {
    let project = malformed_lock_project();

    let mut command = ash_command();
    command.args([
        "run",
        &format!("{}:main", project.main_path().to_str().expect("utf8")),
    ]);

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid package name"));
}

#[test]
fn malformed_lock_commit_fails_closed_without_resolving_vendor() {
    let project = VendoredProject::new();
    write_malformed_helper_commit(&project);

    let mut command = ash_command();
    command.args(["check", project.main_path().to_str().expect("utf8")]);

    command.assert().failure().stderr(predicate::str::contains(
        "locked git commit must be a full 40-character commit hash",
    ));
}

#[test]
fn run_fails_closed_on_malformed_lock_commit() {
    let project = VendoredProject::new();
    write_malformed_helper_commit(&project);

    let mut command = ash_command();
    command.args([
        "run",
        &format!("{}:main", project.main_path().to_str().expect("utf8")),
    ]);

    command.assert().failure().stderr(predicate::str::contains(
        "locked git commit must be a full 40-character commit hash",
    ));
}

#[test]
fn check_fails_closed_on_required_lock_signature_mismatch() {
    let project = VendoredProject::new();
    let mut lock_text = fs::read_to_string(project.root.join("ash.lock")).expect("lock");
    lock_text.push_str(
        "\n[signing.lock]\nrequired = true\npackage_manifest_digest = \"sha256:0000000000000000000000000000000000000000000000000000000000000000\"\n",
    );
    fs::write(project.root.join("ash.lock"), lock_text).expect("signed lock mismatch");

    let mut command = ash_command();
    command.args(["check", project.main_path().to_str().expect("utf8")]);

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("lock signature mismatch"));
}

#[test]
fn check_rejects_git_protocol_lock_url_before_resolving_vendored_root() {
    let project = VendoredProject::new();
    write_helper_git_url(&project, "git://example.invalid/helper.git");

    let mut command = ash_command();
    command.args(["check", project.main_path().to_str().expect("utf8")]);

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("untrusted git protocol"))
        .stderr(predicate::str::contains("git"));
}

#[test]
fn run_rejects_http_protocol_lock_url_before_resolving_vendored_root() {
    let project = VendoredProject::new();
    write_helper_git_url(&project, "http://example.invalid/helper.git");

    let mut command = ash_command();
    command.args([
        "run",
        &format!("{}:main", project.main_path().to_str().expect("utf8")),
    ]);

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("untrusted git protocol"))
        .stderr(predicate::str::contains("http"));
}

#[test]
fn explicit_vendor_root_does_not_bypass_lock_commit_validation() {
    let project = VendoredProject::new();
    write_malformed_helper_commit(&project);
    fs::remove_file(project.root.join("ash.toml")).expect("remove manifest");

    let mut command = ash_command();
    command
        .env("ASH_DEP_ROOTS", project.root.join("vendor/ash"))
        .args(["check", project.main_path().to_str().expect("utf8")]);

    command.assert().failure().stderr(predicate::str::contains(
        "locked git commit must be a full 40-character commit hash",
    ));
}

#[test]
fn explicit_vendor_root_fails_closed_on_unrelated_malformed_lock_package_name() {
    let project = VendoredProject::new();
    write_unrelated_malformed_name_before_valid_helper(&project);
    fs::remove_file(project.root.join("ash.toml")).expect("remove manifest");

    let mut command = ash_command();
    command
        .env("ASH_DEP_ROOTS", project.root.join("vendor/ash"))
        .args(["check", project.main_path().to_str().expect("utf8")]);

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid package name"));
}

#[test]
fn explicit_vendor_root_fails_closed_on_unrelated_malformed_lock_commit() {
    let project = VendoredProject::new();
    write_unrelated_malformed_commit_before_valid_helper(&project);
    fs::remove_file(project.root.join("ash.toml")).expect("remove manifest");

    let mut command = ash_command();
    command
        .env("ASH_DEP_ROOTS", project.root.join("vendor/ash"))
        .args(["check", project.main_path().to_str().expect("utf8")]);

    command.assert().failure().stderr(predicate::str::contains(
        "locked git commit must be a full 40-character commit hash",
    ));
}

#[test]
fn explicit_vendor_package_root_fails_closed_on_unrelated_malformed_lock_package_name() {
    let project = VendoredProject::new();
    write_unrelated_malformed_name_before_valid_helper(&project);
    fs::remove_file(project.root.join("ash.toml")).expect("remove manifest");

    let mut command = ash_command();
    command
        .env("ASH_DEP_ROOTS", project.root.join("vendor/ash/helper"))
        .args(["check", project.main_path().to_str().expect("utf8")]);

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid package name"));
}

#[test]
fn explicit_vendor_package_root_fails_closed_on_unrelated_malformed_lock_commit() {
    let project = VendoredProject::new();
    write_unrelated_malformed_commit_before_valid_helper(&project);
    fs::remove_file(project.root.join("ash.toml")).expect("remove manifest");

    let mut command = ash_command();
    command
        .env("ASH_DEP_ROOTS", project.root.join("vendor/ash/helper"))
        .args(["check", project.main_path().to_str().expect("utf8")]);

    command.assert().failure().stderr(predicate::str::contains(
        "locked git commit must be a full 40-character commit hash",
    ));
}

#[test]
fn project_without_vendor_root_does_not_require_lockfile() {
    let temp = tempfile::tempdir().expect("temp project");
    let root = temp.path();
    let src = root.join("src");
    fs::create_dir_all(&src).expect("src");
    fs::write(root.join("ash.toml"), "[package]\nname = \"app\"\n").expect("manifest");
    fs::write(
        src.join("local.ash"),
        "pub type LocalToken = LocalToken { value: Int };\n",
    )
    .expect("local module");
    let main = src.join("main.ash");
    fs::write(
        &main,
        "use local::{LocalToken}\nworkflow main() -> LocalToken { ret LocalToken { value: 1 }; }\n",
    )
    .expect("main");

    let mut command = ash_command();
    command.args(["check", main.to_str().expect("utf8")]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("[OK]"));
}

#[test]
fn explicit_vendor_package_root_does_not_expose_top_level_modules() {
    let project = project_with_unlocked_top_level_module_inside_locked_package();

    let mut command = ash_command();
    command
        .env("ASH_DEP_ROOTS", project.root.join("vendor/ash/helper"))
        .args(["check", project.main_path().to_str().expect("utf8")]);

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("module 'util' not found"));
}

#[test]
fn explicit_cache_shaped_dependency_root_does_not_bypass_lock_boundary() {
    let temp = tempfile::tempdir().expect("temp project");
    let root = temp.path();
    let checkout = root.join("checkouts/helper-0000000000000000/not-a-locked-commit");
    fs::create_dir_all(&checkout).expect("checkout");
    fs::write(
        checkout.join("mod.ash"),
        "pub type HelperToken = HelperToken { value: Int };\n",
    )
    .expect("helper module");
    let main = root.join("main.ash");
    fs::write(
        &main,
        "use helper::{HelperToken}\nworkflow main() -> HelperToken { ret HelperToken { value: 7 }; }\n",
    )
    .expect("main");

    let mut command = ash_command();
    command
        .env("ASH_DEPENDENCY_ROOTS", checkout)
        .args(["check", main.to_str().expect("utf8")]);

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("module 'helper' not found"));
}

fn write_malformed_helper_commit(project: &VendoredProject) {
    fs::write(
        project.root.join("ash.lock"),
        format!("[[package]]\nname = \"helper\"\ngit = \"{HELPER_GIT_URL}\"\ncommit = \"short\"\n"),
    )
    .expect("malformed lock");
}

fn write_helper_git_url(project: &VendoredProject, git_url: &str) {
    fs::write(
        project.root.join("ash.lock"),
        format!(
            "[[package]]\nname = \"helper\"\ngit = \"{git_url}\"\ncommit = \"{HELPER_COMMIT}\"\n"
        ),
    )
    .expect("lock git url");
}

fn write_unrelated_malformed_name_before_valid_helper(project: &VendoredProject) {
    fs::write(
        project.root.join("ash.lock"),
        format!(
            "[[package]]\nname = \"../evil\"\ngit = \"file:///tmp/evil\"\ncommit = \"{HELPER_COMMIT}\"\n\n[[package]]\nname = \"helper\"\ngit = \"{HELPER_GIT_URL}\"\ncommit = \"{HELPER_COMMIT}\"\n",
        ),
    )
    .expect("malformed unrelated lock name");
}

fn write_unrelated_malformed_commit_before_valid_helper(project: &VendoredProject) {
    fs::write(
        project.root.join("ash.lock"),
        format!(
            "[[package]]\nname = \"unrelated\"\ngit = \"file:///tmp/unrelated\"\ncommit = \"short\"\n\n[[package]]\nname = \"helper\"\ngit = \"{HELPER_GIT_URL}\"\ncommit = \"{HELPER_COMMIT}\"\n",
        ),
    )
    .expect("malformed unrelated lock commit");
}

fn malformed_lock_project() -> VendoredProject {
    let project = VendoredProject::new();
    fs::write(
        project.root.join("ash.lock"),
        "[[package]]\nname = \"../escape\"\ngit = \"file:///tmp/escape\"\ncommit = \"0123456789abcdef0123456789abcdef01234567\"\n",
    )
    .expect("malformed lock");
    fs::write(
        project.root.join("src/main.ash"),
        "use escape::{EscapeToken}\nworkflow main() -> EscapeToken { ret EscapeToken { value: 9 }; }\n",
    )
    .expect("main");
    let escape = project.root.join("vendor/escape");
    fs::create_dir_all(&escape).expect("escape vendor dir");
    fs::write(
        escape.join("escape.ash"),
        "pub type EscapeToken = EscapeToken { value: Int };\n",
    )
    .expect("escape module");
    project
}

#[test]
fn unlocked_vendor_package_is_not_importable() {
    let project = VendoredProject::new();
    let evil = project.root.join("vendor/ash/evil");
    fs::create_dir_all(&evil).expect("evil vendor dir");
    fs::write(
        evil.join("mod.ash"),
        "pub type EvilToken = EvilToken { value: Int };\n",
    )
    .expect("evil module");
    fs::write(
        project.root.join("src/main.ash"),
        "use evil::{EvilToken}\nworkflow main() -> EvilToken { ret EvilToken { value: 9 }; }\n",
    )
    .expect("main");

    let mut command = ash_command();
    command.args(["check", project.main_path().to_str().expect("utf8")]);

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("module 'evil' not found"));
}

#[test]
fn run_does_not_import_unlocked_vendor_package() {
    let project = project_with_unlocked_evil_package();

    let mut command = ash_command();
    command.args([
        "run",
        &format!("{}:main", project.main_path().to_str().expect("utf8")),
    ]);

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("module 'evil' not found"));
}

fn project_with_unlocked_evil_package() -> VendoredProject {
    let project = VendoredProject::new();
    let evil = project.root.join("vendor/ash/evil");
    fs::create_dir_all(&evil).expect("evil vendor dir");
    fs::write(
        evil.join("mod.ash"),
        "pub type EvilToken = EvilToken { value: Int };\n",
    )
    .expect("evil module");
    fs::write(
        project.root.join("src/main.ash"),
        "use evil::{EvilToken}\nworkflow main() -> EvilToken { ret EvilToken { value: 9 }; }\n",
    )
    .expect("main");
    project
}

#[test]
fn unlocked_top_level_module_inside_locked_package_is_not_importable() {
    let project = VendoredProject::new();
    fs::write(
        project.root.join("vendor/ash/helper/util.ash"),
        "pub type UtilToken = UtilToken { value: Int };\n",
    )
    .expect("util module");
    fs::write(
        project.root.join("src/main.ash"),
        "use util::{UtilToken}\nworkflow main() -> UtilToken { ret UtilToken { value: 9 }; }\n",
    )
    .expect("main");

    let mut command = ash_command();
    command.args(["check", project.main_path().to_str().expect("utf8")]);

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("module 'util' not found"));
}

#[test]
fn run_does_not_import_top_level_module_inside_locked_package() {
    let project = project_with_unlocked_top_level_module_inside_locked_package();

    let mut command = ash_command();
    command.args([
        "run",
        &format!("{}:main", project.main_path().to_str().expect("utf8")),
    ]);

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("module 'util' not found"));
}

fn project_with_unlocked_top_level_module_inside_locked_package() -> VendoredProject {
    let project = VendoredProject::new();
    fs::write(
        project.root.join("vendor/ash/helper/util.ash"),
        "pub type UtilToken = UtilToken { value: Int };\n",
    )
    .expect("util module");
    fs::write(
        project.root.join("src/main.ash"),
        "use util::{UtilToken}\nworkflow main() -> UtilToken { ret UtilToken { value: 9 }; }\n",
    )
    .expect("main");
    project
}
