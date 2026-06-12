# TASK-1431: Phase 141 Corpus Cross-Language Evaluation

## Status: ✅ Complete

## Description

Re-run and extend the Phase 141 benchmark corpus against the remediated Phase 142 cross-language tools. Produce the missing TASK-1426 evaluation evidence: accuracy, latency, token/tool-call impact where measurable, and honest limitations.

## Specification Reference

- PLAN-141: MCP benchmark baseline
- `scripts/benchmark/`
- TASK-1426: Phase evaluation
- PLAN-143 remediation scope

## Dependencies

- 📝 TASK-1430: Cross-language config and positive fixtures

## Requirements

### Functional Requirements

1. Extend the benchmark harness to invoke cross-language tools for Rust-heavy/mixed tasks T1–T6 where appropriate.
2. Record baseline Phase 141 results and Phase 143/remediated results side by side.
3. Measure at least: Ash → Rust success rate, Rust → Ash success rate, false-positive rate for configured corpus, average/p95 lookup latency, cache hit/miss latency.
4. Record unmeasured metrics honestly. If memory/startup cannot be measured in this task, mark them explicitly with a follow-up rather than claiming completion.
5. Update `docs/notes/PHASE-142-PERFORMANCE-BENCHMARK.md` or create `docs/notes/PHASE-143-CROSS-LANGUAGE-EVALUATION.md` with methodology, command lines, raw result path, and recommendations.

### Property Requirements

No proptest requirement; this is an evaluation/harness task. The harness should have deterministic fixture tests for expected result parsing.

## TDD Steps

### Step 1: Harness contract tests

**Files:**
- `scripts/benchmark/harness.py`
- `scripts/benchmark/corpus.py`
- optional new `scripts/benchmark/cross_language.py`

Add tests or a dry-run mode proving the new cross-language tasks are included and results schema includes accuracy/latency fields.

### Step 2: Run corpus

Run the benchmark and save raw outputs under `docs/notes/` or another tracked report path if compact. Large generated outputs should be summarized and ignored if inappropriate for git.

### Step 3: Write report

Compare against Phase 141 and explicitly state whether the 80%+ target is met.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 18
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - python3 scripts/benchmark/harness.py --help
  - python3 scripts/benchmark/harness.py --mode mcp --include-cross-language
  - cargo bench -p ash-mcp-bench --bench daemon_latency -- daemon_cache/cache_hit
  - cargo test -p ash-mcp-bench
  - cargo clippy -p ash-mcp-bench --all-targets -- -D warnings
  - cargo fmt --check
checklist:
  - [x] Phase 141 baseline comparison is recorded
  - [x] Cross-language accuracy is measured, not inferred
  - [x] Latency report includes average and p95 or explains why not
  - [x] Memory/startup metrics are measured or explicitly deferred with follow-up ownership
  - [x] Report command lines are reproducible
```

## Dependencies for Next Task

This task provides closeout evidence for TASK-1432.


## Implementation Evidence

- Extended `scripts/benchmark/harness.py` with `--include-cross-language`.
- Wrote `docs/notes/PHASE-143-MCP-CROSS-LANGUAGE-EVALUATION.md` with Phase 141 subset and cross-language metrics.
