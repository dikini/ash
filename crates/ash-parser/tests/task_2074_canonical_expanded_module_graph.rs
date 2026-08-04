//! TASK-2074 RED evidence for the canonical expanded module graph.
//!
//! This first slice specifies the parser-owned, shallow expansion boundary. It
//! deliberately does not specify the later syntax-dependency prepass.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::ModuleKey;
use ash_parser::module::{ModuleBody, ModuleItem, ModuleSource};
use ash_parser::surface::{Definition, Expr, visit_expr};
use ash_parser::{
    CanonicalExpandedModuleGraph, CanonicalModuleExpansionError, CanonicalModuleGraph,
    CanonicalModuleGraphResolver,
};
use proptest::prelude::*;

static NEXT_TEMP_TREE: AtomicUsize = AtomicUsize::new(0);

/// A real source tree removed automatically after each test.
struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(label: &str) -> Self {
        let serial = NEXT_TEMP_TREE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "ash-task-2074-{label}-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create TASK-2074 fixture directory");
        Self { root }
    }

    fn write(&self, relative: impl AsRef<Path>, source: &str) -> PathBuf {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("fixture path has a parent"))
            .expect("create TASK-2074 fixture parent");
        fs::write(&path, source).expect("write TASK-2074 fixture");
        path
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn resolve_graph(source: &str, label: &str) -> (CanonicalModuleGraph, ModuleKey) {
    let tree = TempTree::new(label);
    let root_path = tree.write("src/main.ash", source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("fixture builds one canonical parsed graph");
    (graph, root_key)
}

fn try_expand(
    parsed: CanonicalModuleGraph,
) -> Result<CanonicalExpandedModuleGraph, CanonicalModuleExpansionError> {
    CanonicalExpandedModuleGraph::try_expand(parsed)
}

fn item_labels(body: &ModuleBody) -> Vec<String> {
    body.items()
        .iter()
        .map(|item| match item {
            ModuleItem::Use(_) => "use".to_owned(),
            ModuleItem::Definition(Definition::Macro(definition)) => {
                format!("macro:{}", definition.name)
            }
            ModuleItem::Definition(Definition::Function(definition)) => {
                format!("fn:{}", definition.name)
            }
            ModuleItem::Definition(_) => "definition".to_owned(),
            ModuleItem::ModuleDecl(declaration) => format!("mod:{}", declaration.name),
        })
        .collect()
}

fn contains_macro_invocation(body: &ModuleBody) -> bool {
    body.definitions().iter().any(|definition| {
        let Definition::Function(function) = definition else {
            return false;
        };
        let mut found = false;
        visit_expr(&function.body, &mut |expr| {
            found |= matches!(expr, Expr::MacroInvocation { .. });
        });
        found
    })
}

#[test]
fn shallow_expansion_preserves_parsed_use_and_complete_source_order() {
    let (parsed, root_key) = resolve_graph(
        r#"
            use crate::support::Thing;
            macro inc(x) => add(x, 1);
            fn direct(n: Int) -> Int { inc!(n) }
            mod child { fn untouched() {} }
        "#,
        "shallow-order",
    );
    let parsed_labels = item_labels(
        parsed
            .module_unit(&root_key)
            .expect("root parsed unit exists")
            .body(),
    );

    let expanded = try_expand(parsed).expect("supported direct macro expansion succeeds");
    let root = expanded
        .module(&root_key)
        .expect("expanded graph contains its parsed root key");

    assert_eq!(
        item_labels(root.body()),
        parsed_labels,
        "expansion must retain the parsed use, definitions, child declaration, and their order"
    );
    assert_eq!(
        item_labels(root.body()),
        ["use", "macro:inc", "fn:direct", "mod:child"],
        "the read-only expanded body must expose the complete ordered ModuleBody"
    );
    assert!(
        !contains_macro_invocation(root.body()),
        "the direct function owned by the root key must be expanded"
    );
}

#[test]
fn inline_child_expansion_sidecars_belong_only_to_the_child_key() {
    let (parsed, root_key) = resolve_graph(
        r#"
            mod child {
                macro inc(x) => add(x, 1);
                fn child_direct(n: Int) -> Int { inc!(n) }
            }
        "#,
        "inline-sidecars",
    );
    let child_key = root_key
        .child("child")
        .expect("fixture inline child key is canonical");

    let expanded = try_expand(parsed).expect("supported child macro expansion succeeds");
    let root = expanded.module(&root_key).expect("expanded root exists");
    let child = expanded.module(&child_key).expect("expanded child exists");

    assert!(
        root.origins().is_empty(),
        "shallow root expansion must not absorb the inline child's origin sidecars"
    );
    assert!(
        root.hygiene().is_empty(),
        "shallow root expansion must not absorb the inline child's hygiene sidecars"
    );
    assert!(
        !child.origins().is_empty(),
        "the child-owned macro expansion must retain its origin sidecar under the child key"
    );
    assert!(
        !child.hygiene().is_empty(),
        "the child-owned macro expansion must retain its hygiene sidecars under the child key"
    );
    assert!(
        !contains_macro_invocation(child.body()),
        "the child record must expand definitions directly owned by the child key"
    );

    let root_child = root
        .body()
        .module_decls()
        .first()
        .expect("root retains its parsed inline child declaration");
    let ModuleSource::Inline(raw_child_body) = &root_child.source else {
        panic!("fixture child remains an inline structural declaration")
    };
    assert!(
        contains_macro_invocation(raw_child_body.as_ref()),
        "root expansion must leave the nested declaration payload unchanged instead of recursively expanding it"
    );
}

#[test]
fn carrier_owns_the_exact_parsed_graph_and_publishes_all_keys_atomically() {
    let (parsed, root_key) = resolve_graph(
        r#"
            mod first { fn one() {} }
            mod second { fn two() {} }
        "#,
        "owned-atomic-carrier",
    );
    let first_key = root_key.child("first").expect("first child key");
    let second_key = root_key.child("second").expect("second child key");

    let expanded = try_expand(parsed).expect("an expansion without syntax failures succeeds");

    assert_eq!(expanded.parsed_graph().root_key(), &root_key);
    assert!(
        expanded.parsed_graph().module_unit(&first_key).is_some(),
        "the carrier must retain the consumed canonical parsed graph"
    );
    assert_eq!(
        expanded
            .modules()
            .map(|module| module.key().clone())
            .collect::<Vec<_>>(),
        [root_key, first_key, second_key],
        "the successful public value must contain exactly one ordered record per parsed key"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn shallow_expansion_retains_order_for_generated_direct_definitions(
        function_count in 1usize..=4,
    ) {
        let functions = (0..function_count)
            .map(|index| format!("fn direct_{index}(n: Int) -> Int {{ inc!(n) }}"))
            .collect::<Vec<_>>()
            .join("\n");
        let source = format!(
            "use crate::support::Thing;\n\
             macro inc(x) => add(x, 1);\n\
             {functions}\n\
             mod child {{ fn untouched() {{}} }}\n"
        );
        let (parsed, root_key) = resolve_graph(&source, "generated-shallow-order");
        let expected = item_labels(
            parsed
                .module_unit(&root_key)
                .expect("generated root unit exists")
                .body(),
        );

        let expanded = try_expand(parsed).expect("generated supported expansion succeeds");
        let root = expanded
            .module(&root_key)
            .expect("generated expanded root exists");

        prop_assert_eq!(item_labels(root.body()), expected);
        prop_assert!(!contains_macro_invocation(root.body()));
        prop_assert_eq!(root.origins().len(), function_count);
    }
}
