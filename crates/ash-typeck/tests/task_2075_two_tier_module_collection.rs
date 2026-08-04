//! TASK-2075 RED contract for complete two-tier module collection.
//!
//! The collector consumes only TASK-2074's immutable expanded graph. It publishes a
//! checker-internal snapshot and a separate import-facing, name-only view, or publishes
//! neither output when removed target syntax is encountered.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::ModuleKey;
use ash_parser::surface::{CapabilityDef, Definition, EffectType, Visibility};
use ash_parser::{CanonicalExpandedModuleGraph, CanonicalModuleGraphResolver};
use ash_typeck::canonical_module_collection::{
    CanonicalCollectionDisposition, CanonicalDeclarationKind, CanonicalModuleCollectionErrorKind,
    CanonicalNamespace, collect_canonical_expanded_module_graph,
    validate_definition_batch_for_test,
};

static NEXT_TEMP_TREE: AtomicUsize = AtomicUsize::new(0);

struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(label: &str) -> Self {
        let serial = NEXT_TEMP_TREE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "ash-task-2075-module-collection-{label}-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create TASK-2075 parser fixture directory");
        Self { root }
    }

    fn write(&self, relative: impl AsRef<Path>, source: &str) -> PathBuf {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("fixture path has a parent"))
            .expect("create TASK-2075 fixture parent directory");
        fs::write(&path, source).expect("write TASK-2075 parser fixture");
        path
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ExpectedDomainCase {
    kind: CanonicalDeclarationKind,
    disposition: CanonicalCollectionDisposition,
}

const EXPECTED_DOMAIN: [ExpectedDomainCase; 22] = [
    collect(
        CanonicalDeclarationKind::Notation,
        CanonicalNamespace::Notation,
    ),
    collect(CanonicalDeclarationKind::Macro, CanonicalNamespace::Macro),
    reject(CanonicalDeclarationKind::Capability),
    collect(
        CanonicalDeclarationKind::ResourceType,
        CanonicalNamespace::TypeDomain,
    ),
    collect(
        CanonicalDeclarationKind::Type,
        CanonicalNamespace::TypeDomain,
    ),
    collect(
        CanonicalDeclarationKind::Newtype,
        CanonicalNamespace::TypeDomain,
    ),
    collect(
        CanonicalDeclarationKind::EffectAlias,
        CanonicalNamespace::RowName,
    ),
    collect(
        CanonicalDeclarationKind::EffectGroup,
        CanonicalNamespace::RowName,
    ),
    collect(
        CanonicalDeclarationKind::DataKind,
        CanonicalNamespace::PromotedKind,
    ),
    collect(
        CanonicalDeclarationKind::TypeFn,
        CanonicalNamespace::TypeComputation,
    ),
    collect(
        CanonicalDeclarationKind::PropositionPredicate,
        CanonicalNamespace::Proposition,
    ),
    collect(CanonicalDeclarationKind::Policy, CanonicalNamespace::Policy),
    collect(CanonicalDeclarationKind::Role, CanonicalNamespace::Role),
    collect(
        CanonicalDeclarationKind::Interface,
        CanonicalNamespace::Interface,
    ),
    internal_only(
        CanonicalDeclarationKind::Impl,
        CanonicalNamespace::ImplementationRegistry,
    ),
    collect(
        CanonicalDeclarationKind::Function,
        CanonicalNamespace::ValueCallable,
    ),
    collect(
        CanonicalDeclarationKind::Handler,
        CanonicalNamespace::ValueCallable,
    ),
    collect(
        CanonicalDeclarationKind::BuiltinFn,
        CanonicalNamespace::ValueCallable,
    ),
    collect(
        CanonicalDeclarationKind::SealedDomain,
        CanonicalNamespace::TypeDomain,
    ),
    collect(CanonicalDeclarationKind::Law, CanonicalNamespace::Evidence),
    collect(
        CanonicalDeclarationKind::Proof,
        CanonicalNamespace::Evidence,
    ),
    collect(
        CanonicalDeclarationKind::ModuleDecl,
        CanonicalNamespace::StructuralModule,
    ),
];

const fn collect(
    kind: CanonicalDeclarationKind,
    namespace: CanonicalNamespace,
) -> ExpectedDomainCase {
    ExpectedDomainCase {
        kind,
        disposition: CanonicalCollectionDisposition::Collect {
            namespace,
            publish_in_name_view: true,
        },
    }
}

const fn internal_only(
    kind: CanonicalDeclarationKind,
    namespace: CanonicalNamespace,
) -> ExpectedDomainCase {
    ExpectedDomainCase {
        kind,
        disposition: CanonicalCollectionDisposition::Collect {
            namespace,
            publish_in_name_view: false,
        },
    }
}

const fn reject(kind: CanonicalDeclarationKind) -> ExpectedDomainCase {
    ExpectedDomainCase {
        kind,
        disposition: CanonicalCollectionDisposition::RejectAtomically,
    }
}

fn expanded_fixture() -> (CanonicalExpandedModuleGraph, ModuleKey) {
    let tree = TempTree::new("representative");
    let root_path = tree.write(
        "src/main.ash",
        r#"
            pub mod api {
                pub fn greet(value: Int) -> Int { value + 1 }
            }
            pub fn entry(value: Int) -> Int { api::greet(value) }
        "#,
    );
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let parsed = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("fixture resolves through the canonical parser graph");
    let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect("fixture expands through the canonical expanded graph");
    (expanded, root_key)
}

#[test]
fn definition_domain_is_closed_exhaustive_and_has_one_collection_disposition_per_kind() {
    assert_eq!(EXPECTED_DOMAIN.len(), CanonicalDeclarationKind::ALL.len());

    for kind in CanonicalDeclarationKind::ALL.iter() {
        assert_eq!(
            EXPECTED_DOMAIN
                .iter()
                .filter(|expected| expected.kind == *kind)
                .count(),
            1,
            "closed declaration kind {kind:?} must have exactly one domain row"
        );
    }

    for (index, expected) in EXPECTED_DOMAIN.iter().enumerate() {
        assert!(
            CanonicalDeclarationKind::ALL.contains(&expected.kind),
            "domain row {:?} must name a closed declaration kind",
            expected.kind
        );
        assert_eq!(
            expected.kind.collection_disposition(),
            expected.disposition,
            "wrong namespace or name-view policy for {:?}",
            expected.kind
        );
        assert!(
            !EXPECTED_DOMAIN[..index]
                .iter()
                .any(|earlier| earlier.kind == expected.kind),
            "duplicate declaration-domain row for {:?}",
            expected.kind
        );
    }

    assert_eq!(
        CanonicalDeclarationKind::Capability.collection_disposition(),
        CanonicalCollectionDisposition::RejectAtomically,
        "removed capability syntax must abort publication of both views"
    );
}

#[test]
fn removed_capability_rejects_a_supported_sibling_batch_before_either_view_is_published() {
    let (expanded, root_key) = expanded_fixture();
    let supported = expanded
        .module(&root_key)
        .expect("fixture root remains in the expanded graph")
        .body()
        .definitions()
        .iter()
        .find(|definition| matches!(definition, Definition::Function(_)))
        .expect("fixture contains one supported function sibling")
        .clone();
    let capability_span = ash_parser::Span::new(700, 730, 20, 5);
    let removed = Definition::Capability(CapabilityDef {
        visibility: Visibility::Public,
        name: "removed_io".into(),
        effect: EffectType::Write,
        params: Vec::new(),
        return_type: None,
        constraints: Vec::new(),
        target_provider: None,
        target_action: None,
        span: capability_span,
    });
    let definitions = [supported, removed];

    // This hidden validation seam shares the production classifier and staging path, but its
    // `Result<(), _>` cannot publish or return either carrier even for a successful batch.
    let error = match validate_definition_batch_for_test(&root_key, &definitions) {
        Ok(()) => panic!("removed capability syntax must reject the complete sibling batch"),
        Err(error) => error,
    };

    assert_eq!(
        error.kind(),
        CanonicalModuleCollectionErrorKind::RemovedCapabilitySyntax
    );
    assert_eq!(error.module_key(), &root_key);
    assert_eq!(error.declaration_name(), Some("removed_io"));
    assert_eq!(error.declaration_span(), capability_span);
}

#[test]
fn representative_expanded_graph_publishes_separate_internal_and_name_only_views() {
    let (expanded, root_key) = expanded_fixture();
    let collected = collect_canonical_expanded_module_graph(&expanded)
        .expect("a supported expanded graph collects atomically");
    let snapshot = collected
        .internal_snapshot(&root_key)
        .expect("root checker-internal snapshot is published");
    let name_view = collected
        .provisional_name_view(&root_key)
        .expect("root import-facing name view is published");

    let module = snapshot
        .entries()
        .find(|entry| entry.declared_name() == Some("api"))
        .expect("structural module declaration is collected internally");
    assert_eq!(module.kind(), CanonicalDeclarationKind::ModuleDecl);
    assert_eq!(module.namespace(), CanonicalNamespace::StructuralModule);

    let function = snapshot
        .entries()
        .find(|entry| entry.declared_name() == Some("entry"))
        .expect("callable declaration is collected internally");
    assert_eq!(function.kind(), CanonicalDeclarationKind::Function);
    assert_eq!(function.namespace(), CanonicalNamespace::ValueCallable);

    let public_names = name_view
        .entries()
        .map(|entry| (entry.lookup_name(), entry.namespace()))
        .collect::<Vec<_>>();
    assert!(public_names.contains(&("api", CanonicalNamespace::StructuralModule)));
    assert!(public_names.contains(&("entry", CanonicalNamespace::ValueCallable)));
}
