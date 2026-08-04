//! TASK-2060 RED contract tests for the checked public module-interface handoff.
//!
//! This target owns only the core export-closed carrier. Import binding,
//! lowering, admission, and runtime behavior remain downstream work.

use ash_core::Visibility;
use ash_core::module_graph::{ModuleArtifact, ModuleArtifactOrigin, ModuleKey};
use ash_core::module_interface::{
    ModuleInterfaceBinding, ModuleInterfaceBindingKind, PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION,
    PublicModuleInterface,
};
use proptest::prelude::*;

fn root_key() -> ModuleKey {
    ModuleKey::root("garden").expect("fixture crate name is canonical")
}

fn child(parent: &ModuleKey, name: &str) -> ModuleKey {
    parent
        .child(name)
        .expect("fixture module name is canonical")
}

fn file_artifact(
    key: ModuleKey,
    parent: Option<ModuleKey>,
    children: Vec<ModuleKey>,
) -> ModuleArtifact {
    ModuleArtifact::new(
        key,
        ModuleArtifactOrigin::File("src/module.ash".into()),
        parent,
        children,
    )
    .expect("fixture artifact is structurally valid")
}

fn public_value(
    visible_name: &str,
    defining_module: ModuleKey,
    defining_name: &str,
) -> ModuleInterfaceBinding {
    ModuleInterfaceBinding::declaration(
        visible_name,
        defining_module,
        defining_name,
        ModuleInterfaceBindingKind::Value,
        Visibility::Public,
        ModuleArtifactOrigin::File("src/definitions.ash".into()),
    )
}

#[test]
fn public_interface_is_schema_versioned_and_retains_its_module_artifact_identity() {
    let root = root_key();
    let artifact = file_artifact(root.clone(), None, Vec::new());
    let interface = PublicModuleInterface::new(artifact.clone(), Vec::new())
        .expect("an empty root interface is export-closed");

    assert_eq!(
        interface.schema_version(),
        PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION
    );
    assert_eq!(interface.artifact().key(), &root);
    assert_eq!(interface.artifact().origin(), artifact.origin());
}

#[test]
fn public_child_and_declaration_bindings_retain_visibility_defining_identity_and_origin() {
    let root = root_key();
    let child_key = child(&root, "api");
    let artifact = file_artifact(root.clone(), None, vec![child_key.clone()]);
    let child_origin = ModuleArtifactOrigin::File("src/api.ash".into());
    let child_binding =
        ModuleInterfaceBinding::child("api", child_key, Visibility::Public, child_origin.clone());
    let declaration = public_value("serve", root.clone(), "serve");
    let declaration_origin = declaration.origin().clone();
    let declaration_identity = declaration.defining_identity().clone();
    let interface = PublicModuleInterface::new(artifact, vec![declaration, child_binding])
        .expect("public child and declaration bindings are export-closed");

    let bindings = interface.bindings();
    let child = bindings
        .iter()
        .find(|binding| binding.visible_name() == "api")
        .expect("child binding is retained");
    let declaration = bindings
        .iter()
        .find(|binding| binding.visible_name() == "serve")
        .expect("declaration binding is retained");

    assert_eq!(child.kind(), ModuleInterfaceBindingKind::ChildModule);
    assert_eq!(child.visibility(), Visibility::Public);
    assert_eq!(child.origin(), &child_origin);
    assert_eq!(declaration.kind(), ModuleInterfaceBindingKind::Value);
    assert_eq!(declaration.visibility(), Visibility::Public);
    assert_eq!(declaration.defining_identity(), &declaration_identity);
    assert_eq!(declaration.origin(), &declaration_origin);
}

#[test]
fn public_reexport_alias_changes_only_the_visible_name() {
    let root = root_key();
    let artifact = file_artifact(root.clone(), None, Vec::new());
    let defining = public_value("original", root, "original");
    let alias = defining.reexport_as("renamed");
    let interface = PublicModuleInterface::new(artifact, vec![alias.clone()])
        .expect("a public alias preserves a public defining identity");

    assert_eq!(alias.visible_name(), "renamed");
    assert_eq!(alias.defining_identity(), defining.defining_identity());
    assert_ne!(alias.visible_name(), defining.visible_name());
    assert_eq!(interface.bindings(), &[alias]);
}

#[test]
fn duplicate_visible_bindings_reject_before_a_public_interface_is_published() {
    let root = root_key();
    let artifact = file_artifact(root.clone(), None, Vec::new());
    let first = public_value("shared", root.clone(), "first");
    let second = public_value("shared", root, "second");

    assert!(
        PublicModuleInterface::new(artifact, vec![first, second]).is_err(),
        "duplicate visible names must reject rather than publish a partial interface"
    );
}

#[test]
fn declaration_constructor_cannot_forge_a_child_module_binding_for_publication() {
    let root = root_key();
    let artifact = file_artifact(root.clone(), None, Vec::new());
    let forged = ModuleInterfaceBinding::declaration(
        "forged_child",
        root,
        "forged_child",
        ModuleInterfaceBindingKind::ChildModule,
        Visibility::Public,
        ModuleArtifactOrigin::File("src/forged.ash".into()),
    );

    assert!(
        PublicModuleInterface::new(artifact, vec![forged]).is_err(),
        "a declaration identity must never publish through the child-module namespace"
    );
}

#[test]
fn generic_typed_declarations_reject_until_the_existing_summary_owns_their_identity() {
    let root = root_key();
    let artifact = file_artifact(root.clone(), None, Vec::new());
    let generic = PublicModuleInterface::new(
        artifact.clone(),
        vec![
            public_value("value", root.clone(), "value"),
            ModuleInterfaceBinding::declaration(
                "callable",
                root.clone(),
                "callable",
                ModuleInterfaceBindingKind::Callable,
                Visibility::Public,
                ModuleArtifactOrigin::File("src/callable.ash".into()),
            ),
            ModuleInterfaceBinding::declaration(
                "syntax",
                root.clone(),
                "syntax",
                ModuleInterfaceBindingKind::SyntaxMacro,
                Visibility::Public,
                ModuleArtifactOrigin::File("src/syntax.ash".into()),
            ),
        ],
    )
    .expect("value, callable, and syntax namespaces remain generic core bindings");
    assert_eq!(generic.bindings().len(), 3);

    let typed = ModuleInterfaceBinding::declaration(
        "Widget",
        root,
        "Widget",
        ModuleInterfaceBindingKind::Type,
        Visibility::Public,
        ModuleArtifactOrigin::File("src/widget.ash".into()),
    );

    assert!(
        PublicModuleInterface::new(artifact, vec![typed]).is_err(),
        "typed declarations must use the existing ModuleSemanticSummary identity contract"
    );
}

#[test]
fn deserialized_declaration_child_kind_is_rejected_before_publication() {
    let root = root_key();
    let artifact = file_artifact(root.clone(), None, Vec::new());
    let interface =
        PublicModuleInterface::new(artifact, vec![public_value("ordinary", root, "ordinary")])
            .expect("ordinary public declaration is export-closed");
    let mut payload = serde_json::to_value(interface).expect("interface serializes");
    payload["bindings"][0]["defining_identity"]["identity"]["kind"] =
        serde_json::json!("child_module");

    assert!(
        serde_json::from_value::<PublicModuleInterface>(payload).is_err(),
        "cache deserialization must reject a declaration forged into the child-module namespace"
    );
}

#[test]
fn deserialized_declaration_identity_rejects_unknown_nested_fields() {
    let root = root_key();
    let artifact = file_artifact(root.clone(), None, Vec::new());
    let interface =
        PublicModuleInterface::new(artifact, vec![public_value("ordinary", root, "ordinary")])
            .expect("ordinary public declaration is export-closed");
    let mut payload = serde_json::to_value(interface).expect("interface serializes");
    payload["bindings"][0]["defining_identity"]["identity"]["forged_identity"] =
        serde_json::json!(true);

    assert!(
        serde_json::from_value::<PublicModuleInterface>(payload).is_err(),
        "cache deserialization must reject unknown declaration-identity fields"
    );
}

#[test]
fn deserialized_declaration_identity_rejects_unknown_adjacent_enum_fields() {
    let root = root_key();
    let artifact = file_artifact(root.clone(), None, Vec::new());
    let interface =
        PublicModuleInterface::new(artifact, vec![public_value("ordinary", root, "ordinary")])
            .expect("ordinary public declaration is export-closed");
    let mut payload = serde_json::to_value(interface).expect("interface serializes");
    payload["bindings"][0]["defining_identity"]["forged_variant_field"] = serde_json::json!(true);

    assert!(
        serde_json::from_value::<PublicModuleInterface>(payload).is_err(),
        "cache deserialization must reject unknown fields beside a defining-identity enum tag"
    );
}

#[test]
fn inline_child_origin_with_a_different_parent_rejects_before_publication() {
    let root = root_key();
    let child_key = child(&root, "nested");
    let artifact = file_artifact(root.clone(), None, vec![child_key.clone()]);
    let wrong_parent = ModuleKey::root("other").expect("fixture crate name is canonical");
    let child_binding = ModuleInterfaceBinding::child(
        "nested",
        child_key,
        Visibility::Public,
        ModuleArtifactOrigin::Inline {
            parent: wrong_parent,
            declaration_offset: 17,
        },
    );

    assert!(
        PublicModuleInterface::new(artifact, vec![child_binding]).is_err(),
        "diagnostic source origin must agree with the enclosing artifact without becoming identity"
    );
}

#[test]
fn syntax_bindings_remain_syntax_only_and_never_runtime_callable() {
    let root = root_key();
    let artifact = file_artifact(root.clone(), None, Vec::new());
    let macro_binding = ModuleInterfaceBinding::declaration(
        "rewrite",
        root.clone(),
        "rewrite",
        ModuleInterfaceBindingKind::SyntaxMacro,
        Visibility::Public,
        ModuleArtifactOrigin::File("src/syntax.ash".into()),
    );
    let notation_binding = ModuleInterfaceBinding::declaration(
        "<+>",
        root,
        "<+>",
        ModuleInterfaceBindingKind::SyntaxNotation,
        Visibility::Public,
        ModuleArtifactOrigin::File("src/syntax.ash".into()),
    );
    let interface = PublicModuleInterface::new(artifact, vec![macro_binding, notation_binding])
        .expect("public syntax metadata is not a runtime export");

    for binding in interface.bindings() {
        assert!(matches!(
            binding.kind(),
            ModuleInterfaceBindingKind::SyntaxMacro | ModuleInterfaceBindingKind::SyntaxNotation
        ));
        assert!(!binding.is_runtime_callable());
    }
}

#[test]
fn malformed_or_unsupported_interface_cache_schema_is_rejected() {
    let artifact = file_artifact(root_key(), None, Vec::new());
    let interface = PublicModuleInterface::new(artifact, Vec::new())
        .expect("empty interface serializes for cache validation");
    let mut unsupported = serde_json::to_value(&interface).expect("interface serializes");
    unsupported["schema_version"] = serde_json::json!(PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION + 1);

    assert!(serde_json::from_value::<PublicModuleInterface>(unsupported).is_err());
    assert!(
        serde_json::from_value::<PublicModuleInterface>(serde_json::json!({
            "schema_version": PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION,
        }))
        .is_err()
    );
}

proptest! {
    #[test]
    fn public_interface_never_accepts_private_bindings(name in "[a-z]{1,8}") {
        let root = root_key();
        let artifact = file_artifact(root.clone(), None, Vec::new());
        let private = ModuleInterfaceBinding::declaration(
            &name,
            root,
            &name,
            ModuleInterfaceBindingKind::Value,
            Visibility::Private,
            ModuleArtifactOrigin::File("src/private.ash".into()),
        );

        prop_assert!(PublicModuleInterface::new(artifact, vec![private]).is_err());
    }

    #[test]
    fn normalized_public_bindings_are_independent_of_input_order(reverse in any::<bool>()) {
        let root = root_key();
        let artifact = file_artifact(root.clone(), None, Vec::new());
        let alpha = public_value("alpha", root.clone(), "alpha");
        let beta = public_value("beta", root, "beta");
        let mut reordered = vec![alpha.clone(), beta.clone()];
        if reverse {
            reordered.reverse();
        }

        let canonical = PublicModuleInterface::new(artifact.clone(), vec![alpha, beta])
            .expect("distinct public bindings are export-closed");
        let reordered = PublicModuleInterface::new(artifact, reordered)
            .expect("input order cannot affect public-interface validity");

        prop_assert_eq!(canonical.bindings(), reordered.bindings());
    }
}
