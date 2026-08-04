//! TASK-2066 RED contracts for finalizing checked module interfaces in type checking.
//!
//! The boundary consumes parser-owned [`ash_parser::ModuleUnit`] acquisition
//! facts and a type-checker collection. A raw Core carrier never becomes an
//! import authority until the collection finalizes it.

use ash_core::Visibility;
use ash_core::module_graph::{ModuleArtifact, ModuleArtifactOrigin, ModuleKey};
use ash_core::module_interface::{
    ModuleInterfaceBinding, ModuleInterfaceBindingKind, PublicModuleInterface,
};
use ash_parser::module::{ModuleDecl, ModuleItem};
use ash_parser::parse_utils::CommentTable;
use ash_parser::surface::{
    Definition, Expr, FnDef, Literal, MacroDef, Visibility as SurfaceVisibility,
};
use ash_parser::{ModuleBody, ModuleUnit, Span, parse_surface_file};
use ash_typeck::TypeEnv;
use ash_typeck::module_interface_finalization::{
    FinalizedModuleInterface, ModuleInterfaceFinalizationError, TypeEnvModuleInterfaceCollection,
};
use proptest::prelude::*;

fn module_unit_and_raw_interface(crate_name: &str) -> (ModuleUnit, PublicModuleInterface) {
    let key = ModuleKey::root(crate_name).expect("fixture crate name is canonical");
    let artifact = ModuleArtifact::new(
        key,
        ModuleArtifactOrigin::File(format!("src/{crate_name}.ash")),
        None,
        Vec::new(),
    )
    .expect("fixture artifact is structurally valid");
    let unit = ModuleUnit::new(
        artifact.clone(),
        ModuleBody::empty(Span::default()),
        None,
        CommentTable::default(),
    );
    let raw = PublicModuleInterface::new(artifact, Vec::new())
        .expect("an empty raw core interface is export-closed");

    (unit, raw)
}

fn module_unit_with_public_callable_and_raw_interface(
    crate_name: &str,
) -> (ModuleUnit, PublicModuleInterface) {
    let key = ModuleKey::root(crate_name).expect("fixture crate name is canonical");
    let artifact = ModuleArtifact::new(
        key.clone(),
        ModuleArtifactOrigin::File(format!("src/{crate_name}.ash")),
        None,
        Vec::new(),
    )
    .expect("fixture artifact is structurally valid");
    let span = Span::default();
    let unit = ModuleUnit::new(
        artifact.clone(),
        ModuleBody::from_definitions(
            vec![Definition::Function(FnDef {
                visibility: SurfaceVisibility::Public,
                name: "serve".into(),
                type_params: Vec::new(),
                params: Vec::new(),
                return_type: None,
                proposition_tail: None,
                contract: None,
                body: Expr::Literal(Literal::Int(0)),
                span,
            })],
            span,
        ),
        None,
        CommentTable::default(),
    );
    let raw = PublicModuleInterface::new(
        artifact,
        vec![ModuleInterfaceBinding::declaration(
            "serve",
            key,
            "serve",
            ModuleInterfaceBindingKind::Callable,
            Visibility::Public,
            ModuleArtifactOrigin::File(format!("src/{crate_name}.ash")),
        )],
    )
    .expect("the callable raw carrier is export-closed");

    (unit, raw)
}

fn finalized_view(finalized: &FinalizedModuleInterface) -> &PublicModuleInterface {
    finalized.interface()
}

#[test]
fn typeenv_collection_finalizes_only_the_matching_module_unit_and_core_artifact() {
    let (module_unit, raw_interface) = module_unit_and_raw_interface("garden");
    let mut type_env = TypeEnv::new();
    let collection = TypeEnvModuleInterfaceCollection::collect(&mut type_env, &module_unit)
        .expect("an empty parser module unit has collectable type-checker provenance");

    let finalized: FinalizedModuleInterface = collection
        .finalize(raw_interface)
        .expect("a collected unit finalizes only its matching raw core interface");

    assert_eq!(finalized.module_key(), module_unit.artifact().key());
    assert_eq!(
        finalized_view(&finalized).artifact(),
        module_unit.artifact()
    );
}

#[test]
fn raw_core_interface_for_a_different_module_rejects_before_finalization() {
    let (module_unit, _) = module_unit_and_raw_interface("garden");
    let (_, mismatched_raw_interface) = module_unit_and_raw_interface("orchard");
    let mut type_env = TypeEnv::new();
    let collection = TypeEnvModuleInterfaceCollection::collect(&mut type_env, &module_unit)
        .expect("fixture parser/type checker facts collect");

    let error = collection
        .finalize(mismatched_raw_interface)
        .expect_err("a raw Core carrier cannot bypass the parser/type-checker module identity");

    assert!(matches!(
        error,
        ModuleInterfaceFinalizationError::ArtifactKeyMismatch { .. }
    ));
}

#[test]
fn matching_key_with_changed_artifact_facts_rejects_before_finalization() {
    let key = ModuleKey::root("garden").expect("fixture crate name is canonical");
    let collected_artifact = ModuleArtifact::new(
        key.clone(),
        ModuleArtifactOrigin::File("src/garden.ash".into()),
        None,
        Vec::new(),
    )
    .expect("fixture artifact is structurally valid");
    let module_unit = ModuleUnit::new(
        collected_artifact.clone(),
        ModuleBody::empty(Span::default()),
        None,
        CommentTable::default(),
    );
    let changed_artifact = ModuleArtifact::new(
        key,
        ModuleArtifactOrigin::File("src/other-garden.ash".into()),
        None,
        Vec::new(),
    )
    .expect("same-key fixture artifact is structurally valid");
    let raw_interface = PublicModuleInterface::new(changed_artifact, Vec::new())
        .expect("the raw core carrier is export-closed");
    let mut type_env = TypeEnv::new();
    let collection = TypeEnvModuleInterfaceCollection::collect(&mut type_env, &module_unit)
        .expect("fixture parser/type checker facts collect");

    let error = collection
        .finalize(raw_interface)
        .expect_err("a key match cannot hide different artifact facts");

    assert!(matches!(
        error,
        ModuleInterfaceFinalizationError::ArtifactMismatch { .. }
    ));
}

#[test]
fn parser_backed_public_child_binding_finalizes() {
    let root = ModuleKey::root("garden").expect("fixture crate name is canonical");
    let child = root.child("api").expect("fixture child name is canonical");
    let artifact = ModuleArtifact::new(
        root,
        ModuleArtifactOrigin::File("src/garden.ash".into()),
        None,
        vec![child.clone()],
    )
    .expect("fixture artifact is structurally valid");
    let span = Span::default();
    let module_unit = ModuleUnit::new(
        artifact.clone(),
        ModuleBody::from_items(
            vec![ModuleItem::ModuleDecl(ModuleDecl::file(
                "api".into(),
                SurfaceVisibility::Public,
                span,
            ))],
            span,
        ),
        None,
        CommentTable::default(),
    );
    let child_binding = ModuleInterfaceBinding::child(
        "api",
        child,
        Visibility::Public,
        ModuleArtifactOrigin::File("src/api.ash".into()),
    );
    let raw_interface = PublicModuleInterface::new(artifact, vec![child_binding.clone()])
        .expect("the raw child binding is export-closed");
    let mut type_env = TypeEnv::new();

    let finalized = TypeEnvModuleInterfaceCollection::collect(&mut type_env, &module_unit)
        .expect("parser public child collection succeeds")
        .finalize(raw_interface)
        .expect("matching public child finalizes");

    assert_eq!(
        finalized.interface().bindings(),
        std::slice::from_ref(&child_binding)
    );
}

#[test]
fn parser_backed_public_macro_finalizes_as_syntax_only_metadata() {
    let key = ModuleKey::root("garden").expect("fixture crate name is canonical");
    let artifact = ModuleArtifact::new(
        key.clone(),
        ModuleArtifactOrigin::File("src/garden.ash".into()),
        None,
        Vec::new(),
    )
    .expect("fixture artifact is structurally valid");
    let span = Span::default();
    let module_unit = ModuleUnit::new(
        artifact.clone(),
        ModuleBody::from_items(
            vec![ModuleItem::Definition(Definition::Macro(MacroDef {
                visibility: SurfaceVisibility::Public,
                name: "rewrite".into(),
                params: Vec::new(),
                typed_signature: None,
                body: Expr::Literal(Literal::Int(0)),
                span,
            }))],
            span,
        ),
        None,
        CommentTable::default(),
    );
    let raw_interface = PublicModuleInterface::new(
        artifact,
        vec![ModuleInterfaceBinding::declaration(
            "rewrite",
            key,
            "rewrite",
            ModuleInterfaceBindingKind::SyntaxMacro,
            Visibility::Public,
            ModuleArtifactOrigin::File("src/garden.ash".into()),
        )],
    )
    .expect("the raw macro carrier is export-closed");
    let mut type_env = TypeEnv::new();

    let finalized = TypeEnvModuleInterfaceCollection::collect(&mut type_env, &module_unit)
        .expect("parser public macro collection succeeds")
        .finalize(raw_interface)
        .expect("matching public macro finalizes");
    let macro_binding = &finalized.interface().bindings()[0];

    assert_eq!(
        macro_binding.kind(),
        ModuleInterfaceBindingKind::SyntaxMacro
    );
    assert!(!macro_binding.is_runtime_callable());
}

#[test]
fn empty_module_unit_collection_rejects_an_uncollected_public_declaration() {
    let (module_unit, empty_raw_interface) = module_unit_and_raw_interface("garden");
    let uncollected = ModuleInterfaceBinding::declaration(
        "serve",
        module_unit.artifact().key().clone(),
        "serve",
        ModuleInterfaceBindingKind::Value,
        Visibility::Public,
        ModuleArtifactOrigin::File("src/garden.ash".into()),
    );
    let raw_interface =
        PublicModuleInterface::new(empty_raw_interface.artifact().clone(), vec![uncollected])
            .expect("the raw Core carrier itself is structurally export-closed");
    let mut type_env = TypeEnv::new();
    let collection = TypeEnvModuleInterfaceCollection::collect(&mut type_env, &module_unit)
        .expect("the empty parser module unit has collectable provenance");

    let error = collection
        .finalize(raw_interface)
        .expect_err("a public declaration needs parser/type-checker collection evidence");

    assert!(matches!(
        error,
        ModuleInterfaceFinalizationError::UncollectedPublicBinding { .. }
    ));
}

#[test]
fn public_callable_collection_registers_its_module_facts_and_rejects_conflicting_markers() {
    let (module_unit, raw_interface) = module_unit_with_public_callable_and_raw_interface("garden");
    let mut unregistered_type_env = TypeEnv::new();

    let finalized =
        TypeEnvModuleInterfaceCollection::collect(&mut unregistered_type_env, &module_unit)
            .expect("collection registers a fresh TypeEnv's parser callable facts")
            .finalize(raw_interface)
            .expect("the collected callable finalizes its matching raw projection");

    assert_eq!(
        finalized.interface().bindings()[0].kind(),
        ModuleInterfaceBindingKind::Callable
    );

    let conflicting_program = parse_surface_file("pub handler serve(comp: Unit) -> Unit { comp }")
        .expect("the conflicting handler fixture parses");
    let mut conflicting_type_env = TypeEnv::new();
    conflicting_type_env
        .register_surface_declarations(&conflicting_program.definitions)
        .expect("the conflicting handler marker registers");

    let error = TypeEnvModuleInterfaceCollection::collect(&mut conflicting_type_env, &module_unit)
        .expect_err("a handler marker cannot justify the function's callable fact");

    assert!(matches!(
        error,
        ModuleInterfaceFinalizationError::UnregisteredPublicCallable { .. }
    ));
}

#[test]
fn typeenv_collection_rejects_same_named_callable_from_a_different_module() {
    let (garden_unit, garden_raw_interface) =
        module_unit_with_public_callable_and_raw_interface("garden");
    let (orchard_unit, _) = module_unit_with_public_callable_and_raw_interface("orchard");
    let mut type_env = TypeEnv::new();
    TypeEnvModuleInterfaceCollection::collect(&mut type_env, &garden_unit)
        .expect("the TypeEnv collection registers and accepts garden's callable")
        .finalize(garden_raw_interface)
        .expect("garden's matching raw projection finalizes");

    let error = TypeEnvModuleInterfaceCollection::collect(&mut type_env, &orchard_unit).expect_err(
        "a TypeEnv finalization context cannot reuse garden's callable fact for orchard",
    );

    assert!(matches!(
        error,
        ModuleInterfaceFinalizationError::TypeEnvModuleKeyMismatch { .. }
    ));
}

#[test]
fn public_callable_with_an_unknown_declared_type_rejects_collection() {
    let key = ModuleKey::root("garden").expect("fixture crate name is canonical");
    let artifact = ModuleArtifact::new(
        key.clone(),
        ModuleArtifactOrigin::File("src/garden.ash".into()),
        None,
        Vec::new(),
    )
    .expect("fixture artifact is structurally valid");
    let parsed = parse_surface_file("pub fn serve(value: MissingType) { value }")
        .expect("the unknown-type function remains parser-valid surface syntax");
    let module_unit = ModuleUnit::new(
        artifact.clone(),
        ModuleBody::from_definitions(parsed.definitions, parsed.span),
        parsed.path,
        parsed.comments,
    );
    let _raw_interface = PublicModuleInterface::new(
        artifact,
        vec![ModuleInterfaceBinding::declaration(
            "serve",
            key,
            "serve",
            ModuleInterfaceBindingKind::Callable,
            Visibility::Public,
            ModuleArtifactOrigin::File("src/garden.ash".into()),
        )],
    )
    .expect("the callable raw carrier is export-closed");
    let mut type_env = TypeEnv::new();

    let error = TypeEnvModuleInterfaceCollection::collect(&mut type_env, &module_unit)
        .expect_err("collection must reject an unresolved public callable declaration type");
    assert!(matches!(
        error,
        ModuleInterfaceFinalizationError::TypeEnvDeclarationRegistration { .. }
    ));
}

#[test]
fn finalized_interface_exposes_only_an_immutable_inner_core_projection() {
    let (module_unit, raw_interface) = module_unit_and_raw_interface("garden");
    let mut type_env = TypeEnv::new();
    let finalized = TypeEnvModuleInterfaceCollection::collect(&mut type_env, &module_unit)
        .expect("fixture parser/type checker facts collect")
        .finalize(raw_interface)
        .expect("matching collection finalizes the core projection");
    let retained = finalized_view(&finalized).clone();

    assert_eq!(finalized_view(&finalized), &retained);
    assert_eq!(
        finalized_view(&finalized).artifact().key(),
        finalized.module_key(),
        "the immutable projection retains the finalized module identity"
    );
}

proptest! {
    #[test]
    fn matching_module_finalization_is_deterministic(crate_name in "[a-z][a-z0-9_]{0,7}") {
        let (first_unit, first_raw) = module_unit_and_raw_interface(&crate_name);
        let (second_unit, second_raw) = module_unit_and_raw_interface(&crate_name);
        let mut first_type_env = TypeEnv::new();
        let mut second_type_env = TypeEnv::new();

        let first = TypeEnvModuleInterfaceCollection::collect(&mut first_type_env, &first_unit)
            .expect("generated canonical fixture collects")
            .finalize(first_raw)
            .expect("matching generated fixture finalizes");
        let second = TypeEnvModuleInterfaceCollection::collect(&mut second_type_env, &second_unit)
            .expect("generated canonical fixture collects")
            .finalize(second_raw)
            .expect("matching generated fixture finalizes");

        prop_assert_eq!(first.module_key(), second.module_key());
        prop_assert_eq!(first.interface(), second.interface());
    }
}
