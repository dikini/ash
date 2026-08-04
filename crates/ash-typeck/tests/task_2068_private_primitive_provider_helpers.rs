//! TASK-2068 RED contracts for private primitive provider helpers.
//!
//! The opt-in direct public re-export route may check inherited helpers inside
//! a selected provider, but its public fragment must retain only the direct
//! child identity and the explicitly selected public re-export target.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::ModuleKey;
use ash_parser::surface::{Definition, FnDef, Visibility};
use ash_parser::{CanonicalModuleGraph, CanonicalModuleGraphResolver};
use ash_typeck::{
    CanonicalDirectPrimitiveInterfaceImportError, CanonicalPrimitiveInterfaceError, Type,
    check_direct_primitive_interface_fragments, resolve_direct_primitive_interface_imports,
};
use proptest::prelude::*;

static NEXT_TEMP_TREE: AtomicUsize = AtomicUsize::new(0);

/// A real parser fixture whose drop implementation removes its source tree.
struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(label: &str) -> Self {
        let serial = NEXT_TEMP_TREE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "ash-task-2068-private-provider-helpers-{label}-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create temporary parser fixture tree");
        Self { root }
    }

    fn write(&self, relative: impl AsRef<Path>, source: &str) -> PathBuf {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("fixture path has a parent"))
            .expect("create fixture parent directory");
        fs::write(&path, source).expect("write parser fixture source");
        path
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn inline_provider_graph(
    provider_body: &str,
    root_body: &str,
    label: &str,
) -> (ModuleKey, CanonicalModuleGraph) {
    let tree = TempTree::new(label);
    let root_path = tree.write(
        "src/main.ash",
        &format!("pub mod api {{ {provider_body} }} {root_body}"),
    );
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("inline provider fixture must resolve through the canonical parser graph");
    (root_key, graph)
}

fn file_provider_graph(
    root_source: &str,
    provider_source: &str,
    label: &str,
) -> (ModuleKey, CanonicalModuleGraph) {
    let tree = TempTree::new(label);
    let root_path = tree.write("src/main.ash", root_source);
    tree.write("src/api.ash", provider_source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("file-backed provider fixture must resolve through the canonical parser graph");
    (root_key, graph)
}

fn function<'a>(graph: &'a CanonicalModuleGraph, module: &ModuleKey, name: &str) -> &'a FnDef {
    graph
        .module_unit(module)
        .expect("fixture module has an acquired canonical unit")
        .body()
        .definitions()
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == name => Some(function),
            _ => None,
        })
        .expect("fixture function remains in its parser-owned module unit")
}

fn first_use_span(graph: &CanonicalModuleGraph, module: &ModuleKey) -> ash_parser::Span {
    graph
        .module_unit(module)
        .expect("fixture module has an acquired canonical unit")
        .body()
        .uses()
        .first()
        .expect("fixture module contains a parsed public use declaration")
        .span
}

fn int_to_int() -> Type {
    Type::Fn(vec![Type::Int], Box::new(Type::Int))
}

#[test]
fn inline_private_helper_is_checked_but_projects_only_the_public_target_and_alias() {
    let (root_key, graph) = inline_provider_graph(
        r#"
            fn normalize(value: Int) -> Int { value + 1 }
            pub fn greet(value: Int) -> Int { normalize(value) }
        "#,
        "pub use crate::api::greet as welcome;",
        "inline-private-helper-positive",
    );
    let api_key = root_key
        .child("api")
        .expect("fixture child key is canonical");
    let api_origin = graph
        .module_unit(&api_key)
        .expect("provider unit is graph-owned")
        .artifact()
        .origin()
        .clone();
    let greet = function(&graph, &api_key, "greet");
    let use_span = first_use_span(&graph, &root_key);
    let plan = resolve_direct_primitive_interface_imports(&graph)
        .expect("the exact public re-export still selects only the public target");

    let fragments = check_direct_primitive_interface_fragments(&graph, &plan)
        .expect("an inherited primitive helper is a checked provider implementation detail");
    let child = fragments
        .public_child("api")
        .expect("the public structural child is retained");
    let reexport = fragments
        .reexport("welcome")
        .expect("the explicit public target alias is retained");

    assert_eq!(child.module_key(), &api_key);
    assert_eq!(child.visibility(), &Visibility::Public);
    assert_eq!(reexport.defining_identity().module_key(), &api_key);
    assert_eq!(reexport.defining_identity().name(), "greet");
    assert_eq!(reexport.declaration_span(), greet.span);
    assert_eq!(reexport.origin(), &api_origin);
    assert_eq!(reexport.signature(), &int_to_int());
    assert_eq!(reexport.use_span(), use_span);
    assert_eq!(reexport.visibility(), &Visibility::Public);
    assert!(
        fragments.reexport("normalize").is_none(),
        "the private helper must never become a root-visible public re-export"
    );
    assert!(
        fragments.reexport("greet").is_none(),
        "the provider child must not implicitly flatten its public target into the root"
    );
}

#[test]
fn file_and_inline_private_helper_providers_have_equal_normalized_public_fragments() {
    let provider_source = r#"
        fn normalize(value: Int) -> Int { value + 1 }
        pub fn greet(value: Int) -> Int { normalize(value) }
    "#;
    let (inline_root, inline_graph) = inline_provider_graph(
        provider_source,
        "pub use crate::api::greet as welcome;",
        "inline-private-helper-parity",
    );
    let (file_root, file_graph) = file_provider_graph(
        "pub mod api; pub use crate::api::greet as welcome;",
        provider_source,
        "file-private-helper-parity",
    );
    let inline_plan = resolve_direct_primitive_interface_imports(&inline_graph)
        .expect("the inline helper provider is structurally planned");
    let file_plan = resolve_direct_primitive_interface_imports(&file_graph)
        .expect("the file helper provider is structurally planned");
    let inline_fragments = check_direct_primitive_interface_fragments(&inline_graph, &inline_plan)
        .expect("the inline helper provider is checked before public fragment publication");
    let file_fragments = check_direct_primitive_interface_fragments(&file_graph, &file_plan)
        .expect("the file helper provider is checked before public fragment publication");
    let inline_child = inline_fragments
        .public_child("api")
        .expect("the inline fragment retains the public child");
    let file_child = file_fragments
        .public_child("api")
        .expect("the file fragment retains the public child");
    let inline_reexport = inline_fragments
        .reexport("welcome")
        .expect("the inline fragment retains the public alias");
    let file_reexport = file_fragments
        .reexport("welcome")
        .expect("the file fragment retains the public alias");
    let inline_api = inline_root
        .child("api")
        .expect("the inline fixture child key is canonical");
    let file_api = file_root
        .child("api")
        .expect("the file fixture child key is canonical");

    assert_eq!(inline_child.module_key(), &inline_api);
    assert_eq!(file_child.module_key(), &file_api);
    assert_eq!(inline_child.visibility(), file_child.visibility());
    assert_eq!(inline_reexport.visible_name(), file_reexport.visible_name());
    assert_eq!(
        inline_reexport.defining_identity().module_key(),
        file_reexport.defining_identity().module_key()
    );
    assert_eq!(
        inline_reexport.defining_identity().name(),
        file_reexport.defining_identity().name()
    );
    assert_eq!(inline_reexport.signature(), file_reexport.signature());
    assert_eq!(inline_reexport.visibility(), file_reexport.visibility());
    assert!(inline_fragments.reexport("normalize").is_none());
    assert!(file_fragments.reexport("normalize").is_none());
}

#[test]
fn reexporting_a_private_helper_rejects_at_the_helper_declaration() {
    let (root_key, graph) = inline_provider_graph(
        r#"
            fn helper(value: Int) -> Int { value }
            pub fn greet(value: Int) -> Int { helper(value) }
        "#,
        "pub use crate::api::helper as exposed;",
        "private-helper-target",
    );
    let api_key = root_key
        .child("api")
        .expect("fixture child key is canonical");
    let helper = function(&graph, &api_key, "helper");

    let error = resolve_direct_primitive_interface_imports(&graph)
        .expect_err("a private helper cannot form the selected public re-export target");

    match error {
        CanonicalDirectPrimitiveInterfaceImportError::PrivateTarget {
            defining_module,
            function,
            declaration_span,
            ..
        } => {
            assert_eq!(defining_module, api_key);
            assert_eq!(function.as_ref(), "helper");
            assert_eq!(declaration_span, helper.span);
        }
        other => panic!("expected anchored private helper target rejection, got {other:?}"),
    }
}

#[test]
fn nonprimitive_private_helper_signature_rejects_at_the_helper_declaration() {
    let (root_key, graph) = inline_provider_graph(
        r#"
            fn helper(value: Option<Int>) -> Option<Int> { value }
            pub fn greet(value: Int) -> Int { value + 1 }
        "#,
        "pub use crate::api::greet as welcome;",
        "nonprimitive-private-helper",
    );
    let api_key = root_key
        .child("api")
        .expect("fixture child key is canonical");
    let helper = function(&graph, &api_key, "helper");
    let plan = resolve_direct_primitive_interface_imports(&graph)
        .expect("the direct public plan selects only the public primitive target");

    let error = check_direct_primitive_interface_fragments(&graph, &plan)
        .expect_err("a nonprimitive private helper is outside the checked provider subset");

    match error {
        CanonicalPrimitiveInterfaceError::NonPrimitiveTarget {
            defining_module,
            function,
            declaration_span,
        } => {
            assert_eq!(defining_module, api_key);
            assert_eq!(function.as_ref(), "helper");
            assert_eq!(declaration_span, helper.span);
        }
        other => panic!("expected anchored private helper signature rejection, got {other:?}"),
    }
}

#[test]
fn late_invalid_private_helper_rejects_the_whole_fragment_before_publication() {
    let (root_key, graph) = inline_provider_graph(
        r#"
            pub fn greet(value: Int) -> Int { value + 1 }
            fn late_helper(value: Int) -> Int { true }
        "#,
        "pub use crate::api::greet as welcome;",
        "late-invalid-private-helper",
    );
    let api_key = root_key
        .child("api")
        .expect("fixture child key is canonical");
    let late_helper = function(&graph, &api_key, "late_helper");
    let plan = resolve_direct_primitive_interface_imports(&graph)
        .expect("the valid selected public target is planned before provider body checking");

    let error = check_direct_primitive_interface_fragments(&graph, &plan).expect_err(
        "a later invalid private helper must reject atomically instead of publishing welcome",
    );

    match error {
        CanonicalPrimitiveInterfaceError::ProviderCheck {
            defining_module,
            function,
            declaration_span,
            ..
        } => {
            assert_eq!(defining_module, api_key);
            assert_eq!(function.as_ref(), "late_helper");
            assert_eq!(declaration_span, late_helper.span);
        }
        other => panic!("expected atomic private helper body-check rejection, got {other:?}"),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn generated_private_helpers_preserve_only_public_target_identity_signature_provenance_and_use_span(
        suffix in "[a-z0-9_]{0,12}",
    ) {
        let helper = format!("helper_{suffix}");
        let alias = format!("alias_{suffix}");
        let provider = format!(
            "fn {helper}(value: Int) -> Int {{ value + 1 }} pub fn greet(value: Int) -> Int {{ {helper}(value) }}"
        );
        let root = format!("pub use crate::api::greet as {alias};");
        let (root_key, graph) = inline_provider_graph(
            &provider,
            &root,
            "generated-private-helper",
        );
        let api_key = root_key.child("api").expect("fixture child key is canonical");
        let api_origin = graph
            .module_unit(&api_key)
            .expect("provider unit is graph-owned")
            .artifact()
            .origin()
            .clone();
        let greet = function(&graph, &api_key, "greet");
        let use_span = first_use_span(&graph, &root_key);
        let plan = resolve_direct_primitive_interface_imports(&graph)
            .expect("each generated private helper retains a public selected target");
        let fragments = check_direct_primitive_interface_fragments(&graph, &plan)
            .expect("each generated private helper is checked without public projection");
        let reexport = fragments
            .reexport(&alias)
            .expect("the explicit generated public alias is retained");

        prop_assert_eq!(reexport.defining_identity().module_key(), &api_key);
        prop_assert_eq!(reexport.defining_identity().name(), "greet");
        prop_assert_eq!(reexport.declaration_span(), greet.span);
        prop_assert_eq!(reexport.origin(), &api_origin);
        prop_assert_eq!(reexport.signature(), &int_to_int());
        prop_assert_eq!(reexport.use_span(), use_span);
        prop_assert_eq!(reexport.visibility(), &Visibility::Public);
        prop_assert!(fragments.reexport(&helper).is_none());
        prop_assert!(fragments.reexport("greet").is_none());
    }
}

fn contains_exact_identifier(source: &str, identifier: &str) -> bool {
    source.match_indices(identifier).any(|(start, _)| {
        let before_is_identifier = source[..start]
            .chars()
            .next_back()
            .is_some_and(|character| character == '_' || character.is_ascii_alphanumeric());
        let after_is_identifier = source[start + identifier.len()..]
            .chars()
            .next()
            .is_some_and(|character| character == '_' || character.is_ascii_alphanumeric());
        !before_is_identifier && !after_is_identifier
    })
}

#[test]
fn private_helper_fragment_checker_has_no_compatibility_or_runtime_authority() {
    let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/canonical_primitive_interface_fragments.rs");
    let source = fs::read_to_string(&source_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 requires the dedicated private-helper fragment checker at {}: {error}",
            source_path.display()
        )
    });

    for forbidden_identifier in [
        "ModuleIdentity",
        "ModuleId",
        "ModuleGraph",
        "LegacyModuleResolver",
        "ModuleResolver",
        "NameBinder",
        "bind_simple_parsed_uses",
        "CanonicalBoundModuleSet",
        "PublicModuleInterface",
        "FinalizedModuleInterface",
        "InterfaceImportResolver",
        "CheckedInterfaceStore",
        "TypeEnvModuleInterfaceCollection",
        "RawCoreProgram",
        "CoreExpr",
        "CpsProgram",
        "Engine",
        "Default",
        "eval",
        "evaluate",
        "execute",
    ] {
        assert!(
            !contains_exact_identifier(&source, forbidden_identifier),
            "the private-helper fragment checker must not depend on compatibility, final-interface, evaluator, lowering, or runtime authority: {forbidden_identifier}"
        );
    }
    for forbidden_bypass in [
        "module_interface_finalization",
        "interface_import_resolver",
        "canonical_module_binder",
        "module_core_cps_lowering",
        "std::fs",
        "read_to_string",
        "parse_surface",
        "resolve_root",
        "from_legacy",
        "into_legacy",
    ] {
        assert!(
            !source.contains(forbidden_bypass),
            "the private-helper fragment checker may consume only canonical graph, exact public aliases, primitive checked facts, and ordinary Type/provenance data: {forbidden_bypass}"
        );
    }
    assert!(
        !source.contains("pub fn new("),
        "the immutable fragment result must not expose a public constructor"
    );
}
