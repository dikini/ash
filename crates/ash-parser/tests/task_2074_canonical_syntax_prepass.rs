//! TASK-2074 syntax-only dependency prepass contract evidence.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::ModuleKey;
use ash_parser::module::ModuleBody;
use ash_parser::surface::{Definition, ExpansionError, Expr, SurfaceOrigin, visit_expr};
use ash_parser::{
    CanonicalExpandedModuleGraph, CanonicalModuleExpansionError, CanonicalModuleGraph,
    CanonicalModuleGraphResolver, CanonicalSyntaxDependencyCycle, CanonicalSyntaxImportFailure,
    CanonicalSyntaxImportFailureKind, Span,
};
use proptest::prelude::*;

static NEXT_TEMP_TREE: AtomicUsize = AtomicUsize::new(0);

struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(label: &str) -> Self {
        let serial = NEXT_TEMP_TREE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "ash-task-2074-syntax-{label}-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create TASK-2074 syntax fixture directory");
        Self { root }
    }

    fn write(&self, relative: impl AsRef<Path>, source: &str) -> PathBuf {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("fixture path has a parent"))
            .expect("create TASK-2074 syntax fixture parent");
        fs::write(&path, source).expect("write TASK-2074 syntax fixture");
        path
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Acquires all source through the canonical resolver, then drops the source
/// tree before returning. Expansion therefore cannot reread source or paths.
fn resolve_graph(source: &str, label: &str) -> (CanonicalModuleGraph, ModuleKey) {
    let tree = TempTree::new(label);
    let root_path = tree.write("src/main.ash", source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("fixture builds one canonical parsed graph");
    (graph, root_key)
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

fn first_macro_span(body: &ModuleBody) -> Span {
    body.definitions()
        .iter()
        .find_map(|definition| match definition {
            Definition::Macro(definition) => Some(definition.span),
            _ => None,
        })
        .expect("fixture contains a macro declaration")
}

fn first_function_span(body: &ModuleBody) -> Span {
    body.definitions()
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(definition) => Some(definition.span),
            _ => None,
        })
        .expect("fixture contains a function declaration")
}

fn first_invocation_span(body: &ModuleBody) -> Span {
    let mut invocation_span = None;
    for definition in body.definitions() {
        let Definition::Function(function) = definition else {
            continue;
        };
        visit_expr(&function.body, &mut |expr| {
            if let Expr::MacroInvocation { invocation } = expr {
                invocation_span.get_or_insert(invocation.span);
            }
        });
    }
    invocation_span.expect("fixture contains a macro invocation")
}

fn first_operator_section_span(body: &ModuleBody) -> Span {
    let mut section_span = None;
    for definition in body.definitions() {
        let Definition::Function(function) = definition else {
            continue;
        };
        visit_expr(&function.body, &mut |expr| {
            if let Expr::OperatorSection { section } = expr {
                section_span.get_or_insert(section.span);
            }
        });
    }
    section_span.expect("fixture contains an operator section")
}

fn first_use_span(graph: &CanonicalModuleGraph, key: &ModuleKey) -> Span {
    graph
        .module_unit(key)
        .expect("fixture module exists")
        .body()
        .uses()
        .first()
        .expect("fixture module contains a use")
        .span
}

fn assert_syntax_import_failure(
    error: &CanonicalModuleExpansionError,
    kind: CanonicalSyntaxImportFailureKind,
    consumer_key: &ModuleKey,
    provider_key: Option<&ModuleKey>,
    use_span: Span,
    declaration_span: Option<Span>,
) {
    assert!(matches!(
        error,
        CanonicalModuleExpansionError::InvalidSyntaxImport { .. }
    ));
    let failure: &CanonicalSyntaxImportFailure = error
        .syntax_import_failure()
        .expect("invalid syntax import exposes its anchored failure");
    assert_eq!(failure.kind(), kind);
    assert_eq!(failure.consumer_key(), consumer_key);
    assert_eq!(failure.provider_key(), provider_key);
    assert_eq!(failure.use_span(), use_span);
    assert_eq!(failure.declaration_span(), declaration_span);
}

fn assert_cycle(
    error: &CanonicalModuleExpansionError,
    expected: &[(&ModuleKey, &ModuleKey, Span)],
) {
    assert!(matches!(
        error,
        CanonicalModuleExpansionError::SyntaxDependencyCycle { .. }
    ));
    let cycle: &CanonicalSyntaxDependencyCycle = error
        .syntax_dependency_cycle()
        .expect("syntax cycle variant exposes its stable ordered edges");
    let actual = cycle
        .edges()
        .iter()
        .map(|edge| (edge.importer_key(), edge.provider_key(), edge.use_span()))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

#[test]
fn local_public_macro_summary_is_available_in_its_own_module() {
    let (parsed, root_key) = resolve_graph(
        r#"
            pub macro inc(x) => add(x, 1);
            fn direct(n: Int) -> Int { inc!(n) }
        "#,
        "local-public",
    );

    let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect("a public macro remains available in its defining module");
    let root = expanded.module(&root_key).expect("expanded root exists");
    assert!(!contains_macro_invocation(root.body()));
}

#[test]
fn canonical_use_alias_resolves_a_public_macro_summary() {
    let (parsed, root_key) = resolve_graph(
        r#"
            mod a_provider {
                pub macro inc(x) => add(x, 1);
            }
            mod z_consumer {
                use crate::a_provider::inc as plus_one;
                fn run(n: Int) -> Int { plus_one!(n) }
            }
        "#,
        "public-alias",
    );
    let consumer_key = root_key.child("z_consumer").expect("consumer key");

    let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect("canonical alias imports the provider's public macro summary");
    let consumer = expanded
        .module(&consumer_key)
        .expect("expanded consumer exists");
    assert!(!contains_macro_invocation(consumer.body()));
    assert!(consumer.origins().iter().any(|origin| matches!(
        origin.origin,
        SurfaceOrigin::MacroExpansion { ref expansion_id, .. } if expansion_id.as_ref() == "inc"
    )));
}

#[test]
fn provider_expands_before_lexically_earlier_consumer() {
    let (parsed, root_key) = resolve_graph(
        r#"
            mod a_consumer {
                use crate::z_provider::inc;
                fn run(n: Int) -> Int { inc!(n) }
            }
            mod z_provider {
                pub macro inc(x) => add(x, 1);
            }
        "#,
        "provider-before-consumer",
    );
    let consumer_key = root_key.child("a_consumer").expect("consumer key");

    let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect("syntax dependencies, not canonical key or source order, expand providers first");
    assert!(!contains_macro_invocation(
        expanded
            .module(&consumer_key)
            .expect("expanded consumer exists")
            .body()
    ));
}

#[test]
fn private_macro_import_rejects_at_declaration_and_use_anchors() {
    let (parsed, root_key) = resolve_graph(
        r#"
            mod provider { macro hidden(x) => x; }
            mod consumer {
                use crate::provider::hidden;
                fn run(n: Int) -> Int { hidden!(n) }
            }
        "#,
        "private-macro",
    );
    let provider_key = root_key.child("provider").expect("provider key");
    let consumer_key = root_key.child("consumer").expect("consumer key");
    let use_span = first_use_span(&parsed, &consumer_key);
    let declaration_span = first_macro_span(
        parsed
            .module_unit(&provider_key)
            .expect("provider exists")
            .body(),
    );

    let error = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect_err("private macro summary is not importable");
    assert_syntax_import_failure(
        &error,
        CanonicalSyntaxImportFailureKind::PrivateMacro,
        &consumer_key,
        Some(&provider_key),
        use_span,
        Some(declaration_span),
    );
}

#[test]
fn non_macro_declaration_cannot_supply_a_syntax_summary() {
    let (parsed, root_key) = resolve_graph(
        r#"
            mod provider { pub fn helper(n: Int) -> Int { n } }
            mod consumer {
                use crate::provider::helper;
                fn run(n: Int) -> Int { helper!(n) }
            }
        "#,
        "non-macro-summary",
    );
    let provider_key = root_key.child("provider").expect("provider key");
    let consumer_key = root_key.child("consumer").expect("consumer key");
    let use_span = first_use_span(&parsed, &consumer_key);
    let declaration_span = first_function_span(
        parsed
            .module_unit(&provider_key)
            .expect("provider exists")
            .body(),
    );

    let error = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect_err("ordinary declarations cannot masquerade as macro summaries");
    assert_syntax_import_failure(
        &error,
        CanonicalSyntaxImportFailureKind::NonMacroDeclaration,
        &consumer_key,
        Some(&provider_key),
        use_span,
        Some(declaration_span),
    );
}

#[test]
fn missing_public_macro_summary_rejects_at_the_use_anchor() {
    let (parsed, root_key) = resolve_graph(
        r#"
            mod provider { pub fn other() {} }
            mod consumer {
                use crate::provider::missing;
                fn run(n: Int) -> Int { missing!(n) }
            }
        "#,
        "missing-summary",
    );
    let provider_key = root_key.child("provider").expect("provider key");
    let consumer_key = root_key.child("consumer").expect("consumer key");
    let use_span = first_use_span(&parsed, &consumer_key);

    let error = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect_err("missing syntax summary rejects before local expansion");
    assert_syntax_import_failure(
        &error,
        CanonicalSyntaxImportFailureKind::MissingSummary,
        &consumer_key,
        Some(&provider_key),
        use_span,
        None,
    );
}

#[test]
fn two_module_syntax_cycle_reports_stable_key_and_use_edges() {
    let (parsed, root_key) = resolve_graph(
        r#"
            mod a {
                use crate::b::b_macro;
                pub macro a_macro(x) => b_macro!(x);
            }
            mod b {
                use crate::a::a_macro;
                pub macro b_macro(x) => a_macro!(x);
            }
        "#,
        "two-cycle",
    );
    let a = root_key.child("a").expect("a key");
    let b = root_key.child("b").expect("b key");
    let a_use = first_use_span(&parsed, &a);
    let b_use = first_use_span(&parsed, &b);

    let error = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect_err("two-module syntax dependency cycle rejects atomically");
    assert_cycle(&error, &[(&a, &b, a_use), (&b, &a, b_use)]);
}

#[test]
fn three_module_syntax_cycle_reports_stable_key_and_use_edges() {
    let (parsed, root_key) = resolve_graph(
        r#"
            mod a {
                use crate::b::b_macro;
                pub macro a_macro(x) => b_macro!(x);
            }
            mod b {
                use crate::c::c_macro;
                pub macro b_macro(x) => c_macro!(x);
            }
            mod c {
                use crate::a::a_macro;
                pub macro c_macro(x) => a_macro!(x);
            }
        "#,
        "three-cycle",
    );
    let a = root_key.child("a").expect("a key");
    let b = root_key.child("b").expect("b key");
    let c = root_key.child("c").expect("c key");
    let a_use = first_use_span(&parsed, &a);
    let b_use = first_use_span(&parsed, &b);
    let c_use = first_use_span(&parsed, &c);

    let error = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect_err("three-module syntax dependency cycle rejects atomically");
    assert_cycle(&error, &[(&a, &b, a_use), (&b, &c, b_use), (&c, &a, c_use)]);
}

#[test]
fn public_provider_notation_remains_inactive_without_a_canonical_summary() {
    let (parsed, root_key) = resolve_graph(
        r#"
            mod provider {
                pub fn combine(a: Int, b: Int) -> Int { a + b }
                pub infixl 6 <+> = combine
            }
            mod consumer {
                use crate::provider::combine;
                fn run(n: Int) -> Int { (n <+>) }
            }
        "#,
        "notation-no-summary",
    );
    let consumer_key = root_key.child("consumer").expect("consumer key");
    let consumer_body = parsed
        .module_unit(&consumer_key)
        .expect("consumer exists")
        .body();
    let expected_span = first_operator_section_span(consumer_body);

    let error = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect_err("a public notation declaration cannot leak without a canonical summary");
    let failure = error
        .expansion_failure()
        .expect("unresolved imported notation remains a local expansion failure");
    assert_eq!(failure.module_key(), &consumer_key);
    assert!(matches!(
        failure.expansion_error(),
        ExpansionError::UnresolvedOperatorSection { span, operator }
            if *span == expected_span && operator.as_ref() == "<+>"
    ));
}

#[test]
fn item_generating_macro_attempt_rejects_without_publishing_generated_items() {
    let (parsed, root_key) = resolve_graph(
        r#"
            mod provider { pub macro passthrough(x) => x; }
            mod consumer {
                use crate::provider::passthrough;
                fn run() { passthrough!{fn generated() {}} }
            }
        "#,
        "item-generating-attempt",
    );
    let consumer_key = root_key.child("consumer").expect("consumer key");
    let expected_span = first_invocation_span(
        parsed
            .module_unit(&consumer_key)
            .expect("consumer exists")
            .body(),
    );

    let error = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect_err("item-shaped token-tree macro output remains outside the target domain");
    let failure = error
        .expansion_failure()
        .expect("unsupported item-generating attempt is anchored to its consumer module");
    assert_eq!(failure.module_key(), &consumer_key);
    assert!(matches!(
        failure.expansion_error(),
        ExpansionError::MacroTokenTreeReparseFailed {
            span,
            name,
            ..
        }
            if *span == expected_span && name.as_ref() == "passthrough"
    ));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn canonical_public_macro_aliases_expand_independently_of_key_order(
        alias_index in 0u8..16,
        consumer_sorts_first in any::<bool>(),
    ) {
        let alias = format!("imported_{alias_index}");
        let (provider_name, consumer_name) = if consumer_sorts_first {
            ("z_provider", "a_consumer")
        } else {
            ("a_provider", "z_consumer")
        };
        let source = format!(
            "mod {consumer_name} {{\n\
                 use crate::{provider_name}::inc as {alias};\n\
                 fn run(n: Int) -> Int {{ {alias}!(n) }}\n\
             }}\n\
             mod {provider_name} {{ pub macro inc(x) => add(x, 1); }}\n"
        );
        let (parsed, root_key) = resolve_graph(&source, "generated-alias-order");
        let consumer_key = root_key.child(consumer_name).expect("generated consumer key");

        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("every canonical public macro alias expands after dependency ordering");
        let consumer = expanded
            .module(&consumer_key)
            .expect("generated consumer exists");

        prop_assert!(!contains_macro_invocation(consumer.body()));
        prop_assert_eq!(consumer.origins().len(), 1);
    }
}
