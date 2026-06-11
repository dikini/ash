# TASK-1401: Single-file find-references via MCP

## Status: 📝 Planned

## Description

Implement same-file find-references so agents can ask "where is `helper` used?" within the file they are editing. Cross-file references remain deferred.

## Specification Reference

- [PLAN-140: MCP Agent Intelligence Spike](../PLAN-140-MCP-AGENT-INTELLIGENCE-SPIKE.md)
- [SPEC-038: Rust LSP / MCP Research 2025](../../spec/SPEC-038-RUST-LSP-MCP-RESEARCH-2025.md)
- [SPEC-043: Incremental Analysis Engine](../../spec/SPEC-043-INCREMENTAL-ANALYSIS.md)

## Dependencies

- TASK-1399 complete.
- Existing `goto::goto_definition` token resolution logic.

## Requirements

### Functional Requirements

- In `ash-lsp-core/src/goto.rs`, add `find_references(module, source, line, col) -> Vec<Location>`.
- Resolve the token at the requested position using existing `token_at_offset`.
- Walk the source text and return every span where the same identifier token appears.
- Include the definition site as the first result.
- Return results sorted by source order.
- Expose as MCP tool `ash_find_references` replacing the current placeholder.

### Non-Functional Requirements

- Same-file only; tool description must state this limitation honestly.
- Token equality must be exact (no substring matches).
- Robust to identifiers separated by non-identifier characters.

## Files

- Modify: `crates/ash-lsp-core/src/goto.rs`
- Modify: `crates/ash-lsp-core/src/position.rs` if helper needed for token boundary detection.
- Modify: `crates/ash-mcp/src/lib.rs`
- Modify: `crates/ash-mcp/src/tests.rs`

## TDD Steps

1. Write unit test: references to a function include definition and call site.
2. Write unit test: references to a workflow name include only its declaration.
3. Write unit test: references to a capability name in `observe sensor` resolve.
4. Write unit test: no false positives for substring matches (e.g., `help` vs `helper`).
5. Implement `find_references` and expose via MCP.

## Verification

- [ ] `cargo test -p ash-lsp-core -p ash-mcp` passes.
- [ ] `cargo clippy -p ash-lsp-core -p ash-mcp --all-targets -- -D warnings` passes.
- [ ] `cargo fmt --check` passes.
- [ ] Placeholder message removed from `ash_find_references`.
- [ ] CHANGELOG.md updated.
