use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const HELPER_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const HELPER_GIT_URL: &str = "file:///tmp/helper";

struct VendoredProject {
    _temp: TempDir,
    root: PathBuf,
    main: PathBuf,
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
        .env_remove("ASH_LIBRARY_PATH");
    command
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

fn write_malformed_helper_commit(project: &VendoredProject) {
    fs::write(
        project.root.join("ash.lock"),
        format!("[[package]]\nname = \"helper\"\ngit = \"{HELPER_GIT_URL}\"\ncommit = \"short\"\n"),
    )
    .expect("malformed lock");
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
