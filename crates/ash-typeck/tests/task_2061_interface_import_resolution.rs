//! TASK-2061 RED contracts for interface-driven import resolution.
//!
//! These tests use only in-memory checked public interfaces. They deliberately
//! do not invoke parser import resolution, filesystem discovery, or Engine
//! export tables.

use ash_core::Visibility;
use ash_core::module_graph::{ModuleArtifact, ModuleArtifactOrigin, ModuleKey};
use ash_core::module_interface::{
    ModuleInterfaceBinding, ModuleInterfaceBindingKind, PublicModuleInterface,
};
use ash_parser::module::{ModuleBody, ModuleItem};
use ash_parser::{ModuleUnit, parse_surface_file};
use ash_typeck::TypeEnv;
use ash_typeck::interface_import_resolver::{
    CheckedInterfaceStore, CheckedInterfaceStoreError, InterfaceImportDiagnostic,
    InterfaceImportEnvironment, InterfaceImportMember, InterfaceImportPath, InterfaceImportRequest,
    InterfaceImportResolver,
};
use ash_typeck::module_interface_finalization::{
    FinalizedModuleInterface, TypeEnvModuleInterfaceCollection,
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

fn artifact(
    key: ModuleKey,
    structural_parent: Option<ModuleKey>,
    children: Vec<ModuleKey>,
) -> ModuleArtifact {
    ModuleArtifact::new(
        key,
        ModuleArtifactOrigin::File("src/fixture.ash".into()),
        structural_parent,
        children,
    )
    .expect("fixture artifact is structurally valid")
}

fn public_declaration(
    visible_name: &str,
    module: ModuleKey,
    defining_name: &str,
    kind: ModuleInterfaceBindingKind,
) -> ModuleInterfaceBinding {
    ModuleInterfaceBinding::declaration(
        visible_name,
        module,
        defining_name,
        kind,
        Visibility::Public,
        ModuleArtifactOrigin::File("src/fixture.ash".into()),
    )
}

fn parser_module_unit(artifact: ModuleArtifact, source: &str) -> ModuleUnit {
    let parsed = parse_surface_file(source).expect("fixture module source parses");
    let span = parsed.span;
    let body = ModuleBody::from_items(
        parsed
            .definitions
            .into_iter()
            .map(ModuleItem::Definition)
            .chain(parsed.module_decls.into_iter().map(ModuleItem::ModuleDecl))
            .collect(),
        span,
    );

    ModuleUnit::new(artifact, body, parsed.path, parsed.comments)
}

fn finalize(
    module_unit: &ModuleUnit,
    raw_interface: PublicModuleInterface,
) -> FinalizedModuleInterface {
    let mut type_env = TypeEnv::new();
    type_env
        .register_surface_declarations(module_unit.body().definitions())
        .expect("fixture callable declarations register under the TypeEnv");

    TypeEnvModuleInterfaceCollection::collect(&mut type_env, module_unit)
        .expect("fixture parser module unit collects under the TypeEnv")
        .finalize(raw_interface)
        .expect("fixture raw interface is justified by parser/type-checker collection")
}

fn import_path(segments: &[&str]) -> InterfaceImportPath {
    InterfaceImportPath::new(segments).expect("fixture import path is canonical")
}

fn resolver(interfaces: Vec<FinalizedModuleInterface>) -> InterfaceImportResolver {
    let store = CheckedInterfaceStore::new(interfaces)
        .expect("fixture finalized interfaces have unique canonical module identities");
    InterfaceImportResolver::new(store)
}

struct RootApiInterfaces {
    api: ModuleKey,
    root_serve: ModuleInterfaceBinding,
    api_serve: ModuleInterfaceBinding,
    interfaces: Vec<FinalizedModuleInterface>,
}

fn root_and_api_interfaces() -> RootApiInterfaces {
    let root = root_key();
    let api = child(&root, "api");
    let root_serve = public_declaration(
        "serve",
        root.clone(),
        "serve",
        ModuleInterfaceBindingKind::Callable,
    );
    let api_serve = public_declaration(
        "serve",
        api.clone(),
        "serve",
        ModuleInterfaceBindingKind::Callable,
    );
    let syntax_macro = public_declaration(
        "rewrite",
        api.clone(),
        "rewrite",
        ModuleInterfaceBindingKind::SyntaxMacro,
    );
    let root_artifact = artifact(root.clone(), None, vec![api.clone()]);
    let root_unit = parser_module_unit(root_artifact.clone(), "pub fn serve() {}\npub mod api;");
    let root_interface = finalize(
        &root_unit,
        PublicModuleInterface::new(
            root_artifact,
            vec![
                root_serve.clone(),
                ModuleInterfaceBinding::child(
                    "api",
                    api.clone(),
                    Visibility::Public,
                    ModuleArtifactOrigin::File("src/api.ash".into()),
                ),
            ],
        )
        .expect("root raw carrier is export-closed"),
    );
    let api_artifact = artifact(api.clone(), Some(root.clone()), Vec::new());
    let api_unit = parser_module_unit(
        api_artifact.clone(),
        "pub fn serve() {}\npub macro rewrite() => 0;",
    );
    let api_interface = finalize(
        &api_unit,
        PublicModuleInterface::new(api_artifact, vec![api_serve.clone(), syntax_macro])
            .expect("api raw carrier is export-closed"),
    );

    RootApiInterfaces {
        api,
        root_serve,
        api_serve,
        interfaces: vec![root_interface, api_interface],
    }
}

#[test]
fn explicit_child_import_alias_preserves_the_defining_identity() {
    let fixture = root_and_api_interfaces();
    let expected_identity = fixture.api_serve.defining_identity().clone();
    let resolver = resolver(fixture.interfaces);
    let mut environment = InterfaceImportEnvironment::new();

    resolver
        .resolve(
            &mut environment,
            &[InterfaceImportRequest::explicit(
                import_path(&["garden", "api", "serve"]),
                "launch",
            )],
        )
        .expect("a public checked child declaration resolves through an alias");

    let resolved = environment
        .lookup("launch")
        .expect("alias enters the importing environment");
    assert_eq!(resolved.defining_identity(), &expected_identity);
    assert_ne!(resolved.visible_name(), "launch");
}

#[test]
fn checked_interface_store_rejects_duplicate_finalized_module_keys() {
    let mut fixture = root_and_api_interfaces();
    let duplicate = fixture.interfaces[0].clone();
    let expected_module = duplicate.module_key().clone();

    let error = CheckedInterfaceStore::new(vec![fixture.interfaces.remove(0), duplicate])
        .expect_err("two finalized wrappers may not claim one canonical module key");

    assert!(matches!(
        error,
        CheckedInterfaceStoreError::DuplicateModule { module } if module == expected_module
    ));
}

#[test]
fn group_import_is_atomic_when_any_member_is_unknown() {
    let fixture = root_and_api_interfaces();
    let root_identity = fixture.root_serve.defining_identity().clone();
    let resolver = resolver(fixture.interfaces);
    let mut environment = InterfaceImportEnvironment::new();
    resolver
        .resolve(
            &mut environment,
            &[InterfaceImportRequest::explicit(
                import_path(&["garden", "serve"]),
                "already_bound",
            )],
        )
        .expect("fixture precondition resolves");

    let error = resolver
        .resolve(
            &mut environment,
            &[InterfaceImportRequest::group(
                import_path(&["garden", "api"]),
                vec![
                    InterfaceImportMember::renamed("serve", "staged"),
                    InterfaceImportMember::named("unknown"),
                ],
            )],
        )
        .expect_err("an unknown group member rejects the complete group");

    assert!(matches!(
        error,
        InterfaceImportDiagnostic::UnresolvedImport { .. }
    ));
    assert_eq!(
        environment
            .lookup("already_bound")
            .expect("prior environment is retained")
            .defining_identity(),
        &root_identity
    );
    assert!(
        environment.lookup("staged").is_err(),
        "the resolved group member must not leak from a rejected group"
    );
}

#[test]
fn group_import_rejects_duplicate_local_aliases_without_publishing_members() {
    let fixture = root_and_api_interfaces();
    let resolver = resolver(fixture.interfaces);
    let mut environment = InterfaceImportEnvironment::new();

    let error = resolver
        .resolve(
            &mut environment,
            &[InterfaceImportRequest::group(
                import_path(&["garden", "api"]),
                vec![
                    InterfaceImportMember::renamed("serve", "shared"),
                    InterfaceImportMember::renamed("rewrite", "shared"),
                ],
            )],
        )
        .expect_err("duplicate local aliases reject the complete staged group");

    assert!(matches!(
        error,
        InterfaceImportDiagnostic::DuplicateLocalBinding { local_name } if local_name == "shared"
    ));
    assert!(matches!(
        environment.lookup("shared"),
        Err(InterfaceImportDiagnostic::UnresolvedBinding { .. })
    ));
}

#[test]
fn explicit_import_wins_over_a_glob_regardless_of_glob_order() {
    let fixture = root_and_api_interfaces();
    let root_identity = fixture.root_serve.defining_identity().clone();
    let resolver = resolver(fixture.interfaces);
    let mut environment = InterfaceImportEnvironment::new();

    resolver
        .resolve(
            &mut environment,
            &[
                InterfaceImportRequest::explicit(import_path(&["garden", "serve"]), "serve"),
                InterfaceImportRequest::glob(import_path(&["garden", "api"])),
            ],
        )
        .expect("an explicit local binding takes precedence over a later glob");

    assert_eq!(
        environment
            .lookup("serve")
            .expect("explicit binding remains resolvable")
            .defining_identity(),
        &root_identity
    );
}

#[test]
fn distinct_glob_identities_report_ambiguity_at_lookup() {
    let root = root_key();
    let alpha = child(&root, "alpha");
    let beta = child(&root, "beta");
    let root_artifact = artifact(root.clone(), None, vec![alpha.clone(), beta.clone()]);
    let root_unit = parser_module_unit(root_artifact.clone(), "pub mod alpha;\npub mod beta;");
    let root_interface = finalize(
        &root_unit,
        PublicModuleInterface::new(
            root_artifact,
            vec![
                ModuleInterfaceBinding::child(
                    "alpha",
                    alpha.clone(),
                    Visibility::Public,
                    ModuleArtifactOrigin::File("src/alpha.ash".into()),
                ),
                ModuleInterfaceBinding::child(
                    "beta",
                    beta.clone(),
                    Visibility::Public,
                    ModuleArtifactOrigin::File("src/beta.ash".into()),
                ),
            ],
        )
        .expect("root raw carrier is export-closed"),
    );
    let alpha_artifact = artifact(alpha.clone(), Some(root.clone()), Vec::new());
    let alpha_unit = parser_module_unit(alpha_artifact.clone(), "pub fn serve() {}");
    let alpha_interface = finalize(
        &alpha_unit,
        PublicModuleInterface::new(
            alpha_artifact,
            vec![public_declaration(
                "serve",
                alpha,
                "serve",
                ModuleInterfaceBindingKind::Callable,
            )],
        )
        .expect("alpha raw carrier is export-closed"),
    );
    let beta_artifact = artifact(beta.clone(), Some(root), Vec::new());
    let beta_unit = parser_module_unit(beta_artifact.clone(), "pub fn serve() {}");
    let beta_interface = finalize(
        &beta_unit,
        PublicModuleInterface::new(
            beta_artifact,
            vec![public_declaration(
                "serve",
                beta,
                "serve",
                ModuleInterfaceBindingKind::Callable,
            )],
        )
        .expect("beta raw carrier is export-closed"),
    );
    let resolver = resolver(vec![root_interface, alpha_interface, beta_interface]);
    let mut environment = InterfaceImportEnvironment::new();

    resolver
        .resolve(
            &mut environment,
            &[
                InterfaceImportRequest::glob(import_path(&["garden", "alpha"])),
                InterfaceImportRequest::glob(import_path(&["garden", "beta"])),
            ],
        )
        .expect("glob candidates may enter the environment before lookup");

    assert!(matches!(
        environment.lookup("serve"),
        Err(InterfaceImportDiagnostic::AmbiguousBinding { .. })
    ));
}

#[test]
fn public_child_without_a_checked_interface_is_rejected() {
    let mut fixture = root_and_api_interfaces();
    let root_interface = fixture.interfaces.remove(0);
    let expected_missing = fixture.api.clone();
    let resolver = resolver(vec![root_interface]);
    let mut environment = InterfaceImportEnvironment::new();

    let error = resolver
        .resolve(
            &mut environment,
            &[InterfaceImportRequest::explicit(
                import_path(&["garden", "api", "serve"]),
                "serve",
            )],
        )
        .expect_err("a public child cannot resolve without its checked interface");

    assert!(matches!(
        error,
        InterfaceImportDiagnostic::MissingCheckedInterface { module } if module == expected_missing
    ));
}

#[test]
fn syntax_macro_import_remains_nonruntime_metadata() {
    let fixture = root_and_api_interfaces();
    let resolver = resolver(fixture.interfaces);
    let mut environment = InterfaceImportEnvironment::new();

    resolver
        .resolve(
            &mut environment,
            &[InterfaceImportRequest::explicit(
                import_path(&["garden", "api", "rewrite"]),
                "rewrite",
            )],
        )
        .expect("public syntax macro metadata resolves from a checked interface");

    let binding = environment
        .lookup("rewrite")
        .expect("syntax metadata enters the importing environment");
    assert_eq!(binding.kind(), ModuleInterfaceBindingKind::SyntaxMacro);
    assert!(!binding.is_runtime_callable());
}

#[test]
fn private_bindings_cannot_enter_a_public_interface_or_import_environment() {
    let root = root_key();
    let private = ModuleInterfaceBinding::declaration(
        "secret",
        root.clone(),
        "secret",
        ModuleInterfaceBindingKind::Value,
        Visibility::Private,
        ModuleArtifactOrigin::File("src/secret.ash".into()),
    );

    assert!(
        PublicModuleInterface::new(artifact(root, None, Vec::new()), vec![private]).is_err(),
        "private declarations must fail before an import resolver can observe them"
    );
}

#[test]
fn duplicate_explicit_local_bindings_report_a_specific_diagnostic() {
    let fixture = root_and_api_interfaces();
    let resolver = resolver(fixture.interfaces);
    let mut environment = InterfaceImportEnvironment::new();

    let error = resolver
        .resolve(
            &mut environment,
            &[
                InterfaceImportRequest::explicit(import_path(&["garden", "serve"]), "shared"),
                InterfaceImportRequest::explicit(
                    import_path(&["garden", "api", "serve"]),
                    "shared",
                ),
            ],
        )
        .expect_err("two explicit imports may not publish the same local binding");

    assert!(matches!(
        error,
        InterfaceImportDiagnostic::DuplicateLocalBinding { .. }
    ));
}

proptest! {
    #[test]
    fn explicit_child_import_alias_preserves_the_provider_defining_identity(
        local_alias in "alias_[a-z][a-z0-9_]{0,15}",
    ) {
        let fixture = root_and_api_interfaces();
        let expected_identity = fixture.api_serve.defining_identity().clone();
        let resolver = resolver(fixture.interfaces);
        let mut environment = InterfaceImportEnvironment::new();

        resolver
            .resolve(
                &mut environment,
                &[InterfaceImportRequest::explicit(
                    import_path(&["garden", "api", "serve"]),
                    local_alias.as_str(),
                )],
            )
            .expect("each canonical local alias resolves from the finalized checked interface");

        let resolved = environment
            .lookup(local_alias.as_str())
            .expect("the generated local alias enters the importing environment");
        prop_assert_eq!(resolved.defining_identity(), &expected_identity);
        prop_assert_ne!(resolved.visible_name(), local_alias.as_str());
    }
}
