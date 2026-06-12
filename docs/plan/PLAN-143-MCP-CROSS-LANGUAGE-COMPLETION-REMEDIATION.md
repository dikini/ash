# PLAN-143: MCP Cross-Language Completion Remediation

## Phase: 143

## Status: 📝 Planned

## Goal

Repair Phase 142's completion gaps so ash-mcp cross-language integration is real, wired, measurable, and honestly documented. This phase converts the Phase 142 review findings into atomic remediation tasks with explicit gates.

## Background

A post-merge review of Phase 142 on `main` found that focused `ash-mcp` gates pass, but the phase overclaims completion:

1. `ash_find_rust_implementation` and `ash_find_ash_usage` are present in a split-out source file but are not wired into the compiled crate or MCP tool registry.
2. The committed `rust_parser.rs` claims real `syn` AST parsing, but currently performs textual line-pattern matching.
3. Phase/task status surfaces still say planned/not-started in the phase-owned plan and task files while `PLAN-INDEX.md` says complete.
4. TASK-1426's evaluation report was not fulfilled: no Phase 141 corpus re-run, accuracy metrics, productivity metrics, or full latency/memory/startup comparison.
5. Default cross-language config loading has no committed project config at a path the server reads.
6. Cleanup left a stale prunable worktree and an accidental tracked path with a space.

## Scope

### In scope

1. Wire Phase 142 cross-language MCP tools into the real compiled `ash-mcp` crate.
2. Replace textual Rust symbol search with actual `syn` parsing and span extraction, or explicitly downgrade docs if a narrower implementation is chosen.
3. Add committed cross-language config fixtures and positive end-to-end tests.
4. Re-run and extend the Phase 141 benchmark corpus against the new cross-language tools.
5. Reconcile Phase 142 and Phase 143 status surfaces honestly.
6. Clean stale worktree metadata and accidental tracked artifacts.

### Out of scope

- Full cross-language type checking.
- rust-analyzer integration beyond using the existing rust source parser surface.
- Live editor incremental sync.
- Whole-workspace semantic graph indexing beyond the benchmark/evaluation harness needed for this remediation.

## Tasks

| Task | Description | Estimate | Status |
|------|-------------|----------|--------|
| [TASK-1427](tasks/TASK-1427-phase142-status-and-artifact-hygiene.md) | Reconcile Phase 142 status surfaces and artifact hygiene | 3h | 📝 Planned |
| [TASK-1428](tasks/TASK-1428-wire-cross-language-mcp-tools.md) | Wire cross-language MCP tools into compiled ash-mcp | 5h | 📝 Planned |
| [TASK-1429](tasks/TASK-1429-real-syn-rust-symbol-parser.md) | Implement real `syn` Rust source parsing and symbol spans | 6h | 📝 Planned |
| [TASK-1430](tasks/TASK-1430-cross-language-config-and-positive-fixtures.md) | Add project config loading and positive fixture coverage | 4h | 📝 Planned |
| [TASK-1431](tasks/TASK-1431-phase141-corpus-cross-language-evaluation.md) | Re-run Phase 141 corpus with cross-language tools and report metrics | 6h | 📝 Planned |
| [TASK-1432](tasks/TASK-1432-phase143-closeout-and-rereview.md) | Close out Phase 143 with clean gates, status reconciliation, and independent review | 3h | 📝 Planned |

## Deliverables

- `ash-mcp` health/tool registry includes `ash_find_rust_implementation` and `ash_find_ash_usage`.
- Positive MCP/tool-level tests prove Ash → Rust and Rust → Ash lookup works with committed config.
- `rust_parser.rs` uses `syn::parse_file` and span extraction for supported Rust items, with honest fallback/error behavior.
- Benchmark/evaluation report records Phase 141 baseline comparison, cross-language accuracy, lookup latency, memory/startup status, and recommendations.
- `PLAN-142`, `TASK-1420`–`TASK-1426`, `PLAN-143`, `TASK-1427`–`TASK-1432`, `PLAN-INDEX.md`, and `CHANGELOG.md` agree on actual state.

## Verification Strategy

Focused gates:

```bash
cargo fmt --check
cargo test -p ash-mcp
cargo clippy -p ash-mcp --all-targets -- -D warnings
cargo test -p ash-mcp-bench
cargo clippy -p ash-mcp-bench --all-targets -- -D warnings
cargo bench -p ash-mcp-bench --bench daemon_latency -- daemon_cache/cache_hit
python3 scripts/benchmark/harness.py --mode mcp --include-cross-language
```

Broad gate for closeout when practical:

```bash
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --lib -- --test-threads=1
```

If broad gates reveal unrelated pre-existing failures, classify them using the project ZTB protocol and record exact evidence in TASK-1432.

## Risks

| Risk | Mitigation |
|------|------------|
| `syn` span extraction needs extra crate features | Start TASK-1429 with a failing compile/test fixture and use the documented `rust-skills` syn 2.0 patterns |
| MCP macro tool registration conflicts when moving functions | TASK-1428 owns the module wiring and health/tool-router tests before parser changes |
| Benchmark harness cannot exercise MCP tools directly | TASK-1431 may add a lightweight direct Rust harness, but must document any deviation from Phase 141 methodology |
| Status remediation accidentally overclaims Phase 142 again | TASK-1432 requires final re-review focused on source-of-truth drift |

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-12 | Create Phase 143 instead of silently patching Phase 142 | Phase 142 was already merged; remediation needs explicit traceability and review gates |
| 2026-06-12 | Keep scope bounded to tool wiring, parser reality, config, benchmarks, and status corpus | Matches review findings and avoids expanding into full cross-language type checking |

---

**Phase Authority**: [Ash Implementation Plan](PLAN-INDEX.md)  
**Review Source**: Post-merge Phase 142 review on `main`  
**Related Work**: [Phase 140](PLAN-140-MCP-AGENT-INTELLIGENCE-SPIKE.md), [Phase 141](PLAN-141-MCP-BENCHMARK.md), [Phase 142](PLAN-142-MCP-CROSS-LANGUAGE-INTEGRATION.md)
