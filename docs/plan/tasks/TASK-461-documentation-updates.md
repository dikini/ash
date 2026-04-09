# TASK-461: Update Unified Action System Documentation

## Status: Planned

## Description

Update active docs, examples, and API references so they describe the unified provider trait,
evaluated `Action` arguments, and the new interpreter/provider boundary accurately.

## Specification Reference

- [DESIGN-015: Unified Action System](../../design/DESIGN-015-UNIFIED-ACTION-SYSTEM.md)
- [SPEC-010: Embedding](../../spec/SPEC-010-EMBEDDING.md)
- [SPEC-017: Capability Integration](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)

## Dependencies

- ✅ [TASK-459](TASK-459-remove-old-provider-trait.md)
- ✅ [TASK-460](TASK-460-error-handling-unified.md)

## Requirements

1. Update design/spec/API references that still document split provider traits or string-plus-args
   provider dispatch.
2. Refresh active examples for custom provider implementations.
3. Keep historical documents intact unless they are active normative/API references.
4. Update plan references and closeout notes where needed.

## TDD Steps

### Red

- Active docs/examples still show the old provider trait shape or imply deferred action evaluation.

### Green

- Active documentation consistently describes unified `Action` and provider APIs.
- Examples compile or remain obviously truthful to current APIs.

## Completion Checklist

- [ ] active docs/spec/API references updated
- [ ] example snippets refreshed for unified provider trait
- [ ] no active doc still teaches the split provider interface
- [ ] `CHANGELOG.md` updated

## Implementation Notes

- Prefer touching active docs only; leave historical phase records as history unless they are now misleading as current guidance.
