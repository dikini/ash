//! Module resolution algorithm for the Ash parser.
//!
//! This module provides functionality to discover and resolve module dependencies
//! in Ash source files. It supports Rust-style module resolution where `mod foo;`
//! looks for `foo.ash` or `foo/mod.ash`.
//!
//! This resolver also supports multi-crate resolution with dependency management.

use ash_core::module_graph::{
    CrateId, ModuleArtifact, ModuleArtifactOrigin, ModuleGraph, ModuleId, ModuleKey, ModuleNode,
    ModuleSource as CoreModuleSource,
};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::module::{ModuleBody, ModuleDecl, ModuleSource as ParsedModuleSource, ModuleUnit};
use crate::surface::{DependencyDecl, ModuleFile, Visibility};
use crate::token::Span;

/// The source form retained by AST-derived module discovery.
///
/// File-backed declarations remain on the compatibility filesystem-resolution
/// path. Inline declarations are deliberately retained without attempting
/// filesystem acquisition; TASK-2059 owns their module-unit realization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveredModuleSource {
    File,
    Inline,
}

/// A structural child declaration copied from an authoritative [`ModuleFile`].
///
/// This is the resolver's non-authorizing handoff to the later identity and
/// source-acquisition tasks. It keeps the parser-owned spelling and anchor
/// intact while preventing a source-text scanner from manufacturing children.
#[derive(Debug, Clone)]
pub struct DiscoveredModuleDecl {
    /// Parser-owned child name.
    pub name: Box<str>,
    /// Parser-owned visibility modifier.
    pub visibility: Visibility,
    /// Parser-owned source form.
    pub source: DiscoveredModuleSource,
    /// Parser-owned declaration origin.
    pub span: Span,
    /// Source file that supplied this declaration.
    pub path: PathBuf,
}

impl DiscoveredModuleDecl {
    fn from_ast(declaration: &ModuleDecl, path: &Path) -> Self {
        let source = match declaration.source {
            ParsedModuleSource::File => DiscoveredModuleSource::File,
            ParsedModuleSource::Inline(_) => DiscoveredModuleSource::Inline,
        };

        Self {
            name: declaration.name.clone(),
            visibility: declaration.visibility.clone(),
            source,
            span: declaration.span,
            path: path.to_path_buf(),
        }
    }
}

/// Derive structural child declarations from an authoritative [`ModuleFile`].
///
/// The records preserve parser-originated name, visibility, source form, span,
/// and source path for downstream module-realization tasks. Duplicate names
/// fail before a caller can publish a graph node or edge.
///
/// # Errors
///
/// Returns [`ResolveError::DuplicateModuleDeclaration`] when the parsed file
/// declares the same child name more than once.
pub fn discover_module_declarations(
    module_file: &ModuleFile,
    path: &Path,
) -> Result<Vec<DiscoveredModuleDecl>, ResolveError> {
    let mut declarations = Vec::with_capacity(module_file.module_decls.len());
    let mut names = HashMap::with_capacity(module_file.module_decls.len());

    for declaration in &module_file.module_decls {
        let discovered = DiscoveredModuleDecl::from_ast(declaration, path);
        if let Some(first_span) = names.insert(discovered.name.clone(), discovered.span) {
            return Err(ResolveError::DuplicateModuleDeclaration {
                module_name: discovered.name.to_string(),
                path: discovered.path,
                first_line: first_span.line,
                first_column: first_span.column,
                line: discovered.span.line,
                column: discovered.span.column,
            });
        }
        declarations.push(discovered);
    }

    Ok(declarations)
}

/// File system abstraction trait for testability.
///
/// Implementations can provide real file system access or mock implementations
/// for testing.
pub trait Fs: Send + Sync {
    /// Read the contents of a file at the given path.
    /// Returns `Some(String)` if the file exists and can be read, `None` otherwise.
    fn read_file(&self, path: &Path) -> Option<String>;

    /// Check if a file exists at the given path.
    fn file_exists(&self, path: &Path) -> bool;
}

/// Errors that can occur during module resolution.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ResolveError {
    /// A module was not found at the expected location.
    #[error("module not found: {module_name} (expected at {expected_path})")]
    ModuleNotFound {
        /// The name of the module that was not found.
        module_name: String,
        /// The path where the module was expected.
        expected_path: PathBuf,
    },

    /// A circular dependency was detected.
    #[error("circular dependency detected: {cycle}")]
    CircularDependency {
        /// Description of the circular dependency cycle.
        cycle: String,
    },

    /// Failed to parse a module declaration.
    #[error("parse error in {path}: {message}")]
    ParseError {
        /// The path to the file that failed to parse.
        path: PathBuf,
        /// Description of the parse error.
        message: String,
    },

    /// A parsed module file declared the same child name more than once.
    #[error(
        "duplicate module declaration `{module_name}` in {path} at {line}:{column} (first declared at {first_line}:{first_column})"
    )]
    DuplicateModuleDeclaration {
        /// The duplicated child module name.
        module_name: String,
        /// The source file containing both declarations.
        path: PathBuf,
        /// The first declaration's source line.
        first_line: usize,
        /// The first declaration's source column.
        first_column: usize,
        /// The duplicate declaration's source line.
        line: usize,
        /// The duplicate declaration's source column.
        column: usize,
    },

    /// A parsed module unit declared the same child name more than once,
    /// retaining both parser-owned declaration spans for a structural caller.
    #[error(
        "duplicate module declaration `{module_name}` in {path} at {declaration_span:?} (first declared at {first_declaration_span:?})"
    )]
    DuplicateModuleDeclarationWithSpans {
        /// The duplicated child module name.
        module_name: String,
        /// The source file containing both declarations.
        path: PathBuf,
        /// The original declaration's parser-owned source anchor.
        first_declaration_span: Span,
        /// The later duplicate declaration's parser-owned source anchor.
        declaration_span: Span,
    },

    /// A crate was not found at the expected location.
    #[error("dependency crate not found: {crate_name} (expected at {expected_path})")]
    CrateNotFound {
        /// The name of the crate that was not found.
        crate_name: String,
        /// The path where the crate was expected.
        expected_path: PathBuf,
    },

    /// A duplicate crate name was detected.
    #[error("duplicate crate name: {crate_name}")]
    DuplicateCrateName {
        /// The duplicate crate name.
        crate_name: String,
    },

    /// A duplicate dependency alias was declared in the same crate.
    #[error("duplicate dependency alias: {alias} in crate {crate_name}")]
    DuplicateDependencyAlias {
        /// The duplicate alias.
        alias: String,
        /// The crate that declared the duplicate alias.
        crate_name: String,
    },

    /// A circular dependency between crates was detected.
    #[error("crate dependency cycle detected: {cycle}")]
    CrateCycle {
        /// Description of the circular dependency cycle.
        cycle: String,
    },

    /// A source-acquisition child could not be found from its declaration.
    #[error(
        "module unit not found: {module_name} declared at {parent_path}:{declaration_span:?} (expected at {expected_path})"
    )]
    ModuleUnitNotFound {
        /// Parser-owned child spelling that could not be acquired.
        module_name: String,
        /// Enclosing source file that contains the child declaration.
        parent_path: PathBuf,
        /// Parser-owned declaration span within `parent_path`.
        declaration_span: Span,
        /// First source candidate considered for this declaration.
        expected_path: PathBuf,
    },

    /// A parsed declaration could not form a canonical child module key.
    #[error(
        "invalid module unit identity for {module_name} declared at {parent_path}:{declaration_span:?}: {message}"
    )]
    InvalidModuleUnitIdentity {
        /// Parser-owned child spelling that failed canonical validation.
        module_name: String,
        /// Enclosing source file that contains the child declaration.
        parent_path: PathBuf,
        /// Parser-owned declaration span within `parent_path`.
        declaration_span: Span,
        /// Canonical-key validation detail.
        message: String,
    },

    /// A fully parsed module body could not form a structural artifact.
    #[error("invalid module artifact for {path}: {message}")]
    InvalidModuleArtifact {
        /// File or parent-source path used for diagnostic anchoring.
        path: PathBuf,
        /// Structural validation detail.
        message: String,
    },
}

/// Real file system implementation of the `Fs` trait.
struct RealFs;

impl Fs for RealFs {
    fn read_file(&self, path: &Path) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }

    fn file_exists(&self, path: &Path) -> bool {
        path.is_file()
    }
}

/// Acquires one fully parsed module unit from a file or inline declaration.
///
/// This resolver deliberately has no graph, cache, import binding, interface,
/// lowering, or Engine responsibilities. It selects a source, parses a file
/// exactly once (or reuses an inline body), validates its artifact, and only
/// then returns the completed [`ModuleUnit`].
pub struct ModuleUnitResolver {
    fs: Box<dyn Fs>,
}

/// Selects the diagnostic fidelity required by an acquisition caller.
///
/// The public unit-acquisition API retains its existing line/column duplicate
/// error contract. Canonical graph construction needs the parser-owned spans
/// in order to return a structural diagnostic anchored at the declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DuplicateDiagnosticMode {
    LegacyCoordinates,
    ParsedSpans,
}

/// Parser-owned root acquisition facts used only by canonical graph construction.
///
/// The root unit keeps the same ordered-body carrier as child units, while the
/// optional metadata retains the parsed outer preamble for the graph handoff.
pub(crate) struct CanonicalRootAcquisition {
    pub(crate) module_unit: ModuleUnit,
    pub(crate) crate_metadata: Option<crate::surface::CrateRootMetadata>,
}

/// Private root-acquisition failure detail used only by canonical graph construction.
///
/// Public [`ModuleUnitResolver::acquire_root`] callers keep receiving the
/// established generic [`ResolveError::ParseError`] contract. The canonical
/// route additionally retains a parsed crate preamble and incomplete
/// inline-header context without creating synthetic declaration diagnostics.
pub(crate) enum CanonicalRootAcquisitionFailure {
    Resolve(ResolveError),
    MalformedInline {
        module_name: Box<str>,
        declaration_span: Span,
        error_span: Span,
    },
}

/// Private child-acquisition detail used only by canonical graph construction.
///
/// Public [`ModuleUnitResolver::acquire_child`] callers intentionally retain
/// the established generic [`ResolveError::ParseError`] contract. The
/// canonical graph route preserves a parsed malformed-inline header so it can
/// report the canonical nested child that never became a complete AST node.
pub(crate) enum CanonicalChildAcquisitionFailure {
    Resolve(ResolveError),
    MalformedInline(Box<CanonicalMalformedInlineChild>),
}

/// Parser-owned malformed-inline detail retained by the canonical child route.
///
/// The payload is boxed because only an error path needs its full source
/// provenance, while successful child acquisition should not carry it.
pub(crate) struct CanonicalMalformedInlineChild {
    pub(crate) module_name: Box<str>,
    pub(crate) source_path: PathBuf,
    pub(crate) declaration_span: Span,
    pub(crate) error_span: Span,
    pub(crate) message: String,
}

impl CanonicalChildAcquisitionFailure {
    fn into_resolve_error(self) -> ResolveError {
        match self {
            Self::Resolve(error) => error,
            Self::MalformedInline(detail) => ResolveError::ParseError {
                path: detail.source_path,
                message: detail.message,
            },
        }
    }
}

impl From<ResolveError> for CanonicalChildAcquisitionFailure {
    fn from(error: ResolveError) -> Self {
        Self::Resolve(error)
    }
}

impl Default for ModuleUnitResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleUnitResolver {
    /// Creates a unit resolver backed by the real filesystem.
    #[must_use]
    pub fn new() -> Self {
        Self {
            fs: Box::new(RealFs),
        }
    }

    /// Creates a unit resolver with a testable filesystem implementation.
    #[must_use]
    pub fn with_fs(fs: Box<dyn Fs>) -> Self {
        Self { fs }
    }

    /// Acquires the root source as a fully parsed module unit.
    ///
    /// Root acquisition uses the same ordered-body parser and artifact carrier
    /// as file children. Its artifact has no structural parent and records the
    /// supplied `root_key` as its canonical identity.
    ///
    /// # Errors
    ///
    /// Returns [`ResolveError`] when the source cannot be read or parsed, or
    /// when its parsed declarations cannot form a valid module artifact.
    pub fn acquire_root(
        &self,
        root_key: ModuleKey,
        root_path: &Path,
    ) -> Result<ModuleUnit, ResolveError> {
        let content = self
            .fs
            .read_file(root_path)
            .ok_or_else(|| ResolveError::ModuleNotFound {
                module_name: root_key.to_string(),
                expected_path: root_path.to_path_buf(),
            })?;
        let (body, comments) =
            crate::parse_module_body_with_path(&content, root_path).map_err(|failure| {
                ResolveError::ParseError {
                    path: root_path.to_path_buf(),
                    message: failure
                        .errors()
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("; "),
                }
            })?;
        Self::root_unit_from_body(root_key, root_path, body, comments)
    }

    /// Acquires a root for canonical graph construction with parser-owned
    /// malformed-inline recovery detail.
    pub(crate) fn acquire_root_for_canonical_graph(
        &self,
        root_key: ModuleKey,
        root_path: &Path,
    ) -> Result<CanonicalRootAcquisition, CanonicalRootAcquisitionFailure> {
        let content = self.fs.read_file(root_path).ok_or_else(|| {
            CanonicalRootAcquisitionFailure::Resolve(ResolveError::ModuleNotFound {
                module_name: root_key.to_string(),
                expected_path: root_path.to_path_buf(),
            })
        })?;
        let (body, comments, crate_metadata) =
            crate::parse_root_module_body_with_path(&content, root_path).map_err(|failure| {
                failure.malformed_inline().map_or_else(
                    || {
                        CanonicalRootAcquisitionFailure::Resolve(ResolveError::ParseError {
                            path: root_path.to_path_buf(),
                            message: failure
                                .errors()
                                .iter()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()
                                .join("; "),
                        })
                    },
                    |malformed_inline| CanonicalRootAcquisitionFailure::MalformedInline {
                        module_name: malformed_inline.name.clone(),
                        declaration_span: malformed_inline.header_span,
                        error_span: malformed_inline.error_span,
                    },
                )
            })?;
        let module_unit = Self::root_unit_from_body(root_key, root_path, body, comments)
            .map_err(CanonicalRootAcquisitionFailure::Resolve)?;
        Ok(CanonicalRootAcquisition {
            module_unit,
            crate_metadata,
        })
    }

    fn root_unit_from_body(
        root_key: ModuleKey,
        root_path: &Path,
        body: ModuleBody,
        comments: crate::parse_utils::CommentTable,
    ) -> Result<ModuleUnit, ResolveError> {
        let mut declaration_spans =
            HashMap::<ModuleKey, Span>::with_capacity(body.module_decls().len());
        let mut child_keys = Vec::with_capacity(body.module_decls().len());
        for declaration in body.module_decls() {
            let child_key = Self::module_unit_child_key(&root_key, root_path, declaration)?;
            if let Some(first_span) = declaration_spans.insert(child_key.clone(), declaration.span)
            {
                return Err(ResolveError::DuplicateModuleDeclarationWithSpans {
                    module_name: declaration.name.to_string(),
                    path: root_path.to_path_buf(),
                    first_declaration_span: first_span,
                    declaration_span: declaration.span,
                });
            }
            child_keys.push(child_key);
        }
        let artifact = ModuleArtifact::new(
            root_key,
            ModuleArtifactOrigin::File(root_path.display().to_string()),
            None,
            child_keys,
        )
        .map_err(|error| ResolveError::InvalidModuleArtifact {
            path: root_path.to_path_buf(),
            message: error.to_string(),
        })?;

        Ok(ModuleUnit::new(
            artifact,
            body,
            Some(root_path.to_string_lossy().into_owned().into()),
            comments,
        ))
    }

    /// Acquires the child declared by `declaration` below `parent_key`.
    ///
    /// For `mod child;`, lookup prefers `child.ash` over `child/mod.ash` and
    /// parses the selected file exactly once through the ordered-body parser.
    /// For `mod child { ... }`, no filesystem operation occurs; the already
    /// parsed inline body is reused. Either route returns only after the body
    /// and its structurally validated artifact are complete.
    pub fn acquire_child(
        &self,
        parent_key: &ModuleKey,
        parent_path: &Path,
        declaration: &ModuleDecl,
    ) -> Result<ModuleUnit, ResolveError> {
        self.acquire_child_with_diagnostic_mode(
            parent_key,
            parent_path,
            declaration,
            DuplicateDiagnosticMode::LegacyCoordinates,
        )
        .map_err(CanonicalChildAcquisitionFailure::into_resolve_error)
    }

    /// Acquires a child for canonical graph construction with full anchors.
    ///
    /// This remains private so callers of [`Self::acquire_child`] preserve the
    /// existing [`ResolveError::DuplicateModuleDeclaration`] contract.
    pub(crate) fn acquire_child_for_canonical_graph(
        &self,
        parent_key: &ModuleKey,
        parent_path: &Path,
        declaration: &ModuleDecl,
    ) -> Result<ModuleUnit, CanonicalChildAcquisitionFailure> {
        self.acquire_child_with_diagnostic_mode(
            parent_key,
            parent_path,
            declaration,
            DuplicateDiagnosticMode::ParsedSpans,
        )
    }

    fn acquire_child_with_diagnostic_mode(
        &self,
        parent_key: &ModuleKey,
        parent_path: &Path,
        declaration: &ModuleDecl,
        duplicate_diagnostic_mode: DuplicateDiagnosticMode,
    ) -> Result<ModuleUnit, CanonicalChildAcquisitionFailure> {
        let key = Self::module_unit_child_key(parent_key, parent_path, declaration)?;

        match &declaration.source {
            ParsedModuleSource::File => {
                let source_path = self.resolve_child_path(parent_path, declaration)?;
                let content = self.fs.read_file(&source_path).ok_or_else(|| {
                    ResolveError::ModuleUnitNotFound {
                        module_name: declaration.name.to_string(),
                        parent_path: parent_path.to_path_buf(),
                        declaration_span: declaration.span,
                        expected_path: source_path.clone(),
                    }
                })?;
                let (body, comments) = crate::parse_module_body_with_path(&content, &source_path)
                    .map_err(|failure| {
                    let message = failure
                        .errors()
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("; ");
                    match failure.malformed_inline() {
                        Some(malformed_inline) => {
                            CanonicalChildAcquisitionFailure::MalformedInline(Box::new(
                                CanonicalMalformedInlineChild {
                                    module_name: malformed_inline.name.clone(),
                                    source_path: source_path.clone(),
                                    declaration_span: malformed_inline.header_span,
                                    error_span: malformed_inline.error_span,
                                    message,
                                },
                            ))
                        }
                        None => {
                            CanonicalChildAcquisitionFailure::Resolve(ResolveError::ParseError {
                                path: source_path.clone(),
                                message,
                            })
                        }
                    }
                })?;
                let artifact = self.build_artifact(
                    parent_key,
                    key,
                    ModuleArtifactOrigin::File(source_path.display().to_string()),
                    &body,
                    &source_path,
                    duplicate_diagnostic_mode,
                )?;
                Ok(ModuleUnit::new(
                    artifact,
                    body,
                    Some(source_path.to_string_lossy().into_owned().into()),
                    comments,
                ))
            }
            ParsedModuleSource::Inline(body) => {
                let source_path = parent_path.to_path_buf();
                let body = (**body).clone();
                let artifact = self.build_artifact(
                    parent_key,
                    key,
                    ModuleArtifactOrigin::Inline {
                        parent: parent_key.clone(),
                        declaration_offset: declaration.span.start,
                    },
                    &body,
                    &source_path,
                    duplicate_diagnostic_mode,
                )?;
                Ok(ModuleUnit::new(
                    artifact,
                    body,
                    Some(source_path.to_string_lossy().into_owned().into()),
                    crate::parse_utils::CommentTable::default(),
                ))
            }
        }
    }

    fn build_artifact(
        &self,
        parent_key: &ModuleKey,
        key: ModuleKey,
        origin: ModuleArtifactOrigin,
        body: &ModuleBody,
        path: &Path,
        duplicate_diagnostic_mode: DuplicateDiagnosticMode,
    ) -> Result<ModuleArtifact, ResolveError> {
        let mut declaration_spans =
            HashMap::<ModuleKey, Span>::with_capacity(body.module_decls().len());
        let mut child_keys = Vec::with_capacity(body.module_decls().len());
        for declaration in body.module_decls() {
            let child_key = Self::module_unit_child_key(&key, path, declaration)?;
            if let Some(first_span) = declaration_spans.insert(child_key.clone(), declaration.span)
            {
                return Err(match duplicate_diagnostic_mode {
                    DuplicateDiagnosticMode::LegacyCoordinates => {
                        ResolveError::DuplicateModuleDeclaration {
                            module_name: declaration.name.to_string(),
                            path: path.to_path_buf(),
                            first_line: first_span.line,
                            first_column: first_span.column,
                            line: declaration.span.line,
                            column: declaration.span.column,
                        }
                    }
                    DuplicateDiagnosticMode::ParsedSpans => {
                        ResolveError::DuplicateModuleDeclarationWithSpans {
                            module_name: declaration.name.to_string(),
                            path: path.to_path_buf(),
                            first_declaration_span: first_span,
                            declaration_span: declaration.span,
                        }
                    }
                });
            }
            child_keys.push(child_key);
        }

        ModuleArtifact::new(key, origin, Some(parent_key.clone()), child_keys).map_err(|error| {
            ResolveError::InvalidModuleArtifact {
                path: path.to_path_buf(),
                message: error.to_string(),
            }
        })
    }

    pub(crate) fn resolve_child_path(
        &self,
        parent_path: &Path,
        declaration: &ModuleDecl,
    ) -> Result<PathBuf, ResolveError> {
        let parent_dir = parent_path.parent().unwrap_or(Path::new("."));
        let file_module = parent_dir.join(format!("{}.ash", declaration.name));
        if self.fs.file_exists(&file_module) {
            return Ok(file_module);
        }

        let directory_module = parent_dir.join(&*declaration.name).join("mod.ash");
        if self.fs.file_exists(&directory_module) {
            return Ok(directory_module);
        }

        Err(ResolveError::ModuleUnitNotFound {
            module_name: declaration.name.to_string(),
            parent_path: parent_path.to_path_buf(),
            declaration_span: declaration.span,
            expected_path: file_module,
        })
    }

    /// Creates a unit child key through the shared canonical-key contract.
    fn module_unit_child_key(
        parent_key: &ModuleKey,
        parent_path: &Path,
        declaration: &ModuleDecl,
    ) -> Result<ModuleKey, ResolveError> {
        parent_key.child(&declaration.name).map_err(|error| {
            ResolveError::InvalidModuleUnitIdentity {
                module_name: declaration.name.to_string(),
                parent_path: parent_path.to_path_buf(),
                declaration_span: declaration.span,
                message: error.to_string(),
            }
        })
    }
}

/// Compatibility-only resolver that discovers and resolves legacy graph dependencies.
///
/// This resolver walks the module hierarchy starting from a root file,
/// parsing `mod foo;` declarations and locating the corresponding files.
/// It supports both file modules (`foo.ash`) and directory modules
/// (`foo/mod.ash`), following Rust's module resolution convention. It cannot
/// feed the canonical graph, interface, binding, lowering, or admission
/// routes; use `CanonicalModuleGraphResolver` for parser-stage structure.
pub struct LegacyModuleResolver {
    fs: Box<dyn Fs>,
}

impl LegacyModuleResolver {
    /// Create a new module resolver with real file system access.
    pub fn new() -> Self {
        Self {
            fs: Box::new(RealFs),
        }
    }

    /// Create a new module resolver with a custom file system implementation.
    ///
    /// This is useful for testing with mock file systems.
    pub fn with_fs(fs: Box<dyn Fs>) -> Self {
        Self { fs }
    }

    /// Resolve a crate starting from the given root file path.
    ///
    /// Discovers all modules reachable from the root and builds a complete
    /// `ModuleGraph`. Returns an error if a module cannot be found or if
    /// a circular dependency is detected.
    ///
    /// This method also parses crate root metadata and recursively resolves
    /// all declared dependency crates.
    pub fn resolve_crate(&self, root_path: impl AsRef<Path>) -> Result<ModuleGraph, ResolveError> {
        let root_path = root_path.as_ref();
        let mut graph = ModuleGraph::new();
        let mut visited = HashSet::new();
        let mut resolution_stack = Vec::new();

        // Track loaded crates by path and name to prevent duplicates
        let mut crate_paths: HashMap<PathBuf, CrateId> = HashMap::new();
        let mut crate_names: HashMap<String, CrateId> = HashMap::new();
        // Track crates currently being resolved for cycle detection
        let mut crate_resolution_stack: Vec<String> = Vec::new();

        // Resolve the root crate with its dependencies
        let (root_id, _root_crate_id) = self.resolve_crate_internal(
            root_path,
            &mut graph,
            &mut visited,
            &mut resolution_stack,
            &mut crate_paths,
            &mut crate_names,
            &mut crate_resolution_stack,
        )?;

        graph.set_root(root_id);
        Ok(graph)
    }

    /// Internal method to resolve a crate and its dependencies.
    ///
    /// Returns the root module ID and the crate ID for this crate.
    #[allow(clippy::too_many_arguments)]
    fn resolve_crate_internal(
        &self,
        root_path: &Path,
        graph: &mut ModuleGraph,
        visited: &mut HashSet<PathBuf>,
        resolution_stack: &mut Vec<PathBuf>,
        crate_paths: &mut HashMap<PathBuf, CrateId>,
        crate_names: &mut HashMap<String, CrateId>,
        crate_resolution_stack: &mut Vec<String>,
    ) -> Result<(ModuleId, CrateId), ResolveError> {
        let canonical_path = root_path;

        // Check if this crate path is already loaded
        if let Some(&existing_crate_id) = crate_paths.get(canonical_path) {
            // Check if this crate is currently being resolved (cycle detection)
            let existing_crate = graph.get_crate(existing_crate_id).unwrap();
            let crate_name_str = &existing_crate.name;
            if let Some(pos) = crate_resolution_stack
                .iter()
                .position(|n| n == crate_name_str)
            {
                let cycle = crate_resolution_stack[pos..]
                    .iter()
                    .cloned()
                    .chain(std::iter::once(crate_name_str.clone()))
                    .collect::<Vec<_>>()
                    .join(" -> ");
                return Err(ResolveError::CrateCycle { cycle });
            }
            // Find the root module for this crate
            return Ok((existing_crate.root_module, existing_crate_id));
        }

        // Read the root file content
        let content =
            self.fs
                .read_file(canonical_path)
                .ok_or_else(|| ResolveError::ModuleNotFound {
                    module_name: canonical_path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into(),
                    expected_path: canonical_path.to_path_buf(),
                })?;

        // Parse the root exactly once and retain that AST for structural
        // resolution below. Crate metadata shares this same source carrier.
        let root_module_file = self.parse_module_file(&content, canonical_path)?;
        let metadata = root_module_file.crate_metadata.clone().unwrap_or_else(|| {
            crate::surface::CrateRootMetadata {
                crate_name: canonical_path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into(),
                dependencies: Vec::new(),
                span: crate::token::Span::new(0, 0, 1, 1),
            }
        });

        // Check for duplicate crate name
        if let Some(&existing_crate_id) = crate_names.get(metadata.crate_name.as_ref()) {
            let existing_crate = graph.get_crate(existing_crate_id).unwrap();
            // Only error if it's a different path (same crate re-loaded is OK)
            if existing_crate.root_path != canonical_path.display().to_string() {
                return Err(ResolveError::DuplicateCrateName {
                    crate_name: metadata.crate_name.to_string(),
                });
            }
        }

        // Check for crate dependency cycle
        let crate_name_str = metadata.crate_name.to_string();
        if let Some(pos) = crate_resolution_stack
            .iter()
            .position(|n| n == &crate_name_str)
        {
            let cycle = crate_resolution_stack[pos..]
                .iter()
                .cloned()
                .chain(std::iter::once(crate_name_str.clone()))
                .collect::<Vec<_>>()
                .join(" -> ");
            return Err(ResolveError::CrateCycle { cycle });
        }

        // Create the crate entry first (before resolving modules)
        // This gives us a CrateId to assign to all modules in this crate
        let root_path_str = canonical_path.display().to_string();
        let crate_id = graph.create_crate_entry(crate_name_str.clone(), root_path_str);

        // Track this crate
        crate_paths.insert(canonical_path.to_path_buf(), crate_id);
        crate_names.insert(crate_name_str.clone(), crate_id);

        // Resolve the root module of this crate (which also resolves submodules)
        // Pass the CrateId so all modules get properly assigned
        let root_module_id = self.resolve_module(
            root_path,
            canonical_path,
            graph,
            visited,
            resolution_stack,
            Some(crate_id),
            Some(root_module_file),
        )?;

        // Update the crate with the root module
        graph.set_crate_root_module(crate_id, root_module_id);

        // Now resolve dependencies
        crate_resolution_stack.push(crate_name_str.clone());

        for dep in &metadata.dependencies {
            self.resolve_dependency_crate(
                crate_id,
                dep,
                graph,
                visited,
                resolution_stack,
                crate_paths,
                crate_names,
                crate_resolution_stack,
            )?;
        }

        crate_resolution_stack.pop();

        Ok((root_module_id, crate_id))
    }

    /// Resolve a dependency crate and register it in the graph.
    #[allow(clippy::too_many_arguments)]
    fn resolve_dependency_crate(
        &self,
        declaring_crate: CrateId,
        dependency: &DependencyDecl,
        graph: &mut ModuleGraph,
        visited: &mut HashSet<PathBuf>,
        resolution_stack: &mut Vec<PathBuf>,
        crate_paths: &mut HashMap<PathBuf, CrateId>,
        crate_names: &mut HashMap<String, CrateId>,
        crate_resolution_stack: &mut Vec<String>,
    ) -> Result<CrateId, ResolveError> {
        let alias = dependency.alias.to_string();
        let dep_path_str = dependency.root_path.to_string();

        // Get the declaring crate's directory for relative path resolution
        let declaring_crate_info = graph.get_crate(declaring_crate).unwrap();
        let declaring_crate_dir = Path::new(&declaring_crate_info.root_path)
            .parent()
            .unwrap_or(Path::new("."));

        // Resolve the dependency path relative to the declaring crate
        let dep_path = declaring_crate_dir.join(&dep_path_str);
        // Normalize the path to resolve ".." and "." segments
        let canonical_dep_path = normalize_path(&dep_path);

        // Check for duplicate alias in the declaring crate
        let declaring_crate_mut = graph.get_crate_mut(declaring_crate).unwrap();
        if declaring_crate_mut.dependencies.contains_key(&alias) {
            return Err(ResolveError::DuplicateDependencyAlias {
                alias,
                crate_name: declaring_crate_mut.name.clone(),
            });
        }
        // Mutable borrow ends here via drop of the reference

        // Check if this crate is already loaded by path
        if let Some(&existing_crate_id) = crate_paths.get(&canonical_dep_path) {
            // Check for cycle: if this crate is currently being resolved, that's a cycle
            let existing_crate = graph.get_crate(existing_crate_id).unwrap();
            let dep_crate_name = &existing_crate.name;
            if let Some(pos) = crate_resolution_stack
                .iter()
                .position(|n| n == dep_crate_name)
            {
                let cycle = crate_resolution_stack[pos..]
                    .iter()
                    .cloned()
                    .chain(std::iter::once(dep_crate_name.clone()))
                    .collect::<Vec<_>>()
                    .join(" -> ");
                return Err(ResolveError::CrateCycle { cycle });
            }
            // Register the dependency alias
            graph.add_dependency(declaring_crate, alias, existing_crate_id);
            return Ok(existing_crate_id);
        }

        // Check that the dependency file exists
        if !self.fs.file_exists(&canonical_dep_path) {
            return Err(ResolveError::CrateNotFound {
                crate_name: alias,
                expected_path: canonical_dep_path.clone(),
            });
        }

        // Recursively resolve the dependency crate
        let (_, dep_crate_id) = self.resolve_crate_internal(
            &canonical_dep_path,
            graph,
            visited,
            resolution_stack,
            crate_paths,
            crate_names,
            crate_resolution_stack,
        )?;

        // Register the dependency alias
        graph.add_dependency(declaring_crate, alias, dep_crate_id);

        Ok(dep_crate_id)
    }

    /// Resolve a single module and its dependencies.
    ///
    /// # Arguments
    /// * `requested_path` - The path used to locate this module (may be relative)
    /// * `canonical_path` - The canonical path for deduplication
    /// * `graph` - The module graph being built
    /// * `visited` - Set of already-resolved module paths
    /// * `resolution_stack` - Stack of modules currently being resolved (for cycle detection)
    /// * `preparsed_module_file` - Root module AST retained from crate metadata parsing
    #[allow(clippy::too_many_arguments)]
    fn resolve_module(
        &self,
        requested_path: &Path,
        canonical_path: &Path,
        graph: &mut ModuleGraph,
        visited: &mut HashSet<PathBuf>,
        resolution_stack: &mut Vec<PathBuf>,
        crate_id: Option<CrateId>,
        preparsed_module_file: Option<ModuleFile>,
    ) -> Result<ModuleId, ResolveError> {
        // Check for circular dependencies
        if let Some(pos) = resolution_stack.iter().position(|p| p == canonical_path) {
            let cycle = resolution_stack[pos..]
                .iter()
                .map(|p| p.display().to_string())
                .chain(std::iter::once(canonical_path.display().to_string()))
                .collect::<Vec<_>>()
                .join(" -> ");
            return Err(ResolveError::CircularDependency { cycle });
        }

        // Check if already resolved - find existing module ID
        if visited.contains(canonical_path) {
            for (id, node) in &graph.nodes {
                #[allow(clippy::collapsible_if)]
                if let CoreModuleSource::File(file_path) = &node.source {
                    if Path::new(file_path) == canonical_path {
                        return Ok(*id);
                    }
                }
            }
        }

        // The root is already parsed while obtaining crate metadata. Every
        // other resolved file is read and parsed exactly once here.
        let module_file = match preparsed_module_file {
            Some(module_file) => module_file,
            None => {
                let content = self.fs.read_file(canonical_path).ok_or_else(|| {
                    ResolveError::ModuleNotFound {
                        module_name: requested_path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into(),
                        expected_path: requested_path.to_path_buf(),
                    }
                })?;
                self.parse_module_file(&content, canonical_path)?
            }
        };
        let module_decls = discover_module_declarations(&module_file, canonical_path)?;

        // Determine the module name
        let module_name = if let Some(file_stem) = canonical_path.file_stem() {
            if file_stem == "mod" {
                // Directory module: use parent directory name
                canonical_path
                    .parent()
                    .and_then(|p| p.file_stem())
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "mod".to_string())
            } else {
                // Regular file module: use file stem
                file_stem.to_string_lossy().into_owned()
            }
        } else {
            "unknown".to_string()
        };

        let source = CoreModuleSource::File(canonical_path.display().to_string());
        let node = ModuleNode::new(module_name, source);
        let module_id = graph.add_node(node);

        // Assign this module to the crate (if crate_id is provided)
        if let Some(cid) = crate_id {
            graph.assign_module_to_crate(module_id, cid);
        }

        visited.insert(canonical_path.to_path_buf());
        resolution_stack.push(canonical_path.to_path_buf());

        for declaration in module_decls {
            match declaration.source {
                DiscoveredModuleSource::File => {
                    let child_path = self.resolve_child_module_path(&declaration)?;
                    let child_id = self.resolve_module(
                        &child_path,
                        &child_path,
                        graph,
                        visited,
                        resolution_stack,
                        crate_id, // Propagate crate_id to children
                        None,
                    )?;
                    graph.add_edge(module_id, child_id);
                }
                DiscoveredModuleSource::Inline => {
                    let source = CoreModuleSource::Inline {
                        parent: module_id,
                        offset: declaration.span.start,
                    };
                    let child_id =
                        graph.add_node(ModuleNode::new(declaration.name.to_string(), source));
                    if let Some(cid) = crate_id {
                        graph.assign_module_to_crate(child_id, cid);
                    }
                    graph.add_edge(module_id, child_id);
                }
            }
        }

        resolution_stack.pop();

        Ok(module_id)
    }

    /// Parse one resolved source file into the authoritative module carrier.
    ///
    /// # Errors
    ///
    /// Returns [`ResolveError::ParseError`] if the source cannot be parsed as
    /// a [`ModuleFile`].
    fn parse_module_file(&self, content: &str, path: &Path) -> Result<ModuleFile, ResolveError> {
        crate::parse_surface_file_with_path(content, Some(path)).map_err(|errors| {
            let message = errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; ");
            ResolveError::ParseError {
                path: path.to_path_buf(),
                message,
            }
        })
    }

    /// Resolve the path for a child module.
    ///
    /// Tries `foo.ash` first, then `foo/mod.ash` (Rust-style).
    fn resolve_child_module_path(
        &self,
        declaration: &DiscoveredModuleDecl,
    ) -> Result<PathBuf, ResolveError> {
        let parent_dir = declaration.path.parent().unwrap_or(Path::new("."));

        // Try file module first: `foo.ash`
        let file_module = parent_dir.join(format!("{}.ash", declaration.name));
        if self.fs.file_exists(&file_module) {
            return Ok(file_module);
        }

        // Try directory module: `foo/mod.ash`
        let dir_module = parent_dir.join(&*declaration.name).join("mod.ash");
        if self.fs.file_exists(&dir_module) {
            return Ok(dir_module);
        }

        // Neither found - return error with the first expected path
        Err(ResolveError::ModuleNotFound {
            module_name: declaration.name.to_string(),
            expected_path: file_module,
        })
    }
}

/// Normalize a path by resolving `.` and `..` components.
/// This does not access the filesystem (unlike `canonicalize`).
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut result = Vec::new();

    // Preserve the prefix (e.g., Windows drive letter)
    if let Some(prefix) = components.peek().and_then(|c| match c {
        std::path::Component::Prefix(p) => Some(*p),
        _ => None,
    }) {
        result.push(std::path::Component::Prefix(prefix));
        components.next();
    }

    for component in components {
        match component {
            std::path::Component::Prefix(_) => {
                // Already handled above
            }
            std::path::Component::RootDir => {
                result.push(component);
            }
            std::path::Component::CurDir => {
                // Skip "."
            }
            std::path::Component::ParentDir => {
                // Pop the last component if it's not a RootDir
                if let Some(last) = result.last() {
                    match last {
                        std::path::Component::RootDir => {
                            // Can't go above root, keep the ..
                            result.push(component);
                        }
                        std::path::Component::Prefix(_) => {
                            // Can't go above prefix, keep the ..
                            result.push(component);
                        }
                        std::path::Component::ParentDir => {
                            // Multiple .. in a row, keep it
                            result.push(component);
                        }
                        _ => {
                            // Pop the normal component
                            result.pop();
                        }
                    }
                } else {
                    // Empty result, keep the ..
                    result.push(component);
                }
            }
            std::path::Component::Normal(name) => {
                result.push(std::path::Component::Normal(name));
            }
        }
    }

    result.into_iter().collect()
}

impl Default for LegacyModuleResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::ModuleSource as ParsedModuleSource;
    use crate::surface::Visibility;
    use proptest::prelude::*;
    use std::collections::HashMap;

    /// Mock file system for testing.
    struct MockFs {
        files: HashMap<PathBuf, String>,
    }

    impl MockFs {
        /// Create an empty mock file system.
        fn new() -> Self {
            Self {
                files: HashMap::new(),
            }
        }

        /// Add a file to the mock file system (builder pattern).
        fn with_file(mut self, path: impl AsRef<Path>, content: impl Into<String>) -> Self {
            self.files
                .insert(path.as_ref().to_path_buf(), content.into());
            self
        }
    }

    impl Fs for MockFs {
        fn read_file(&self, path: &Path) -> Option<String> {
            self.files.get(path).cloned()
        }

        fn file_exists(&self, path: &Path) -> bool {
            self.files.contains_key(path)
        }
    }

    // ========================================================================
    // Single File Tests
    // ========================================================================

    #[test]
    fn test_resolve_single_file_no_modules() {
        // Test: Resolving a single file with no module declarations
        let fs = MockFs::new().with_file("main.ash", "fn Main() {}");
        let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

        let graph = resolver.resolve_crate("main.ash").unwrap();

        assert_eq!(graph.nodes.len(), 1);
        assert!(graph.root.is_some());

        let root_id = graph.root.unwrap();
        let root_node = graph.get_node(root_id).unwrap();
        assert_eq!(root_node.name, "main");
        assert!(root_node.children.is_empty());
    }

    #[test]
    fn test_resolve_single_file_with_comments() {
        // Test: File with comments but no actual module declarations
        let fs = MockFs::new().with_file(
            "main.ash",
            "-- This is a comment\n-- mod fake;\nfn Main() {}",
        );
        let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

        let graph = resolver.resolve_crate("main.ash").unwrap();

        assert_eq!(graph.nodes.len(), 1);
        let root_id = graph.root.unwrap();
        let root_node = graph.get_node(root_id).unwrap();
        assert!(root_node.children.is_empty());
    }

    #[test]
    fn resolver_discovers_file_children_declared_by_module_file_ast() {
        let source = r#"
            mod private_child;
            pub mod public_child;
            pub(crate) mod crate_child;
            pub(super) mod parent_child;
            pub(in crate::nested) mod restricted_child;
            fn Main() {}
        "#;

        let module_file = crate::parse_surface_file(source)
            .expect("the declaration forms should parse into one ModuleFile");
        let declarations: Vec<_> = module_file
            .module_decls
            .iter()
            .map(|declaration| {
                (
                    declaration.name.as_ref(),
                    declaration.visibility.clone(),
                    declaration.source.clone(),
                )
            })
            .collect();
        assert_eq!(
            declarations,
            vec![
                (
                    "private_child",
                    Visibility::Inherited,
                    ParsedModuleSource::File,
                ),
                ("public_child", Visibility::Public, ParsedModuleSource::File,),
                ("crate_child", Visibility::Crate, ParsedModuleSource::File,),
                (
                    "parent_child",
                    Visibility::Super { levels: 1 },
                    ParsedModuleSource::File,
                ),
                (
                    "restricted_child",
                    Visibility::Restricted {
                        path: "crate::nested".into(),
                    },
                    ParsedModuleSource::File,
                ),
            ]
        );

        let fs = MockFs::new()
            .with_file("main.ash", source)
            .with_file("private_child.ash", "fn Private() {}")
            .with_file("public_child.ash", "fn Public() {}")
            .with_file("crate_child.ash", "fn Crate() {}")
            .with_file("parent_child.ash", "fn Parent() {}")
            .with_file("restricted_child.ash", "fn Restricted() {}");
        let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

        let graph = resolver
            .resolve_crate("main.ash")
            .expect("parsed file-module declarations should create child edges");
        let root = graph.get_node(graph.root.expect("a resolved crate has a root"));
        let child_names: Vec<_> = root
            .expect("root node should exist")
            .children
            .iter()
            .map(|child| {
                graph
                    .get_node(*child)
                    .expect("child node should exist")
                    .name
                    .as_str()
            })
            .collect();

        assert_eq!(
            child_names,
            vec![
                "private_child",
                "public_child",
                "crate_child",
                "parent_child",
                "restricted_child",
            ]
        );
    }

    #[test]
    fn comment_and_string_lookalikes_do_not_create_file_module_edges() {
        let fs = MockFs::new()
            .with_file(
                "main.ash",
                r#"
                    mod declared;
                    -- mod comment_lookalike;
                    fn Main() { "mod string_lookalike;" }
                "#,
            )
            .with_file("declared.ash", "fn Declared() {}")
            .with_file("comment_lookalike.ash", "fn CommentLookalike() {}")
            .with_file("string_lookalike.ash", "fn StringLookalike() {}");
        let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

        let graph = resolver
            .resolve_crate("main.ash")
            .expect("text lookalikes are not declarations and must not be resolved");
        let root = graph.get_node(graph.root.expect("a resolved crate has a root"));
        let child_names: Vec<_> = root
            .expect("root node should exist")
            .children
            .iter()
            .map(|child| {
                graph
                    .get_node(*child)
                    .expect("child node should exist")
                    .name
                    .as_str()
            })
            .collect();

        assert_eq!(child_names, vec!["declared"]);
    }

    proptest! {
        #[test]
        fn arbitrary_comment_or_string_lookalikes_do_not_change_discovered_child_keys(
            lookalike in "[a-z][a-z0-9_]{0,15}",
            use_string_literal in any::<bool>(),
        ) {
            let lookalike_line = if use_string_literal {
                format!("fn Main() {{ \"mod {lookalike};\" }}")
            } else {
                format!("-- mod {lookalike};\nfn Main() {{}}")
            };
            let source = format!("mod declared;\n{lookalike_line}");
            let fs = MockFs::new()
                .with_file("main.ash", source)
                .with_file("declared.ash", "fn Declared() {}")
                .with_file(format!("{lookalike}.ash"), "fn Lookalike() {}");
            let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

            let graph = resolver
                .resolve_crate("main.ash")
                .expect("comments and literals must remain non-authoritative");
            let root = graph.get_node(graph.root.expect("a resolved crate has a root"));
            let child_names: Vec<_> = root
                .expect("root node should exist")
                .children
                .iter()
                .map(|child| graph.get_node(*child).expect("child node should exist").name.as_str())
                .collect();

            prop_assert_eq!(child_names, vec!["declared"]);
        }
    }

    #[test]
    fn malformed_module_syntax_returns_parser_error_before_graph_construction() {
        let source = "mod child\nfn Main() {}";
        assert!(
            crate::parse_surface_file(source).is_err(),
            "the source must be rejected by the ModuleFile parser"
        );

        let fs = MockFs::new()
            .with_file("main.ash", source)
            .with_file("child.ash", "fn Child() {}");
        let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

        let result = resolver.resolve_crate("main.ash");
        assert!(
            matches!(result, Err(ResolveError::ParseError { ref path, .. }) if path == Path::new("main.ash")),
            "malformed source must fail through the parser before the resolver constructs a graph: {result:?}"
        );
    }

    #[test]
    fn duplicate_file_module_declarations_are_rejected() {
        let fs = MockFs::new()
            .with_file("main.ash", "mod child;\nmod child;\nfn Main() {}")
            .with_file("child.ash", "fn Child() {}");
        let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

        let result = resolver.resolve_crate("main.ash");
        assert!(
            result.is_err(),
            "duplicate parsed file-module declarations must be rejected instead of publishing duplicate child edges"
        );
    }

    // ========================================================================
    // Child Module Tests (File Module)
    // ========================================================================

    #[test]
    fn test_resolve_with_file_module() {
        // Test: `mod foo;` -> `foo.ash`
        let fs = MockFs::new()
            .with_file("main.ash", "mod foo;\nfn Main() {}")
            .with_file("foo.ash", "interface Bar { read() -> Unit }");
        let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

        let graph = resolver.resolve_crate("main.ash").unwrap();

        assert_eq!(graph.nodes.len(), 2);

        let root_id = graph.root.unwrap();
        let root_node = graph.get_node(root_id).unwrap();
        assert_eq!(root_node.name, "main");
        assert_eq!(root_node.children.len(), 1);

        let child_id = root_node.children[0];
        let child_node = graph.get_node(child_id).unwrap();
        assert_eq!(child_node.name, "foo");
    }

    #[test]
    fn test_resolve_with_pub_file_module() {
        // Test: `pub mod foo;` -> `foo.ash`
        let fs = MockFs::new()
            .with_file("main.ash", "pub mod foo;\nfn Main() {}")
            .with_file("foo.ash", "interface Bar { read() -> Unit }");
        let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

        let graph = resolver.resolve_crate("main.ash").unwrap();

        assert_eq!(graph.nodes.len(), 2);
        let root_id = graph.root.unwrap();
        let root_node = graph.get_node(root_id).unwrap();
        assert_eq!(root_node.children.len(), 1);
    }

    #[test]
    fn test_resolve_with_pub_crate_file_module() {
        // Test: `pub(crate) mod foo;` -> `foo.ash`
        let fs = MockFs::new()
            .with_file("main.ash", "pub(crate) mod foo;\nfn Main() {}")
            .with_file("foo.ash", "interface Bar { read() -> Unit }");
        let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

        let graph = resolver.resolve_crate("main.ash").unwrap();

        assert_eq!(graph.nodes.len(), 2);
    }

    #[test]
    fn test_resolve_multiple_file_modules() {
        // Test: Multiple file-based modules
        let fs = MockFs::new()
            .with_file("main.ash", "mod foo;\nmod bar;\nfn Main() {}")
            .with_file("foo.ash", "interface Foo { read() -> Unit }")
            .with_file("bar.ash", "interface Bar { read() -> Unit }");
        let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

        let graph = resolver.resolve_crate("main.ash").unwrap();

        assert_eq!(graph.nodes.len(), 3);

        let root_id = graph.root.unwrap();
        let root_node = graph.get_node(root_id).unwrap();
        assert_eq!(root_node.children.len(), 2);

        // Verify both children exist
        let child_names: Vec<_> = root_node
            .children
            .iter()
            .map(|&id| graph.get_node(id).unwrap().name.clone())
            .collect();
        assert!(child_names.contains(&"foo".to_string()));
        assert!(child_names.contains(&"bar".to_string()));
    }

    #[test]
    fn test_resolve_nested_file_modules() {
        // Test: Nested modules (file modules containing modules)
        let fs = MockFs::new()
            .with_file("main.ash", "mod foo;\nfn Main() {}")
            .with_file("foo.ash", "mod bar;\ninterface Foo { read() -> Unit }")
            .with_file("bar.ash", "interface Bar { read() -> Unit }");
        let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

        let graph = resolver.resolve_crate("main.ash").unwrap();

        assert_eq!(graph.nodes.len(), 3);

        let root_id = graph.root.unwrap();
        let root_node = graph.get_node(root_id).unwrap();
        assert_eq!(root_node.children.len(), 1);

        let foo_id = root_node.children[0];
        let foo_node = graph.get_node(foo_id).unwrap();
        assert_eq!(foo_node.name, "foo");
        assert_eq!(foo_node.children.len(), 1);

        let bar_id = foo_node.children[0];
        let bar_node = graph.get_node(bar_id).unwrap();
        assert_eq!(bar_node.name, "bar");
    }

    // ========================================================================
    // Directory Module Tests
    // ========================================================================

    #[test]
    fn test_resolve_with_directory_module() {
        // Test: `mod foo;` -> `foo/mod.ash` (directory module)
        let fs = MockFs::new()
            .with_file("main.ash", "mod utils;\nfn Main() {}")
            .with_file("utils/mod.ash", "interface Utils { read() -> Unit }");
        let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

        let graph = resolver.resolve_crate("main.ash").unwrap();

        assert_eq!(graph.nodes.len(), 2);

        let root_id = graph.root.unwrap();
        let root_node = graph.get_node(root_id).unwrap();
        assert_eq!(root_node.children.len(), 1);

        let child_id = root_node.children[0];
        let child_node = graph.get_node(child_id).unwrap();
        assert_eq!(child_node.name, "utils"); // Directory name, not "mod"
    }

    #[test]
    fn test_resolve_file_module_preferred_over_directory() {
        // Test: `foo.ash` takes precedence over `foo/mod.ash`
        let fs = MockFs::new()
            .with_file("main.ash", "mod foo;\nfn Main() {}")
            .with_file("foo.ash", "-- File module")
            .with_file("foo/mod.ash", "-- Directory module");
        let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

        let graph = resolver.resolve_crate("main.ash").unwrap();

        assert_eq!(graph.nodes.len(), 2);

        let root_id = graph.root.unwrap();
        let root_node = graph.get_node(root_id).unwrap();
        let child_id = root_node.children[0];
        let child_node = graph.get_node(child_id).unwrap();

        // Should resolve to file module, not directory
        assert_eq!(child_node.name, "foo");
        assert_eq!(child_node.source, CoreModuleSource::File("foo.ash".into()));
    }

    #[test]
    fn test_resolve_directory_module_with_children() {
        // Test: Directory module can have its own children
        let fs = MockFs::new()
            .with_file("main.ash", "mod utils;\nfn Main() {}")
            .with_file(
                "utils/mod.ash",
                "mod helpers;\ninterface Utils { read() -> Unit }",
            )
            .with_file("utils/helpers.ash", "interface Help { read() -> Unit }");
        let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

        let graph = resolver.resolve_crate("main.ash").unwrap();

        assert_eq!(graph.nodes.len(), 3);

        let root_id = graph.root.unwrap();
        let root_node = graph.get_node(root_id).unwrap();
        assert_eq!(root_node.children.len(), 1);

        let utils_id = root_node.children[0];
        let utils_node = graph.get_node(utils_id).unwrap();
        assert_eq!(utils_node.name, "utils");
        assert_eq!(utils_node.children.len(), 1);
    }

    // ========================================================================
    // Circular Dependency Tests
    // ========================================================================

    #[test]
    fn test_detect_circular_dependency_two_modules() {
        // Test: A -> B -> A
        let fs = MockFs::new()
            .with_file("a.ash", "mod b;\nfn A() {}")
            .with_file("b.ash", "mod a;\nfn B() {}");
        let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

        let result = resolver.resolve_crate("a.ash");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ResolveError::CircularDependency { .. }));
        let err_str = err.to_string();
        assert!(err_str.contains("circular dependency"));
    }

    #[test]
    fn test_detect_circular_dependency_three_modules() {
        // Test: A -> B -> C -> A
        let fs = MockFs::new()
            .with_file("a.ash", "mod b;\nfn A() {}")
            .with_file("b.ash", "mod c;\nfn B() {}")
            .with_file("c.ash", "mod a;\nfn C() {}");
        let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

        let result = resolver.resolve_crate("a.ash");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ResolveError::CircularDependency { .. }));
    }

    #[test]
    fn test_detect_self_reference() {
        // Test: A -> A (self-referential)
        let fs = MockFs::new().with_file("a.ash", "mod a;\nfn A() {}");
        let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

        let result = resolver.resolve_crate("a.ash");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ResolveError::CircularDependency { .. }));
    }

    // ========================================================================
    // Error Cases
    // ========================================================================

    #[test]
    fn test_module_not_found() {
        // Test: Module declared but file doesn't exist
        let fs = MockFs::new().with_file("main.ash", "mod missing;\nfn Main() {}");
        let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

        let result = resolver.resolve_crate("main.ash");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ResolveError::ModuleNotFound { .. }));
        let err_str = err.to_string();
        assert!(err_str.contains("missing"));
    }

    #[test]
    fn test_root_file_not_found() {
        // Test: Root file doesn't exist
        let fs = MockFs::new();
        let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

        let result = resolver.resolve_crate("nonexistent.ash");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ResolveError::ModuleNotFound { .. }));
    }

    #[test]
    fn test_module_not_found_shows_expected_path() {
        // Test: Error message includes expected path
        let fs = MockFs::new().with_file("main.ash", "mod foo;\nfn Main() {}");
        let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

        let result = resolver.resolve_crate("main.ash");

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(err_str.contains("foo"));
        assert!(err_str.contains("foo.ash"));
    }

    // ========================================================================
    // Inline Module Tests
    // ========================================================================

    #[test]
    fn test_inline_modules_publish_structural_children_without_file_resolution() {
        // Inline modules create structural graph children without filesystem acquisition.
        let fs = MockFs::new().with_file(
            "main.ash",
            "mod foo { interface Bar { read() -> Unit } }\nfn Main() {}",
        );
        let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

        let graph = resolver.resolve_crate("main.ash").unwrap();

        assert_eq!(graph.nodes.len(), 2);
        let root_id = graph.root.unwrap();
        let root_node = graph.get_node(root_id).unwrap();
        assert_eq!(root_node.children.len(), 1);
        let inline_child = graph.get_node(root_node.children[0]).unwrap();
        assert_eq!(inline_child.name, "foo");
        assert_eq!(
            inline_child.source,
            CoreModuleSource::Inline {
                parent: root_id,
                offset: 0,
            }
        );
    }

    // ========================================================================
    // Complex Scenario Tests
    // ========================================================================

    #[test]
    fn test_complex_module_tree() {
        // Test: Complex tree with both file and directory modules
        let fs = MockFs::new()
            .with_file("src/main.ash", "mod core;\nmod utils;\nfn Main() {}")
            .with_file(
                "src/core.ash",
                "mod types;\ninterface Core { read() -> Unit }",
            )
            .with_file("src/types.ash", "interface Types { read() -> Unit }")
            .with_file(
                "src/utils/mod.ash",
                "mod helpers;\ninterface Utils { read() -> Unit }",
            )
            .with_file(
                "src/utils/helpers.ash",
                "interface Helpers { read() -> Unit }",
            );
        let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

        let graph = resolver.resolve_crate("src/main.ash").unwrap();

        assert_eq!(graph.nodes.len(), 5);

        let root_id = graph.root.unwrap();
        let root_node = graph.get_node(root_id).unwrap();
        assert_eq!(root_node.children.len(), 2); // core, utils

        // Find utils node and check its children
        let utils_id = root_node
            .children
            .iter()
            .find(|&&id| graph.get_node(id).unwrap().name == "utils")
            .copied();
        assert!(utils_id.is_some());

        let utils_node = graph.get_node(utils_id.unwrap()).unwrap();
        assert_eq!(utils_node.children.len(), 1); // helpers
    }

    #[test]
    fn test_shared_module_not_duplicated() {
        // Test: Same module imported from multiple places is not duplicated
        let fs = MockFs::new()
            .with_file("main.ash", "mod a;\nmod b;\nfn Main() {}")
            .with_file("a.ash", "mod shared;\nfn A() {}")
            .with_file("b.ash", "mod shared;\nfn B() {}")
            .with_file("shared.ash", "interface Shared { read() -> Unit }");
        let resolver = LegacyModuleResolver::with_fs(Box::new(fs));

        let graph = resolver.resolve_crate("main.ash").unwrap();

        // Should have 4 modules, not 5 (shared should be shared)
        assert_eq!(graph.nodes.len(), 4);

        let root_id = graph.root.unwrap();
        let root_node = graph.get_node(root_id).unwrap();
        assert_eq!(root_node.children.len(), 2); // a, b
    }
}
