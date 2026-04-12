# TASK-541: `ash check` module-file support

## Status: Draft (v2)

## Description

Add a module-file check path in `ash check` following the SPEC-009 §4.1a `ModuleFile` model. Non-workflow files are validated for type/fn/use parse correctness with sibling type cross-reference resolution via pre-declaration.

## Spec Reference

- [SPEC-030](../../spec/SPEC-030-MODULE-TYPE-RESOLUTION.md) §5
- [DESIGN-026](../../design/DESIGN-026-MODULE-TYPE-RESOLUTION-REMEDIATION.md) D3

## Dependencies

- [TASK-539](TASK-539-two-pass-type-collection.md)

## Requirements

1. `ash check` follows SPEC-009 ModuleFile model (not fallback).
2. Non-workflow files validated for type/fn/use correctness.
3. Sibling type cross-references resolve via pre-declaration.
4. Output format per SPEC-030 §5.2.

## Completion Checklist

- [ ] Module-file validation implemented
- [ ] `ash check std/src/llm/types.ash` succeeds
- [ ] Output shows type/function counts
- [ ] Invalid types report specific errors

