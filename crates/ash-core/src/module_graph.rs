//! Module graph for tracking module dependencies and hierarchy
//!
//! The module graph tracks:
//! - Module hierarchy (parent-child relationships)
//! - Import dependencies between modules
//! - Source origins (file or inline)
//! - Crate identity and cross-crate dependencies

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a module in the graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModuleId(pub usize);

/// Unique identifier for a crate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CrateId(pub usize);

/// Information about a crate in the module graph
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrateInfo {
    /// Name of the crate
    pub name: String,
    /// Root module of the crate
    pub root_module: ModuleId,
    /// Root path of the crate (e.g., directory containing Cargo.toml or main file)
    pub root_path: String,
    /// Dependencies: alias -> crate id
    pub dependencies: HashMap<String, CrateId>,
}

impl CrateInfo {
    /// Create a new CrateInfo with the given name, root module, and root path
    pub fn new(name: String, root_module: ModuleId, root_path: String) -> Self {
        Self {
            name,
            root_module,
            root_path,
            dependencies: HashMap::new(),
        }
    }
}

/// Source of a module's content
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleSource {
    /// Module loaded from a file
    File(String),
    /// Module defined inline within another module
    Inline {
        /// Parent module that contains this inline module
        parent: ModuleId,
        /// Byte offset within the parent module's source
        offset: usize,
    },
}

/// A node in the module graph representing a single module
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleNode {
    /// Name of the module
    pub name: String,
    /// Source origin of the module
    pub source: ModuleSource,
    /// Child modules (submodules defined within this module)
    pub children: Vec<ModuleId>,
    /// Modules imported by this module
    pub imports: Vec<ModuleId>,
    /// Parent module (if any) - set when added via add_edge
    pub parent: Option<ModuleId>,
}

/// Graph structure tracking all modules and their relationships
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ModuleGraph {
    /// All modules in the graph, keyed by their ID
    pub nodes: HashMap<ModuleId, ModuleNode>,
    /// Root module of the graph (entry point)
    pub root: Option<ModuleId>,
    /// Next available module ID
    next_id: usize,
    /// All crates in the graph, keyed by their ID
    pub crates: HashMap<CrateId, CrateInfo>,
    /// Track which crate each module belongs to
    module_to_crate: HashMap<ModuleId, CrateId>,
    /// Next available crate ID
    next_crate_id: usize,
}

impl ModuleNode {
    /// Create a new module node with the given name and source
    pub fn new(name: String, source: ModuleSource) -> Self {
        Self {
            name,
            source,
            children: Vec::new(),
            imports: Vec::new(),
            parent: None,
        }
    }
}

impl ModuleGraph {
    /// Create a new empty module graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root: None,
            next_id: 0,
            crates: HashMap::new(),
            module_to_crate: HashMap::new(),
            next_crate_id: 0,
        }
    }

    /// Add a crate to the graph and return its assigned ID
    pub fn add_crate(&mut self, name: String, root_path: String, root_module: ModuleId) -> CrateId {
        let id = CrateId(self.next_crate_id);
        self.next_crate_id += 1;
        let crate_info = CrateInfo::new(name, root_module, root_path);
        self.crates.insert(id, crate_info);
        // Also associate the root module with this crate
        self.module_to_crate.insert(root_module, id);
        id
    }

    /// Get the crate ID for a module (lookup which crate owns the module)
    pub fn crate_id_for_module(&self, module: ModuleId) -> Option<CrateId> {
        self.module_to_crate.get(&module).copied()
    }

    /// Get the name of a crate by its ID
    pub fn crate_name(&self, crate_id: CrateId) -> Option<&str> {
        self.crates.get(&crate_id).map(|info| info.name.as_str())
    }

    /// Get a reference to crate info by ID
    pub fn get_crate(&self, crate_id: CrateId) -> Option<&CrateInfo> {
        self.crates.get(&crate_id)
    }

    /// Get a mutable reference to crate info by ID
    pub fn get_crate_mut(&mut self, crate_id: CrateId) -> Option<&mut CrateInfo> {
        self.crates.get_mut(&crate_id)
    }

    /// Add a dependency to a crate
    /// The alias is how the dependency is referred to within this crate
    pub fn add_dependency(&mut self, crate_id: CrateId, alias: String, target: CrateId) {
        if let Some(crate_info) = self.crates.get_mut(&crate_id) {
            crate_info.dependencies.insert(alias, target);
        }
    }

    /// Lookup a dependency target by alias within a crate
    /// Returns the CrateId of the dependency if found
    pub fn dependency_target(&self, crate_id: CrateId, alias: &str) -> Option<CrateId> {
        self.crates
            .get(&crate_id)
            .and_then(|info| info.dependencies.get(alias).copied())
    }

    /// Associate a module with a crate
    pub fn assign_module_to_crate(&mut self, module: ModuleId, crate_id: CrateId) {
        self.module_to_crate.insert(module, crate_id);
    }

    /// Get the crate for a module (legacy method name, delegates to crate_id_for_module)
    pub fn crate_for(&self, module: ModuleId) -> Option<CrateId> {
        self.crate_id_for_module(module)
    }

    /// Set crate membership for a module (legacy method name, delegates to assign_module_to_crate)
    pub fn set_crate(&mut self, module: ModuleId, crate_id: CrateId) {
        self.assign_module_to_crate(module, crate_id);
    }

    /// Add a node to the graph and return its assigned ID
    pub fn add_node(&mut self, node: ModuleNode) -> ModuleId {
        let id = ModuleId(self.next_id);
        self.next_id += 1;
        self.nodes.insert(id, node);
        id
    }

    /// Get a reference to a node by its ID
    pub fn get_node(&self, id: ModuleId) -> Option<&ModuleNode> {
        self.nodes.get(&id)
    }

    /// Add an edge (parent-child relationship) between two modules
    /// The child is added to the parent's children list
    /// The parent is set on the child's parent reference
    pub fn add_edge(&mut self, parent: ModuleId, child: ModuleId) {
        if let Some(parent_node) = self.nodes.get_mut(&parent) {
            parent_node.children.push(child);
        }
        // Set parent reference on child
        if let Some(child_node) = self.nodes.get_mut(&child) {
            child_node.parent = Some(parent);
        }
    }

    /// Set the root module of the graph
    pub fn set_root(&mut self, root: ModuleId) {
        self.root = Some(root);
    }

    /// Get the root module ID if set
    pub fn get_root(&self) -> Option<&ModuleId> {
        self.root.as_ref()
    }

    /// Get the root module node if set
    pub fn get_root_node(&self) -> Option<&ModuleNode> {
        self.root.and_then(|id| self.nodes.get(&id))
    }

    /// Add an import relationship (module imports another module)
    pub fn add_import(&mut self, importer: ModuleId, imported: ModuleId) {
        if let Some(importer_node) = self.nodes.get_mut(&importer) {
            importer_node.imports.push(imported);
        }
    }

    /// Iterator over ancestors from module up to root
    /// Returns an iterator that yields the module itself, then its parent, grandparent, etc.
    pub fn ancestors(&self, module: ModuleId) -> impl Iterator<Item = ModuleId> + '_ {
        std::iter::successors(Some(module), |&m| self.nodes.get(&m).and_then(|n| n.parent))
    }

    /// Resolve a path string to a ModuleId
    ///
    /// Path format: ["crate", "foo", "bar"] representing crate::foo::bar
    /// Returns None if the path cannot be resolved.
    pub fn resolve_path(&self, path: &[String]) -> Option<ModuleId> {
        // Start from root
        let mut current = self.root?;

        for component in path {
            // Handle "crate" as the root module name
            if component == "crate" {
                // Check if current is root (for the first component)
                if current == self.root? {
                    continue;
                }
            }

            // Find child with matching name
            let node = self.nodes.get(&current)?;
            let child = node.children.iter().find(|&&child_id| {
                self.nodes
                    .get(&child_id)
                    .map(|n| &n.name == component)
                    .unwrap_or(false)
            })?;
            current = *child;
        }
        Some(current)
    }

    /// Check if `module` is a descendant of (or the same as) `ancestor`
    ///
    /// Returns true if:
    /// - module == ancestor (same module)
    /// - module is a child, grandchild, etc. of ancestor
    pub fn is_descendant_or_same(&self, module: ModuleId, ancestor: ModuleId) -> bool {
        if module == ancestor {
            return true;
        }
        // Check if ancestor appears in module's ancestor chain
        self.ancestors(module).any(|m| m == ancestor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // ModuleNode Creation Tests
    // ============================================================

    #[test]
    fn test_create_module_node_with_file_source() {
        let node = ModuleNode::new("main".into(), ModuleSource::File("main.ash".into()));
        assert_eq!(node.name, "main");
        assert_eq!(node.source, ModuleSource::File("main.ash".into()));
        assert!(node.children.is_empty());
        assert!(node.imports.is_empty());
    }

    #[test]
    fn test_create_module_node_with_inline_source() {
        let parent_id = ModuleId(0);
        let node = ModuleNode::new(
            "inline_mod".into(),
            ModuleSource::Inline {
                parent: parent_id,
                offset: 42,
            },
        );
        assert_eq!(node.name, "inline_mod");
        assert_eq!(
            node.source,
            ModuleSource::Inline {
                parent: parent_id,
                offset: 42
            }
        );
        assert!(node.children.is_empty());
        assert!(node.imports.is_empty());
    }

    // ============================================================
    // ModuleGraph Node Management Tests
    // ============================================================

    #[test]
    fn test_create_empty_graph() {
        let graph = ModuleGraph::new();
        assert!(graph.nodes.is_empty());
        assert!(graph.root.is_none());
    }

    #[test]
    fn test_add_node_to_graph() {
        let mut graph = ModuleGraph::new();
        let node = ModuleNode::new("main".into(), ModuleSource::File("main.ash".into()));
        let id = graph.add_node(node);

        assert_eq!(graph.nodes.len(), 1);
        assert!(graph.nodes.contains_key(&id));
        assert_eq!(graph.nodes[&id].name, "main");
    }

    #[test]
    fn test_add_multiple_nodes() {
        let mut graph = ModuleGraph::new();
        let node1 = ModuleNode::new("a".into(), ModuleSource::File("a.ash".into()));
        let node2 = ModuleNode::new("b".into(), ModuleSource::File("b.ash".into()));

        let id1 = graph.add_node(node1);
        let id2 = graph.add_node(node2);

        assert_eq!(graph.nodes.len(), 2);
        assert_ne!(id1, id2);
        assert_eq!(graph.nodes[&id1].name, "a");
        assert_eq!(graph.nodes[&id2].name, "b");
    }

    #[test]
    fn test_get_node() {
        let mut graph = ModuleGraph::new();
        let node = ModuleNode::new("test".into(), ModuleSource::File("test.ash".into()));
        let id = graph.add_node(node);

        assert!(graph.get_node(id).is_some());
        assert_eq!(graph.get_node(id).unwrap().name, "test");

        let nonexistent = ModuleId(999);
        assert!(graph.get_node(nonexistent).is_none());
    }

    // ============================================================
    // ModuleGraph Edge (Parent-Child) Tests
    // ============================================================

    #[test]
    fn test_add_edge_parent_child() {
        let mut graph = ModuleGraph::new();
        let root = graph.add_node(ModuleNode::new(
            "main".into(),
            ModuleSource::File("main.ash".into()),
        ));
        let child = graph.add_node(ModuleNode::new(
            "foo".into(),
            ModuleSource::File("foo.ash".into()),
        ));

        graph.add_edge(root, child);

        assert_eq!(graph.nodes[&root].children, vec![child]);
    }

    #[test]
    fn test_add_multiple_children() {
        let mut graph = ModuleGraph::new();
        let root = graph.add_node(ModuleNode::new(
            "main".into(),
            ModuleSource::File("main.ash".into()),
        ));
        let child1 = graph.add_node(ModuleNode::new(
            "a".into(),
            ModuleSource::File("a.ash".into()),
        ));
        let child2 = graph.add_node(ModuleNode::new(
            "b".into(),
            ModuleSource::File("b.ash".into()),
        ));

        graph.add_edge(root, child1);
        graph.add_edge(root, child2);

        assert_eq!(graph.nodes[&root].children.len(), 2);
        assert!(graph.nodes[&root].children.contains(&child1));
        assert!(graph.nodes[&root].children.contains(&child2));
    }

    // ============================================================
    // Root Module Tracking Tests
    // ============================================================

    #[test]
    fn test_set_root() {
        let mut graph = ModuleGraph::new();
        let root = graph.add_node(ModuleNode::new(
            "main".into(),
            ModuleSource::File("main.ash".into()),
        ));

        graph.set_root(root);

        assert_eq!(graph.root, Some(root));
    }

    #[test]
    fn test_get_root() {
        let mut graph = ModuleGraph::new();
        let root = graph.add_node(ModuleNode::new(
            "main".into(),
            ModuleSource::File("main.ash".into()),
        ));
        let _child = graph.add_node(ModuleNode::new(
            "child".into(),
            ModuleSource::File("child.ash".into()),
        ));

        graph.set_root(root);

        assert_eq!(graph.get_root(), Some(&root));
        assert!(graph.get_root_node().is_some());
        assert_eq!(graph.get_root_node().unwrap().name, "main");
    }

    #[test]
    fn test_root_not_set_initially() {
        let graph = ModuleGraph::new();
        assert!(graph.get_root().is_none());
        assert!(graph.get_root_node().is_none());
    }

    // ============================================================
    // Import Tests
    // ============================================================

    #[test]
    fn test_add_import() {
        let mut graph = ModuleGraph::new();
        let importer = graph.add_node(ModuleNode::new(
            "main".into(),
            ModuleSource::File("main.ash".into()),
        ));
        let imported = graph.add_node(ModuleNode::new(
            "lib".into(),
            ModuleSource::File("lib.ash".into()),
        ));

        graph.add_import(importer, imported);

        assert_eq!(graph.nodes[&importer].imports, vec![imported]);
    }

    #[test]
    fn test_add_multiple_imports() {
        let mut graph = ModuleGraph::new();
        let main = graph.add_node(ModuleNode::new(
            "main".into(),
            ModuleSource::File("main.ash".into()),
        ));
        let lib1 = graph.add_node(ModuleNode::new(
            "lib1".into(),
            ModuleSource::File("lib1.ash".into()),
        ));
        let lib2 = graph.add_node(ModuleNode::new(
            "lib2".into(),
            ModuleSource::File("lib2.ash".into()),
        ));

        graph.add_import(main, lib1);
        graph.add_import(main, lib2);

        assert_eq!(graph.nodes[&main].imports.len(), 2);
        assert!(graph.nodes[&main].imports.contains(&lib1));
        assert!(graph.nodes[&main].imports.contains(&lib2));
    }

    // ============================================================
    // TASK-334: resolve_path and is_descendant_or_same tests
    // ============================================================

    #[test]
    fn test_resolve_path_root() {
        let mut graph = ModuleGraph::new();
        let root = graph.add_node(ModuleNode::new(
            "crate".into(),
            ModuleSource::File("main.ash".into()),
        ));
        graph.set_root(root);

        // Resolve "crate" should return root
        let result = graph.resolve_path(&["crate".to_string()]);
        assert_eq!(result, Some(root));
    }

    #[test]
    fn test_resolve_path_nested() {
        let mut graph = ModuleGraph::new();
        let root = graph.add_node(ModuleNode::new(
            "crate".into(),
            ModuleSource::File("main.ash".into()),
        ));
        graph.set_root(root);

        let foo = graph.add_node(ModuleNode::new(
            "foo".into(),
            ModuleSource::File("foo.ash".into()),
        ));
        graph.add_edge(root, foo);

        let bar = graph.add_node(ModuleNode::new(
            "bar".into(),
            ModuleSource::File("foo/bar.ash".into()),
        ));
        graph.add_edge(foo, bar);

        // Resolve "crate::foo"
        let result = graph.resolve_path(&["crate".to_string(), "foo".to_string()]);
        assert_eq!(result, Some(foo));

        // Resolve "crate::foo::bar"
        let result =
            graph.resolve_path(&["crate".to_string(), "foo".to_string(), "bar".to_string()]);
        assert_eq!(result, Some(bar));
    }

    #[test]
    fn test_resolve_path_nonexistent() {
        let mut graph = ModuleGraph::new();
        let root = graph.add_node(ModuleNode::new(
            "crate".into(),
            ModuleSource::File("main.ash".into()),
        ));
        graph.set_root(root);

        // Resolve non-existent path
        let result = graph.resolve_path(&["crate".to_string(), "nonexistent".to_string()]);
        assert_eq!(result, None);
    }

    #[test]
    fn test_is_descendant_or_same_same_module() {
        let mut graph = ModuleGraph::new();
        let root = graph.add_node(ModuleNode::new(
            "crate".into(),
            ModuleSource::File("main.ash".into()),
        ));
        graph.set_root(root);

        // Same module is considered descendant of itself
        assert!(graph.is_descendant_or_same(root, root));
    }

    #[test]
    fn test_is_descendant_or_same_child() {
        let mut graph = ModuleGraph::new();
        let root = graph.add_node(ModuleNode::new(
            "crate".into(),
            ModuleSource::File("main.ash".into()),
        ));
        graph.set_root(root);

        let foo = graph.add_node(ModuleNode::new(
            "foo".into(),
            ModuleSource::File("foo.ash".into()),
        ));
        graph.add_edge(root, foo);

        // foo is a descendant of root
        assert!(graph.is_descendant_or_same(foo, root));
        // root is NOT a descendant of foo
        assert!(!graph.is_descendant_or_same(root, foo));
    }

    #[test]
    fn test_is_descendant_or_same_grandchild() {
        let mut graph = ModuleGraph::new();
        let root = graph.add_node(ModuleNode::new(
            "crate".into(),
            ModuleSource::File("main.ash".into()),
        ));
        graph.set_root(root);

        let foo = graph.add_node(ModuleNode::new(
            "foo".into(),
            ModuleSource::File("foo.ash".into()),
        ));
        graph.add_edge(root, foo);

        let bar = graph.add_node(ModuleNode::new(
            "bar".into(),
            ModuleSource::File("foo/bar.ash".into()),
        ));
        graph.add_edge(foo, bar);

        // bar is a descendant of root (grandchild)
        assert!(graph.is_descendant_or_same(bar, root));
        // bar is a descendant of foo
        assert!(graph.is_descendant_or_same(bar, foo));
        // foo is NOT a descendant of bar
        assert!(!graph.is_descendant_or_same(foo, bar));
        // root is NOT a descendant of bar
        assert!(!graph.is_descendant_or_same(root, bar));
    }

    #[test]
    fn test_is_descendant_or_same_sibling() {
        let mut graph = ModuleGraph::new();
        let root = graph.add_node(ModuleNode::new(
            "crate".into(),
            ModuleSource::File("main.ash".into()),
        ));
        graph.set_root(root);

        let foo = graph.add_node(ModuleNode::new(
            "foo".into(),
            ModuleSource::File("foo.ash".into()),
        ));
        graph.add_edge(root, foo);

        let bar = graph.add_node(ModuleNode::new(
            "bar".into(),
            ModuleSource::File("bar.ash".into()),
        ));
        graph.add_edge(root, bar);

        // Siblings are not descendants of each other
        assert!(!graph.is_descendant_or_same(foo, bar));
        assert!(!graph.is_descendant_or_same(bar, foo));
        // Both are descendants of root
        assert!(graph.is_descendant_or_same(foo, root));
        assert!(graph.is_descendant_or_same(bar, root));
    }

    // ============================================================
    // TASK-338: Crate Identity Tests
    // ============================================================

    #[test]
    fn test_add_crate_record() {
        let mut graph = ModuleGraph::new();
        let root_module = graph.add_node(ModuleNode::new(
            "my_crate".into(),
            ModuleSource::File("src/lib.ash".into()),
        ));

        let crate_id = graph.add_crate(
            "my_crate".to_string(),
            "/path/to/my_crate".to_string(),
            root_module,
        );

        // Verify crate was added
        assert!(graph.crates.contains_key(&crate_id));

        // Verify crate info
        let crate_info = graph.get_crate(crate_id).unwrap();
        assert_eq!(crate_info.name, "my_crate");
        assert_eq!(crate_info.root_path, "/path/to/my_crate");
        assert_eq!(crate_info.root_module, root_module);
        assert!(crate_info.dependencies.is_empty());
    }

    #[test]
    fn test_assign_module_to_crate() {
        let mut graph = ModuleGraph::new();

        // Create a crate
        let root_module = graph.add_node(ModuleNode::new(
            "my_crate".into(),
            ModuleSource::File("src/lib.ash".into()),
        ));
        let crate_id = graph.add_crate(
            "my_crate".to_string(),
            "/path/to/my_crate".to_string(),
            root_module,
        );

        // Create another module and assign it to the crate
        let sub_module = graph.add_node(ModuleNode::new(
            "sub".into(),
            ModuleSource::File("src/sub.ash".into()),
        ));
        graph.assign_module_to_crate(sub_module, crate_id);

        // Verify the module is associated with the crate
        assert_eq!(graph.crate_id_for_module(sub_module), Some(crate_id));
    }

    #[test]
    fn test_lookup_crate_for_module() {
        let mut graph = ModuleGraph::new();

        // Create first crate
        let crate1_root = graph.add_node(ModuleNode::new(
            "crate1".into(),
            ModuleSource::File("crate1/src/lib.ash".into()),
        ));
        let crate1_id = graph.add_crate(
            "crate1".to_string(),
            "/path/to/crate1".to_string(),
            crate1_root,
        );

        // Create second crate
        let crate2_root = graph.add_node(ModuleNode::new(
            "crate2".into(),
            ModuleSource::File("crate2/src/lib.ash".into()),
        ));
        let crate2_id = graph.add_crate(
            "crate2".to_string(),
            "/path/to/crate2".to_string(),
            crate2_root,
        );

        // Create a module in crate1
        let crate1_sub = graph.add_node(ModuleNode::new(
            "sub".into(),
            ModuleSource::File("crate1/src/sub.ash".into()),
        ));
        graph.assign_module_to_crate(crate1_sub, crate1_id);

        // Verify lookups
        assert_eq!(graph.crate_id_for_module(crate1_root), Some(crate1_id));
        assert_eq!(graph.crate_id_for_module(crate1_sub), Some(crate1_id));
        assert_eq!(graph.crate_id_for_module(crate2_root), Some(crate2_id));

        // Verify unassigned module returns None
        let unassigned = graph.add_node(ModuleNode::new(
            "orphan".into(),
            ModuleSource::File("orphan.ash".into()),
        ));
        assert_eq!(graph.crate_id_for_module(unassigned), None);
    }

    #[test]
    fn test_lookup_dependency_target_by_alias() {
        let mut graph = ModuleGraph::new();

        // Create main crate
        let main_root = graph.add_node(ModuleNode::new(
            "main".into(),
            ModuleSource::File("main/src/lib.ash".into()),
        ));
        let main_crate_id =
            graph.add_crate("main".to_string(), "/path/to/main".to_string(), main_root);

        // Create dependency crate
        let dep_root = graph.add_node(ModuleNode::new(
            "dependency".into(),
            ModuleSource::File("dep/src/lib.ash".into()),
        ));
        let dep_crate_id = graph.add_crate(
            "dependency".to_string(),
            "/path/to/dep".to_string(),
            dep_root,
        );

        // Add dependency with alias
        graph.add_dependency(main_crate_id, "dep".to_string(), dep_crate_id);

        // Verify dependency lookup
        assert_eq!(
            graph.dependency_target(main_crate_id, "dep"),
            Some(dep_crate_id)
        );

        // Verify non-existent alias returns None
        assert_eq!(graph.dependency_target(main_crate_id, "nonexistent"), None);

        // Verify non-existent crate returns None
        assert_eq!(graph.dependency_target(CrateId(999), "dep"), None);
    }

    #[test]
    fn test_crate_name_lookup() {
        let mut graph = ModuleGraph::new();

        let root_module = graph.add_node(ModuleNode::new(
            "test_crate".into(),
            ModuleSource::File("src/lib.ash".into()),
        ));
        let crate_id = graph.add_crate(
            "test_crate".to_string(),
            "/path/to/test_crate".to_string(),
            root_module,
        );

        // Verify crate name lookup
        assert_eq!(graph.crate_name(crate_id), Some("test_crate"));

        // Verify non-existent crate returns None
        assert_eq!(graph.crate_name(CrateId(999)), None);
    }

    #[test]
    fn test_add_crate_auto_assigns_root_module() {
        let mut graph = ModuleGraph::new();

        let root_module = graph.add_node(ModuleNode::new(
            "auto_crate".into(),
            ModuleSource::File("src/lib.ash".into()),
        ));
        let crate_id = graph.add_crate(
            "auto_crate".to_string(),
            "/path/to/auto_crate".to_string(),
            root_module,
        );

        // Root module should be automatically associated with the crate
        assert_eq!(graph.crate_id_for_module(root_module), Some(crate_id));
    }

    #[test]
    fn test_multiple_dependencies() {
        let mut graph = ModuleGraph::new();

        // Create main crate
        let main_root = graph.add_node(ModuleNode::new(
            "main".into(),
            ModuleSource::File("main/src/lib.ash".into()),
        ));
        let main_crate_id =
            graph.add_crate("main".to_string(), "/path/to/main".to_string(), main_root);

        // Create multiple dependencies
        let dep1_root = graph.add_node(ModuleNode::new(
            "dep1".into(),
            ModuleSource::File("dep1/src/lib.ash".into()),
        ));
        let dep1_id = graph.add_crate("dep1".to_string(), "/path/to/dep1".to_string(), dep1_root);

        let dep2_root = graph.add_node(ModuleNode::new(
            "dep2".into(),
            ModuleSource::File("dep2/src/lib.ash".into()),
        ));
        let dep2_id = graph.add_crate("dep2".to_string(), "/path/to/dep2".to_string(), dep2_root);

        // Add multiple dependencies with different aliases
        graph.add_dependency(main_crate_id, "first".to_string(), dep1_id);
        graph.add_dependency(main_crate_id, "second".to_string(), dep2_id);

        // Verify all dependencies are stored
        let main_crate = graph.get_crate(main_crate_id).unwrap();
        assert_eq!(main_crate.dependencies.len(), 2);
        assert_eq!(main_crate.dependencies.get("first"), Some(&dep1_id));
        assert_eq!(main_crate.dependencies.get("second"), Some(&dep2_id));

        // Verify lookups work correctly
        assert_eq!(
            graph.dependency_target(main_crate_id, "first"),
            Some(dep1_id)
        );
        assert_eq!(
            graph.dependency_target(main_crate_id, "second"),
            Some(dep2_id)
        );
    }

    #[test]
    fn test_legacy_crate_for_backward_compatibility() {
        let mut graph = ModuleGraph::new();

        let root_module = graph.add_node(ModuleNode::new(
            "legacy_crate".into(),
            ModuleSource::File("src/lib.ash".into()),
        ));
        let crate_id = graph.add_crate(
            "legacy_crate".to_string(),
            "/path/to/legacy_crate".to_string(),
            root_module,
        );

        // Test legacy crate_for method
        assert_eq!(graph.crate_for(root_module), Some(crate_id));

        // Test legacy set_crate method
        let new_module = graph.add_node(ModuleNode::new(
            "new_mod".into(),
            ModuleSource::File("src/new.ash".into()),
        ));
        graph.set_crate(new_module, crate_id);
        assert_eq!(graph.crate_id_for_module(new_module), Some(crate_id));
    }
}
