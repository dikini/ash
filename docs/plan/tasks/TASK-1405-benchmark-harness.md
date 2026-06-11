# TASK-1405: Build benchmark harness and corpus

## Status: 📝 Planned

## Description

Create a reproducible benchmark harness that can drive both MCP-enabled and
baseline agents through standardized codebase exploration tasks, collecting
tokens, time, and accuracy metrics.

## Specification Reference

- [PLAN-141: MCP Agent Effectiveness Benchmark](../PLAN-141-MCP-BENCHMARK.md)

## Dependencies

- Phase 140 complete (MCP tools available).

## Requirements

### Functional Requirements

- Create `crates/ash-mcp/benches/agent_benchmark.rs` or standalone script
  `scripts/mcp-benchmark.py` that:
  - Defines a corpus of 5–10 real codebase exploration tasks using files from
    `crates/ash-core/src/`, `crates/ash-parser/src/`, etc.
  - Can run in two modes: `--baseline` (no MCP) and `--mcp` (with ash-mcp).
  - Measures per-task:
    - Total tokens consumed (input + output)
    - Number of tool calls
    - Wall-clock time to correct answer
    - Accuracy score (0 = wrong, 0.5 = partially correct, 1 = correct)
- Corpus tasks must require cross-file understanding (e.g., "Find all
  implementations of the `Effect` trait", "Where is `capability sensor` defined
  and where is it used?").

### Non-Functional Requirements

- Harness must be deterministic (fixed task ordering, no randomness).
- Must not mutate the filesystem.
- Must emit JSON results for programmatic analysis.

## Files

- Create: `scripts/mcp-benchmark.py` (or `crates/ash-mcp/benches/agent_benchmark.rs`)
- Create: `docs/plan/tasks/TASK-1405-benchmark-harness.md` (this file)

## TDD Steps

1. Write harness skeleton with mock tasks.
2. Verify it can launch a Hermes session and capture output.
3. Add real tasks from the Ash codebase.
4. Verify JSON output schema.

## Verification

- [ ] Harness runs without errors in both `--baseline` and `--mcp` modes.
- [ ] JSON output contains all required fields.
- [ ] `cargo fmt --check` / `black --check` passes.
- [ ] CHANGELOG.md updated.
