use ashgrove::rewrite_project_manifest_preserving_trust_metadata;
use assert_cmd::Command;
use predicates::prelude::*;

mod support;

#[test]
fn task_983_lock_rewrite_preserves_nested_trust_and_signing_metadata() {
    let project = support::project_fixture();
    let dep = support::git_dep_fixture();
    std::fs::write(
        project.path().join("ash.toml"),
        format!(
            "[package]\nname = \"app\"\n\n[dependencies.dep]\nversion = \"0.2.0\"\nregistry = \"ash.test\"\ngit = \"{}\"\ntag = \"v1\"\n",
            dep.url()
        ),
    )
    .expect("manifest");
    std::fs::write(
        project.path().join("ash.lock"),
        r#"
version = 1

[trust]
policy = "preserve-only"

[trust.signing]
mode = "optional"
keyring = "ash-test"

[trust.attestations.release]
predicate = "https://example.invalid/ash-release"

[signing]
algorithm = "minisign"

[signing.release]
signature = "opaque-signature"

[[package]]
name = "stale"
git = "file:///tmp/stale"
commit = "0123456789abcdef0123456789abcdef01234567"
"#,
    )
    .expect("stale lock");
    let roots = support::xdg_fixture();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["lock", "--project", project.path().to_str().expect("utf8")])
        .envs(roots.env())
        .assert()
        .success();

    let lock_text = std::fs::read_to_string(project.path().join("ash.lock")).expect("lock");
    let lock: toml::Value = toml::from_str(&lock_text).expect("serialized lock TOML");
    assert_eq!(
        lock.get("trust")
            .and_then(toml::Value::as_table)
            .and_then(|trust| trust.get("signing"))
            .and_then(toml::Value::as_table)
            .and_then(|signing| signing.get("mode"))
            .and_then(toml::Value::as_str),
        Some("optional")
    );
    assert_eq!(
        lock.get("trust")
            .and_then(toml::Value::as_table)
            .and_then(|trust| trust.get("attestations"))
            .and_then(toml::Value::as_table)
            .and_then(|attestations| attestations.get("release"))
            .and_then(toml::Value::as_table)
            .and_then(|release| release.get("predicate"))
            .and_then(toml::Value::as_str),
        Some("https://example.invalid/ash-release")
    );
    assert_eq!(
        lock.get("signing")
            .and_then(toml::Value::as_table)
            .and_then(|signing| signing.get("release"))
            .and_then(toml::Value::as_table)
            .and_then(|release| release.get("signature"))
            .and_then(toml::Value::as_str),
        Some("opaque-signature")
    );
    let package = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .and_then(|packages| packages.first())
        .expect("rewritten package");
    assert_eq!(
        package.get("version").and_then(toml::Value::as_str),
        Some("0.2.0")
    );
    assert_eq!(
        package.get("registry").and_then(toml::Value::as_str),
        Some("ash.test")
    );
    assert_eq!(
        package
            .get("source")
            .and_then(toml::Value::as_str)
            .map(|source| source.strip_prefix("git+").expect("git source")),
        Some(dep.url().as_str())
    );
    assert_eq!(
        package
            .get("requested")
            .and_then(toml::Value::as_table)
            .and_then(|requested| requested.get("tag"))
            .and_then(toml::Value::as_str),
        Some("v1")
    );
    assert_eq!(
        package
            .get("resolved")
            .and_then(toml::Value::as_table)
            .and_then(|resolved| resolved.get("rev"))
            .and_then(toml::Value::as_str),
        Some(dep.commit("v1").as_str())
    );
}

#[test]
fn task_983_manifest_rewrite_preserves_unknown_trust_tables() {
    let project = support::project_fixture();
    std::fs::write(
        project.path().join("ash.toml"),
        r#"
[package]
name = "app"

[toolchain]
ash = "ash-0.1.0+test.source.manifest983"

[trust]
policy = "future-preserve"

[trust.signing]
unknown_mode = "opaque"

[trust.signing.required_evidence]
predicate = "custom"

[signing.release]
certificate = "opaque-cert"
"#,
    )
    .expect("manifest");

    rewrite_project_manifest_preserving_trust_metadata(project.path()).expect("rewrite manifest");

    let manifest_text = std::fs::read_to_string(project.path().join("ash.toml")).expect("manifest");
    let manifest: toml::Value = toml::from_str(&manifest_text).expect("manifest TOML");
    assert_eq!(
        manifest
            .get("trust")
            .and_then(toml::Value::as_table)
            .and_then(|trust| trust.get("signing"))
            .and_then(toml::Value::as_table)
            .and_then(|signing| signing.get("unknown_mode"))
            .and_then(toml::Value::as_str),
        Some("opaque")
    );
    assert_eq!(
        manifest
            .get("trust")
            .and_then(toml::Value::as_table)
            .and_then(|trust| trust.get("signing"))
            .and_then(toml::Value::as_table)
            .and_then(|signing| signing.get("required_evidence"))
            .and_then(toml::Value::as_table)
            .and_then(|evidence| evidence.get("predicate"))
            .and_then(toml::Value::as_str),
        Some("custom")
    );
    assert_eq!(
        manifest
            .get("signing")
            .and_then(toml::Value::as_table)
            .and_then(|signing| signing.get("release"))
            .and_then(toml::Value::as_table)
            .and_then(|release| release.get("certificate"))
            .and_then(toml::Value::as_str),
        Some("opaque-cert")
    );
}

#[test]
fn task_983_diagnostics_distinguish_trust_preservation_from_enforcement() {
    let project = support::project_fixture();
    let dep = support::git_dep_fixture();
    std::fs::write(
        project.path().join("ash.toml"),
        format!(
            "[package]\nname = \"app\"\n\n[dependencies.dep]\ngit = \"{}\"\ntag = \"v1\"\n",
            dep.url()
        ),
    )
    .expect("manifest");
    std::fs::write(
        project.path().join("ash.lock"),
        "[trust]\npolicy = \"future-preserve\"\n[signing]\nrequired = true\n",
    )
    .expect("lock trust metadata");
    let roots = support::xdg_fixture();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["lock", "--project", project.path().to_str().expect("utf8")])
        .envs(roots.env())
        .assert()
        .success()
        .stdout(predicate::str::contains("preserved trust metadata"))
        .stdout(predicate::str::contains(
            "mandatory trust enforcement is not performed",
        ))
        .stdout(predicate::str::contains("trust enforced").not());
}
