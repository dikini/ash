# Phase 143: MCP Cross-Language Remediation Evaluation

## Metadata

| Field | Value |
|-------|-------|
| Date | 2026-06-12T19:08:19Z |
| Baseline JSON | `/tmp/phase143-baseline.json` |
| MCP JSON | `/tmp/phase143-mcp-final12.json` |
| Git commit | 0d761852b6731709d27c5de7c8a1cebed9f60f59 |
| ash-mcp version | 0.1.0 |
| Cross-language evaluator | Configured mappings exercised through `ash_find_rust_implementation` and `ash_find_ash_usage` MCP tools |

## Phase 141 Corpus Subset Re-run

This evaluation re-ran a bounded subset of the Phase 141 corpus (`T1`, `T7`, `T9`) to verify that Phase 143 did not regress the existing MCP benchmark harness while adding cross-language evidence.

| Metric | Baseline | MCP | Delta |
|--------|----------|-----|-------|
| Total time | 148ms | 478ms | +330ms |
| Tool calls | 12 | 6 | -6 |
| Approx tokens | ~38884 | ~1536 | -37348 |
| Average accuracy | 0.67 | 0.50 | -0.17 |

Per-task MCP results preserve the Phase 141 boundary: `.ash`-native tasks benefit most, while Rust-source exploration should use the explicit cross-language mapping tools rather than `ash_workspace_symbols` alone.

## Cross-Language Mapping Evaluation

| Metric | Value |
|--------|-------|
| Total configured mappings evaluated | 3 |
| Ash → Rust accuracy | 100.00% |
| Rust → Ash accuracy | 100.00% |
| False-positive rate | 0.00% (0/2 negative probes) |
| Average lookup latency | 25.709ms |
| P95 lookup latency | 26.023ms |
| Total cross-language evaluation latency | 86.571ms |
| Memory usage | Not measured by this scripted harness |
| Startup time | Not measured by this scripted harness |

### Mapping Rows

| Ash symbol | Rust symbol | Rust file | Ash→Rust | Rust→Ash | Ash usage count | Latency |
|------------|-------------|-----------|----------|----------|-----------------|---------|
| `Effect` | `ash_core::effect::Effect` | `/home/dikini/Projects/ash/.worktrees/phase-143-mcp-remediation/crates/ash-core/src/effect.rs` | ✅ | ✅ | 3 | 23.29ms |
| `CapabilityProvider` | `ash_core::capability::CapabilityProvider` | `/home/dikini/Projects/ash/.worktrees/phase-143-mcp-remediation/crates/ash-core/src/capability.rs` | ✅ | ✅ | 1 | 24.124ms |
| `CapabilityError` | `ash_core::capability::CapabilityError` | `/home/dikini/Projects/ash/.worktrees/phase-143-mcp-remediation/crates/ash-core/src/capability.rs` | ✅ | ✅ | 1 | 24.988ms |

## Independent Review Follow-up

Codex review found and Phase 143 addressed twenty-two closeout issues:

1. Workspace resolution used the compile-time manifest root instead of the request workspace.
2. Qualified Ash variant symbols such as `Effect::Epistemic` were not normalized to configured base symbols.
3. Rust → Ash usage scanning originally reported only the first substring per line.
4. The cross-language evaluation originally used hardcoded direct probes instead of the delivered MCP tools.
5. Rust → Ash usage scanning needed to skip comment/string matches.
6. `--include-cross-language` needed its own MCP-mode/binary guard.
7. Reverse usage scanning needed to honor configured `ash_extensions`.
8. Namespace-qualified Ash symbols such as `std::types::Effect` needed terminal-symbol fallback.
9. Reverse usage scanning needed to mask all Ash comment forms used by the parser (`//`, `--`, and block comments).
10. Token-boundary checks needed to treat hyphen as an Ash identifier continuation character.
11. Fully-qualified Ash enum variants such as `std::types::Effect::Epistemic` needed segment-based mapping normalization.
12. Reverse Ash usage scans needed to scan from the discovered config/workspace root rather than the server cwd.
13. Rust associated-item lookups such as `ash_core::effect::Effect::join` needed progressive containing-module file resolution.
14. `syn` traversal needed to include trait items for trait-method mappings.
15. Associated-item lookups needed to preserve the containing type/trait to avoid ambiguous `new`/`from` matches.
16. The benchmark needed measured negative probes before reporting a false-positive rate.
17. Config discovery needed to avoid loading build-time Ash checkout config for unrelated workspaces.
18. Nested-module Rust symbols needed to remain terminal-symbol lookups instead of being mistaken for associated items.
19. Workspace-root fallback needed to avoid leaking the server cwd config into unrelated scratch files.
20. P95 lookup latency needed an actual percentile calculation instead of using the maximum sample.
21. Ash mapping lookup needed exact/qualified-parent precedence before terminal fallback.
22. Reverse usage string masking needed escaped-quote handling to avoid string-literal false positives.

Regression coverage now includes qualified variant lookup, namespace-qualified fallback, fully-qualified namespace+variant lookup, associated-method source lookup, trait-item lookup, container-specific impl-method lookup, config-root discovery from a crate subdirectory, token-aware multi-usage scanning with comment/string/comment-form masking, configured-extension scanning, and hyphenated identifier false-positive guards. This report's cross-language section is generated from configured mappings exercised through the MCP tools, including negative probes for the false-positive metric.

## Interpretation

- Phase 143 resolves the Phase 142 review blocker that cross-language claims lacked positive evidence: committed config and fixture-backed MCP calls now prove configured Ash symbols resolve to real Rust source files and Rust symbols resolve back to Ash usages.
- The existing Phase 141 general corpus benchmark still shows MCP token efficiency on selected tasks, but it should not be read as full Rust source indexing.
- Memory and startup metrics remain out of scope for this scripted remediation report; daemon latency benchmarks remain documented separately in `docs/notes/PHASE-142-PERFORMANCE-BENCHMARK.md`.

## Recommendation

Close Phase 143 after final gates and independent review remain green. Future work should add startup/memory measurement if those become acceptance gates and consider parser-backed Ash semantic usage classification beyond the current token-aware lexical scanner.
