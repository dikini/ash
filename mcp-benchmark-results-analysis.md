# MCP Agent Effectiveness Benchmark Results

## Metadata

| Field | Value |
|-------|-------|
| Baseline mode | baseline |
| MCP mode | mcp |
| Timestamp | 2026-06-12T05:27:42Z |
| Git commit | 5db0ff41aa5bbc021d7897fc7c34dde1828f5008 |
| ash-mcp version | 0.1.0 |

## Aggregate Comparison

| Metric | Baseline | MCP | Delta |
|--------|----------|-----|-------|
| Total time | 279ms | 4659ms | +4380ms |
| Total tool calls | 34 | 18 | -16 |
| Total tokens | ~134527 | ~3612 | ~+130,915 (+97.3%) |
| Avg accuracy | 0.44 | 0.22 | -0.22 |

## Per-Task Comparison

| Task | Baseline (ms/calls/tokens/acc) | MCP (ms/calls/tokens/acc) | Winner |
|------|-------------------------------|---------------------------|--------|
| T1 | 36ms / 4 / ~5229 / 1.0 | 511ms / 2 / ~352 / 0.0 | Baseline |
| T10 | 28ms / 4 / ~6159 / 0.0 | 535ms / 2 / ~432 / 0.5 | MCP |
| T2 | 26ms / 4 / ~14668 / 0.5 | 529ms / 2 / ~361 / 0.0 | Baseline |
| T3 | 28ms / 4 / ~42226 / 0.5 | 461ms / 2 / ~358 / 0.0 | Baseline |
| T4 | 19ms / 4 / ~8945 / 0.0 | 619ms / 2 / ~399 / 0.0 | MCP |
| T5 | 24ms / 4 / ~18954 / 0.5 | 461ms / 2 / ~364 / 0.0 | Baseline |
| T6 | 30ms / 2 / ~4830 / 0.5 | 519ms / 2 / ~356 / 0.0 | Baseline |
| T7 | 49ms / 4 / ~10598 / 1.0 | 514ms / 2 / ~418 / 0.5 | Baseline |
| T9 | 39ms / 4 / ~22918 / 0.0 | 510ms / 2 / ~572 / 1.0 | MCP |

## Interpretation

- **Token efficiency**: MCP uses fewer tokens overall.
  This is because MCP returns structured JSON results instead of full file contents.
- **Accuracy**: The accuracy scores reflect whether the expected files were found.
  Baseline grep often finds more files (higher recall) but with more noise.
  MCP is more precise when the symbol exists in `.ash` files but misses `.rs` files
  (current limitation of `ash_workspace_symbols`).
- **Tool calls**: MCP uses fewer tool calls because a single `ash_workspace_symbols`
  query replaces multiple `grep` + `read_file` operations.

## Limitations

1. **`.rs` vs `.ash`**: The current MCP tools only parse `.ash` files. Tasks involving
   `.rs` files (T1–T6, T8) show low MCP accuracy because the tools can't index Rust source.
2. **Token estimation**: Token counts are approximate (~4 chars/token). Real LLM tokenizers
   will differ.
3. **No agent loop**: This benchmark simulates agent behavior with scripted tool calls,
   not a real LLM agent. Actual token usage in a live agent loop may vary.
4. **Small corpus**: 9 tasks is a small sample. Results may not generalize to all
   codebase exploration patterns.

## Recommendation

Based on these results:

- **For `.ash` files**: MCP tools show promise — T9 (workflow primitives) achieved
  perfect accuracy with ~97% fewer tokens than baseline.
- **For `.rs` files**: MCP tools are currently ineffective. Cross-file analysis for
  Rust source would require extending `ash-lsp-core` to index `.rs` files or
  integrating with rust-analyzer.
- **Next step**: Extend the benchmark to measure real LLM agent loops (TASK-1406),
  or invest in making MCP tools work for `.rs` files before scaling.
