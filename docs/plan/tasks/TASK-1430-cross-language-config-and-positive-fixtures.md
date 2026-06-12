# TASK-1430: Cross-Language Config and Positive Fixtures

## Status: ✅ Complete

## Description

Add committed cross-language configuration and positive fixture coverage so `ash_find_rust_implementation`, `ash_find_ash_usage`, and `ash_hover_with_rust_context` can be exercised without relying on untracked local files.

## Specification Reference

- TASK-1420: Cross-language configuration schema
- TASK-1421/TASK-1422/TASK-1424: tool behavior
- PLAN-143 remediation scope

## Dependencies

- 📝 TASK-1428: Tool wiring
- 📝 TASK-1429: Real syn parser

## Requirements

### Functional Requirements

1. Commit a project-local config at a path the server actually loads, preferably `.ash/cross_lang_config.yaml` or update loader/config API with a test-only explicit path.
2. Include at least three high-confidence mappings covering different Rust item kinds.
3. Add positive tests for Ash → Rust lookup returning `found: true` with real file/line/column.
4. Add positive tests for Rust → Ash usage lookup over committed `.ash` fixtures or stdlib files.
5. Add positive hover test showing `rust_context` is populated for a mapped symbol.
6. Ensure tests do not depend on developer home paths such as `~/.ash`.

### Property Requirements

Roundtrip/property tests for config serialization should remain green. Add a property that duplicate/empty mappings are rejected or resolved according to the config contract.

## TDD Steps

### Step 1: RED positive fixtures

**Files:**
- `.ash/cross_lang_config.yaml` or fixture path under `crates/ash-mcp/tests/fixtures/`
- `crates/ash-mcp/tests/cross_language_tools.rs`

Write tests that currently fail because default config returns no mappings.

### Step 2: Loader and fixture integration

Update config loading only as needed. Prefer deterministic workspace-relative loading over current-directory ambiguity.

### Step 3: Positive tool assertions

Assert exact `found: true`, symbol, kind, file suffix, and non-zero line/column. Avoid tests that accept `None` for known symbols.

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
  - cargo test -p ash-mcp cross_language_tools -- --nocapture
  - cargo test -p ash-mcp config -- --nocapture
  - cargo clippy -p ash-mcp --all-targets -- -D warnings
  - cargo fmt --check
checklist:
  - [x] Committed config is loaded by tests and/or server
  - [x] Known Ash symbol returns found Rust location
  - [x] Known Rust symbol returns Ash usage evidence
  - [x] Hover includes rust_context for mapped symbol
  - [x] No tests accept None for known positive mappings
```

## Dependencies for Next Task

This task provides the positive data needed by TASK-1431 evaluation.


## Implementation Evidence

- Added `.ash/cross_lang_config.yaml` and Ash fixture coverage.
- Added positive tests for `Effect` Ash→Rust and Rust→Ash lookup.
