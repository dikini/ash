# TASK-049: API Documentation

## Status: 🟢 Complete

## Description

Generate and publish comprehensive API documentation for all public crates using rustdoc.

## Specification Reference

- AGENTS.md - Documentation requirements
- rust-skills - Documentation conventions

## Requirements

### Documentation Standards

All public items must have:
- Module-level documentation (`//!`)
- Struct/enum/trait documentation (`///`)
- Function/method documentation (`///`)
- Examples in doc comments
- Panic documentation where relevant
- Error documentation where relevant

### Crate Documentation

**ash-core:**
- Effect system
- Value system
- Workflow AST
- Provenance types
- Pattern matching

**ash-parser:**
- Token types
- Lexer
- Parser
- Surface AST
- Lowering
- Error types

**ash-typeck:**
- Type system
- Type inference
- Constraint solving
- Error types

**ash-interp:**
- Runtime context
- Expression evaluator
- Pattern matcher
- Capability providers
- Policy system

**ash-provenance:**
- Trace recording
- Lineage tracking
- Audit export
- Integrity verification

**ash-cli:**
- Command structure
- CLI arguments
- Exit codes

### Documentation Examples

```rust
//! # Ash Core
//!
//! Core types and data structures for the Ash workflow language.
//!
//! ## Effect System
//!
//! The effect system tracks computational power using a lattice:
//!
//! ```
//! use ash_core::Effect;
//!
//! let e1 = Effect::Epistemic;
//! let e2 = Effect::Operational;
//!
//! // Join takes the maximum effect
//! assert_eq!(e1.join(e2), Effect::Operational);
//! ```

/// Effect levels form a complete lattice.
///
/// Effects track the computational power of operations, from
/// read-only (epistemic) to side-effecting (operational).
///
/// # Ordering
///
/// Effects are ordered from least to most powerful:
/// ```
/// use ash_core::Effect;
///
/// assert!(Effect::Epistemic < Effect::Deliberative);
/// assert!(Effect::Deliberative < Effect::Evaluative);
/// assert!(Effect::Evaluative < Effect::Operational);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Effect {
    /// Read-only operations
    Epistemic,
    /// Analysis and planning
    Deliberative,
    /// Policy evaluation
    Evaluative,
    /// Side-effecting actions
    Operational,
}

impl Effect {
    /// Compute the least upper bound (join) of two effects.
    ///
    /// The join represents the combined effect of sequential
    /// composition.
    ///
    /// # Examples
    ///
    /// ```
    /// use ash_core::Effect;
    ///
    /// let e1 = Effect::Epistemic;
    /// let e2 = Effect::Operational;
    ///
    /// // Sequential composition of epistemic then operational
    /// // results in operational
    /// assert_eq!(e1.join(e2), Effect::Operational);
    /// ```
    pub fn join(self, other: Effect) -> Effect {
        std::cmp::max(self, other)
    }
}
```

### Documentation Build

```bash
# Build documentation
cargo doc --workspace --no-deps

# Open documentation
cargo doc --workspace --no-deps --open

# Check documentation links
cargo doc --workspace --no-deps --document-private-items
```

### README Files

Each crate needs a README with:
- Purpose
- Key types
- Usage example
- Links to full docs

**ash-core README:**

```markdown
# ash-core

Core types for the Ash workflow language.

## Usage

```toml
[dependencies]
ash-core = "0.1.0"
```

## Example

```rust
use ash_core::{Effect, Value, Workflow};

let effect = Effect::Epistemic.join(Effect::Operational);
assert_eq!(effect, Effect::Operational);
```

## Documentation

See the [API documentation](https://docs.rs/ash-core) for details.
```

### Public API Review

Review each crate for:
- Public items that should be private
- Missing documentation
- Missing examples
- Unnecessary public types

## TDD Steps

### Step 1: Document ash-core

Add docs to all public items.

### Step 2: Document ash-parser

Add docs to all public items.

### Step 3: Document ash-typeck

Add docs to all public items.

### Step 4: Document ash-interp

Add docs to all public items.

### Step 5: Document ash-provenance

Add docs to all public items.

### Step 6: Document ash-cli

Add docs to all public items.

### Step 7: Review Public API

Audit for unnecessary public items.

### Step 8: Build and Test Docs

Run cargo doc and verify.

## Completion Checklist

- [ ] ash-core documented
- [ ] ash-parser documented
- [ ] ash-typeck documented
- [ ] ash-interp documented
- [ ] ash-provenance documented
- [ ] ash-cli documented
- [ ] Module-level docs
- [ ] Examples in doc comments
- [ ] README files
- [ ] Public API reviewed
- [ ] cargo doc succeeds
- [ ] No missing docs warnings

## Self-Review Questions

1. **Completeness**: Is everything documented?
2. **Quality**: Are examples helpful?
3. **Accuracy**: Is documentation correct?

## Estimated Effort

6 hours

## Dependencies

- All crates

## Blocked By

- All implementation tasks

## Blocks

- None
