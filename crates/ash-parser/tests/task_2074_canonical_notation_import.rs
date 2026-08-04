//! TASK-2074 canonical public notation-summary transport evidence.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::{ModuleArtifactOrigin, ModuleKey};
use ash_parser::surface::{CallablePath, Definition, NotationAssociativity, Visibility};
use ash_parser::{
    CanonicalExpandedModuleGraph, CanonicalModuleExpansionError, CanonicalModuleGraph,
    CanonicalModuleGraphResolver, CanonicalNotationFixityKey, CanonicalNotationImportFailure,
    CanonicalNotationImportFailureKind, CanonicalNotationPatternPart,
    CanonicalSyntaxDependencyCycle, Span,
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
            "ash-task-2074-notation-{label}-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create TASK-2074 notation fixture directory");
        Self { root }
    }

    fn write(&self, relative: impl AsRef<Path>, source: &str) -> PathBuf {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("fixture path has a parent"))
            .expect("create TASK-2074 notation fixture parent");
        fs::write(&path, source).expect("write TASK-2074 notation fixture");
        path
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[derive(Debug, Clone)]
struct ExpectedDeclaration {
    target: CallablePath,
    span: Span,
}

#[derive(Debug, Clone)]
struct ExpectedModuleContext {
    source_path: Option<Box<str>>,
    artifact_origin: ModuleArtifactOrigin,
}

fn module_context(graph: &CanonicalModuleGraph, key: &ModuleKey) -> ExpectedModuleContext {
    let unit = graph.module_unit(key).expect("fixture module exists");
    ExpectedModuleContext {
        source_path: unit.source_path().map(Into::into),
        artifact_origin: unit.artifact().origin().clone(),
    }
}

fn use_span_at(graph: &CanonicalModuleGraph, key: &ModuleKey, index: usize) -> Span {
    graph
        .module_unit(key)
        .expect("fixture consumer exists")
        .body()
        .uses()
        .get(index)
        .expect("fixture consumer has the expected notation use")
        .span
}

fn notation_spans(graph: &CanonicalModuleGraph, key: &ModuleKey) -> Vec<Span> {
    graph
        .module_unit(key)
        .expect("fixture notation provider exists")
        .body()
        .definitions()
        .iter()
        .filter_map(|definition| match definition {
            Definition::Notation(declaration) => Some(declaration.span),
            _ => None,
        })
        .collect()
}

fn macro_span(graph: &CanonicalModuleGraph, key: &ModuleKey) -> Span {
    graph
        .module_unit(key)
        .expect("fixture macro provider exists")
        .body()
        .definitions()
        .iter()
        .find_map(|definition| match definition {
            Definition::Macro(declaration) => Some(declaration.span),
            _ => None,
        })
        .expect("fixture has a macro declaration")
}

fn child_declaration_span(
    graph: &CanonicalModuleGraph,
    parent_key: &ModuleKey,
    child_name: &str,
) -> Span {
    graph
        .module_unit(parent_key)
        .expect("fixture structural parent exists")
        .body()
        .module_decls()
        .iter()
        .find(|declaration| declaration.name.as_ref() == child_name)
        .expect("fixture has the expected child declaration")
        .span
}

#[allow(clippy::too_many_arguments)]
fn assert_notation_dependency_failure(
    error: &CanonicalModuleExpansionError,
    kind: CanonicalNotationImportFailureKind,
    consumer_key: &ModuleKey,
    consumer_context: &ExpectedModuleContext,
    provider_key: Option<&ModuleKey>,
    provider_context: Option<&ExpectedModuleContext>,
    use_span: Span,
    declaration_spans: &[Span],
) {
    let failure: &CanonicalNotationImportFailure = error
        .notation_import_failure()
        .expect("invalid notation dependency exposes one typed anchored failure");
    assert_eq!(failure.kind(), kind);
    assert_eq!(failure.consumer_key(), consumer_key);
    assert_eq!(
        failure.consumer_source_path(),
        consumer_context.source_path.as_deref()
    );
    assert_eq!(
        failure.consumer_artifact_origin(),
        &consumer_context.artifact_origin
    );
    assert_eq!(failure.provider_key(), provider_key);
    assert_eq!(
        failure.provider_source_path(),
        provider_context.and_then(|context| context.source_path.as_deref())
    );
    assert_eq!(
        failure.provider_artifact_origin(),
        provider_context.map(|context| &context.artifact_origin)
    );
    assert_eq!(failure.use_span(), use_span);
    assert_eq!(failure.declaration_spans(), declaration_spans);
}

fn assert_notation_dependency_cycle(
    error: &CanonicalModuleExpansionError,
    expected: &[(
        &ModuleKey,
        &ModuleKey,
        Span,
        &ExpectedModuleContext,
        &ExpectedModuleContext,
        Span,
    )],
) {
    let cycle: &CanonicalSyntaxDependencyCycle = error
        .syntax_dependency_cycle()
        .expect("notation dependencies participate in the canonical syntax cycle");
    assert_eq!(cycle.edges().len(), expected.len());
    for (edge, expected_edge) in cycle.edges().iter().zip(expected) {
        let (
            importer_key,
            provider_key,
            use_span,
            importer_context,
            provider_context,
            provider_declaration_span,
        ) = expected_edge;
        assert_eq!(edge.importer_key(), *importer_key);
        assert_eq!(edge.provider_key(), *provider_key);
        assert_eq!(edge.use_span(), *use_span);
        assert_eq!(
            edge.importer_source_path(),
            importer_context.source_path.as_deref()
        );
        assert_eq!(
            edge.importer_artifact_origin(),
            &importer_context.artifact_origin
        );
        assert_eq!(
            edge.provider_source_path(),
            provider_context.source_path.as_deref()
        );
        assert_eq!(
            edge.provider_artifact_origin(),
            &provider_context.artifact_origin
        );
        assert_eq!(edge.provider_declaration_span(), *provider_declaration_span);
    }
}

#[test]
fn canonical_public_notation_summary() {
    let tree = TempTree::new("public-summary");
    let root_path = tree.write("src/main.ash", "pub mod provider;\npub mod consumer;\n");
    tree.write(
        "src/provider.ash",
        r#"
            pub suffix 4 <*> = trailing_target
            pub mixfix _ between _ and _ = between_target
            pub infixl 6 <*> = combine_target
            pub prefix 9 <*> = leading_target
        "#,
    );
    tree.write(
        "src/consumer.ash",
        r#"
            use crate::provider::(<*>);
            use crate::provider::(_ between _ and _);
            fn untouched(value: Int) -> Int { value }
        "#,
    );

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let provider_key = root_key
        .child("provider")
        .expect("provider key is canonical");
    let consumer_key = root_key
        .child("consumer")
        .expect("consumer key is canonical");
    let parsed = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key, root_path)
        .expect("fixture builds one canonical parsed graph");

    let provider = parsed
        .module_unit(&provider_key)
        .expect("parsed provider exists");
    let provider_source_path = provider.source_path().map(str::to_owned);
    let provider_artifact_origin = provider.artifact().origin().clone();
    assert!(matches!(
        provider_artifact_origin,
        ModuleArtifactOrigin::File(_)
    ));
    let declarations = provider
        .body()
        .definitions()
        .iter()
        .filter_map(|definition| {
            let Definition::Notation(declaration) = definition else {
                return None;
            };
            Some((
                declaration.target.name.to_string(),
                ExpectedDeclaration {
                    target: declaration.target.clone(),
                    span: declaration.span,
                },
            ))
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(declarations.len(), 4, "fixture has four public summaries");

    let consumer = parsed
        .module_unit(&consumer_key)
        .expect("parsed consumer exists");
    let operator_use_span = consumer.body().uses()[0].span;
    let mixfix_use_span = consumer.body().uses()[1].span;

    // The tree is no longer needed after canonical source acquisition. Summary
    // construction and transport must consume typed retained AST/provenance.
    drop(tree);
    let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect("public notation summaries transport without activating notation");
    let consumer = expanded
        .module(&consumer_key)
        .expect("expanded consumer exists");
    let imports = consumer.notation_imports();
    assert_eq!(
        imports.len(),
        4,
        "both exact selectors transport every full-key variant"
    );

    let expected = [
        (
            &[
                CanonicalNotationPatternPart::Hole,
                CanonicalNotationPatternPart::Token("between".into()),
                CanonicalNotationPatternPart::Hole,
                CanonicalNotationPatternPart::Token("and".into()),
                CanonicalNotationPatternPart::Hole,
            ][..],
            CanonicalNotationFixityKey::Mixfix,
            "between_target",
            mixfix_use_span,
        ),
        (
            &[CanonicalNotationPatternPart::Token("<*>".into())][..],
            CanonicalNotationFixityKey::Prefix {
                precedence: Some(9),
            },
            "leading_target",
            operator_use_span,
        ),
        (
            &[CanonicalNotationPatternPart::Token("<*>".into())][..],
            CanonicalNotationFixityKey::Infix {
                associativity: NotationAssociativity::Left,
                precedence: 6,
            },
            "combine_target",
            operator_use_span,
        ),
        (
            &[CanonicalNotationPatternPart::Token("<*>".into())][..],
            CanonicalNotationFixityKey::Suffix {
                precedence: Some(4),
            },
            "trailing_target",
            operator_use_span,
        ),
    ];

    for (notation_import, (pattern, fixity, target_name, use_span)) in imports.iter().zip(expected)
    {
        assert_eq!(notation_import.provider_key(), &provider_key);
        assert_eq!(
            notation_import.provider_source_path(),
            provider_source_path.as_deref()
        );
        assert_eq!(
            notation_import.provider_artifact_origin(),
            &provider_artifact_origin
        );
        assert_eq!(notation_import.use_span(), use_span);

        let summary = notation_import.summary();
        assert_eq!(summary.key().pattern(), pattern);
        assert_eq!(summary.key().fixity(), &fixity);
        assert_eq!(summary.visibility(), &Visibility::Public);

        let declaration = declarations
            .get(target_name)
            .expect("expected provider declaration exists");
        assert_eq!(summary.target(), &declaration.target);
        assert_eq!(summary.declaration_span(), declaration.span);
    }
}

fn notation_declaration_permutation(mut rank: usize) -> [&'static str; 3] {
    let mut declarations = [
        "pub suffix 4 <*> = trailing_target",
        "pub infixl 6 <*> = combine_target",
        "pub prefix 9 <*> = leading_target",
    ];
    for slot in 0..declarations.len() {
        let remaining = declarations.len() - slot;
        let selected = slot + rank % remaining;
        declarations.swap(slot, selected);
        rank /= remaining;
    }
    declarations
}

fn notation_import_projection(
    provider_declarations: &[&str],
    label: &str,
) -> Vec<(
    Box<[CanonicalNotationPatternPart]>,
    CanonicalNotationFixityKey,
    Box<str>,
)> {
    notation_import_projection_from_source(&provider_declarations.join("\n"), label)
}

fn notation_import_projection_from_source(
    provider_source: &str,
    label: &str,
) -> Vec<(
    Box<[CanonicalNotationPatternPart]>,
    CanonicalNotationFixityKey,
    Box<str>,
)> {
    let tree = TempTree::new(label);
    let root_path = tree.write("src/main.ash", "pub mod provider;\npub mod consumer;\n");
    tree.write("src/provider.ash", provider_source);
    tree.write(
        "src/consumer.ash",
        "use crate::provider::(<*>);\nfn untouched(value: Int) -> Int { value }\n",
    );

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let consumer_key = root_key
        .child("consumer")
        .expect("consumer key is canonical");
    let parsed = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key, root_path)
        .expect("permuted notation fixture builds a canonical parsed graph");
    drop(tree);

    let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect("permuted public notation summaries transport");
    expanded
        .module(&consumer_key)
        .expect("expanded consumer exists")
        .notation_imports()
        .iter()
        .map(|notation_import| {
            let summary = notation_import.summary();
            (
                summary.key().pattern().into(),
                summary.key().fixity().clone(),
                summary.target().name.as_ref().into(),
            )
        })
        .collect()
}

#[test]
fn public_notation_summary_order_covers_every_provider_declaration_permutation() {
    let baseline = notation_import_projection(
        &notation_declaration_permutation(0),
        "deterministic-baseline",
    );
    let mut permutations = BTreeSet::new();

    for permutation_rank in 0..6 {
        let declaration_order = notation_declaration_permutation(permutation_rank);
        assert!(
            permutations.insert(declaration_order),
            "each factoradic rank must select a unique declaration order"
        );
        let projection =
            notation_import_projection(&declaration_order, "deterministic-exhaustive-permutation");
        assert_eq!(projection.len(), 3);
        assert_eq!(projection, baseline);
    }

    assert_eq!(permutations.len(), 6);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn public_notation_summary_projection_ignores_provider_source_trivia(
        indentation in 0usize..5,
        blank_lines in 0usize..3,
        retain_comment_lines in any::<bool>(),
    ) {
        let baseline = notation_import_projection(
            &notation_declaration_permutation(0),
            "trivia-baseline",
        );
        let indent = " ".repeat(indentation);
        let separator = "\n".repeat(blank_lines + 1);
        let comment = if retain_comment_lines {
            "// nonsemantic provider formatting\n"
        } else {
            ""
        };
        let provider_source = format!(
            "{comment}{indent}pub suffix 4 <*> = trailing_target{separator}\
             {comment}{indent}pub infixl 6 <*> = combine_target{separator}\
             {comment}{indent}pub prefix 9 <*> = leading_target\n"
        );
        let formatted = notation_import_projection_from_source(
            &provider_source,
            "provider-source-trivia",
        );

        prop_assert_eq!(formatted.len(), 3);
        prop_assert_eq!(formatted, baseline);
    }
}

#[test]
fn notation_dependency_private_declaration_rejects_with_complete_context() {
    let tree = TempTree::new("dependency-private-notation");
    let root_path = tree.write("src/main.ash", "pub mod provider;\npub mod consumer;\n");
    tree.write("src/provider.ash", "infixl 6 <+> = combine\n");
    tree.write(
        "src/consumer.ash",
        "use crate::provider::(<+>);\nfn untouched(value: Int) -> Int { value }\n",
    );

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let provider_key = root_key.child("provider").expect("provider key");
    let consumer_key = root_key.child("consumer").expect("consumer key");
    let parsed = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key, root_path)
        .expect("private notation fixture parses");
    let consumer_context = module_context(&parsed, &consumer_key);
    let provider_context = module_context(&parsed, &provider_key);
    let use_span = use_span_at(&parsed, &consumer_key, 0);
    let declaration_spans = notation_spans(&parsed, &provider_key);

    let error = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect_err("a private notation declaration must reject the whole graph");
    assert_notation_dependency_failure(
        &error,
        CanonicalNotationImportFailureKind::PrivateNotation,
        &consumer_key,
        &consumer_context,
        Some(&provider_key),
        Some(&provider_context),
        use_span,
        &declaration_spans,
    );
}

#[test]
fn notation_dependency_private_structural_path_rejects_at_module_and_use_anchors() {
    let tree = TempTree::new("dependency-private-path");
    let root_path = tree.write("src/main.ash", "mod provider;\npub mod consumer;\n");
    tree.write("src/provider.ash", "pub infixl 6 <+> = combine\n");
    tree.write("src/consumer.ash", "use crate::provider::(<+>);\n");

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let provider_key = root_key.child("provider").expect("provider key");
    let consumer_key = root_key.child("consumer").expect("consumer key");
    let parsed = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("private structural path fixture parses");
    let consumer_context = module_context(&parsed, &consumer_key);
    let provider_context = module_context(&parsed, &provider_key);
    let use_span = use_span_at(&parsed, &consumer_key, 0);
    let private_module_span = child_declaration_span(&parsed, &root_key, "provider");

    let error = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect_err("notation behind a private structural path must reject atomically");
    assert_notation_dependency_failure(
        &error,
        CanonicalNotationImportFailureKind::PrivateModulePath,
        &consumer_key,
        &consumer_context,
        Some(&provider_key),
        Some(&provider_context),
        use_span,
        &[private_module_span],
    );
}

#[test]
fn notation_dependency_missing_selector_summary_rejects_at_exact_use() {
    let tree = TempTree::new("dependency-missing-summary");
    let root_path = tree.write("src/main.ash", "pub mod provider;\npub mod consumer;\n");
    tree.write("src/provider.ash", "pub infixl 6 <-> = subtract\n");
    tree.write("src/consumer.ash", "use crate::provider::(<+>);\n");

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let provider_key = root_key.child("provider").expect("provider key");
    let consumer_key = root_key.child("consumer").expect("consumer key");
    let parsed = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key, root_path)
        .expect("missing summary fixture parses");
    let consumer_context = module_context(&parsed, &consumer_key);
    let provider_context = module_context(&parsed, &provider_key);
    let use_span = use_span_at(&parsed, &consumer_key, 0);

    let error = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect_err("a missing exact notation summary must reject the whole graph");
    assert_notation_dependency_failure(
        &error,
        CanonicalNotationImportFailureKind::MissingSummary,
        &consumer_key,
        &consumer_context,
        Some(&provider_key),
        Some(&provider_context),
        use_span,
        &[],
    );
}

#[test]
fn notation_dependency_local_and_imported_full_key_overlap_rejects_both_declarations() {
    let tree = TempTree::new("dependency-local-imported-overlap");
    let root_path = tree.write("src/main.ash", "pub mod provider;\npub mod consumer;\n");
    tree.write(
        "src/provider.ash",
        "\n\n\npub infixl 6 <+> = provider_combine\n",
    );
    tree.write(
        "src/consumer.ash",
        "infixl 6 <+> = local_combine\nuse crate::provider::(<+>);\n",
    );

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let provider_key = root_key.child("provider").expect("provider key");
    let consumer_key = root_key.child("consumer").expect("consumer key");
    let parsed = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key, root_path)
        .expect("local/imported overlap fixture parses");
    let consumer_context = module_context(&parsed, &consumer_key);
    let provider_context = module_context(&parsed, &provider_key);
    let use_span = use_span_at(&parsed, &consumer_key, 0);
    let local_span = notation_spans(&parsed, &consumer_key)[0];
    let provider_span = notation_spans(&parsed, &provider_key)[0];

    let error = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect_err("a local/imported full-key overlap must reject atomically");
    assert_notation_dependency_failure(
        &error,
        CanonicalNotationImportFailureKind::ConflictingActiveKey,
        &consumer_key,
        &consumer_context,
        Some(&provider_key),
        Some(&provider_context),
        use_span,
        &[local_span, provider_span],
    );
}

#[test]
fn notation_dependency_first_local_import_conflict_excludes_unrelated_group_anchors() {
    let tree = TempTree::new("dependency-local-imported-independent-groups");
    let root_path = tree.write(
        "src/main.ash",
        "pub mod a_provider;\npub mod b_provider;\npub mod consumer;\n",
    );
    tree.write("src/a_provider.ash", "pub infixl 6 <+> = add\n");
    tree.write("src/b_provider.ash", "pub prefix 9 <*> = lead\n");
    tree.write(
        "src/consumer.ash",
        "infixl 6 <+> = local_add\nprefix 9 <*> = local_lead\nuse crate::b_provider::(<*>);\nuse crate::a_provider::(<+>);\n",
    );

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let a_provider = root_key.child("a_provider").expect("a provider key");
    let consumer_key = root_key.child("consumer").expect("consumer key");
    let parsed = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key, root_path)
        .expect("independent local/import conflict groups parse");
    let consumer_context = module_context(&parsed, &consumer_key);
    let provider_context = module_context(&parsed, &a_provider);
    let use_span = use_span_at(&parsed, &consumer_key, 1);
    let local_span = notation_spans(&parsed, &consumer_key)[0];
    let provider_span = notation_spans(&parsed, &a_provider)[0];

    let error = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect_err("the canonical first independent conflict rejects atomically");
    assert_notation_dependency_failure(
        &error,
        CanonicalNotationImportFailureKind::ConflictingActiveKey,
        &consumer_key,
        &consumer_context,
        Some(&a_provider),
        Some(&provider_context),
        use_span,
        &[local_span, provider_span],
    );
}

#[test]
fn notation_dependency_two_imported_full_key_variants_reject_in_stable_provider_order() {
    let tree = TempTree::new("dependency-imported-overlap");
    let root_path = tree.write(
        "src/main.ash",
        "pub mod a_provider;\npub mod b_provider;\npub mod consumer;\n",
    );
    tree.write("src/a_provider.ash", "pub infixl 6 <+> = zeta_target\n");
    tree.write(
        "src/b_provider.ash",
        "\n\npub infixl 6 <+> = alpha_target\n",
    );
    tree.write(
        "src/consumer.ash",
        "use crate::b_provider::(<+>);\nuse crate::a_provider::(<+>);\n",
    );

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let a_provider = root_key.child("a_provider").expect("a provider key");
    let b_provider = root_key.child("b_provider").expect("b provider key");
    let consumer_key = root_key.child("consumer").expect("consumer key");
    let parsed = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key, root_path)
        .expect("imported overlap fixture parses");
    let consumer_context = module_context(&parsed, &consumer_key);
    let b_provider_context = module_context(&parsed, &b_provider);
    let b_provider_use_span = use_span_at(&parsed, &consumer_key, 0);
    let a_span = notation_spans(&parsed, &a_provider)[0];
    let b_span = notation_spans(&parsed, &b_provider)[0];

    let error = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect_err("two imported equal full keys must reject atomically");
    assert_notation_dependency_failure(
        &error,
        CanonicalNotationImportFailureKind::ConflictingActiveKey,
        &consumer_key,
        &consumer_context,
        Some(&b_provider),
        Some(&b_provider_context),
        b_provider_use_span,
        &[a_span, b_span],
    );
}

#[test]
fn notation_dependency_incompatible_precedence_and_associativity_retain_all_provider_anchors() {
    let tree = TempTree::new("dependency-incompatible-fixity");
    let root_path = tree.write(
        "src/main.ash",
        "pub mod a_provider;\npub mod b_provider;\npub mod consumer;\n",
    );
    tree.write("src/a_provider.ash", "pub infixl 6 <+> = left_combine\n");
    tree.write(
        "src/b_provider.ash",
        "\n\n\npub infixr 7 <+> = right_combine\n",
    );
    tree.write(
        "src/consumer.ash",
        "use crate::a_provider::(<+>);\nuse crate::b_provider::(<+>);\n",
    );

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let a_provider = root_key.child("a_provider").expect("a provider key");
    let b_provider = root_key.child("b_provider").expect("b provider key");
    let consumer_key = root_key.child("consumer").expect("consumer key");
    let parsed = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key, root_path)
        .expect("incompatible fixity fixture parses");
    let consumer_context = module_context(&parsed, &consumer_key);
    let provider_context = module_context(&parsed, &b_provider);
    let use_span = use_span_at(&parsed, &consumer_key, 1);
    let declaration_spans = [
        notation_spans(&parsed, &a_provider)[0],
        notation_spans(&parsed, &b_provider)[0],
    ];

    let error = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect_err("incompatible imported precedence/associativity must reject atomically");
    assert_notation_dependency_failure(
        &error,
        CanonicalNotationImportFailureKind::ConflictingActiveKey,
        &consumer_key,
        &consumer_context,
        Some(&b_provider),
        Some(&provider_context),
        use_span,
        &declaration_spans,
    );
}

#[test]
fn notation_dependency_local_prefix_and_imported_infix_same_pattern_are_compatible() {
    let tree = TempTree::new("dependency-compatible-local-imported-classes");
    let root_path = tree.write("src/main.ash", "pub mod provider;\npub mod consumer;\n");
    tree.write("src/provider.ash", "pub infixl 6 <*> = combine\n");
    tree.write(
        "src/consumer.ash",
        "prefix 9 <*> = leading\nuse crate::provider::(<*>);\n",
    );

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let provider_key = root_key.child("provider").expect("provider key");
    let consumer_key = root_key.child("consumer").expect("consumer key");
    let parsed = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key, root_path)
        .expect("compatible local/imported fixity classes parse");

    let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect("prefix and infix classes for one pattern may coexist");
    let consumer = expanded.module(&consumer_key).expect("consumer expands");
    let imports = consumer.notation_imports();
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].provider_key(), &provider_key);
    assert_eq!(
        imports[0].summary().key().fixity(),
        &CanonicalNotationFixityKey::Infix {
            associativity: NotationAssociativity::Left,
            precedence: 6,
        }
    );
}

#[test]
fn notation_dependency_imported_prefix_and_infix_from_distinct_providers_are_compatible() {
    let tree = TempTree::new("dependency-compatible-imported-classes");
    let root_path = tree.write(
        "src/main.ash",
        "pub mod prefix_provider;\npub mod infix_provider;\npub mod consumer;\n",
    );
    tree.write("src/prefix_provider.ash", "pub prefix 9 <*> = leading\n");
    tree.write("src/infix_provider.ash", "pub infixl 6 <*> = combine\n");
    tree.write(
        "src/consumer.ash",
        "use crate::prefix_provider::(<*>);\nuse crate::infix_provider::(<*>);\n",
    );

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let prefix_provider = root_key
        .child("prefix_provider")
        .expect("prefix provider key");
    let infix_provider = root_key
        .child("infix_provider")
        .expect("infix provider key");
    let consumer_key = root_key.child("consumer").expect("consumer key");
    let parsed = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key, root_path)
        .expect("compatible imported fixity classes parse");

    let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect("imported prefix and infix classes for one pattern may coexist");
    let consumer = expanded.module(&consumer_key).expect("consumer expands");
    let imports = consumer.notation_imports();
    assert_eq!(imports.len(), 2);
    let actual = imports
        .iter()
        .map(|import| (import.provider_key(), import.summary().key().fixity()))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual,
        BTreeSet::from([
            (
                &prefix_provider,
                &CanonicalNotationFixityKey::Prefix {
                    precedence: Some(9),
                },
            ),
            (
                &infix_provider,
                &CanonicalNotationFixityKey::Infix {
                    associativity: NotationAssociativity::Left,
                    precedence: 6,
                },
            ),
        ])
    );
}

#[test]
fn notation_dependency_two_module_macro_notation_cycle_has_stable_edge_order() {
    let tree = TempTree::new("dependency-two-cycle");
    let root_path = tree.write("src/main.ash", "pub mod a;\npub mod b;\n");
    tree.write(
        "src/a.ash",
        "use crate::b::(<b>);\npub macro a_macro(x) => x;\n",
    );
    tree.write(
        "src/b.ash",
        "use crate::a::a_macro;\npub prefix 8 <b> = b_target\nfn run(n: Int) -> Int { a_macro!(n) }\n",
    );

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let a = root_key.child("a").expect("a key");
    let b = root_key.child("b").expect("b key");
    let parsed = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key, root_path)
        .expect("two-module mixed syntax cycle fixture parses");
    let a_context = module_context(&parsed, &a);
    let b_context = module_context(&parsed, &b);
    let a_use = use_span_at(&parsed, &a, 0);
    let b_use = use_span_at(&parsed, &b, 0);
    let a_macro_span = macro_span(&parsed, &a);
    let b_notation_span = notation_spans(&parsed, &b)[0];

    let error = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect_err("a mixed macro/notation dependency cycle must reject atomically");
    assert_notation_dependency_cycle(
        &error,
        &[
            (&a, &b, a_use, &a_context, &b_context, b_notation_span),
            (&b, &a, b_use, &b_context, &a_context, a_macro_span),
        ],
    );
}

#[test]
fn notation_dependency_three_module_notation_cycle_has_stable_edge_order() {
    let tree = TempTree::new("dependency-three-cycle");
    let root_path = tree.write("src/main.ash", "pub mod a;\npub mod b;\npub mod c;\n");
    tree.write(
        "src/a.ash",
        "use crate::b::(<b>);\npub prefix 8 <a> = a_target\n",
    );
    tree.write(
        "src/b.ash",
        "use crate::c::(<c>);\npub prefix 8 <b> = b_target\n",
    );
    tree.write(
        "src/c.ash",
        "use crate::a::(<a>);\npub prefix 8 <c> = c_target\n",
    );

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let a = root_key.child("a").expect("a key");
    let b = root_key.child("b").expect("b key");
    let c = root_key.child("c").expect("c key");
    let parsed = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key, root_path)
        .expect("three-module notation cycle fixture parses");
    let a_context = module_context(&parsed, &a);
    let b_context = module_context(&parsed, &b);
    let c_context = module_context(&parsed, &c);
    let a_use = use_span_at(&parsed, &a, 0);
    let b_use = use_span_at(&parsed, &b, 0);
    let c_use = use_span_at(&parsed, &c, 0);
    let a_notation_span = notation_spans(&parsed, &a)[0];
    let b_notation_span = notation_spans(&parsed, &b)[0];
    let c_notation_span = notation_spans(&parsed, &c)[0];

    let error = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect_err("a three-module notation dependency cycle must reject atomically");
    assert_notation_dependency_cycle(
        &error,
        &[
            (&a, &b, a_use, &a_context, &b_context, b_notation_span),
            (&b, &c, b_use, &b_context, &c_context, c_notation_span),
            (&c, &a, c_use, &c_context, &a_context, a_notation_span),
        ],
    );
}

#[test]
fn notation_dependency_valid_sibling_plus_invalid_edge_returns_only_error() {
    let tree = TempTree::new("dependency-atomic-sibling");
    let root_path = tree.write(
        "src/main.ash",
        "pub mod good_provider;\npub mod bad_provider;\npub mod consumer;\n",
    );
    tree.write("src/good_provider.ash", "pub infixl 6 <+> = combine\n");
    tree.write("src/bad_provider.ash", "pub infixl 6 <-> = subtract\n");
    tree.write(
        "src/consumer.ash",
        "use crate::good_provider::(<+>);\nuse crate::bad_provider::(<*>);\n",
    );

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let bad_provider = root_key.child("bad_provider").expect("bad provider key");
    let consumer_key = root_key.child("consumer").expect("consumer key");
    let parsed = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key, root_path)
        .expect("atomic sibling fixture parses");
    let consumer_context = module_context(&parsed, &consumer_key);
    let provider_context = module_context(&parsed, &bad_provider);
    let invalid_use_span = use_span_at(&parsed, &consumer_key, 1);

    let result = CanonicalExpandedModuleGraph::try_expand(parsed);
    let error = result.expect_err(
        "one invalid notation edge must discard the valid sibling and publish no graph",
    );
    assert_notation_dependency_failure(
        &error,
        CanonicalNotationImportFailureKind::MissingSummary,
        &consumer_key,
        &consumer_context,
        Some(&bad_provider),
        Some(&provider_context),
        invalid_use_span,
        &[],
    );
}
