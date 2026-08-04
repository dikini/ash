//! TASK-2062 RED contracts for module-aware Core/CPS lowering.
//!
//! The lowering boundary receives only finalizer-issued module interfaces,
//! resolver-produced binding facts, and an already-materialized Core program.
//! It must not rediscover parser source or rely on Engine state.

use ash_core::Visibility;
use ash_core::core_ash::{CoreAtom, CoreExpr, CoreRow};
use ash_core::core_ash_lower::CoreLoweringContext;
use ash_core::core_ash_typecheck::CoreTypeCheckEnv;
use ash_core::core_ash_validate::RawCoreProgram;
use ash_core::cps::ContRef;
use ash_core::module_graph::{ModuleArtifact, ModuleArtifactOrigin, ModuleKey};
use ash_core::module_interface::{
    ModuleInterfaceBinding, ModuleInterfaceBindingKind, ModuleInterfaceDeclarationIdentity,
    ModuleInterfaceDefiningIdentity, PublicModuleInterface,
};
use ash_core::module_lowering::{ModuleCoreArtifact, ModuleCpsArtifact, ResolvedModuleImport};
use ash_parser::module::{ModuleBody, ModuleItem};
use ash_parser::{ModuleUnit, parse_surface_file};
use ash_typeck::TypeEnv;
use ash_typeck::interface_import_resolver::{
    CheckedInterfaceStore, InterfaceImportDiagnostic, InterfaceImportEnvironment,
    InterfaceImportPath, InterfaceImportRequest, InterfaceImportResolver,
};
use ash_typeck::module_core_cps_lowering::{
    ExpectedResolvedImport, ModuleCoreCpsLoweringError, lower_finalized_module_to_core_cps,
};
use ash_typeck::module_interface_finalization::{
    FinalizedModuleInterface, TypeEnvModuleInterfaceCollection,
};

fn root_key(crate_name: &str) -> ModuleKey {
    ModuleKey::root(crate_name).expect("fixture crate name is canonical")
}

fn child(parent: &ModuleKey, name: &str) -> ModuleKey {
    parent
        .child(name)
        .expect("fixture module name is canonical")
}

fn artifact(
    key: ModuleKey,
    origin: ModuleArtifactOrigin,
    structural_parent: Option<ModuleKey>,
    children: Vec<ModuleKey>,
) -> ModuleArtifact {
    ModuleArtifact::new(key, origin, structural_parent, children)
        .expect("fixture artifact is structurally valid")
}

fn public_callable(
    visible_name: &str,
    module: ModuleKey,
    origin: ModuleArtifactOrigin,
) -> ModuleInterfaceBinding {
    ModuleInterfaceBinding::declaration(
        visible_name,
        module,
        visible_name,
        ModuleInterfaceBindingKind::Callable,
        Visibility::Public,
        origin,
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

fn self_contained_core_request() -> (RawCoreProgram, CoreTypeCheckEnv, CoreLoweringContext) {
    (
        RawCoreProgram::new(CoreExpr::Atom(CoreAtom::LitInt(7))),
        CoreTypeCheckEnv::default(),
        CoreLoweringContext::new(ContRef::Label("halt".into()), CoreRow::default()),
    )
}

fn core_request_that_would_fail_type_check()
-> (RawCoreProgram, CoreTypeCheckEnv, CoreLoweringContext) {
    (
        RawCoreProgram::new(CoreExpr::Atom(CoreAtom::Var(
            "not_in_the_environment".into(),
        ))),
        CoreTypeCheckEnv::default(),
        CoreLoweringContext::new(ContRef::Label("halt".into()), CoreRow::default()),
    )
}

struct LoweringFixture {
    subject: FinalizedModuleInterface,
    imports: InterfaceImportEnvironment,
    remote_identity: ModuleInterfaceDefiningIdentity,
    remote_origin: ModuleArtifactOrigin,
}

fn fixture_with_resolved_remote_alias() -> LoweringFixture {
    let garden = root_key("garden");
    let api = child(&garden, "api");
    let garden_artifact = artifact(
        garden.clone(),
        ModuleArtifactOrigin::File("src/garden.ash".into()),
        None,
        vec![api.clone()],
    );
    let garden_unit = parser_module_unit(garden_artifact.clone(), "pub mod api;");
    let garden_interface = finalize(
        &garden_unit,
        PublicModuleInterface::new(
            garden_artifact,
            vec![ModuleInterfaceBinding::child(
                "api",
                api.clone(),
                Visibility::Public,
                ModuleArtifactOrigin::Inline {
                    parent: garden.clone(),
                    declaration_offset: 0,
                },
            )],
        )
        .expect("root raw carrier is export-closed"),
    );

    let api_origin = ModuleArtifactOrigin::Inline {
        parent: garden.clone(),
        declaration_offset: 0,
    };
    let api_artifact = artifact(api.clone(), api_origin.clone(), Some(garden), Vec::new());
    let api_unit = parser_module_unit(api_artifact.clone(), "pub fn serve() {}");
    let remote = public_callable("serve", api, api_origin);
    let api_interface = finalize(
        &api_unit,
        PublicModuleInterface::new(api_artifact, vec![remote.clone()])
            .expect("inline child raw carrier is export-closed"),
    );

    let subject_key = root_key("client");
    let subject_artifact = artifact(
        subject_key.clone(),
        ModuleArtifactOrigin::File("src/client.ash".into()),
        None,
        Vec::new(),
    );
    let subject_unit = parser_module_unit(subject_artifact.clone(), "pub fn main() {}");
    let subject = finalize(
        &subject_unit,
        PublicModuleInterface::new(
            subject_artifact.clone(),
            vec![public_callable(
                "main",
                subject_key,
                subject_artifact.origin().clone(),
            )],
        )
        .expect("subject raw carrier is export-closed"),
    );

    let store = CheckedInterfaceStore::new(vec![garden_interface, api_interface])
        .expect("fixture finalized interfaces have unique canonical identities");
    let resolver = InterfaceImportResolver::new(store);
    let mut imports = InterfaceImportEnvironment::new();
    resolver
        .resolve(
            &mut imports,
            &[InterfaceImportRequest::explicit(
                import_path(&["garden", "api", "serve"]),
                "remote",
            )],
        )
        .expect("checked public child declaration resolves under an alias");

    LoweringFixture {
        subject,
        imports,
        remote_identity: remote.defining_identity().clone(),
        remote_origin: remote.origin().clone(),
    }
}

fn ambiguous_import_environment() -> InterfaceImportEnvironment {
    let garden = root_key("garden");
    let alpha = child(&garden, "alpha");
    let beta = child(&garden, "beta");
    let garden_artifact = artifact(
        garden.clone(),
        ModuleArtifactOrigin::File("src/garden.ash".into()),
        None,
        vec![alpha.clone(), beta.clone()],
    );
    let garden_unit = parser_module_unit(garden_artifact.clone(), "pub mod alpha;\npub mod beta;");
    let garden_interface = finalize(
        &garden_unit,
        PublicModuleInterface::new(
            garden_artifact,
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
    let alpha_artifact = artifact(
        alpha.clone(),
        ModuleArtifactOrigin::File("src/alpha.ash".into()),
        Some(garden.clone()),
        Vec::new(),
    );
    let alpha_unit = parser_module_unit(alpha_artifact.clone(), "pub fn serve() {}");
    let alpha_interface = finalize(
        &alpha_unit,
        PublicModuleInterface::new(
            alpha_artifact.clone(),
            vec![public_callable(
                "serve",
                alpha,
                alpha_artifact.origin().clone(),
            )],
        )
        .expect("alpha raw carrier is export-closed"),
    );
    let beta_artifact = artifact(
        beta.clone(),
        ModuleArtifactOrigin::File("src/beta.ash".into()),
        Some(garden),
        Vec::new(),
    );
    let beta_unit = parser_module_unit(beta_artifact.clone(), "pub fn serve() {}");
    let beta_interface = finalize(
        &beta_unit,
        PublicModuleInterface::new(
            beta_artifact.clone(),
            vec![public_callable(
                "serve",
                beta,
                beta_artifact.origin().clone(),
            )],
        )
        .expect("beta raw carrier is export-closed"),
    );

    let store = CheckedInterfaceStore::new(vec![garden_interface, alpha_interface, beta_interface])
        .expect("fixture finalized interfaces have unique canonical identities");
    let resolver = InterfaceImportResolver::new(store);
    let mut imports = InterfaceImportEnvironment::new();
    resolver
        .resolve(
            &mut imports,
            &[
                InterfaceImportRequest::glob(import_path(&["garden", "alpha"])),
                InterfaceImportRequest::glob(import_path(&["garden", "beta"])),
            ],
        )
        .expect("distinct glob candidates enter the checked import environment");

    imports
}

fn assert_import_preserved(
    imports: &[ResolvedModuleImport],
    expected_identity: &ModuleInterfaceDefiningIdentity,
    expected_origin: &ModuleArtifactOrigin,
) {
    assert_eq!(imports.len(), 1, "one requested import is retained");
    let imported = &imports[0];
    assert_eq!(imported.local_name(), "remote");
    assert_eq!(imported.binding().defining_identity(), expected_identity);
    assert_eq!(imported.binding().origin(), expected_origin);
}

fn assert_module_facts_preserved(
    core: &ModuleCoreArtifact,
    cps: &ModuleCpsArtifact,
    fixture: &LoweringFixture,
) {
    let expected_artifact = fixture.subject.interface().artifact();
    assert_eq!(core.module_artifact(), expected_artifact);
    assert_eq!(cps.module_artifact(), expected_artifact);
    assert_eq!(core.module_artifact().key(), fixture.subject.module_key());
    assert_eq!(cps.module_artifact().origin(), expected_artifact.origin());
    assert_import_preserved(
        core.imports(),
        &fixture.remote_identity,
        &fixture.remote_origin,
    );
    assert_import_preserved(
        cps.imports(),
        &fixture.remote_identity,
        &fixture.remote_origin,
    );
}

#[test]
fn finalized_module_lowering_preserves_exact_artifact_and_aliased_defining_identity() {
    let fixture = fixture_with_resolved_remote_alias();
    let expected_import = ExpectedResolvedImport::new("remote", fixture.remote_identity.clone());
    let (raw_program, type_env, lowering_context) = self_contained_core_request();

    let (core, cps) = lower_finalized_module_to_core_cps(
        &fixture.subject,
        &fixture.imports,
        &[expected_import],
        raw_program,
        type_env,
        lowering_context,
    )
    .expect("a finalized module and checked alias lower without source or Engine authority");

    assert_module_facts_preserved(&core, &cps, &fixture);
}

#[test]
fn unresolved_or_ambiguous_requested_import_rejects_before_core_lowering() {
    let fixture = fixture_with_resolved_remote_alias();
    let (raw_program, type_env, lowering_context) = core_request_that_would_fail_type_check();

    let unresolved = lower_finalized_module_to_core_cps(
        &fixture.subject,
        &fixture.imports,
        &[ExpectedResolvedImport::new(
            "missing",
            fixture.remote_identity.clone(),
        )],
        raw_program,
        type_env,
        lowering_context,
    )
    .expect_err("a missing requested local import rejects before Core type checking");
    assert!(matches!(
        unresolved,
        ModuleCoreCpsLoweringError::ImportResolution(
            InterfaceImportDiagnostic::UnresolvedBinding { local_name }
        ) if local_name == "missing"
    ));

    let ambiguous_imports = ambiguous_import_environment();
    let (raw_program, type_env, lowering_context) = core_request_that_would_fail_type_check();
    let ambiguous = lower_finalized_module_to_core_cps(
        &fixture.subject,
        &ambiguous_imports,
        &[ExpectedResolvedImport::new(
            "serve",
            fixture.remote_identity.clone(),
        )],
        raw_program,
        type_env,
        lowering_context,
    )
    .expect_err("an ambiguous glob binding rejects before Core type checking");
    assert!(matches!(
        ambiguous,
        ModuleCoreCpsLoweringError::ImportResolution(
            InterfaceImportDiagnostic::AmbiguousBinding { local_name }
        ) if local_name == "serve"
    ));
}

#[test]
fn mutated_expected_defining_identity_rejects_the_stale_resolved_import_fact() {
    let fixture = fixture_with_resolved_remote_alias();
    let ModuleInterfaceDefiningIdentity::Declaration(actual) = &fixture.remote_identity else {
        panic!("fixture remote alias names a declaration");
    };
    let forged_identity =
        ModuleInterfaceDefiningIdentity::Declaration(ModuleInterfaceDeclarationIdentity::new(
            child(&root_key("garden"), "forged"),
            actual.name.clone(),
            actual.kind,
        ));
    let (raw_program, type_env, lowering_context) = self_contained_core_request();

    let error = lower_finalized_module_to_core_cps(
        &fixture.subject,
        &fixture.imports,
        &[ExpectedResolvedImport::new(
            "remote",
            forged_identity.clone(),
        )],
        raw_program,
        type_env,
        lowering_context,
    )
    .expect_err("a forged expected identity must not be rewritten from an alias");

    assert!(matches!(
        error,
        ModuleCoreCpsLoweringError::StaleResolvedImportIdentity {
            local_name,
            expected,
            actual,
        } if local_name == "remote"
            && expected == forged_identity
            && actual == fixture.remote_identity
    ));
}
