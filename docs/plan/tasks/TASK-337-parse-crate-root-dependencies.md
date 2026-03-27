# TASK-337: Add Crate Root and Dependency Syntax

## Status: 🔴 Critical

## Problem

Ash has no user-facing source syntax for crate identity or dependency crate roots. The current module system only supports a single crate discovered from one entry file, so real cross-crate boundaries cannot be expressed or validated.

**Current (Missing Contract):**
```ash
mod workflows;
use crate::workflows::main;
```

**Required (Phase 55 Contract):**
```ash
crate app;

dependency util from "../util/main.ash";
dependency policy from "../policy/main.ash";

use external::util::sanitize::normalize;
```

## Files to Modify

- `docs/spec/SPEC-009-MODULES.md`
- `docs/spec/SPEC-012-IMPORTS.md`
- `crates/ash-parser/src/surface.rs`
- `crates/ash-parser/src/lib.rs`
- `crates/ash-parser/src/parse_crate_root.rs` (new)
- `crates/ash-parser/tests/crate_root_parser.rs` (new)

## Implementation (TDD)

### Step 1: Write failing parser tests

Add coverage for:

```rust
#[test]
fn test_parse_crate_root_name() { }

#[test]
fn test_parse_crate_root_with_dependencies() { }

#[test]
fn test_parse_dependency_requires_quoted_path() { }

#[test]
fn test_parse_dependency_without_crate_decl_rejected() { }
```

### Step 2: Add explicit AST types

```rust
pub struct CrateRootMetadata {
    pub crate_name: Box<str>,
    pub dependencies: Vec<DependencyDecl>,
    pub span: Span,
}

pub struct DependencyDecl {
    pub alias: Box<str>,
    pub root_path: Box<str>,
    pub span: Span,
}
```

### Step 3: Implement the parser

Expected syntax:

```rust
crate app;
dependency util from "../util/main.ash";
```

The parser should:
- require exactly one crate declaration in a root metadata block
- allow zero or more dependency declarations after the crate declaration
- preserve spans for diagnostics

### Step 4: Update specs and examples

Document:
- root-only `crate` declaration
- `dependency <alias> from "<path>";`
- `use external::<alias>::...`

## Verification

```bash
cargo test --package ash-parser crate_root_parser --quiet
cargo test --package ash-parser --quiet
```

## Completion Checklist

- [ ] Root metadata syntax defined in SPEC-009 and SPEC-012
- [ ] AST types added for crate metadata
- [ ] Parser accepts valid crate/dependency declarations
- [ ] Parser rejects malformed crate/dependency declarations
- [ ] `external::<alias>::...` examples are documented
- [ ] CHANGELOG.md update planned for implementation phase

**Estimated Hours:** 2-3
**Priority:** Critical (public contract)
**Dependencies:** None
**Related:** Phase 55 design, Phase 54 follow-up limitation
