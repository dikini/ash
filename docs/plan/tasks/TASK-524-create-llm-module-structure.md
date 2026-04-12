# TASK-524: Create std/src/llm/ module structure

## Status: Draft

## Description

Create the module root for the LLM stdlib with proper module declarations and re-exports. This is the top-level `llm` module that contains shared vocabulary (types and prompt functions) usable by any LLM provider.

## Specification Reference

- [DESIGN-025: LLM Standard Library](../../design/DESIGN-025-LLM-STDLIB.md) (D1: Protocol Module Architecture)
- [SPEC-029: LLM Standard Library](../../spec/SPEC-029-LLM-STDLIB.md) (SS2: Module Structure)
- [PLAN-025: LLM Standard Library](../PLAN-025-LLM-STDLIB.md)

## Dependencies

- Phase 75 (Pure Functions) must support `fn` parsing in .ash files.

## Requirements

1. Create `std/src/llm/mod.ash` with module declarations for `types` and `prompt` submodules.
2. Re-export all public types and functions so `use llm::{Message, user}` is valid.
3. The module must parse without errors via the engine.

## Guidance

Follow the pattern of `std/src/io/mod.ash` for module root structure. This module is provider-agnostic -- nothing OpenAI-specific lives here.

## Likely Files

- Create: `std/src/llm/mod.ash`

## TDD Steps

### Red

1. Write test: parsing `std/src/llm/mod.ash` succeeds via engine.

### Green

Create the module file with correct declarations.

## Completion Checklist

- [ ] `std/src/llm/mod.ash` created with module declarations
- [ ] Re-exports for types and prompt submodules
- [ ] File parses without errors
