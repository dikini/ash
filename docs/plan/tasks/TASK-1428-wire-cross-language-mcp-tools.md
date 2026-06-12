# TASK-1428: Wire Cross-Language MCP Tools

## Status: 📝 Planned

## Description

Wire `ash_find_rust_implementation` and `ash_find_ash_usage` into the compiled `ash-mcp` crate and MCP tool registry. Phase 142 left implementations in an unreferenced source file; this task makes the public server surface real and tests that it remains registered.

## Specification Reference

- PLAN-142 deliverables: `ash_find_rust_implementation`, `ash_find_ash_usage`
- PLAN-143 remediation scope
- `crates/ash-mcp/src/lib.rs`
- `crates/ash-mcp/src/lib_symbol_mapping.rs`

## Dependencies

- 📝 TASK-1427: Status and artifact hygiene

## Requirements

### Functional Requirements

1. Ensure `lib_symbol_mapping.rs` is compiled by `ash-mcp` or move its contents into a compiled module.
2. Ensure MCP macro/tool-handler registration includes both cross-language tools.
3. Update `ash_mcp_health` to list both tools.
4. Add tests that fail if either tool name is absent from health output or if the wrapper functions are not callable.
5. Preserve existing `ash_hover_with_rust_context` behavior.

### Property Requirements

No proptest requirement; registration is deterministic. Add table/fixture tests for all expected tool names.

## TDD Steps

### Step 1: RED registration tests

**File:** `crates/ash-mcp/src/tests.rs` or `crates/ash-mcp/tests/cross_language_tools.rs`

Add tests asserting:
- health tool includes `ash_find_rust_implementation`,
- health tool includes `ash_find_ash_usage`,
- the compiled crate exposes callable test wrappers for both.

### Step 2: Wire module and tools

**Files:**
- `crates/ash-mcp/src/lib.rs`
- `crates/ash-mcp/src/lib_symbol_mapping.rs`

Add `mod lib_symbol_mapping;` or refactor into a named module. Resolve imports, visibility, and duplicate helper names. Add test-only public wrappers as needed.

### Step 3: Verify no dead split modules

Search for unreferenced Phase 142 source files and remove/rename any stale scaffolding.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-mcp health_tool_contains_cross_language_tools -- --nocapture
  - cargo test -p ash-mcp cross_language -- --nocapture
  - cargo clippy -p ash-mcp --all-targets -- -D warnings
  - cargo fmt --check
checklist:
  - [ ] Both cross-language tools are compiled and registered
  - [ ] Health output lists both tools
  - [ ] Tests fail if tools are removed from registry
  - [ ] No unreferenced implementation file remains for claimed tool surface
```

## Dependencies for Next Task

This task unblocks TASK-1430 positive tool fixtures and TASK-1431 corpus evaluation.
