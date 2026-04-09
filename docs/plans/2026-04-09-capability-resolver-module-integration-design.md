# Capability Resolver Module Integration Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the Phase 70 bridge resolver with module-system-owned capability resolution.

**Architecture:** Capability declarations, exports, imports, and re-exports become the source of
truth for symbolic operational call targets. One capability-resolution context is built from the
module/import pipeline and passed into lowering and compile-time checking, while explicit
`provider:action(...)` remains direct.

**Tech Stack:** Rust 2024, `ash-parser`, `ash-typeck`, module graph/import resolver pipeline,
active specs in `docs/spec/`.

---

## Recommended Approach

1. Freeze the normative contract first in `docs/spec/`.
2. Extend module/export/import metadata so capability declarations carry canonical
   `(provider, action)` targets.
3. Build one capability-resolution context from that metadata.
4. Migrate lowering and type checking away from local built-in resolvers.
5. Bootstrap std capability symbols through the same authoritative path.

## Key Risks

- Leaving symbolic resolution duplicated between lowering and type checking.
- Treating module-qualified symbolic names as provider syntax instead of module-path resolution.
- Removing the bridge in docs before the implementation actually stops using it.

## Canonical Outputs

- [DESIGN-017](../design/DESIGN-017-MODULE-OWNED-CAPABILITY-RESOLUTION.md)
- [PLAN-017](../plan/PLAN-017-MODULE-OWNED-CAPABILITY-RESOLUTION.md)
- [TASK-471](../plan/tasks/TASK-471-spec-module-owned-capability-resolution.md) through
  [TASK-479](../plan/tasks/TASK-479-module-owned-capability-resolution-verification.md)
