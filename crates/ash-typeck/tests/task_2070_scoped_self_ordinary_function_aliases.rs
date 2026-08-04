//! TASK-2070 RED contracts for direct same-module ordinary-function aliases.
//!
//! This target reserves only inherited, explicitly renamed, two-segment
//! `self` aliases. The dedicated route preserves parser provenance without
//! manufacturing cross-module import edges or cycle authority.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::ModuleKey;
use ash_parser::surface::{Definition, FnDef, Visibility};
use ash_parser::{CanonicalModuleGraph, CanonicalModuleGraphResolver, Span};
use ash_typeck::{
    CanonicalBoundSelfOrdinaryFunctionAliasSet, CanonicalProvisionalModuleScopes,
    CanonicalResolvedSelfOrdinaryFunctionAliases, CanonicalStructuralImportError,
    bind_scoped_self_ordinary_function_imports,
    resolve_scoped_self_ordinary_function_imports_with_scopes,
};
use proptest::prelude::*;
use sha2::{Digest, Sha256};

static NEXT_TEMP_TREE: AtomicUsize = AtomicUsize::new(0);

struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(label: &str) -> Self {
        let serial = NEXT_TEMP_TREE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "ash-task-2070-scoped-self-aliases-{label}-{}-{serial}",
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

fn parsed_graph(source: &str, label: &str) -> (ModuleKey, CanonicalModuleGraph) {
    let tree = TempTree::new(label);
    let root_path = tree.write("src/main.ash", source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("fixture source must resolve through the canonical parser graph");
    (root_key, graph)
}

fn file_backed_graph(
    root_source: &str,
    module_name: &str,
    module_source: &str,
    label: &str,
) -> (ModuleKey, CanonicalModuleGraph) {
    let tree = TempTree::new(label);
    let root_path = tree.write("src/main.ash", root_source);
    tree.write(format!("src/{module_name}.ash"), module_source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("file-backed fixture source resolves through the canonical parser graph");
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

fn first_use_span(graph: &CanonicalModuleGraph, module: &ModuleKey) -> Span {
    graph
        .module_unit(module)
        .expect("fixture importing module has an acquired canonical unit")
        .body()
        .uses()
        .first()
        .expect("fixture importing module contains a parsed use declaration")
        .span
}

fn use_spans(graph: &CanonicalModuleGraph, module: &ModuleKey) -> Vec<Span> {
    graph
        .module_unit(module)
        .expect("fixture importing module has an acquired canonical unit")
        .body()
        .uses()
        .iter()
        .map(|use_declaration| use_declaration.span)
        .collect()
}

fn scopes(graph: &CanonicalModuleGraph) -> CanonicalProvisionalModuleScopes {
    CanonicalProvisionalModuleScopes::from_graph(graph)
        .expect("a structurally complete parser graph derives immutable provisional scopes")
}

fn resolve(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
) -> Result<CanonicalResolvedSelfOrdinaryFunctionAliases, CanonicalStructuralImportError> {
    resolve_scoped_self_ordinary_function_imports_with_scopes(graph, scopes)
}

fn braced_item_source<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing planned item {signature}"));
    let body_start = source[start..]
        .find('{')
        .map(|offset| start + offset)
        .expect("planned item has a body");
    let mut depth = 0_usize;
    for (offset, character) in source[body_start..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[start..body_start + offset + 1];
                }
            }
            _ => {}
        }
    }
    panic!("planned item {signature} has an unterminated body")
}

fn function_source<'a>(source: &'a str, function_name: &str) -> &'a str {
    braced_item_source(source, &format!("pub fn {function_name}"))
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
fn root_and_nested_self_aliases_respect_same_module_visibility_regions() {
    let (empty_root, empty_graph) = parsed_graph("fn untouched() -> Int { 0 }", "zero-aliases");
    let empty_scopes = scopes(&empty_graph);
    let empty_plan = resolve(&empty_graph, &empty_scopes)
        .expect("a graph with zero use declarations produces an empty dedicated result");
    let empty_bound = bind_scoped_self_ordinary_function_imports(&empty_graph, &empty_scopes)
        .expect("the binder projects the empty dedicated result");
    assert_eq!(empty_plan.binding(&empty_root, "untouched_alias"), None);
    assert_eq!(empty_bound.binding(&empty_root, "untouched_alias"), None);

    let (root_key, graph) = parsed_graph(
        r#"
            fn root_private() -> Int { 1 }
            use self::root_private as root_alias;

            pub mod nested {
                pub fn public_target() -> Int { 2 }
                pub(crate) fn crate_target() -> Int { 3 }
                pub(super) fn super_target() -> Int { 4 }
                pub(in crate::nested) fn restricted_target() -> Int { 5 }
                pub(self) fn self_target() -> Int { 6 }
                fn inherited_target() -> Int { 7 }

                use self::public_target as public_alias;
                use self::crate_target as crate_alias;
                use self::super_target as super_alias;
                use self::restricted_target as restricted_alias;
                use self::self_target as self_alias;
                use self::inherited_target as inherited_alias;
            }
        "#,
        "root-nested-visibility",
    );
    let nested_key = module_key(&root_key, &["nested"]);
    let scope_snapshot = scopes(&graph);
    let plan = resolve(&graph, &scope_snapshot)
        .expect("all same-module visibility regions permit their explicit self aliases");
    let bound = bind_scoped_self_ordinary_function_imports(&graph, &scope_snapshot)
        .expect("multiple distinct root and nested aliases bind together");

    let expected = [
        (
            &root_key,
            "root_alias",
            "root_private",
            Visibility::Inherited,
        ),
        (
            &nested_key,
            "public_alias",
            "public_target",
            Visibility::Public,
        ),
        (
            &nested_key,
            "crate_alias",
            "crate_target",
            Visibility::Crate,
        ),
        (
            &nested_key,
            "super_alias",
            "super_target",
            Visibility::Super { levels: 1 },
        ),
        (
            &nested_key,
            "restricted_alias",
            "restricted_target",
            Visibility::Restricted {
                path: "crate::nested".into(),
            },
        ),
        (&nested_key, "self_alias", "self_target", Visibility::Self_),
        (
            &nested_key,
            "inherited_alias",
            "inherited_target",
            Visibility::Inherited,
        ),
    ];

    for (module, alias, target, visibility) in expected {
        let plan_binding = plan
            .binding(module, alias)
            .unwrap_or_else(|| panic!("resolver retains distinct alias {alias}"));
        assert_eq!(bound.binding(module, alias), Some(plan_binding));
        assert_eq!(plan_binding.local_alias(), alias);
        assert_eq!(plan_binding.defining_identity().module_key(), module);
        assert_eq!(plan_binding.defining_identity().name(), target);
        assert_eq!(plan_binding.visibility(), &visibility);
    }
}

#[test]
fn self_alias_preserves_defining_identity_and_provenance() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod nested {
                pub(self) fn normalize(value: Int) -> Int { value }
                use self::normalize as normalize_here;
            }
        "#,
        "identity-provenance",
    );
    let nested_key = module_key(&root_key, &["nested"]);
    let target = function(&graph, &nested_key, "normalize");
    let target_origin = graph
        .module_unit(&nested_key)
        .expect("target module is graph-owned")
        .artifact()
        .origin()
        .clone();
    let complete_use_span = first_use_span(&graph, &nested_key);
    let scope_snapshot = scopes(&graph);

    let plan = resolve(&graph, &scope_snapshot)
        .expect("the resolver accepts a direct explicit different self alias");
    let bound = bind_scoped_self_ordinary_function_imports(&graph, &scope_snapshot)
        .expect("the dedicated binder projects the resolved alias");
    let plan_binding = plan
        .binding(&nested_key, "normalize_here")
        .expect("the resolver exposes the local alias binding");
    let bound_binding = bound
        .binding(&nested_key, "normalize_here")
        .expect("the binder exposes the same local alias binding");

    assert_eq!(bound_binding, plan_binding);
    for binding in [plan_binding, bound_binding] {
        assert_eq!(binding.local_alias(), "normalize_here");
        assert_eq!(binding.defining_identity().module_key(), &nested_key);
        assert_eq!(binding.defining_identity().name(), "normalize");
        assert_eq!(binding.declaration_span(), target.span);
        assert_eq!(binding.origin(), &target_origin);
        assert_eq!(binding.visibility(), &Visibility::Self_);
        assert_eq!(binding.use_span(), complete_use_span);
    }
}

#[test]
fn self_alias_emits_no_edge_and_never_selects_a_false_cycle() {
    let (root_key, graph) = parsed_graph(
        r#"
            fn root_target() -> Int { 1 }
            use self::root_target as root_alias;
            pub mod nested {
                fn nested_target() -> Int { 2 }
                use self::nested_target as nested_alias;
            }
        "#,
        "no-edge-no-false-cycle",
    );
    let nested_key = module_key(&root_key, &["nested"]);
    let scope_snapshot = scopes(&graph);
    let plan: CanonicalResolvedSelfOrdinaryFunctionAliases = resolve(&graph, &scope_snapshot)
        .expect("same-module aliases cannot create self-loop import cycles");
    let bound: CanonicalBoundSelfOrdinaryFunctionAliasSet =
        bind_scoped_self_ordinary_function_imports(&graph, &scope_snapshot)
            .expect("the binder never selects a false cycle for same-module aliases");
    assert!(plan.binding(&root_key, "root_alias").is_some());
    assert!(plan.binding(&nested_key, "nested_alias").is_some());
    assert!(bound.binding(&root_key, "root_alias").is_some());
    assert!(bound.binding(&nested_key, "nested_alias").is_some());

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let planner_source =
        fs::read_to_string(manifest_dir.join("src/canonical_simple_import_planner.rs"))
            .expect("read the dedicated self-alias resolver source");
    let resolver_source = function_source(
        &planner_source,
        "resolve_scoped_self_ordinary_function_imports_with_scopes",
    );
    for forbidden in [
        "CanonicalSimpleImportEdge",
        "CanonicalResolvedSimpleImports",
        "detect_import_cycle",
        "ImportCycle",
    ] {
        assert!(
            !resolver_source.contains(forbidden),
            "the dedicated self-alias resolver must omit edge/cycle authority: {forbidden}",
        );
    }
}

#[test]
fn self_alias_rejects_shape_and_visibility_boundaries() {
    let (root_key, graph) = parsed_graph(
        "fn target() -> Int { 1 } use self::target as local;",
        "exact-accepted-shape",
    );
    let scope_snapshot = scopes(&graph);
    let plan = resolve(&graph, &scope_snapshot)
        .expect("an inherited exact self function path with a different alias is accepted");
    assert!(plan.binding(&root_key, "local").is_some());

    let unsupported_cases = [
        (
            "direct-child-nonfunction",
            "mod child { fn target() -> Int { 1 } } use self::child as local;",
        ),
        (
            "child-traversal",
            "mod child { fn target() -> Int { 1 } } use self::child::target as local;",
        ),
        ("natural-name", "fn target() -> Int { 1 } use self::target;"),
        (
            "equal-alias",
            "fn target() -> Int { 1 } use self::target as target;",
        ),
        (
            "crate-path",
            "fn target() -> Int { 1 } use crate::target as local;",
        ),
        (
            "super-path",
            "pub mod nested { fn target() -> Int { 1 } use super::nested as local; }",
        ),
        (
            "unprefixed-path",
            "fn target() -> Int { 1 } use target as local;",
        ),
        (
            "grouped-path",
            "fn target() -> Int { 1 } use self::{target as local};",
        ),
        ("glob-path", "fn target() -> Int { 1 } use self::*;"),
        (
            "public-use",
            "fn target() -> Int { 1 } pub use self::target as local;",
        ),
        (
            "restricted-use",
            "fn target() -> Int { 1 } pub(crate) use self::target as local;",
        ),
        (
            "mixed-valid-and-invalid-uses",
            "fn first() -> Int { 1 } fn second() -> Int { 2 } use self::first as first_alias; use self::{second as second_alias};",
        ),
        (
            "direct-type-nonfunction",
            "type Target = Int; use self::Target as local;",
        ),
    ];

    for (label, source) in unsupported_cases {
        let (_, graph) = parsed_graph(source, label);
        let scope_snapshot = scopes(&graph);
        let resolver_error = resolve(&graph, &scope_snapshot)
            .expect_err("every non-exact self-alias form is rejected before publication");
        let binder_error = bind_scoped_self_ordinary_function_imports(&graph, &scope_snapshot)
            .expect_err("the binder preserves each unsupported resolver diagnostic");
        assert_eq!(binder_error, resolver_error, "{label}");
        assert!(
            matches!(
                resolver_error,
                CanonicalStructuralImportError::Unsupported { .. }
            ),
            "{label} must be Unsupported, got {resolver_error:?}",
        );
    }

    let (_, unresolved_graph) = parsed_graph(
        "fn target() -> Int { 1 } use self::missing as local;",
        "unresolved-target",
    );
    let unresolved_scopes = scopes(&unresolved_graph);
    assert!(matches!(
        resolve(&unresolved_graph, &unresolved_scopes),
        Err(CanonicalStructuralImportError::Unresolved { .. })
    ));
    assert!(matches!(
        bind_scoped_self_ordinary_function_imports(&unresolved_graph, &unresolved_scopes),
        Err(CanonicalStructuralImportError::Unresolved { .. })
    ));

    let (_, inaccessible_graph) = parsed_graph(
        r#"
            pub mod allowed {}
            pub(in crate::allowed) fn target() -> Int { 1 }
            use self::target as local;
        "#,
        "inaccessible-target",
    );
    let inaccessible_scopes = scopes(&inaccessible_graph);
    assert!(matches!(
        resolve(&inaccessible_graph, &inaccessible_scopes),
        Err(CanonicalStructuralImportError::Inaccessible { .. })
    ));
    assert!(matches!(
        bind_scoped_self_ordinary_function_imports(&inaccessible_graph, &inaccessible_scopes),
        Err(CanonicalStructuralImportError::Inaccessible { .. })
    ));
}

#[test]
fn self_alias_valid_sibling_failure_is_atomic() {
    let (_, graph) = parsed_graph(
        r#"
            pub mod valid {
                fn target() -> Int { 1 }
                use self::target as valid_alias;
            }
            pub mod failing {
                use self::missing as missing_alias;
            }
        "#,
        "valid-and-failing-siblings",
    );
    let scope_snapshot = scopes(&graph);
    let resolver_error = resolve(&graph, &scope_snapshot)
        .expect_err("a failing sibling prevents publication of the valid sibling's alias set");
    let binder_error = bind_scoped_self_ordinary_function_imports(&graph, &scope_snapshot)
        .expect_err("the binder cannot project a partial graph result");
    assert_eq!(binder_error, resolver_error);
    assert!(matches!(
        resolver_error,
        CanonicalStructuralImportError::Unresolved { .. }
    ));

    let (root_key, duplicate_graph) = parsed_graph(
        r#"
            fn first() -> Int { 1 }
            fn second() -> Int { 2 }
            use self::first as shared;
            use self::second as shared;
        "#,
        "duplicate-valid-aliases",
    );
    let duplicate_spans = use_spans(&duplicate_graph, &root_key);
    let duplicate_scopes = scopes(&duplicate_graph);
    let resolver_error = resolve(&duplicate_graph, &duplicate_scopes)
        .expect_err("the second otherwise-valid alias duplicates the first local spelling");
    let binder_error =
        bind_scoped_self_ordinary_function_imports(&duplicate_graph, &duplicate_scopes)
            .expect_err("duplicate aliases publish no bound alias set");
    assert_eq!(binder_error, resolver_error);
    match resolver_error {
        CanonicalStructuralImportError::DuplicateBinding {
            importing_module,
            name,
            use_span,
        } => {
            assert_eq!(importing_module, root_key);
            assert_eq!(name.as_ref(), "shared");
            assert_eq!(use_span, duplicate_spans[1]);
        }
        other => panic!("expected second-use duplicate binding diagnostic, got {other:?}"),
    }

    let (root_key, collision_graph) = parsed_graph(
        r#"
            fn target() -> Int { 1 }
            fn occupied() -> Int { 2 }
            use self::target as occupied;
        "#,
        "local-alias-collision",
    );
    let occupied = function(&collision_graph, &root_key, "occupied");
    let collision_use_span = first_use_span(&collision_graph, &root_key);
    let collision_scopes = scopes(&collision_graph);
    let resolver_error = resolve(&collision_graph, &collision_scopes)
        .expect_err("a self alias cannot overwrite a direct local function");
    let binder_error =
        bind_scoped_self_ordinary_function_imports(&collision_graph, &collision_scopes)
            .expect_err("a local collision publishes no bound alias set");
    assert_eq!(binder_error, resolver_error);
    match resolver_error {
        CanonicalStructuralImportError::LocalDeclarationCollision {
            importing_module,
            name,
            declaration_span,
            use_span,
        } => {
            assert_eq!(importing_module, root_key);
            assert_eq!(name, "occupied");
            assert_eq!(declaration_span, occupied.span);
            assert_eq!(use_span, collision_use_span);
        }
        other => panic!("expected local alias collision diagnostic, got {other:?}"),
    }
}

#[test]
fn self_alias_file_and_inline_normalized_scope_results_match() {
    let (inline_root, inline_graph) = parsed_graph(
        r#"
            pub mod api {
                pub(self) fn normalize(value: Int) -> Int { value }
                use self::normalize as normalize_here;
            }
        "#,
        "inline-normalized-result",
    );
    let (file_root, file_graph) = file_backed_graph(
        "pub mod api;",
        "api",
        r#"
            pub(self) fn normalize(value: Int) -> Int { value }
            use self::normalize as normalize_here;
        "#,
        "file-normalized-result",
    );
    let inline_module = module_key(&inline_root, &["api"]);
    let file_module = module_key(&file_root, &["api"]);
    let inline_scopes = scopes(&inline_graph);
    let file_scopes = scopes(&file_graph);
    assert_eq!(
        inline_scopes.normalized_scope_projection(),
        file_scopes.normalized_scope_projection(),
        "file and inline acquisition must derive the same normalized provisional scopes",
    );
    let inline_plan =
        resolve(&inline_graph, &inline_scopes).expect("the inline same-module alias resolves");
    let file_plan =
        resolve(&file_graph, &file_scopes).expect("the file-backed same-module alias resolves");
    let inline_bound = bind_scoped_self_ordinary_function_imports(&inline_graph, &inline_scopes)
        .expect("the inline result binds");
    let file_bound = bind_scoped_self_ordinary_function_imports(&file_graph, &file_scopes)
        .expect("the file-backed result binds");
    let inline_binding = inline_bound
        .binding(&inline_module, "normalize_here")
        .expect("the inline projection retains the alias");
    let file_binding = file_bound
        .binding(&file_module, "normalize_here")
        .expect("the file projection retains the alias");

    assert_eq!(
        inline_plan.binding(&inline_module, "normalize_here"),
        Some(inline_binding)
    );
    assert_eq!(
        file_plan.binding(&file_module, "normalize_here"),
        Some(file_binding)
    );
    assert_ne!(
        inline_binding.origin(),
        file_binding.origin(),
        "normalization deliberately ignores file-versus-inline acquisition provenance",
    );
    let normalized_inline = (
        inline_binding.local_alias(),
        inline_binding.defining_identity().module_key(),
        inline_binding.defining_identity().name(),
        inline_binding.visibility(),
    );
    let normalized_file = (
        file_binding.local_alias(),
        file_binding.defining_identity().module_key(),
        file_binding.defining_identity().name(),
        file_binding.visibility(),
    );
    assert_eq!(normalized_inline, normalized_file);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn self_alias_generated_names_aliases_source_forms_and_visibility(
        name_suffix in "[a-z][a-z0-9_]{0,8}",
        alias_suffix in "[a-z][a-z0-9_]{0,8}",
        nested in any::<bool>(),
        file_backed in any::<bool>(),
        visibility_case in 0_u8..6,
        alias_count in 1_usize..=3,
    ) {
        let module_name = format!("scope_{name_suffix}");
        let (visibility_source, expected_visibility) = match visibility_case {
            0 => ("pub".to_owned(), Visibility::Public),
            1 => ("pub(crate)".to_owned(), Visibility::Crate),
            2 if nested => ("pub(super)".to_owned(), Visibility::Super { levels: 1 }),
            2 => ("pub(crate)".to_owned(), Visibility::Crate),
            3 if nested => (
                format!("pub(in crate::{module_name})"),
                Visibility::Restricted {
                    path: format!("crate::{module_name}").into(),
                },
            ),
            3 => (
                "pub(in crate)".to_owned(),
                Visibility::Restricted { path: "crate".into() },
            ),
            4 => ("pub(self)".to_owned(), Visibility::Self_),
            _ => (String::new(), Visibility::Inherited),
        };
        let generated_aliases = (0..alias_count)
            .map(|index| {
                (
                    format!("target_{name_suffix}_{index}"),
                    format!("alias_{alias_suffix}_{index}"),
                )
            })
            .collect::<Vec<_>>();
        let module_source = generated_aliases
            .iter()
            .map(|(function_name, alias)| {
                format!(
                    "{visibility_source} fn {function_name}() -> Int {{ 1 }} use self::{function_name} as {alias};"
                )
            })
            .collect::<Vec<_>>()
            .join(" ");

        let (root_key, graph, importing_segments) = if nested && file_backed {
            let (root_key, graph) = file_backed_graph(
                &format!("pub mod {module_name};"),
                &module_name,
                &module_source,
                "generated-file-nested-self-alias",
            );
            (root_key, graph, vec![module_name.as_str()])
        } else if nested {
            let (root_key, graph) = parsed_graph(
                &format!("pub mod {module_name} {{ {module_source} }}"),
                "generated-inline-nested-self-alias",
            );
            (root_key, graph, vec![module_name.as_str()])
        } else if file_backed {
            let auxiliary = format!("aux_{name_suffix}");
            let (root_key, graph) = file_backed_graph(
                &format!("{module_source} pub mod {auxiliary};"),
                &auxiliary,
                "",
                "generated-file-root-self-alias",
            );
            (root_key, graph, Vec::new())
        } else {
            let auxiliary = format!("aux_{name_suffix}");
            let (root_key, graph) = parsed_graph(
                &format!("{module_source} pub mod {auxiliary} {{}}"),
                "generated-inline-root-self-alias",
            );
            (root_key, graph, Vec::new())
        };
        let importing_module = module_key(&root_key, &importing_segments);
        let scope_snapshot = scopes(&graph);
        let plan = resolve(&graph, &scope_snapshot)
            .expect("generated direct self alias is inside its same-module visibility region");
        let bound = bind_scoped_self_ordinary_function_imports(&graph, &scope_snapshot)
            .expect("generated resolver result projects to the dedicated bound set");
        for (function_name, alias) in &generated_aliases {
            let plan_binding = plan
                .binding(&importing_module, alias)
                .expect("generated resolver result retains every distinct alias");
            let bound_binding = bound
                .binding(&importing_module, alias)
                .expect("generated bound result retains every distinct alias");

            prop_assert_eq!(bound_binding, plan_binding);
            prop_assert_eq!(plan_binding.local_alias(), alias.as_str());
            prop_assert_eq!(plan_binding.defining_identity().module_key(), &importing_module);
            prop_assert_eq!(plan_binding.defining_identity().name(), function_name.as_str());
            prop_assert_eq!(plan_binding.visibility(), &expected_visibility);
        }
    }
}

#[test]
fn self_alias_has_dedicated_authority_only() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let planner_source =
        fs::read_to_string(manifest_dir.join("src/canonical_simple_import_planner.rs"))
            .expect("read the self-alias planner authority boundary");
    let structural_binder_source =
        fs::read_to_string(manifest_dir.join("src/canonical_structural_module_binder.rs"))
            .expect("read the dedicated structural binder boundary");
    let generic_binder_source =
        fs::read_to_string(manifest_dir.join("src/canonical_module_binder.rs"))
            .expect("read the unchanged generic binder boundary");
    let lib_source = fs::read_to_string(manifest_dir.join("src/lib.rs"))
        .expect("read the type-checker public export boundary");

    let generic_binder_checksum = format!("{:x}", Sha256::digest(generic_binder_source.as_bytes()));
    assert_eq!(
        generic_binder_checksum, "aea47f6aae83b4b7e3bfaa9ee9a7561d76b12a5f82b50c42d3fd2d98e23ed8b6",
        "TASK-2070 must leave the generic compatibility binder byte-for-byte unchanged",
    );
    for forbidden_identifier in [
        "CanonicalSelfOrdinaryFunctionAliasBinding",
        "CanonicalResolvedSelfOrdinaryFunctionAliases",
        "CanonicalBoundSelfOrdinaryFunctionAliasSet",
        "resolve_scoped_self_ordinary_function_imports_with_scopes",
        "bind_scoped_self_ordinary_function_imports",
    ] {
        assert!(
            !contains_exact_identifier(&generic_binder_source, forbidden_identifier),
            "the generic binder must remain non-authorizing and omit {forbidden_identifier}",
        );
    }

    let legacy_binding_start = planner_source
        .find("pub struct CanonicalBoundModuleBinding")
        .expect("the established generic binding carrier remains present");
    let legacy_binding_end = planner_source[legacy_binding_start..]
        .find("/// One resolved cross-module simple import dependency.")
        .map(|offset| legacy_binding_start + offset)
        .expect("the established generic binding carrier retains its source boundary");
    let legacy_binding_source = &planner_source[legacy_binding_start..legacy_binding_end];
    for forbidden_identifier in ["local_alias", "use_span", "into_bound_alias_set"] {
        assert!(
            !contains_exact_identifier(legacy_binding_source, forbidden_identifier),
            "CanonicalBoundModuleBinding must remain unchanged and omit {forbidden_identifier}",
        );
    }

    for required_identifier in [
        "CanonicalSelfOrdinaryFunctionAliasBinding",
        "CanonicalResolvedSelfOrdinaryFunctionAliases",
        "CanonicalBoundSelfOrdinaryFunctionAliasSet",
        "resolve_scoped_self_ordinary_function_imports_with_scopes",
    ] {
        assert!(
            contains_exact_identifier(&planner_source, required_identifier),
            "the planner must contain the dedicated self-alias contract: {required_identifier}",
        );
        assert!(
            contains_exact_identifier(&lib_source, required_identifier),
            "lib.rs must export the dedicated self-alias contract: {required_identifier}",
        );
    }
    assert!(
        planner_source.contains("pub(crate) fn into_bound_alias_set"),
        "the resolved-to-bound conversion remains crate-private",
    );
    let resolved_result_source = braced_item_source(
        &planner_source,
        "pub struct CanonicalResolvedSelfOrdinaryFunctionAliases",
    );
    assert!(
        !contains_exact_identifier(resolved_result_source, "import_edges"),
        "the dedicated resolved result structurally has no import-edge field",
    );
    assert_eq!(
        planner_source.matches(".into_bound_alias_set(").count(),
        0,
        "the planner defines the private conversion but never invokes it",
    );
    assert_eq!(
        structural_binder_source
            .matches(".into_bound_alias_set(")
            .count(),
        1,
        "only the dedicated structural binder calls the private conversion",
    );
    assert!(
        structural_binder_source.contains("bind_scoped_self_ordinary_function_imports"),
        "only the dedicated structural binder exposes the binding projection",
    );
    assert!(
        structural_binder_source.contains(".map(|plan| plan.into_bound_alias_set())"),
        "the dedicated binder only projects the successful dedicated resolver result",
    );
    assert!(
        lib_source.contains("bind_scoped_self_ordinary_function_imports"),
        "lib.rs exports the dedicated binder without changing the generic route",
    );
}
