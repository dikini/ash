use super::support::*;

#[test]
fn process_run_declares_host_hook_metadata() {
    let metadata = builtin_host_hook_metadata("process::run")
        .expect("implemented process host builtin must declare metadata");

    assert_eq!(metadata.builtin_name, "process::run");
    assert_eq!(metadata.operation_identity, "process.run");
    assert_eq!(metadata.effect, Effect::Operational);
    assert_eq!(metadata.required_rows, &["process.run"]);
    assert_eq!(metadata.sandbox_policy, "process-command");
    assert_eq!(metadata.provenance_policy, "host.process.run");
    assert!(
        !metadata.grants_authority,
        "metadata describes host authority requirements but must not grant authority"
    );
}

#[test]
fn pure_structural_builtins_do_not_require_host_metadata() {
    let table = builtin_dispatch_table();
    let entry = table
        .get("string::concat")
        .expect("string::concat should be in the dispatch table");

    assert!(entry.implemented);
    assert!(!builtin_requires_host_hook_metadata(
        "string::concat",
        entry
    ));
    assert!(builtin_host_hook_metadata("string::concat").is_none());
    validate_builtin_host_hook_metadata("string::concat", entry, None)
        .expect("pure builtin should not require host hook metadata");
}

#[test]
fn implemented_host_builtin_without_metadata_fails_closed() {
    let entry = BuiltinEntry {
        arity: 1,
        variadic: false,
        implemented: true,
    };

    let err = validate_builtin_host_hook_metadata("process::danger", &entry, None)
        .expect_err("implemented host builtin without metadata must fail closed");

    assert_eq!(
        err,
        BuiltinHostHookMetadataError::MissingHostHookMetadata {
            builtin_name: "process::danger".to_string()
        }
    );
    assert!(
        err.to_string().contains("missing host hook metadata"),
        "diagnostic should be structured and clear: {err}"
    );
}

#[test]
fn unimplemented_provider_backed_builtin_can_remain_forward_declared() {
    let table = builtin_dispatch_table();
    let entry = table
        .get("http::get")
        .expect("http::get should be forward declared");

    assert!(!entry.implemented);
    assert!(builtin_requires_host_hook_metadata("http::get", entry));
    validate_builtin_host_hook_metadata("http::get", entry, None)
        .expect("forward declaration should fail at execution, not at metadata inventory time");
}
