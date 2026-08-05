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
    Definition, ExpandedSurfaceOrigin, ExpansionId, Expr, IdentifierHygieneMetadata,
    NormalizedNotationPatternPart, NotationAssociativity, Visibility,
};
use ash_parser::{
    CanonicalExpandedModuleGraph, CanonicalModuleGraph, CanonicalModuleGraphResolver, Span,
};
use ash_typeck::canonical_module_collection::{
    CanonicalCollectedModuleRef, CanonicalCollectionDisposition, CanonicalCollectionRule,
    CanonicalDeclarationIdentity, CanonicalDeclarationKind, CanonicalDeclarationOriginKey,
    CanonicalLookupKey, CanonicalModuleCollection, CanonicalModuleCollectionError,
    CanonicalModuleCollectionErrorKind, CanonicalNamespace, CanonicalNotationFixity,
    collect_canonical_expanded_module_graph,
};
use ash_typeck::{
    CanonicalProvisionalModuleScopes, resolve_scoped_self_ordinary_function_imports_with_scopes,
    resolve_simple_parsed_imports_with_scopes,
};
use proptest::prelude::*;
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

fn expanded_source(label: &str, source: &str) -> (CanonicalExpandedModuleGraph, ModuleKey) {
    let tree = TempTree::new(label);
    let root_path = tree.write("src/main.ash", source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let parsed = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("fixture resolves through the canonical parser graph");
    let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect("fixture expands through the canonical expanded graph");
    (expanded, root_key)
}

fn parsed_source(label: &str, source: &str) -> (CanonicalModuleGraph, ModuleKey) {
    let tree = TempTree::new(label);
    let root_path = tree.write("src/main.ash", source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let parsed = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("fixture resolves through the canonical parser graph");
    (parsed, root_key)
}

fn expanded_file_backed_source(
    label: &str,
    root_source: &str,
    module_name: &str,
    module_source: &str,
) -> (CanonicalExpandedModuleGraph, ModuleKey) {
    let tree = TempTree::new(label);
    let root_path = tree.write("src/main.ash", root_source);
    tree.write(format!("src/{module_name}.ash"), module_source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let parsed = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("file-backed fixture resolves through the canonical parser graph");
    let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect("fixture expands through the canonical expanded graph");
    (expanded, root_key)
}

fn expanded_fixture() -> (CanonicalExpandedModuleGraph, ModuleKey) {
    expanded_source(
        "representative",
        r#"
            pub mod api {
                pub fn greet(value: Int) -> Int { value + 1 }
            }
            pub fn entry(value: Int) -> Int { api::greet(value) }
        "#,
    )
}

fn expect_collection_rule_error(
    expanded: &CanonicalExpandedModuleGraph,
    module_key: &ModuleKey,
    declaration_name: &str,
    kind: CanonicalModuleCollectionErrorKind,
    namespace: CanonicalNamespace,
    rule: CanonicalCollectionRule,
) -> CanonicalModuleCollectionError {
    let error = collect_canonical_expanded_module_graph(expanded)
        .expect_err("invalid collection input must publish no paired carriers");
    assert_eq!(error.kind(), kind);
    assert_eq!(error.module_key(), module_key);
    assert_eq!(error.declaration_name(), Some(declaration_name));
    assert_eq!(error.namespace(), Some(namespace));
    assert_eq!(error.rule(), Some(rule));
    error
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

    let mut collected_entry_accessors = Vec::new();
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
                        if matches!(
                            carrier,
                            "CanonicalCollectedEntry" | "CanonicalProvisionalNameEntry"
                        ) {
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
                            let accessor = (function.sig.ident.to_string(), type_shape(output)?);
                            if carrier == "CanonicalCollectedEntry" {
                                collected_entry_accessors.push(accessor);
                            } else {
                                provisional_accessors.push(accessor);
                            }
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
    collected_entry_accessors.sort_unstable();
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
            ("source_anchor".to_owned(), "Span".to_owned()),
        ]
    {
        return Err("collected entry field shape differs from exact contract".to_owned());
    }
    let mut expected_collected_entry_accessors = vec![
        ("callable_body".to_owned(), "Option<&Expr>".to_owned()),
        ("declared_name".to_owned(), "Option<&str>".to_owned()),
        (
            "identity".to_owned(),
            "&CanonicalDeclarationIdentity".to_owned(),
        ),
        ("kind".to_owned(), "CanonicalDeclarationKind".to_owned()),
        ("lookup_key".to_owned(), "&CanonicalLookupKey".to_owned()),
        ("namespace".to_owned(), "CanonicalNamespace".to_owned()),
        (
            "raw_definition".to_owned(),
            "Option<&Definition>".to_owned(),
        ),
        ("source_anchor".to_owned(), "Span".to_owned()),
    ];
    expected_collected_entry_accessors.sort_unstable();
    if collected_entry_accessors != expected_collected_entry_accessors {
        return Err("collected entry accessor shape differs from exact contract".to_owned());
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct NormalizedIdentity {
    module_key: ModuleKey,
    kind: CanonicalDeclarationKind,
    canonical_parent: Option<Box<Self>>,
}

fn normalized_identity(identity: &CanonicalDeclarationIdentity) -> NormalizedIdentity {
    NormalizedIdentity {
        module_key: identity.module_key().clone(),
        kind: identity.kind(),
        canonical_parent: identity
            .canonical_parent()
            .map(normalized_identity)
            .map(Box::new),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizedInternalEntry {
    identity: NormalizedIdentity,
    declared_name: Option<String>,
    lookup_key: CanonicalLookupKey,
    namespace: CanonicalNamespace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizedProvisionalEntry {
    identity: NormalizedIdentity,
    lookup_name: String,
    lookup_key: CanonicalLookupKey,
    namespace: CanonicalNamespace,
    visibility: Visibility,
    exportable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizedModuleProjection {
    module_key: ModuleKey,
    internal: Vec<NormalizedInternalEntry>,
    provisional: Vec<NormalizedProvisionalEntry>,
}

/// Project only Type-layer collection facts that are invariant between file and inline source
/// acquisition. Parser artifact paths, spans, raw definition payloads, source ordinals, and
/// expansion sidecars are deliberately omitted: those are provenance owned by the parser and
/// can differ while the canonical module key and declaration/name facts remain equal.
fn normalized_collection_projection(
    collection: &CanonicalModuleCollection,
) -> Vec<NormalizedModuleProjection> {
    let mut modules = collection
        .modules()
        .map(|module| {
            let mut internal = module
                .internal_snapshot()
                .entries()
                .map(|entry| NormalizedInternalEntry {
                    identity: normalized_identity(entry.identity()),
                    declared_name: entry.declared_name().map(str::to_owned),
                    lookup_key: entry.lookup_key().clone(),
                    namespace: entry.namespace(),
                })
                .collect::<Vec<_>>();
            let mut provisional = module
                .provisional_name_view()
                .entries()
                .map(|entry| NormalizedProvisionalEntry {
                    identity: normalized_identity(entry.identity()),
                    lookup_name: entry.lookup_name().to_owned(),
                    lookup_key: entry.lookup_key().clone(),
                    namespace: entry.namespace(),
                    visibility: entry.visibility().clone(),
                    exportable: entry.is_exportable(),
                })
                .collect::<Vec<_>>();
            internal.sort_by(|left, right| {
                (
                    left.lookup_key.clone(),
                    left.declared_name.clone(),
                    left.identity.clone(),
                )
                    .cmp(&(
                        right.lookup_key.clone(),
                        right.declared_name.clone(),
                        right.identity.clone(),
                    ))
            });
            provisional.sort_by(|left, right| {
                (
                    left.lookup_key.clone(),
                    left.lookup_name.clone(),
                    left.identity.clone(),
                )
                    .cmp(&(
                        right.lookup_key.clone(),
                        right.lookup_name.clone(),
                        right.identity.clone(),
                    ))
            });
            NormalizedModuleProjection {
                module_key: module.module_key().clone(),
                internal,
                provisional,
            }
        })
        .collect::<Vec<_>>();
    modules.sort_by(|left, right| left.module_key.cmp(&right.module_key));
    modules
}

fn generated_collection_graph(case: usize) -> (CanonicalExpandedModuleGraph, ModuleKey) {
    let name = format!("generated_{case}");
    let child_name = format!("child_{case}");
    let visibility = match case % 4 {
        0 => "pub",
        1 => "pub(crate)",
        _ => "",
    };
    let child_body = if case.is_multiple_of(3) {
        format!(
            "{visibility} fn child_fn_{case}() -> Int {{ 1 }} {visibility} type ChildType_{case} = Int;"
        )
    } else {
        format!(
            "{visibility} type ChildType_{case} = Left_{case} | Right_{case}; {visibility} fn child_fn_{case}() -> Int {{ 1 }}"
        )
    };
    let root_body = match case % 8 {
        0 => format!("{visibility} fn {name}() -> Int {{ 1 }}"),
        1 => format!("fn {name}() -> Int {{ 1 }}"),
        2 => format!(
            "{visibility} type Shared_{case} = Int; {visibility} fn Shared_{case}() -> Int {{ 1 }}"
        ),
        3 => format!("{visibility} type Parent_{case} = Left_{case} | Right_{case};"),
        4 => format!("{visibility} interface Show_{case}<T> {{ show(T) -> T }}"),
        5 => format!(
            "{visibility} interface Show_{case}<T> {{ show(T) -> T }} impl Show_{case}<Int> {{ show(value) = value }}"
        ),
        6 => format!(
            "{visibility} macro inc_{case}(x) => add(x, 1); {visibility} fn generated_{case}(value: Int) -> Int {{ inc_{case}!(value) }}"
        ),
        _ if case % 16 == 7 => format!(
            "{visibility} fn first_{case}() -> Int {{ 1 }} {visibility} fn second_{case}() -> Int {{ 2 }}"
        ),
        _ => format!(
            "{visibility} fn second_{case}() -> Int {{ 2 }} {visibility} fn first_{case}() -> Int {{ 1 }}"
        ),
    };
    let child_declaration = if case.is_multiple_of(2) {
        format!("pub mod {child_name} {{ {child_body} }}")
    } else {
        format!("pub mod {child_name};")
    };
    let root_source = format!("{child_declaration} {root_body}");
    if case.is_multiple_of(2) {
        expanded_source("generated-inline-collection", &root_source)
    } else {
        expanded_file_backed_source(
            "generated-file-collection",
            &root_source,
            &child_name,
            &child_body,
        )
    }
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
    collected
        .revalidate_against(&expanded)
        .expect("an unchanged expanded graph remains stable under revalidation");
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
    assert_eq!(
        function.source_anchor(),
        raw_function.span,
        "internal entries retain a direct source anchor alongside raw facts"
    );
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
fn inline_child_snapshot_retains_expanded_raw_callable_and_owns_its_sidecars() {
    let (expanded, root_key) = expanded_source(
        "inline-child-expanded-raw-facts",
        r#"
            pub mod child {
                pub macro inc(x) => add(x, 1);
                pub fn generated(value: Int) -> Int { inc!(value) }
            }
        "#,
    );
    let child_key = root_key.child("child").expect("child key is canonical");
    let expanded_child = expanded
        .module(&child_key)
        .expect("expanded graph contains the inline child");
    let expected_origins = expanded_child.origins().to_vec();
    let expected_hygiene = expanded_child.hygiene().to_vec();
    let expected_function = expanded_child
        .body()
        .definitions()
        .iter()
        .find(|definition| {
            matches!(definition, Definition::Function(function) if function.name.as_ref() == "generated")
        })
        .expect("expanded child retains its generated function")
        .clone();
    let Definition::Function(expected_function_shape) = &expected_function else {
        unreachable!("fixture lookup selected a function")
    };
    let Expr::Block {
        tail_expr: Some(expanded_tail),
        ..
    } = &expected_function_shape.body
    else {
        panic!("fixture function has an expanded block body")
    };
    assert!(
        matches!(expanded_tail.as_ref(), Expr::Call { func, .. } if func.as_ref() == "add"),
        "the fixture must reach collection only after macro invocation expansion"
    );
    assert!(!expected_origins.is_empty());
    assert!(!expected_hygiene.is_empty());

    let collected = collect_canonical_expanded_module_graph(&expanded)
        .expect("expanded inline child collects into paired carriers");
    let root_snapshot = collected
        .internal_snapshot(&root_key)
        .expect("root snapshot is published");
    assert!(root_snapshot.expansion_origins().is_empty());
    assert!(root_snapshot.hygiene().is_empty());

    let child_snapshot = collected
        .internal_snapshot(&child_key)
        .expect("child snapshot is published");
    assert_eq!(child_snapshot.expansion_origins(), expected_origins);
    assert_eq!(child_snapshot.hygiene(), expected_hygiene);
    let internal = child_snapshot
        .entries()
        .find(|entry| entry.declared_name() == Some("generated"))
        .expect("expanded callable is retained internally");
    assert_eq!(internal.raw_definition(), Some(&expected_function));
    assert_eq!(internal.source_anchor(), expected_function_shape.span);
    assert_eq!(
        internal.callable_body(),
        Some(&expected_function_shape.body),
        "the internal carrier retains the expanded callable body"
    );
    assert_eq!(origin_source_ordinal(internal.identity().origin_key()), 1);

    let name = collected
        .provisional_name_view(&child_key)
        .expect("child name view is published")
        .entries()
        .find(|entry| entry.lookup_name() == "generated")
        .expect("public callable is provisionally visible");
    assert_eq!(name.identity(), internal.identity());
    assert_eq!(name.lookup_key(), internal.lookup_key());
    assert_eq!(name.namespace(), internal.namespace());
    assert_eq!(name.visibility(), &expected_function_shape.visibility);
    assert!(name.is_exportable());
    assert_eq!(name.origin_anchor(), expected_function_shape.span);
    assert_eq!(name.source_ordinal(), 1);
}

#[test]
fn collection_classifies_structural_type_interface_and_value_namespaces() {
    let (expanded, root_key) = expanded_source(
        "namespace-classification",
        r#"
            pub mod child {}
            pub type Choice = Pick | Skip;
            pub interface Show<T> {}
            pub fn run(value: Int) -> Int { value }
        "#,
    );
    let collected = collect_canonical_expanded_module_graph(&expanded)
        .expect("valid declarations collect into paired views");
    let view = collected
        .provisional_name_view(&root_key)
        .expect("root name view is published");
    let snapshot = collected
        .internal_snapshot(&root_key)
        .expect("root internal snapshot is published");
    for (name, namespace) in [
        ("child", CanonicalNamespace::StructuralModule),
        ("Choice", CanonicalNamespace::TypeDomain),
        ("Pick", CanonicalNamespace::ValueCallable),
        ("Skip", CanonicalNamespace::ValueCallable),
        ("Show", CanonicalNamespace::Interface),
        ("run", CanonicalNamespace::ValueCallable),
    ] {
        assert!(
            view.entries()
                .any(|entry| entry.lookup_name() == name && entry.namespace() == namespace),
            "missing {namespace:?} entry {name}"
        );
    }
    let internal_module = snapshot
        .entries()
        .find(|entry| entry.declared_name() == Some("child"))
        .expect("module declaration is retained internally");
    let provisional_module = view
        .entries()
        .find(|entry| entry.lookup_name() == "child")
        .expect("module declaration is visible provisionally");
    assert_eq!(internal_module.kind(), CanonicalDeclarationKind::ModuleDecl);
    assert_eq!(internal_module.raw_definition(), None);
    assert_eq!(
        internal_module.source_anchor(),
        provisional_module.origin_anchor(),
        "structural module entries retain their source anchor without a raw definition"
    );
}

#[test]
fn duplicate_spelling_in_one_namespace_rejects_without_publication() {
    let (expanded, root_key) = expanded_source(
        "same-bucket-duplicate",
        "pub fn duplicate() -> Int { 1 } pub fn duplicate() -> Int { 2 }",
    );

    let error = expect_collection_rule_error(
        &expanded,
        &root_key,
        "duplicate",
        CanonicalModuleCollectionErrorKind::DuplicateLookupKey,
        CanonicalNamespace::ValueCallable,
        CanonicalCollectionRule::DuplicateLookupKey,
    );
    assert!(error.canonical_parent().is_none());
}

#[test]
fn same_spelling_across_buckets_is_preserved_as_contextual_ambiguity() {
    let (expanded, root_key) = expanded_source(
        "cross-bucket-spelling",
        "pub type Shared = SharedValue; pub fn Shared() -> Int { 1 }",
    );
    let collected = collect_canonical_expanded_module_graph(&expanded)
        .expect("different namespace buckets may retain the same spelling");
    let mut namespaces = collected
        .provisional_name_view(&root_key)
        .expect("root name view is published")
        .entries()
        .filter(|entry| entry.lookup_name() == "Shared")
        .map(|entry| entry.namespace())
        .collect::<Vec<_>>();
    namespaces.sort_unstable();
    assert_eq!(
        namespaces,
        [
            CanonicalNamespace::TypeDomain,
            CanonicalNamespace::ValueCallable
        ]
    );
}

#[test]
fn constructors_are_scoped_by_canonical_parent_and_semantic_level() {
    let (expanded, root_key) = expanded_source(
        "parent-scoped-constructors",
        r#"
            pub type Nat = Z | S(Nat);
            pub data kind NatKind from type Nat;
            pub sealed type domain Closed { Hidden; }
        "#,
    );
    let collected = collect_canonical_expanded_module_graph(&expanded)
        .expect("constructor parents and levels collect canonically");
    let snapshot = collected
        .internal_snapshot(&root_key)
        .expect("root internal snapshot is published");

    for (name, namespace, entry_kind, parent_kind) in [
        (
            "Z",
            CanonicalNamespace::ValueCallable,
            CanonicalDeclarationKind::Function,
            CanonicalDeclarationKind::Type,
        ),
        (
            "Z",
            CanonicalNamespace::PromotedKind,
            CanonicalDeclarationKind::DataKind,
            CanonicalDeclarationKind::DataKind,
        ),
        (
            "Hidden",
            CanonicalNamespace::TypeDomain,
            CanonicalDeclarationKind::SealedDomain,
            CanonicalDeclarationKind::SealedDomain,
        ),
    ] {
        let entry = snapshot
            .entries()
            .find(|entry| entry.declared_name() == Some(name) && entry.namespace() == namespace)
            .unwrap_or_else(|| panic!("missing parent-scoped {namespace:?} entry {name}"));
        assert_eq!(entry.kind(), entry_kind);
        assert_eq!(
            entry
                .identity()
                .canonical_parent()
                .map(|parent| parent.kind()),
            Some(parent_kind)
        );
    }
    assert!(
        collected
            .provisional_name_view(&root_key)
            .expect("root name view is published")
            .entries()
            .all(|entry| {
                entry.lookup_name() != "Hidden"
                    || entry.namespace() != CanonicalNamespace::ValueCallable
            }),
        "sealed constructors must not become standalone values"
    );
    let ordinary = snapshot
        .entries()
        .filter(|entry| {
            matches!(entry.declared_name(), Some("Z" | "S"))
                && entry.namespace() == CanonicalNamespace::ValueCallable
        })
        .collect::<Vec<_>>();
    assert_eq!(ordinary.len(), 2);
    assert_eq!(
        ordinary[0].identity().canonical_parent(),
        ordinary[1].identity().canonical_parent()
    );
    assert_ne!(ordinary[0].identity(), ordinary[1].identity());
}

#[test]
fn type_alias_does_not_synthesize_a_value_constructor() {
    let (expanded, root_key) = expanded_source("alias-no-constructor", "pub type Alias = Int;");
    let collected = collect_canonical_expanded_module_graph(&expanded)
        .expect("a type alias collects without inventing a constructor");
    let snapshot = collected
        .internal_snapshot(&root_key)
        .expect("root internal snapshot is published");
    assert_eq!(snapshot.entries().count(), 1);
    assert!(snapshot.entries().all(|entry| {
        entry.declared_name() != Some("Int")
            && entry.namespace() != CanonicalNamespace::ValueCallable
    }));
}

#[test]
fn newtype_constructor_is_a_callable_function_under_its_newtype_parent() {
    let (expanded, root_key) = expanded_source(
        "newtype-constructor-kind",
        "pub newtype UserId = UserId(Int);",
    );
    let collected = collect_canonical_expanded_module_graph(&expanded)
        .expect("newtype and its constructor collect");
    let constructor = collected
        .internal_snapshot(&root_key)
        .expect("root internal snapshot is published")
        .entries()
        .find(|entry| {
            entry.declared_name() == Some("UserId")
                && entry.namespace() == CanonicalNamespace::ValueCallable
        })
        .expect("newtype constructor is collected in the callable bucket");
    assert_eq!(constructor.kind(), CanonicalDeclarationKind::Function);
    assert_eq!(
        constructor
            .identity()
            .canonical_parent()
            .map(|parent| parent.kind()),
        Some(CanonicalDeclarationKind::Newtype)
    );
}

#[test]
fn impl_members_are_internal_and_never_enter_the_provisional_view() {
    let (expanded, root_key) = expanded_source(
        "impl-member-visibility",
        r#"
            interface Show<T> { show(T) -> T }
            impl Show<Int> { show(value) = value }
        "#,
    );
    let collected = collect_canonical_expanded_module_graph(&expanded)
        .expect("impl members collect as internal parent-scoped facts");
    let member = collected
        .internal_snapshot(&root_key)
        .expect("root internal snapshot is published")
        .entries()
        .find(|entry| {
            entry.declared_name() == Some("show")
                && entry
                    .identity()
                    .canonical_parent()
                    .is_some_and(|parent| parent.kind() == CanonicalDeclarationKind::Impl)
        })
        .expect("impl method is retained internally");
    assert!(
        collected
            .provisional_name_view(&root_key)
            .expect("root name view is published")
            .entries()
            .all(|entry| entry.identity() != member.identity()),
        "impl members never become provisional import authority"
    );
}

#[test]
fn imported_interface_impl_is_deferred_and_remains_internal_only() {
    let (expanded, root_key) = expanded_source(
        "imported-interface-impl-deferral",
        r#"
            pub mod provider {
                pub interface Show {}
            }
            pub mod api {
                use crate::provider::Show;
                pub impl Show {
                    show(value) = value
                }
            }
        "#,
    );
    let collected = collect_canonical_expanded_module_graph(&expanded)
        .expect("an imported interface implementation is deferred for finalization");
    let api_key = root_key.child("api").expect("api module key is canonical");
    let snapshot = collected
        .internal_snapshot(&api_key)
        .expect("api internal snapshot is published");
    let implementation = snapshot
        .entries()
        .find(|entry| {
            entry.kind() == CanonicalDeclarationKind::Impl && entry.declared_name() == Some("Show")
        })
        .expect("imported interface implementation is retained internally");
    assert!(
        collected
            .provisional_name_view(&api_key)
            .expect("api provisional name view is published")
            .entries()
            .all(|entry| entry.identity().canonical_parent() != Some(implementation.identity())),
        "implementation members remain out of the provisional name view"
    );
}

#[test]
fn nested_interface_and_impl_members_retain_raw_shapes_spans_bodies_and_ordinals() {
    let (expanded, root_key) = expanded_source(
        "nested-member-raw-facts",
        r#"
            pub interface Show<T> {
                show(T) -> T
                hide(T) -> T
                law stable(value: T): value == value
                law symmetric(left: T, right: T): left == right
            }
            impl Show<Int> {
                show(value) = value
                hide(value) = value
                proof stable(value: Int) { by_definition }
                proof symmetric(left: Int, right: Int) { by_definition }
            }
        "#,
    );
    let collected = collect_canonical_expanded_module_graph(&expanded)
        .expect("nested interface and impl members collect");
    let snapshot = collected
        .internal_snapshot(&root_key)
        .expect("root internal snapshot is published");
    let name_view = collected
        .provisional_name_view(&root_key)
        .expect("root name view is published");

    for (name, member_ordinal, expected_kind, expected_namespace) in [
        (
            "show",
            0,
            CanonicalDeclarationKind::Function,
            CanonicalNamespace::ValueCallable,
        ),
        (
            "hide",
            1,
            CanonicalDeclarationKind::Function,
            CanonicalNamespace::ValueCallable,
        ),
        (
            "stable",
            0,
            CanonicalDeclarationKind::Law,
            CanonicalNamespace::Evidence,
        ),
        (
            "symmetric",
            1,
            CanonicalDeclarationKind::Law,
            CanonicalNamespace::Evidence,
        ),
    ] {
        let internal = snapshot
            .entries()
            .find(|entry| {
                entry.declared_name() == Some(name)
                    && entry.kind() == expected_kind
                    && entry
                        .identity()
                        .canonical_parent()
                        .is_some_and(|parent| parent.kind() == CanonicalDeclarationKind::Interface)
            })
            .unwrap_or_else(|| panic!("missing internal interface member {name}"));
        assert_eq!(internal.namespace(), expected_namespace);
        assert_eq!(
            origin_source_ordinal(internal.identity().origin_key()),
            member_ordinal
        );
        let Definition::Interface(raw_interface) = internal
            .raw_definition()
            .expect("interface member retains its expanded raw parent declaration")
        else {
            panic!("interface member must retain an interface shape")
        };
        let expected_span = raw_interface
            .methods
            .iter()
            .find(|method| method.name.as_ref() == name)
            .map(|method| method.span)
            .or_else(|| {
                raw_interface
                    .laws
                    .iter()
                    .find(|law| law.name.as_ref() == name)
                    .map(|law| law.span)
            })
            .expect("raw interface contains the matching member span");
        assert_ne!(expected_span.start, expected_span.end);
        assert_eq!(internal.source_anchor(), expected_span);

        let provisional = name_view
            .entries()
            .find(|entry| entry.identity() == internal.identity())
            .expect("interface member identity is mirrored into the name-only view");
        assert_eq!(provisional.lookup_key(), internal.lookup_key());
        assert_eq!(provisional.namespace(), internal.namespace());
        assert_eq!(provisional.visibility(), &Visibility::Inherited);
        assert!(!provisional.is_exportable());
        assert_eq!(provisional.origin_anchor(), internal.source_anchor());
        assert_eq!(provisional.source_ordinal(), member_ordinal);
    }

    for (name, member_ordinal, expected_kind) in [
        ("show", 0, CanonicalDeclarationKind::Function),
        ("hide", 1, CanonicalDeclarationKind::Function),
        ("stable", 0, CanonicalDeclarationKind::Proof),
        ("symmetric", 1, CanonicalDeclarationKind::Proof),
    ] {
        let internal = snapshot
            .entries()
            .find(|entry| {
                entry.declared_name() == Some(name)
                    && entry.kind() == expected_kind
                    && entry
                        .identity()
                        .canonical_parent()
                        .is_some_and(|parent| parent.kind() == CanonicalDeclarationKind::Impl)
            })
            .unwrap_or_else(|| panic!("missing internal impl member {name}"));
        assert_eq!(
            origin_source_ordinal(internal.identity().origin_key()),
            member_ordinal
        );
        let Definition::Impl(raw_impl) = internal
            .raw_definition()
            .expect("impl member retains its expanded raw parent declaration")
        else {
            panic!("impl member must retain an impl shape")
        };
        let expected_span = raw_impl
            .methods
            .iter()
            .find(|method| method.name.as_ref() == name)
            .map(|method| {
                assert!(
                    matches!(&method.body, Expr::Variable { name, .. } if name.as_ref() == "value"),
                    "raw impl method body is retained"
                );
                method.span
            })
            .or_else(|| {
                raw_impl
                    .proofs
                    .iter()
                    .find(|proof| proof.name.as_ref() == name)
                    .map(|proof| proof.span)
            })
            .expect("raw impl contains the matching member span");
        assert_ne!(expected_span.start, expected_span.end);
        assert_eq!(internal.source_anchor(), expected_span);
        assert!(
            name_view
                .entries()
                .all(|entry| entry.identity() != internal.identity()),
            "impl member raw facts stay out of the provisional view"
        );
    }
}

#[test]
fn syntax_role_and_evidence_declarations_use_their_required_namespaces() {
    let (expanded, root_key) = expanded_source(
        "remaining-namespaces",
        r#"
            pub fn combine(left: Int, right: Int) -> Int { left + right }
            pub infixl 6 <+> = combine
            pub role reviewer { capabilities: [] }
            pub law reflexive(x: Int): x == x
            pub proof witness(x: Int) { by_definition }
        "#,
    );
    let collected = collect_canonical_expanded_module_graph(&expanded)
        .expect("syntax, role, and evidence declarations collect");
    let view = collected
        .provisional_name_view(&root_key)
        .expect("root name view is published");
    let notation = view
        .entries()
        .find(|entry| entry.namespace() == CanonicalNamespace::Notation)
        .expect("notation entry uses its dedicated namespace");
    let notation_key = notation
        .lookup_key()
        .notation_key()
        .expect("notation lookup retains a typed key");
    assert!(matches!(
        notation_key.pattern().parts(),
        [NormalizedNotationPatternPart::Token(spelling)] if spelling.as_ref() == "<+>"
    ));
    assert_eq!(
        notation_key.fixity(),
        CanonicalNotationFixity::Infix {
            associativity: NotationAssociativity::Left,
            precedence: 6,
        }
    );
    assert!(notation.lookup_name().contains("<+>"));
    assert!(notation.lookup_name().contains("precedence: 6"));
    for (name, namespace) in [
        ("reviewer", CanonicalNamespace::Role),
        ("reflexive", CanonicalNamespace::Evidence),
        ("witness", CanonicalNamespace::Evidence),
    ] {
        assert!(
            view.entries()
                .any(|entry| entry.lookup_name() == name && entry.namespace() == namespace),
            "missing {namespace:?} entry {name}"
        );
    }
    assert_eq!(
        CanonicalDeclarationKind::Policy.collection_disposition(),
        CanonicalCollectionDisposition::Collect {
            namespace: CanonicalNamespace::Policy,
            publish_in_name_view: true,
        },
        "policy classification remains explicit despite lacking active source grammar"
    );
}

#[test]
fn same_constructor_spelling_is_allowed_under_distinct_parents() {
    let (expanded, root_key) = expanded_source(
        "same-constructor-spelling",
        r#"
            pub type Left = Same | LeftOnly;
            pub type Right = Same | RightOnly;
        "#,
    );
    let collected = collect_canonical_expanded_module_graph(&expanded)
        .expect("constructors collide only within their canonical parent");
    let snapshot = collected
        .internal_snapshot(&root_key)
        .expect("root internal snapshot is published");
    let parents = snapshot
        .entries()
        .filter(|entry| {
            entry.declared_name() == Some("Same")
                && entry.namespace() == CanonicalNamespace::ValueCallable
        })
        .map(|entry| {
            entry
                .identity()
                .canonical_parent()
                .expect("constructor retains its canonical parent")
        })
        .collect::<Vec<_>>();
    assert_eq!(parents.len(), 2);
    assert_eq!(parents[0].kind(), CanonicalDeclarationKind::Type);
    assert_eq!(parents[1].kind(), CanonicalDeclarationKind::Type);
    assert_ne!(parents[0], parents[1]);
}

#[test]
fn same_interface_member_spelling_is_allowed_under_distinct_parents() {
    let (expanded, root_key) = expanded_source(
        "same-member-spelling",
        r#"
            pub interface First<T> { same(T) -> T }
            pub interface Second<T> { same(T) -> T }
        "#,
    );
    let collected = collect_canonical_expanded_module_graph(&expanded)
        .expect("members collide only within their canonical parent");
    let snapshot = collected
        .internal_snapshot(&root_key)
        .expect("root internal snapshot is published");
    let parents = snapshot
        .entries()
        .filter(|entry| {
            entry.declared_name() == Some("same")
                && entry.namespace() == CanonicalNamespace::ValueCallable
        })
        .map(|entry| {
            entry
                .identity()
                .canonical_parent()
                .expect("member retains its canonical parent")
        })
        .collect::<Vec<_>>();
    assert_eq!(parents.len(), 2);
    assert_eq!(parents[0].kind(), CanonicalDeclarationKind::Interface);
    assert_eq!(parents[1].kind(), CanonicalDeclarationKind::Interface);
    assert_ne!(parents[0], parents[1]);
}

#[test]
fn distinct_full_interface_applications_do_not_overlap() {
    let (distinct, root_key) = expanded_source(
        "distinct-impl-heads",
        "interface Show<T> {} impl Show<Int> {} impl Show<String> {}",
    );
    let collected = collect_canonical_expanded_module_graph(&distinct)
        .expect("distinct full interface applications do not overlap");
    assert_eq!(
        collected
            .internal_snapshot(&root_key)
            .expect("root internal snapshot is published")
            .entries()
            .filter(|entry| entry.kind() == CanonicalDeclarationKind::Impl)
            .count(),
        2
    );
    assert!(
        collected
            .provisional_name_view(&root_key)
            .expect("root name view is published")
            .entries()
            .all(|entry| entry.namespace() != CanonicalNamespace::ImplementationRegistry)
    );
}

#[test]
fn duplicate_full_interface_application_rejects_as_overlap() {
    let (overlap, overlap_key) = expanded_source(
        "overlapping-impl-heads",
        "interface Show<T> {} impl Show<Int> {} impl Show<Int> {}",
    );
    let error = expect_collection_rule_error(
        &overlap,
        &overlap_key,
        "Show",
        CanonicalModuleCollectionErrorKind::OverlappingImplementation,
        CanonicalNamespace::ImplementationRegistry,
        CanonicalCollectionRule::ImplOverlap,
    );
    assert!(error.canonical_parent().is_none());
}

#[test]
fn generic_impl_head_overlaps_its_concrete_full_application() {
    let (expanded, root_key) = expanded_source(
        "generic-impl-overlap",
        "interface Pair<A, B> {} impl<T> Pair<List<T>, String> {} impl Pair<List<Int>, String> {}",
    );

    let error = expect_collection_rule_error(
        &expanded,
        &root_key,
        "Pair",
        CanonicalModuleCollectionErrorKind::OverlappingImplementation,
        CanonicalNamespace::ImplementationRegistry,
        CanonicalCollectionRule::ImplOverlap,
    );
    assert!(error.canonical_parent().is_none());
}

#[test]
fn alpha_renamed_permuted_computation_rows_overlap_in_full_impl_heads() {
    let (expanded, root_key) = expanded_source(
        "computation-row-impl-overlap",
        r#"
            interface Handles<F> {}
            impl<r> Handles<(Int) -> {Fs::read, Clock::tick | r} String> {}
            impl<s> Handles<(Int) -> {Clock::tick, Fs::read | s} String> {}
        "#,
    );

    let error = expect_collection_rule_error(
        &expanded,
        &root_key,
        "Handles",
        CanonicalModuleCollectionErrorKind::OverlappingImplementation,
        CanonicalNamespace::ImplementationRegistry,
        CanonicalCollectionRule::ImplOverlap,
    );
    assert!(error.canonical_parent().is_none());
}

#[test]
fn open_computation_row_impl_overlaps_matching_closed_row_extension() {
    let (expanded, root_key) = expanded_source(
        "open-closed-row-impl-overlap",
        r#"
            interface Handles<F> {}
            impl<r> Handles<(Int) -> {Fs::read | r} String> {}
            impl Handles<(Int) -> {Fs::read, Clock::tick} String> {}
        "#,
    );

    let error = expect_collection_rule_error(
        &expanded,
        &root_key,
        "Handles",
        CanonicalModuleCollectionErrorKind::OverlappingImplementation,
        CanonicalNamespace::ImplementationRegistry,
        CanonicalCollectionRule::ImplOverlap,
    );
    assert!(error.canonical_parent().is_none());
}

#[test]
fn unresolved_impl_interface_identity_rejects_without_spelling_fallback() {
    let (expanded, root_key) = expanded_source("unresolved-impl-interface", "impl Missing<Int> {}");
    let expected_span = expanded
        .module(&root_key)
        .expect("root module exists")
        .body()
        .definitions()
        .iter()
        .find_map(|definition| match definition {
            Definition::Impl(implementation) => Some(implementation.span),
            _ => None,
        })
        .expect("fixture contains the unresolved impl");

    let error = expect_collection_rule_error(
        &expanded,
        &root_key,
        "Missing",
        CanonicalModuleCollectionErrorKind::InterfaceIdentityUnavailable,
        CanonicalNamespace::ImplementationRegistry,
        CanonicalCollectionRule::InterfaceIdentityUnavailable,
    );
    assert!(error.canonical_parent().is_none());
    assert_eq!(error.declaration_span(), expected_span);
}

#[test]
fn duplicate_impl_head_across_sibling_modules_rejects_graph_wide_at_later_impl() {
    let (expanded, root_key) = expanded_source(
        "cross-module-impl-overlap",
        r#"
            interface Show<T> {}
            pub mod alpha { impl Show<Int> {} }
            pub mod omega { impl Show<Int> {} }
        "#,
    );
    let omega_key = root_key.child("omega").expect("child key is canonical");
    let expected_span = expanded
        .module(&omega_key)
        .expect("later impl module exists")
        .body()
        .definitions()
        .iter()
        .find_map(|definition| match definition {
            Definition::Impl(implementation) => Some(implementation.span),
            _ => None,
        })
        .expect("later module contains the overlapping impl");

    let error = expect_collection_rule_error(
        &expanded,
        &omega_key,
        "Show",
        CanonicalModuleCollectionErrorKind::OverlappingImplementation,
        CanonicalNamespace::ImplementationRegistry,
        CanonicalCollectionRule::ImplOverlap,
    );
    assert!(error.canonical_parent().is_none());
    assert_eq!(error.declaration_span(), expected_span);
}

#[test]
fn failing_child_discards_valid_sibling_and_all_paired_publication() {
    let (expanded, root_key) = expanded_source(
        "atomic-sibling-failure",
        r#"
            pub mod healthy { pub fn okay() -> Int { 1 } }
            pub mod broken {
                pub fn clash() -> Int { 1 }
                pub fn clash() -> Int { 2 }
            }
        "#,
    );
    let broken_key = root_key.child("broken").expect("child key is canonical");

    let error = expect_collection_rule_error(
        &expanded,
        &broken_key,
        "clash",
        CanonicalModuleCollectionErrorKind::DuplicateLookupKey,
        CanonicalNamespace::ValueCallable,
        CanonicalCollectionRule::DuplicateLookupKey,
    );
    assert!(error.canonical_parent().is_none());
    assert_ne!(error.module_key(), &root_key);
}

fn candidate_declaration_span(
    candidate: &CanonicalExpandedModuleGraph,
    module_key: &ModuleKey,
    declaration_name: &str,
) -> Span {
    candidate
        .module(module_key)
        .expect("candidate graph contains the changed module")
        .body()
        .definitions()
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(definition) if definition.name.as_ref() == declaration_name => {
                Some(definition.span)
            }
            Definition::Handler(definition) if definition.name.as_ref() == declaration_name => {
                Some(definition.span)
            }
            _ => None,
        })
        .expect("candidate declaration retains a source anchor")
}

fn expect_source_drift(
    baseline: &CanonicalExpandedModuleGraph,
    candidate: &CanonicalExpandedModuleGraph,
    module_key: &ModuleKey,
    declaration_name: &str,
) {
    let collection = collect_canonical_expanded_module_graph(baseline)
        .expect("baseline graph collects before candidate revalidation");
    collect_canonical_expanded_module_graph(candidate)
        .expect("a changed source graph remains independently collectible");
    let expected_span = candidate_declaration_span(candidate, module_key, declaration_name);
    let error = collection
        .revalidate_against(candidate)
        .expect_err("source drift must reject before a replacement pair is published");
    assert_eq!(
        error.kind(),
        CanonicalModuleCollectionErrorKind::SourceDrift
    );
    assert_eq!(error.rule(), Some(CanonicalCollectionRule::SourceDrift));
    assert_eq!(error.module_key(), module_key);
    assert_eq!(error.declaration_name(), Some(declaration_name));
    assert_eq!(error.declaration_span(), expected_span);
}

#[test]
fn task_7_name_drift_rejects_during_candidate_revalidation() {
    let (baseline, root_key) =
        expanded_source("drift-name-baseline", "pub fn entry() -> Int { 1 }");
    let (candidate, _) = expanded_source("drift-name-candidate", "pub fn renamed() -> Int { 1 }");
    expect_source_drift(&baseline, &candidate, &root_key, "renamed");
}

#[test]
fn task_7_kind_drift_rejects_during_candidate_revalidation() {
    let (baseline, root_key) =
        expanded_source("drift-kind-baseline", "pub fn entry() -> Int { 1 }");
    let (candidate, _) = expanded_source(
        "drift-kind-candidate",
        "pub handler entry(comp: () -> {} Int) -> Int { 1 }",
    );
    expect_source_drift(&baseline, &candidate, &root_key, "entry");
}

#[test]
fn task_7_visibility_drift_rejects_during_candidate_revalidation() {
    let (baseline, root_key) =
        expanded_source("drift-visibility-baseline", "fn entry() -> Int { 1 }");
    let (candidate, _) =
        expanded_source("drift-visibility-candidate", "pub fn entry() -> Int { 1 }");
    expect_source_drift(&baseline, &candidate, &root_key, "entry");
}

#[test]
fn task_7_signature_drift_rejects_during_candidate_revalidation() {
    let (baseline, root_key) = expanded_source(
        "drift-signature-baseline",
        "pub fn entry(value: Int) -> Int { value }",
    );
    let (candidate, _) = expanded_source(
        "drift-signature-candidate",
        "pub fn entry(value: String) -> Int { 1 }",
    );
    expect_source_drift(&baseline, &candidate, &root_key, "entry");
}

#[test]
fn task_7_body_drift_rejects_during_candidate_revalidation() {
    let (baseline, root_key) =
        expanded_source("drift-body-baseline", "pub fn entry() -> Int { 1 }");
    let (candidate, _) = expanded_source("drift-body-candidate", "pub fn entry() -> Int { 2 }");
    expect_source_drift(&baseline, &candidate, &root_key, "entry");
}

#[test]
fn task_7_source_order_drift_rejects_during_candidate_revalidation() {
    let (baseline, root_key) = expanded_source(
        "drift-order-baseline",
        "pub fn first() -> Int { 1 } pub fn second() -> Int { 2 }",
    );
    let (candidate, _) = expanded_source(
        "drift-order-candidate",
        "pub fn second() -> Int { 2 } pub fn first() -> Int { 1 }",
    );
    expect_source_drift(&baseline, &candidate, &root_key, "second");
}

#[test]
fn task_7_expansion_sidecar_drift_rejects_during_candidate_revalidation() {
    let (baseline, root_key) = expanded_source(
        "drift-sidecar-baseline",
        r#"
            pub macro inc(x) => add(x, 1);
            pub fn generated(value: Int) -> Int { inc!(value) }
        "#,
    );
    let (candidate, _) = expanded_source(
        "drift-sidecar-candidate",
        r#"
            pub macro inc(x) => add(x, 1);

            pub fn generated(value: Int) -> Int { inc!(value) }
        "#,
    );
    let baseline_module = baseline.module(&root_key).expect("baseline root exists");
    let candidate_module = candidate.module(&root_key).expect("candidate root exists");
    assert_ne!(baseline_module.origins(), candidate_module.origins());
    assert_ne!(baseline_module.hygiene(), candidate_module.hygiene());
    let collection = collect_canonical_expanded_module_graph(&baseline)
        .expect("baseline graph collects before candidate revalidation");
    collect_canonical_expanded_module_graph(&candidate)
        .expect("candidate graph remains independently collectible");
    let error = collection
        .revalidate_against(&candidate)
        .expect_err("sidecar drift must reject before replacement publication");
    assert_eq!(
        error.kind(),
        CanonicalModuleCollectionErrorKind::SourceDrift
    );
    assert_eq!(error.rule(), Some(CanonicalCollectionRule::SourceDrift));
    assert_eq!(error.module_key(), &root_key);
    assert!(error.declaration_name().is_some());
    assert_ne!(error.declaration_span(), Span::default());
}

#[test]
fn task_7_valid_sibling_remains_unpublished_when_candidate_sibling_drifts() {
    let (baseline, root_key) = expanded_source(
        "drift-sibling-baseline",
        r#"
            pub mod healthy { pub fn okay() -> Int { 1 } }
            pub mod changed { pub fn stable() -> Int { 1 } }
        "#,
    );
    let (candidate, _) = expanded_source(
        "drift-sibling-candidate",
        r#"
            pub mod healthy { pub fn okay() -> Int { 1 } }
            pub mod changed { pub fn stable() -> Int { 2 } }
        "#,
    );
    let changed_key = root_key.child("changed").expect("changed key is canonical");
    let collection = collect_canonical_expanded_module_graph(&baseline)
        .expect("baseline graph collects before candidate revalidation");
    collect_canonical_expanded_module_graph(&candidate)
        .expect("candidate graph remains independently collectible");
    let error = collection
        .revalidate_against(&candidate)
        .expect_err("changed sibling must discard replacement publication atomically");
    assert_eq!(
        error.kind(),
        CanonicalModuleCollectionErrorKind::SourceDrift
    );
    assert_eq!(error.rule(), Some(CanonicalCollectionRule::SourceDrift));
    assert_eq!(error.module_key(), &changed_key);
    assert_eq!(error.declaration_name(), Some("stable"));
    assert_ne!(error.declaration_span(), Span::default());
    assert!(collection.internal_snapshot(&root_key).is_some());
    assert!(collection.provisional_name_view(&root_key).is_some());
    assert!(collection.internal_snapshot(&changed_key).is_some());
    assert!(collection.provisional_name_view(&changed_key).is_some());
}

#[test]
fn carrier_source_fence_enforces_private_construction_and_exact_name_view() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/canonical_module_collection.rs");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    inspect_carrier_source(&source).unwrap_or_else(|error| panic!("carrier fence: {error}"));
}

#[test]
fn task_8_file_and_inline_collections_have_equal_normalized_type_projection() {
    let inline_source = r#"
        pub mod child {
            pub type Child = Left | Right;
            pub fn make(value: Int) -> Int { value }
        }
        pub type Root = Int;
        pub fn entry(value: Int) -> Int { value }
    "#;
    let (inline, inline_root) = expanded_source("file-inline-normalized-inline", inline_source);
    let (file, file_root) = expanded_file_backed_source(
        "file-inline-normalized-file",
        "pub mod child; pub type Root = Int; pub fn entry(value: Int) -> Int { value }",
        "child",
        "pub type Child = Left | Right; pub fn make(value: Int) -> Int { value }",
    );
    assert_eq!(inline_root, file_root);

    let inline_collection = collect_canonical_expanded_module_graph(&inline)
        .expect("inline source collects into paired views");
    let file_collection = collect_canonical_expanded_module_graph(&file)
        .expect("file-backed source collects into paired views");
    assert_eq!(
        normalized_collection_projection(&inline_collection),
        normalized_collection_projection(&file_collection),
        "Type-layer collection facts must be independent of file-versus-inline acquisition; spans, raw definitions, source ordinals, and parser sidecars are provenance-only",
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn task_8_generated_collection_witness_preserves_two_tier_identity_and_namespace_facts(
        case in 0_usize..32,
    ) {
        let (expanded, root_key) = generated_collection_graph(case);
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("generated supported declaration graph collects atomically");
        let projection = normalized_collection_projection(&collection);
        prop_assert!(!projection.is_empty());

        for module in collection.modules() {
            let internal = module.internal_snapshot().entries().collect::<Vec<_>>();
            let provisional = module.provisional_name_view().entries().collect::<Vec<_>>();
            for entry in provisional {
                let matching = internal.iter().find(|candidate| {
                    candidate.identity() == entry.identity()
                        && candidate.lookup_key() == entry.lookup_key()
                });
                prop_assert!(
                    matching.is_some(),
                    "every provisional name must mirror one internal identity"
                );
                let matching = matching.expect("checked above");
                prop_assert_eq!(matching.namespace(), entry.namespace());
                prop_assert_eq!(matching.lookup_key(), entry.lookup_key());
                prop_assert_eq!(
                    entry.lookup_name(),
                    matching
                        .declared_name()
                        .unwrap_or_else(|| entry.lookup_name()),
                    "ordinary generated declarations retain their source spelling"
                );
                prop_assert_eq!(
                    entry.is_exportable(),
                    matches!(entry.visibility(), Visibility::Public),
                    "exportability remains a bounded collection fact derived only from public visibility"
                );
            }
        }

        prop_assert!(
            projection.iter().any(|module| module.module_key == root_key),
            "the generated graph always retains its root module"
        );
    }
}

#[test]
fn task_8_compatibility_routes_keep_task_2068_and_task_2070_authority_bounded() {
    let (graph, root_key) = parsed_source(
        "task-8-compatibility-task-2068",
        r#"
            pub mod api {
                pub fn target(value: Int) -> Int { value }
            }
            use crate::api::target as imported_target;
        "#,
    );
    let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
        .expect("TASK-2068 scopes remain derivable from the canonical parser graph");
    let imports = resolve_simple_parsed_imports_with_scopes(&graph, &scopes)
        .expect("TASK-2068 scoped import route remains unchanged");
    assert!(imports.binding(&root_key, "imported_target").is_some());

    let (alias_graph, alias_root_key) = parsed_source(
        "task-8-compatibility-task-2070",
        r#"
            pub mod api {
                pub fn target(value: Int) -> Int { value }
                use self::target as local_target;
            }
        "#,
    );
    let alias_api_key = alias_root_key
        .child("api")
        .expect("fixture child key is canonical");
    let alias_scopes = CanonicalProvisionalModuleScopes::from_graph(&alias_graph)
        .expect("TASK-2070 scopes remain derivable from the canonical parser graph");
    let aliases =
        resolve_scoped_self_ordinary_function_imports_with_scopes(&alias_graph, &alias_scopes)
            .expect("TASK-2070 self-alias route remains unchanged");
    assert!(aliases.binding(&alias_api_key, "local_target").is_some());
}

#[test]
fn task_8_collection_source_excludes_downstream_authority_layers() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/canonical_module_collection.rs");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    for forbidden_identifier in [
        "ModuleIdentity",
        "ModuleId",
        "ModuleGraph",
        "LegacyModuleResolver",
        "ModuleResolver",
        "NameBinder",
        "VisibilityChecker",
        "CanonicalModuleBinder",
        "CanonicalBinding",
        "CanonicalBound",
        "CanonicalProvisionalModuleScopes",
        "CanonicalResolvedSimpleImports",
        "CanonicalChecked",
        "CheckedInterfaceStore",
        "TypeEnvModuleInterfaceCollection",
        "FinalizedModuleInterface",
        "PublicModuleInterface",
        "InterfaceImportResolver",
        "CoreExpr",
        "RawCoreProgram",
        "CpsProgram",
        "Engine",
        "Admission",
        "RuntimeValue",
        "Cli",
        "Daemon",
    ] {
        assert!(
            !contains_exact_identifier(&source, forbidden_identifier),
            "TASK-2075 collection must not acquire downstream authority carrier {forbidden_identifier}"
        );
    }
    for forbidden_route in [
        "resolve_simple_parsed_imports_with_scopes",
        "resolve_scoped_self_ordinary_function_imports_with_scopes",
        "bind_scoped_self_ordinary_function_imports",
        "module_interface_finalization",
        "module_core_cps_lowering",
        "engine_execute",
        "admission_validate",
        "runtime_execute",
        "is_visible_in_module",
        "interface_import_resolver",
        "parse_surface",
        "resolve_root",
        "from_legacy",
        "into_legacy",
    ] {
        assert!(
            !source.contains(forbidden_route),
            "TASK-2075 collection must not invoke downstream authority route {forbidden_route}"
        );
    }
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
            source_anchor: Span,
        }
        impl CanonicalCollectedEntry {
            pub fn identity(&self) -> &CanonicalDeclarationIdentity { unimplemented!() }
            pub fn lookup_key(&self) -> &CanonicalLookupKey { unimplemented!() }
            pub fn declared_name(&self) -> Option<&str> { unimplemented!() }
            pub fn kind(&self) -> CanonicalDeclarationKind { unimplemented!() }
            pub fn namespace(&self) -> CanonicalNamespace { unimplemented!() }
            pub fn raw_definition(&self) -> Option<&Definition> { unimplemented!() }
            pub fn callable_body(&self) -> Option<&Expr> { unimplemented!() }
            pub fn source_anchor(&self) -> Span { unimplemented!() }
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
