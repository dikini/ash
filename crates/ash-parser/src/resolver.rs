//! Module resolution algorithm for the Ash parser.
//!
//! This module provides functionality to discover and resolve module dependencies
//! in Ash source files. It supports Rust-style module resolution where `mod foo;`
//! looks for `foo.ash` or `foo/mod.ash`.
//!
//! This resolver also supports multi-crate resolution with dependency management.

use ash_core::module_graph::{
    CrateId, ModuleGraph, ModuleId, ModuleNode, ModuleSource as CoreModuleSource,
};
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::input::new_input;
use crate::parse_crate_root::parse_crate_root_metadata;
use crate::surface::DependencyDecl;

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

/// Module resolver that discovers and resolves module dependencies.
///
/// The resolver walks the module hierarchy starting from a root file,
/// parsing `mod foo;` declarations and locating the corresponding files.
/// It supports both file modules (`foo.ash`) and directory modules
/// (`foo/mod.ash`), following Rust's module resolution convention.
pub struct ModuleResolver {
    fs: Box<dyn Fs>,
}

impl ModuleResolver {
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

        // Parse crate root metadata (or use default if not present)
        // If the file starts with a crate declaration, parse it strictly
        // Otherwise, use the file stem as the crate name for backward compatibility
        let metadata = if content.trim().starts_with("crate") {
            self.parse_crate_root_metadata(&content, canonical_path)?
        } else {
            crate::surface::CrateRootMetadata {
                crate_name: canonical_path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into(),
                dependencies: Vec::new(),
                span: crate::token::Span::new(0, 0, 1, 1),
            }
        };

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

        // Resolve the root module of this crate (which also resolves submodules)
        let root_module_id =
            self.resolve_module(root_path, canonical_path, graph, visited, resolution_stack)?;

        // Add this crate to the graph
        let root_path_str = canonical_path.display().to_string();
        let crate_id = graph.add_crate(crate_name_str.clone(), root_path_str, root_module_id);

        // Track this crate
        crate_paths.insert(canonical_path.to_path_buf(), crate_id);
        crate_names.insert(crate_name_str.clone(), crate_id);

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

    /// Parse crate root metadata from file content.
    fn parse_crate_root_metadata(
        &self,
        content: &str,
        path: &Path,
    ) -> Result<crate::surface::CrateRootMetadata, ResolveError> {
        let mut input = new_input(content);

        parse_crate_root_metadata(&mut input).map_err(|_e| ResolveError::ParseError {
            path: path.to_path_buf(),
            message: "Failed to parse crate root metadata".to_string(),
        })
    }

    /// Resolve a single module and its dependencies.
    ///
    /// # Arguments
    /// * `requested_path` - The path used to locate this module (may be relative)
    /// * `canonical_path` - The canonical path for deduplication
    /// * `graph` - The module graph being built
    /// * `visited` - Set of already-resolved module paths
    /// * `resolution_stack` - Stack of modules currently being resolved (for cycle detection)
    fn resolve_module(
        &self,
        requested_path: &Path,
        canonical_path: &Path,
        graph: &mut ModuleGraph,
        visited: &mut HashSet<PathBuf>,
        resolution_stack: &mut Vec<PathBuf>,
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

        // Read the file
        let content =
            self.fs
                .read_file(canonical_path)
                .ok_or_else(|| ResolveError::ModuleNotFound {
                    module_name: requested_path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into(),
                    expected_path: requested_path.to_path_buf(),
                })?;

        // Parse module declarations from content
        let module_decls = self.parse_module_decls(&content, canonical_path)?;

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

        visited.insert(canonical_path.to_path_buf());
        resolution_stack.push(canonical_path.to_path_buf());

        // Resolve child modules
        for decl in module_decls {
            let child_path = self.resolve_child_module_path(canonical_path, &decl)?;
            let child_id =
                self.resolve_module(&child_path, &child_path, graph, visited, resolution_stack)?;
            graph.add_edge(module_id, child_id);
        }

        resolution_stack.pop();

        Ok(module_id)
    }

    /// Parse module declarations from file content.
    fn parse_module_decls(&self, content: &str, _path: &Path) -> Result<Vec<String>, ResolveError> {
        let mut decls = Vec::new();

        // Simple parsing: look for `mod <name>;` or `pub mod <name>;` patterns
        // This is a simplified approach - in production, we'd use the full parser
        for line in content.lines() {
            let trimmed = line.trim();

            // Skip comments
            if trimmed.starts_with("--") {
                continue;
            }

            // Check for module declaration pattern
            // Matches: `mod foo;`, `pub mod foo;`, `pub(crate) mod foo;`, etc.
            if let Some(mod_pos) = trimmed.find("mod ") {
                // Simple heuristic: look for "mod " followed by identifier
                let after_mod = &trimmed[mod_pos + 4..];

                // Extract the module name (identifier)
                let name_end = after_mod
                    .find(|c: char| c.is_whitespace() || c == ';' || c == '{')
                    .unwrap_or(after_mod.len());
                let name = &after_mod[..name_end];

                // Check if it's a file-based module (ends with `;`)
                if after_mod[name_end..].trim_start().starts_with(';') && !name.is_empty() {
                    decls.push(name.to_string());
                }
            }
        }

        Ok(decls)
    }

    /// Resolve the path for a child module.
    ///
    /// Tries `foo.ash` first, then `foo/mod.ash` (Rust-style).
    fn resolve_child_module_path(
        &self,
        parent_path: &Path,
        module_name: &str,
    ) -> Result<PathBuf, ResolveError> {
        let parent_dir = parent_path.parent().unwrap_or(Path::new("."));

        // Try file module first: `foo.ash`
        let file_module = parent_dir.join(format!("{}.ash", module_name));
        if self.fs.file_exists(&file_module) {
            return Ok(file_module);
        }

        // Try directory module: `foo/mod.ash`
        let dir_module = parent_dir.join(module_name).join("mod.ash");
        if self.fs.file_exists(&dir_module) {
            return Ok(dir_module);
        }

        // Neither found - return error with the first expected path
        Err(ResolveError::ModuleNotFound {
            module_name: module_name.to_string(),
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

impl Default for ModuleResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let fs = MockFs::new().with_file("main.ash", "workflow Main {}");
        let resolver = ModuleResolver::with_fs(Box::new(fs));

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
            "-- This is a comment\n-- mod fake;\nworkflow Main {}",
        );
        let resolver = ModuleResolver::with_fs(Box::new(fs));

        let graph = resolver.resolve_crate("main.ash").unwrap();

        assert_eq!(graph.nodes.len(), 1);
        let root_id = graph.root.unwrap();
        let root_node = graph.get_node(root_id).unwrap();
        assert!(root_node.children.is_empty());
    }

    // ========================================================================
    // Child Module Tests (File Module)
    // ========================================================================

    #[test]
    fn test_resolve_with_file_module() {
        // Test: `mod foo;` -> `foo.ash`
        let fs = MockFs::new()
            .with_file("main.ash", "mod foo;\nworkflow Main {}")
            .with_file("foo.ash", "capability Bar: observe();");
        let resolver = ModuleResolver::with_fs(Box::new(fs));

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
            .with_file("main.ash", "pub mod foo;\nworkflow Main {}")
            .with_file("foo.ash", "capability Bar: observe();");
        let resolver = ModuleResolver::with_fs(Box::new(fs));

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
            .with_file("main.ash", "pub(crate) mod foo;\nworkflow Main {}")
            .with_file("foo.ash", "capability Bar: observe();");
        let resolver = ModuleResolver::with_fs(Box::new(fs));

        let graph = resolver.resolve_crate("main.ash").unwrap();

        assert_eq!(graph.nodes.len(), 2);
    }

    #[test]
    fn test_resolve_multiple_file_modules() {
        // Test: Multiple file-based modules
        let fs = MockFs::new()
            .with_file("main.ash", "mod foo;\nmod bar;\nworkflow Main {}")
            .with_file("foo.ash", "capability Foo: observe();")
            .with_file("bar.ash", "capability Bar: observe();");
        let resolver = ModuleResolver::with_fs(Box::new(fs));

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
            .with_file("main.ash", "mod foo;\nworkflow Main {}")
            .with_file("foo.ash", "mod bar;\ncapability Foo: observe();")
            .with_file("bar.ash", "capability Bar: observe();");
        let resolver = ModuleResolver::with_fs(Box::new(fs));

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
            .with_file("main.ash", "mod utils;\nworkflow Main {}")
            .with_file("utils/mod.ash", "capability Utils: observe();");
        let resolver = ModuleResolver::with_fs(Box::new(fs));

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
            .with_file("main.ash", "mod foo;\nworkflow Main {}")
            .with_file("foo.ash", "-- File module")
            .with_file("foo/mod.ash", "-- Directory module");
        let resolver = ModuleResolver::with_fs(Box::new(fs));

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
            .with_file("main.ash", "mod utils;\nworkflow Main {}")
            .with_file(
                "utils/mod.ash",
                "mod helpers;\ncapability Utils: observe();",
            )
            .with_file("utils/helpers.ash", "capability Help: observe();");
        let resolver = ModuleResolver::with_fs(Box::new(fs));

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
            .with_file("a.ash", "mod b;\nworkflow A {}")
            .with_file("b.ash", "mod a;\nworkflow B {}");
        let resolver = ModuleResolver::with_fs(Box::new(fs));

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
            .with_file("a.ash", "mod b;\nworkflow A {}")
            .with_file("b.ash", "mod c;\nworkflow B {}")
            .with_file("c.ash", "mod a;\nworkflow C {}");
        let resolver = ModuleResolver::with_fs(Box::new(fs));

        let result = resolver.resolve_crate("a.ash");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ResolveError::CircularDependency { .. }));
    }

    #[test]
    fn test_detect_self_reference() {
        // Test: A -> A (self-referential)
        let fs = MockFs::new().with_file("a.ash", "mod a;\nworkflow A {}");
        let resolver = ModuleResolver::with_fs(Box::new(fs));

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
        let fs = MockFs::new().with_file("main.ash", "mod missing;\nworkflow Main {}");
        let resolver = ModuleResolver::with_fs(Box::new(fs));

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
        let resolver = ModuleResolver::with_fs(Box::new(fs));

        let result = resolver.resolve_crate("nonexistent.ash");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ResolveError::ModuleNotFound { .. }));
    }

    #[test]
    fn test_module_not_found_shows_expected_path() {
        // Test: Error message includes expected path
        let fs = MockFs::new().with_file("main.ash", "mod foo;\nworkflow Main {}");
        let resolver = ModuleResolver::with_fs(Box::new(fs));

        let result = resolver.resolve_crate("main.ash");

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(err_str.contains("foo"));
        assert!(err_str.contains("foo.ash"));
    }

    // ========================================================================
    // Inline Module Tests (should be ignored for file resolution)
    // ========================================================================

    #[test]
    fn test_inline_modules_ignored() {
        // Test: Inline modules (mod foo { ... }) should not trigger file resolution
        let fs = MockFs::new().with_file(
            "main.ash",
            "mod foo { capability Bar: observe(); }\nworkflow Main {}",
        );
        let resolver = ModuleResolver::with_fs(Box::new(fs));

        let graph = resolver.resolve_crate("main.ash").unwrap();

        // Should only have main module, no children from inline
        assert_eq!(graph.nodes.len(), 1);
        let root_id = graph.root.unwrap();
        let root_node = graph.get_node(root_id).unwrap();
        assert!(root_node.children.is_empty());
    }

    // ========================================================================
    // Complex Scenario Tests
    // ========================================================================

    #[test]
    fn test_complex_module_tree() {
        // Test: Complex tree with both file and directory modules
        let fs = MockFs::new()
            .with_file("src/main.ash", "mod core;\nmod utils;\nworkflow Main {}")
            .with_file("src/core.ash", "mod types;\ncapability Core: observe();")
            .with_file("src/types.ash", "capability Types: observe();")
            .with_file(
                "src/utils/mod.ash",
                "mod helpers;\ncapability Utils: observe();",
            )
            .with_file("src/utils/helpers.ash", "capability Helpers: observe();");
        let resolver = ModuleResolver::with_fs(Box::new(fs));

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
            .with_file("main.ash", "mod a;\nmod b;\nworkflow Main {}")
            .with_file("a.ash", "mod shared;\nworkflow A {}")
            .with_file("b.ash", "mod shared;\nworkflow B {}")
            .with_file("shared.ash", "capability Shared: observe();");
        let resolver = ModuleResolver::with_fs(Box::new(fs));

        let graph = resolver.resolve_crate("main.ash").unwrap();

        // Should have 4 modules, not 5 (shared should be shared)
        assert_eq!(graph.nodes.len(), 4);

        let root_id = graph.root.unwrap();
        let root_node = graph.get_node(root_id).unwrap();
        assert_eq!(root_node.children.len(), 2); // a, b
    }
}
