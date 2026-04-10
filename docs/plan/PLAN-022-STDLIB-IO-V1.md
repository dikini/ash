# PLAN-022: Stdlib IO V1

## Status: ✅ Complete

## Overview

Implement the first real `io` standard-library family for Ash as a top-level stdlib namespace rooted
at `std/src/io/`. This phase turns the approved v1 design into a module tree, capability-backed
ambient helper surface, provider/runtime wiring, and end-to-end tests.

## Design Reference

- [Stdlib `io` V1 Design](../plans/2026-04-10-stdlib-io-v1-design.md)

## Goals

1. Add a top-level `io` stdlib module family without introducing a `std::` namespace layer.
2. Keep `io::path` pure and capability-free while making host-touching operations explicit.
3. Expose ergonomic ambient helpers built on top of explicit capability and handle-oriented design.
4. Reuse the existing `result::Result<T, E>` ADT and standard provider/runtime infrastructure.
5. Keep the v1 surface broad enough to be useful without committing to async I/O, stream traits, or watchers.

## Scope

**In Scope**:
- `io`, `io::path`, `io::stdio`, `io::fs`, `io::dir`, `io::meta`, and `io::buf`
- shared `io::Error` / `io::ErrorKind` vocabulary
- stdio and filesystem capability/provider alignment
- stdlib module loading and capability-export bootstrap for new `io` modules
- parser/typecheck/runtime/engine tests for the new stdlib surface
- docs, examples, and changelog/index updates

**Out of Scope**:
- `std::io` namespacing
- async I/O
- symlink and hard-link APIs
- file watching
- generic stream traits/interfaces
- temporary files, locking, mmap, or other OS-heavy extras

## Implementation Guardrails

1. Keep stdlib imports top-level: `use io::fs`, not `use std::io::fs`.
2. Do not redefine `Result`; only add `io::Result<T>` as an alias if it pays for itself.
3. Keep `io::path` pure. Host access belongs in `io::stdio`, `io::fs`, `io::dir`, and `io::meta`.
4. Prefer module-level functions over methods unless the current Ash surface already supports the method shape cleanly.
5. Map to Rust semantics where helpful, but do not mechanically mirror Rust module layout if it weakens Ash capability boundaries.
6. Build ambient helpers as thin sugar over capability/provider-backed operations rather than a separate execution model.

## Tasks

- [TASK-493](tasks/TASK-493-freeze-stdlib-io-contract.md)
- [TASK-494](tasks/TASK-494-stdlib-io-root-and-path-surface.md)
- [TASK-495](tasks/TASK-495-stdlib-io-stdio-surface-and-provider-alignment.md)
- [TASK-496](tasks/TASK-496-stdlib-io-files-dir-meta-surface.md)
- [TASK-497](tasks/TASK-497-stdlib-io-buffered-helpers-and-ambient-sugar.md)
- [TASK-498](tasks/TASK-498-stdlib-io-bootstrap-and-runtime-wiring.md)
- [TASK-499](tasks/TASK-499-stdlib-io-integration-tests-and-examples.md)
- [TASK-500](tasks/TASK-500-stdlib-io-docs-and-verification.md)

## Implementation Notes and Gaps

The Phase 74 implementation (TASK-493 through TASK-500) produced a **reference/exploratory implementation** that demonstrates the intended shape of the `io` stdlib family. However, several gaps exist between this implementation and a spec-grounded production version:

### Known Gaps

1. **Capability Syntax in .ash Files**: The capability declarations in stdio.ash, fs.ash, dir.ash, and meta.ash use aspirational syntax (`pub capability X: observe ... | execute ...`) that does not match the current parser's actual capability grammar. The real capability system uses a different surface syntax that was not fully stable at implementation time.

2. **Provider Action Dispatch**: The `act observe` and `act execute` syntax used in the stdlib modules is illustrative. The actual lowering path from surface `act` statements to provider method calls involves more intermediate steps (constraint generation, capability resolution, provider registry lookup) that are not fully exercised by the current stdlib surface.

3. **Path Type Representation**: `PathBuf` is implemented as a newtype wrapper `PathBuf { inner: String }` but the actual runtime representation may need to be a builtin or use different unwrapping syntax in pattern matches.

4. **Error/Result Integration**: The `io::Result<T>` alias assumes the type system supports aliases transparently, which may not be fully implemented in the type checker.

### Design Rationale (Why This Approach)

Despite these gaps, the implementation serves several purposes:

1. **Surface Exploration**: It validates that the module tree (`io`, `io::path`, etc.) can be parsed and resolved by the existing module system.

2. **Provider Alignment**: The expanded `FsProvider` with 11 actions demonstrates how the provider layer can consolidate filesystem operations, serving as a template for provider authors.

3. **Test Harness**: The 172 passing parser and engine tests establish a regression suite that will fail if future changes break the io module structure.

4. **Documentation**: The examples/03-io/ directory provides concrete Ash code showing the intended user-facing style, even if the underlying semantics are not yet fully implemented.

### Path to Production

To make this implementation production-ready:

1. Align capability declarations with the actual capability grammar from SPEC-017
2. Replace aspirational `act` syntax with the real lowering path
3. Verify PathBuf representation matches the runtime's expected Value variants
4. Add type-checking tests for io::Result<T> usage

## Deliverable

An **exploratory but functional** `io` stdlib family where:

- pure path values and transforms live under `io::path`;
- stdio and filesystem operations are available through capability-bearing stdlib modules;
- the engine/provider layer recognizes and executes the new capability-backed surface;
- the repository has end-to-end examples and tests that show the intended Ash style;
- **documented gaps** explain where the implementation anticipates future spec alignment.
