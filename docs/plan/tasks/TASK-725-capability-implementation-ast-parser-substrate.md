# TASK-725: Add parser and surface-AST substrate for `capability impl` declarations.

## Status: ✅ Complete

## Task Type

Parser/AST

## Description

Add parser and surface-AST substrate for `capability impl` declarations.

## Specification Reference

- SPEC-052

## Dependencies

- ✅ TASK-724: prerequisite task

## Requirements

### Functional Requirements

1. Add a surface AST carrier for capability implementation recipes.
2. Parse target interface name, implementation name, dependency clauses, and operation bodies.
3. Represent dependencies on resources, capabilities, and configuration values without resolving them yet.
4. Preserve operation body ASTs for later type checking/execution.

### Property Requirements (proptest)

```rust
// Add property-based tests for parser round-trips, conformance invariants,
// authority non-widening, resource identity preservation, or split/join
// behavior where this task introduces executable semantics.
// Docs-only tasks must instead include a corpus consistency sweep.
```

## TDD Steps

### Step 1: Write Tests or Corpus Checks (Red)

For implementation tasks, add failing tests before code changes. For docs/planning tasks, add or run corpus checks that fail or report missing references before updating docs.

### Step 2: Implement or Write Docs (Green)

Make the minimal focused change required by this task while preserving the Ash semantic tower:

```text
Pure < Effectful / Act < Proc < Workflow
```

### Step 3: Integration (Green)

Wire only the layer owned by this task. Do not silently implement downstream runtime behavior from later tasks.

### Step 4: Verification

Required verification for this task class:

- Parser/type/runtime tasks: focused crate tests plus affected integration tests.
- Docs/planning tasks: `git diff --check` plus cross-reference sweep for changed docs.
- All code tasks: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --workspace` before completion.

## Verification Steps

- [x] Requirements above are satisfied.
- [x] New tests or docs checks cover the task-owned behavior.
- [x] Existing public behavior remains compatible unless the spec explicitly says otherwise.
- [x] CHANGELOG.md is updated for implementation/tooling/docs-policy changes.
- [x] PLAN-INDEX.md status is updated only when the task is actually complete.

## Completion Evidence

- Capability implementation AST/parser substrate is implemented as Phase 101 parser/module substrate only.
- Added surface AST carriers for Ash-defined capability implementations, required resource clauses, and operation implementation methods.
- Parser accepts `capability impl Name for Interface ... { ... }` and preserves method modes, signatures, target interface, visibility, and spans.
- Malformed implementation headers and duplicate operation implementations are covered by Phase 101 parser tests.
- Focused verification: `cargo test -p ash-parser --test phase_101_capability_implementation_parser`.
- Final closeout verification also runs `cargo test --workspace --all-targets --all-features`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo fmt --check`, `cargo doc --workspace --no-deps`, and `git diff --check`.

## Dependencies for Next Task

This task outputs:

- Parsed implementation recipes survive into module metadata.

## Notes

- Preserve existing `pub capability` and Rust `CapabilityProvider` compatibility unless the task explicitly owns a migration.
- Do not allow ambient capability/resource lookup to bypass explicit admission.
- Do not manufacture external authority from Ash-defined code.
- Keep resource handles environment-owned unless a later spec explicitly introduces first-class `ResourceRef<T>` values.
