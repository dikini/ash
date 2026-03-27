# TASK-339: Implement Dependency-Aware Multi-Crate Loading

## Status: 🔴 Critical

## Problem

`ModuleResolver` only resolves one crate root and scans lines for `mod foo;`. It does not parse root metadata, load dependency crates, or detect dependency-level cycles and duplicate crate identities.

**Current (Single-Crate Only):**
```rust
pub fn resolve_crate(&self, root_path: impl AsRef<Path>) -> Result<ModuleGraph, ResolveError> {
    let root_id = self.resolve_module(root_path, root_path, &mut graph, &mut visited, &mut resolution_stack)?;
    graph.set_root(root_id);
    Ok(graph)
}
```

**Required (Dependency-Aware):**
```rust
let metadata = parse_crate_root_metadata(&content, canonical_path)?;
let crate_id = graph.add_crate(metadata.crate_name, canonical_path, module_id);
for dependency in metadata.dependencies {
    self.resolve_dependency_crate(crate_id, dependency, graph, visited, resolution_stack)?;
}
```

## Files to Modify

- `crates/ash-parser/src/resolver.rs`
- `crates/ash-parser/src/parse_crate_root.rs`
- `crates/ash-parser/tests/multi_crate_resolver.rs` (new)

## Implementation (TDD)

### Step 1: Write failing resolver tests with `MockFs`

Add coverage for:

```rust
#[test]
fn test_resolve_root_with_one_dependency_crate() { }

#[test]
fn test_reject_duplicate_dependency_alias() { }

#[test]
fn test_reject_duplicate_crate_name() { }

#[test]
fn test_detect_dependency_cycle() { }

#[test]
fn test_missing_dependency_root_file_errors() { }
```

### Step 2: Parse root metadata before dependency traversal

The resolver should:
- read the root file
- parse crate metadata
- register the crate in the graph
- continue resolving in-crate `mod` declarations

### Step 3: Resolve declared dependency crates recursively

Each dependency declaration should:
- resolve a file path relative to the declaring crate root
- parse that dependency crate’s root metadata
- load its modules
- register the alias edge in the graph

### Step 4: Preserve existing module discovery rules

Do not regress:
- `foo.ash`
- `foo/mod.ash`
- module cycle detection within one crate

## Verification

```bash
cargo test --package ash-parser resolver --quiet
cargo test --package ash-parser multi_crate_resolver --quiet
```

## Completion Checklist

- [ ] root metadata parsed during crate resolution
- [ ] dependency crates load recursively
- [ ] duplicate alias and duplicate crate detection added
- [ ] dependency cycle detection added
- [ ] existing single-crate resolver tests stay green
- [ ] CHANGELOG.md update planned for implementation phase

**Estimated Hours:** 3-4
**Priority:** Critical (loader correctness)
**Dependencies:** TASK-337, TASK-338
**Related:** TASK-340
