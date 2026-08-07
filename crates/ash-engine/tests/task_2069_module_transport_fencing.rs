//! TASK-2069 RED contract for canonical Engine module transport and scanner fences.
//!
//! The transport below is deliberately non-authorizing data.  It must validate
//! a complete checked Core/CPS/interface closure before exposing it to the
//! later TASK-2063 linking boundary.

use ash_core::Visibility;
use ash_core::core_ash::CoreExpr;
use ash_core::core_ash_typecheck::{CoreTypeCheckEnv, type_check_core_program};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};
use ash_core::cps::{Atom as CpsAtom, ContRef, EffectRow, Term as CpsTerm};
use ash_core::module_graph::{ModuleArtifact, ModuleArtifactOrigin, ModuleKey};
use ash_core::module_interface::{
    ModuleInterfaceBinding, ModuleInterfaceBindingKind, ModuleInterfaceDependency,
    PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION, PublicModuleInterface,
};
use ash_core::module_lowering::{ModuleCoreArtifact, ModuleCpsArtifact, ResolvedModuleImport};
use ash_engine::{CheckedModuleArtifactInput, CheckedModuleTransport, CheckedModuleTransportCache};
use std::fs;

fn artifact(
    key: ModuleKey,
    origin: ModuleArtifactOrigin,
    parent: Option<ModuleKey>,
    children: Vec<ModuleKey>,
) -> ModuleArtifact {
    ModuleArtifact::new(key, origin, parent, children).expect("fixture artifact is canonical")
}

fn checked_core(value: i64) -> ash_core::core_ash_typecheck::TypedCoreProgram {
    let raw = RawCoreProgram::new(CoreExpr::Atom(ash_core::core_ash::CoreAtom::LitInt(value)));
    let validated = validate_core_program(raw).expect("fixture Core validates");
    type_check_core_program(validated, &CoreTypeCheckEnv::default())
        .expect("fixture Core type-checks")
}

fn cps(value: i64) -> CpsTerm {
    CpsTerm::Jump {
        cont: ContRef::Label("__answer".to_owned()),
        arg: CpsAtom::Int(value),
        row: EffectRow::default(),
    }
}

struct Fixture {
    root: ModuleKey,
    root_input: CheckedModuleArtifactInput,
    dependency_input: CheckedModuleArtifactInput,
    forged_root_origin: CheckedModuleArtifactInput,
}

fn fixture() -> Fixture {
    let root = ModuleKey::root("transport_app").expect("fixture root is canonical");
    let dependency = root
        .child("dependency")
        .expect("fixture child is canonical");
    let root_origin = ModuleArtifactOrigin::File("fixtures/transport_app.ash".to_owned());
    let dependency_origin = ModuleArtifactOrigin::File("fixtures/dependency.ash".to_owned());
    let root_artifact = artifact(root.clone(), root_origin, None, vec![dependency.clone()]);
    let dependency_artifact = artifact(
        dependency.clone(),
        dependency_origin.clone(),
        Some(root.clone()),
        Vec::new(),
    );
    let root_interface = PublicModuleInterface::with_dependencies(
        root_artifact.clone(),
        vec![ModuleInterfaceBinding::child(
            "dependency",
            dependency.clone(),
            Visibility::Public,
            dependency_origin,
        )],
        vec![ModuleInterfaceDependency::new(
            dependency.clone(),
            PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION,
        )],
        None,
    )
    .expect("fixture root interface is closed");
    let dependency_interface = PublicModuleInterface::new(dependency_artifact.clone(), Vec::new())
        .expect("fixture dependency interface is closed");
    let root_core = ModuleCoreArtifact::new_with_interface_metadata(
        root_artifact,
        Vec::new(),
        root_interface.schema_version(),
        root_interface.dependencies().to_vec(),
        checked_core(7),
    );
    let dependency_core = ModuleCoreArtifact::new(dependency_artifact, Vec::new(), checked_core(0));
    let root_cps = ModuleCpsArtifact::from_core_artifact(&root_core, cps(7));
    let dependency_cps = ModuleCpsArtifact::from_core_artifact(&dependency_core, cps(0));

    let forged_artifact = artifact(
        root.clone(),
        ModuleArtifactOrigin::File("fixtures/forged.ash".to_owned()),
        None,
        vec![dependency],
    );
    let forged_core = ModuleCoreArtifact::new(forged_artifact, Vec::new(), checked_core(7));
    let forged_cps = ModuleCpsArtifact::from_core_artifact(&forged_core, cps(7));

    Fixture {
        root: root.clone(),
        root_input: CheckedModuleArtifactInput::new(root_interface, root_core, root_cps),
        dependency_input: CheckedModuleArtifactInput::new(
            dependency_interface,
            dependency_core,
            dependency_cps,
        ),
        forged_root_origin: CheckedModuleArtifactInput::new(
            PublicModuleInterface::with_dependencies(
                artifact(
                    root.clone(),
                    ModuleArtifactOrigin::File("fixtures/transport_app.ash".to_owned()),
                    None,
                    vec![
                        root.child("dependency")
                            .expect("fixture child is canonical"),
                    ],
                ),
                vec![ModuleInterfaceBinding::child(
                    "dependency",
                    root.child("dependency")
                        .expect("fixture child is canonical"),
                    Visibility::Public,
                    ModuleArtifactOrigin::File("fixtures/dependency.ash".to_owned()),
                )],
                vec![ModuleInterfaceDependency::new(
                    root.child("dependency")
                        .expect("fixture child is canonical"),
                    PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION,
                )],
                None,
            )
            .expect("forged fixture interface is structurally valid"),
            forged_core,
            forged_cps,
        ),
    }
}

#[test]
fn checked_transport_accepts_complete_canonical_closure_in_key_order() {
    let fixture = fixture();
    let transport = CheckedModuleTransport::new(
        fixture.root.clone(),
        vec![fixture.dependency_input, fixture.root_input],
    )
    .expect("complete checked closure is accepted");

    let keys = transport
        .modules()
        .iter()
        .map(|module| module.interface().artifact().key().clone())
        .collect::<Vec<_>>();
    assert_eq!(
        keys,
        vec![
            fixture.root.clone(),
            fixture.root.child("dependency").unwrap()
        ]
    );
    assert_eq!(transport.root(), &fixture.root);
}

#[test]
fn checked_transport_cache_uses_canonical_root_identity_without_path_aliases() {
    let fixture = fixture();
    let transport = CheckedModuleTransport::new(
        fixture.root.clone(),
        vec![fixture.dependency_input, fixture.root_input],
    )
    .expect("complete checked closure is accepted");
    let mut cache = CheckedModuleTransportCache::default();

    cache
        .insert(transport.clone())
        .expect("the canonical root cache key is initially unused");
    assert_eq!(cache.get(&fixture.root), Some(&transport));
    assert_eq!(transport.cache_key(), fixture.root.cache_key());

    let renamed_display_key = ModuleKey::root("transport_app").expect("canonical key is stable");
    assert_eq!(cache.get(&renamed_display_key), Some(&transport));
    assert!(
        cache
            .insert(transport)
            .expect_err("a second artifact cannot overwrite a canonical cache entry")
            .to_string()
            .contains("duplicate canonical root")
    );
}

#[test]
fn checked_transport_rejects_missing_dependency_and_forged_origin_atomically() {
    let fixture = fixture();
    assert!(
        CheckedModuleTransport::new(fixture.root.clone(), vec![fixture.root_input.clone()])
            .is_err(),
        "missing declared dependency must reject before a transport is published"
    );
    assert!(
        CheckedModuleTransport::new(
            fixture.root,
            vec![fixture.dependency_input, fixture.forged_root_origin]
        )
        .is_err(),
        "Core/CPS/interface origin disagreement must reject atomically"
    );
}

#[test]
fn checked_transport_rejects_cyclic_dependency_snapshot_atomically() {
    let fixture = fixture();
    let root = fixture.root.clone();
    let dependency = root
        .child("dependency")
        .expect("fixture child is canonical");
    let dependency_artifact = fixture.dependency_input.interface().artifact().clone();
    let dependency_interface = PublicModuleInterface::with_dependencies(
        dependency_artifact,
        Vec::new(),
        vec![ModuleInterfaceDependency::new(
            root.clone(),
            PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION,
        )],
        None,
    )
    .expect("cyclic dependency fixture interface is locally shaped");
    let dependency_core = ModuleCoreArtifact::new_with_interface_metadata(
        dependency_interface.artifact().clone(),
        Vec::new(),
        dependency_interface.schema_version(),
        dependency_interface.dependencies().to_vec(),
        checked_core(0),
    );
    let dependency_cps = ModuleCpsArtifact::from_core_artifact(&dependency_core, cps(0));
    let cyclic_dependency_input =
        CheckedModuleArtifactInput::new(dependency_interface, dependency_core, dependency_cps);

    let error = CheckedModuleTransport::new(
        root.clone(),
        vec![fixture.root_input, cyclic_dependency_input],
    )
    .expect_err("a cyclic canonical dependency snapshot must reject before transport publication");
    assert!(
        matches!(
            error,
            ash_engine::CheckedModuleTransportError::ModuleDependencyCycle { ref module }
                if *module == root || *module == dependency
        ),
        "expected a canonical dependency-cycle rejection, got {error:?}"
    );
}

#[test]
fn checked_transport_accepts_same_module_public_metadata_import() {
    let module = ModuleKey::root("same_module_metadata").expect("module key is canonical");
    let origin = ModuleArtifactOrigin::File("fixtures/same_module_metadata.ash".to_owned());
    let artifact = artifact(module.clone(), origin.clone(), None, Vec::new());
    let metadata_binding = ModuleInterfaceBinding::declaration(
        "Reviewer",
        module.clone(),
        "Reviewer",
        ModuleInterfaceBindingKind::Role,
        Visibility::Public,
        origin,
    );
    let interface = PublicModuleInterface::new(artifact.clone(), vec![metadata_binding.clone()])
        .expect("same-module metadata interface is export-closed");
    let import = ResolvedModuleImport::new("Reviewer", metadata_binding);
    let core = ModuleCoreArtifact::new_with_interface_metadata(
        artifact,
        vec![import],
        interface.schema_version(),
        Vec::new(),
        checked_core(1),
    );
    let cps = ModuleCpsArtifact::from_core_artifact(&core, cps(1));
    let input = CheckedModuleArtifactInput::new(interface, core, cps);

    CheckedModuleTransport::new(module, vec![input])
        .expect("public metadata imported from the same module remains transportable");
}

#[test]
fn checked_transport_accepts_cross_module_public_metadata_imports() {
    let fixture = fixture();
    let provider = fixture
        .root
        .child("dependency")
        .expect("provider key is canonical");
    let provider_origin = fixture
        .dependency_input
        .interface()
        .artifact()
        .origin()
        .clone();
    let role = ModuleInterfaceBinding::declaration(
        "Reviewer",
        provider.clone(),
        "Reviewer",
        ModuleInterfaceBindingKind::Role,
        Visibility::Public,
        provider_origin.clone(),
    );
    let policy = ModuleInterfaceBinding::declaration(
        "RateLimit",
        provider,
        "RateLimit",
        ModuleInterfaceBindingKind::Policy,
        Visibility::Public,
        provider_origin,
    );
    let provider_interface = PublicModuleInterface::new(
        fixture.dependency_input.interface().artifact().clone(),
        vec![role.clone(), policy.clone()],
    )
    .expect("provider metadata interface is export-closed");
    let provider_core = ModuleCoreArtifact::new(
        provider_interface.artifact().clone(),
        Vec::new(),
        checked_core(0),
    );
    let provider_input = CheckedModuleArtifactInput::new(
        provider_interface,
        provider_core.clone(),
        ModuleCpsArtifact::from_core_artifact(&provider_core, cps(0)),
    );

    let root_interface = fixture.root_input.interface().clone();
    let root_artifact = root_interface.artifact().clone();
    let root_core = ModuleCoreArtifact::new_with_interface_metadata(
        root_artifact,
        vec![
            ResolvedModuleImport::new("reviewer", role),
            ResolvedModuleImport::new("rate", policy),
        ],
        root_interface.schema_version(),
        root_interface.dependencies().to_vec(),
        checked_core(7),
    );
    let root_input = CheckedModuleArtifactInput::new(
        root_interface,
        root_core.clone(),
        ModuleCpsArtifact::from_core_artifact(&root_core, cps(7)),
    );

    CheckedModuleTransport::new(fixture.root, vec![provider_input, root_input])
        .expect("public role and policy imports remain metadata-only transport");
}

#[test]
fn checked_transport_rejects_duplicate_canonical_module_keys() {
    let fixture = fixture();
    assert!(
        CheckedModuleTransport::new(
            fixture.root,
            vec![fixture.root_input.clone(), fixture.root_input]
        )
        .is_err(),
        "duplicate canonical module keys must not overwrite one another"
    );
}

#[test]
fn checked_transport_rejects_failed_and_unlisted_structural_entries() {
    let fixture = fixture();
    let failed_dependency = CheckedModuleArtifactInput::failed(
        fixture.dependency_input.interface().clone(),
        "type checking failed",
    );
    assert!(
        CheckedModuleTransport::new(
            fixture.root.clone(),
            vec![fixture.root_input, failed_dependency]
        )
        .is_err(),
        "a failed checker result must not cross the Engine transport boundary"
    );

    let broken_root = ModuleKey::root("broken_structure").expect("fixture root is canonical");
    let broken_child = broken_root
        .child("child")
        .expect("fixture child is canonical");
    let broken_root_artifact = artifact(
        broken_root.clone(),
        ModuleArtifactOrigin::File("fixtures/broken_root.ash".to_owned()),
        None,
        Vec::new(),
    );
    let broken_child_artifact = artifact(
        broken_child,
        ModuleArtifactOrigin::File("fixtures/broken_child.ash".to_owned()),
        Some(broken_root.clone()),
        Vec::new(),
    );
    let broken_root_core =
        ModuleCoreArtifact::new(broken_root_artifact.clone(), Vec::new(), checked_core(1));
    let broken_child_core =
        ModuleCoreArtifact::new(broken_child_artifact.clone(), Vec::new(), checked_core(2));
    let broken_root_input = CheckedModuleArtifactInput::new(
        PublicModuleInterface::new(broken_root_artifact, Vec::new())
            .expect("broken root interface is structurally valid"),
        broken_root_core.clone(),
        ModuleCpsArtifact::from_core_artifact(&broken_root_core, cps(1)),
    );
    let broken_child_input = CheckedModuleArtifactInput::new(
        PublicModuleInterface::new(broken_child_artifact, Vec::new())
            .expect("broken child interface is structurally valid"),
        broken_child_core.clone(),
        ModuleCpsArtifact::from_core_artifact(&broken_child_core, cps(2)),
    );
    assert!(
        CheckedModuleTransport::new(broken_root, vec![broken_root_input, broken_child_input])
            .is_err(),
        "a child not listed by its parent must reject before closure publication"
    );
}

#[test]
fn checked_transport_rejects_forged_import_and_export_identities() {
    let fixture = fixture();
    let dependency = fixture
        .root
        .child("dependency")
        .expect("fixture child is canonical");
    let forged_import = ResolvedModuleImport::new(
        "forged",
        ModuleInterfaceBinding::declaration(
            "forged",
            dependency,
            "not_exported",
            ModuleInterfaceBindingKind::Callable,
            Visibility::Public,
            fixture
                .dependency_input
                .interface()
                .artifact()
                .origin()
                .clone(),
        ),
    );
    let forged_core = ModuleCoreArtifact::new(
        fixture.root_input.interface().artifact().clone(),
        vec![forged_import],
        checked_core(7),
    );
    let forged_import_input = CheckedModuleArtifactInput::new(
        fixture.root_input.interface().clone(),
        forged_core.clone(),
        ModuleCpsArtifact::from_core_artifact(&forged_core, cps(7)),
    );
    assert!(
        CheckedModuleTransport::new(
            fixture.root.clone(),
            vec![fixture.dependency_input.clone(), forged_import_input]
        )
        .is_err(),
        "an imported identity absent from the public target view must reject"
    );

    let missing_target = ModuleKey::root("missing_provider").expect("fixture key is canonical");
    let root_artifact = artifact(
        fixture.root.clone(),
        ModuleArtifactOrigin::File("fixtures/transport_app.ash".to_owned()),
        None,
        Vec::new(),
    );
    let forged_interface = PublicModuleInterface::new(
        root_artifact.clone(),
        vec![ModuleInterfaceBinding::declaration(
            "missing",
            missing_target,
            "serve",
            ModuleInterfaceBindingKind::Callable,
            Visibility::Public,
            ModuleArtifactOrigin::File("fixtures/missing_provider.ash".to_owned()),
        )],
    )
    .expect("fixture interface permits the forged target before Engine closure validation");
    let forged_export_core = ModuleCoreArtifact::new(root_artifact, Vec::new(), checked_core(7));
    let forged_export_input = CheckedModuleArtifactInput::new(
        forged_interface,
        forged_export_core.clone(),
        ModuleCpsArtifact::from_core_artifact(&forged_export_core, cps(7)),
    );
    assert!(
        CheckedModuleTransport::new(fixture.root, vec![forged_export_input]).is_err(),
        "a public export whose defining module is absent must reject"
    );
}

#[test]
fn checked_transport_rejects_import_target_missing_from_declared_dependencies() {
    let fixture = fixture();
    let dependency = fixture
        .root
        .child("dependency")
        .expect("fixture child is canonical");
    let dependency_origin = fixture
        .dependency_input
        .interface()
        .artifact()
        .origin()
        .clone();
    let dependency_binding = ModuleInterfaceBinding::declaration(
        "exported",
        dependency,
        "exported",
        ModuleInterfaceBindingKind::Callable,
        Visibility::Public,
        dependency_origin,
    );
    let dependency_interface = PublicModuleInterface::new(
        fixture.dependency_input.interface().artifact().clone(),
        vec![dependency_binding.clone()],
    )
    .expect("dependency interface with a public callable is closed");
    let dependency_core = ModuleCoreArtifact::new(
        dependency_interface.artifact().clone(),
        Vec::new(),
        checked_core(0),
    );
    let dependency_input = CheckedModuleArtifactInput::new(
        dependency_interface,
        dependency_core.clone(),
        ModuleCpsArtifact::from_core_artifact(&dependency_core, cps(0)),
    );

    let forged_import = ResolvedModuleImport::new("remote", dependency_binding);
    let root_interface = PublicModuleInterface::with_dependencies(
        fixture.root_input.interface().artifact().clone(),
        fixture.root_input.interface().bindings().to_vec(),
        Vec::new(),
        None,
    )
    .expect("root interface without the forged dependency is locally shaped");
    let root_core = ModuleCoreArtifact::new_with_interface_metadata(
        root_interface.artifact().clone(),
        vec![forged_import],
        root_interface.schema_version(),
        Vec::new(),
        checked_core(7),
    );
    let root_input = CheckedModuleArtifactInput::new(
        root_interface,
        root_core.clone(),
        ModuleCpsArtifact::from_core_artifact(&root_core, cps(7)),
    );

    let error = CheckedModuleTransport::new(fixture.root, vec![dependency_input, root_input])
        .expect_err("an undeclared imported dependency must reject before transport publication");
    assert!(matches!(
        error,
        ash_engine::CheckedModuleTransportError::UndeclaredImportDependency { .. }
    ));
}

#[test]
fn checked_transport_accepts_import_of_published_structural_child() {
    let fixture = fixture();
    let child = fixture
        .root
        .child("dependency")
        .expect("child key is canonical");
    let child_binding = fixture
        .root_input
        .interface()
        .bindings()
        .iter()
        .find(|binding| {
            binding.defining_identity()
                == &ash_core::module_interface::ModuleInterfaceDefiningIdentity::ChildModule(
                    child.clone(),
                )
        })
        .expect("fixture publishes the structural child")
        .clone();
    let root_interface = fixture.root_input.interface().clone();
    let root_core = ModuleCoreArtifact::new_with_interface_metadata(
        root_interface.artifact().clone(),
        vec![ResolvedModuleImport::new("dependency", child_binding)],
        root_interface.schema_version(),
        root_interface.dependencies().to_vec(),
        checked_core(7),
    );
    let root_input = CheckedModuleArtifactInput::new(
        root_interface,
        root_core.clone(),
        ModuleCpsArtifact::from_core_artifact(&root_core, cps(7)),
    );

    CheckedModuleTransport::new(fixture.root, vec![fixture.dependency_input, root_input])
        .expect("a public structural child is a valid import target");
}

#[test]
fn checked_transport_rejects_import_of_unpublished_structural_child() {
    let root = ModuleKey::root("private_child_import").expect("root key is canonical");
    let child = root.child("private_api").expect("child key is canonical");
    let root_origin = ModuleArtifactOrigin::File("fixtures/private_child_import.ash".to_owned());
    let child_origin = ModuleArtifactOrigin::File("fixtures/private_api.ash".to_owned());
    let root_artifact = artifact(root.clone(), root_origin, None, vec![child.clone()]);
    let child_artifact = artifact(
        child.clone(),
        child_origin.clone(),
        Some(root.clone()),
        Vec::new(),
    );

    let child_interface = PublicModuleInterface::new(child_artifact.clone(), Vec::new())
        .expect("child interface is locally shaped");
    let child_core = ModuleCoreArtifact::new(child_artifact, Vec::new(), checked_core(0));
    let child_input = CheckedModuleArtifactInput::new(
        child_interface,
        child_core.clone(),
        ModuleCpsArtifact::from_core_artifact(&child_core, cps(0)),
    );

    let forged_import = ResolvedModuleImport::new(
        "private_api",
        ModuleInterfaceBinding::child(
            "private_api",
            child.clone(),
            Visibility::Public,
            child_origin,
        ),
    );
    let root_interface = PublicModuleInterface::with_dependencies(
        root_artifact.clone(),
        Vec::new(),
        vec![ModuleInterfaceDependency::new(
            child,
            PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION,
        )],
        None,
    )
    .expect("root interface can omit the private child export");
    let root_core = ModuleCoreArtifact::new_with_interface_metadata(
        root_artifact,
        vec![forged_import],
        root_interface.schema_version(),
        root_interface.dependencies().to_vec(),
        checked_core(1),
    );
    let root_input = CheckedModuleArtifactInput::new(
        root_interface,
        root_core.clone(),
        ModuleCpsArtifact::from_core_artifact(&root_core, cps(1)),
    );

    let error = CheckedModuleTransport::new(root, vec![child_input, root_input])
        .expect_err("an unpublished structural child must not become an import target");
    assert!(matches!(
        error,
        ash_engine::CheckedModuleTransportError::BindingIdentityUnavailable { .. }
    ));
}

#[test]
fn checked_transport_rejects_export_target_missing_from_declared_dependencies() {
    let fixture = fixture();
    let dependency = fixture
        .root
        .child("dependency")
        .expect("fixture child is canonical");
    let dependency_origin = fixture
        .dependency_input
        .interface()
        .artifact()
        .origin()
        .clone();
    let exported_binding = ModuleInterfaceBinding::declaration(
        "exported",
        dependency.clone(),
        "exported",
        ModuleInterfaceBindingKind::Callable,
        Visibility::Public,
        dependency_origin.clone(),
    );
    let dependency_interface = PublicModuleInterface::new(
        fixture.dependency_input.interface().artifact().clone(),
        vec![exported_binding],
    )
    .expect("dependency interface with a public callable is closed");
    let dependency_core = ModuleCoreArtifact::new(
        dependency_interface.artifact().clone(),
        Vec::new(),
        checked_core(0),
    );
    let dependency_input = CheckedModuleArtifactInput::new(
        dependency_interface,
        dependency_core.clone(),
        ModuleCpsArtifact::from_core_artifact(&dependency_core, cps(0)),
    );
    let reexport = ModuleInterfaceBinding::declaration(
        "alias",
        dependency,
        "exported",
        ModuleInterfaceBindingKind::Callable,
        Visibility::Public,
        dependency_origin,
    );
    let mut bindings = fixture.root_input.interface().bindings().to_vec();
    bindings.push(reexport);
    let root_interface = PublicModuleInterface::with_dependencies(
        fixture.root_input.interface().artifact().clone(),
        bindings,
        Vec::new(),
        None,
    )
    .expect("root interface without the forged dependency is locally shaped");
    let root_core = ModuleCoreArtifact::new_with_interface_metadata(
        root_interface.artifact().clone(),
        Vec::new(),
        root_interface.schema_version(),
        Vec::new(),
        checked_core(7),
    );
    let root_input = CheckedModuleArtifactInput::new(
        root_interface,
        root_core.clone(),
        ModuleCpsArtifact::from_core_artifact(&root_core, cps(7)),
    );

    let error = CheckedModuleTransport::new(fixture.root, vec![dependency_input, root_input])
        .expect_err("an external export target absent from dependencies must reject");
    assert!(matches!(
        error,
        ash_engine::CheckedModuleTransportError::UndeclaredExportDependency { .. }
    ));
}

#[test]
fn checked_transport_rejects_reexport_when_defining_module_does_not_publish_identity() {
    let root = ModuleKey::root("reexport_fence").expect("root key is canonical");
    let provider = root.child("provider").expect("provider key is canonical");
    let reexport = root.child("reexport").expect("reexport key is canonical");
    let provider_origin = ModuleArtifactOrigin::File("fixtures/provider.ash".to_owned());
    let reexport_origin = ModuleArtifactOrigin::File("fixtures/reexport.ash".to_owned());

    let root_artifact = artifact(
        root.clone(),
        ModuleArtifactOrigin::File("fixtures/reexport_fence.ash".to_owned()),
        None,
        vec![provider.clone(), reexport.clone()],
    );
    let provider_artifact = artifact(
        provider.clone(),
        provider_origin.clone(),
        Some(root.clone()),
        Vec::new(),
    );
    let reexport_artifact = artifact(
        reexport.clone(),
        reexport_origin.clone(),
        Some(root.clone()),
        Vec::new(),
    );

    let root_interface = PublicModuleInterface::new(
        root_artifact.clone(),
        vec![
            ModuleInterfaceBinding::child(
                "provider",
                provider.clone(),
                Visibility::Public,
                provider_origin.clone(),
            ),
            ModuleInterfaceBinding::child(
                "reexport",
                reexport,
                Visibility::Public,
                reexport_origin,
            ),
        ],
    )
    .expect("root interface is structurally valid");
    let provider_interface = PublicModuleInterface::new(provider_artifact.clone(), Vec::new())
        .expect("provider interface is intentionally missing the forged declaration");
    let reexport_interface = PublicModuleInterface::new(
        reexport_artifact.clone(),
        vec![ModuleInterfaceBinding::declaration(
            "alias",
            provider,
            "serve",
            ModuleInterfaceBindingKind::Callable,
            Visibility::Public,
            provider_origin,
        )],
    )
    .expect("re-export interface is locally shaped");

    let root_core = ModuleCoreArtifact::new(root_artifact, Vec::new(), checked_core(0));
    let provider_core = ModuleCoreArtifact::new(provider_artifact, Vec::new(), checked_core(1));
    let reexport_core = ModuleCoreArtifact::new(reexport_artifact, Vec::new(), checked_core(2));
    let root_input = CheckedModuleArtifactInput::new(
        root_interface,
        root_core.clone(),
        ModuleCpsArtifact::from_core_artifact(&root_core, cps(0)),
    );
    let provider_input = CheckedModuleArtifactInput::new(
        provider_interface,
        provider_core.clone(),
        ModuleCpsArtifact::from_core_artifact(&provider_core, cps(1)),
    );
    let reexport_input = CheckedModuleArtifactInput::new(
        reexport_interface,
        reexport_core.clone(),
        ModuleCpsArtifact::from_core_artifact(&reexport_core, cps(2)),
    );

    assert!(
        CheckedModuleTransport::new(root, vec![root_input, provider_input, reexport_input])
            .is_err(),
        "a re-export must not manufacture a declaration absent from its defining module"
    );
}

#[test]
fn checked_core_cps_transport_retains_interface_schema_and_dependency_snapshot() {
    let fixture = fixture();
    let root_interface = fixture.root_input.interface().clone();
    let root_artifact = root_interface.artifact().clone();
    let root_core = ModuleCoreArtifact::new_with_interface_metadata(
        root_artifact.clone(),
        Vec::new(),
        root_interface.schema_version(),
        root_interface.dependencies().to_vec(),
        checked_core(7),
    );
    assert_eq!(
        root_core.interface_schema_version(),
        root_interface.schema_version()
    );
    assert_eq!(root_core.dependencies(), root_interface.dependencies());
    let root_cps = ModuleCpsArtifact::from_core_artifact(&root_core, cps(7));
    assert_eq!(
        root_cps.interface_schema_version(),
        root_interface.schema_version()
    );
    assert_eq!(root_cps.dependencies(), root_interface.dependencies());

    let stale_cps = ModuleCpsArtifact::new_with_interface_metadata(
        root_artifact,
        Vec::new(),
        root_interface.schema_version() + 1,
        root_interface.dependencies().to_vec(),
        cps(7),
    );
    let stale_input = CheckedModuleArtifactInput::new(root_interface, root_core, stale_cps);
    assert!(
        CheckedModuleTransport::new(fixture.root, vec![stale_input]).is_err(),
        "interface schema disagreement across Core/CPS must reject before transport publication"
    );
}

#[test]
fn module_file_function_count_uses_expanded_ast_when_strings_contain_braces() {
    let path = std::env::temp_dir().join(format!(
        "ash-task-2069-ast-function-count-{}-{}.ash",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos()
    ));
    fs::write(&path, r#"pub fn brace_text() -> String { "{" }"#)
        .expect("write AST function-count fixture");

    let engine = ash_engine::Engine::new()
        .build()
        .expect("engine should build for the scanner-fence fixture");
    let result = engine
        .check_module_file(&path)
        .expect("the parsed function with a brace in its string should be valid");

    assert_eq!(result.fn_count, 1);
    assert!(result.warnings.is_empty());
    let _ = fs::remove_file(path);
}

#[test]
fn parser_owned_imports_survive_crate_root_metadata_preamble() {
    let imports = ash_engine::module_loader::parse_module_imports(
        "crate app;\nuse crate::api::{serve as remote};\nfn entry() -> Int { 1 }",
    )
    .expect("parser-owned imports should parse after crate metadata");

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_segments, ["crate", "api"]);
    assert_eq!(imports[0].selections.len(), 1);
    assert!(matches!(
        &imports[0].selections[0],
        ash_engine::module_loader::ImportSelection::Named { name, alias }
            if name == "serve" && alias.as_deref() == Some("remote")
    ));
}
