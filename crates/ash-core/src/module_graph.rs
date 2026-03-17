//! Module graph for tracking module dependencies and hierarchy
//!
//! The module graph tracks:
//! - Module hierarchy (parent-child relationships)
//! - Import dependencies between modules
//! - Source origins (file or inline)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a module in the graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModuleId(pub usize);

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
}

impl ModuleNode {
    /// Create a new module node with the given name and source
    pub fn new(name: String, source: ModuleSource) -> Self {
        Self {
            name,
            source,
            children: Vec::new(),
            imports: Vec::new(),
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
        }
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
    pub fn add_edge(&mut self, parent: ModuleId, child: ModuleId) {
        if let Some(parent_node) = self.nodes.get_mut(&parent) {
            parent_node.children.push(child);
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
}
