# TASK-540: Load child modules on `pub mod`

## Status: Draft (v2)

## Description

Extend `collect_module_exports` to process `pub mod <name>;` lines by resolving the child module path, recursively loading its exports, and storing them under the child module name. Does NOT merge into parent exports (baseline SPEC-009/SPEC-012 semantics preserved).

## Spec Reference

- [SPEC-030](../../spec/SPEC-030-MODULE-TYPE-RESOLUTION.md) §4
- [DESIGN-026](../../design/DESIGN-026-MODULE-TYPE-RESOLUTION-REMEDIATION.md) D2

## Dependencies

- [TASK-539](TASK-539-two-pass-type-collection.md)

## Requirements

1. `pub mod <name>;` resolves and loads child module exports.
2. Child exports available for qualified access (`llm::types::Role`).
3. Parent-level access (`llm::Role`) still requires explicit `pub use`.
4. Unknown module path reports error.

## Completion Checklist

- [ ] `pub mod` processing implemented
- [ ] Qualified path resolution works
- [ ] Explicit re-export still required for parent-level access
- [ ] Unknown module error reported

