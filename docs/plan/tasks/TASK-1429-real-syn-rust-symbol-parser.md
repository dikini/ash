# TASK-1429: Real syn Rust Symbol Parser

## Status: 📝 Planned

## Description

Replace Phase 142's textual Rust declaration search with actual `syn` parsing and span extraction for supported Rust items. If a construct is unsupported, the parser must return an honest miss/error rather than fabricating locations.

## Specification Reference

- PLAN-142 TASK-1421 Rust symbol location requirement
- PLAN-143 remediation scope
- rust-skills reference: `references/syn-2-0-api-patterns.md`
- `crates/ash-mcp/src/rust_parser.rs`

## Dependencies

- 📝 TASK-1428: Wire cross-language MCP tools

## Requirements

### Functional Requirements

1. Use `syn::parse_file` with `full`/`parsing` features to parse Rust source files.
2. Support at minimum: `struct`, `enum`, `trait`, `type`, free `fn`, `mod`, and inherent/trait `impl` item heads where feasible.
3. Extract line/column spans from `proc_macro2` spans or an equivalent reliable span strategy. If span locations require an additional feature, add it explicitly and document why.
4. Remove or justify `#[allow(clippy::cast_possible_truncation)]`; prefer checked conversions or span types matching the API.
5. Add regression tests proving comments/string literals do not produce false symbol locations.
6. Keep graceful `Ok(None)` behavior for unknown symbols.

### Property Requirements

Add property tests for unknown/random symbol names against a fixture file: lookup should never panic and should return `None` unless the generated name matches a fixture declaration.

## TDD Steps

### Step 1: RED parser fixtures

**File:** `crates/ash-mcp/src/rust_parser.rs`

Add tests with a temporary Rust file containing:
- real declarations,
- comments containing fake declarations,
- string literals containing fake declarations,
- nested modules or impl blocks if in scope.

Textual matching should fail these tests before implementation.

### Step 2: Implement syn traversal

Parse with `syn::parse_file`, traverse `syn::Item` variants, and use the syn 2.0 patterns from `rust-skills`.

### Step 3: Integrate with tool lookup

Ensure `find_rust_symbol_location_real` consumes the parser result and preserves exact file/line/column values.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-mcp rust_parser -- --nocapture
  - cargo test -p ash-mcp symbol_mapping -- --nocapture
  - cargo clippy -p ash-mcp --all-targets -- -D warnings
  - cargo fmt --check
checklist:
  - [ ] Parser uses syn rather than line-pattern matching
  - [ ] Comments/string literals do not create false positives
  - [ ] Supported item kinds have tests
  - [ ] Unknown symbols return Ok(None)
  - [ ] CHANGELOG and task docs do not overclaim unsupported syntax
```

## Dependencies for Next Task

This task provides the parser reality needed by TASK-1430 and TASK-1431.
