# TASK-1407: Cross-reference precision/recall vs. grep

## Status: 📝 Planned

## Description

Quantify how often `ash_goto_definition` and `ash_find_references` return the
correct location compared to a naive `grep` baseline.

## Specification Reference

- [PLAN-141: MCP Agent Effectiveness Benchmark](../PLAN-141-MCP-BENCHMARK.md)

## Dependencies

- Phase 140 complete.

## Requirements

### Functional Requirements

- Sample 50 symbol names from the Ash codebase (functions, types, capabilities,
  workflows) across `crates/ash-core/src/`, `crates/ash-parser/src/`,
  `crates/ash-typeck/src/`.
- For each symbol:
  1. Run `ash_goto_definition` and record the returned file/line.
  2. Run `grep -rn "symbol_name" --include="*.ash"` and collect all matches.
  3. Manually verify ground truth (definition site).
  4. Run `ash_find_references` (where applicable) and compare to grep results.
- Compute precision and recall for both MCP tools vs. grep baseline.

### Non-Functional Requirements

- Sample must include symbols from multiple crates and symbol kinds.
- Ground truth must be verified by a human (or by AST inspection).

## Files

- Create: `scripts/precision-recall-benchmark.py`
- Create: `docs/notes/MCP-PRECISION-RECALL.json`

## TDD Steps

1. Sample 10 symbols; verify ground truth manually.
2. Run MCP tools and grep; compare results.
3. Scale to 50 symbols.
4. Compute precision/recall.

## Verification

- [ ] 50 symbols sampled and verified.
- [ ] Precision/recall scores computed for goto-definition and find-references.
- [ ] Results documented in JSON and markdown.
- [ ] CHANGELOG.md updated.
