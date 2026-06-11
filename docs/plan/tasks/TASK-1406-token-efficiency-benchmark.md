# TASK-1406: Token-efficiency benchmark — MCP vs. baseline

## Status: 📝 Planned

## Description

Run the benchmark harness to measure whether MCP-enabled agents consume fewer
tokens than baseline agents on real codebase exploration tasks.

## Specification Reference

- [PLAN-141: MCP Agent Effectiveness Benchmark](../PLAN-141-MCP-BENCHMARK.md)
- [TASK-1405: Benchmark harness](TASK-1405-benchmark-harness.md)

## Dependencies

- TASK-1405 complete.

## Requirements

### Functional Requirements

- Run each of the 5–10 corpus tasks in both `--baseline` and `--mcp` modes.
- Record total tokens (input + output) per task per mode.
- Run N=3 repetitions per task per mode; report median and standard deviation.
- Compute aggregate metrics:
  - Mean tokens saved per task (baseline − MCP)
  - Percentage token reduction
  - Tasks where MCP was worse (if any)

### Non-Functional Requirements

- Use fresh Hermes sessions for each run to avoid prompt caching bias.
- Randomize task order across runs to avoid ordering effects.
- Document any tasks where MCP failed to produce an answer.

## Files

- Modify: `scripts/mcp-benchmark.py` (add token counting)
- Create: `docs/notes/MCP-BENCHMARK-RAW.json` (raw results)

## TDD Steps

1. Run harness on 1 task in both modes; verify token counts are captured.
2. Scale to full corpus.
3. Compute aggregates; verify no negative token counts.

## Verification

- [ ] All corpus tasks run in both modes.
- [ ] Raw JSON results committed.
- [ ] Aggregate metrics computed and documented.
- [ ] CHANGELOG.md updated.
