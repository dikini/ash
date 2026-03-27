# TASK-340: Resolve External Imports and Enforce Cross-Crate Visibility

## Status: 🔴 Critical

## Problem

`ImportResolver` only resolves `crate::...` paths and still contains visibility logic designed for a single-root graph. After Phase 55 syntax and loader work, imports from declared dependencies must resolve through explicit external aliases and enforce real crate boundaries.

**Current (Non-Compliant):**
```rust
// resolver only accepts crate-rooted paths
use crate::foo::bar;
```

**Required (Cross-Crate Aware):**
```ash
use external::util::sanitize::normalize;
```

And cross-crate visibility must behave as:

```rust
if importing_crate != target_crate {
    matches!(visibility, Visibility::Public)
}
```

## Files to Modify

- `crates/ash-parser/src/import_resolver.rs`
- `crates/ash-parser/src/parse_use.rs`

## Implementation (TDD)

### Step 1: Write failing resolver tests

Add coverage for:

```rust
#[test]
fn test_external_import_public_item_allowed() { }

#[test]
fn test_external_import_pub_crate_rejected() { }

#[test]
fn test_external_import_pub_super_rejected() { }

#[test]
fn test_external_import_undeclared_alias_rejected() { }

#[test]
fn test_external_glob_import_skips_non_public_items() { }
```

### Step 2: Add explicit external-path handling

Expected dispatch:

```rust
match first_segment {
    "crate" => self.resolve_current_crate_path(importing_module, rest)?,
    "external" => self.resolve_external_path(importing_module, rest)?,
    _ => return Err(ImportError::InvalidPrefix { prefix: first_segment.into() }),
}
```

### Step 3: Enforce cross-crate visibility in one place

Expected rule:

```rust
if self.module_graph.crate_id_for_module(importing_module)
    != self.module_graph.crate_id_for_module(target_module)
{
    return matches!(visibility, Visibility::Public);
}
```

### Step 4: Add parser tests for external import syntax

Document accepted forms:

```rust
use external::util::item;
use external::util::{a, b as c};
```

## Verification

```bash
cargo test --package ash-parser import_resolver --quiet
cargo test --package ash-parser parse_use --quiet
```

## Completion Checklist

- [ ] `external::<alias>::...` imports resolve through declared dependencies
- [ ] undeclared external aliases fail with explicit error
- [ ] only `pub` crosses crate boundaries
- [ ] parser tests document external import syntax
- [ ] existing `crate::...` imports still pass
- [ ] clippy clean for touched parser/resolver code

**Estimated Hours:** 2-3
**Priority:** Critical (security boundary)
**Dependencies:** TASK-338, TASK-339
**Related:** TASK-341
