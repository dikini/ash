# MCP Agent Effectiveness Benchmark Results

## Metadata

| Field | Value |
|-------|-------|
| Baseline mode | baseline |
| MCP mode | mcp |
| Timestamp | 2026-06-11T21:14:41Z |
| Git commit | 11b9036452589162837b18065343c82eabd63ead |
| ash-mcp version | 0.1.0 |

## Aggregate Comparison

| Metric | Baseline | MCP | Delta |
|--------|----------|-----|-------|
| Total time | 280ms | 408ms | +128ms |
| Total tool calls | 34 | 18 | -16 |
| Total tokens | ~134747 | ~3985 | ~+130,762 (+97.0%) |
| Avg accuracy | 0.44 | 0.22 | -0.22 |

## Per-Task Comparison

| Task | Baseline (ms/calls/tokens/acc) | MCP (ms/calls/tokens/acc) | Winner |
|------|-------------------------------|---------------------------|--------|
| T1 | 35ms / 4 / ~5255 / 1.0 | 49ms / 2 / ~377 / 0.0 | Baseline |
| T10 | 22ms / 4 / ~6185 / 0.0 | 46ms / 2 / ~483 / 0.5 | MCP |
| T2 | 27ms / 4 / ~14695 / 0.5 | 45ms / 2 / ~388 / 0.0 | Baseline |
| T3 | 30ms / 4 / ~42252 / 0.5 | 44ms / 2 / ~385 / 0.0 | Baseline |
| T4 | 20ms / 4 / ~8970 / 0.0 | 45ms / 2 / ~435 / 0.0 | MCP |
| T5 | 28ms / 4 / ~18981 / 0.5 | 46ms / 2 / ~390 / 0.0 | Baseline |
| T6 | 37ms / 2 / ~4839 / 0.5 | 44ms / 2 / ~380 / 0.0 | Baseline |
| T7 | 33ms / 4 / ~10625 / 1.0 | 44ms / 2 / ~470 / 0.5 | Baseline |
| T9 | 48ms / 4 / ~22945 / 0.0 | 45ms / 2 / ~677 / 1.0 | MCP |

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
