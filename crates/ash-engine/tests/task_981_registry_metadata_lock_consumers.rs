//! Registry metadata lockfile consumer coverage for TASK-981.

use ash_engine::module_loader::load_ordinary_file;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;

#[tokio::test]
async fn task_981_engine_accepts_registry_metadata_without_registry_resolution() {
    let project = tempfile::tempdir().expect("project");
    let cache = tempfile::tempdir().expect("xdg cache");
    let dep = tempfile::tempdir().expect("git dep");
    let src = project.path().join("src");
    std::fs::create_dir_all(&src).expect("src");
    let git_url = format!("file://{}", dep.path().display());
    let commit = write_fetched_helper_checkout(cache.path(), dep.path(), &git_url);
    std::fs::write(
        project.path().join("ash.toml"),
        "[package]\nname = \"app\"\n",
    )
    .expect("manifest");
    std::fs::write(
        project.path().join("ash.lock"),
        format!(
            "[[package]]\nname = \"helper\"\nversion = \"0.2.0\"\nregistry = \"ash.test\"\nsource = \"git+{git_url}\"\nrequested = {{ tag = \"v1\" }}\nresolved = {{ rev = \"{commit}\" }}\n",
        ),
    )
    .expect("lock");
    let main = src.join("main.ash");
    std::fs::write(
        &main,
        "use helper::{HelperToken}\nfn main() -> HelperToken { return HelperToken { value: 7 }; }\n",
    )
    .expect("main");

    let cache_root = cache.path().to_path_buf();
    let loaded = temp_env::async_with_vars(
        [
            ("XDG_CACHE_HOME", Some(cache_root.as_os_str())),
            ("ASH_DEP_ROOTS", None::<&std::ffi::OsStr>),
            ("ASH_DEPENDENCY_ROOTS", None::<&std::ffi::OsStr>),
            ("ASH_LIBRARY_PATH", None::<&std::ffi::OsStr>),
        ],
        async { load_ordinary_file(&main) },
    )
    .await
    .expect("registry metadata in ash.lock should not trigger hosted registry resolution");

    assert!(
        loaded
            .imported_type_defs
            .iter()
            .any(|def| def.name == "HelperToken")
    );
}

#[tokio::test]
async fn task_981_engine_rejects_non_git_source_even_with_redundant_git() {
    let fixture = poisoned_lock_fixture(
        "registry+https://registry.example.invalid/helper",
        RedundantGit::Valid,
    );
    let cache_root = fixture.cache.path().to_path_buf();
    let error = temp_env::async_with_vars(
        [
            ("XDG_CACHE_HOME", Some(cache_root.as_os_str())),
            ("ASH_DEP_ROOTS", None::<&std::ffi::OsStr>),
            ("ASH_DEPENDENCY_ROOTS", None::<&std::ffi::OsStr>),
            ("ASH_LIBRARY_PATH", None::<&std::ffi::OsStr>),
        ],
        async { load_ordinary_file(&fixture.main) },
    )
    .await
    .expect_err("non-git source must fail closed before redundant git cache lookup");

    assert!(
        error
            .to_string()
            .contains("ash.lock package source must be git+ URL"),
        "{error}"
    );
}

#[tokio::test]
async fn task_981_engine_rejects_source_git_mismatch() {
    let fixture = poisoned_lock_fixture(
        "git+file:///tmp/ash-task-981-other-helper",
        RedundantGit::Valid,
    );
    let cache_root = fixture.cache.path().to_path_buf();
    let error = temp_env::async_with_vars(
        [
            ("XDG_CACHE_HOME", Some(cache_root.as_os_str())),
            ("ASH_DEP_ROOTS", None::<&std::ffi::OsStr>),
            ("ASH_DEPENDENCY_ROOTS", None::<&std::ffi::OsStr>),
            ("ASH_LIBRARY_PATH", None::<&std::ffi::OsStr>),
        ],
        async { load_ordinary_file(&fixture.main) },
    )
    .await
    .expect_err("source/git mismatch must fail closed before redundant git cache lookup");

    assert!(
        error
            .to_string()
            .contains("ash.lock package source does not match redundant git URL"),
        "{error}"
    );
}

#[tokio::test]
async fn task_981_engine_rejects_vendored_non_git_source_even_with_redundant_git() {
    let fixture = poisoned_vendor_lock_fixture(
        "registry+https://registry.example.invalid/helper",
        RedundantGit::Valid,
    );
    let cache_root = fixture.cache.path().to_path_buf();
    let error = temp_env::async_with_vars(
        [
            ("XDG_CACHE_HOME", Some(cache_root.as_os_str())),
            ("ASH_DEP_ROOTS", None::<&std::ffi::OsStr>),
            ("ASH_DEPENDENCY_ROOTS", None::<&std::ffi::OsStr>),
            ("ASH_LIBRARY_PATH", None::<&std::ffi::OsStr>),
        ],
        async { load_ordinary_file(&fixture.main) },
    )
    .await
    .expect_err("non-git source must fail closed even when vendor package exists");

    assert!(
        error
            .to_string()
            .contains("ash.lock package source must be git+ URL"),
        "{error}"
    );
}

#[tokio::test]
async fn task_981_engine_rejects_vendored_source_git_mismatch() {
    let fixture = poisoned_vendor_lock_fixture(
        "git+file:///tmp/ash-task-981-other-helper",
        RedundantGit::Valid,
    );
    let cache_root = fixture.cache.path().to_path_buf();
    let error = temp_env::async_with_vars(
        [
            ("XDG_CACHE_HOME", Some(cache_root.as_os_str())),
            ("ASH_DEP_ROOTS", None::<&std::ffi::OsStr>),
            ("ASH_DEPENDENCY_ROOTS", None::<&std::ffi::OsStr>),
            ("ASH_LIBRARY_PATH", None::<&std::ffi::OsStr>),
        ],
        async { load_ordinary_file(&fixture.main) },
    )
    .await
    .expect_err("source/git mismatch must fail closed even when vendor package exists");

    assert!(
        error
            .to_string()
            .contains("ash.lock package source does not match redundant git URL"),
        "{error}"
    );
}

#[tokio::test]
async fn task_981_engine_rejects_explicit_vendor_root_non_git_source() {
    let fixture = poisoned_vendor_lock_fixture(
        "registry+https://registry.example.invalid/helper",
        RedundantGit::Valid,
    );
    let cache_root = fixture.cache.path().to_path_buf();
    let vendor_root = fixture.vendor_root.clone();
    let error = temp_env::async_with_vars(
        [
            ("XDG_CACHE_HOME", Some(cache_root.as_os_str())),
            ("ASH_DEP_ROOTS", Some(vendor_root.as_os_str())),
            ("ASH_DEPENDENCY_ROOTS", None::<&std::ffi::OsStr>),
            ("ASH_LIBRARY_PATH", None::<&std::ffi::OsStr>),
        ],
        async { load_ordinary_file(&fixture.main) },
    )
    .await
    .expect_err("explicit vendor roots must validate lock source before use");

    assert!(
        error
            .to_string()
            .contains("ash.lock package source must be git+ URL"),
        "{error}"
    );
}

#[derive(Clone, Copy)]
enum RedundantGit {
    Valid,
}

struct PoisonedLockFixture {
    _project: tempfile::TempDir,
    cache: tempfile::TempDir,
    _dep: tempfile::TempDir,
    main: std::path::PathBuf,
}

struct PoisonedVendorLockFixture {
    _project: tempfile::TempDir,
    cache: tempfile::TempDir,
    _dep: tempfile::TempDir,
    vendor_root: std::path::PathBuf,
    main: std::path::PathBuf,
}

fn poisoned_lock_fixture(source: &str, redundant_git: RedundantGit) -> PoisonedLockFixture {
    let project = tempfile::tempdir().expect("project");
    let cache = tempfile::tempdir().expect("xdg cache");
    let dep = tempfile::tempdir().expect("git dep");
    let src = project.path().join("src");
    std::fs::create_dir_all(&src).expect("src");
    let git_url = format!("file://{}", dep.path().display());
    let commit = write_fetched_helper_checkout(cache.path(), dep.path(), &git_url);
    let redundant_git = match redundant_git {
        RedundantGit::Valid => git_url.as_str(),
    };
    std::fs::write(
        project.path().join("ash.toml"),
        "[package]\nname = \"app\"\n",
    )
    .expect("manifest");
    std::fs::write(
        project.path().join("ash.lock"),
        format!(
            "[[package]]\nname = \"helper\"\nversion = \"0.2.0\"\nsource = \"{source}\"\ngit = \"{redundant_git}\"\ncommit = \"{commit}\"\n",
        ),
    )
    .expect("lock");
    let main = src.join("main.ash");
    std::fs::write(
        &main,
        "use helper::{HelperToken}\nfn main() -> HelperToken { return HelperToken { value: 7 }; }\n",
    )
    .expect("main");

    PoisonedLockFixture {
        _project: project,
        cache,
        _dep: dep,
        main,
    }
}

fn poisoned_vendor_lock_fixture(
    source: &str,
    redundant_git: RedundantGit,
) -> PoisonedVendorLockFixture {
    let project = tempfile::tempdir().expect("project");
    let cache = tempfile::tempdir().expect("xdg cache");
    let dep = tempfile::tempdir().expect("git dep");
    let src = project.path().join("src");
    std::fs::create_dir_all(&src).expect("src");
    let git_url = format!("file://{}", dep.path().display());
    let commit = init_helper_git_dep(dep.path());
    let redundant_git = match redundant_git {
        RedundantGit::Valid => git_url.as_str(),
    };
    std::fs::write(
        project.path().join("ash.toml"),
        "[package]\nname = \"app\"\n",
    )
    .expect("manifest");
    std::fs::write(
        project.path().join("ash.lock"),
        format!(
            "[[package]]\nname = \"helper\"\nversion = \"0.2.0\"\nsource = \"{source}\"\ngit = \"{redundant_git}\"\ncommit = \"{commit}\"\n",
        ),
    )
    .expect("lock");
    let vendor_package = project.path().join("vendor/ash/helper");
    std::fs::create_dir_all(&vendor_package).expect("vendor package");
    std::fs::write(
        vendor_package.join("mod.ash"),
        "pub type HelperToken = HelperToken { value: Int };\n",
    )
    .expect("vendored helper module");
    let main = src.join("main.ash");
    std::fs::write(
        &main,
        "use helper::{HelperToken}\nfn main() -> HelperToken { return HelperToken { value: 7 }; }\n",
    )
    .expect("main");

    PoisonedVendorLockFixture {
        vendor_root: project.path().join("vendor/ash"),
        _project: project,
        cache,
        _dep: dep,
        main,
    }
}

fn write_fetched_helper_checkout(
    cache: &std::path::Path,
    dep: &std::path::Path,
    url: &str,
) -> String {
    let commit = init_helper_git_dep(dep);
    let checkout = cache
        .join("ash/git/checkouts")
        .join(format!("helper-{}", git_url_digest(url)))
        .join(&commit);
    run_git(
        std::path::Path::new("."),
        &[
            "clone",
            dep.to_str().expect("utf8"),
            checkout.to_str().expect("utf8"),
        ],
    );
    commit
}

fn init_helper_git_dep(dep: &std::path::Path) -> String {
    std::fs::write(
        dep.join("mod.ash"),
        "pub type HelperToken = HelperToken { value: Int };\n",
    )
    .expect("helper module");
    run_git(dep, &["init"]);
    run_git(dep, &["config", "user.email", "ash@example.invalid"]);
    run_git(dep, &["config", "user.name", "Ash Test"]);
    run_git(dep, &["add", "."]);
    run_git(dep, &["commit", "-m", "initial"]);
    run_git(dep, &["tag", "v1"]);
    git_output(dep, &["rev-parse", "HEAD"]).trim().to_string()
}

fn git_url_digest(url: &str) -> String {
    let digest = Sha256::digest(url.as_bytes());
    let mut out = String::new();
    for byte in &digest[..8] {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

fn run_git(dir: &std::path::Path, args: &[&str]) {
    let status = std::process::Command::new("git")
        .args(["-c", "commit.gpgsign=false", "-c", "tag.gpgSign=false"])
        .args(args)
        .current_dir(dir)
        .status()
        .expect("git");
    assert!(status.success(), "git {args:?}");
}

fn git_output(dir: &std::path::Path, args: &[&str]) -> String {
    let output = std::process::Command::new("git")
        .args(["-c", "commit.gpgsign=false", "-c", "tag.gpgSign=false"])
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git");
    assert!(output.status.success(), "git {args:?}");
    String::from_utf8(output.stdout).expect("git stdout")
}
