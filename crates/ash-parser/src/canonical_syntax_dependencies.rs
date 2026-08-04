//! AST-only syntax dependencies for canonical parser module expansion.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use ash_core::module_graph::{ModuleArtifactOrigin, ModuleKey};

use crate::canonical_module_graph::CanonicalModuleGraph;
use crate::module::ModuleUnit;
use crate::surface::{
    Definition, ExpansionError, Expr, MacroDef, Visibility, visit_exprs_in_definition,
};
use crate::token::Span;
use crate::use_tree::{Use, UsePath};

/// Classification of a rejected syntax-only import.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalSyntaxImportFailureKind {
    /// A structural module segment on the provider path is not public.
    PrivateModulePath,
    /// The named macro exists but is not publicly importable.
    PrivateMacro,
    /// The name resolves to a declaration that is not a macro.
    NonMacroDeclaration,
    /// No syntax summary exists for the requested name.
    MissingSummary,
    /// Two imported macros use the same consumer-local name.
    DuplicateLocalName,
    /// The use path lies outside the bounded canonical `crate::...::name` form.
    UnsupportedPath,
}

impl fmt::Display for CanonicalSyntaxImportFailureKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::PrivateModulePath => "private structural module path",
            Self::PrivateMacro => "private macro",
            Self::NonMacroDeclaration => "non-macro declaration",
            Self::MissingSummary => "missing syntax summary",
            Self::DuplicateLocalName => "duplicate local syntax name",
            Self::UnsupportedPath => "unsupported syntax import path",
        })
    }
}

/// Anchored rejection of one syntax-only import requested by a macro use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalSyntaxImportFailure {
    kind: CanonicalSyntaxImportFailureKind,
    consumer_key: ModuleKey,
    consumer_source_path: Option<Box<str>>,
    consumer_artifact_origin: ModuleArtifactOrigin,
    provider_key: Option<ModuleKey>,
    provider_source_path: Option<Box<str>>,
    provider_artifact_origin: Option<ModuleArtifactOrigin>,
    use_span: Span,
    declaration_span: Option<Span>,
}

impl CanonicalSyntaxImportFailure {
    /// Returns the reason this syntax import was rejected.
    #[must_use]
    pub const fn kind(&self) -> CanonicalSyntaxImportFailureKind {
        self.kind
    }
    /// Returns the canonical module requesting syntax.
    #[must_use]
    pub fn consumer_key(&self) -> &ModuleKey {
        &self.consumer_key
    }
    /// Returns the consumer's enclosing source path, when available.
    #[must_use]
    pub fn consumer_source_path(&self) -> Option<&str> {
        self.consumer_source_path.as_deref()
    }
    /// Returns the consumer's durable parsed artifact origin.
    #[must_use]
    pub fn consumer_artifact_origin(&self) -> &ModuleArtifactOrigin {
        &self.consumer_artifact_origin
    }
    /// Returns the canonical provider module, when resolved.
    #[must_use]
    pub fn provider_key(&self) -> Option<&ModuleKey> {
        self.provider_key.as_ref()
    }
    /// Returns the provider's enclosing source path, when available.
    #[must_use]
    pub fn provider_source_path(&self) -> Option<&str> {
        self.provider_source_path.as_deref()
    }
    /// Returns the provider's durable parsed artifact origin, when resolved.
    #[must_use]
    pub fn provider_artifact_origin(&self) -> Option<&ModuleArtifactOrigin> {
        self.provider_artifact_origin.as_ref()
    }
    /// Returns the exact parsed use anchor.
    #[must_use]
    pub const fn use_span(&self) -> Span {
        self.use_span
    }
    /// Returns the rejected declaration anchor, when one resolved.
    #[must_use]
    pub const fn declaration_span(&self) -> Option<Span> {
        self.declaration_span
    }
}

impl fmt::Display for CanonicalSyntaxImportFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} requested by `{}` at {:?}",
            self.kind, self.consumer_key, self.use_span
        )
    }
}

impl std::error::Error for CanonicalSyntaxImportFailure {}

/// Provider-owned failure while closing or validating a public macro template.
#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalSyntaxProviderFailure {
    provider_key: ModuleKey,
    source_path: Option<Box<str>>,
    artifact_origin: ModuleArtifactOrigin,
    declaration_span: Span,
    source: ExpansionError,
}

impl CanonicalSyntaxProviderFailure {
    pub(crate) fn new(
        provider_key: ModuleKey,
        provider: &ModuleUnit,
        declaration_span: Span,
        source: ExpansionError,
    ) -> Self {
        Self {
            provider_key,
            source_path: provider.source_path().map(Into::into),
            artifact_origin: provider.artifact().origin().clone(),
            declaration_span,
            source,
        }
    }

    /// Returns the canonical provider whose template failed.
    #[must_use]
    pub fn provider_key(&self) -> &ModuleKey {
        &self.provider_key
    }
    /// Returns the provider's enclosing source path, when available.
    #[must_use]
    pub fn source_path(&self) -> Option<&str> {
        self.source_path.as_deref()
    }
    /// Returns the provider's durable parsed artifact origin.
    #[must_use]
    pub fn artifact_origin(&self) -> &ModuleArtifactOrigin {
        &self.artifact_origin
    }
    /// Returns the public macro declaration anchor.
    #[must_use]
    pub const fn declaration_span(&self) -> Span {
        self.declaration_span
    }
    /// Returns the underlying template expansion or validation error.
    #[must_use]
    pub fn expansion_error(&self) -> &ExpansionError {
        &self.source
    }
}

impl fmt::Display for CanonicalSyntaxProviderFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid syntax provider `{}` at {:?}: {}",
            self.provider_key, self.declaration_span, self.source
        )
    }
}

impl std::error::Error for CanonicalSyntaxProviderFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// Retained provenance for one authorized syntax-only macro import.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalSyntaxImport {
    provider_key: ModuleKey,
    exported_name: Box<str>,
    local_name: Box<str>,
    provider_declaration_span: Span,
    use_span: Span,
}

impl CanonicalSyntaxImport {
    /// Returns the canonical provider identity.
    #[must_use]
    pub fn provider_key(&self) -> &ModuleKey {
        &self.provider_key
    }
    /// Returns the name exported by the provider.
    #[must_use]
    pub fn exported_name(&self) -> &str {
        &self.exported_name
    }
    /// Returns the alias visible in the consumer.
    #[must_use]
    pub fn local_name(&self) -> &str {
        &self.local_name
    }
    /// Returns the provider macro declaration anchor.
    #[must_use]
    pub const fn provider_declaration_span(&self) -> Span {
        self.provider_declaration_span
    }
    /// Returns the exact parsed use anchor.
    #[must_use]
    pub const fn use_span(&self) -> Span {
        self.use_span
    }
}

/// One importer-to-provider edge in a rejected syntax dependency cycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalSyntaxDependencyEdge {
    importer_key: ModuleKey,
    provider_key: ModuleKey,
    use_span: Span,
    importer_source_path: Option<Box<str>>,
    importer_artifact_origin: ModuleArtifactOrigin,
    provider_source_path: Option<Box<str>>,
    provider_artifact_origin: ModuleArtifactOrigin,
    provider_declaration_span: Span,
}

impl CanonicalSyntaxDependencyEdge {
    /// Returns the canonical module importing syntax.
    #[must_use]
    pub fn importer_key(&self) -> &ModuleKey {
        &self.importer_key
    }
    /// Returns the canonical module providing syntax.
    #[must_use]
    pub fn provider_key(&self) -> &ModuleKey {
        &self.provider_key
    }
    /// Returns the exact parsed use anchor.
    #[must_use]
    pub const fn use_span(&self) -> Span {
        self.use_span
    }
    /// Returns the importer's enclosing source path, when available.
    #[must_use]
    pub fn importer_source_path(&self) -> Option<&str> {
        self.importer_source_path.as_deref()
    }
    /// Returns the importer's durable parsed artifact origin.
    #[must_use]
    pub fn importer_artifact_origin(&self) -> &ModuleArtifactOrigin {
        &self.importer_artifact_origin
    }
    /// Returns the provider's enclosing source path, when available.
    #[must_use]
    pub fn provider_source_path(&self) -> Option<&str> {
        self.provider_source_path.as_deref()
    }
    /// Returns the provider's durable parsed artifact origin.
    #[must_use]
    pub fn provider_artifact_origin(&self) -> &ModuleArtifactOrigin {
        &self.provider_artifact_origin
    }
    /// Returns the provider macro declaration anchor.
    #[must_use]
    pub const fn provider_declaration_span(&self) -> Span {
        self.provider_declaration_span
    }
}

/// Stable ordered syntax edges forming a canonical dependency cycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalSyntaxDependencyCycle {
    edges: Box<[CanonicalSyntaxDependencyEdge]>,
}

impl CanonicalSyntaxDependencyCycle {
    /// Returns importer-to-provider edges in deterministic cycle order.
    #[must_use]
    pub fn edges(&self) -> &[CanonicalSyntaxDependencyEdge] {
        &self.edges
    }
}

impl fmt::Display for CanonicalSyntaxDependencyCycle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("canonical syntax dependency cycle")?;
        for edge in &self.edges {
            write!(
                formatter,
                " {} -> {} at {:?}",
                edge.importer_key, edge.provider_key, edge.use_span
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for CanonicalSyntaxDependencyCycle {}

#[derive(Debug, Clone)]
pub(crate) struct CanonicalSyntaxImportRequest {
    provenance: CanonicalSyntaxImport,
}

impl CanonicalSyntaxImportRequest {
    pub(crate) fn provenance(&self) -> &CanonicalSyntaxImport {
        &self.provenance
    }
}

/// Successful private prepass output consumed by atomic graph expansion.
pub(crate) struct CanonicalSyntaxPrepass {
    order: Vec<ModuleKey>,
    requests: BTreeMap<ModuleKey, Vec<CanonicalSyntaxImportRequest>>,
    requested_exports: BTreeMap<ModuleKey, BTreeSet<Box<str>>>,
}

impl CanonicalSyntaxPrepass {
    pub(crate) fn order(&self) -> &[ModuleKey] {
        &self.order
    }
    pub(crate) fn requests(&self, key: &ModuleKey) -> &[CanonicalSyntaxImportRequest] {
        self.requests
            .get(key)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }
    pub(crate) fn requested_exports(&self, key: &ModuleKey) -> impl Iterator<Item = &str> {
        self.requested_exports
            .get(key)
            .into_iter()
            .flat_map(|names| names.iter().map(AsRef::as_ref))
    }
}

/// Private errors mapped into the public canonical expansion boundary.
pub(crate) enum CanonicalSyntaxPrepassError {
    InvalidSyntaxImport(Box<CanonicalSyntaxImportFailure>),
    SyntaxDependencyCycle(Box<CanonicalSyntaxDependencyCycle>),
}

/// Runs the bounded AST-only syntax dependency prepass.
pub(crate) fn prepare_canonical_syntax_dependencies(
    graph: &CanonicalModuleGraph,
) -> Result<CanonicalSyntaxPrepass, CanonicalSyntaxPrepassError> {
    let mut requests = BTreeMap::<ModuleKey, Vec<CanonicalSyntaxImportRequest>>::new();
    let mut requested_exports = BTreeMap::<ModuleKey, BTreeSet<Box<str>>>::new();
    let mut dependencies = BTreeMap::<ModuleKey, Vec<CanonicalSyntaxDependencyEdge>>::new();

    for (consumer_key, consumer_unit) in graph.module_units() {
        dependencies.entry(consumer_key.clone()).or_default();
        let invocation_names = macro_invocation_names(consumer_unit.body().definitions());
        let local_macro_names = consumer_unit
            .body()
            .definitions()
            .iter()
            .filter_map(|definition| match definition {
                Definition::Macro(definition) => Some(definition.name.clone()),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        let mut imported_local_names = BTreeSet::<Box<str>>::new();

        for use_declaration in consumer_unit.body().uses() {
            let Some(local_name) = simple_use_local_name(use_declaration) else {
                continue;
            };
            if !invocation_names.contains(local_name) || local_macro_names.contains(local_name) {
                continue;
            }
            let Some((provider_key, exported_name)) =
                resolve_simple_crate_use(graph.root_key(), use_declaration)
            else {
                return Err(invalid_import(
                    graph,
                    consumer_key,
                    consumer_unit,
                    CanonicalSyntaxImportFailureKind::UnsupportedPath,
                    None,
                    use_declaration.span,
                    None,
                ));
            };
            let Some(provider_unit) = graph.module_unit(&provider_key) else {
                return Err(invalid_import(
                    graph,
                    consumer_key,
                    consumer_unit,
                    CanonicalSyntaxImportFailureKind::MissingSummary,
                    Some(provider_key),
                    use_declaration.span,
                    None,
                ));
            };
            if let Some(private_span) = first_private_provider_path_span(graph, &provider_key) {
                return Err(invalid_import(
                    graph,
                    consumer_key,
                    consumer_unit,
                    CanonicalSyntaxImportFailureKind::PrivateModulePath,
                    Some(provider_key),
                    use_declaration.span,
                    Some(private_span),
                ));
            }

            let macro_definition = if let Some(definition) =
                find_macro_declaration(provider_unit.body().definitions(), &exported_name)
            {
                if !matches!(definition.visibility, Visibility::Public) {
                    return Err(invalid_import(
                        graph,
                        consumer_key,
                        consumer_unit,
                        CanonicalSyntaxImportFailureKind::PrivateMacro,
                        Some(provider_key),
                        use_declaration.span,
                        Some(definition.span),
                    ));
                }
                definition
            } else if let Some(span) =
                find_non_macro_declaration_span(provider_unit.body().definitions(), &exported_name)
            {
                return Err(invalid_import(
                    graph,
                    consumer_key,
                    consumer_unit,
                    CanonicalSyntaxImportFailureKind::NonMacroDeclaration,
                    Some(provider_key),
                    use_declaration.span,
                    Some(span),
                ));
            } else {
                return Err(invalid_import(
                    graph,
                    consumer_key,
                    consumer_unit,
                    CanonicalSyntaxImportFailureKind::MissingSummary,
                    Some(provider_key),
                    use_declaration.span,
                    None,
                ));
            };

            if !imported_local_names.insert(local_name.into()) {
                return Err(invalid_import(
                    graph,
                    consumer_key,
                    consumer_unit,
                    CanonicalSyntaxImportFailureKind::DuplicateLocalName,
                    Some(provider_key),
                    use_declaration.span,
                    Some(macro_definition.span),
                ));
            }

            let provenance = CanonicalSyntaxImport {
                provider_key: provider_key.clone(),
                exported_name: exported_name.clone().into_boxed_str(),
                local_name: local_name.into(),
                provider_declaration_span: macro_definition.span,
                use_span: use_declaration.span,
            };
            requests
                .entry(consumer_key.clone())
                .or_default()
                .push(CanonicalSyntaxImportRequest { provenance });
            requested_exports
                .entry(provider_key.clone())
                .or_default()
                .insert(exported_name.into_boxed_str());
            dependencies.entry(consumer_key.clone()).or_default().push(
                CanonicalSyntaxDependencyEdge {
                    importer_key: consumer_key.clone(),
                    provider_key,
                    use_span: use_declaration.span,
                    importer_source_path: consumer_unit.source_path().map(Into::into),
                    importer_artifact_origin: consumer_unit.artifact().origin().clone(),
                    provider_source_path: provider_unit.source_path().map(Into::into),
                    provider_artifact_origin: provider_unit.artifact().origin().clone(),
                    provider_declaration_span: macro_definition.span,
                },
            );
        }
    }

    for edges in dependencies.values_mut() {
        edges.sort_by(|left, right| {
            left.provider_key
                .cmp(&right.provider_key)
                .then(left.use_span.start.cmp(&right.use_span.start))
        });
    }
    let order = stable_provider_first_order(graph, &dependencies)?;
    Ok(CanonicalSyntaxPrepass {
        order,
        requests,
        requested_exports,
    })
}

fn invalid_import(
    graph: &CanonicalModuleGraph,
    consumer_key: &ModuleKey,
    consumer: &ModuleUnit,
    kind: CanonicalSyntaxImportFailureKind,
    provider_key: Option<ModuleKey>,
    use_span: Span,
    declaration_span: Option<Span>,
) -> CanonicalSyntaxPrepassError {
    let provider = provider_key.as_ref().and_then(|key| graph.module_unit(key));
    CanonicalSyntaxPrepassError::InvalidSyntaxImport(Box::new(CanonicalSyntaxImportFailure {
        kind,
        consumer_key: consumer_key.clone(),
        consumer_source_path: consumer.source_path().map(Into::into),
        consumer_artifact_origin: consumer.artifact().origin().clone(),
        provider_key,
        provider_source_path: provider.and_then(ModuleUnit::source_path).map(Into::into),
        provider_artifact_origin: provider.map(|unit| unit.artifact().origin().clone()),
        use_span,
        declaration_span,
    }))
}

fn first_private_provider_path_span(
    graph: &CanonicalModuleGraph,
    provider_key: &ModuleKey,
) -> Option<Span> {
    let mut parent_key = graph.root_key().clone();
    for segment in provider_key.segments() {
        let parent = graph.module_unit(&parent_key)?;
        let declaration = parent
            .body()
            .module_decls()
            .iter()
            .find(|declaration| declaration.name.as_ref() == segment)?;
        if !matches!(declaration.visibility, Visibility::Public) {
            return Some(declaration.span);
        }
        parent_key = parent_key.child(segment).ok()?;
    }
    None
}

fn simple_use_local_name(use_declaration: &Use) -> Option<&str> {
    let UsePath::Simple(path) = &use_declaration.path else {
        return None;
    };
    use_declaration
        .alias
        .as_deref()
        .or_else(|| path.segments.last().map(AsRef::as_ref))
}

fn resolve_simple_crate_use(
    root: &ModuleKey,
    use_declaration: &Use,
) -> Option<(ModuleKey, String)> {
    let UsePath::Simple(path) = &use_declaration.path else {
        return None;
    };
    if path.segments.len() < 2 || path.segments.first()?.as_ref() != "crate" {
        return None;
    }
    let exported_name = path.segments.last()?.to_string();
    let provider = path.segments[1..path.segments.len() - 1]
        .iter()
        .try_fold(root.clone(), |key, segment| key.child(segment.as_ref()))
        .ok()?;
    Some((provider, exported_name))
}

fn macro_invocation_names(definitions: &[Definition]) -> BTreeSet<Box<str>> {
    let mut names = BTreeSet::new();
    for definition in definitions {
        visit_exprs_in_definition(definition, &mut |expression| {
            if let Expr::MacroInvocation { invocation } = expression {
                names.insert(invocation.name.clone());
            }
        });
    }
    names
}

fn find_macro_declaration<'a>(definitions: &'a [Definition], name: &str) -> Option<&'a MacroDef> {
    definitions.iter().find_map(|definition| match definition {
        Definition::Macro(definition) if definition.name.as_ref() == name => Some(definition),
        _ => None,
    })
}

fn find_non_macro_declaration_span(definitions: &[Definition], name: &str) -> Option<Span> {
    definitions
        .iter()
        .filter_map(definition_name_span)
        .find_map(|(definition_name, span)| (definition_name == name).then_some(span))
}

fn definition_name_span(definition: &Definition) -> Option<(&str, Span)> {
    match definition {
        Definition::Notation(definition) => Some((&definition.pattern.raw, definition.span)),
        Definition::Macro(_) | Definition::Impl(_) => None,
        Definition::Capability(definition) => Some((&definition.name, definition.span)),
        Definition::ResourceType(definition) => Some((&definition.name, definition.span)),
        Definition::Type(definition) => Some((&definition.name, definition.span)),
        Definition::Newtype(definition) => Some((&definition.name, definition.span)),
        Definition::EffectAlias(definition) => Some((&definition.name, definition.span)),
        Definition::EffectGroup(definition) => Some((&definition.name, definition.span)),
        Definition::DataKind(definition) => Some((&definition.name, definition.span)),
        Definition::TypeFn(definition) => Some((&definition.name, definition.span)),
        Definition::PropositionPredicate(definition) => Some((&definition.name, definition.span)),
        Definition::Policy(definition) => Some((&definition.name, definition.span)),
        Definition::Role(definition) => Some((&definition.name, definition.span)),
        Definition::Interface(definition) => Some((&definition.name, definition.span)),
        Definition::Function(definition) => Some((&definition.name, definition.span)),
        Definition::Handler(definition) => Some((&definition.name, definition.span)),
        Definition::BuiltinFn(definition) => Some((&definition.name, definition.span)),
        Definition::SealedDomain(definition) => Some((&definition.name, definition.span)),
        Definition::Law(definition) => Some((&definition.name, definition.span)),
        Definition::Proof(definition) => Some((&definition.name, definition.span)),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisitState {
    Visiting,
    Complete,
}

fn stable_provider_first_order(
    graph: &CanonicalModuleGraph,
    dependencies: &BTreeMap<ModuleKey, Vec<CanonicalSyntaxDependencyEdge>>,
) -> Result<Vec<ModuleKey>, CanonicalSyntaxPrepassError> {
    let mut states = BTreeMap::new();
    let mut node_stack = Vec::new();
    let mut edge_stack = Vec::new();
    let mut order = Vec::new();
    for (key, _) in graph.module_units() {
        if !states.contains_key(key) {
            visit_dependency(
                key,
                dependencies,
                &mut states,
                &mut node_stack,
                &mut edge_stack,
                &mut order,
            )?;
        }
    }
    Ok(order)
}

fn visit_dependency(
    key: &ModuleKey,
    dependencies: &BTreeMap<ModuleKey, Vec<CanonicalSyntaxDependencyEdge>>,
    states: &mut BTreeMap<ModuleKey, VisitState>,
    node_stack: &mut Vec<ModuleKey>,
    edge_stack: &mut Vec<CanonicalSyntaxDependencyEdge>,
    order: &mut Vec<ModuleKey>,
) -> Result<(), CanonicalSyntaxPrepassError> {
    states.insert(key.clone(), VisitState::Visiting);
    node_stack.push(key.clone());
    if let Some(edges) = dependencies.get(key) {
        for edge in edges {
            match states.get(&edge.provider_key) {
                Some(VisitState::Complete) => continue,
                Some(VisitState::Visiting) => {
                    let cycle_start = node_stack
                        .iter()
                        .position(|node| node == &edge.provider_key)
                        .unwrap_or(edge_stack.len());
                    let mut cycle_edges = edge_stack[cycle_start..].to_vec();
                    cycle_edges.push(edge.clone());
                    return Err(CanonicalSyntaxPrepassError::SyntaxDependencyCycle(
                        Box::new(CanonicalSyntaxDependencyCycle {
                            edges: cycle_edges.into_boxed_slice(),
                        }),
                    ));
                }
                None => {
                    edge_stack.push(edge.clone());
                    visit_dependency(
                        &edge.provider_key,
                        dependencies,
                        states,
                        node_stack,
                        edge_stack,
                        order,
                    )?;
                    edge_stack.pop();
                }
            }
        }
    }
    node_stack.pop();
    states.insert(key.clone(), VisitState::Complete);
    order.push(key.clone());
    Ok(())
}
