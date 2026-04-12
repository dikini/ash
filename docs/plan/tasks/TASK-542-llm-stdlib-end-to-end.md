# TASK-542: Validate LLM stdlib end-to-end

## Status: Draft

## Description

End-to-end verification that all LLM stdlib module files parse, resolve, and check correctly through the full `ash check` and module-loader paths.

## Spec Reference

- [SPEC-030: Module Type Resolution](../../spec/SPEC-030-MODULE-TYPE-RESOLUTION.md) §3.4, §4.4, §5.4

## Dependencies

- [TASK-540](TASK-540-transitive-pub-mod-loading.md)
- [TASK-541](TASK-541-ash-check-module-files.md)

## Requirements

1. `ash check std/src/llm/types.ash` succeeds.
2. `ash check std/src/llm/mod.ash` succeeds.
3. `use llm::types::Role` from a workflow resolves.
4. `use llm::Role` via mod.ash re-export resolves.
5. Existing stdlib tests updated to use module-loader API.

## Completion Checklist

- [ ] All stdlib files pass `ash check`
- [ ] Import paths resolve correctly
- [ ] Tests updated from string-matching to structural checks

