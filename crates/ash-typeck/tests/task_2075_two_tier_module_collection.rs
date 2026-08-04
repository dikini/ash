//! TASK-2075 RED contract for complete two-tier module collection.
//!
//! The collector consumes only TASK-2074's immutable expanded graph. It publishes a
//! checker-internal snapshot and a separate import-facing, name-only view, or publishes
//! neither output when removed target syntax is encountered.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::ModuleKey;
use ash_parser::surface::{
    Definition, ExpandedSurfaceOrigin, ExpansionId, IdentifierHygieneMetadata, Visibility,
};
use ash_parser::{CanonicalExpandedModuleGraph, CanonicalModuleGraphResolver};
use ash_typeck::canonical_module_collection::{
    CanonicalCollectedModuleRef, CanonicalCollectionDisposition, CanonicalDeclarationIdentity,
    CanonicalDeclarationKind, CanonicalDeclarationOriginKey, CanonicalLookupKey,
    CanonicalModuleCollection, CanonicalNamespace, collect_canonical_expanded_module_graph,
};
use syn::{Fields, GenericArgument, ImplItem, Item, PathArguments, ReturnType, Type};

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

const CARRIER_NAMES: [&str; 8] = [
    "CanonicalDeclarationIdentity",
    "CanonicalLookupKey",
    "CanonicalCollectedEntry",
    "CanonicalProvisionalNameEntry",
    "CanonicalCollectedModuleSnapshot",
    "CanonicalProvisionalNameView",
    "CanonicalCollectedModuleRef",
    "CanonicalModuleCollection",
];

fn is_visible(visibility: &syn::Visibility) -> bool {
    !matches!(visibility, syn::Visibility::Inherited)
}

fn type_name(ty: &Type) -> Option<&syn::Ident> {
    let Type::Path(path) = ty else { return None };
    path.path.segments.last().map(|segment| &segment.ident)
}

fn type_shape(ty: &Type) -> Result<String, String> {
    match ty {
        Type::Path(path) if path.qself.is_none() => path
            .path
            .segments
            .iter()
            .map(|segment| {
                let mut shape = segment.ident.to_string();
                match &segment.arguments {
                    PathArguments::None => {}
                    PathArguments::AngleBracketed(arguments) => {
                        let arguments = arguments
                            .args
                            .iter()
                            .map(|argument| match argument {
                                GenericArgument::Type(ty) => type_shape(ty),
                                _ => Err("non-type generic argument".to_owned()),
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        shape.push('<');
                        shape.push_str(&arguments.join(","));
                        shape.push('>');
                    }
                    PathArguments::Parenthesized(_) => {
                        return Err("parenthesized path arguments".to_owned());
                    }
                }
                Ok(shape)
            })
            .collect::<Result<Vec<_>, String>>()
            .map(|segments| segments.join("::")),
        Type::Reference(reference) if reference.mutability.is_none() => {
            type_shape(&reference.elem).map(|inner| format!("&{inner}"))
        }
        Type::Tuple(tuple) => tuple
            .elems
            .iter()
            .map(type_shape)
            .collect::<Result<Vec<_>, _>>()
            .map(|elements| format!("({})", elements.join(","))),
        Type::Slice(slice) => type_shape(&slice.elem).map(|inner| format!("[{inner}]")),
        _ => Err("unsupported type shape".to_owned()),
    }
}

fn origin_source_ordinal(origin: &CanonicalDeclarationOriginKey) -> usize {
    match origin {
        CanonicalDeclarationOriginKey::Source { source_ordinal } => *source_ordinal,
        CanonicalDeclarationOriginKey::Expanded {
            expansion_id,
            source_ordinal,
        } => {
            let _: &ExpansionId = expansion_id;
            *source_ordinal
        }
    }
}

fn inspect_carrier_source(source: &str) -> Result<(), String> {
    let file = syn::parse_file(source).map_err(|error| error.to_string())?;
    let mut collected_entry_fields = Vec::new();
    let mut provisional_fields = Vec::new();
    let mut snapshot_fields = Vec::new();
    let mut collection_fields = Vec::new();
    for carrier in CARRIER_NAMES {
        let structures = file
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Struct(item) if item.ident == carrier => Some(item),
                _ => None,
            })
            .collect::<Vec<_>>();
        if structures.len() != 1 {
            return Err(format!("expected one {carrier} struct"));
        }
        let Fields::Named(fields) = &structures[0].fields else {
            return Err(format!("{carrier} must use named private fields"));
        };
        if fields.named.iter().any(|field| is_visible(&field.vis)) {
            return Err(format!("{carrier} exposes a public or restricted field"));
        }
        let field_shapes = || {
            fields
                .named
                .iter()
                .map(|field| {
                    Ok((
                        field.ident.as_ref().expect("named field").to_string(),
                        type_shape(&field.ty)?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()
        };
        match carrier {
            "CanonicalCollectedEntry" => collected_entry_fields = field_shapes()?,
            "CanonicalProvisionalNameEntry" => provisional_fields = field_shapes()?,
            "CanonicalCollectedModuleSnapshot" => snapshot_fields = field_shapes()?,
            "CanonicalModuleCollection" => collection_fields = field_shapes()?,
            _ => {}
        }
    }

    let paired_modules = file
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Struct(item) if item.ident == "CanonicalCollectedModule" => Some(item),
            _ => None,
        })
        .collect::<Vec<_>>();
    if paired_modules.len() != 1 || is_visible(&paired_modules[0].vis) {
        return Err("expected one private CanonicalCollectedModule".to_owned());
    }
    let Fields::Named(paired_fields) = &paired_modules[0].fields else {
        return Err("CanonicalCollectedModule must use named private fields".to_owned());
    };
    if paired_fields
        .named
        .iter()
        .any(|field| is_visible(&field.vis))
    {
        return Err("CanonicalCollectedModule exposes a field".to_owned());
    }
    let mut paired_fields = paired_fields
        .named
        .iter()
        .map(|field| {
            Ok((
                field.ident.as_ref().expect("named field").to_string(),
                type_shape(&field.ty)?,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;

    let mut provisional_accessors = Vec::new();
    for carrier in CARRIER_NAMES {
        for implementation in file.items.iter().filter_map(|item| match item {
            Item::Impl(item)
                if item.trait_.is_none()
                    && type_name(&item.self_ty).is_some_and(|name| name == carrier) =>
            {
                Some(item)
            }
            _ => None,
        }) {
            for item in &implementation.items {
                match item {
                    ImplItem::Fn(function) if is_visible(&function.vis) => {
                        let Some(receiver) = function.sig.receiver() else {
                            return Err(format!(
                                "{carrier} exposes builder {}",
                                function.sig.ident
                            ));
                        };
                        if carrier == "CanonicalProvisionalNameEntry" {
                            if !matches!(function.vis, syn::Visibility::Public(_))
                                || receiver.reference.is_none()
                                || receiver
                                    .reference
                                    .as_ref()
                                    .is_some_and(|(_, lifetime)| lifetime.is_some())
                                || receiver.mutability.is_some()
                                || receiver.colon_token.is_some()
                                || function.sig.inputs.len() != 1
                                || !function.sig.generics.params.is_empty()
                                || function.sig.generics.where_clause.is_some()
                                || function.sig.constness.is_some()
                                || function.sig.asyncness.is_some()
                                || function.sig.unsafety.is_some()
                                || function.sig.abi.is_some()
                                || function.sig.variadic.is_some()
                            {
                                return Err(format!(
                                    "non-read-only accessor {}",
                                    function.sig.ident
                                ));
                            }
                            let ReturnType::Type(_, output) = &function.sig.output else {
                                return Err(format!(
                                    "accessor {} has no result",
                                    function.sig.ident
                                ));
                            };
                            provisional_accessors
                                .push((function.sig.ident.to_string(), type_shape(output)?));
                        }
                    }
                    ImplItem::Const(item) if is_visible(&item.vis) => {
                        return Err(format!("{carrier} exposes const {}", item.ident));
                    }
                    ImplItem::Type(item) if is_visible(&item.vis) => {
                        return Err(format!("{carrier} exposes type {}", item.ident));
                    }
                    _ => {}
                }
            }
        }
    }

    collected_entry_fields.sort_unstable();
    provisional_fields.sort_unstable();
    provisional_accessors.sort_unstable();
    snapshot_fields.sort_unstable();
    collection_fields.sort_unstable();
    paired_fields.sort_unstable();
    if collected_entry_fields
        != [
            ("declared_name".to_owned(), "Option<Box<str>>".to_owned()),
            (
                "identity".to_owned(),
                "CanonicalDeclarationIdentity".to_owned(),
            ),
            ("lookup_key".to_owned(), "CanonicalLookupKey".to_owned()),
            ("raw_definition".to_owned(), "Option<Definition>".to_owned()),
        ]
    {
        return Err("collected entry field shape differs from exact contract".to_owned());
    }
    let mut expected_fields = vec![
        ("exportable".to_owned(), "bool".to_owned()),
        (
            "identity".to_owned(),
            "CanonicalDeclarationIdentity".to_owned(),
        ),
        ("lookup_key".to_owned(), "CanonicalLookupKey".to_owned()),
        ("lookup_name".to_owned(), "Box<str>".to_owned()),
        ("namespace".to_owned(), "CanonicalNamespace".to_owned()),
        ("origin_anchor".to_owned(), "Span".to_owned()),
        ("source_ordinal".to_owned(), "usize".to_owned()),
        ("visibility".to_owned(), "Visibility".to_owned()),
    ];
    let mut expected_accessors = vec![
        (
            "identity".to_owned(),
            "&CanonicalDeclarationIdentity".to_owned(),
        ),
        ("is_exportable".to_owned(), "bool".to_owned()),
        ("lookup_key".to_owned(), "&CanonicalLookupKey".to_owned()),
        ("lookup_name".to_owned(), "&str".to_owned()),
        ("namespace".to_owned(), "CanonicalNamespace".to_owned()),
        ("origin_anchor".to_owned(), "Span".to_owned()),
        ("source_ordinal".to_owned(), "usize".to_owned()),
        ("visibility".to_owned(), "&Visibility".to_owned()),
    ];
    expected_fields.sort_unstable();
    expected_accessors.sort_unstable();
    if provisional_fields != expected_fields || provisional_accessors != expected_accessors {
        return Err("provisional field/accessor shape differs from exact contract".to_owned());
    }
    if snapshot_fields
        != [
            (
                "entries".to_owned(),
                "Box<[CanonicalCollectedEntry]>".to_owned(),
            ),
            (
                "expansion_origins".to_owned(),
                "Box<[ExpandedSurfaceOrigin]>".to_owned(),
            ),
            (
                "hygiene".to_owned(),
                "Box<[IdentifierHygieneMetadata]>".to_owned(),
            ),
        ]
    {
        return Err("internal snapshot field shape differs from exact contract".to_owned());
    }
    if collection_fields
        != [(
            "modules".to_owned(),
            "BTreeMap<ModuleKey,CanonicalCollectedModule>".to_owned(),
        )]
    {
        return Err("collection must contain exactly one paired module map".to_owned());
    }
    if paired_fields
        != [
            (
                "internal_snapshot".to_owned(),
                "CanonicalCollectedModuleSnapshot".to_owned(),
            ),
            (
                "provisional_name_view".to_owned(),
                "CanonicalProvisionalNameView".to_owned(),
            ),
        ]
    {
        return Err("paired module field shape differs from exact contract".to_owned());
    }
    Ok(())
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
fn representative_expanded_graph_publishes_separate_internal_and_name_only_views() {
    let (expanded, root_key) = expanded_fixture();
    let collected: CanonicalModuleCollection = collect_canonical_expanded_module_graph(&expanded)
        .expect("a supported expanded graph collects atomically");
    assert_eq!(collected.modules().count(), 2);
    let module: CanonicalCollectedModuleRef<'_> =
        collected.module(&root_key).expect("read-only module query");
    assert_eq!(module.module_key(), &root_key);
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

    let identity: &CanonicalDeclarationIdentity = function.identity();
    assert_eq!(identity.module_key(), &root_key);
    assert_eq!(identity.kind(), CanonicalDeclarationKind::Function);
    let parent: Option<&CanonicalDeclarationIdentity> = identity.canonical_parent();
    assert!(parent.is_none());
    let origin_key: &CanonicalDeclarationOriginKey = identity.origin_key();
    assert!(matches!(
        origin_key,
        CanonicalDeclarationOriginKey::Source { .. }
    ));
    let lookup: &CanonicalLookupKey = function.lookup_key();
    assert_eq!(lookup.namespace(), CanonicalNamespace::ValueCallable);
    assert_eq!(lookup.visible_local_key(), "entry");
    let raw_definition: &Definition = function
        .raw_definition()
        .expect("internal entry retains its raw definition");
    let Definition::Function(raw_function) = raw_definition else {
        panic!("function entry retains the matching raw function definition");
    };
    assert_eq!(function.callable_body(), Some(&raw_function.body));
    assert_eq!(origin_source_ordinal(origin_key), 1);
    let _: &[ExpandedSurfaceOrigin] = snapshot.expansion_origins();
    let _: &[IdentifierHygieneMetadata] = snapshot.hygiene();

    let provisional = name_view
        .entries()
        .find(|entry| entry.lookup_name() == "entry")
        .expect("provisional entry");
    assert_eq!(identity.origin_key(), provisional.identity().origin_key());
    assert_eq!(lookup, provisional.lookup_key());
    assert_eq!(provisional.identity(), identity);
    assert_eq!(provisional.lookup_name(), "entry");
    assert_eq!(provisional.namespace(), CanonicalNamespace::ValueCallable);
    assert_eq!(provisional.visibility(), &Visibility::Public);
    assert!(provisional.is_exportable());
    assert_eq!(provisional.origin_anchor(), raw_function.span);
    assert_eq!(provisional.source_ordinal(), 1);
}

#[test]
fn carrier_source_fence_enforces_private_construction_and_exact_name_view() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/canonical_module_collection.rs");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    inspect_carrier_source(&source).unwrap_or_else(|error| panic!("carrier fence: {error}"));
}

fn valid_fence_fixture() -> String {
    r#"
        struct CanonicalDeclarationIdentity { module_key: ModuleKey }
        struct CanonicalLookupKey { namespace: Namespace }
        struct CanonicalCollectedEntry {
            identity: CanonicalDeclarationIdentity,
            lookup_key: CanonicalLookupKey,
            declared_name: Option<Box<str>>,
            raw_definition: Option<Definition>,
        }
        struct CanonicalCollectedModuleSnapshot {
            entries: Box<[CanonicalCollectedEntry]>,
            expansion_origins: Box<[ExpandedSurfaceOrigin]>,
            hygiene: Box<[IdentifierHygieneMetadata]>,
        }
        struct CanonicalProvisionalNameView { entries: Vec<Opaque> }
        struct CanonicalCollectedModuleRef<'a> { collection: &'a Opaque }
        struct CanonicalCollectedModule {
            internal_snapshot: CanonicalCollectedModuleSnapshot,
            provisional_name_view: CanonicalProvisionalNameView,
        }
        struct CanonicalModuleCollection {
            modules: BTreeMap<ModuleKey, CanonicalCollectedModule>
        }
        struct CanonicalProvisionalNameEntry {
            identity: CanonicalDeclarationIdentity, lookup_name: Box<str>,
            lookup_key: CanonicalLookupKey, namespace: CanonicalNamespace,
            visibility: Visibility, exportable: bool,
            origin_anchor: Span, source_ordinal: usize,
        }
        impl CanonicalProvisionalNameEntry {
            pub fn identity(&self) -> &CanonicalDeclarationIdentity { unimplemented!() }
            pub fn lookup_name(&self) -> &str { unimplemented!() }
            pub fn lookup_key(&self) -> &CanonicalLookupKey { unimplemented!() }
            pub fn namespace(&self) -> CanonicalNamespace { unimplemented!() }
            pub fn visibility(&self) -> &Visibility { unimplemented!() }
            pub fn is_exportable(&self) -> bool { unimplemented!() }
            pub fn origin_anchor(&self) -> Span { unimplemented!() }
            pub fn source_ordinal(&self) -> usize { unimplemented!() }
        }
    "#
    .to_owned()
}

#[test]
fn syn_fence_handles_adversarial_source_without_substring_false_results() {
    let valid = valid_fence_fixture();
    let decorated = format!(
        r###"// pub struct CanonicalLookupKey {{ pub leaked: Definition }}
        /* {{ nested /* impl CanonicalLookupKey {{}} */ comment }} */
        const TEXT: &str = r#"pub struct Fake {{ pub body: Expr }}"#;
        {valid}"###
    );
    inspect_carrier_source(&decorated).expect("comments, raw strings, and braces are syntax-safe");

    let fields = [
        ("CanonicalDeclarationIdentity", "module_key: ModuleKey"),
        ("CanonicalLookupKey", "namespace: Namespace"),
        (
            "CanonicalCollectedEntry",
            "identity: CanonicalDeclarationIdentity",
        ),
        (
            "CanonicalProvisionalNameEntry",
            "identity: CanonicalDeclarationIdentity",
        ),
        (
            "CanonicalCollectedModuleSnapshot",
            "entries: Box<[CanonicalCollectedEntry]>",
        ),
        ("CanonicalProvisionalNameView", "entries: Vec<Opaque>"),
        ("CanonicalCollectedModuleRef", "collection: &'a Opaque"),
        (
            "CanonicalModuleCollection",
            "modules: BTreeMap<ModuleKey, CanonicalCollectedModule>",
        ),
    ];
    for (carrier, field) in fields {
        for visibility in ["pub", "pub(crate)", "pub(super)", "pub(in crate)"] {
            let visible = valid.replacen(field, &format!("{visibility} {field}"), 1);
            assert!(
                inspect_carrier_source(&visible).is_err(),
                "{carrier} {visibility}"
            );
        }
        for associated in [
            "pub const LEAK: usize = 0;",
            "pub(crate) fn from_parts() -> Self { unimplemented!() }",
        ] {
            let exposed = format!("{valid}\nimpl<'a> {carrier} {{ {associated} }}");
            assert!(
                inspect_carrier_source(&exposed).is_err(),
                "{carrier} {associated}"
            );
        }
    }
    assert!(
        inspect_carrier_source(&valid.replacen("lookup_name: Box<str>", "lookup_name: Score", 1))
            .is_err()
    );
    assert!(
        inspect_carrier_source(&valid.replacen(
            "lookup_name(&self) -> &str",
            "lookup_name(&self) -> &Score",
            1
        ))
        .is_err()
    );
    assert!(
        inspect_carrier_source(&valid.replacen(
            "raw_definition: Option<Definition>",
            "callable_body: Option<Definition>",
            1,
        ))
        .is_err()
    );
    for extra in [
        "raw_definition: Option<Definition>, callable_body: Option<Expr>",
        "raw_definition: Option<Definition>, second_definition: Option<Definition>",
    ] {
        assert!(
            inspect_carrier_source(&valid.replacen("raw_definition: Option<Definition>", extra, 1))
                .is_err()
        );
    }
    assert!(
        inspect_carrier_source(&valid.replacen(
            "entries: Box<[CanonicalCollectedEntry]>",
            "module_key: ModuleKey, entries: Box<[CanonicalCollectedEntry]>",
            1,
        ))
        .is_err()
    );
    assert!(
        inspect_carrier_source(&valid.replacen(
            "modules: BTreeMap<ModuleKey, CanonicalCollectedModule>",
            "internal_snapshots: BTreeMap<ModuleKey, Snapshot>, provisional_name_views: BTreeMap<ModuleKey, View>",
            1,
        ))
        .is_err()
    );
    for paired in [
        "internal_snapshot: Option<CanonicalCollectedModuleSnapshot>",
        "internal_snapshot: CanonicalCollectedModuleSnapshot, authority: FinalInterface",
    ] {
        assert!(
            inspect_carrier_source(&valid.replacen(
                "internal_snapshot: CanonicalCollectedModuleSnapshot",
                paired,
                1,
            ))
            .is_err()
        );
    }
    assert!(
        inspect_carrier_source(&valid.replacen(
            "provisional_name_view: CanonicalProvisionalNameView,",
            "",
            1,
        ))
        .is_err()
    );
}
