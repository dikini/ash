//! TASK-2063 RED contracts for Engine-sealed linked-module admission.
//!
//! The public Module Core/CPS carriers below are deliberately forgeable
//! transport data. Only the Engine may validate their complete canonical
//! closure, seal an admitted execution route, and issue a request for it.

use ash_core::Value;
use ash_core::Visibility;
use ash_core::core_ash::{CoreAtom, CoreExpr};
use ash_core::core_ash_typecheck::{CoreTypeCheckEnv, TypedCoreProgram, type_check_core_program};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};
use ash_core::cps::{Atom as CpsAtom, ContRef, EffectRow, Term as CpsTerm};
use ash_core::module_graph::{ModuleArtifact, ModuleArtifactOrigin, ModuleKey};
use ash_core::module_interface::{
    ModuleInterfaceBinding, ModuleInterfaceDependency, PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION,
    PublicModuleInterface,
};
use ash_core::module_lowering::{ModuleCoreArtifact, ModuleCpsArtifact};
use ash_core::semantic_summary::{SourceAnchor, SourceOrigin};
use ash_engine::{
    CanonicalTerminalEnvelopeV1, Engine, LinkedModuleArtifactInput, LinkedModuleClosure,
};

fn module_artifact(
    key: ModuleKey,
    origin: ModuleArtifactOrigin,
    structural_parent: Option<ModuleKey>,
    children: Vec<ModuleKey>,
) -> ModuleArtifact {
    ModuleArtifact::new(key, origin, structural_parent, children)
        .expect("fixture module artifact is structurally canonical")
}

fn source_anchor(path: &str, label: &str) -> SourceAnchor {
    SourceAnchor::new(SourceOrigin::File(path.to_owned()), None, label)
}

fn checked_core_literal(value: i64) -> TypedCoreProgram {
    let raw = RawCoreProgram::new(CoreExpr::Atom(CoreAtom::LitInt(value)));
    let validated = validate_core_program(raw).expect("fixture literal Core validates");
    type_check_core_program(validated, &CoreTypeCheckEnv::default())
        .expect("fixture literal Core type-checks")
}

fn handler_free_answer(value: i64) -> CpsTerm {
    CpsTerm::Jump {
        cont: ContRef::Label("__answer".to_owned()),
        arg: CpsAtom::Int(value),
        row: EffectRow::default(),
    }
}

struct LinkedFixture {
    root_key: ModuleKey,
    root: LinkedModuleArtifactInput,
    dependency: LinkedModuleArtifactInput,
    root_with_mismatched_cps_origin: LinkedModuleArtifactInput,
    failed_dependency: LinkedModuleArtifactInput,
}

fn linked_fixture() -> LinkedFixture {
    let root_key = ModuleKey::root("linked_app").expect("fixture root key is canonical");
    let dependency_key = root_key
        .child("dependency")
        .expect("fixture child key is canonical");
    let root_origin = ModuleArtifactOrigin::File("fixtures/linked_app.ash".to_owned());
    let dependency_origin = ModuleArtifactOrigin::File("fixtures/dependency.ash".to_owned());
    let root_artifact = module_artifact(
        root_key.clone(),
        root_origin.clone(),
        None,
        vec![dependency_key.clone()],
    );
    let dependency_artifact = module_artifact(
        dependency_key.clone(),
        dependency_origin.clone(),
        Some(root_key.clone()),
        Vec::new(),
    );

    let root_interface = PublicModuleInterface::with_dependencies(
        root_artifact.clone(),
        vec![ModuleInterfaceBinding::child(
            "dependency",
            dependency_key.clone(),
            Visibility::Public,
            dependency_origin,
        )],
        vec![ModuleInterfaceDependency::new(
            dependency_key,
            PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION,
        )],
        None,
    )
    .expect("fixture root interface declares its canonical dependency");
    let dependency_interface = PublicModuleInterface::new(dependency_artifact.clone(), Vec::new())
        .expect("fixture child interface is export-closed");

    let root_core =
        ModuleCoreArtifact::new(root_artifact.clone(), Vec::new(), checked_core_literal(7));
    let root_cps = ModuleCpsArtifact::from_core_artifact(&root_core, handler_free_answer(7));
    let dependency_core = ModuleCoreArtifact::new(
        dependency_artifact.clone(),
        Vec::new(),
        checked_core_literal(0),
    );
    let dependency_cps =
        ModuleCpsArtifact::from_core_artifact(&dependency_core, handler_free_answer(0));

    let forged_root_artifact = module_artifact(
        root_key.clone(),
        ModuleArtifactOrigin::File("fixtures/forged-linked_app.ash".to_owned()),
        None,
        vec![root_artifact.child_keys()[0].clone()],
    );
    let forged_root_cps =
        ModuleCpsArtifact::new(forged_root_artifact, Vec::new(), handler_free_answer(7));

    LinkedFixture {
        root_key,
        root: LinkedModuleArtifactInput::new(
            root_interface.clone(),
            root_core.clone(),
            root_cps,
            source_anchor("fixtures/linked_app.ash", "linked_app"),
        ),
        dependency: LinkedModuleArtifactInput::new(
            dependency_interface.clone(),
            dependency_core,
            dependency_cps,
            source_anchor("fixtures/dependency.ash", "dependency"),
        ),
        root_with_mismatched_cps_origin: LinkedModuleArtifactInput::new(
            root_interface,
            root_core,
            forged_root_cps,
            source_anchor("fixtures/linked_app.ash", "linked_app"),
        ),
        // A failed lower/type-checker entry has no Core/CPS artifact to route.
        // The Engine must reject the entire closure before minting admission.
        failed_dependency: LinkedModuleArtifactInput::failed(
            dependency_interface,
            source_anchor("fixtures/dependency.ash", "dependency"),
            "type checking failed",
        ),
    }
}

#[test]
fn shuffled_complete_linked_closure_admits_and_executes_to_a_canonical_terminal() {
    let fixture = linked_fixture();
    let closure =
        LinkedModuleClosure::new(fixture.root_key, vec![fixture.dependency, fixture.root]);
    let engine = Engine::new().build().expect("Engine builds");

    let admitted = engine
        .admit_linked_module_closure(closure)
        .expect("the complete checked closure admits through the Engine-owned boundary");
    let (request, _cancellation) = engine
        .new_admitted_program_request(&admitted, None)
        .expect("only the Engine issues an execution request after admission");
    let terminal = futures::executor::block_on(engine.execute_admitted_program(&request))
        .expect("the Engine-issued request executes through the checked CPS route");

    assert_eq!(
        terminal,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(7)),
        "the admitted root program returns its canonical terminal value"
    );
}

#[test]
fn missing_declared_dependency_rejects_before_execution_route_creation() {
    let fixture = linked_fixture();
    let closure = LinkedModuleClosure::new(fixture.root_key, vec![fixture.root]);
    let engine = Engine::new().build().expect("Engine builds");

    assert!(
        engine.admit_linked_module_closure(closure).is_err(),
        "the Engine must reject a closure that omits a declared dependency before issuing a route"
    );
}

#[test]
fn core_cps_artifact_origin_mismatch_rejects_before_execution_route_creation() {
    let fixture = linked_fixture();
    let closure = LinkedModuleClosure::new(
        fixture.root_key,
        vec![fixture.dependency, fixture.root_with_mismatched_cps_origin],
    );
    let engine = Engine::new().build().expect("Engine builds");

    assert!(
        engine.admit_linked_module_closure(closure).is_err(),
        "the Engine must reject Core/CPS provenance whose canonical key agrees but origin differs"
    );
}

#[test]
fn failed_dependency_rejects_before_an_admitted_execution_route_exists() {
    let fixture = linked_fixture();
    let closure = LinkedModuleClosure::new(
        fixture.root_key,
        vec![fixture.root, fixture.failed_dependency],
    );
    let engine = Engine::new().build().expect("Engine builds");

    assert!(
        engine.admit_linked_module_closure(closure).is_err(),
        "a failed dependency must reject before the Engine creates an admitted execution route"
    );
}
