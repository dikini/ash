//! TASK-2068 RED contract for scoped grouped `super` ordinary-function imports.
//!
//! This target reserves the Type-layer binding plan for inherited non-root
//! `use super::{function, function as local}` declarations. Every member keeps
//! its parser-owned span; the route neither widens generic binding nor
//! authorizes later layers.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::{ModuleArtifactOrigin, ModuleKey};
use ash_parser::surface::{Definition, FnDef, Visibility};
use ash_parser::use_tree::UsePath;
use ash_parser::{CanonicalModuleGraph, CanonicalModuleGraphResolver, Span};
use ash_typeck::{
    CanonicalProvisionalModuleScopes, CanonicalStructuralImportError,
    bind_scoped_super_grouped_ordinary_function_imports,
    resolve_scoped_super_grouped_ordinary_function_imports_with_scopes,
};
use proptest::prelude::*;
use sha2::{Digest, Sha256};

static NEXT_TEMP_TREE: AtomicUsize = AtomicUsize::new(0);

/// A real parser fixture whose drop implementation removes its source tree.
struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(label: &str) -> Self {
        let serial = NEXT_TEMP_TREE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "ash-task-2068-scoped-super-grouped-imports-{label}-{}-{serial}",
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

fn parsed_graph(source: &str) -> (ModuleKey, CanonicalModuleGraph) {
    let tree = TempTree::new("positive");
    let root_path = tree.write("src/main.ash", source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("fixture source must resolve through the canonical parser graph");
    (root_key, graph)
}

fn file_backed_graph(
    root_source: &str,
    module_path: impl AsRef<Path>,
    module_source: &str,
    label: &str,
) -> (ModuleKey, CanonicalModuleGraph) {
    let tree = TempTree::new(label);
    let root_path = tree.write("src/main.ash", root_source);
    tree.write(module_path, module_source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("file-backed fixture source must resolve through the canonical parser graph");
    (root_key, graph)
}

fn module_key(root: &ModuleKey, segments: &[&str]) -> ModuleKey {
    segments.iter().fold(root.clone(), |key, segment| {
        key.child(segment)
            .expect("fixture module path remains canonical")
    })
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
        .expect("fixture function remains parser-owned by its canonical module")
}

fn group_member_spans(graph: &CanonicalModuleGraph, module: &ModuleKey) -> Vec<Span> {
    group_member_spans_at(graph, module, 0)
}

fn group_member_spans_at(
    graph: &CanonicalModuleGraph,
    module: &ModuleKey,
    use_index: usize,
) -> Vec<Span> {
    let use_declaration = graph
        .module_unit(module)
        .expect("fixture importing module has an acquired canonical unit")
        .body()
        .uses()
        .get(use_index)
        .expect("fixture importing module contains the selected grouped parsed use declaration");
    let UsePath::Nested(_, members) = &use_declaration.path else {
        panic!("fixture must retain parser-owned grouped use members");
    };
    members.iter().map(|member| member.span).collect()
}

fn use_span_at(graph: &CanonicalModuleGraph, module: &ModuleKey, use_index: usize) -> Span {
    graph
        .module_unit(module)
        .expect("fixture importing module has an acquired canonical unit")
        .body()
        .uses()
        .get(use_index)
        .expect("fixture importing module contains the selected parsed use declaration")
        .span
}

#[test]
fn scoped_super_grouped_imports_project_parent_members_with_natural_and_aliased_local_names() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod parent {
                pub fn first() -> Int { 1 }
                pub fn second() -> Int { 2 }
                pub mod child {
                    use super::{first, second as renamed};
                }
            }
        "#,
    );
    let parent = module_key(&root_key, &["parent"]);
    let child = module_key(&root_key, &["parent", "child"]);
    let member_spans = group_member_spans(&graph, &child);
    let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
        .expect("the canonical parser graph derives immutable provisional scopes");

    let plan = resolve_scoped_super_grouped_ordinary_function_imports_with_scopes(&graph, &scopes)
        .expect("a non-root child may group-import its public parent functions through super");
    let bound = bind_scoped_super_grouped_ordinary_function_imports(&graph, &scopes)
        .expect("the dedicated binder projects the successful scoped-super-group plan");

    assert_eq!(member_spans.len(), 2);
    assert_ne!(member_spans[0], member_spans[1]);
    assert_eq!(plan.import_edges().len(), 2);

    for (index, local_name, defining_name) in [(0, "first", "first"), (1, "renamed", "second")] {
        let binding = plan
            .binding(&child, local_name)
            .expect("each grouped member is staged under its selected local name");
        let edge = plan
            .import_edges()
            .iter()
            .find(|edge| edge.local_name() == local_name)
            .expect("each parent member retains its own cross-module edge");

        assert_eq!(bound.binding(&child, local_name), Some(binding));
        assert_eq!(binding.defining_identity().module_key(), &parent);
        assert_eq!(binding.defining_identity().name(), defining_name);
        assert_eq!(edge.importing_module(), &child);
        assert_eq!(edge.defining_module(), &parent);
        assert_eq!(edge.use_span(), member_spans[index]);
    }
}

#[test]
fn scoped_super_grouped_imports_preserve_each_member_identity_and_parser_provenance() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod host {
                pub mod provider {
                    pub fn first() -> Int { 1 }
                    pub(crate) fn second() -> Int { 2 }
                }
                pub mod client {
                    use super::provider::{first, second as local_second};
                }
            }
        "#,
    );
    let provider = module_key(&root_key, &["host", "provider"]);
    let client = module_key(&root_key, &["host", "client"]);
    let target_origin = graph
        .module_unit(&provider)
        .expect("fixture provider module has an acquired canonical unit")
        .artifact()
        .origin()
        .clone();
    let member_spans = group_member_spans(&graph, &client);
    let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
        .expect("the canonical parser graph derives immutable provisional scopes");
    let plan = resolve_scoped_super_grouped_ordinary_function_imports_with_scopes(&graph, &scopes)
        .expect("public and crate-visible sibling members resolve through one leading super");
    let bound = bind_scoped_super_grouped_ordinary_function_imports(&graph, &scopes)
        .expect("the dedicated binder retains every staged scoped-super grouped member");

    assert_eq!(member_spans.len(), 2);
    assert_ne!(member_spans[0], member_spans[1]);
    assert_eq!(plan.import_edges().len(), 2);

    for (index, local_name, defining_name, visibility) in [
        (0, "first", "first", Visibility::Public),
        (1, "local_second", "second", Visibility::Crate),
    ] {
        let target = graph
            .module_unit(&provider)
            .expect("fixture provider module has an acquired canonical unit")
            .body()
            .definitions()
            .iter()
            .find_map(|definition| match definition {
                Definition::Function(function) if function.name.as_ref() == defining_name => {
                    Some(function)
                }
                _ => None,
            })
            .expect("fixture provider retains each selected ordinary function");
        let binding = plan
            .binding(&client, local_name)
            .expect("each grouped member retains its selected local name");
        let edge = plan
            .import_edges()
            .iter()
            .find(|edge| edge.local_name() == local_name)
            .expect("each cross-module grouped member retains its own import edge");

        assert_eq!(bound.binding(&client, local_name), Some(binding));
        assert_eq!(binding.defining_identity().module_key(), &provider);
        assert_eq!(binding.defining_identity().name(), defining_name);
        assert_eq!(binding.declaration_span(), target.span);
        assert_eq!(binding.origin(), &target_origin);
        assert_eq!(binding.visibility(), &visibility);
        assert_eq!(edge.importing_module(), &client);
        assert_eq!(edge.defining_module(), &provider);
        assert_eq!(edge.local_name(), local_name);
        assert_eq!(edge.defining_identity(), binding.defining_identity());
        assert_eq!(edge.declaration_span(), target.span);
        assert_eq!(edge.origin(), &target_origin);
        assert_eq!(edge.visibility(), &visibility);
        assert_eq!(edge.use_span(), member_spans[index]);
    }
}

#[test]
fn scoped_super_grouped_imports_enforce_visibility_regions_for_parent_sibling_and_structural_paths()
{
    let permitted_cases = [
        (
            "public-parent",
            r#"
                pub mod host {
                    pub fn target() -> Int { 1 }
                    pub mod client { use super::{target as local}; }
                }
            "#,
            &["host", "client"][..],
            &["host"][..],
            Visibility::Public,
            true,
        ),
        (
            "crate-parent",
            r#"
                pub mod host {
                    pub(crate) fn target() -> Int { 1 }
                    pub mod client { use super::{target as local}; }
                }
            "#,
            &["host", "client"][..],
            &["host"][..],
            Visibility::Crate,
            true,
        ),
        (
            "super-parent",
            r#"
                pub mod host {
                    pub(super) fn target() -> Int { 1 }
                    pub mod client { use super::{target as local}; }
                }
            "#,
            &["host", "client"][..],
            &["host"][..],
            Visibility::Super { levels: 1 },
            true,
        ),
        (
            "restricted-parent",
            r#"
                pub mod host {
                    pub(in crate::host) fn target() -> Int { 1 }
                    pub mod client { use super::{target as local}; }
                }
            "#,
            &["host", "client"][..],
            &["host"][..],
            Visibility::Restricted {
                path: "crate::host".into(),
            },
            true,
        ),
        (
            "self-same-module-no-edge",
            r#"
                pub mod host {
                    pub mod client {
                        pub(self) fn target() -> Int { 1 }
                        use super::client::{target as local};
                    }
                }
            "#,
            &["host", "client"][..],
            &["host", "client"][..],
            Visibility::Self_,
            false,
        ),
        (
            "public-sibling-full-path",
            r#"
                pub mod host {
                    pub mod provider { pub mod api { pub fn target() -> Int { 1 } } }
                    pub mod client { use super::provider::api::{target as local}; }
                }
            "#,
            &["host", "client"][..],
            &["host", "provider", "api"][..],
            Visibility::Public,
            true,
        ),
        (
            "crate-sibling-full-path",
            r#"
                pub mod host {
                    pub mod provider { pub mod api { pub(crate) fn target() -> Int { 1 } } }
                    pub mod client { use super::provider::api::{target as local}; }
                }
            "#,
            &["host", "client"][..],
            &["host", "provider", "api"][..],
            Visibility::Crate,
            true,
        ),
        (
            "super-sibling",
            r#"
                pub mod host {
                    pub mod provider { pub(super) fn target() -> Int { 1 } }
                    pub mod client { use super::provider::{target as local}; }
                }
            "#,
            &["host", "client"][..],
            &["host", "provider"][..],
            Visibility::Super { levels: 1 },
            true,
        ),
        (
            "restricted-sibling-full-path",
            r#"
                pub mod host {
                    pub mod provider {
                        pub mod api { pub(in crate::host) fn target() -> Int { 1 } }
                    }
                    pub mod client { use super::provider::api::{target as local}; }
                }
            "#,
            &["host", "client"][..],
            &["host", "provider", "api"][..],
            Visibility::Restricted {
                path: "crate::host".into(),
            },
            true,
        ),
    ];

    for (label, source, importer_segments, target_segments, expected_visibility, expects_edge) in
        permitted_cases
    {
        let (root_key, graph) = parsed_graph(source);
        let importer = module_key(&root_key, importer_segments);
        let target_module = module_key(&root_key, target_segments);
        let member_span = group_member_spans(&graph, &importer)[0];
        let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
            .expect("the canonical parser graph derives immutable provisional scopes");
        let plan =
            resolve_scoped_super_grouped_ordinary_function_imports_with_scopes(&graph, &scopes)
                .expect(
                    "the selected parent or sibling importer lies within this visibility region",
                );
        let bound = bind_scoped_super_grouped_ordinary_function_imports(&graph, &scopes)
            .expect("the dedicated binder projects every visibility-permitted group");
        let binding = plan
            .binding(&importer, "local")
            .expect("the resolver retains the permitted grouped local name");
        assert_eq!(bound.binding(&importer, "local"), Some(binding), "{label}");
        assert_eq!(
            binding.defining_identity().module_key(),
            &target_module,
            "{label}"
        );
        assert_eq!(binding.visibility(), &expected_visibility, "{label}");
        if expects_edge {
            let edge = plan
                .import_edges()
                .iter()
                .find(|edge| edge.local_name() == "local")
                .expect("the cross-module member retains its import edge");
            assert_eq!(edge.use_span(), member_span, "{label}");
        } else {
            assert!(
                plan.import_edges().is_empty(),
                "{label}: a same-module grouped-super binding has no import edge"
            );
        }
    }

    let rejected_cases = [
        (
            "public-behind-private-full-path",
            r#"
                pub mod host {
                    pub mod provider { mod hidden { pub fn target() -> Int { 1 } } }
                    pub mod client { use super::provider::hidden::{target as local}; }
                }
            "#,
            &["host", "client"][..],
            &["host", "provider"][..],
            "hidden",
            Visibility::Inherited,
        ),
        (
            "crate-behind-private-full-path",
            r#"
                pub mod host {
                    pub mod provider {
                        mod hidden { pub(crate) fn target() -> Int { 1 } }
                    }
                    pub mod client { use super::provider::hidden::{target as local}; }
                }
            "#,
            &["host", "client"][..],
            &["host", "provider"][..],
            "hidden",
            Visibility::Inherited,
        ),
        (
            "super-sibling-outside-provider-region",
            r#"
                pub mod host {
                    pub mod provider {
                        pub mod api { pub(super) fn target() -> Int { 1 } }
                    }
                    pub mod client { use super::provider::api::{target as local}; }
                }
            "#,
            &["host", "client"][..],
            &["host", "provider", "api"][..],
            "target",
            Visibility::Super { levels: 1 },
        ),
        (
            "restricted-sibling-outside-provider-region",
            r#"
                pub mod host {
                    pub mod provider {
                        pub mod api {
                            pub(in crate::host::provider) fn target() -> Int { 1 }
                        }
                    }
                    pub mod client { use super::provider::api::{target as local}; }
                }
            "#,
            &["host", "client"][..],
            &["host", "provider", "api"][..],
            "target",
            Visibility::Restricted {
                path: "crate::host::provider".into(),
            },
        ),
    ];

    for (label, source, importer_segments, defining_segments, rejected_name, expected_visibility) in
        rejected_cases
    {
        let (root_key, graph) = parsed_graph(source);
        let importer = module_key(&root_key, importer_segments);
        let declaration_owner = module_key(&root_key, defining_segments);
        let declaration_span = if rejected_name == "hidden" {
            graph
                .module_unit(&declaration_owner)
                .expect("fixture structural parent remains graph-owned")
                .body()
                .module_decls()
                .iter()
                .find(|declaration| declaration.name.as_ref() == rejected_name)
                .expect("fixture retains the rejected structural declaration")
                .span
        } else {
            function(&graph, &declaration_owner, rejected_name).span
        };
        let defining_module = if rejected_name == "hidden" {
            declaration_owner
                .child(rejected_name)
                .expect("fixture structural child key remains canonical")
        } else {
            declaration_owner
        };
        let member_span = group_member_spans(&graph, &importer)[0];
        let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
            .expect("the canonical parser graph derives immutable provisional scopes");
        let resolver_error =
            resolve_scoped_super_grouped_ordinary_function_imports_with_scopes(&graph, &scopes)
                .expect_err("a denied visibility region publishes no grouped resolver plan");
        let binder_error = bind_scoped_super_grouped_ordinary_function_imports(&graph, &scopes)
            .expect_err("a denied visibility region publishes no grouped binding set");

        assert_eq!(binder_error, resolver_error, "{label}");
        match binder_error {
            CanonicalStructuralImportError::Inaccessible {
                declaration_span: rejected_declaration_span,
                use_span,
                defining_module: rejected_defining_module,
                violated_visibility,
                ..
            } => {
                assert_eq!(rejected_declaration_span, declaration_span, "{label}");
                assert_eq!(use_span, member_span, "{label}");
                assert_eq!(rejected_defining_module, defining_module, "{label}");
                assert_eq!(violated_visibility, expected_visibility, "{label}");
            }
            other => panic!("expected an anchored {label} visibility diagnostic, got {other:?}"),
        }
    }
}

#[test]
fn scoped_super_grouped_imports_reject_every_reserved_shape_at_its_member_or_whole_use_boundary() {
    let member_cases = [
        (
            "root-super",
            r#"
                pub fn target() -> Int { 1 }
                use super::{target};
            "#,
            &[][..],
        ),
        (
            "repeated-super-path",
            r#"
                pub mod outer { pub mod child { use super::super::{target}; } }
            "#,
            &["outer", "child"][..],
        ),
        (
            "final-member-super",
            r#"
                pub mod host {
                    pub fn super() -> Int { 1 }
                    pub mod child { use super::{super}; }
                }
            "#,
            &["host", "child"][..],
        ),
        (
            "outer-alias",
            r#"
                pub mod host {
                    pub fn target() -> Int { 1 }
                    pub mod child { use super::{target} as outer; }
                }
            "#,
            &["host", "child"][..],
        ),
        (
            "self-head",
            r#"
                pub mod host { pub mod child { use self::{target}; } }
            "#,
            &["host", "child"][..],
        ),
        (
            "crate-head",
            r#"
                pub mod host { pub mod child { use crate::{target}; } }
            "#,
            &["host", "child"][..],
        ),
        (
            "unprefixed-head",
            r#"
                pub mod host { pub mod child { use target::{target}; } }
            "#,
            &["host", "child"][..],
        ),
        (
            "stdlib-head",
            r#"
                pub mod host { pub mod child { use std::{target}; } }
            "#,
            &["host", "child"][..],
        ),
        (
            "external-head",
            r#"
                pub mod host { pub mod child { use external::{target}; } }
            "#,
            &["host", "child"][..],
        ),
        (
            "public-use",
            r#"
                pub mod host {
                    pub fn target() -> Int { 1 }
                    pub mod child { pub use super::{target}; }
                }
            "#,
            &["host", "child"][..],
        ),
        (
            "restricted-use",
            r#"
                pub mod host {
                    pub fn target() -> Int { 1 }
                    pub mod child { pub(crate) use super::{target}; }
                }
            "#,
            &["host", "child"][..],
        ),
    ];

    for (label, source, importer_segments) in member_cases {
        let (root_key, graph) = parsed_graph(source);
        let importer = module_key(&root_key, importer_segments);
        let member_span = group_member_spans(&graph, &importer)[0];
        let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
            .expect("the canonical parser graph derives immutable provisional scopes");
        let resolver_error =
            resolve_scoped_super_grouped_ordinary_function_imports_with_scopes(&graph, &scopes)
                .expect_err("a parsed reserved grouped shape publishes no resolver plan");
        let binder_error = bind_scoped_super_grouped_ordinary_function_imports(&graph, &scopes)
            .expect_err("a parsed reserved grouped shape publishes no binding set");

        assert_eq!(binder_error, resolver_error, "{label}");
        match binder_error {
            CanonicalStructuralImportError::Unsupported { span, .. } => {
                assert_eq!(span, member_span, "{label}");
            }
            other => panic!("expected a member-anchored {label} rejection, got {other:?}"),
        }
    }

    let no_member_cases = [
        (
            "empty-group",
            r#"
                pub mod host { pub mod child { use super::{}; } }
            "#,
        ),
        (
            "simple-path",
            r#"
                pub mod host {
                    pub fn target() -> Int { 1 }
                    pub mod child { use super::target; }
                }
            "#,
        ),
        (
            "glob-path",
            r#"
                pub mod host {
                    pub fn target() -> Int { 1 }
                    pub mod child { use super::*; }
                }
            "#,
        ),
    ];

    for (label, source) in no_member_cases {
        let (root_key, graph) = parsed_graph(source);
        let importer = module_key(&root_key, &["host", "child"]);
        let use_span = use_span_at(&graph, &importer, 0);
        let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
            .expect("the canonical parser graph derives immutable provisional scopes");
        let resolver_error =
            resolve_scoped_super_grouped_ordinary_function_imports_with_scopes(&graph, &scopes)
                .expect_err("a no-member route shape publishes no resolver plan");
        let binder_error = bind_scoped_super_grouped_ordinary_function_imports(&graph, &scopes)
            .expect_err("a no-member route shape publishes no binding set");

        assert_eq!(binder_error, resolver_error, "{label}");
        match binder_error {
            CanonicalStructuralImportError::Unsupported { span, .. } => {
                assert_eq!(span, use_span, "{label}");
            }
            other => panic!("expected a whole-use {label} rejection, got {other:?}"),
        }
    }
}

#[test]
fn scoped_super_grouped_imports_reject_a_later_type_member_at_its_own_span_atomically() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod host {
                pub fn available() -> Int { 1 }
                pub type Payload = Int;
                pub mod client { use super::{available, Payload as local_payload}; }
            }
        "#,
    );
    let client = module_key(&root_key, &["host", "client"]);
    let rejected_member_span = group_member_spans(&graph, &client)[1];
    let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
        .expect("the canonical parser graph derives immutable provisional scopes");
    let resolver_error =
        resolve_scoped_super_grouped_ordinary_function_imports_with_scopes(&graph, &scopes)
            .expect_err("a later type member rejects the complete grouped resolver result");
    let binder_error = bind_scoped_super_grouped_ordinary_function_imports(&graph, &scopes)
        .expect_err("a later type member publishes no partial grouped binding set");

    assert_eq!(binder_error, resolver_error);
    match binder_error {
        CanonicalStructuralImportError::Unsupported { span, reason } => {
            assert_eq!(span, rejected_member_span);
            assert_eq!(reason, "only ordinary function targets are accepted");
        }
        other => panic!("expected an anchored non-function member rejection, got {other:?}"),
    }
}

#[test]
fn scoped_super_grouped_imports_preflight_final_super_before_private_child_visibility() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod host {
                mod hidden { pub fn super() -> Int { 1 } }
                pub mod child { use super::hidden::{super}; }
            }
        "#,
    );
    let child = module_key(&root_key, &["host", "child"]);
    let member_span = group_member_spans(&graph, &child)[0];
    let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
        .expect("the canonical parser graph derives immutable provisional scopes");
    let resolver_error =
        resolve_scoped_super_grouped_ordinary_function_imports_with_scopes(&graph, &scopes)
            .expect_err("a final super member rejects before private-child visibility lookup");
    let binder_error = bind_scoped_super_grouped_ordinary_function_imports(&graph, &scopes)
        .expect_err("the dedicated binder publishes no partial set after final-super rejection");

    assert_eq!(binder_error, resolver_error);
    match binder_error {
        CanonicalStructuralImportError::Unsupported { span, reason } => {
            assert_eq!(span, member_span);
            assert_eq!(
                reason,
                "a scoped super grouped import accepts exactly one leading super"
            );
        }
        other => {
            panic!("expected final-super rejection before private-child visibility, got {other:?}")
        }
    }
}

#[test]
fn scoped_super_grouped_imports_document_nested_group_parser_rejection_before_the_dedicated_route()
{
    let tree = TempTree::new("nested-group-parser-rejection");
    let root_path = tree.write(
        "src/main.ash",
        r#"
            pub mod host {
                pub mod child { use super::{provider::{target}}; }
            }
        "#,
    );
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new().resolve_root(root_key, root_path);

    assert!(
        graph.is_err(),
        "nested group syntax produces no Use AST, so it is parser-rejection evidence rather than a dedicated-route diagnostic"
    );
}

#[test]
fn scoped_super_grouped_imports_reject_member_aliases_that_collide_with_local_functions_atomically()
{
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod host {
                pub fn first() -> Int { 1 }
                pub fn second() -> Int { 2 }
                pub mod child {
                    fn local() -> Int { 3 }
                    use super::{first, second as local};
                }
            }
        "#,
    );
    let child = module_key(&root_key, &["host", "child"]);
    let local = function(&graph, &child, "local");
    let rejected_member_span = group_member_spans(&graph, &child)[1];
    let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
        .expect("the canonical parser graph derives immutable provisional scopes");
    let resolver_error =
        resolve_scoped_super_grouped_ordinary_function_imports_with_scopes(&graph, &scopes)
            .expect_err("a member alias may not overwrite an ordinary local function");
    let binder_error = bind_scoped_super_grouped_ordinary_function_imports(&graph, &scopes)
        .expect_err("a local-collision error publishes neither a plan nor a bound set");

    assert_eq!(binder_error, resolver_error);
    match binder_error {
        CanonicalStructuralImportError::LocalDeclarationCollision {
            importing_module,
            name,
            declaration_span,
            use_span,
        } => {
            assert_eq!(importing_module, child);
            assert_eq!(name, "local");
            assert_eq!(declaration_span, local.span);
            assert_eq!(use_span, rejected_member_span);
        }
        other => panic!("expected an anchored grouped-super local collision, got {other:?}"),
    }
}

#[test]
fn scoped_super_grouped_imports_reject_duplicate_natural_and_aliased_bindings_with_member_spans() {
    let duplicate_cases = [
        (
            "same-group-natural-natural",
            r#"
                pub mod host {
                    pub fn first() -> Int { 1 }
                    pub mod child { use super::{first, first}; }
                }
            "#,
            "first",
            0,
            1,
        ),
        (
            "same-group-natural-alias",
            r#"
                pub mod host {
                    pub fn first() -> Int { 1 }
                    pub fn second() -> Int { 2 }
                    pub mod child { use super::{first, second as first}; }
                }
            "#,
            "first",
            0,
            1,
        ),
        (
            "same-group-alias-alias",
            r#"
                pub mod host {
                    pub fn first() -> Int { 1 }
                    pub fn second() -> Int { 2 }
                    pub mod child { use super::{first as shared, second as shared}; }
                }
            "#,
            "shared",
            0,
            1,
        ),
        (
            "across-uses-natural-natural",
            r#"
                pub mod host {
                    pub fn first() -> Int { 1 }
                    pub mod child {
                        use super::{first};
                        use super::{first};
                    }
                }
            "#,
            "first",
            1,
            0,
        ),
        (
            "across-uses-natural-alias",
            r#"
                pub mod host {
                    pub fn first() -> Int { 1 }
                    pub fn second() -> Int { 2 }
                    pub mod child {
                        use super::{first};
                        use super::{second as first};
                    }
                }
            "#,
            "first",
            1,
            0,
        ),
        (
            "across-uses-alias-alias",
            r#"
                pub mod host {
                    pub fn first() -> Int { 1 }
                    pub fn second() -> Int { 2 }
                    pub mod child {
                        use super::{first as shared};
                        use super::{second as shared};
                    }
                }
            "#,
            "shared",
            1,
            0,
        ),
    ];

    for (label, source, duplicate_name, rejected_use_index, rejected_member_index) in
        duplicate_cases
    {
        let (root_key, graph) = parsed_graph(source);
        let child = module_key(&root_key, &["host", "child"]);
        let rejected_member_span =
            group_member_spans_at(&graph, &child, rejected_use_index)[rejected_member_index];
        let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
            .expect("the canonical parser graph derives immutable provisional scopes");
        let resolver_error =
            resolve_scoped_super_grouped_ordinary_function_imports_with_scopes(&graph, &scopes)
                .expect_err("a duplicate local spelling rejects the complete staged grouped plan");
        let binder_error = bind_scoped_super_grouped_ordinary_function_imports(&graph, &scopes)
            .expect_err("a duplicate local spelling publishes no partial grouped bound set");

        assert_eq!(binder_error, resolver_error, "{label}");
        match binder_error {
            CanonicalStructuralImportError::DuplicateBinding {
                importing_module,
                name,
                use_span,
            } => {
                assert_eq!(importing_module, child, "{label}");
                assert_eq!(name.as_ref(), duplicate_name, "{label}");
                assert_eq!(use_span, rejected_member_span, "{label}");
            }
            other => panic!("expected anchored {label} duplicate rejection, got {other:?}"),
        }
    }
}

#[test]
fn scoped_super_grouped_imports_reject_a_deterministic_parent_sibling_cycle_atomically() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod host {
                pub fn host_fn() -> Int { 0 }
                pub mod a {
                    use super::{host_fn};
                    use super::b::{b_fn};
                    pub fn a_fn() -> Int { 1 }
                }
                pub mod b {
                    use super::c::{c_fn};
                    pub fn b_fn() -> Int { 2 }
                }
                pub mod c {
                    use super::b::{b_fn};
                    pub fn c_fn() -> Int { 3 }
                }
            }
        "#,
    );
    let a = module_key(&root_key, &["host", "a"]);
    let b = module_key(&root_key, &["host", "b"]);
    let c = module_key(&root_key, &["host", "c"]);
    let b_member_span = group_member_spans(&graph, &b)[0];
    let c_member_span = group_member_spans(&graph, &c)[0];
    let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
        .expect("the canonical parser graph derives immutable provisional scopes");
    let resolver_error =
        resolve_scoped_super_grouped_ordinary_function_imports_with_scopes(&graph, &scopes)
            .expect_err("the b-to-c-to-b member edges close a canonical import cycle");
    let binder_error = bind_scoped_super_grouped_ordinary_function_imports(&graph, &scopes)
        .expect_err("a cycle publishes neither the earlier plan nor any bound set");

    assert_eq!(binder_error, resolver_error);
    match binder_error {
        CanonicalStructuralImportError::ImportCycle { edges } => {
            assert_eq!(edges.edges().len(), 2);
            assert!(
                edges
                    .edges()
                    .iter()
                    .all(|edge| edge.importing_module() != &a),
                "the earlier valid parent and sibling members remain unavailable after cycle rejection",
            );
            assert_eq!(edges.edges()[0].importing_module(), &b);
            assert_eq!(edges.edges()[0].defining_module(), &c);
            assert_eq!(edges.edges()[0].use_span(), b_member_span);
            assert_eq!(edges.edges()[1].importing_module(), &c);
            assert_eq!(edges.edges()[1].defining_module(), &b);
            assert_eq!(edges.edges()[1].use_span(), c_member_span);
        }
        other => panic!("expected a deterministic grouped-super import cycle, got {other:?}"),
    }
}

#[test]
fn scoped_super_grouped_imports_keep_file_and_inline_parity_without_erasing_member_facts() {
    let (inline_root, inline_graph) = parsed_graph(
        r#"
            pub mod host {
                pub mod provider {
                    pub fn normalize(value: Int) -> Int { value }
                    pub fn sanitize(value: Int) -> Int { value }
                }
                pub mod child {
                    use super::provider::{normalize, sanitize as clean};
                }
            }
        "#,
    );
    let (file_root, file_graph) = file_backed_graph(
        r#"
            pub mod host {
                pub mod provider;
                pub mod child {
                    use super::provider::{normalize, sanitize as clean};
                }
            }
        "#,
        "src/provider.ash",
        r#"
            pub fn normalize(value: Int) -> Int { value }
            pub fn sanitize(value: Int) -> Int { value }
        "#,
        "file-inline-member-parity",
    );
    let inline_host = module_key(&inline_root, &["host"]);
    let file_host = module_key(&file_root, &["host"]);
    let inline_provider = module_key(&inline_root, &["host", "provider"]);
    let file_provider = module_key(&file_root, &["host", "provider"]);
    let inline_child = module_key(&inline_root, &["host", "child"]);
    let file_child = module_key(&file_root, &["host", "child"]);
    let inline_member_spans = group_member_spans(&inline_graph, &inline_child);
    let file_member_spans = group_member_spans(&file_graph, &file_child);
    let inline_scopes = CanonicalProvisionalModuleScopes::from_graph(&inline_graph)
        .expect("the inline graph derives immutable provisional scopes");
    let file_scopes = CanonicalProvisionalModuleScopes::from_graph(&file_graph)
        .expect("the file graph derives immutable provisional scopes");
    let inline_plan = resolve_scoped_super_grouped_ordinary_function_imports_with_scopes(
        &inline_graph,
        &inline_scopes,
    )
    .expect("the inline grouped-super graph is scope-resolvable");
    let file_plan = resolve_scoped_super_grouped_ordinary_function_imports_with_scopes(
        &file_graph,
        &file_scopes,
    )
    .expect("the file-backed grouped-super graph is scope-resolvable");
    let inline_bound =
        bind_scoped_super_grouped_ordinary_function_imports(&inline_graph, &inline_scopes)
            .expect("the inline plan projects to binding-only facts");
    let file_bound = bind_scoped_super_grouped_ordinary_function_imports(&file_graph, &file_scopes)
        .expect("the file-backed plan projects to binding-only facts");

    assert_eq!(inline_root, file_root);
    assert_eq!(inline_host, file_host);
    assert_eq!(inline_provider, file_provider);
    assert_eq!(inline_child, file_child);
    assert_eq!(inline_member_spans.len(), 2);
    assert_eq!(file_member_spans.len(), 2);
    assert_ne!(inline_member_spans[0], inline_member_spans[1]);
    assert_ne!(file_member_spans[0], file_member_spans[1]);

    for (index, local_name, defining_name) in
        [(0, "normalize", "normalize"), (1, "clean", "sanitize")]
    {
        let inline_binding = inline_plan
            .binding(&inline_child, local_name)
            .expect("the inline plan retains each local member name");
        let file_binding = file_plan
            .binding(&file_child, local_name)
            .expect("the file plan retains each local member name");
        let inline_edge = inline_plan
            .import_edges()
            .iter()
            .find(|edge| edge.local_name() == local_name)
            .expect("the inline plan retains an edge for each member");
        let file_edge = file_plan
            .import_edges()
            .iter()
            .find(|edge| edge.local_name() == local_name)
            .expect("the file plan retains an edge for each member");

        assert_eq!(
            inline_bound.binding(&inline_child, local_name),
            Some(inline_binding)
        );
        assert_eq!(
            file_bound.binding(&file_child, local_name),
            Some(file_binding)
        );
        assert_eq!(
            inline_binding.defining_identity(),
            file_binding.defining_identity()
        );
        assert_eq!(
            inline_binding.defining_identity().module_key(),
            &inline_provider
        );
        assert_eq!(
            file_binding.defining_identity().module_key(),
            &file_provider
        );
        assert_eq!(inline_binding.defining_identity().name(), defining_name);
        assert_eq!(file_binding.defining_identity().name(), defining_name);
        assert_eq!(
            inline_binding.declaration_span(),
            function(&inline_graph, &inline_provider, defining_name).span
        );
        assert_eq!(
            file_binding.declaration_span(),
            function(&file_graph, &file_provider, defining_name).span
        );
        assert_eq!(inline_binding.visibility(), file_binding.visibility());
        assert!(matches!(
            inline_binding.origin(),
            ModuleArtifactOrigin::Inline { parent, .. } if parent == &inline_host
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
        assert_eq!(inline_edge.visibility(), file_edge.visibility());
        assert_eq!(inline_edge.use_span(), inline_member_spans[index]);
        assert_eq!(file_edge.use_span(), file_member_spans[index]);
        assert_eq!(
            inline_edge.declaration_span(),
            inline_binding.declaration_span()
        );
        assert_eq!(
            file_edge.declaration_span(),
            file_binding.declaration_span()
        );
        assert_eq!(inline_edge.origin(), inline_binding.origin());
        assert_eq!(file_edge.origin(), file_binding.origin());
    }
}

fn rendered_member(name: &str, alias: Option<&str>) -> String {
    alias.map_or_else(|| name.to_owned(), |alias| format!("{name} as {alias}"))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn scoped_super_grouped_imports_match_generated_members_to_canonical_visibility_regions_and_importer_shapes(
        function_suffix in "[a-z][a-z0-9_]{0,8}",
        alias_suffix in "[a-z][a-z0-9_]{0,8}",
        structural_child_depth in 0_u8..3,
        function_count in 1_usize..4,
        alias_even_members in any::<bool>(),
        private_child_seed in 0_u8..3,
        visibility_case in 0_u8..6,
        reversed_members in any::<bool>(),
        importer_shape in 0_u8..3,
    ) {
        let host = format!("host_{function_suffix}");
        let client = format!("client_{function_suffix}");
        let provider = format!("provider_{function_suffix}");
        let private_structural_child = importer_shape == 1 && private_child_seed != 0;
        let structural_child_depth = if importer_shape == 1 {
            if private_structural_child {
                usize::from(structural_child_depth.max(1))
            } else {
                usize::from(structural_child_depth)
            }
        } else {
            0
        };
        let structural_children: Vec<_> = (0..structural_child_depth)
            .map(|index| format!("sibling_{function_suffix}_{index}"))
            .collect();
        let private_child_index = if private_structural_child {
            Some(usize::from(private_child_seed) % structural_child_depth)
        } else {
            None
        };
        let function_names: Vec<_> = (0..function_count)
            .map(|index| format!("member_{function_suffix}_{index}"))
            .collect();
        let local_names: Vec<_> = function_names
            .iter()
            .enumerate()
            .map(|(index, name)| {
                if importer_shape == 2 || (alias_even_members && index % 2 == 0) {
                    format!("local_{alias_suffix}_{index}")
                } else {
                    name.clone()
                }
            })
            .collect();
        let member_indices = if reversed_members {
            (0..function_count).rev().collect::<Vec<_>>()
        } else {
            (0..function_count).collect::<Vec<_>>()
        };
        let members = member_indices
            .iter()
            .map(|&index| {
                rendered_member(
                    &function_names[index],
                    (function_names[index] != local_names[index])
                        .then_some(local_names[index].as_str()),
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        let expected_visibility = match visibility_case {
            0 => Visibility::Public,
            1 => Visibility::Crate,
            2 => Visibility::Super { levels: 1 },
            3 => Visibility::Self_,
            4 => Visibility::Restricted {
                path: format!("crate::{host}").into(),
            },
            _ => Visibility::Inherited,
        };
        let function_visibility = match visibility_case {
            0 => "pub ".to_owned(),
            1 => "pub(crate) ".to_owned(),
            2 => "pub(super) ".to_owned(),
            3 => "pub(self) ".to_owned(),
            4 => format!("pub(in crate::{host}) "),
            _ => String::new(),
        };
        let function_declarations = function_names
            .iter()
            .map(|name| format!("{function_visibility}fn {name}() -> Int {{ 1 }}"))
            .collect::<Vec<_>>()
            .join(" ");
        let (source, importer_segments, target_segments) = match importer_shape {
            0 => (
                format!(
                    "pub mod {host} {{ {function_declarations} pub mod {client} {{ use super::{{{members}}}; }} }}"
                ),
                vec![host.clone(), client.clone()],
                vec![host.clone()],
            ),
            1 => {
                let target_declarations = structural_children
                    .iter()
                    .enumerate()
                    .rev()
                    .fold(function_declarations, |nested, (index, child)| {
                        let visibility = if private_child_index == Some(index) {
                            ""
                        } else {
                            "pub "
                        };
                        format!("{visibility}mod {child} {{ {nested} }}")
                    });
                let route = if structural_children.is_empty() {
                    format!("super::{provider}")
                } else {
                    format!("super::{provider}::{}", structural_children.join("::"))
                };
                let mut target_segments = vec![host.clone(), provider.clone()];
                target_segments.extend(structural_children.iter().cloned());
                (
                    format!(
                        "pub mod {host} {{ pub mod {provider} {{ {target_declarations} }} pub mod {client} {{ use {route}::{{{members}}}; }} }}"
                    ),
                    vec![host.clone(), client.clone()],
                    target_segments,
                )
            }
            _ => (
                format!(
                    "pub mod {host} {{ pub mod {client} {{ {function_declarations} use super::{client}::{{{members}}}; }} }}"
                ),
                vec![host.clone(), client.clone()],
                vec![host.clone(), client.clone()],
            ),
        };
        let (root_key, graph) = parsed_graph(&source);
        let importer_segment_refs = importer_segments
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let target_segment_refs = target_segments
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let importer = module_key(&root_key, &importer_segment_refs);
        let target_module = module_key(&root_key, &target_segment_refs);
        let member_spans = group_member_spans(&graph, &importer);
        let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
            .expect("generated canonical parser graph derives immutable provisional scopes");
        let resolver = resolve_scoped_super_grouped_ordinary_function_imports_with_scopes(&graph, &scopes);
        let binder = bind_scoped_super_grouped_ordinary_function_imports(&graph, &scopes);
        let function_visible = scopes
            .is_visible_from(&expected_visibility, &target_module, &importer)
            .expect("the canonical predicate decides the generated target visibility region");
        let mut first_inaccessible_child = None;
        for (index, child) in structural_children.iter().enumerate() {
            let child_visibility = if private_child_index == Some(index) {
                Visibility::Inherited
            } else {
                Visibility::Public
            };
            let child_segment_refs = target_segments[..index + 3]
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>();
            let child_module = module_key(&root_key, &child_segment_refs);
            let child_visible = scopes
                .is_visible_from(&child_visibility, &child_module, &importer)
                .expect("the canonical predicate decides every generated structural child region");
            if !child_visible && first_inaccessible_child.is_none() {
                let declaring_segment_refs = target_segments[..index + 2]
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>();
                let declaring_module = module_key(&root_key, &declaring_segment_refs);
                let declaration_span = graph
                    .module_unit(&declaring_module)
                    .expect("the generated structural parent remains graph-owned")
                    .body()
                    .module_decls()
                    .iter()
                    .find(|declaration| declaration.name.as_ref() == child)
                    .expect("the generated structural child retains its parser declaration")
                    .span;
                first_inaccessible_child = Some((declaration_span, child_module, child_visibility));
            }
        }

        prop_assert_eq!(member_spans.len(), function_count);

        if first_inaccessible_child.is_none() && function_visible {
            let plan = resolver.expect(
                "a generated route within every canonical visibility region is admitted",
            );
            let bound = binder.expect(
                "the dedicated binder projects every generated visible grouped-super binding",
            );

            prop_assert_eq!(
                plan.import_edges().len(),
                if importer == target_module { 0 } else { function_count },
            );
            for (&member_index, member_span) in member_indices
                .iter()
                .zip(&member_spans)
            {
                let local_name = &local_names[member_index];
                let defining_name = &function_names[member_index];
                let binding = plan
                    .binding(&importer, local_name)
                    .expect("each visible grouped member retains its generated local name");

                prop_assert_eq!(bound.binding(&importer, local_name), Some(binding));
                prop_assert_eq!(binding.defining_identity().module_key(), &target_module);
                prop_assert_eq!(binding.defining_identity().name(), defining_name);
                prop_assert_eq!(binding.visibility(), &expected_visibility);
                if importer != target_module {
                    let edge = plan
                        .import_edges()
                        .iter()
                        .find(|edge| edge.local_name() == local_name)
                        .expect("each cross-module member retains its own import edge");
                    prop_assert_eq!(edge.use_span(), *member_span);
                }
            }
        } else {
            let resolver_error = resolver.expect_err(
                "a generated route outside a canonical visibility region publishes no plan",
            );
            let binder_error = binder.expect_err(
                "the dedicated binder publishes no grouped-super bindings after visibility rejection",
            );

            prop_assert_eq!(&binder_error, &resolver_error);
            let (expected_declaration_span, expected_defining_module, expected_visibility) =
                first_inaccessible_child.unwrap_or_else(|| {
                    (
                        function(&graph, &target_module, &function_names[member_indices[0]]).span,
                        target_module.clone(),
                        expected_visibility.clone(),
                    )
                });

            match binder_error {
                CanonicalStructuralImportError::Inaccessible {
                    declaration_span,
                    use_span,
                    defining_module,
                    violated_visibility,
                    ..
                } => {
                    prop_assert_eq!(declaration_span, expected_declaration_span);
                    prop_assert_eq!(use_span, member_spans[0]);
                    prop_assert_eq!(defining_module, expected_defining_module);
                    prop_assert_eq!(violated_visibility, expected_visibility);
                }
                other => prop_assert!(
                    false,
                    "expected the generated grouped-super visibility error at its first member span, got {other:?}"
                ),
            }
        }
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

fn rust_sources_containing(path: &Path, identifier: &str) -> Vec<PathBuf> {
    if path.is_file() {
        return (path.extension().is_some_and(|extension| extension == "rs")
            && fs::read_to_string(path)
                .expect("read Rust source under an authority fence")
                .contains(identifier))
        .then(|| path.to_path_buf())
        .into_iter()
        .collect();
    }

    let mut sources = fs::read_dir(path)
        .expect("read authority-fenced source directory")
        .flatten()
        .flat_map(|entry| rust_sources_containing(&entry.path(), identifier))
        .collect::<Vec<_>>();
    sources.sort();
    sources
}

#[test]
fn scoped_super_grouped_import_route_has_only_dedicated_type_layer_authority() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source_dir = manifest_dir.join("src");
    let dedicated_path = source_dir.join("canonical_structural_module_binder.rs");
    let planner_path = source_dir.join("canonical_simple_import_planner.rs");
    let generic_path = source_dir.join("canonical_module_binder.rs");
    let lib_path = source_dir.join("lib.rs");
    let dedicated_source = fs::read_to_string(&dedicated_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 requires the dedicated grouped-super binder at {}: {error}",
            dedicated_path.display()
        )
    });
    let planner_source = fs::read_to_string(&planner_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 requires the grouped-super planner at {}: {error}",
            planner_path.display()
        )
    });
    let generic_source = fs::read_to_string(&generic_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 retains the generic compatibility binder at {}: {error}",
            generic_path.display()
        )
    });
    let lib_source = fs::read_to_string(&lib_path).unwrap_or_else(|error| {
        panic!(
            "read type-checker public exports at {}: {error}",
            lib_path.display()
        )
    });

    assert!(
        dedicated_source.contains("bind_scoped_super_grouped_ordinary_function_imports"),
        "only the dedicated binder owns the grouped-super binding projection",
    );
    assert!(
        dedicated_source
            .contains("resolve_scoped_super_grouped_ordinary_function_imports_with_scopes"),
        "the dedicated binder must delegate to the grouped-super resolver",
    );
    assert!(
        dedicated_source.contains(".map(|plan| plan.into_bound_set())"),
        "the dedicated binder may only project a successful resolver plan",
    );
    assert!(
        planner_source
            .contains("resolve_scoped_super_grouped_ordinary_function_imports_with_scopes"),
        "the scoped grouped-super resolver remains planner-owned",
    );
    assert!(
        lib_source.contains("resolve_scoped_super_grouped_ordinary_function_imports_with_scopes")
            && lib_source.contains("bind_scoped_super_grouped_ordinary_function_imports"),
        "lib.rs re-exports exactly the dedicated grouped-super Type-layer entry points",
    );

    let mut expected_resolver_authority = vec![
        dedicated_path.clone(),
        planner_path.clone(),
        lib_path.clone(),
    ];
    expected_resolver_authority.sort();
    assert_eq!(
        rust_sources_containing(
            &source_dir,
            "resolve_scoped_super_grouped_ordinary_function_imports_with_scopes",
        ),
        expected_resolver_authority,
        "only the planner, dedicated binder, and public Type-layer export may name the grouped-super resolver",
    );
    let mut expected_binder_authority = vec![dedicated_path.clone(), lib_path.clone()];
    expected_binder_authority.sort();
    assert_eq!(
        rust_sources_containing(
            &source_dir,
            "bind_scoped_super_grouped_ordinary_function_imports"
        ),
        expected_binder_authority,
        "only the dedicated binder and public Type-layer export may name the grouped-super binder",
    );

    let actual_generic_digest = format!("{:x}", Sha256::digest(generic_source.as_bytes()));
    assert_eq!(
        actual_generic_digest, "aea47f6aae83b4b7e3bfaa9ee9a7561d76b12a5f82b50c42d3fd2d98e23ed8b6",
        "the generic compatibility binder must remain byte-for-byte unchanged",
    );
    for forbidden_generic_identifier in [
        "CanonicalStructuralImportError",
        "CanonicalProvisionalModuleScopes",
        "resolve_scoped_super_grouped_ordinary_function_imports_with_scopes",
        "bind_scoped_super_grouped_ordinary_function_imports",
    ] {
        assert!(
            !contains_exact_identifier(&generic_source, forbidden_generic_identifier),
            "the generic binder must remain generic-only and omit {forbidden_generic_identifier}",
        );
    }

    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("ash-typeck lives below the workspace root");
    for later_layer_source in [
        source_dir.join("module_interface_finalization.rs"),
        source_dir.join("module_core_cps_lowering.rs"),
        workspace_root.join("crates/ash-engine/src"),
        workspace_root.join("crates/ash-runtime/src"),
        workspace_root.join("crates/ash-cli/src"),
    ] {
        for route_identifier in [
            "resolve_scoped_super_grouped_ordinary_function_imports_with_scopes",
            "bind_scoped_super_grouped_ordinary_function_imports",
        ] {
            assert!(
                rust_sources_containing(&later_layer_source, route_identifier).is_empty(),
                "later-layer source {} must not consume grouped-super authority {route_identifier}",
                later_layer_source.display(),
            );
        }
    }
}
