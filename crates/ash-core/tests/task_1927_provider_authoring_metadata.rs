use ash_core::capability::{
    CapabilityProvider, ProviderAuthoringMetadata, ProviderMetadataError,
    ProviderOperationMetadata, validate_provider_authoring_metadata,
};
use ash_core::{Effect, Value};

#[test]
fn provider_metadata_requires_operation_surface() {
    let metadata = ProviderAuthoringMetadata::new("empty-provider");

    let err = validate_provider_authoring_metadata(&metadata)
        .expect_err("provider metadata without operations must fail closed");

    assert_eq!(
        err,
        ProviderMetadataError::MissingOperationSurface {
            provider_name: "empty-provider".to_string()
        }
    );
}

#[test]
fn provider_operation_metadata_carries_authority_shape_without_granting_authority() {
    let metadata = ProviderAuthoringMetadata::new("fs").with_operation(
        ProviderOperationMetadata::new("read_to_string", Effect::Epistemic)
            .with_required_row("fs.read")
            .with_constraint("paths")
            .with_resource("filesystem")
            .with_sandbox_policy("host.fs.read")
            .with_provenance_policy("host.fs.read.redacted"),
    );

    validate_provider_authoring_metadata(&metadata).expect("metadata should be valid");

    let operation = metadata
        .operation("read_to_string")
        .expect("operation exists");
    assert_eq!(operation.effect, Effect::Epistemic);
    assert_eq!(operation.required_rows, vec!["fs.read"]);
    assert_eq!(operation.constraints, vec!["paths"]);
    assert_eq!(operation.resources, vec!["filesystem"]);
    assert_eq!(operation.sandbox_policy.as_deref(), Some("host.fs.read"));
    assert_eq!(
        operation.provenance_policy.as_deref(),
        Some("host.fs.read.redacted")
    );
    assert!(
        !operation.grants_authority,
        "operation metadata describes requirements but must not grant authority"
    );
}

#[test]
fn provider_metadata_rejects_authority_widening_operation() {
    let metadata = ProviderAuthoringMetadata::new("danger").with_operation(
        ProviderOperationMetadata::new("run", Effect::Operational)
            .with_required_row("process.run")
            .with_sandbox_policy("host.process.run")
            .with_provenance_policy("host.process.run.redacted")
            .with_authority_grant_for_test(true),
    );

    let err = validate_provider_authoring_metadata(&metadata)
        .expect_err("metadata must fail when it claims to grant authority");

    assert_eq!(
        err,
        ProviderMetadataError::AuthorityWideningOperation {
            provider_name: "danger".to_string(),
            operation_name: "run".to_string()
        }
    );
}

#[test]
fn provider_without_authored_metadata_fails_closed() {
    #[derive(Debug)]
    struct UnauthoredProvider;

    #[async_trait::async_trait]
    impl CapabilityProvider for UnauthoredProvider {
        fn name(&self) -> &str {
            "unauthored"
        }

        fn effect(&self) -> Effect {
            Effect::Operational
        }

        async fn observe(
            &self,
            _constraints: &[ash_core::Constraint],
        ) -> Result<Value, ash_core::capability::CapabilityError> {
            Ok(Value::Null)
        }

        async fn execute(
            &self,
            _action_name: &str,
            _args: &[Value],
        ) -> Result<Value, ash_core::capability::CapabilityError> {
            Ok(Value::Null)
        }
    }

    let provider = UnauthoredProvider;
    let metadata = provider.provider_metadata();

    assert_eq!(metadata.provider_name, "unauthored");
    let err = validate_provider_authoring_metadata(&metadata)
        .expect_err("providers without operation metadata must fail closed");
    assert_eq!(
        err,
        ProviderMetadataError::MissingOperationSurface {
            provider_name: "unauthored".to_string()
        }
    );
}
