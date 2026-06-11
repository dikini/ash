# TASK-1402: Agent evaluation harness for MCP tools

## Status: 📝 Planned

## Description

Create a reproducible evaluation suite that measures whether the MCP tools actually answer agent-style questions correctly. This gives us evidence before scaling cross-file analysis.

## Specification Reference

- [PLAN-140: MCP Agent Intelligence Spike](../PLAN-140-MCP-AGENT-INTELLIGENCE-SPIKE.md)

## Dependencies

- TASK-1400 and TASK-1401 complete.

## Requirements

### Functional Requirements

- Create fixture directory `crates/ash-mcp/tests/agent_queries/fixtures/` with at least three `.ash` files spanning multiple definitions.
- Add integration test file `crates/ash-mcp/tests/agent_queries.rs` that:
  - Loads fixtures.
  - Runs `ash_workspace_symbols`, `ash_find_references`, and `ash_goto_definition`.
  - Asserts expected results (file, line, name) for each query.
- Report precision/recall-style metrics in test output:
  - Number of queries.
  - Number passed.
  - Any failures with expected vs actual.

### Non-Functional Requirements

- Fixtures must be stable and version-controlled.
- Tests must run with `cargo test -p ash-mcp --test agent_queries`.
- Failures must print enough context to diagnose without re-running.

## Files

- Create: `crates/ash-mcp/tests/agent_queries.rs`
- Create: `crates/ash-mcp/tests/agent_queries/fixtures/lib.ash`
- Create: `crates/ash-mcp/tests/agent_queries/fixtures/main.ash`
- Create: `crates/ash-mcp/tests/agent_queries/fixtures/sensor.ash`

## TDD Steps

1. Write fixtures with known symbol locations.
2. Write integration tests with explicit expected results.
3. Run tests; expect failures if tools not yet implemented.
4. Implement/fix tools until tests pass.
5. Add metric summary output.

## Verification

- [ ] `cargo test -p ash-mcp --test agent_queries` passes.
- [ ] `cargo clippy -p ash-mcp --all-targets -- -D warnings` passes.
- [ ] `cargo fmt --check` passes.
- [ ] Test output reports query count and pass rate.
- [ ] CHANGELOG.md updated.
