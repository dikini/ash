# SPEC-084: Flaky-Test Quarantine and Distributed Orchestration

**Status:** Implemented MVP
**Date:** 2026-06-15
**Builds on:** [SPEC-081](SPEC-081-LAW-TEST-EVIDENCE-SUBSTRATE.md)
**Plan:** [PLAN-148: Flaky-Test Quarantine and Distributed Orchestration](../plan/PLAN-148-FLAKY-TEST-QUARANTINE-AND-DISTRIBUTED-ORCHESTRATION.md)

## Summary

Add operational test-runner capabilities for retry/flake classification, quarantine metadata, shard planning, and deterministic result merging.

## Motivation

Phase 145 made empirical law evidence explicit but intentionally left several important `ash test` gaps visible. This specification defines the next scoped slice for those gaps while preserving the project rule that Ash law/test/proof authors and executors validate supported behavior through an Ash executable, not through Cargo or Rust test harnesses.

## Evidence Boundary

Implementation agents must distinguish three command classes:

1. **Implementation health** — Rust commands such as `cargo test`, `cargo clippy`, and `cargo fmt`; useful and required for implementers.
2. **Candidate Ash final surface** — direct invocations of an Ash-under-test executable:

   ```bash
   ${ASH_UNDER_TEST:?set Ash candidate binary} test fixtures/<phase-fixture> --format json
   ```

3. **Release/install parity** — ordinary `ash` on PATH after install/release catches up; closeout must either prove parity or record an explicit handoff.

`cargo run -p ash-cli -- test ...` is never final-surface evidence.

## Scope

### In Scope

- Retry policy and flake classification.
- Quarantine metadata and malformed-quarantine diagnostics.
- Shard plan format and local shard execution.
- Deterministic merge of shard JSON outputs.
- No-Cargo flake/quarantine/shard fixtures.

### Non-Goals

- coverage/mutation semantics
- new generator/shrinker semantics
- remote cluster provisioning
- proof-producing synthesis

## Required Agent Skills

Implementation agents must load and follow:

- `rust-skills` for Rust code, public APIs, proptest coverage, error handling, and clippy-clean implementation.
- `ash-language-feature-spec-writing` for Ash surface/parser/typechecker/runner contracts and final-surface Ash examples.
- `test-driven-development` when implementing code slices: write failing Rust/Ash-facing tests before production changes.
- `verification-before-completion` before marking any task complete.
- `systematic-debugging` for any unexpected runner, parser, or property failure.

## Examples

```bash
$ASH_UNDER_TEST test fixtures/phase148-flakes --retries 2 --format json
$ASH_UNDER_TEST test fixtures/phase148-shards --shard 1/3 --format json > shard-1.json
$ASH_UNDER_TEST test --merge-results shard-*.json --format json
```

Quarantined flaky tests should be visible but not silently counted as ordinary passes.

## Result and Reporting Requirements

- JSON output must remain machine-readable and stable enough for later orchestration.
- Unsupported cases must be `deferred`, `untested`, or explicit errors; they must not be counted as passing evidence.
- Repro artifacts must include enough data for a direct `$ASH_UNDER_TEST test ...` replay when the phase owns execution behavior.
- Human output should summarize the new capability without hiding caveats.

## Implemented MVP Semantics

- `--retries N` retries failing authored test rows up to `N` times and attaches per-attempt evidence. A row that fails before eventually passing is classified with `flake.status = "flaky"`; ordinary success is not hidden.
- `--shard INDEX/TOTAL` uses one-based deterministic local shard selection over the sorted discovered authored-test list and emits `shard.schema_version = "ash-shard-v1.0"`.
- `--merge-results FILE...` reads shard JSON files without rerunning tests, rejects invalid shard ranges, failed shard envelopes, missing tests arrays, duplicate shard IDs, missing shard IDs, and duplicate `(path, name)` test rows, and emits `merge.schema_version = "ash-merge-v1.0"`.
- `-- @test quarantine: <reason>` metadata keeps a row visible while remapping it to `skip` with `quarantine.original_outcome`; an empty quarantine directive fails closed with an explicit error.

The first slice is local orchestration only: no remote worker lifecycle, queue protocol, artifact upload, or release/install parity beyond the candidate `$ASH_UNDER_TEST` binary is claimed by this spec.

## Implementation Tasks

- [TASK-1474](../plan/tasks/TASK-1474-flake-orchestration-audit.md): Audit runner orchestration seams
- [TASK-1475](../plan/tasks/TASK-1475-retry-policy-and-flake-schema.md): Define retry policy and flake schema
- [TASK-1476](../plan/tasks/TASK-1476-flaky-test-quarantine-metadata.md): Implement quarantine metadata handling
- [TASK-1477](../plan/tasks/TASK-1477-flake-final-surface-fixtures.md): Add flaky/quarantine final-surface fixtures
- [TASK-1478](../plan/tasks/TASK-1478-shard-plan-schema.md): Define shard plan schema
- [TASK-1479](../plan/tasks/TASK-1479-local-shard-execution.md): Implement local shard execution
- [TASK-1480](../plan/tasks/TASK-1480-distributed-result-merge.md): Implement distributed result merge
- [TASK-1481](../plan/tasks/TASK-1481-flake-orchestration-closeout.md): Close out flake/orchestration phase

## Changelog

### 2026-06-15

- Created this planning specification and registered PLAN-148 / TASK-1474 through TASK-1481.
- Implemented the Phase 148 MVP for retries/flakes, quarantine metadata, deterministic local shards, shard JSON merge, no-Cargo fixtures, and final-surface evidence.
