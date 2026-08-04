use super::*;

use ash_parser::parse_surface_file;
use ash_parser::surface::{CapabilityDef, EffectType};

fn supported_function() -> Definition {
    parse_surface_file("fn supported(value: Int) -> Int { value }")
        .expect("valid function fixture parses")
        .definitions
        .into_iter()
        .find(|definition| matches!(definition, Definition::Function(_)))
        .expect("fixture contains a function")
}

#[test]
fn supported_definition_batch_validates_without_constructing_carriers() {
    let module_key = ModuleKey::root("app").expect("test module key is canonical");
    let definitions = [supported_function()];

    assert_eq!(validate_definition_batch(&module_key, &definitions), Ok(()));
}

#[test]
fn removed_capability_rejects_a_supported_sibling_batch_without_carriers() {
    let module_key = ModuleKey::root("app").expect("test module key is canonical");
    let capability_span = Span::new(700, 730, 20, 5);
    let removed = Definition::Capability(CapabilityDef {
        visibility: Visibility::Public,
        name: "removed_io".into(),
        effect: EffectType::Write,
        params: Vec::new(),
        return_type: None,
        constraints: Vec::new(),
        target_provider: None,
        target_action: None,
        span: capability_span,
    });
    let definitions = [supported_function(), removed];

    let result: Result<(), CanonicalModuleCollectionError> =
        validate_definition_batch(&module_key, &definitions);
    let error = result.expect_err("removed syntax rejects before producing any carrier");
    assert_eq!(
        error.kind(),
        CanonicalModuleCollectionErrorKind::RemovedCapabilitySyntax
    );
    assert_eq!(error.module_key(), &module_key);
    assert_eq!(error.declaration_name(), Some("removed_io"));
    assert_eq!(error.declaration_span(), capability_span);
}
