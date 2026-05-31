use std::path::{Path, PathBuf};

use assert_cmd::Command;

mod support;

#[test]
fn task_989_gitignored_agents_state_can_change_during_source_install() {
    let source = support::source_workspace_fixture();
    write_and_commit_gitignore(source.path(), "/.agents/\n/.dirty\n");
    write_file(
        source.path().join(".agents/status/dashboard.json"),
        "before\n",
    );
    write_file(source.path().join(".dirty"), "ignored legacy sentinel\n");
    assert_git_clean(source.path());
    let expected_id = source_toolchain_id(source.path());
    let roots = support::xdg_fixture();
    let fake = FakeCargo::new(
        source.path(),
        FakeCargoAction::mutate_after_absent_check(
            ".agents/status/dashboard.json",
            ".agents/status/dashboard.json",
        ),
    );

    ashgrove_cmd()
        .args([
            "install",
            "--from",
            "source",
            "--path",
            source.path().to_str().expect("utf8 source"),
        ])
        .envs(roots.env())
        .envs(fake.env())
        .env("PATH", fake.path_env())
        .assert()
        .success();

    let observation = fake.observation();
    assert!(observation.contains("copy_absent=.agents/status/dashboard.json"));
    assert!(observation.contains("mutated=.agents/status/dashboard.json"));
    let record = install_record(&roots, &expected_id);
    assert_digest_field(&record, "source_payload_digest");
    assert_eq!(
        record
            .get("source_payload_digest_policy")
            .and_then(toml::Value::as_str),
        Some("source-root-v2-gitignore-local-state")
    );
    assert!(record.get("source_archive_digest").is_none());
}

#[test]
fn task_989_gitignored_nested_target_is_excluded_from_digest_and_copy() {
    let source = support::source_workspace_fixture();
    write_and_commit_gitignore(source.path(), "crates/ash-bench/target/\n");
    write_file(
        source.path().join("crates/ash-bench/target/generated.txt"),
        "before\n",
    );
    assert_git_clean(source.path());
    let expected_id = source_toolchain_id(source.path());
    let roots = support::xdg_fixture();
    let fake = FakeCargo::new(
        source.path(),
        FakeCargoAction::mutate_after_absent_check(
            "crates/ash-bench/target/generated.txt",
            "crates/ash-bench/target/generated.txt",
        ),
    );

    ashgrove_cmd()
        .args([
            "install",
            "--from",
            "source",
            "--path",
            source.path().to_str().expect("utf8 source"),
        ])
        .envs(roots.env())
        .envs(fake.env())
        .env("PATH", fake.path_env())
        .assert()
        .success();

    let observation = fake.observation();
    assert!(observation.contains("copy_absent=crates/ash-bench/target/generated.txt"));
    assert!(observation.contains("mutated=crates/ash-bench/target/generated.txt"));
    assert!(
        roots
            .toolchain(&expected_id)
            .join("manifest.toml")
            .is_file()
    );
}

#[test]
fn task_989_nonignored_payload_mutation_fails_before_publish() {
    let source = support::source_workspace_fixture();
    let expected_id = source.toolchain_id();
    let roots = support::xdg_fixture();
    let fake = FakeCargo::new(source.path(), FakeCargoAction::mutate("std/src/lib.ash"));

    ashgrove_cmd()
        .args([
            "install",
            "--from",
            "source",
            "--path",
            source.path().to_str().expect("utf8 source"),
        ])
        .envs(roots.env())
        .envs(fake.env())
        .env("PATH", fake.path_env())
        .assert()
        .failure()
        .stderr(predicates::str::contains("source-payload-changed"));

    assert!(!roots.toolchain(&expected_id).exists());
}

#[test]
fn task_989_nonignored_dirty_source_still_rejects_without_override() {
    let source = support::source_workspace_fixture();
    write_file(
        source.path().join("untracked-source-payload.txt"),
        "dirty\n",
    );
    let roots = support::xdg_fixture();

    ashgrove_cmd()
        .args([
            "install",
            "--from",
            "source",
            "--path",
            source.path().to_str().expect("utf8 source"),
        ])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains("dirty source"));
}

#[test]
fn task_989_update_from_source_uses_same_payload_policy_as_install() {
    let source = support::source_workspace_fixture();
    write_and_commit_gitignore(source.path(), "/.agents/\n/.dirty\n");
    write_file(
        source.path().join(".agents/status/dashboard.json"),
        "before\n",
    );
    write_file(source.path().join(".dirty"), "ignored legacy sentinel\n");
    assert_git_clean(source.path());
    let expected_id = source_toolchain_id(source.path());
    let roots = support::xdg_fixture();
    let fake = FakeCargo::new(
        source.path(),
        FakeCargoAction::mutate_after_absent_check(
            ".agents/status/dashboard.json",
            ".agents/status/dashboard.json",
        ),
    );

    ashgrove_cmd()
        .args([
            "update",
            "--to",
            &expected_id,
            "--from",
            "source",
            "--path",
            source.path().to_str().expect("utf8 source"),
        ])
        .envs(roots.env())
        .envs(fake.env())
        .env("PATH", fake.path_env())
        .assert()
        .success();

    let observation = fake.observation();
    assert!(observation.contains("copy_absent=.agents/status/dashboard.json"));
    let record = install_record(&roots, &expected_id);
    assert_digest_field(&record, "source_payload_digest");
    assert!(record.get("source_archive_digest").is_none());
}

#[test]
fn task_989_source_archive_digest_policy_does_not_use_source_root_ignores() {
    let parent = support::source_workspace_fixture();
    let source_path = parent.path().join("archive");
    support::create_source_workspace(&source_path);
    write_file(
        source_path.join("release-source.toml"),
        "schema_version = 1\norigin_commit = \"abcdef1234567890\"\n\n[attestation]\norigin_commit = \"abcdef1234567890\"\n",
    );
    write_file(source_path.join(".gitignore"), "/.agents/\n");
    write_file(
        source_path.join(".agents/status/dashboard.json"),
        "archive local state\n",
    );
    let roots = support::xdg_fixture();
    let expected_id = "ash-0.1.0+source.abcdef123456";
    let fake = FakeCargo::new(
        &source_path,
        FakeCargoAction::check_present(".agents/status/dashboard.json"),
    );

    ashgrove_cmd()
        .args([
            "install",
            "--from",
            "source",
            "--path",
            source_path.to_str().expect("utf8 source"),
        ])
        .envs(roots.env())
        .envs(fake.env())
        .env("PATH", fake.path_env())
        .assert()
        .success();

    let observation = fake.observation();
    assert!(observation.contains("copy_present=.agents/status/dashboard.json"));
    let record = install_record(&roots, expected_id);
    assert_digest_field(&record, "source_archive_digest");
    assert!(record.get("source_payload_digest").is_none());
    assert!(record.get("source_payload_digest_policy").is_none());
}

#[test]
fn task_989_source_shaped_archive_requires_attestation_inside_unrelated_git_worktree() {
    let parent = support::source_workspace_fixture();
    let source_path = parent.path().join("archive-without-attestation");
    support::create_source_workspace(&source_path);
    write_file(
        source_path.join("release-source.toml"),
        &format!(
            "schema_version = 1\norigin_commit = \"{}\"\n",
            parent.revision()
        ),
    );
    let roots = support::xdg_fixture();
    let fake = FakeCargo::new(&source_path, FakeCargoAction::none());

    ashgrove_cmd()
        .args([
            "install",
            "--from",
            "source",
            "--path",
            source_path.to_str().expect("utf8 source"),
            "--allow-dirty-source",
        ])
        .envs(roots.env())
        .envs(fake.env())
        .env("PATH", fake.path_env())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "source archive attestation evidence is required",
        ));
}

#[test]
fn task_989_git_payload_membership_failure_fails_closed() {
    let source = support::source_workspace_fixture();
    let roots = support::xdg_fixture();
    let fake_git = tempfile::tempdir().expect("fake git bin");
    support::write_tool_script(
        &fake_git.path().join("git"),
        &format!(
            r#"case "$*" in
  "rev-parse HEAD") printf '%s\n' '{}' ;;
  "status --porcelain") exit 0 ;;
  "ls-files --cached --others --exclude-standard -z") printf '%s\n' 'membership unavailable' >&2; exit 2 ;;
  "config --get remote.origin.url") exit 1 ;;
  *) printf 'unexpected git args: %s\n' "$*" >&2; exit 3 ;;
esac
"#,
            source.revision()
        ),
    );
    let path = format!(
        "{}:{}",
        fake_git.path().display(),
        std::env::var("PATH").expect("PATH")
    );

    ashgrove_cmd()
        .args([
            "install",
            "--from",
            "source",
            "--path",
            source.path().to_str().expect("utf8 source"),
        ])
        .envs(roots.env())
        .env("PATH", path)
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "source payload membership failed",
        ));
}

#[test]
fn task_989_git_worktree_classification_failure_fails_closed_before_non_git_fallback() {
    let parent = support::source_workspace_fixture();
    let source_path = parent.path().join("nested-source-root");
    support::create_source_workspace(&source_path);
    let roots = support::xdg_fixture();
    let fake_bin = tempfile::tempdir().expect("fake tools bin");
    let observation = fake_bin.path().join("cargo-observation.txt");
    support::write_tool_script(
        &fake_bin.path().join("git"),
        r#"case "$*" in
  "rev-parse --is-inside-work-tree") printf '%s\n' 'classification unavailable' >&2; exit 2 ;;
  "rev-parse HEAD") printf '%s\n' 'identity unavailable' >&2; exit 2 ;;
  "status --porcelain") exit 0 ;;
  "config --get remote.origin.url") exit 1 ;;
  *) printf 'unexpected git args: %s\n' "$*" >&2; exit 3 ;;
esac
"#,
    );
    support::write_tool_script(
        &fake_bin.path().join("cargo"),
        &format!(
            r#"printf 'cargo-ran\n' > '{}'
mkdir -p "$CARGO_TARGET_DIR/debug"
for tool in ash ashgrove; do
  printf '#!/bin/sh\nexit 0\n' > "$CARGO_TARGET_DIR/debug/$tool"
  chmod +x "$CARGO_TARGET_DIR/debug/$tool"
done
"#,
            observation.display()
        ),
    );
    let path = format!(
        "{}:{}",
        fake_bin.path().display(),
        std::env::var("PATH").expect("PATH")
    );

    ashgrove_cmd()
        .args([
            "install",
            "--from",
            "source",
            "--path",
            source_path.to_str().expect("utf8 source"),
            "--allow-unidentified-source",
        ])
        .envs(roots.env())
        .env("PATH", path)
        .assert()
        .failure()
        .stderr(predicates::str::contains("git work tree detection failed"));
    assert!(!observation.exists());
}

#[test]
fn task_989_non_git_source_root_builtin_local_state_is_excluded_from_digest_and_copy() {
    let source = support::unidentified_source_workspace_fixture();
    write_file(
        source.path().join(".agents/status/dashboard.json"),
        "non-git local state\n",
    );
    let roots = support::xdg_fixture();
    let fake = FakeCargo::new(
        source.path(),
        FakeCargoAction::mutate_after_absent_check(
            ".agents/status/dashboard.json",
            ".agents/status/dashboard.json",
        ),
    );

    ashgrove_cmd()
        .args([
            "install",
            "--from",
            "source",
            "--path",
            source.path().to_str().expect("utf8 source"),
            "--allow-unidentified-source",
        ])
        .envs(roots.env())
        .envs(fake.env())
        .env("PATH", fake.path_env())
        .assert()
        .success();

    let observation = fake.observation();
    assert!(observation.contains("copy_absent=.agents/status/dashboard.json"));
    assert!(observation.contains("mutated=.agents/status/dashboard.json"));
    let expected_id = source_toolchain_id_for_unidentified(source.path());
    let record = install_record(&roots, &expected_id);
    assert_digest_field(&record, "source_payload_digest");
    assert_eq!(
        record
            .get("source_payload_digest_policy")
            .and_then(toml::Value::as_str),
        Some("source-root-v2-gitignore-local-state")
    );
    assert!(record.get("source_archive_digest").is_none());
}

#[test]
fn task_991_ignored_original_root_cargo_lock_does_not_force_locked_build() {
    let source = support::source_workspace_fixture();
    write_and_commit_gitignore(source.path(), "/Cargo.lock\n");
    write_file(source.path().join("Cargo.lock"), "# ignored local lock\n");
    assert_git_clean(source.path());
    let roots = support::xdg_fixture();
    let fake = FakeCargo::new(source.path(), FakeCargoAction::check_absent("Cargo.lock"));

    ashgrove_cmd()
        .args([
            "install",
            "--from",
            "source",
            "--path",
            source.path().to_str().expect("utf8 source"),
        ])
        .envs(roots.env())
        .envs(fake.env())
        .env("PATH", fake.path_env())
        .assert()
        .success();

    let observation = fake.observation();
    assert!(
        !cargo_argv_contains(&observation, "--locked"),
        "ignored original-root Cargo.lock must not force --locked in isolated source build\n{observation}"
    );
}

#[test]
fn task_991_tracked_copied_cargo_lock_keeps_locked_build() {
    let source = support::source_workspace_fixture();
    write_file(source.path().join("Cargo.lock"), "# tracked lock\n");
    run_git(source.path(), &["add", "Cargo.lock"]);
    run_git(
        source.path(),
        &[
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-m",
            "add tracked cargo lock",
        ],
    );
    assert_git_clean(source.path());
    let roots = support::xdg_fixture();
    let fake = FakeCargo::new(source.path(), FakeCargoAction::check_present("Cargo.lock"));

    ashgrove_cmd()
        .args([
            "install",
            "--from",
            "source",
            "--path",
            source.path().to_str().expect("utf8 source"),
        ])
        .envs(roots.env())
        .envs(fake.env())
        .env("PATH", fake.path_env())
        .assert()
        .success();

    let observation = fake.observation();
    assert!(
        cargo_argv_contains(&observation, "--locked"),
        "tracked copied Cargo.lock must keep --locked in isolated source build\n{observation}"
    );
}

struct FakeCargo {
    bin: tempfile::TempDir,
    observation: PathBuf,
    source: PathBuf,
    action: FakeCargoAction,
}

impl FakeCargo {
    fn new(source: &Path, action: FakeCargoAction) -> Self {
        let bin = tempfile::tempdir().expect("fake cargo bin");
        let observation = bin.path().join("observation.txt");
        support::write_tool_script(
            &bin.path().join("cargo"),
            r#"set -eu
: "${TASK_989_SOURCE_ROOT:?}"
: "${TASK_989_OBSERVATION:?}"
printf 'pwd=%s\n' "$PWD" > "$TASK_989_OBSERVATION"
printf 'argv=%s\n' "$*" >> "$TASK_989_OBSERVATION"
if [ -n "${TASK_989_COPY_ABSENT_REL:-}" ]; then
  if [ -e "$PWD/$TASK_989_COPY_ABSENT_REL" ]; then
    printf 'copy_unexpected=%s\n' "$TASK_989_COPY_ABSENT_REL" >> "$TASK_989_OBSERVATION"
    exit 41
  fi
  printf 'copy_absent=%s\n' "$TASK_989_COPY_ABSENT_REL" >> "$TASK_989_OBSERVATION"
fi
if [ -n "${TASK_989_COPY_PRESENT_REL:-}" ]; then
  if [ ! -e "$PWD/$TASK_989_COPY_PRESENT_REL" ]; then
    printf 'copy_missing=%s\n' "$TASK_989_COPY_PRESENT_REL" >> "$TASK_989_OBSERVATION"
    exit 42
  fi
  printf 'copy_present=%s\n' "$TASK_989_COPY_PRESENT_REL" >> "$TASK_989_OBSERVATION"
fi
if [ -n "${TASK_989_MUTATE_REL:-}" ]; then
  mkdir -p "$TASK_989_SOURCE_ROOT/$(dirname "$TASK_989_MUTATE_REL")"
  printf 'mutated by fake cargo\n' >> "$TASK_989_SOURCE_ROOT/$TASK_989_MUTATE_REL"
  printf 'mutated=%s\n' "$TASK_989_MUTATE_REL" >> "$TASK_989_OBSERVATION"
fi
mkdir -p "$CARGO_TARGET_DIR/debug"
for tool in ash ashgrove; do
  printf '#!/bin/sh\nexit 0\n' > "$CARGO_TARGET_DIR/debug/$tool"
  chmod +x "$CARGO_TARGET_DIR/debug/$tool"
done
"#,
        );
        Self {
            bin,
            observation,
            source: source.to_path_buf(),
            action,
        }
    }

    fn env(&self) -> Vec<(&'static str, String)> {
        let mut env = vec![
            ("TASK_989_SOURCE_ROOT", self.source.display().to_string()),
            (
                "TASK_989_OBSERVATION",
                self.observation.display().to_string(),
            ),
        ];
        if let Some(rel) = &self.action.copy_absent_rel {
            env.push(("TASK_989_COPY_ABSENT_REL", rel.clone()));
        }
        if let Some(rel) = &self.action.copy_present_rel {
            env.push(("TASK_989_COPY_PRESENT_REL", rel.clone()));
        }
        if let Some(rel) = &self.action.mutate_rel {
            env.push(("TASK_989_MUTATE_REL", rel.clone()));
        }
        env
    }

    fn path_env(&self) -> String {
        format!(
            "{}:{}",
            self.bin.path().display(),
            std::env::var("PATH").expect("PATH")
        )
    }

    fn observation(&self) -> String {
        std::fs::read_to_string(&self.observation).expect("fake cargo observation")
    }
}

struct FakeCargoAction {
    copy_absent_rel: Option<String>,
    copy_present_rel: Option<String>,
    mutate_rel: Option<String>,
}

impl FakeCargoAction {
    fn none() -> Self {
        Self {
            copy_absent_rel: None,
            copy_present_rel: None,
            mutate_rel: None,
        }
    }

    fn mutate(rel: &str) -> Self {
        Self {
            copy_absent_rel: None,
            copy_present_rel: None,
            mutate_rel: Some(rel.to_string()),
        }
    }

    fn mutate_after_absent_check(absent_rel: &str, mutate_rel: &str) -> Self {
        Self {
            copy_absent_rel: Some(absent_rel.to_string()),
            copy_present_rel: None,
            mutate_rel: Some(mutate_rel.to_string()),
        }
    }

    fn check_absent(rel: &str) -> Self {
        Self {
            copy_absent_rel: Some(rel.to_string()),
            copy_present_rel: None,
            mutate_rel: None,
        }
    }

    fn check_present(rel: &str) -> Self {
        Self {
            copy_absent_rel: None,
            copy_present_rel: Some(rel.to_string()),
            mutate_rel: None,
        }
    }
}

fn cargo_argv_contains(observation: &str, expected: &str) -> bool {
    observation
        .lines()
        .find_map(|line| line.strip_prefix("argv="))
        .is_some_and(|argv| argv.split_whitespace().any(|arg| arg == expected))
}

fn ashgrove_cmd() -> Command {
    Command::cargo_bin("ashgrove").expect("ashgrove binary")
}

fn write_and_commit_gitignore(source: &Path, contents: &str) {
    write_file(source.join(".gitignore"), contents);
    run_git(source, &["add", ".gitignore"]);
    run_git(
        source,
        &[
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-m",
            "add task 989 gitignore",
        ],
    );
}

fn source_toolchain_id(source: &Path) -> String {
    let rev = git_output(source, &["rev-parse", "HEAD"]);
    format!("ash-0.1.0+source.{}", &rev[..12])
}

fn source_toolchain_id_for_unidentified(source: &Path) -> String {
    let digest = sha256_hex(source.display().to_string().as_bytes());
    format!("ash-0.1.0+source.unidentified{}", &digest[..12])
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    format!("{:x}", Sha256::digest(bytes))
}

fn assert_git_clean(source: &Path) {
    assert_eq!(git_output(source, &["status", "--porcelain"]), "");
}

fn run_git(source: &Path, args: &[&str]) {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(source)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {} failed\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_output(source: &Path, args: &[&str]) -> String {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(source)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {} failed\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("git stdout utf8")
        .trim()
        .to_string()
}

fn write_file(path: PathBuf, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent");
    }
    std::fs::write(path, contents).expect("write file");
}

fn install_record(roots: &support::XdgFixture, id: &str) -> toml::map::Map<String, toml::Value> {
    let text =
        std::fs::read_to_string(roots.toolchain(id).join("install-record.toml")).expect("record");
    toml::from_str::<toml::Value>(&text)
        .expect("parse record")
        .as_table()
        .expect("record table")
        .clone()
}

fn assert_digest_field(record: &toml::map::Map<String, toml::Value>, key: &str) {
    let digest = record
        .get(key)
        .and_then(toml::Value::as_str)
        .unwrap_or_else(|| panic!("missing {key}"));
    let hex = digest.strip_prefix("sha256:").expect("sha256 prefix");
    assert_eq!(hex.len(), 64);
    assert!(hex.chars().all(|ch| ch.is_ascii_hexdigit()));
}
