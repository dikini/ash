# TASK-2058: Canonical Module Identity and Artifacts

**Status:** Planned
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§2, 5, 7-8
**Owned rule:** MOD-REAL-001
**Run-route impact:** prerequisite

## Description

Extend the module graph from topology-only records to canonical, crate-qualified module identities and durable module-unit artifacts that later stages can share without parser-private or Engine-private rediscovery.

## Dependencies

- ✅ TASK-2056 — target contract and seam audit.
- 📝 TASK-2057 — AST-derived structural declarations; may be integrated after its public handoff is stable.

## Current → target

**Current files:** `crates/ash-core/src/module_graph.rs`, `crates/ash-parser/src/resolver.rs`, semantic-summary modules in `crates/ash-core`.

**Current state:** graph nodes record names, parent/child/import edges, crates, and file/inline source tags. They do not provide the complete stable identity and module-unit contract SPEC-103 requires.

**Target state:** `ash-core` owns canonical module path/key, source origin, structural/import dependency facts, parsed/expanded/check-state references, interface schema identity, and diagnostic anchors. The exact Rust shapes may vary, but no downstream semantic interface uses a bare name or filesystem path as identity.

## Requirements

1. Define canonical equality, display, serialization, and cache-key behavior for crate-qualified paths.
2. Preserve defining identity through aliases and re-exports without minting a second identity.
3. Represent file and inline source origins without requiring inline text reconstruction.
4. Make structural-parent and child-key queries deterministic.
5. Reject duplicate canonical child identities before interface publication.

## TDD Steps and evidence

1. Add unit tests for canonical nested paths, `mod.ash` directory identity, file identity, inline identity, crate distinction, and parent/child round trips.
2. Add proptest coverage for path segment construction and structural-tree invariants.
3. Add serde/cache-key round trips where the carrier crosses summary/cache boundaries.
4. Confirm existing graph consumers compile only after identity migration is explicit.

## Completion checklist

- [ ] Canonical module identity is crate-qualified and independent of aliases/filesystem spelling.
- [ ] File and inline origins are represented without rebuilding inline text.
- [ ] Duplicate child identity and topology tests pass.
- [ ] Focused core/parser tests, fmt, and clippy pass.

## Handoffs

- **Consumes:** AST discovery facts from TASK-2057 and existing `ModuleGraph` topology.
- **Produces:** core-owned `ModuleKey`/module-unit artifact contract for TASK-2059 through TASK-2063.
- **Downstream owner:** TASK-2059 owns source acquisition; TASK-2060 owns checked interfaces.
- **Non-goals:** import binding, public export selection, Core/CPS lowering, persistent disk cache, or runtime module values.

## Files and verification

**Files:** `crates/ash-core/src/module_graph.rs`, core semantic-summary modules, `crates/ash-parser/src/resolver.rs`, focused core/parser tests.

```text
cargo test -p ash-core module_graph
cargo test -p ash-parser resolver
cargo clippy -p ash-core -p ash-parser --all-targets -- -D warnings
cargo fmt --check
```
