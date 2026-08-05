//! TASK-2069 RED contract for canonical Engine module transport.
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
use ash_engine::{CheckedModuleArtifactInput, CheckedModuleTransport};

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
    let root_core = ModuleCoreArtifact::new(root_artifact, Vec::new(), checked_core(7));
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
