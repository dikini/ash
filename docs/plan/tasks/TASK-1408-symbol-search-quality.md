# TASK-1408: Workspace symbol search latency and relevance

## Status: 📝 Planned

## Description

Measure the latency and result quality of `ash_workspace_symbols` compared to
naive directory scanning.

## Specification Reference

- [PLAN-141: MCP Agent Effectiveness Benchmark](../PLAN-141-MCP-BENCHMARK.md)

## Dependencies

- Phase 140 complete.

## Requirements

### Functional Requirements

- Define 10 search queries (e.g., "sensor", "read", "helper", "main",
  "capability", "workflow", "effect", "observe", "set", "send").
- For each query:
  1. Measure `ash_workspace_symbols` latency (time to first result).
  2. Measure naive `find . -name "*.ash" | xargs grep -l` latency.
  3. Judge relevance of top-5 MCP results (human or heuristic: exact match >
     substring > no match).
- Compute:
  - Mean/median latency for both methods.
  - Relevance score (0–1) for top-5 results.
  - False positive rate (results that don't match the query intent).

### Non-Functional Requirements

- Latency measured on warm filesystem cache (run once to warm, then measure).
- Queries must cover common agent search patterns.

## Files

- Create: `scripts/symbol-search-benchmark.py`
- Create: `docs/notes/MCP-SYMBOL-SEARCH.json`

## TDD Steps

1. Run 3 queries; verify latency measurement is stable.
2. Scale to 10 queries.
3. Compute aggregates.

## Verification

- [ ] 10 queries measured.
- [ ] Latency and relevance metrics computed.
- [ ] Results documented.
- [ ] CHANGELOG.md updated.
