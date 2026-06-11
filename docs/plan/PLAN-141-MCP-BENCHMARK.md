# PLAN-141: MCP Agent Effectiveness Benchmark

## Phase: 141

## Status: 📝 Planned

## Goal

Measure whether the Ash MCP tools (delivered in Phase 140) materially improve
agent productivity on real codebase tasks. Produce reproducible, quantitative
evidence that guides the decision to scale cross-file analysis or pivot.

## Background

Phase 140 delivered working MCP tools (workspace symbols, find-references,
goto-definition, diagnostics, hover, completion) and an evaluation harness that
proves correctness on synthetic fixtures. What remains unknown is whether these
tools reduce token burn, speed up task completion, and improve accuracy on
real Ash codebases compared to baseline text-search agents.

This phase is a **measurement spike**, not a feature build. The deliverable is
a benchmark report with numbers, not new LSP capabilities.

## Scope

### In scope

1. **Token-efficiency benchmark** — compare MCP-enabled vs. baseline agents on
   real codebase exploration tasks; measure tokens, tool calls, time, accuracy.
2. **Cross-reference precision/recall** — quantify `goto_definition` and
   `find_references` accuracy against `grep` baseline on a sampled symbol set.
3. **Workspace symbol search quality** — measure latency and relevance of
   `ash_workspace_symbols` vs. naive directory scan.
4. **Benchmark harness** — reproducible script/crate that runs the above and
   emits a structured report.

### Out of scope

- New MCP tools or LSP features (those belong in a follow-up phase).
- Human UX studies (we measure agent behavior, not human IDE usage).
- Cross-file analysis implementation (deferred pending benchmark results).

## Specification

- [MCP-SPIKE-RESULTS.md](../notes/MCP-SPIKE-RESULTS.md) — Phase 140 outcomes.
- [SPEC-038: Rust LSP / MCP Research 2025](../spec/SPEC-038-RUST-LSP-MCP-RESEARCH-2025.md)

## Tasks

| Task | Description | Estimate | Status |
|------|-------------|----------|--------|
| [TASK-1405](tasks/TASK-1405-benchmark-harness.md) | Build benchmark harness and corpus | 4h | 📝 Planned |
| [TASK-1406](tasks/TASK-1406-token-efficiency-benchmark.md) | Token-efficiency benchmark: MCP vs. baseline | 4h | 📝 Planned |
| [TASK-1407](tasks/TASK-1407-precision-recall-benchmark.md) | Cross-reference precision/recall vs. grep | 3h | 📝 Planned |
| [TASK-1408](tasks/TASK-1408-symbol-search-quality.md) | Workspace symbol search latency/relevance | 2h | 📝 Planned |
| [TASK-1409](tasks/TASK-1409-benchmark-report.md) | Compile benchmark report and recommendation | 3h | 📝 Planned |

## Deliverable

- `docs/notes/MCP-BENCHMARK-RESULTS.md` with:
  - Token-efficiency comparison table (MCP vs. baseline).
  - Precision/recall scores for goto-definition and find-references.
  - Symbol search latency and relevance metrics.
  - Recommendation: scale / pivot / integrate.

## Timeline

~16 hours (bounded measurement spike).

## Risks

| Risk | Mitigation |
|------|------------|
| Agent token counts noisy across runs | Run each task N=3 times, report median + stddev. |
| Baseline agent cheats with prior knowledge | Use fresh Hermes sessions; randomize task order. |
| Corpus too small to generalize | Sample 50+ symbols; use full `crates/` tree as task substrate. |
| Hermes session logs don't expose token counts | Fallback: instrument via OpenRouter API or proxy. |

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| TBD | Use `crates/` as benchmark corpus | Real codebase, already present, no external deps. |
| TBD | Measure tokens via Hermes session DB + OpenRouter proxy | Hermes logs token usage per session; if unavailable, proxy MCP traffic. |

## Notes

- This plan was created after Phase 140 closeout. The benchmark results will
  inform whether Phase 142 (cross-file analysis scale-up) is justified.
- Baseline agent = Hermes with `file` + `terminal` toolsets only (no MCP).
- MCP-enabled agent = Hermes with `ash-mcp` server configured.
