//! TASK-2068 RED contract for scoped glob ordinary-function imports.

use std::fs;
use std::path::{Path, PathBuf};

use ash_core::module_graph::{ModuleArtifactOrigin, ModuleKey};
use ash_parser::CanonicalModuleGraphResolver;
use ash_parser::surface::{Definition, FnDef, Visibility};
use ash_typeck::{
    CanonicalProvisionalModuleScopes, CanonicalStructuralImportError,
    bind_scoped_glob_ordinary_function_imports,
    resolve_scoped_glob_ordinary_function_imports_with_scopes,
};
use proptest::prelude::*;
use sha2::{Digest, Sha256};

#[test]
fn scoped_glob_imports_two_public_ordinary_functions() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2068-scoped-glob-imports-{}",
        std::process::id()
    ));
    fs::create_dir_all(&fixture_root).expect("create temporary parser fixture directory");
    let root_path = fixture_root.join("main.ash");
    fs::write(
        &root_path,
        r#"
            pub mod api {
                pub fn encode(value: Int) -> Int { value }
                pub fn decode(value: Int) -> Int { value }
            }
            pub mod client {
                use crate::api::*;
            }
        "#,
    )
    .expect("write parser fixture source");

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("fixture source resolves through the canonical parser graph");
    let client = root_key
        .child("client")
        .expect("fixture client module key remains canonical");
    let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
        .expect("fixture graph derives immutable provisional scopes");

    let plan = resolve_scoped_glob_ordinary_function_imports_with_scopes(&graph, &scopes)
        .expect("the scoped glob resolver imports both public functions");
    let bound = bind_scoped_glob_ordinary_function_imports(&graph, &scopes)
        .expect("the scoped glob binder projects the resolver plan");

    for name in ["encode", "decode"] {
        assert!(plan.binding(&client, name).is_some());
        assert_eq!(bound.binding(&client, name), plan.binding(&client, name));
    }

    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn scoped_glob_import_plan_and_binder_preserve_each_function_identity_and_full_use_provenance() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2068-scoped-glob-identity-{}",
        std::process::id()
    ));
    fs::create_dir_all(&fixture_root).expect("create temporary parser fixture directory");
    let root_path = fixture_root.join("main.ash");
    fs::write(
        &root_path,
        r#"
            pub mod api {
                pub fn encode(value: Int) -> Int { value }
                pub fn decode(value: Int) -> Int { value }
            }
            pub mod client {
                use crate::api::*;
            }
        "#,
    )
    .expect("write parser fixture source");

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("fixture source resolves through the canonical parser graph");
    let api = root_key
        .child("api")
        .expect("fixture api module key remains canonical");
    let client = root_key
        .child("client")
        .expect("fixture client module key remains canonical");
    let api_unit = graph
        .module_unit(&api)
        .expect("fixture api unit remains graph-owned");
    let client_unit = graph
        .module_unit(&client)
        .expect("fixture importer unit remains graph-owned");
    assert!(
        client_unit.body().definitions().is_empty(),
        "the scoped-glob importer has zero local functions"
    );
    assert_eq!(
        client_unit.body().uses().len(),
        1,
        "the scoped-glob importer has exactly one use declaration"
    );
    let use_span = client_unit.body().uses()[0].span;
    let target_origin = api_unit.artifact().origin().clone();
    let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
        .expect("fixture graph derives immutable provisional scopes");
    let plan = resolve_scoped_glob_ordinary_function_imports_with_scopes(&graph, &scopes)
        .expect("the scoped glob resolver retains every public api function");
    let bound = bind_scoped_glob_ordinary_function_imports(&graph, &scopes)
        .expect("the scoped glob binder projects the complete resolver plan");

    assert_eq!(
        plan.import_edges().len(),
        2,
        "the glob creates one edge per selected ordinary function"
    );
    for name in ["encode", "decode"] {
        let target = api_unit
            .body()
            .definitions()
            .iter()
            .find_map(|definition| match definition {
                Definition::Function(function) if function.name.as_ref() == name => Some(function),
                _ => None,
            })
            .expect("fixture retains the requested public ordinary function");
        let plan_binding = plan
            .binding(&client, name)
            .expect("the resolver stages every natural function spelling");
        let bound_binding = bound
            .binding(&client, name)
            .expect("the binder projects every natural function spelling");
        let matching_edges: Vec<_> = plan
            .import_edges()
            .iter()
            .filter(|edge| edge.local_name() == name)
            .collect();

        assert_eq!(bound_binding, plan_binding);
        assert_eq!(plan_binding.defining_identity().module_key(), &api);
        assert_eq!(plan_binding.defining_identity().name(), name);
        assert_eq!(plan_binding.declaration_span(), target.span);
        assert_eq!(plan_binding.origin(), &target_origin);
        assert_eq!(plan_binding.visibility(), &Visibility::Public);
        assert_eq!(
            matching_edges.len(),
            1,
            "the glob retains exactly one edge for {name}"
        );
        let edge = matching_edges[0];
        assert_eq!(edge.importing_module(), &client);
        assert_eq!(edge.defining_module(), &api);
        assert_eq!(edge.defining_identity(), plan_binding.defining_identity());
        assert_eq!(edge.local_name(), name);
        assert_eq!(edge.use_span(), use_span);
        assert_eq!(edge.declaration_span(), target.span);
        assert_eq!(edge.origin(), &target_origin);
        assert_eq!(edge.visibility(), &Visibility::Public);
    }

    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn scoped_glob_imports_report_private_structural_and_function_visibility_before_any_binding() {
    let cases = [
        (
            "private-structural-module",
            r#"
                mod api { pub fn public_target() -> Int { 1 } }
                pub mod client { use crate::api::*; }
            "#,
            false,
        ),
        (
            "private-target-function",
            r#"
                pub mod api { fn private_target() -> Int { 1 } }
                pub mod client { use crate::api::*; }
            "#,
            true,
        ),
    ];

    for (label, source, private_function) in cases {
        let fixture_root = std::env::temp_dir().join(format!(
            "ash-task-2068-scoped-glob-visibility-{label}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&fixture_root).expect("create temporary parser fixture directory");
        let root_path = fixture_root.join("main.ash");
        fs::write(&root_path, source).expect("write parser fixture source");

        let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
        let graph = CanonicalModuleGraphResolver::new()
            .resolve_root(root_key.clone(), root_path)
            .expect("fixture source resolves through the canonical parser graph");
        let api = root_key
            .child("api")
            .expect("fixture api module key remains canonical");
        let client = root_key
            .child("client")
            .expect("fixture client module key remains canonical");
        let use_span = graph
            .module_unit(&client)
            .expect("fixture importer unit remains graph-owned")
            .body()
            .uses()[0]
            .span;
        let declaration_span = if private_function {
            graph
                .module_unit(&api)
                .expect("fixture api unit remains graph-owned")
                .body()
                .definitions()
                .iter()
                .find_map(|definition| match definition {
                    Definition::Function(FnDef { name, span, .. })
                        if name.as_ref() == "private_target" =>
                    {
                        Some(*span)
                    }
                    _ => None,
                })
                .expect("fixture retains the private target declaration")
        } else {
            graph
                .module_unit(&root_key)
                .expect("fixture root unit remains graph-owned")
                .body()
                .module_decls()
                .iter()
                .find(|declaration| declaration.name.as_ref() == "api")
                .expect("fixture retains the private structural module declaration")
                .span
        };
        let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
            .expect("fixture graph derives immutable provisional scopes");
        let resolver_error =
            resolve_scoped_glob_ordinary_function_imports_with_scopes(&graph, &scopes)
                .expect_err("private glob paths fail before a resolver plan is published");
        let binder_error = bind_scoped_glob_ordinary_function_imports(&graph, &scopes)
            .expect_err("private glob paths fail before a binding projection is published");

        assert_eq!(binder_error, resolver_error, "{label}");
        match binder_error {
            CanonicalStructuralImportError::Inaccessible {
                declaration_span: rejected_declaration_span,
                use_span: rejected_use_span,
                defining_module,
                violated_visibility,
                ..
            } => {
                assert_eq!(rejected_declaration_span, declaration_span, "{label}");
                assert_eq!(rejected_use_span, use_span, "{label}");
                assert_eq!(defining_module, api, "{label}");
                assert_eq!(violated_visibility, Visibility::Inherited, "{label}");
            }
            other => panic!("expected a {label} accessibility diagnostic, got {other:?}"),
        }

        let _ = fs::remove_dir_all(fixture_root);
    }
}

#[test]
fn scoped_glob_imports_reject_unsupported_shapes_atomically() {
    let cases = [
        (
            "public-use",
            "glob",
            r#"
                pub mod api { pub fn target() -> Int { 1 } }
                pub mod client { pub use crate::api::*; }
            "#,
        ),
        (
            "restricted-use",
            "glob",
            r#"
                pub mod api { pub fn target() -> Int { 1 } }
                pub mod client { pub(in crate) use crate::api::*; }
            "#,
        ),
        (
            "glob-alias",
            "glob",
            r#"
                pub mod api { pub fn target() -> Int { 1 } }
                pub mod client { use crate::api::* as imported; }
            "#,
        ),
        (
            "explicit-simple-use",
            "simple",
            r#"
                pub mod api { pub fn target() -> Int { 1 } }
                pub mod client { use crate::api::target; }
            "#,
        ),
        (
            "nested-group-use",
            "nested",
            r#"
                pub mod api { pub fn target() -> Int { 1 } }
                pub mod client { use crate::api::{target}; }
            "#,
        ),
        (
            "self-base",
            "glob",
            r#"
                pub mod api { pub fn target() -> Int { 1 } }
                pub mod client { use self::api::*; }
            "#,
        ),
        (
            "super-base",
            "glob",
            r#"
                pub mod api { pub fn target() -> Int { 1 } }
                pub mod client { use super::api::*; }
            "#,
        ),
        (
            "repeated-super-base",
            "glob",
            r#"
                pub mod api { pub fn target() -> Int { 1 } }
                pub mod outer { pub mod client { use super::super::api::*; } }
            "#,
        ),
        (
            "root-function-glob",
            "glob",
            r#"
                pub fn root_target() -> Int { 1 }
                pub mod client { use crate::*; }
            "#,
        ),
        (
            "unprefixed-base",
            "glob",
            r#"
                pub mod api { pub fn target() -> Int { 1 } }
                pub mod client { use api::*; }
            "#,
        ),
        (
            "external-base",
            "glob",
            r#"
                pub mod api { pub fn target() -> Int { 1 } }
                pub mod client { use external::api::*; }
            "#,
        ),
        (
            "private-root-target",
            "glob",
            r#"
                fn private_target() -> Int { 1 }
                pub mod client { use crate::private_target::*; }
            "#,
        ),
        (
            "malformed-structural-segment",
            "glob",
            r#"
                pub mod api { pub fn target() -> Int { 1 } }
                pub mod client { use crate::api::self::*; }
            "#,
        ),
        (
            "non-function-source-module-target",
            "glob",
            r#"
                pub mod api { pub type Payload = Int; }
                pub mod client { use crate::api::*; }
            "#,
        ),
        (
            "source-module-containing-child-module",
            "glob",
            r#"
                pub mod api { pub mod nested { pub fn target() -> Int { 1 } } }
                pub mod client { use crate::api::*; }
            "#,
        ),
    ];

    for (label, expected_path_kind, source) in cases {
        let fixture_root = std::env::temp_dir().join(format!(
            "ash-task-2068-scoped-glob-unsupported-shape-{label}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&fixture_root).expect("create temporary parser fixture directory");
        let root_path = fixture_root.join("main.ash");
        fs::write(&root_path, source).expect("write parser fixture source");

        let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
        let graph = CanonicalModuleGraphResolver::new()
            .resolve_root(root_key.clone(), root_path)
            .expect("unsupported-shape fixture resolves through the canonical parser graph");
        let client = if label == "repeated-super-base" {
            root_key
                .child("outer")
                .and_then(|outer| outer.child("client"))
                .expect("nested fixture client module key remains canonical")
        } else {
            root_key
                .child("client")
                .expect("fixture client module key remains canonical")
        };
        let use_declaration = &graph
            .module_unit(&client)
            .expect("fixture importer unit remains graph-owned")
            .body()
            .uses()[0];
        let path_kind_matches = matches!(
            (expected_path_kind, &use_declaration.path),
            ("glob", ash_parser::UsePath::Glob(_))
                | ("simple", ash_parser::UsePath::Simple(_))
                | ("nested", ash_parser::UsePath::Nested(_, _))
        );
        assert!(
            path_kind_matches,
            "{label} has the expected parsed use-path form"
        );
        let use_span = use_declaration.span;
        let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
            .expect("fixture graph derives immutable provisional scopes");
        let resolver_error =
            resolve_scoped_glob_ordinary_function_imports_with_scopes(&graph, &scopes)
                .expect_err("an unsupported shape or target publishes no resolver plan");
        let binder_error = bind_scoped_glob_ordinary_function_imports(&graph, &scopes)
            .expect_err("an unsupported shape or target publishes no binding set");

        assert_eq!(binder_error, resolver_error, "{label}");
        match binder_error {
            CanonicalStructuralImportError::Unsupported { span, .. } => {
                assert_eq!(span, use_span, "{label}");
            }
            other => panic!(
                "expected an atomic full-Use-span Unsupported diagnostic for {label}, got {other:?}"
            ),
        }

        let _ = fs::remove_dir_all(fixture_root);
    }
}

#[test]
fn scoped_glob_imports_reject_conflict_atomically() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2068-scoped-glob-local-conflict-{}",
        std::process::id()
    ));
    fs::create_dir_all(&fixture_root).expect("create temporary parser fixture directory");
    let root_path = fixture_root.join("main.ash");
    fs::write(
        &root_path,
        r#"
            pub mod api { pub fn imported() -> Int { 1 } }
            pub mod client {
                fn imported() -> Int { 2 }
                use crate::api::*;
            }
        "#,
    )
    .expect("write parser fixture source");

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("local-conflict fixture resolves through the canonical parser graph");
    let client = root_key
        .child("client")
        .expect("fixture client module key remains canonical");
    let use_span = graph
        .module_unit(&client)
        .expect("fixture importer unit remains graph-owned")
        .body()
        .uses()[0]
        .span;
    let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
        .expect("fixture graph derives immutable provisional scopes");
    let resolver_error = resolve_scoped_glob_ordinary_function_imports_with_scopes(&graph, &scopes)
        .expect_err("a local ordinary function prevents any scoped-glob resolver plan");
    let binder_error = bind_scoped_glob_ordinary_function_imports(&graph, &scopes)
        .expect_err("a local ordinary function prevents any scoped-glob binding set");

    assert_eq!(binder_error, resolver_error);
    match binder_error {
        CanonicalStructuralImportError::Unsupported { span, reason } => {
            assert_eq!(span, use_span);
            assert_eq!(
                reason,
                "a scoped glob importer cannot declare local ordinary functions"
            );
        }
        other => panic!("expected local scoped-glob boundary diagnostic, got {other:?}"),
    }

    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn scoped_glob_imports_reject_ambiguous_candidate_attempt_atomically() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2068-scoped-glob-ambiguous-attempt-{}",
        std::process::id()
    ));
    fs::create_dir_all(&fixture_root).expect("create temporary parser fixture directory");
    let root_path = fixture_root.join("main.ash");
    fs::write(
        &root_path,
        r#"
            pub mod api { pub fn shared() -> Int { 1 } }
            pub mod alternate_api { pub fn shared() -> Int { 2 } }
            pub mod client {
                use crate::api::*;
                use crate::alternate_api::*;
            }
        "#,
    )
    .expect("write parser fixture source");

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("ambiguous-attempt fixture resolves through the canonical parser graph");
    let client = root_key
        .child("client")
        .expect("fixture client module key remains canonical");
    let use_span = graph
        .module_unit(&client)
        .expect("fixture importer unit remains graph-owned")
        .body()
        .uses()[1]
        .span;
    let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
        .expect("fixture graph derives immutable provisional scopes");
    let resolver_error = resolve_scoped_glob_ordinary_function_imports_with_scopes(&graph, &scopes)
        .expect_err("a second glob prevents candidate precedence and resolver-plan publication");
    let binder_error = bind_scoped_glob_ordinary_function_imports(&graph, &scopes)
        .expect_err("a second glob prevents candidate precedence and binding-set publication");

    assert_eq!(binder_error, resolver_error);
    match binder_error {
        CanonicalStructuralImportError::Unsupported { span, reason } => {
            assert_eq!(span, use_span);
            assert_eq!(
                reason,
                "a scoped glob importer requires exactly one use declaration"
            );
        }
        other => panic!("expected scoped-glob ambiguity-boundary diagnostic, got {other:?}"),
    }

    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn scoped_glob_imports_reject_cycle_shaped_boundary_attempt_atomically() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2068-scoped-glob-cycle-shaped-boundary-{}",
        std::process::id()
    ));
    fs::create_dir_all(&fixture_root).expect("create temporary parser fixture directory");
    let root_path = fixture_root.join("main.ash");
    fs::write(
        &root_path,
        r#"
            pub mod a {
                fn supplied_by_a() -> Int { 1 }
                use crate::b::*;
            }
            pub mod b {
                fn supplied_by_b() -> Int { 2 }
                use crate::a::*;
            }
        "#,
    )
    .expect("write cycle-shaped parser fixture source");

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("cycle-shaped fixture resolves through the canonical parser graph");
    let a = root_key
        .child("a")
        .expect("fixture a module key remains canonical");
    let a_use_span = graph
        .module_unit(&a)
        .expect("fixture a unit remains graph-owned")
        .body()
        .uses()[0]
        .span;
    let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
        .expect("fixture graph derives immutable provisional scopes");
    let resolver_error = resolve_scoped_glob_ordinary_function_imports_with_scopes(&graph, &scopes)
        .expect_err("the cycle-shaped boundary attempt publishes no resolver plan");
    let binder_error = bind_scoped_glob_ordinary_function_imports(&graph, &scopes)
        .expect_err("the cycle-shaped boundary attempt publishes no binding set");

    assert_eq!(binder_error, resolver_error);
    match binder_error {
        CanonicalStructuralImportError::Unsupported { span, reason } => {
            assert_eq!(span, a_use_span);
            assert_eq!(
                reason,
                "a scoped glob importer cannot declare local ordinary functions"
            );
        }
        other => panic!(
            "expected the scoped-glob importer boundary diagnostic instead of a graph-cycle error, got {other:?}"
        ),
    }

    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn scoped_glob_imports_match_file_and_inline_scope_facts() {
    let importer_source = "pub mod client { use crate::api::*; }\n";
    let api_prefix = "pub mod api { ";
    let functions_source =
        "pub fn encode(value: Int) -> Int { value } pub fn decode(value: Int) -> Int { value } ";
    let inline_root_source = format!("{importer_source}{api_prefix}{functions_source}}}\n");
    let file_root_source = format!("{importer_source}pub mod api;\n");
    let file_api_source = functions_source;

    let inline_fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2068-scoped-glob-inline-parity-{}",
        std::process::id()
    ));
    fs::create_dir_all(&inline_fixture_root)
        .expect("create inline temporary parser fixture directory");
    let inline_root_path = inline_fixture_root.join("main.ash");
    fs::write(&inline_root_path, inline_root_source).expect("write inline parser fixture source");

    let file_fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2068-scoped-glob-file-parity-{}",
        std::process::id()
    ));
    fs::create_dir_all(&file_fixture_root)
        .expect("create file-backed temporary parser fixture directory");
    let file_root_path = file_fixture_root.join("main.ash");
    fs::write(&file_root_path, file_root_source).expect("write file-backed root fixture source");
    fs::write(file_fixture_root.join("api.ash"), file_api_source)
        .expect("write file-backed api fixture source");

    let inline_root = ModuleKey::root("app").expect("inline fixture crate key is canonical");
    let inline_graph = CanonicalModuleGraphResolver::new()
        .resolve_root(inline_root.clone(), inline_root_path)
        .expect("inline fixture resolves through the canonical parser graph");
    let file_root = ModuleKey::root("app").expect("file fixture crate key is canonical");
    let file_graph = CanonicalModuleGraphResolver::new()
        .resolve_root(file_root.clone(), file_root_path)
        .expect("file-backed fixture resolves through the canonical parser graph");
    let inline_client = inline_root
        .child("client")
        .expect("inline client module key remains canonical");
    let file_client = file_root
        .child("client")
        .expect("file client module key remains canonical");
    let inline_api = inline_root
        .child("api")
        .expect("inline api module key remains canonical");
    let file_api = file_root
        .child("api")
        .expect("file api module key remains canonical");
    let inline_use_span = inline_graph
        .module_unit(&inline_client)
        .expect("inline client unit remains graph-owned")
        .body()
        .uses()[0]
        .span;
    let file_use_span = file_graph
        .module_unit(&file_client)
        .expect("file client unit remains graph-owned")
        .body()
        .uses()[0]
        .span;
    let inline_scopes = CanonicalProvisionalModuleScopes::from_graph(&inline_graph)
        .expect("inline fixture graph derives immutable provisional scopes");
    let file_scopes = CanonicalProvisionalModuleScopes::from_graph(&file_graph)
        .expect("file fixture graph derives immutable provisional scopes");
    let inline_plan =
        resolve_scoped_glob_ordinary_function_imports_with_scopes(&inline_graph, &inline_scopes)
            .expect("inline fixture resolves through the scoped glob route");
    let file_plan =
        resolve_scoped_glob_ordinary_function_imports_with_scopes(&file_graph, &file_scopes)
            .expect("file fixture resolves through the scoped glob route");
    let inline_bound = bind_scoped_glob_ordinary_function_imports(&inline_graph, &inline_scopes)
        .expect("inline scoped glob plan projects to bindings");
    let file_bound = bind_scoped_glob_ordinary_function_imports(&file_graph, &file_scopes)
        .expect("file-backed scoped glob plan projects to bindings");

    assert_eq!(inline_client, file_client);
    assert_eq!(inline_api, file_api);
    assert_eq!(inline_use_span, file_use_span);
    assert_eq!(inline_plan.import_edges().len(), 2);
    assert_eq!(file_plan.import_edges().len(), 2);

    for name in ["encode", "decode"] {
        let inline_binding = inline_plan
            .binding(&inline_client, name)
            .expect("inline plan retains each natural function name");
        let file_binding = file_plan
            .binding(&file_client, name)
            .expect("file plan retains each natural function name");
        let inline_edge = inline_plan
            .import_edges()
            .iter()
            .find(|edge| edge.local_name() == name)
            .expect("inline plan retains one edge for each selected function");
        let file_edge = file_plan
            .import_edges()
            .iter()
            .find(|edge| edge.local_name() == name)
            .expect("file plan retains one edge for each selected function");
        let inline_declaration_span = inline_graph
            .module_unit(&inline_api)
            .expect("inline api unit remains graph-owned")
            .body()
            .definitions()
            .iter()
            .find_map(|definition| match definition {
                Definition::Function(function) if function.name.as_ref() == name => {
                    Some(function.span)
                }
                _ => None,
            })
            .expect("inline api retains each selected function declaration");
        let file_declaration_span = file_graph
            .module_unit(&file_api)
            .expect("file api unit remains graph-owned")
            .body()
            .definitions()
            .iter()
            .find_map(|definition| match definition {
                Definition::Function(function) if function.name.as_ref() == name => {
                    Some(function.span)
                }
                _ => None,
            })
            .expect("file api retains each selected function declaration");

        assert_eq!(
            inline_bound.binding(&inline_client, name),
            Some(inline_binding)
        );
        assert_eq!(file_bound.binding(&file_client, name), Some(file_binding));
        assert_eq!(
            inline_binding.defining_identity(),
            file_binding.defining_identity()
        );
        assert_eq!(inline_binding.defining_identity().module_key(), &inline_api);
        assert_eq!(file_binding.defining_identity().module_key(), &file_api);
        assert_eq!(inline_binding.defining_identity().name(), name);
        assert_eq!(file_binding.defining_identity().name(), name);
        assert_eq!(inline_binding.declaration_span(), inline_declaration_span);
        assert_eq!(file_binding.declaration_span(), file_declaration_span);
        assert_eq!(inline_binding.visibility(), &Visibility::Public);
        assert_eq!(file_binding.visibility(), &Visibility::Public);
        assert!(matches!(
            inline_binding.origin(),
            ModuleArtifactOrigin::Inline { parent, .. } if parent == &inline_root
        ));
        assert!(matches!(
            file_binding.origin(),
            ModuleArtifactOrigin::File(_)
        ));

        assert_eq!(inline_edge.importing_module(), file_edge.importing_module());
        assert_eq!(inline_edge.defining_module(), file_edge.defining_module());
        assert_eq!(
            inline_edge.defining_identity(),
            file_edge.defining_identity()
        );
        assert_eq!(inline_edge.local_name(), file_edge.local_name());
        assert_eq!(inline_edge.use_span(), inline_use_span);
        assert_eq!(file_edge.use_span(), file_use_span);
        assert_eq!(inline_edge.use_span(), file_edge.use_span());
        assert_eq!(
            inline_edge.declaration_span(),
            inline_binding.declaration_span()
        );
        assert_eq!(
            file_edge.declaration_span(),
            file_binding.declaration_span()
        );
        assert_eq!(inline_edge.visibility(), file_edge.visibility());
        assert_eq!(inline_edge.visibility(), &Visibility::Public);
        assert_eq!(inline_edge.origin(), inline_binding.origin());
        assert_eq!(file_edge.origin(), file_binding.origin());
    }

    let _ = fs::remove_dir_all(inline_fixture_root);
    let _ = fs::remove_dir_all(file_fixture_root);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn scoped_glob_imports_generated_depth_count_visibility_and_source_forms(
        public_child_depth in 1_usize..5,
        function_count in 1_usize..5,
        function_visibility_is_public in any::<bool>(),
        structural_path_is_public in any::<bool>(),
        private_child_seed in any::<u8>(),
        file_backed_source in any::<bool>(),
    ) {
        let child_segments: Vec<_> = (0..public_child_depth)
            .map(|index| format!("scope_{index}"))
            .collect();
        let path = child_segments.join("::");
        let function_names: Vec<_> = (0..function_count)
            .map(|index| format!("generated_function_{index}"))
            .collect();
        let private_child_index = (!structural_path_is_public)
            .then_some(usize::from(private_child_seed) % public_child_depth);
        let function_visibility = if function_visibility_is_public {
            "pub "
        } else {
            ""
        };
        let functions = function_names
            .iter()
            .map(|name| format!("{function_visibility}fn {name}(value: Int) -> Int {{ value }}"))
            .collect::<Vec<_>>()
            .join(" ");
        let mut target_module_source = functions;
        for index in (1..public_child_depth).rev() {
            let visibility = if private_child_index == Some(index) {
                ""
            } else {
                "pub "
            };
            target_module_source = format!(
                "{visibility}mod {} {{ {target_module_source} }}",
                child_segments[index]
            );
        }
        let root_child_visibility = if private_child_index == Some(0) {
            ""
        } else {
            "pub "
        };
        let fixture_root = std::env::temp_dir().join(format!(
            "ash-task-2068-scoped-glob-property-depth-{public_child_depth}-functions-{function_count}-function-public-{function_visibility_is_public}-path-public-{structural_path_is_public}-file-{file_backed_source}-{}",
            std::process::id(),
        ));
        fs::create_dir_all(&fixture_root).expect("create temporary parser fixture directory");
        let root_path = fixture_root.join("main.ash");
        let importer_source = format!("pub mod client {{ use crate::{path}::*; }}");
        if file_backed_source {
            fs::write(
                &root_path,
                format!(
                    "{importer_source} {root_child_visibility}mod {};",
                    child_segments[0]
                ),
            )
            .expect("write generated file-backed root fixture source");
            fs::write(
                fixture_root.join(format!("{}.ash", child_segments[0])),
                target_module_source,
            )
            .expect("write generated file-backed target fixture source");
        } else {
            fs::write(
                &root_path,
                format!(
                    "{importer_source} {root_child_visibility}mod {} {{ {target_module_source} }}",
                    child_segments[0]
                ),
            )
            .expect("write generated inline parser fixture source");
        }

        let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
        let graph = CanonicalModuleGraphResolver::new()
            .resolve_root(root_key.clone(), root_path)
            .expect("generated fixture resolves through the canonical parser graph");
        let client = root_key
            .child("client")
            .expect("fixture client module key remains canonical");
        let target_module = child_segments.iter().fold(root_key.clone(), |module, segment| {
            module
                .child(segment)
                .expect("generated target module key remains canonical")
        });
        let use_span = graph
            .module_unit(&client)
            .expect("generated importer unit remains graph-owned")
            .body()
            .uses()[0]
            .span;
        let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
            .expect("fixture graph derives immutable provisional scopes");
        let resolver = resolve_scoped_glob_ordinary_function_imports_with_scopes(&graph, &scopes);

        if let Some(private_child_index) = private_child_index {
            let rejected_module = child_segments[..=private_child_index]
                .iter()
                .fold(root_key.clone(), |module, segment| {
                    module
                        .child(segment)
                        .expect("generated rejected module key remains canonical")
                });
            let declaring_module = child_segments[..private_child_index]
                .iter()
                .fold(root_key.clone(), |module, segment| {
                    module
                        .child(segment)
                        .expect("generated declaration-owner module key remains canonical")
                });
            let rejected_declaration_span = graph
                .module_unit(&declaring_module)
                .expect("generated declaration-owner unit remains graph-owned")
                .body()
                .module_decls()
                .iter()
                .find(|declaration| declaration.name.as_ref() == child_segments[private_child_index])
                .expect("generated private structural child retains its declaration")
                .span;
            let expected_path = std::iter::once("crate".into())
                .chain(child_segments.iter().cloned().map(Into::into))
                .collect::<Vec<Box<str>>>();
            let resolver_error = resolver
                .expect_err("a private structural path publishes no scoped-glob resolver plan");

            match resolver_error {
                CanonicalStructuralImportError::Inaccessible {
                    declaration_span,
                    use_span: rejected_use_span,
                    defining_module,
                    attempted_path,
                    violated_visibility,
                } => {
                    prop_assert_eq!(declaration_span, rejected_declaration_span);
                    prop_assert_eq!(rejected_use_span, use_span);
                    prop_assert_eq!(defining_module, rejected_module);
                    prop_assert_eq!(attempted_path, expected_path);
                    prop_assert_eq!(violated_visibility, Visibility::Inherited);
                }
                other => prop_assert!(
                    false,
                    "expected a full-use-span private structural-path diagnostic, got {other:?}"
                ),
            }
        } else if !function_visibility_is_public {
            let rejected_declaration_span = graph
                .module_unit(&target_module)
                .expect("generated target unit remains graph-owned")
                .body()
                .definitions()
                .iter()
                .find_map(|definition| match definition {
                    Definition::Function(FnDef { name, span, .. })
                        if name.as_ref() == function_names[0] =>
                    {
                        Some(*span)
                    }
                    _ => None,
                })
                .expect("generated private function retains its declaration");
            let expected_path = std::iter::once("crate".into())
                .chain(child_segments.iter().cloned().map(Into::into))
                .collect::<Vec<Box<str>>>();
            let resolver_error = resolver
                .expect_err("a private function target publishes no scoped-glob resolver plan");

            match resolver_error {
                CanonicalStructuralImportError::Inaccessible {
                    declaration_span,
                    use_span: rejected_use_span,
                    defining_module,
                    attempted_path,
                    violated_visibility,
                } => {
                    prop_assert_eq!(declaration_span, rejected_declaration_span);
                    prop_assert_eq!(rejected_use_span, use_span);
                    prop_assert_eq!(defining_module, target_module);
                    prop_assert_eq!(attempted_path, expected_path);
                    prop_assert_eq!(violated_visibility, Visibility::Inherited);
                }
                other => prop_assert!(
                    false,
                    "expected a full-use-span private function-target diagnostic, got {other:?}"
                ),
            }
        } else {
            let plan = resolver.expect("the scoped glob resolver imports every generated public function");
            let bound = bind_scoped_glob_ordinary_function_imports(&graph, &scopes)
                .expect("the scoped glob binder projects every generated public-function binding");

            prop_assert_eq!(plan.import_edges().len(), function_count);
            for name in &function_names {
                let plan_binding = plan
                    .binding(&client, name)
                    .expect("the resolver retains every generated public natural name");
                let bound_binding = bound
                    .binding(&client, name)
                    .expect("the binder projects every generated public natural name");
                prop_assert_eq!(bound_binding, plan_binding);
                prop_assert_eq!(plan_binding.defining_identity().module_key(), &target_module);
                prop_assert_eq!(plan_binding.defining_identity().name(), name);
                prop_assert_eq!(plan_binding.visibility(), &Visibility::Public);
            }
        }

        let _ = fs::remove_dir_all(fixture_root);
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

fn source_tree_contains(path: &Path, identifier: &str) -> std::io::Result<bool> {
    if path.is_file() {
        return Ok(path.extension().is_some_and(|extension| extension == "rs")
            && fs::read_to_string(path)?.contains(identifier));
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if source_tree_contains(&entry.path(), identifier)? {
            return Ok(true);
        }
    }

    Ok(false)
}

#[test]
fn scoped_glob_import_route_has_only_dedicated_binding_authority_and_no_later_layer_path() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dedicated_path = manifest_dir.join("src/canonical_structural_module_binder.rs");
    let dedicated_source = fs::read_to_string(&dedicated_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 requires the dedicated scoped-glob binder at {}: {error}",
            dedicated_path.display()
        )
    });
    let generic_path = manifest_dir.join("src/canonical_module_binder.rs");
    let generic_source = fs::read_to_string(&generic_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 retains the generic compatibility binder at {}: {error}",
            generic_path.display()
        )
    });
    let lib_path = manifest_dir.join("src/lib.rs");
    let lib_source = fs::read_to_string(&lib_path).unwrap_or_else(|error| {
        panic!(
            "read type-checker public exports at {}: {error}",
            lib_path.display()
        )
    });
    let planner_path = manifest_dir.join("src/canonical_simple_import_planner.rs");
    let planner_source = fs::read_to_string(&planner_path).unwrap_or_else(|error| {
        panic!(
            "read scoped-glob planner authority boundary at {}: {error}",
            planner_path.display()
        )
    });

    assert!(
        dedicated_source.contains("resolve_scoped_glob_ordinary_function_imports_with_scopes"),
        "only the dedicated scoped binder consumes the scoped-glob resolver",
    );
    assert!(
        dedicated_source.contains("bind_scoped_glob_ordinary_function_imports"),
        "the private structural binder owns the named scoped-glob projection",
    );
    assert!(
        dedicated_source.contains(".map(|plan| plan.into_bound_set())"),
        "the dedicated binder must only project a successful resolver plan",
    );
    assert!(
        lib_source.contains("bind_scoped_glob_ordinary_function_imports"),
        "lib.rs alone re-exports the dedicated scoped-glob binding API",
    );
    assert!(
        lib_source.contains("resolve_scoped_glob_ordinary_function_imports_with_scopes"),
        "lib.rs re-exports the scoped-glob resolver as the public Type-layer entry point",
    );

    let generic_binder_bytes = fs::read(&generic_path).unwrap_or_else(|error| {
        panic!(
            "read generic compatibility binder under an authority fence at {}: {error}",
            generic_path.display()
        )
    });
    let actual_checksum = format!("{:x}", Sha256::digest(generic_binder_bytes));
    assert_eq!(
        actual_checksum, "aea47f6aae83b4b7e3bfaa9ee9a7561d76b12a5f82b50c42d3fd2d98e23ed8b6",
        "the generic compatibility binder must remain byte-for-byte generic-only",
    );

    for forbidden_identifier in [
        "CanonicalStructuralImportError",
        "CanonicalProvisionalModuleScopes",
        "resolve_simple_parsed_imports_with_scopes",
        "resolve_scoped_simple_ordinary_function_imports_with_scopes",
        "resolve_scoped_grouped_ordinary_function_imports_with_scopes",
        "resolve_scoped_super_ordinary_function_imports_with_scopes",
        "resolve_scoped_glob_ordinary_function_imports_with_scopes",
        "bind_scoped_glob_ordinary_function_imports",
    ] {
        assert!(
            !contains_exact_identifier(&generic_source, forbidden_identifier),
            "the generic binder must remain generic-only and omit {forbidden_identifier}",
        );
    }

    for forbidden_identifier in [
        "PublicModuleInterface",
        "FinalizedModuleInterface",
        "InterfaceImportResolver",
        "CheckedInterfaceStore",
        "TypeEnvModuleInterfaceCollection",
        "RawCoreProgram",
        "CoreExpr",
        "CpsProgram",
        "Engine",
        "Admission",
        "Runtime",
        "Daemon",
        "Cli",
    ] {
        assert!(
            !contains_exact_identifier(&dedicated_source, forbidden_identifier),
            "the dedicated scoped-glob binder must not gain wider authority: {forbidden_identifier}",
        );
        assert!(
            !contains_exact_identifier(&planner_source, forbidden_identifier),
            "the scoped-glob planner must not gain wider authority: {forbidden_identifier}",
        );
    }
    for forbidden_bypass in [
        "canonical_module_binder",
        "bind_simple_parsed_uses",
        "interface_import_resolver",
        "module_interface_finalization",
        "module_core_cps_lowering",
        "std::fs",
        "read_to_string",
        "parse_surface",
        "resolve_root",
        "from_legacy",
        "into_legacy",
    ] {
        assert!(
            !dedicated_source.contains(forbidden_bypass),
            "the dedicated scoped-glob binder must not bypass the resolver: {forbidden_bypass}",
        );
        assert!(
            !planner_source.contains(forbidden_bypass),
            "the scoped-glob planner must not bypass parser-owned graph facts: {forbidden_bypass}",
        );
    }

    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("ash-typeck lives below the workspace root");
    for later_layer_source in [
        manifest_dir.join("src/module_interface_finalization.rs"),
        manifest_dir.join("src/module_core_cps_lowering.rs"),
        workspace_root.join("crates/ash-engine/src"),
        workspace_root.join("crates/ash-cli/src"),
    ] {
        for route_identifier in [
            "resolve_scoped_glob_ordinary_function_imports_with_scopes",
            "bind_scoped_glob_ordinary_function_imports",
        ] {
            assert!(
                !source_tree_contains(&later_layer_source, route_identifier).unwrap_or_else(
                    |error| {
                        panic!(
                            "read later-layer authority fence at {}: {error}",
                            later_layer_source.display()
                        )
                    },
                ),
                "later-layer source {} must not consume scoped-glob authority {route_identifier}",
                later_layer_source.display(),
            );
        }
    }
}
