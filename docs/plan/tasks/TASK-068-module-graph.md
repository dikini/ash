# TASK-068: Module Graph Data Structure

## Status: 🟢 Complete

## Description

Implement the module graph data structure for tracking module dependencies.

## Specification Reference

- SPEC-009: Module System - Section 6 Module Graph

## Requirements

### Functional Requirements

1. `ModuleId` type for unique module identifiers
2. `ModuleNode` struct with:
   - name, source, children, imports
3. `ModuleGraph` struct with:
   - nodes: HashMap<ModuleId, ModuleNode>
   - root: Option<ModuleId>

### Property Requirements

```rust
// Module IDs are unique
graph.add_node(node1) != graph.add_node(node2)

// Parent-child relationships
graph.add_edge(parent, child);
graph.get_node(parent).children.contains(&child)

// Root is set once
graph.set_root(root);
graph.root() == Some(root)
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_module_graph_add_node() {
    let mut graph = ModuleGraph::new();
    let id = graph.add_node(ModuleNode::new("foo".into(), ModuleSource::File("foo.ash".into())));
    assert!(graph.get_node(id).is_some());
}

#[test]
fn test_module_graph_edge() {
    let mut graph = ModuleGraph::new();
    let parent = graph.add_node(ModuleNode::new("p".into(), ModuleSource::File("p.ash".into())));
    let child = graph.add_node(ModuleNode::new("c".into(), ModuleSource::File("c.ash".into())));
    
    graph.add_edge(parent, child);
    
    assert!(graph.get_node(parent).unwrap().children.contains(&child));
}

#[test]
fn test_module_graph_root() {
    let mut graph = ModuleGraph::new();
    let root = graph.add_node(ModuleNode::new("main".into(), ModuleSource::File("main.ash".into())));
    
    graph.set_root(root);
    assert_eq!(graph.root(), Some(root));
}
```

### Step 2: Implement (Green)

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(usize);

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleSource {
    File(Box<str>),
    Inline { parent: ModuleId, offset: usize },
}

#[derive(Debug, Clone)]
pub struct ModuleNode {
    pub name: Box<str>,
    pub source: ModuleSource,
    pub children: Vec<ModuleId>,
    pub imports: Vec<ModuleId>,
}

impl ModuleNode {
    pub fn new(name: Box<str>, source: ModuleSource) -> Self {
        Self { name, source, children: Vec::new(), imports: Vec::new() }
    }
}

#[derive(Debug, Default)]
pub struct ModuleGraph {
    nodes: HashMap<ModuleId, ModuleNode>,
    root: Option<ModuleId>,
    next_id: usize,
}

impl ModuleGraph {
    pub fn new() -> Self { Self::default() }
    
    pub fn add_node(&mut self, node: ModuleNode) -> ModuleId {
        let id = ModuleId(self.next_id);
        self.next_id += 1;
        self.nodes.insert(id, node);
        id
    }
    
    pub fn get_node(&self, id: ModuleId) -> Option<&ModuleNode> {
        self.nodes.get(&id)
    }
    
    pub fn add_edge(&mut self, parent: ModuleId, child: ModuleId) {
        if let Some(node) = self.nodes.get_mut(&parent) {
            node.children.push(child);
        }
    }
    
    pub fn set_root(&mut self, root: ModuleId) { self.root = Some(root); }
    pub fn root(&self) -> Option<ModuleId> { self.root }
}
```

## Completion Checklist

- [ ] ModuleId type defined
- [ ] ModuleNode struct defined
- [ ] ModuleGraph with add/get/add_edge
- [ ] Root tracking
- [ ] Tests pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

4 hours

## Dependencies

None

## Blocked By

Nothing

## Blocks

- TASK-069 (Module resolver)
