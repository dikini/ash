# TASK-540: Transitive `pub mod` loading

## Status: Draft

## Description

Extend `collect_module_exports` to process `pub mod <name>;` declarations by resolving the submodule path, recursively loading its exports, and merging them into the parent module.

## Spec Reference

- [SPEC-030: Module Type Resolution](../../spec/SPEC-030-MODULE-TYPE-RESOLUTION.md) §4
- [DESIGN-026](../../design/DESIGN-026-MODULE-TYPE-RESOLUTION-REMEDIATION.md) D3

## Dependencies

- [TASK-539](TASK-539-two-pass-type-collection.md)

## Requirements

1. `pub mod <name>;` triggers recursive loading of the submodule.
2. Only `pub` items from the submodule are merged into parent exports.
3. Circular `pub mod` references are detected and reported as errors.
4. `use llm::Role` resolves through `llm/mod.ash` → `llm/types.ash`.

## Completion Checklist

- [ ] `pub mod` processing implemented
- [ ] Cycle detection works
- [ ] Visibility filtering (pub only) works
- [ ] `use llm::Role` resolves

