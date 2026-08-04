//! TASK-2074 canonical public notation-summary transport evidence.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::{ModuleArtifactOrigin, ModuleKey};
use ash_parser::surface::{CallablePath, Definition, NotationAssociativity, Visibility};
use ash_parser::{
    CanonicalExpandedModuleGraph, CanonicalModuleGraphResolver, CanonicalNotationFixityKey,
    CanonicalNotationPatternPart, Span,
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
