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
    CanonicalSyntaxImportFailureKind, CanonicalSyntaxProviderFailure, Span,
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
        let expression = match definition {
            Definition::Function(function) => &function.body,
            Definition::Macro(macro_definition) => &macro_definition.body,
            _ => return false,
        };
        let mut found = false;
        visit_expr(expression, &mut |expr| {
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
    use_span_at(graph, key, 0)
}

fn use_span_at(graph: &CanonicalModuleGraph, key: &ModuleKey, index: usize) -> Span {
    graph
        .module_unit(key)
        .expect("fixture module exists")
        .body()
        .uses()
        .get(index)
        .expect("fixture module contains a use")
        .span
}

fn child_declaration_span(
    graph: &CanonicalModuleGraph,
    parent_key: &ModuleKey,
    child_name: &str,
) -> Span {
    graph
        .module_unit(parent_key)
        .expect("fixture parent exists")
        .body()
        .module_decls()
        .iter()
        .find(|declaration| declaration.name.as_ref() == child_name)
        .expect("fixture child declaration exists")
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
            pub mod a_provider {
                pub macro inc(x) => add(x, 1);
            }
            pub mod z_consumer {
                use crate::a_provider::inc as plus_one;
                fn run(n: Int) -> Int { plus_one!(n) }
            }
        "#,
        "public-alias",
    );
    let provider_key = root_key.child("a_provider").expect("provider key");
    let consumer_key = root_key.child("z_consumer").expect("consumer key");
    let use_span = first_use_span(&parsed, &consumer_key);
    let provider_declaration_span = first_macro_span(
        parsed
            .module_unit(&provider_key)
            .expect("provider exists")
            .body(),
    );

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
    let imports = consumer.syntax_imports();
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].provider_key(), &provider_key);
    assert_eq!(imports[0].exported_name(), "inc");
    assert_eq!(imports[0].local_name(), "plus_one");
    assert_eq!(
        imports[0].provider_declaration_span(),
        provider_declaration_span
    );
    assert_eq!(imports[0].use_span(), use_span);
}

#[test]
fn provider_expands_before_lexically_earlier_consumer() {
    let (parsed, root_key) = resolve_graph(
        r#"
            pub mod a_consumer {
                use crate::z_provider::inc;
                fn run(n: Int) -> Int { inc!(n) }
            }
            pub mod z_provider {
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
fn three_module_transitive_macro_chain_consumes_provider_output_and_closure() {
    let (parsed, root_key) = resolve_graph(
        r#"
            pub mod a_consumer {
                use crate::m_middle::middle;
                fn run(n: Int) -> Int { middle!(n) }
            }
            pub mod m_middle {
                use crate::z_base::base;
                pub macro middle(x) => base!(x);
            }
            pub mod z_base {
                pub macro base(x) => add(x, 1);
            }
        "#,
        "transitive-provider-closure",
    );
    let consumer_key = root_key.child("a_consumer").expect("consumer key");
    let middle_key = root_key.child("m_middle").expect("middle key");
    let base_key = root_key.child("z_base").expect("base key");

    let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect("base output feeds the middle summary before the consumer expands");
    let middle = expanded
        .module(&middle_key)
        .expect("expanded middle exists");
    let consumer = expanded
        .module(&consumer_key)
        .expect("expanded consumer exists");

    assert!(
        !contains_macro_invocation(middle.body()),
        "the middle provider must consume its imported base macro before publishing output"
    );
    assert!(
        !contains_macro_invocation(consumer.body()),
        "the consumer must receive fully closed middle-provider output"
    );
    assert!(middle.origins().iter().any(|origin| matches!(
        origin.origin,
        SurfaceOrigin::MacroExpansion { ref expansion_id, .. } if expansion_id.as_ref() == "base"
    )));
    assert!(consumer.origins().iter().any(|origin| matches!(
        origin.origin,
        SurfaceOrigin::MacroExpansion { ref expansion_id, .. } if expansion_id.as_ref() == "middle"
    )));
    assert_eq!(middle.syntax_imports()[0].provider_key(), &base_key);
    assert_eq!(consumer.syntax_imports()[0].provider_key(), &middle_key);
}

#[test]
fn private_macro_import_rejects_at_declaration_and_use_anchors() {
    let (parsed, root_key) = resolve_graph(
        r#"
            pub mod provider { macro hidden(x) => x; }
            pub mod consumer {
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
fn private_structural_provider_path_rejects_at_module_declaration_and_use() {
    let (parsed, root_key) = resolve_graph(
        r#"
            mod provider { pub macro visible_macro(x) => x; }
            pub mod consumer {
                use crate::provider::visible_macro;
                fn run(n: Int) -> Int { visible_macro!(n) }
            }
        "#,
        "private-provider-path",
    );
    let provider_key = root_key.child("provider").expect("provider key");
    let consumer_key = root_key.child("consumer").expect("consumer key");
    let use_span = first_use_span(&parsed, &consumer_key);
    let private_module_span = child_declaration_span(&parsed, &root_key, "provider");

    let error = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect_err("a public macro behind a private structural path is not importable");
    assert_syntax_import_failure(
        &error,
        CanonicalSyntaxImportFailureKind::PrivateModulePath,
        &consumer_key,
        Some(&provider_key),
        use_span,
        Some(private_module_span),
    );
}

#[test]
fn non_macro_declaration_cannot_supply_a_syntax_summary() {
    let (parsed, root_key) = resolve_graph(
        r#"
            pub mod provider { pub fn helper(n: Int) -> Int { n } }
            pub mod consumer {
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
            pub mod provider { pub fn other() {} }
            pub mod consumer {
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
fn syntax_sidecars_distinguish_two_providers_exporting_the_same_spelling() {
    let (parsed, root_key) = resolve_graph(
        r#"
            pub mod a_provider { pub macro inc(x) => add(x, 1); }
            pub mod b_provider { pub macro inc(x) => add(x, 2); }
            pub mod consumer {
                use crate::a_provider::inc as first_inc;
                use crate::b_provider::inc as second_inc;
                fn run(n: Int) -> Int { add(first_inc!(n), second_inc!(n)) }
            }
        "#,
        "same-spelling-providers",
    );
    let a_provider = root_key.child("a_provider").expect("a provider key");
    let b_provider = root_key.child("b_provider").expect("b provider key");
    let consumer_key = root_key.child("consumer").expect("consumer key");
    let first_use = use_span_at(&parsed, &consumer_key, 0);
    let second_use = use_span_at(&parsed, &consumer_key, 1);

    let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect("canonical provider identity disambiguates equal exported spellings");
    let consumer = expanded
        .module(&consumer_key)
        .expect("expanded consumer exists");
    assert!(!contains_macro_invocation(consumer.body()));

    let imports = consumer.syntax_imports();
    assert_eq!(imports.len(), 2);
    assert_eq!(imports[0].provider_key(), &a_provider);
    assert_eq!(imports[0].exported_name(), "inc");
    assert_eq!(imports[0].local_name(), "first_inc");
    assert_eq!(imports[0].use_span(), first_use);
    assert_eq!(imports[1].provider_key(), &b_provider);
    assert_eq!(imports[1].exported_name(), "inc");
    assert_eq!(imports[1].local_name(), "second_inc");
    assert_eq!(imports[1].use_span(), second_use);
}

#[test]
fn macro_context_selects_macro_when_same_named_ordinary_declaration_precedes_it() {
    let (parsed, root_key) = resolve_graph(
        r#"
            pub mod provider {
                pub fn id(x: Int) -> Int { x }
                pub macro id(x) => add(x, 1);
            }
            pub mod consumer {
                use crate::provider::id as syntax_id;
                fn run(n: Int) -> Int { syntax_id!(n) }
            }
        "#,
        "macro-namespace-selection",
    );
    let provider_key = root_key.child("provider").expect("provider key");
    let consumer_key = root_key.child("consumer").expect("consumer key");

    let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect("macro invocation context selects the provider's macro namespace");
    let consumer = expanded
        .module(&consumer_key)
        .expect("expanded consumer exists");
    assert!(!contains_macro_invocation(consumer.body()));
    assert_eq!(consumer.syntax_imports().len(), 1);
    assert_eq!(consumer.syntax_imports()[0].provider_key(), &provider_key);
    assert_eq!(consumer.syntax_imports()[0].exported_name(), "id");
}

#[test]
fn duplicate_imported_macro_alias_rejects_at_the_second_use() {
    let (parsed, root_key) = resolve_graph(
        r#"
            pub mod a_provider { pub macro inc(x) => add(x, 1); }
            pub mod b_provider { pub macro bump(x) => add(x, 2); }
            pub mod consumer {
                use crate::a_provider::inc as same;
                use crate::b_provider::bump as same;
                fn run(n: Int) -> Int { same!(n) }
            }
        "#,
        "duplicate-import-alias",
    );
    let b_provider = root_key.child("b_provider").expect("b provider key");
    let consumer_key = root_key.child("consumer").expect("consumer key");
    let second_use = use_span_at(&parsed, &consumer_key, 1);
    let second_declaration = first_macro_span(
        parsed
            .module_unit(&b_provider)
            .expect("b provider exists")
            .body(),
    );

    let error = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect_err("duplicate imported local macro alias rejects in the syntax prepass");
    assert_syntax_import_failure(
        &error,
        CanonicalSyntaxImportFailureKind::DuplicateLocalName,
        &consumer_key,
        Some(&b_provider),
        second_use,
        Some(second_declaration),
    );
}

#[test]
fn malformed_public_template_reports_provider_context_never_consumer_context() {
    let (parsed, root_key) = resolve_graph(
        r#"
            pub mod provider { pub macro bad(x) => free_name; }
            pub mod consumer {
                use crate::provider::bad;
                fn run(n: Int) -> Int { bad!(n) }
            }
        "#,
        "malformed-provider-template",
    );
    let provider_key = root_key.child("provider").expect("provider key");
    let consumer_key = root_key.child("consumer").expect("consumer key");
    let provider_unit = parsed
        .module_unit(&provider_key)
        .expect("provider unit exists");
    let expected_source_path = provider_unit.source_path().map(str::to_owned);
    let expected_artifact_origin = provider_unit.artifact().origin().clone();
    let provider_declaration_span = first_macro_span(provider_unit.body());

    let error = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect_err("malformed public macro template rejects while collecting provider output");
    assert!(matches!(
        &error,
        CanonicalModuleExpansionError::InvalidSyntaxProvider { .. }
    ));
    let failure: &CanonicalSyntaxProviderFailure = error
        .syntax_provider_failure()
        .expect("invalid provider exposes its anchored syntax failure");
    assert_eq!(failure.provider_key(), &provider_key);
    assert_ne!(failure.provider_key(), &consumer_key);
    assert_eq!(failure.source_path(), expected_source_path.as_deref());
    assert_eq!(failure.artifact_origin(), &expected_artifact_origin);
    assert_eq!(failure.declaration_span(), provider_declaration_span);
    assert!(matches!(
        failure.expansion_error(),
        ExpansionError::UnsupportedMacroTemplate { name, reason, .. }
            if name.as_ref() == "bad" && reason.as_ref() == "free variable"
    ));
}

#[test]
fn two_module_syntax_cycle_reports_stable_key_and_use_edges() {
    let (parsed, root_key) = resolve_graph(
        r#"
            pub mod a {
                use crate::b::b_macro;
                pub macro a_macro(x) => b_macro!(x);
            }
            pub mod b {
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
            pub mod a {
                use crate::b::b_macro;
                pub macro a_macro(x) => b_macro!(x);
            }
            pub mod b {
                use crate::c::c_macro;
                pub macro b_macro(x) => c_macro!(x);
            }
            pub mod c {
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
            pub mod provider {
                pub fn combine(a: Int, b: Int) -> Int { a + b }
                pub infixl 6 <+> = combine
            }
            pub mod consumer {
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
            pub mod provider { pub macro passthrough(x) => x; }
            pub mod consumer {
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
            "pub mod {consumer_name} {{\n\
                 use crate::{provider_name}::inc as {alias};\n\
                 fn run(n: Int) -> Int {{ {alias}!(n) }}\n\
             }}\n\
             pub mod {provider_name} {{ pub macro inc(x) => add(x, 1); }}\n"
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
