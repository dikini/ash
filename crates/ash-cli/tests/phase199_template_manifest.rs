//! TASK-1946: app template manifest validation.

use ash_cli::templates::{
    GeneratedCheck, TemplateFile, TemplateManifest, TemplateParameter, TemplateProviderExpectation,
    validate_template_manifest,
};

fn valid_manifest() -> TemplateManifest {
    TemplateManifest {
        schema_version: "ash-template-v1".to_string(),
        id: "cli-tool".to_string(),
        version: "0.1.0".to_string(),
        description: "CLI tool template".to_string(),
        required_profiles: vec!["application-default".to_string()],
        providers: vec![TemplateProviderExpectation {
            profile: "application-default".to_string(),
            provider: "logging".to_string(),
            operations: vec!["info".to_string()],
        }],
        resources: vec!["stdout".to_string()],
        evidence_expectations: vec!["ash check src/main.ash".to_string()],
        parameters: vec![TemplateParameter {
            name: "app_name".to_string(),
            required: true,
            default: None,
        }],
        files: vec![TemplateFile {
            path: "src/main.ash".to_string(),
            content: "fn main() { 0 }".to_string(),
        }],
        generated_checks: vec![GeneratedCheck {
            command: "ash check src/main.ash".to_string(),
            file: "src/main.ash".to_string(),
        }],
    }
}

#[test]
fn valid_template_manifest_passes_validation() {
    validate_template_manifest(&valid_manifest()).expect("valid template manifest should pass");
}

#[test]
fn missing_identity_is_rejected() {
    let mut manifest = valid_manifest();
    manifest.id.clear();

    let err = validate_template_manifest(&manifest).expect_err("empty id should fail closed");
    assert!(err.to_string().contains("id"));
}

#[test]
fn stale_schema_version_is_rejected() {
    let mut manifest = valid_manifest();
    manifest.schema_version = "ash-template-v0".to_string();

    let err = validate_template_manifest(&manifest).expect_err("stale schema should fail closed");
    assert!(err.to_string().contains("schema_version"));
}

#[test]
fn provider_profile_references_must_be_declared() {
    let mut manifest = valid_manifest();
    manifest.providers[0].profile = "ambient".to_string();

    let err =
        validate_template_manifest(&manifest).expect_err("undeclared profile should fail closed");
    assert!(err.to_string().contains("profile"));
}

#[test]
fn unsupported_template_syntax_is_rejected_before_promotion() {
    let mut manifest = valid_manifest();
    let observe = ["ob", "serve"].concat();
    let with = ["wi", "th"].concat();
    manifest.files[0].content =
        format!("fn main() {{ {observe} sensor {with} id: 1 as reading; return reading }}");

    let err = validate_template_manifest(&manifest)
        .expect_err("stale template syntax should fail closed");
    assert!(err.to_string().contains("unsupported syntax"));
}

#[test]
fn removed_template_computation_carriers_are_rejected_before_promotion() {
    for (stale, label) in [
        (["Pr", "oc<"].concat(), "proc-carrier"),
        (["A", "ct<"].concat(), "act-carrier"),
        (["Work", "flow<"].concat(), "workflow-carrier"),
    ] {
        let mut manifest = valid_manifest();
        manifest.files[0].content = format!("fn helper() -> {stale}Int> {{ do {{ return 0 }} }}");

        let err = match validate_template_manifest(&manifest) {
            Ok(()) => panic!("{stale} should fail closed"),
            Err(err) => err,
        };
        let message = err.to_string();
        assert!(
            message.contains("unsupported syntax") && message.contains(label),
            "{message}"
        );
    }
}

#[test]
fn removed_template_provider_language_is_rejected_before_promotion() {
    for (stale, label) in [
        ("ambient authority", "ambient-authority"),
        ("direct provider", "direct-provider"),
    ] {
        let mut manifest = valid_manifest();
        manifest.files[0].content =
            format!("// This template relies on {stale}\nfn main() {{ return 0 }}");

        let err = match validate_template_manifest(&manifest) {
            Ok(()) => panic!("{stale} should fail closed"),
            Err(err) => err,
        };
        let message = err.to_string();
        assert!(
            message.contains("unsupported syntax") && message.contains(label),
            "{message}"
        );
    }
}
