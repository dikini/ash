//! TASK-2073 checked-module finalization and export-closure RED inventory.
//!
//! The executable contract records the active Type-layer owner.  The focused
//! witnesses build real TASK-2075/TASK-2072 inputs and exercise the bounded
//! finalizer as an atomic checked handoff.  Their `red_` names retain the
//! activation inventory identifiers used by the task record.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::ModuleKey;
use ash_parser::{CanonicalExpandedModuleGraph, CanonicalModuleGraphResolver};
use ash_typeck::canonical_checked_module_finalizer::{
    CanonicalCheckedDeclarationFact, CanonicalCheckedModuleFinalization,
    CanonicalCheckedModuleFinalizationError, finalize_canonical_module_collection,
};
use ash_typeck::canonical_module_collection::{
    CanonicalDeclarationIdentity, CanonicalDeclarationKind, CanonicalModuleCollection,
    CanonicalNamespace, collect_canonical_expanded_module_graph,
};
use ash_typeck::{CanonicalParsedImportResult, resolve_parsed_imports_from_collection};
use proptest::prelude::*;

static NEXT_TEMP_TREE: AtomicUsize = AtomicUsize::new(0);

type NormalizedFinalInterfaceProjection = Vec<(
    ModuleKey,
    Vec<(Box<str>, CanonicalDeclarationIdentity)>,
    Vec<(Box<str>, CanonicalDeclarationIdentity)>,
)>;

/// A parser-backed fixture whose source files are removed after graph
/// acquisition.  The canonical graph and expanded graph retain their own
/// source-owned facts, so tests do not accidentally let the finalizer read the
/// fixture directory after this value is dropped.
struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(label: &str) -> Self {
        let serial = NEXT_TEMP_TREE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "ash-task-2073-finalization-{label}-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create TASK-2073 parser fixture tree");
        Self { root }
    }

    fn write(&self, relative: impl AsRef<Path>, source: &str) -> PathBuf {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("fixture path has a parent"))
            .expect("create TASK-2073 fixture parent directory");
        fs::write(&path, source).expect("write TASK-2073 parser fixture");
        path
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn expanded_graph(source: &str, label: &str) -> (ModuleKey, CanonicalExpandedModuleGraph) {
    let tree = TempTree::new(label);
    let root_path = tree.write("src/main.ash", source);
    let root = ModuleKey::root("app").expect("fixture crate key is canonical");
    let parsed = CanonicalModuleGraphResolver::new()
        .resolve_root(root.clone(), root_path)
        .expect("fixture source resolves through the canonical parser graph");
    let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect("fixture source expands through the canonical expanded graph");
    (root, expanded)
}

fn file_graph(
    root_source: &str,
    child_source: &str,
    label: &str,
) -> (ModuleKey, CanonicalExpandedModuleGraph) {
    let tree = TempTree::new(label);
    let root_path = tree.write("src/main.ash", root_source);
    tree.write("src/api.ash", child_source);
    let root = ModuleKey::root("app").expect("fixture crate key is canonical");
    let parsed = CanonicalModuleGraphResolver::new()
        .resolve_root(root.clone(), root_path)
        .expect("file-backed fixture resolves through the canonical parser graph");
    let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect("file-backed fixture expands through the canonical expanded graph");
    (root, expanded)
}

fn collected_inputs(
    root: ModuleKey,
    expanded: CanonicalExpandedModuleGraph,
) -> (
    ModuleKey,
    CanonicalExpandedModuleGraph,
    CanonicalModuleCollection,
    CanonicalParsedImportResult,
) {
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 supplies paired collection views before finalization");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 supplies an atomic parsed-import handoff before finalization");
    (root, expanded, collection, imports)
}

fn normalized_collection_projection(
    collection: &CanonicalModuleCollection,
) -> Vec<(ModuleKey, Vec<(String, CanonicalDeclarationKind)>)> {
    collection
        .modules()
        .map(|module| {
            let entries = module
                .internal_snapshot()
                .entries()
                .filter_map(|entry| {
                    entry
                        .declared_name()
                        .map(|name| (name.to_owned(), entry.kind()))
                })
                .collect();
            (module.module_key().clone(), entries)
        })
        .collect()
}

fn normalized_final_interface_projection(
    finalized: &CanonicalCheckedModuleFinalization,
) -> NormalizedFinalInterfaceProjection {
    finalized
        .modules()
        .map(|module| {
            let private = module
                .private_declarations()
                .map(|declaration| (declaration.name().into(), declaration.identity().clone()))
                .collect();
            let public = module
                .public_exports()
                .map(|export| {
                    (
                        export.local_name().into(),
                        export.defining_identity().clone(),
                    )
                })
                .collect();
            (module.module_key().clone(), private, public)
        })
        .collect()
}

#[test]
fn task_2073_activation_contract_is_recorded_before_finalization_implementation() {
    let task_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/plan/tasks/TASK-2073-checked-module-finalization-and-export-closure.md");
    let task = fs::read_to_string(&task_path).expect("TASK-2073 task file exists");

    assert!(task.contains("**Status:** In progress"));
    assert!(task.contains("**Owned rule:** MOD-REAL-003"));
    assert!(task.contains("CanonicalCollectedModuleSnapshot"));
    assert!(task.contains("CanonicalParsedImportResult"));
    assert!(task.contains("export closure"));
    assert!(task.contains("**Implementation:** partial"));
    assert!(task.contains("**Evidence:** tested"));
    assert!(task.contains("**Parity:** below_spec"));
    assert!(task.contains("## TDD Steps"));
    assert!(task.contains("atomic"));
    for red_case in [
        "red_checked_private_and_public_facts_preserve_provenance",
        "red_builtin_callable_finalization_preserves_public_projection",
        "red_builtin_public_signature_rejects_private_dependency",
        "red_public_callable_signature_imported_dependency_preserves_closure",
        "red_public_callable_imported_newtype_signature_preserves_closure",
        "red_public_function_missing_signature_dependency_rejects_atomically",
        "red_public_builtin_missing_signature_dependency_rejects_atomically",
        "red_public_handler_missing_signature_dependency_rejects_atomically",
        "red_public_callable_proposition_tail_private_dependency_rejects_atomically",
        "red_public_callable_proposition_tail_public_dependency_preserves_closure",
        "red_public_callable_proposition_tail_private_type_rejects_atomically",
        "red_public_callable_proposition_tail_private_row_rejects_atomically",
        "red_public_callable_proposition_tail_private_unqualified_row_dependency_rejects_atomically",
        "red_public_callable_proposition_tail_imported_private_dependency_rejects_atomically",
        "red_handler_callable_finalization_preserves_checked_body_fact",
        "red_public_impl_summary_preserves_body_free_metadata",
        "red_public_impl_private_type_dependency_rejects_atomically",
        "red_public_type_fact_preserves_checked_projection",
        "red_public_type_private_dependency_rejects_atomically",
        "red_public_type_missing_dependency_rejects_atomically",
        "red_public_type_imported_dependency_preserves_closure",
        "red_public_newtype_missing_dependency_rejects_atomically",
        "red_public_resource_missing_dependency_rejects_atomically",
        "red_public_type_domain_and_resource_facts_preserve_namespaces",
        "red_public_interface_fact_preserves_checked_projection",
        "red_public_interface_private_dependency_rejects_atomically",
        "red_public_interface_missing_dependency_rejects_atomically",
        "red_public_sealed_domain_fact_preserves_parent_scoped_constructors",
        "red_public_sealed_domain_private_dependency_rejects_atomically",
        "red_public_effect_row_facts_preserve_non_authorizing_metadata",
        "red_public_effect_row_private_dependency_rejects_atomically",
        "red_public_effect_row_private_role_dependency_rejects_atomically",
        "red_public_effect_row_private_policy_dependency_rejects_atomically",
        "red_public_effect_row_transitive_private_dependency_rejects_atomically",
        "red_public_effect_row_dependency_cycle_rejects_atomically",
        "red_public_effect_row_missing_dependency_rejects_atomically",
        "red_public_imported_row_private_dependency_rejects_atomically",
        "red_public_imported_role_row_private_dependency_rejects_atomically",
        "red_public_imported_role_row_public_dependency_preserves_closure",
        "red_public_imported_policy_row_private_dependency_rejects_atomically",
        "red_public_imported_policy_row_public_dependency_preserves_closure",
        "red_public_data_kind_and_predicate_facts_preserve_namespaces",
        "red_public_data_kind_missing_dependency_rejects_atomically",
        "red_public_imported_data_kind_private_dependency_rejects_atomically",
        "red_public_predicate_private_dependency_rejects_atomically",
        "red_public_role_fact_preserves_namespace_metadata",
        "red_public_type_function_fact_preserves_namespace_metadata",
        "red_public_type_function_private_dependency_rejects_atomically",
        "red_public_type_function_private_pattern_constructor_rejects_atomically",
        "red_public_type_function_private_proposition_dependency_rejects_atomically",
        "red_public_type_function_public_proposition_tail_preserves_metadata",
        "red_public_notation_fact_preserves_namespace_metadata",
        "red_public_notation_private_dependency_rejects_atomically",
        "red_public_notation_missing_dependency_rejects_atomically",
        "red_public_imported_notation_private_dependency_rejects_atomically",
        "red_public_macro_summary_fact_preserves_syntax_metadata",
        "red_public_macro_typed_signature_private_dependency_rejects_atomically",
        "red_public_macro_imported_callable_private_dependency_rejects_atomically",
        "red_public_evidence_facts_preserve_checked_namespace_metadata",
        "red_public_evidence_private_dependency_rejects_atomically",
        "red_public_evidence_imported_callable_private_dependency_rejects_atomically",
        "red_public_policy_imported_callable_private_dependency_rejects_atomically",
        "red_public_interface_law_preserves_parent_scoped_visibility",
        "red_public_interface_law_public_callable_dependency_preserves_closure",
        "red_public_interface_law_private_callable_dependency_rejects_atomically",
        "red_public_interface_law_imported_private_callable_dependency_rejects_atomically",
        "red_public_impl_proof_preserves_parent_scoped_visibility",
        "red_impl_proof_fact_preserves_interface_law_pair",
        "red_public_policy_named_binding_preserves_identity_and_schema",
        "red_public_policy_private_field_dependency_rejects_atomically",
        "red_public_policy_missing_field_dependency_rejects_atomically",
        "red_public_policy_default_type_mismatch_rejects_atomically",
        "red_public_policy_invariant_must_be_boolean_rejects_atomically",
        "red_final_pub_use_requires_export_closed_targets",
        "red_stale_forged_and_incomplete_inputs_reject_atomically",
        "red_imported_binding_visibility_drift_rejects_atomically",
        "red_file_and_inline_final_interfaces_have_equal_projection",
        "red_generated_finalization_closure_is_atomic",
        "red_finalizer_authority_fence_excludes_source_and_provisional_view",
    ] {
        assert!(
            !task.contains(red_case),
            "the task file describes the obligation, while this test target owns its executable RED inventory"
        );
    }
}

#[test]
fn red_checked_private_and_public_facts_preserve_provenance() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                fn normalize(value: Int) -> Int { value + 1 }
                pub fn expose(value: Int) -> Int { normalize(value) }
            }
            pub use crate::api::expose as exported;
        "#,
        "private-public-provenance",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");
    let snapshot = collection
        .internal_snapshot(&api)
        .expect("TASK-2075 exposes the checker-internal API snapshot");
    let private = snapshot
        .entries()
        .find(|entry| entry.declared_name() == Some("normalize"))
        .expect("private callable remains in the internal snapshot");
    let private_source_anchor = private.source_anchor();
    let public = snapshot
        .entries()
        .find(|entry| entry.declared_name() == Some("expose"))
        .expect("public callable remains in the internal snapshot");
    assert!(private.raw_definition().is_some());
    assert!(private.callable_body().is_some());
    assert_eq!(private.identity().module_key(), &api);
    assert_eq!(public.identity().module_key(), &api);
    let staged = imports
        .public_uses()
        .iter()
        .find(|public_use| public_use.binding().local_name() == "exported")
        .expect("TASK-2072 stages the public-use fact before finalization");
    assert_eq!(staged.binding().defining_identity().module_key(), &api);
    assert!(staged.binding().is_reexport());

    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("checked finalization publishes only after every module succeeds");
    let api_interface = finalized
        .module(&api)
        .expect("finalization publishes the checked child module");
    let private = api_interface
        .private_declaration("normalize")
        .expect("private callable remains in the private checked view");
    let public = api_interface
        .private_declaration("expose")
        .expect("public callable remains in the private checked view");
    assert_eq!(private.identity().module_key(), &api);
    assert_eq!(private.declaration_span(), private_source_anchor);
    assert!(private.body_span().is_some());
    assert_eq!(private.origin(), public.origin());
    assert!(private.signature().is_some());
    assert!(private.body_type().is_some());
    assert!(matches!(
        public.visibility(),
        ash_parser::surface::Visibility::Public
    ));
    let public_export = api_interface
        .public_export("expose")
        .expect("public callable is projected into the export view");
    assert_eq!(public_export.defining_identity(), public.identity());
    assert!(public_export.declaration().signature().is_some());
    assert!(api_interface.public_export("normalize").is_none());

    let root_interface = finalized
        .module(&root)
        .expect("finalization publishes the root module");
    let exported = root_interface
        .public_export("exported")
        .expect("finalization publishes the staged public re-export");
    assert_eq!(exported.defining_identity(), public.identity());
    assert!(root_interface.public_export("normalize").is_none());
}

#[test]
fn red_builtin_callable_finalization_preserves_public_projection() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub builtin fn host_add(value: Int) -> Int;
            }
            pub use crate::api::host_add as exported;
            use crate::api::host_add;
            pub fn call(value: Int) -> Int { host_add(value) }
        "#,
        "builtin-public-projection",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("bodyless builtin signatures are finalized atomically");
    let api_interface = finalized
        .module(&api)
        .expect("finalization publishes the builtin child module");
    let builtin = api_interface
        .private_declaration("host_add")
        .expect("builtin remains in the private checked view");
    assert_eq!(builtin.kind(), CanonicalDeclarationKind::BuiltinFn);
    assert!(builtin.signature().is_some());
    assert!(builtin.body_span().is_none());

    let public_export = api_interface
        .public_export("host_add")
        .expect("public builtin is projected into the child interface");
    assert_eq!(public_export.defining_identity(), builtin.identity());
    assert!(public_export.declaration().signature().is_some());

    let root_export = finalized
        .module(&root)
        .expect("finalization publishes the root module")
        .public_export("exported")
        .expect("staged builtin re-export is export-closed");
    assert_eq!(root_export.defining_identity(), builtin.identity());
    let root_call = finalized
        .module(&root)
        .expect("finalization publishes the root module")
        .private_declaration("call")
        .expect("root callable remains in the private checked view");
    assert!(root_call.signature().is_some());
    assert!(root_call.body_type().is_some());
}

#[test]
fn red_builtin_public_signature_rejects_private_dependency() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                type Hidden = Int;
                pub builtin fn expose(value: Hidden) -> Hidden;
            }
            pub use crate::api::expose as exported;
        "#,
        "builtin-private-signature",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public builtin signature cannot leak the private Hidden declaration");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "expose" && dependency.as_ref() == "Hidden"
    ));
}

#[test]
fn red_public_callable_signature_imported_dependency_preserves_closure() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod types {
                pub type Public = Int;
            }
            pub mod api {
                use crate::types::Public;
                pub fn expose(value: Public) -> Public { value }
            }
        "#,
        "public-callable-imported-signature-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("a public callable may depend on an imported public type");
    let expose = finalized
        .module(&api)
        .expect("finalization publishes the callable child module")
        .public_export("expose")
        .expect("the callable remains in the public export projection");
    assert!(expose.declaration().signature().is_some());
}

#[test]
fn red_public_callable_imported_newtype_signature_preserves_closure() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod types {
                pub newtype UserId = UserId(Int);
            }
            pub mod api {
                use crate::types::UserId;
                pub fn expose(value: UserId) -> UserId { value }
            }
        "#,
        "public-callable-imported-newtype-signature-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("a public callable may depend on an imported public newtype identity");
    assert!(
        finalized
            .module(&api)
            .expect("finalization publishes the callable child module")
            .public_export("expose")
            .is_some()
    );
}

#[test]
fn red_public_function_missing_signature_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub fn expose(value: Missing) -> Missing { value }
            }
        "#,
        "public-function-missing-signature-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public function cannot publish an unresolved signature type");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "expose" && dependency.as_ref() == "Missing"
    ));
}

#[test]
fn red_public_builtin_missing_signature_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub builtin fn expose(value: Missing) -> Missing;
            }
        "#,
        "public-builtin-missing-signature-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public builtin cannot publish an unresolved signature type");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "expose" && dependency.as_ref() == "Missing"
    ));
}

#[test]
fn red_public_handler_missing_signature_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub handler expose(value: Missing) -> Missing { value }
            }
        "#,
        "public-handler-missing-signature-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public handler cannot publish an unresolved signature type");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "expose" && dependency.as_ref() == "Missing"
    ));
}

#[test]
fn red_public_callable_proposition_tail_private_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                prop Hidden<X: Type>;
                pub fn expose(value: Int) -> Int where Hidden<Int> { value }
            }
        "#,
        "public-callable-proposition-tail-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public callable cannot publish a private proposition-tail predicate");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "expose" && dependency.as_ref() == "Hidden"
    ));
}

#[test]
fn red_public_callable_proposition_tail_public_dependency_preserves_closure() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub sealed type domain Tree { Leaf; }
                pub prop Visible<X: Tree>;
                pub fn expose(value: Int) -> Int where Visible<Tree> { value }
            }
        "#,
        "public-callable-proposition-tail-public-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("a public callable may retain a public proposition-tail predicate");
    assert!(
        finalized
            .module(&api)
            .expect("finalization publishes the callable child module")
            .public_export("expose")
            .is_some()
    );
}

#[test]
fn red_public_callable_proposition_tail_private_type_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                type Hidden = Int;
                pub fn expose(value: Int) -> Int where Hidden == Int { value }
            }
        "#,
        "public-callable-proposition-tail-private-type",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public callable cannot publish a private proposition-tail type");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "expose" && dependency.as_ref() == "Hidden"
    ));
}

#[test]
fn red_public_callable_proposition_tail_private_row_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                effect group HiddenAudit = { evidence audit_log };
                pub fn expose(value: Int) -> Int where row { group HiddenAudit } { value }
            }
        "#,
        "public-callable-proposition-tail-private-row",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public callable cannot publish a private proposition-tail row");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "expose" && dependency.as_ref() == "HiddenAudit"
    ));
}

#[test]
fn red_public_callable_proposition_tail_private_unqualified_row_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                effect alias HiddenAudit = { evidence audit_log };
                pub fn expose(value: Int) -> Int where row { HiddenAudit } { value }
            }
        "#,
        "public-callable-proposition-tail-private-unqualified-row-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let finalization = finalize_canonical_module_collection(&expanded, &collection, &imports);
    assert!(
        finalization.is_err(),
        "a private proposition-tail row dependency must publish no final interface"
    );
    let error = finalization
        .expect_err("a public callable cannot publish a private proposition-tail row alias");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "expose" && dependency.as_ref() == "HiddenAudit"
    ));
}

#[test]
fn red_public_callable_proposition_tail_imported_private_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod provider {
                pub sealed type domain Tree { Leaf; }
                pub(crate) prop Hidden<X: Tree>;
            }
            pub mod api {
                use crate::provider::Tree;
                use crate::provider::Hidden;
                pub fn expose(value: Int) -> Int where Hidden<Tree> { value }
            }
        "#,
        "public-callable-proposition-tail-imported-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public callable cannot expose an imported private proposition predicate");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "expose" && dependency.as_ref() == "Hidden"
    ));
}

#[test]
fn red_handler_callable_finalization_preserves_checked_body_fact() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                interface Clock<T> { sleep(Int) -> Int }
                type TestClock = SystemClock(Int);
                impl Clock<TestClock> { sleep(milliseconds) = 0 }
                pub handler h(comp: () -> { TestClock::sleep } Int) -> Int {
                    on comp {
                        TestClock::sleep(value, resume) => value,
                        done(result) => result
                    }
                }
            }
            pub use crate::api::h as exported;
        "#,
        "handler-checked-body",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("a valid handler declaration is finalized atomically");
    let api_interface = finalized
        .module(&api)
        .expect("finalization publishes the handler child module");
    let handler = api_interface
        .private_declaration("h")
        .expect("handler remains in the private checked view");
    assert_eq!(handler.kind(), CanonicalDeclarationKind::Handler);
    assert!(handler.signature().is_some());
    assert!(handler.body_span().is_some());
    assert!(handler.body_type().is_some());
    assert!(api_interface.public_export("h").is_some());

    let root_export = finalized
        .module(&root)
        .expect("finalization publishes the root module")
        .public_export("exported")
        .expect("staged handler re-export is export-closed");
    assert_eq!(root_export.defining_identity(), handler.identity());
}

#[test]
fn red_public_impl_summary_preserves_body_free_metadata() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub interface Eq<A> { equiv(A, A) -> Bool }
                pub impl Eq<Int> { equiv(a, b) = a == b }
            }
        "#,
        "public-impl-summary",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("public implementations finalize as body-free summaries");
    let api_interface = finalized
        .module(&api)
        .expect("finalization publishes the implementation child module");
    let implementation = api_interface
        .private_declarations()
        .find(|declaration| {
            declaration.kind() == CanonicalDeclarationKind::Impl && declaration.name() == "Eq"
        })
        .expect("implementation remains in the private checked view");
    assert!(implementation.body_span().is_none());
    assert!(implementation.body_type().is_none());
    assert!(matches!(
        implementation.fact(),
        CanonicalCheckedDeclarationFact::Implementation { summary }
            if summary.interface() == "Eq"
                && summary.type_args().len() == 1
                && summary.methods().iter().any(|name| name.as_ref() == "equiv")
                && summary.proofs().is_empty()
    ));
    let public_implementation = api_interface
        .public_export_in_namespace(CanonicalNamespace::ImplementationRegistry, "Eq")
        .expect("public implementation summary is export-closed");
    assert!(matches!(
        public_implementation.declaration().fact(),
        CanonicalCheckedDeclarationFact::Implementation { summary }
            if summary.interface() == "Eq" && summary.methods().len() == 1
    ));
    assert!(
        api_interface
            .public_export_in_namespace(CanonicalNamespace::ValueCallable, "equiv")
            .is_none(),
        "impl methods remain parent-scoped and non-standalone"
    );
}

#[test]
fn red_public_impl_private_type_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                type Hidden = Int;
                pub interface Eq<A> { equiv(A, A) -> Bool }
                pub impl Eq<Hidden> { equiv(a, b) = a == b }
            }
        "#,
        "public-impl-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public implementation cannot expose a private type argument");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "Eq" && dependency.as_ref() == "Hidden"
    ));
}

#[test]
fn red_public_impl_public_where_bound_preserves_export_closure() {
    let (root, expanded) = expanded_graph(
        r#"
            pub interface Show {}
            pub interface Eq<A> {}
            pub impl<T> Eq<T> where T: Show {}
        "#,
        "public-impl-public-where-bound",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);

    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("a public implementation may retain a public local where-bound interface");
    let implementation = finalized
        .module(&root)
        .expect("finalization publishes the root module")
        .public_export_in_namespace(CanonicalNamespace::ImplementationRegistry, "Eq")
        .expect("the public implementation remains in the implementation registry");
    assert!(matches!(
        implementation.declaration().fact(),
        CanonicalCheckedDeclarationFact::Implementation { summary }
            if summary.where_bounds().iter().any(|(parameter, bound)| {
                parameter.as_ref() == "T" && bound.as_ref() == "Show"
            })
    ));
}

#[test]
fn red_public_impl_private_where_bound_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            interface Show {}
            pub interface Eq<A> {}
            pub impl<T> Eq<T> where T: Show {}
        "#,
        "public-impl-private-where-bound",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public implementation cannot expose a private where-bound interface");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &root && name.as_ref() == "Eq" && dependency.as_ref() == "Show"
    ));
}

#[test]
fn red_public_impl_imported_private_where_bound_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod provider {
                pub(crate) interface Show {}
            }

            pub mod api {
                use crate::provider::Show;
                pub interface Eq<A> {}
                pub impl<T> Eq<T> where T: Show {}
            }
        "#,
        "public-impl-imported-private-where-bound",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public implementation cannot expose an imported private where-bound");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "Eq" && dependency.as_ref() == "Show"
    ));
}

#[test]
fn red_public_impl_missing_where_bound_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub interface Eq<A> {}
            pub impl<T> Eq<T> where T: Show {}
        "#,
        "public-impl-missing-where-bound",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public implementation cannot expose a missing where-bound interface");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &root && name.as_ref() == "Eq" && dependency.as_ref() == "Show"
    ));
}

#[test]
fn red_public_type_fact_preserves_checked_projection() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub type Id = Int;
                pub fn echo(value: Id) -> Id { value }
            }
            pub use crate::api::Id as exported_id;
            pub use crate::api::echo as exported_echo;
        "#,
        "public-type-projection",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("public type declarations and their dependent callables are finalized");
    let api_interface = finalized
        .module(&api)
        .expect("finalization publishes the typed child module");
    let id = api_interface
        .private_declaration("Id")
        .expect("public type remains in the private checked view");
    assert_eq!(id.kind(), CanonicalDeclarationKind::Type);
    assert!(matches!(
        id.fact(),
        CanonicalCheckedDeclarationFact::Type {
            body: ash_parser::surface::TypeBody::Alias(ash_parser::surface::Type::Name(name)),
            ..
        } if name.as_ref() == "Int"
    ));
    assert!(api_interface.public_export("Id").is_some());
    assert!(api_interface.public_export("echo").is_some());

    let root_interface = finalized
        .module(&root)
        .expect("finalization publishes the root module");
    assert_eq!(
        root_interface
            .public_export("exported_id")
            .expect("public type re-export is export-closed")
            .defining_identity(),
        id.identity()
    );
    assert_eq!(
        root_interface
            .public_export("exported_echo")
            .expect("dependent callable re-export is export-closed")
            .declaration()
            .kind(),
        CanonicalDeclarationKind::Function
    );
}

#[test]
fn red_public_type_private_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                type Hidden = Int;
                pub type Public = Hidden;
            }
            pub use crate::api::Public as exported;
        "#,
        "public-type-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public type representation cannot leak a private type");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "Public" && dependency.as_ref() == "Hidden"
    ));
}

#[test]
fn red_public_type_missing_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub type MissingAlias = Missing;
            }
        "#,
        "public-type-missing-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public type cannot publish an unresolved type dependency");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api
            && name.as_ref() == "MissingAlias"
            && dependency.as_ref() == "Missing"
    ));
}

#[test]
fn red_public_type_imported_dependency_preserves_closure() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod types {
                pub type Public = Int;
            }
            pub mod api {
                use crate::types::Public;
                pub type Alias = Public;
            }
        "#,
        "public-type-imported-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("a public type may depend on an imported public type");
    let api_interface = finalized
        .module(&api)
        .expect("finalization publishes the type child module");
    assert!(api_interface.public_export("Alias").is_some());
}

#[test]
fn red_public_newtype_missing_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub newtype Broken = Broken(Missing);
            }
        "#,
        "public-newtype-missing-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public newtype cannot publish an unresolved representation type");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "Broken" && dependency.as_ref() == "Missing"
    ));
}

#[test]
fn red_public_resource_missing_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub resource type Store { value: Missing }
            }
        "#,
        "public-resource-missing-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public resource cannot publish an unresolved field type");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "Store" && dependency.as_ref() == "Missing"
    ));
}

#[test]
fn red_public_type_domain_and_resource_facts_preserve_namespaces() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub newtype UserId = UserId(Int);
                pub resource type Store { value: Int }
            }
        "#,
        "public-domain-resource-projection",
    );
    let (_root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = ModuleKey::root("app")
        .expect("fixture crate key is canonical")
        .child("api")
        .expect("fixture child key is canonical");

    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("public newtype and resource facts are finalized");
    let api_interface = finalized
        .module(&api)
        .expect("finalization publishes the typed child module");
    let newtype = api_interface
        .private_declarations()
        .find(|declaration| declaration.kind() == CanonicalDeclarationKind::Newtype)
        .expect("newtype remains in the private checked view");
    let constructor = api_interface
        .private_declarations()
        .find(|declaration| {
            declaration.kind() == CanonicalDeclarationKind::Function
                && matches!(
                    declaration.fact(),
                    CanonicalCheckedDeclarationFact::Constructor { .. }
                )
        })
        .expect("newtype constructor remains in the private checked view");
    assert!(matches!(
        newtype.fact(),
        CanonicalCheckedDeclarationFact::Newtype { constructor, .. }
            if constructor.as_ref() == "UserId"
    ));
    assert!(matches!(
        constructor.fact(),
        CanonicalCheckedDeclarationFact::Constructor { parent, name }
            if parent == newtype.identity() && name.as_ref() == "UserId"
    ));
    assert!(matches!(
        api_interface
            .private_declarations()
            .find(|declaration| declaration.kind() == CanonicalDeclarationKind::ResourceType)
            .expect("resource type remains in the private checked view")
            .fact(),
        CanonicalCheckedDeclarationFact::ResourceType { fields }
            if fields.iter().any(|(name, _)| name.as_ref() == "value")
    ));
    assert!(
        api_interface
            .public_export_in_namespace(CanonicalNamespace::TypeDomain, "UserId")
            .is_some()
    );
    assert!(
        api_interface
            .public_export_in_namespace(CanonicalNamespace::ValueCallable, "UserId")
            .is_some()
    );
    assert!(
        api_interface
            .public_export_in_namespace(CanonicalNamespace::TypeDomain, "Store")
            .is_some()
    );
    assert!(
        api_interface.public_export("UserId").is_none(),
        "an unqualified lookup must not collapse type and value namespace facts"
    );
}

#[test]
fn red_public_interface_fact_preserves_checked_projection() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub interface Clock<T> { sleep(T) -> Int }
            }
            pub use crate::api::Clock as exported;
        "#,
        "public-interface-projection",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("public interface declarations and method metadata are finalized");
    let api_interface = finalized
        .module(&api)
        .expect("finalization publishes the interface child module");
    let clock = api_interface
        .private_declaration("Clock")
        .expect("public interface remains in the private checked view");
    assert!(matches!(
        clock.fact(),
        CanonicalCheckedDeclarationFact::Interface { definition, .. }
            if definition.name.as_ref() == "Clock"
                && definition.type_params.len() == 1
                && definition.methods.iter().any(|method| {
                    method.name.as_ref() == "sleep"
                        && method.params.len() == 1
                        && matches!(
                            method.params.first(),
                            Some(ash_parser::surface::Type::Name(name))
                                if name.as_ref() == "T"
                        )
                })
    ));
    assert!(api_interface.public_export("Clock").is_some());

    let root_interface = finalized
        .module(&root)
        .expect("finalization publishes the root module");
    let export = root_interface
        .public_export("exported")
        .expect("public interface re-export is export-closed");
    assert_eq!(export.defining_identity(), clock.identity());
    assert!(matches!(
        export.declaration().fact(),
        CanonicalCheckedDeclarationFact::Interface { definition, .. }
            if definition.name.as_ref() == "Clock"
    ));
}

#[test]
fn red_public_interface_private_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                type Hidden = Int;
                pub interface Exposes { read() -> Hidden }
            }
            pub use crate::api::Exposes as exported;
        "#,
        "public-interface-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public interface method cannot leak a private type");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "Exposes" && dependency.as_ref() == "Hidden"
    ));
}

#[test]
fn red_public_interface_missing_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub interface Exposes { read() -> Missing }
            }
        "#,
        "public-interface-missing-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public interface cannot publish an unresolved method type");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "Exposes" && dependency.as_ref() == "Missing"
    ));
}

#[test]
fn red_public_sealed_domain_fact_preserves_parent_scoped_constructors() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub sealed type domain TypeList {
                    Nil;
                    Cons<head: Type, tail: TypeList>;
                }
            }
            pub use crate::api::TypeList as exported;
        "#,
        "public-sealed-domain-projection",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("public sealed domains and marker constructors are finalized");
    let api_interface = finalized
        .module(&api)
        .expect("finalization publishes the sealed-domain child module");
    let domain = api_interface
        .private_declaration("TypeList")
        .expect("sealed domain remains in the private checked view");
    assert_eq!(domain.kind(), CanonicalDeclarationKind::SealedDomain);
    assert!(matches!(
        domain.fact(),
        CanonicalCheckedDeclarationFact::SealedDomain { definition }
            if definition.name.as_ref() == "TypeList"
                && definition.constructors.len() == 2
    ));
    let marker = api_interface
        .private_declarations()
        .find(|declaration| declaration.name() == "Cons")
        .expect("marker constructor remains parent-scoped in the private view");
    assert!(matches!(
        marker.fact(),
        CanonicalCheckedDeclarationFact::SealedDomainConstructor { parent, constructor }
            if parent == domain.identity()
                && constructor.name.as_ref() == "Cons"
                && constructor.fields.len() == 2
    ));
    assert!(matches!(
        marker.visibility(),
        ash_parser::surface::Visibility::Inherited
    ));
    assert!(api_interface.public_export("TypeList").is_some());
    assert!(
        api_interface.public_export("Cons").is_none(),
        "sealed-domain marker constructors remain parent-scoped"
    );

    let root_interface = finalized
        .module(&root)
        .expect("finalization publishes the root module");
    assert_eq!(
        root_interface
            .public_export("exported")
            .expect("public sealed-domain re-export is export-closed")
            .defining_identity(),
        domain.identity()
    );
}

#[test]
fn red_public_sealed_domain_private_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                sealed type domain Hidden { Secret; }
                pub sealed type domain Public { Box<item: Hidden>; }
            }
            pub use crate::api::Public as exported;
        "#,
        "public-sealed-domain-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public sealed domain cannot leak a private domain reference");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "Public" && dependency.as_ref() == "Hidden"
    ));
}

#[test]
fn red_public_effect_row_facts_preserve_non_authorizing_metadata() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub effect alias IO = { PosixFs::read, evidence audit_log };
                pub effect group WorkflowIO = { group IO };
            }
            pub use crate::api::IO as exported_alias;
            pub use crate::api::WorkflowIO as exported_group;
        "#,
        "public-effect-row-projection",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("public effect-row aliases and groups are finalized");
    let api_interface = finalized
        .module(&api)
        .expect("finalization publishes the effect-row child module");
    let alias = api_interface
        .private_declaration("IO")
        .expect("effect alias remains in the private checked view");
    let group = api_interface
        .private_declaration("WorkflowIO")
        .expect("effect group remains in the private checked view");
    assert!(matches!(
        alias.fact(),
        CanonicalCheckedDeclarationFact::EffectAlias { definition }
            if definition.name.as_ref() == "IO" && definition.row.items.len() == 2
    ));
    assert!(matches!(
        group.fact(),
        CanonicalCheckedDeclarationFact::EffectGroup { definition }
            if definition.name.as_ref() == "WorkflowIO" && definition.row.items.len() == 1
    ));
    assert!(api_interface.public_export("IO").is_some());
    assert!(api_interface.public_export("WorkflowIO").is_some());

    let root_interface = finalized
        .module(&root)
        .expect("finalization publishes the root module");
    assert_eq!(
        root_interface
            .public_export("exported_alias")
            .expect("public effect alias re-export is export-closed")
            .defining_identity(),
        alias.identity()
    );
    assert_eq!(
        root_interface
            .public_export("exported_group")
            .expect("public effect group re-export is export-closed")
            .defining_identity(),
        group.identity()
    );
}

#[test]
fn red_public_effect_row_private_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                effect alias Hidden = { evidence audit_log };
                pub effect group Published = { group Hidden };
            }
            pub use crate::api::Published as exported;
        "#,
        "public-effect-row-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public effect group cannot leak a private row alias");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "Published" && dependency.as_ref() == "Hidden"
    ));
}

#[test]
fn red_public_effect_row_private_role_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                role HiddenRole { capabilities: [], obligations: [audit_log] }
                pub effect alias Published = { role HiddenRole };
            }
            pub use crate::api::Published as exported;
        "#,
        "public-effect-row-private-role-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public effect alias cannot leak a private role row dependency");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "Published" && dependency.as_ref() == "HiddenRole"
    ));
}

#[test]
fn red_public_effect_row_private_policy_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                policy HiddenPolicy { marker: Int }
                pub effect alias Published = { policy HiddenPolicy };
            }
            pub use crate::api::Published as exported;
        "#,
        "public-effect-row-private-policy-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public effect alias cannot leak a private policy row dependency");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "Published" && dependency.as_ref() == "HiddenPolicy"
    ));
}

#[test]
fn red_public_qualified_effect_group_private_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod provider {
                pub(crate) effect alias Hidden = { evidence audit_log };
            }
            pub mod api {
                pub effect group Published = { group crate::provider::Hidden };
            }
        "#,
        "public-qualified-effect-group-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public effect group cannot expose a qualified private row alias");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "Published" && dependency.as_ref() == "Hidden"
    ));
}

#[test]
fn red_public_qualified_effect_group_public_dependency_preserves_closure() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod provider {
                pub effect alias Visible = { evidence audit_log };
            }
            pub mod api {
                pub effect group Published = { group crate::provider::Visible };
            }
        "#,
        "public-qualified-effect-group-public-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("a public effect group may expose a public qualified row alias");
    assert!(
        finalized
            .module(&api)
            .expect("api interface is finalized")
            .public_exports()
            .any(|export| export.local_name() == "Published")
    );
}

#[test]
fn red_public_effect_row_transitive_private_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                effect alias Hidden = { evidence audit_log };
                pub effect group PublishedGroup = { Hidden };
                pub effect alias PublishedAlias = { group PublishedGroup };
            }
            pub use crate::api::PublishedAlias as exported;
        "#,
        "public-effect-row-transitive-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let finalization = finalize_canonical_module_collection(&expanded, &collection, &imports);
    assert!(
        matches!(
            finalization,
            Err(CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
                ref module,
                ref name,
                ref dependency,
                ..
            }) if module == &api
                && name.as_ref() == "PublishedGroup"
                && dependency.as_ref() == "Hidden"
        ),
        "a public row dependency chain must reject its private leaf before publication"
    );
}

#[test]
fn red_public_effect_row_dependency_cycle_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub effect alias First = { group Second };
                pub effect group Second = { First };
            }
            pub use crate::api::First as exported;
        "#,
        "public-effect-row-dependency-cycle",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let finalization = finalize_canonical_module_collection(&expanded, &collection, &imports);
    let error = finalization
        .expect_err("a public effect-row dependency cycle must publish no final interface");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::CyclicPublicExportDependency {
            ref module,
            ..
        } if module == &api
    ));
}

#[test]
fn red_public_effect_row_missing_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub effect group Published = { group Missing };
            }
        "#,
        "public-effect-row-missing-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public effect group cannot publish an unresolved row dependency");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api
            && name.as_ref() == "Published"
            && dependency.as_ref() == "Missing"
    ));
}

#[test]
fn red_public_data_kind_and_predicate_facts_preserve_namespaces() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub type Tree = Leaf | Branch(Tree);
                pub data kind TreeKind from type Tree;
                pub prop NonEmpty<Xs: Tree>;
            }
            pub use crate::api::TreeKind as exported_kind;
            pub use crate::api::NonEmpty as exported_predicate;
        "#,
        "public-data-kind-predicate-projection",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("public promoted-kind and proposition metadata finalize atomically");
    let api_interface = finalized
        .module(&api)
        .expect("finalization publishes the metadata child module");
    let data_kind = api_interface
        .private_declaration("TreeKind")
        .expect("promoted data kind remains in the private checked view");
    assert_eq!(data_kind.namespace(), CanonicalNamespace::PromotedKind);
    assert!(matches!(
        data_kind.fact(),
        CanonicalCheckedDeclarationFact::DataKind { definition }
            if definition.name.as_ref() == "TreeKind"
                && definition.source_adt.as_ref() == "Tree"
    ));
    let predicate = api_interface
        .private_declaration("NonEmpty")
        .expect("proposition predicate remains in the private checked view");
    assert_eq!(predicate.namespace(), CanonicalNamespace::Proposition);
    assert!(matches!(
        predicate.fact(),
        CanonicalCheckedDeclarationFact::PropositionPredicate { definition }
            if definition.name.as_ref() == "NonEmpty" && definition.params.len() == 1
    ));
    assert!(
        api_interface
            .public_export_in_namespace(CanonicalNamespace::PromotedKind, "TreeKind")
            .is_some()
    );
    assert!(
        api_interface
            .public_export_in_namespace(CanonicalNamespace::Proposition, "NonEmpty")
            .is_some()
    );
    let root_interface = finalized
        .module(&root)
        .expect("finalization publishes the root module");
    assert_eq!(
        root_interface
            .public_export("exported_kind")
            .expect("public promoted-kind re-export is export-closed")
            .defining_identity(),
        data_kind.identity()
    );
    assert_eq!(
        root_interface
            .public_export("exported_predicate")
            .expect("public proposition re-export is export-closed")
            .defining_identity(),
        predicate.identity()
    );
}

#[test]
fn red_public_data_kind_missing_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub data kind MissingKind from type Missing;
            }
        "#,
        "public-data-kind-missing-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public data kind cannot publish an unresolved source ADT");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api
            && name.as_ref() == "MissingKind"
            && dependency.as_ref() == "Missing"
    ));
}

#[test]
fn red_public_predicate_private_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                type Hidden = Int;
                pub prop Leaks<X: Hidden>;
            }
            pub use crate::api::Leaks as exported;
        "#,
        "public-predicate-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public proposition cannot leak a private type domain");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "Leaks" && dependency.as_ref() == "Hidden"
    ));
}

#[test]
fn red_public_role_fact_preserves_namespace_metadata() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub role reviewer { capabilities: [], obligations: [audit_log] }
            }
            pub use crate::api::reviewer as exported;
        "#,
        "public-role-projection",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("public role metadata finalizes atomically");
    let api_interface = finalized
        .module(&api)
        .expect("finalization publishes the role child module");
    let role = api_interface
        .private_declaration("reviewer")
        .expect("role remains in the private checked view");
    assert_eq!(role.kind(), CanonicalDeclarationKind::Role);
    assert_eq!(role.namespace(), CanonicalNamespace::Role);
    assert!(matches!(
        role.fact(),
        CanonicalCheckedDeclarationFact::Role { definition }
            if definition.name.as_ref() == "reviewer"
                && definition.obligations.len() == 1
    ));
    assert!(
        api_interface
            .public_export_in_namespace(CanonicalNamespace::Role, "reviewer")
            .is_some()
    );
    let root_interface = finalized
        .module(&root)
        .expect("finalization publishes the root module");
    assert_eq!(
        root_interface
            .public_export("exported")
            .expect("public role re-export is export-closed")
            .defining_identity(),
        role.identity()
    );
}

#[test]
fn red_public_type_function_fact_preserves_namespace_metadata() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub sealed type domain Tree { Leaf; Branch<tail: Tree>; }
                pub type fn Identity(xs: Tree) -> Tree {
                    case Identity<xs> = xs;
                }
            }
            pub use crate::api::Identity as exported;
        "#,
        "public-type-function-projection",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("public type-function metadata finalizes atomically");
    let api_interface = finalized
        .module(&api)
        .expect("finalization publishes the type-function child module");
    let type_function = api_interface
        .private_declaration("Identity")
        .expect("type function remains in the private checked view");
    assert_eq!(type_function.kind(), CanonicalDeclarationKind::TypeFn);
    assert_eq!(
        type_function.namespace(),
        CanonicalNamespace::TypeComputation
    );
    assert!(matches!(
        type_function.fact(),
        CanonicalCheckedDeclarationFact::TypeFn { definition }
            if definition.name.as_ref() == "Identity"
                && definition.params.len() == 1
                && definition.equations.len() == 1
    ));
    assert!(
        api_interface
            .public_export_in_namespace(CanonicalNamespace::TypeComputation, "Identity")
            .is_some()
    );
    let root_interface = finalized
        .module(&root)
        .expect("finalization publishes the root module");
    assert_eq!(
        root_interface
            .public_export("exported")
            .expect("public type-function re-export is export-closed")
            .defining_identity(),
        type_function.identity()
    );
}

#[test]
fn red_public_type_function_private_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub sealed type domain Tree { Leaf; }
                type Hidden = Int;
                pub type fn Leak(xs: Tree) -> Hidden {
                    case Leak<xs> = Hidden;
                }
            }
            pub use crate::api::Leak as exported;
        "#,
        "public-type-function-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public type function cannot leak a private result type");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "Leak" && dependency.as_ref() == "Hidden"
    ));
}

#[test]
fn red_public_type_function_private_pattern_constructor_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub sealed type domain Tree { Leaf; }
                pub type fn Leak(xs: Tree) -> Tree {
                    case Leak<Leaf> = xs;
                }
            }
            pub use crate::api::Leak as exported;
        "#,
        "public-type-function-private-pattern-constructor",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public type function cannot leak a private pattern constructor");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "Leak" && dependency.as_ref() == "Leaf"
    ));
}

#[test]
fn red_public_type_function_private_proposition_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub sealed type domain Tree { Leaf; }
                prop Hidden<X: Tree>;
                pub type fn Leak(xs: Tree) -> Tree
                    where Hidden<Tree>
                {
                    case Leak<xs> = xs;
                }
            }
            pub use crate::api::Leak as exported;
        "#,
        "public-type-function-private-proposition-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public type function cannot leak a private proposition predicate");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "Leak" && dependency.as_ref() == "Hidden"
    ));
}

#[test]
fn red_public_type_function_public_proposition_tail_preserves_metadata() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub sealed type domain Tree { Leaf; }
                pub prop Visible<X: Tree>;
                pub type fn Stable(xs: Tree) -> Tree
                    where Visible<Tree>, Tree == Tree
                {
                    case Stable<xs> = xs;
                }
            }
            pub use crate::api::Stable as exported;
        "#,
        "public-type-function-public-proposition-tail",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("a public proposition tail with public dependencies finalizes");
    let declaration = finalized
        .module(&api)
        .expect("finalization publishes the type-function module")
        .private_declaration("Stable")
        .expect("type function remains in the private checked view");
    assert!(matches!(
        declaration.fact(),
        CanonicalCheckedDeclarationFact::TypeFn { definition }
            if definition.proposition_tail.as_ref().is_some_and(|tail| tail.clauses.len() == 2)
    ));
}

#[test]
fn red_public_notation_fact_preserves_namespace_metadata() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub fn combine(left: Int, right: Int) -> Int { left }
                pub infixl 6 <+> = combine;
            }
        "#,
        "public-notation-projection",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("public notation metadata finalizes atomically");
    let api_interface = finalized
        .module(&api)
        .expect("finalization publishes the notation child module");
    let notation = api_interface
        .private_declarations()
        .find(|declaration| declaration.kind() == CanonicalDeclarationKind::Notation)
        .expect("notation remains in the private checked view");
    assert_eq!(notation.kind(), CanonicalDeclarationKind::Notation);
    assert_eq!(notation.namespace(), CanonicalNamespace::Notation);
    assert!(matches!(
        notation.fact(),
        CanonicalCheckedDeclarationFact::Notation { definition }
            if definition.target.name.as_ref() == "combine"
    ));
    assert!(
        api_interface
            .public_export_in_namespace(
                CanonicalNamespace::Notation,
                "infix:left:precedence: 6:<+>"
            )
            .is_some()
    );
}

#[test]
fn red_public_notation_private_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                fn combine(left: Int, right: Int) -> Int { left }
                pub infixl 6 <+> = combine;
            }
        "#,
        "public-notation-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public notation cannot target a private callable");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api
            && name.as_ref() == "infix:left:precedence: 6:<+>"
            && dependency.as_ref() == "combine"
    ));
}

#[test]
fn red_public_qualified_notation_private_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod provider {
                pub(crate) fn combine(left: Int, right: Int) -> Int { left }
            }
            pub mod api {
                pub infixl 6 <+> = provider::combine;
            }
        "#,
        "public-qualified-notation-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public notation cannot expose a qualified private callable");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref dependency,
            ..
        } if module == &api && dependency.as_ref() == "combine"
    ));
}

#[test]
fn red_public_qualified_notation_public_dependency_preserves_closure() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod provider {
                pub fn combine(left: Int, right: Int) -> Int { left }
            }
            pub mod api {
                pub infixl 6 <+> = provider::combine;
            }
        "#,
        "public-qualified-notation-public-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("a public notation may target a public qualified callable");
    assert!(
        finalized
            .module(&api)
            .expect("api interface is finalized")
            .public_exports()
            .any(|export| export.declaration().namespace() == CanonicalNamespace::Notation)
    );
}

#[test]
fn red_public_notation_missing_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub infixl 6 <+> = missing;
            }
        "#,
        "public-notation-missing-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public notation cannot publish an unresolved local target");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api
            && name.as_ref() == "infix:left:precedence: 6:<+>"
            && dependency.as_ref() == "missing"
    ));
}

#[test]
fn red_public_macro_summary_fact_preserves_syntax_metadata() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub macro rewrite(x: Int) -> Int => x;
            }
            pub use crate::api::rewrite as exported;
        "#,
        "public-macro-summary-projection",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("public macro syntax metadata finalizes atomically");
    let api_interface = finalized
        .module(&api)
        .expect("finalization publishes the macro child module");
    let macro_declaration = api_interface
        .private_declaration("rewrite")
        .expect("macro remains in the private checked view");
    assert_eq!(macro_declaration.kind(), CanonicalDeclarationKind::Macro);
    assert_eq!(macro_declaration.namespace(), CanonicalNamespace::Macro);
    assert!(macro_declaration.signature().is_none());
    assert!(matches!(
        macro_declaration.fact(),
        CanonicalCheckedDeclarationFact::Macro { summary }
            if summary.name.as_ref() == "rewrite"
                && summary.params.len() == 1
                && summary.template_fingerprint.param_count == 1
                && summary.input_kind == ash_parser::surface::MacroInputKind::ExprArgs
                && summary.output_kind == ash_parser::surface::MacroOutputKind::Expr
                && summary.hygiene_policy
                    == ash_parser::surface::MacroHygienePolicy::BinderFreeExpression
                && summary.typed_signature.is_some()
    ));
    assert!(
        api_interface
            .public_export_in_namespace(CanonicalNamespace::Macro, "rewrite")
            .is_some()
    );
    let root_interface = finalized
        .module(&root)
        .expect("finalization publishes the root module");
    assert_eq!(
        root_interface
            .public_export("exported")
            .expect("public macro re-export is export-closed")
            .defining_identity(),
        macro_declaration.identity()
    );
}

#[test]
fn red_public_macro_typed_signature_private_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                type Hidden = Int;
                pub macro leak(x: Hidden) -> Hidden => x;
            }
            pub use crate::api::leak as exported;
        "#,
        "public-macro-summary-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public macro typed signature cannot leak a private type");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "leak" && dependency.as_ref() == "Hidden"
    ));
}

#[test]
fn red_public_macro_imported_callable_private_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod provider {
                pub(crate) fn hidden(value: Int) -> Int { value }
            }
            pub mod api {
                use crate::provider::hidden;
                pub macro leak(value: Int) -> Int => hidden(value);
            }
            pub use crate::api::leak as exported;
        "#,
        "public-macro-imported-callable-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public macro cannot expose an imported private callable");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "leak" && dependency.as_ref() == "hidden"
    ));
}

#[test]
fn red_public_evidence_facts_preserve_checked_namespace_metadata() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub law reflexive(value: Int): value == value
            }
            pub use crate::api::reflexive as law_exported;
        "#,
        "public-evidence-projection",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("public evidence metadata finalizes atomically");
    let api_interface = finalized
        .module(&api)
        .expect("finalization publishes the evidence child module");
    let law = api_interface
        .private_declaration("reflexive")
        .expect("law remains in the private checked view");
    assert_eq!(law.kind(), CanonicalDeclarationKind::Law);
    assert_eq!(law.namespace(), CanonicalNamespace::Evidence);
    assert!(matches!(
        law.fact(),
        CanonicalCheckedDeclarationFact::Law { definition }
            if definition.name.as_ref() == "reflexive"
                && definition.params.len() == 1
    ));
    assert!(
        api_interface
            .public_export_in_namespace(CanonicalNamespace::Evidence, "reflexive")
            .is_some()
    );
    let root_interface = finalized
        .module(&root)
        .expect("finalization publishes the root module");
    assert_eq!(
        root_interface
            .public_export("law_exported")
            .expect("public law re-export is export-closed")
            .defining_identity(),
        law.identity()
    );
}

#[test]
fn red_public_evidence_private_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                type Hidden = Int;
                pub law leaks(value: Hidden): value == value
            }
            pub use crate::api::leaks as exported;
        "#,
        "public-evidence-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public law cannot leak a private parameter type");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "leaks" && dependency.as_ref() == "Hidden"
    ));
}

#[test]
fn red_public_evidence_imported_callable_private_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod provider {
                pub(crate) fn hidden(value: Int) -> Bool { value == value }
            }
            pub mod api {
                use crate::provider::hidden;
                pub law leaks(value: Int): hidden(value)
            }
            pub use crate::api::leaks as exported;
        "#,
        "public-evidence-imported-callable-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public law cannot expose an imported private callable");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "leaks" && dependency.as_ref() == "hidden"
    ));
}

#[test]
fn red_public_policy_imported_callable_private_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod provider {
                pub(crate) fn hidden(value: Int) -> Int { value }
            }
            pub mod api {
                use crate::provider::hidden;
                pub policy Leaks { value: Int = hidden(1) }
            }
            pub use crate::api::Leaks as exported;
        "#,
        "public-policy-imported-callable-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public policy schema cannot depend on an imported private callable");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "Leaks" && dependency.as_ref() == "hidden"
    ));
}

#[test]
fn red_public_interface_law_preserves_parent_scoped_visibility() {
    let (root, expanded) = expanded_graph(
        r#"
            pub interface Eq<A> {
                equiv(A, A) -> Bool
                law reflexive(x: A): equiv(x, x)
            }
        "#,
        "public-interface-law-visibility",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);

    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("public interface evidence is retained in the checked interface");
    let interface = finalized
        .module(&root)
        .expect("finalization publishes the root interface");
    let declaration = interface
        .private_declaration("Eq")
        .expect("public interface remains in the private view");
    let CanonicalCheckedDeclarationFact::Interface {
        evidence,
        definition,
    } = declaration.fact()
    else {
        panic!("expected checked interface metadata")
    };
    assert_eq!(definition.name.as_ref(), "Eq");
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0].name(), "reflexive");
    assert_eq!(
        evidence[0].kind(),
        CanonicalDeclarationKind::Law,
        "interface laws retain their evidence kind"
    );
    assert_eq!(
        evidence[0].visibility(),
        &ash_parser::surface::Visibility::Inherited,
        "interface laws are parent-scoped rather than independently public"
    );
    assert!(
        interface
            .public_export_in_namespace(CanonicalNamespace::Evidence, "reflexive")
            .is_none(),
        "a public interface must not flatten its nested law into the module evidence namespace"
    );
}

#[test]
fn red_public_interface_law_private_callable_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            fn hidden(value: Int) -> Bool { value == value }

            pub interface Eq {
                equiv(Int, Int) -> Bool
                law leaks(x: Int): hidden(x)
            }
        "#,
        "public-interface-law-private-callable-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public interface law cannot expose a private callable dependency");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &root && name.as_ref() == "Eq" && dependency.as_ref() == "hidden"
    ));
}

#[test]
fn red_public_interface_law_public_callable_dependency_preserves_closure() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod provider {
                pub fn visible(value: Int) -> Bool { value == value }
            }

            pub mod api {
                use crate::provider::visible;
                pub interface Eq {
                    equiv(Int, Int) -> Bool
                    law sound(x: Int): visible(x)
                }
            }
        "#,
        "public-interface-law-public-callable-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("a public interface law may depend on an imported public callable");
    assert!(
        finalized
            .module(&api)
            .expect("finalization publishes the interface child module")
            .public_export("Eq")
            .is_some()
    );
}

#[test]
fn red_public_interface_law_imported_private_callable_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod provider {
                pub(crate) fn hidden(value: Int) -> Bool { value == value }
            }

            pub mod api {
                use crate::provider::hidden;
                pub interface Eq {
                    equiv(Int, Int) -> Bool
                    law leaks(x: Int): hidden(x)
                }
            }
        "#,
        "public-interface-law-imported-private-callable-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public interface law cannot expose an imported private callable");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "Eq" && dependency.as_ref() == "hidden"
    ));
}

#[test]
fn red_public_module_law_private_qualified_impl_call_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub interface Eq<A> {
                equiv(A, A) -> Bool
            }

            impl Eq<Int> {
                equiv(a, b) = a == b
            }

            pub law reflexive(value: Int): Eq::equiv(value, value)
        "#,
        "public-module-law-private-qualified-impl-call",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public module law cannot expose a private qualified implementation call");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &root && name.as_ref() == "reflexive" && dependency.as_ref() == "Eq"
    ));
}

#[test]
fn red_public_module_law_public_qualified_impl_call_preserves_parent_scoped_method() {
    let (root, expanded) = expanded_graph(
        r#"
            pub interface Eq<A> {
                equiv(A, A) -> Bool
            }

            pub impl Eq<Int> {
                equiv(a, b) = a == b
            }

            pub law reflexive(value: Int): Eq::equiv(value, value)
        "#,
        "public-module-law-public-qualified-impl-call",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);

    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("a public module law may use a public qualified implementation call");
    let interface = finalized
        .module(&root)
        .expect("finalization publishes the root module");
    assert!(
        interface
            .public_export_in_namespace(CanonicalNamespace::ImplementationRegistry, "Eq")
            .is_some(),
        "the matching public implementation is available in the implementation registry"
    );
    assert!(
        interface
            .public_export_in_namespace(CanonicalNamespace::ValueCallable, "equiv")
            .is_none(),
        "interface-owned methods remain parent-scoped rather than standalone callables"
    );
}

#[test]
fn red_public_impl_proof_preserves_parent_scoped_visibility() {
    let (root, expanded) = expanded_graph(
        r#"
            pub interface Eq<A> {
                equiv(A, A) -> Bool
                law reflexive(x: A): equiv(x, x)
            }

            pub impl Eq<Int> {
                equiv(a, b) = a == b
                proof reflexive(x: Int) { by_definition }
            }
        "#,
        "public-impl-proof-visibility",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);

    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("public implementation proofs remain checked parent-scoped metadata");
    let interface = finalized
        .module(&root)
        .expect("finalization publishes the root interface");
    let implementation = interface
        .private_declarations()
        .find(|declaration| {
            declaration.kind() == CanonicalDeclarationKind::Impl && declaration.name() == "Eq"
        })
        .expect("implementation remains in the private view");
    let CanonicalCheckedDeclarationFact::Implementation { summary } = implementation.fact() else {
        panic!("expected checked implementation metadata")
    };
    assert_eq!(summary.proofs().len(), 1);
    assert_eq!(summary.proofs()[0].name(), "reflexive");
    assert_eq!(summary.proofs()[0].kind(), CanonicalDeclarationKind::Proof);
    assert_eq!(
        summary.proofs()[0].visibility(),
        &ash_parser::surface::Visibility::Inherited
    );
    assert!(
        interface
            .public_export_in_namespace(CanonicalNamespace::Evidence, "reflexive")
            .is_none(),
        "an implementation proof remains parent-scoped even when the impl is public"
    );
}

#[test]
fn red_public_policy_named_binding_preserves_identity_and_schema() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub policy <T> RateLimit {
                    requests: Int,
                    payload: T = 0
                } where { requests > 0 }
            }
            pub use crate::api::RateLimit as exported;
        "#,
        "public-policy-projection",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("public policy schema metadata finalizes atomically");
    let api_interface = finalized
        .module(&api)
        .expect("finalization publishes the policy child module");
    let policy = api_interface
        .private_declaration("RateLimit")
        .expect("policy remains in the private checked view");
    assert_eq!(policy.kind(), CanonicalDeclarationKind::Policy);
    assert_eq!(policy.namespace(), CanonicalNamespace::Policy);
    assert!(matches!(
        policy.fact(),
        CanonicalCheckedDeclarationFact::Policy { definition }
            if definition.name.as_ref() == "RateLimit"
                && definition.type_params.len() == 1
                && definition.fields.len() == 2
                && definition.where_clause.is_some()
    ));
    assert!(
        api_interface
            .public_export_in_namespace(CanonicalNamespace::Policy, "RateLimit")
            .is_some()
    );
    let root_interface = finalized
        .module(&root)
        .expect("finalization publishes the root module");
    let exported = root_interface
        .public_export_in_namespace(CanonicalNamespace::Policy, "exported")
        .expect("the named policy binding is a policy-namespace export");
    assert_eq!(exported.local_name(), "exported");
    assert_eq!(exported.defining_identity(), policy.identity());
    assert_eq!(exported.declaration().name(), "RateLimit");
    assert_eq!(
        exported.declaration().kind(),
        CanonicalDeclarationKind::Policy
    );
    assert_eq!(
        exported.declaration().namespace(),
        CanonicalNamespace::Policy
    );
    assert!(matches!(
        exported.declaration().fact(),
        CanonicalCheckedDeclarationFact::Policy { definition }
            if definition.name.as_ref() == "RateLimit"
                && definition.type_params.len() == 1
                && definition.fields.len() == 2
                && definition.where_clause.is_some()
    ));
    assert!(exported.import_span().is_some());
}

#[test]
fn red_impl_proof_fact_preserves_interface_law_pair() {
    let (root, expanded) = expanded_graph(
        r#"
            pub interface Eq<A> {
                equiv(A, A) -> Bool
                law reflexive(x: A): equiv(x, x)
            }

            impl Eq<Int> {
                equiv(a, b) = a == b
                proof reflexive(x: Int) {
                    by_definition
                }
            }
        "#,
        "public-impl-proof-law-pair",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);

    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("interface law and implementation proof finalize as parent-scoped evidence");
    let interface = finalized
        .module(&root)
        .expect("finalization publishes the root module");
    let law = interface
        .private_declarations()
        .find(|declaration| {
            declaration.kind() == CanonicalDeclarationKind::Law
                && declaration.name() == "reflexive"
                && declaration
                    .identity()
                    .canonical_parent()
                    .is_some_and(|parent| parent.kind() == CanonicalDeclarationKind::Interface)
        })
        .expect("interface law remains parent-scoped evidence");
    assert!(matches!(
        law.fact(),
        CanonicalCheckedDeclarationFact::Law { definition }
            if definition.name.as_ref() == "reflexive"
    ));

    let proof = interface
        .private_declarations()
        .find(|declaration| {
            declaration.kind() == CanonicalDeclarationKind::Proof
                && declaration.name() == "reflexive"
                && declaration
                    .identity()
                    .canonical_parent()
                    .is_some_and(|parent| parent.kind() == CanonicalDeclarationKind::Impl)
        })
        .expect("implementation proof remains parent-scoped evidence");
    assert!(matches!(
        proof.fact(),
        CanonicalCheckedDeclarationFact::Proof { definition }
            if definition.name.as_ref() == "reflexive"
    ));
    assert!(
        interface
            .public_export_in_namespace(CanonicalNamespace::Evidence, "reflexive")
            .is_none(),
        "inherited impl proof visibility is not a standalone export"
    );
}

#[test]
fn red_public_policy_private_field_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                type Hidden = Int;
                pub policy Leaks { value: Hidden }
            }
            pub use crate::api::Leaks as exported;
        "#,
        "public-policy-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public policy schema cannot leak a private field type");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "Leaks" && dependency.as_ref() == "Hidden"
    ));
}

#[test]
fn red_public_policy_missing_field_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                pub policy Broken { value: Missing }
            }
        "#,
        "public-policy-missing-field-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public policy cannot publish an unresolved field type");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "Broken" && dependency.as_ref() == "Missing"
    ));
}

#[test]
fn red_public_policy_default_type_mismatch_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub policy InvalidDefault {
                limit: Int = true
            }
        "#,
        "public-policy-default-type-mismatch",
    );
    let (_root, expanded, collection, imports) = collected_inputs(root, expanded);

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a policy default must match its declared field type");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::Policy {
            ref module,
            ref name,
            ref reason,
            ..
        } if module == &ModuleKey::root("app").expect("fixture crate key is canonical")
            && name.as_ref() == "InvalidDefault"
            && reason.contains("default")
    ));
}

#[test]
fn red_public_policy_invariant_must_be_boolean_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub policy InvalidInvariant {
                limit: Int
            } where { limit }
        "#,
        "public-policy-invariant-not-bool",
    );
    let (_root, expanded, collection, imports) = collected_inputs(root, expanded);

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a policy invariant must have Bool type");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::Policy {
            ref module,
            ref name,
            ref reason,
            ..
        } if module == &ModuleKey::root("app").expect("fixture crate key is canonical")
            && name.as_ref() == "InvalidInvariant"
            && reason.contains("Bool")
    ));
}

#[test]
fn red_final_pub_use_requires_export_closed_targets() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api {
                type Hidden = Int;
                pub fn expose(value: Hidden) -> Hidden { value }
            }
            pub use crate::api::expose as exported;
        "#,
        "export-closed-private-signature",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");
    assert!(collection.internal_snapshot(&api).is_some());
    let staged = imports
        .public_uses()
        .iter()
        .find(|public_use| public_use.binding().local_name() == "exported")
        .expect("the parsed resolver stages the candidate re-export");
    assert_eq!(staged.binding().defining_identity().module_key(), &api);

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public signature cannot leak the private Hidden declaration");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "expose" && dependency.as_ref() == "Hidden"
    ));
}

#[test]
fn red_imported_binding_visibility_drift_rejects_atomically() {
    let tree = TempTree::new("imported-binding-visibility-drift");
    let root_path = tree.write("src/main.ash", "pub mod api; use crate::api::hidden;");
    let api_path = tree.write(
        "src/api.ash",
        "pub(crate) fn hidden(value: Int) -> Int { value }",
    );
    let root = ModuleKey::root("app").expect("fixture crate key is canonical");
    let resolver = CanonicalModuleGraphResolver::new();
    let original = resolver
        .resolve_root(root.clone(), &root_path)
        .expect("original visibility graph resolves");
    let original_expanded = CanonicalExpandedModuleGraph::try_expand(original)
        .expect("original visibility graph expands");
    let original_collection = collect_canonical_expanded_module_graph(&original_expanded)
        .expect("original visibility collection succeeds");
    let api = root.child("api").expect("fixture child key is canonical");
    let original_entry = original_collection
        .provisional_name_view(&api)
        .expect("original api provisional view exists")
        .entries()
        .find(|entry| entry.lookup_name() == "hidden")
        .expect("original hidden declaration is collected");
    assert!(matches!(
        original_entry.visibility(),
        ash_parser::surface::Visibility::Crate
    ));

    fs::write(&api_path, "pub fn hidden(value: Int) -> Int { value }")
        .expect("overwrite the same provider file with mutated visibility");
    let mutated = resolver
        .resolve_root(root.clone(), &root_path)
        .expect("mutated visibility graph resolves from the same paths");
    let mutated_expanded = CanonicalExpandedModuleGraph::try_expand(mutated)
        .expect("mutated visibility graph expands");
    let mutated_collection = collect_canonical_expanded_module_graph(&mutated_expanded)
        .expect("mutated visibility collection succeeds");
    let mutated_imports = resolve_parsed_imports_from_collection(
        mutated_expanded.parsed_graph(),
        &mutated_collection,
    )
    .expect("mutated graph and collection resolve the imported binding");
    let mutated_binding = mutated_imports
        .binding(&root, "hidden")
        .expect("mutated imports retain the root binding");
    assert_eq!(
        mutated_binding.defining_identity(),
        original_entry.identity()
    );
    assert!(matches!(
        mutated_binding.declaration_visibility(),
        ash_parser::surface::Visibility::Public
    ));

    let error = finalize_canonical_module_collection(
        &original_expanded,
        &original_collection,
        &mutated_imports,
    )
    .expect_err(
        "a same-identity imported binding with drifted visibility must not publish interfaces",
    );
    assert!(
        format!("{error:?}").contains("BindingVisibilityMismatch"),
        "expected a BindingVisibilityMismatch-style error, got {error:?}"
    );
}

#[test]
fn red_stale_forged_and_incomplete_inputs_reject_atomically() {
    let (root, expanded) = expanded_graph(
        "pub mod api { pub fn stable(value: Int) -> Int { value } } use crate::api::stable;",
        "stale-input-original",
    );
    let (_root, expanded, collection, imports) = collected_inputs(root, expanded);
    assert!(
        imports
            .binding(
                &ModuleKey::root("app").expect("fixture crate key is canonical"),
                "stable"
            )
            .is_some()
    );

    let (_changed_root, changed) = expanded_graph(
        "pub mod api { pub fn changed(value: Int) -> Int { value } } use crate::api::changed;",
        "stale-input-mutated",
    );
    assert!(
        collection.revalidate_against(&changed).is_err(),
        "the collection already detects source/module-key drift before finalization"
    );
    let stale_error = finalize_canonical_module_collection(&changed, &collection, &imports)
        .expect_err("finalization must revalidate the collection before publication");
    assert!(matches!(
        stale_error,
        CanonicalCheckedModuleFinalizationError::GraphMismatch
    ));

    let (_changed_root, _changed_expanded, _changed_collection, changed_imports) = collected_inputs(
        ModuleKey::root("app").expect("fixture crate key is canonical"),
        changed,
    );
    let error = finalize_canonical_module_collection(&expanded, &collection, &changed_imports)
        .expect_err("stale binding facts cannot publish a final interface");
    assert!(matches!(
        error,
        ash_typeck::canonical_checked_module_finalizer::CanonicalCheckedModuleFinalizationError::MissingBindingTarget { .. }
            | ash_typeck::canonical_checked_module_finalizer::CanonicalCheckedModuleFinalizationError::GraphMismatch
    ));
}

#[test]
fn red_file_and_inline_final_interfaces_have_equal_projection() {
    let (inline_root, inline_expanded) = expanded_graph(
        "pub mod api { pub fn value(input: Int) -> Int { input + 1 } } use crate::api::value as imported;",
        "inline-final-interface-parity",
    );
    let (inline_root, inline_expanded, inline_collection, inline_imports) =
        collected_inputs(inline_root, inline_expanded);

    let (file_root, file_expanded) = file_graph(
        "pub mod api; use crate::api::value as imported;",
        "pub fn value(input: Int) -> Int { input + 1 }",
        "file-final-interface-parity",
    );
    let (file_root, file_expanded, file_collection, file_imports) =
        collected_inputs(file_root, file_expanded);

    assert_eq!(inline_root, file_root);
    assert_eq!(
        normalized_collection_projection(&inline_collection),
        normalized_collection_projection(&file_collection),
        "equivalent file/inline declarations reach the same normalized collection input"
    );
    assert_eq!(
        inline_imports.normalized_projection(),
        file_imports.normalized_projection(),
        "equivalent file/inline imports reach the same normalized binding input"
    );

    let inline_final =
        finalize_canonical_module_collection(&inline_expanded, &inline_collection, &inline_imports)
            .expect("inline module finalization succeeds");
    let file_final =
        finalize_canonical_module_collection(&file_expanded, &file_collection, &file_imports)
            .expect("file-backed module finalization succeeds");
    assert_eq!(
        normalized_final_interface_projection(&inline_final),
        normalized_final_interface_projection(&file_final),
        "equivalent file/inline modules publish the same normalized checked interfaces"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn red_generated_finalization_closure_is_atomic(
        public in any::<bool>(),
        function_count in 1usize..=3,
    ) {
        let mut definitions = String::new();
        for index in 0..function_count {
            let visibility = if public { "pub " } else { "" };
            definitions.push_str(&format!(
                "{visibility}fn generated_{index}(value: Int) -> Int {{ value + {index} }}"
            ));
        }
        let source = format!("pub mod api {{ {definitions} }} use crate::api::*;");
        let (root, expanded) = expanded_graph(&source, "generated-finalization-closure");
        let (root, expanded, collection, imports) = collected_inputs(root, expanded);
        let api = root.child("api").expect("fixture child key is canonical");
        prop_assert!(collection.internal_snapshot(&api).is_some());
        if public {
            prop_assert!(!imports.import_edges().is_empty());
        }
        let finalization = finalize_canonical_module_collection(&expanded, &collection, &imports);
        prop_assert!(
            finalization.is_ok(),
            "generated finalization rejects a valid closure: {:?}",
            finalization.err()
        );
        let finalization = finalization.expect("successful generated finalization");
        let api_interface = finalization.module(&api).expect("generated child interface");
        let expected_public = if public { function_count } else { 0 };
        prop_assert_eq!(api_interface.public_exports().count(), expected_public);
        prop_assert_eq!(
            finalization
                .module(&root)
                .expect("generated root interface")
                .public_exports()
                .count(),
            1
        );
    }
}

#[test]
fn red_finalizer_authority_fence_excludes_source_and_provisional_view() {
    let source_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../ash-typeck/src/canonical_checked_module_finalizer.rs");
    let source = fs::read_to_string(&source_path).expect("the finalizer module exists");
    for forbidden in [
        "provisional_name_view",
        "read_to_string",
        "CanonicalModuleGraphResolver",
        "parse_surface",
    ] {
        assert!(
            !source.contains(forbidden),
            "finalization must consume collected/bound facts, not recover authority through {forbidden}"
        );
    }
    // `raw_definition` is intentionally not forbidden: it is retained by the
    // checker-internal snapshot and is part of TASK-2073's authoritative input.
    assert!(source.contains("internal_snapshot"));
    assert!(source.contains("CanonicalParsedImportResult"));
}

#[test]
fn red_public_imported_row_private_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod provider {
                pub(crate) effect alias Hidden = { evidence audit_log };
            }
            pub mod api {
                use crate::provider::Hidden;
                pub effect group Published = { Hidden };
            }
        "#,
        "public-imported-row-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public row cannot expose a crate-private imported row alias");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "Published" && dependency.as_ref() == "Hidden"
    ));
}

#[test]
fn red_public_imported_role_row_private_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod provider {
                pub(crate) role HiddenRole { capabilities: [], obligations: [audit_log] }
            }
            pub mod api {
                use crate::provider::HiddenRole;
                pub effect alias Published = { role HiddenRole };
            }
        "#,
        "public-imported-role-row-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public row cannot expose a crate-private imported role");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "Published" && dependency.as_ref() == "HiddenRole"
    ));
}

#[test]
fn red_public_imported_role_row_public_dependency_preserves_closure() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod provider {
                pub role VisibleRole { capabilities: [], obligations: [audit_log] }
            }
            pub mod api {
                use crate::provider::VisibleRole;
                pub effect alias Published = { role VisibleRole };
            }
        "#,
        "public-imported-role-row-public-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("a public row may expose a public imported role");
    assert!(
        finalized
            .module(&api)
            .expect("api interface is finalized")
            .public_export("Published")
            .is_some()
    );
}

#[test]
fn red_public_imported_policy_row_private_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod provider {
                pub(crate) policy HiddenPolicy { marker: Int }
            }
            pub mod api {
                use crate::provider::HiddenPolicy;
                pub effect alias Published = { policy HiddenPolicy };
            }
            pub use crate::api::Published as exported;
        "#,
        "public-imported-policy-row-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public effect alias cannot leak an imported private policy row");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "Published" && dependency.as_ref() == "HiddenPolicy"
    ));
}

#[test]
fn red_public_imported_policy_row_public_dependency_preserves_closure() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod provider {
                pub policy VisiblePolicy { marker: Int }
            }
            pub mod api {
                use crate::provider::VisiblePolicy;
                pub effect alias Published = { policy VisiblePolicy };
            }
            pub use crate::api::Published as exported;
        "#,
        "public-imported-policy-row-public-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("a public effect alias may expose a public imported policy");
    assert!(
        finalized
            .module(&api)
            .expect("api interface is finalized")
            .public_export("Published")
            .is_some()
    );
}

#[test]
fn red_public_imported_data_kind_private_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod provider {
                pub(crate) type Tree = Int;
            }
            pub mod api {
                use crate::provider::Tree;
                pub data kind TreeKind from type Tree;
            }
        "#,
        "public-imported-data-kind-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public data kind cannot expose a crate-private imported source type");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref name,
            ref dependency,
            ..
        } if module == &api && name.as_ref() == "TreeKind" && dependency.as_ref() == "Tree"
    ));
}

#[test]
fn red_public_imported_notation_private_dependency_rejects_atomically() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod provider {
                pub(crate) fn combine(left: Int, right: Int) -> Int { left }
            }
            pub mod api {
                use crate::provider::combine;
                pub infixl 6 <+> = combine;
            }
        "#,
        "public-imported-notation-private-dependency",
    );
    let (root, expanded, collection, imports) = collected_inputs(root, expanded);
    let api = root.child("api").expect("fixture child key is canonical");

    let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect_err("a public notation cannot expose a crate-private imported callable");
    assert!(matches!(
        error,
        CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            ref module,
            ref dependency,
            ..
        } if module == &api && dependency.as_ref() == "combine"
    ));
}
