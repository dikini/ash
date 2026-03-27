# TASK-338: Extend Module Graph with Crate Identity

## Status: 🔴 Critical

## Problem

`ModuleGraph` currently models one root module and no crate ownership. It cannot answer which crate owns a module or whether an external alias is a declared dependency target.

**Current (Non-Compliant for Multi-Crate):**
```rust
pub struct ModuleGraph {
    pub nodes: HashMap<ModuleId, ModuleNode>,
    pub root: Option<ModuleId>,
    next_id: usize,
}
```

**Required (Crate-Aware Model):**
```rust
pub struct CrateId(pub usize);

pub struct CrateInfo {
    pub name: String,
    pub root_module: ModuleId,
    pub root_path: String,
    pub dependencies: HashMap<String, CrateId>,
}
```

The graph must also be able to map each module back to its owning crate.

## Files to Modify

- `crates/ash-core/src/module_graph.rs`

## Implementation (TDD)

### Step 1: Write failing graph tests

Add coverage for:

```rust
#[test]
fn test_add_crate_record() { }

#[test]
fn test_assign_module_to_crate() { }

#[test]
fn test_lookup_crate_for_module() { }

#[test]
fn test_lookup_dependency_target_by_alias() { }
```

### Step 2: Add crate-aware types and metadata

Implement:

```rust
pub struct CrateId(pub usize);

pub struct CrateInfo {
    pub name: String,
    pub root_module: ModuleId,
    pub root_path: String,
    pub dependencies: HashMap<String, CrateId>,
}
```

Add crate ownership either:
- directly on `ModuleNode`, or
- in an equivalent graph-owned index

### Step 3: Add minimal query helpers

Required helpers:

```rust
pub fn crate_id_for_module(&self, module: ModuleId) -> Option<CrateId>;
pub fn crate_name(&self, crate_id: CrateId) -> Option<&str>;
pub fn dependency_target(&self, crate_id: CrateId, alias: &str) -> Option<CrateId>;
```

## Verification

```bash
cargo test --package ash-core module_graph --quiet
cargo clippy --package ash-core -- -D warnings
```

## Completion Checklist

- [ ] `CrateId` introduced
- [ ] crate metadata stored in `ModuleGraph`
- [ ] module-to-crate ownership lookup implemented
- [ ] dependency alias lookup implemented
- [ ] single-crate graph tests still pass
- [ ] clippy clean

**Estimated Hours:** 2
**Priority:** Critical (foundational model)
**Dependencies:** TASK-337
**Related:** TASK-339, TASK-340
