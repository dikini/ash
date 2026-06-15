# PLAN-148: Flaky-Test Quarantine and Distributed Orchestration

**Status:** ✅ Complete
**Spec:** [SPEC-084: Flaky-Test Quarantine and Distributed Orchestration](../spec/SPEC-084-FLAKY-TEST-QUARANTINE-AND-DISTRIBUTED-ORCHESTRATION.md)
**Depends on:** [PLAN-145: Law Test Evidence Substrate](PLAN-145-LAW-TEST-EVIDENCE-SUBSTRATE.md)
**Task range:** TASK-1474 through TASK-1481
**Estimated effort:** 32h

## Overview

Add operational test-runner capabilities for retry/flake classification, quarantine metadata, shard planning, and deterministic result merging.

## Goals

- [x] Classify flakes with retries instead of hiding them.
- [x] Support quarantine metadata and deterministic local sharding.
- [x] Merge shard results without losing evidence artifacts.

## Non-Goals

- coverage/mutation semantics
- new generator/shrinker semantics
- remote cluster provisioning
- proof-producing synthesis

## Orchestrator Guidance

- Create a dedicated worktree before implementation, for example `.worktrees/phase-148-flaky-test-quarantine-and-distributed-orchestration`.
- Load `rust-skills`, `ash-language-feature-spec-writing`, `test-driven-development`, `systematic-debugging`, and `verification-before-completion` before code work.
- Use rust-analyzer MCP/LSP for Rust symbol tracing before broad text search.
- Keep tasks small and sequential where schema/result formats are dependencies.
- Require direct `$ASH_UNDER_TEST test ...` evidence for user-facing runner behavior; Rust tests alone are bridge evidence.
- Update `CHANGELOG.md` and relevant `reference/tools/test.md` wording in the closeout task.

## Task Plan

| Task | Title | Estimate | Status |
|---|---|---:|---|
| [TASK-1474](tasks/TASK-1474-flake-orchestration-audit.md) | Audit runner orchestration seams | 4h | ✅ Complete |
| [TASK-1475](tasks/TASK-1475-retry-policy-and-flake-schema.md) | Define retry policy and flake schema | 4h | ✅ Complete |
| [TASK-1476](tasks/TASK-1476-flaky-test-quarantine-metadata.md) | Implement quarantine metadata handling | 4h | ✅ Complete |
| [TASK-1477](tasks/TASK-1477-flake-final-surface-fixtures.md) | Add flaky/quarantine final-surface fixtures | 4h | ✅ Complete |
| [TASK-1478](tasks/TASK-1478-shard-plan-schema.md) | Define shard plan schema | 4h | ✅ Complete |
| [TASK-1479](tasks/TASK-1479-local-shard-execution.md) | Implement local shard execution | 4h | ✅ Complete |
| [TASK-1480](tasks/TASK-1480-distributed-result-merge.md) | Implement distributed result merge | 4h | ✅ Complete |
| [TASK-1481](tasks/TASK-1481-flake-orchestration-closeout.md) | Close out flake/orchestration phase | 4h | ✅ Complete |

## Decision Gates

- D1: retry/flake schema lands before quarantine behavior.
- D2: shard plan schema lands before local shard execution.
- D3: merge rejects duplicate/missing shard IDs before producing aggregate success.

## Verification Strategy

Each implementation task must include:

1. Focused Rust tests for new parser/runner/schema behavior.
2. Focused Ash fixture tests where the behavior is user-facing.
3. Direct Ash-under-test command evidence:

   ```bash
   export ASH_UNDER_TEST=/absolute/path/to/candidate/ash
   "$ASH_UNDER_TEST" test fixtures/phase148-... --format json
   ```

4. `cargo fmt --check`, focused `cargo test`, and focused `cargo clippy` for touched crates.

The closeout task owns broad gates and documentation drift checks.

## Completion Evidence

- Focused integration evidence: `cargo test -p ash-cli --test phase148_flake_orchestration -- --nocapture` passed 5/5 tests.
- Focused unit evidence: `cargo test -p ash-cli shard -- --nocapture` passed the two shard parser/assignment tests; flake/quarantine filters selected the Phase 148 integration rows.
- Final-surface evidence with explicit candidate binary:
  - `$ASH_UNDER_TEST test fixtures/phase148-flakes --retries 2 --format json` emitted `ash-flake-v1.0` with `flaky: 1` and per-attempt evidence.
  - `$ASH_UNDER_TEST test fixtures/phase148-quarantine --format json` emitted a visible quarantined row remapped to `skip` with original outcome `fail`.
  - `$ASH_UNDER_TEST test fixtures/phase148-quarantine-malformed --format json` failed closed with `malformed quarantine metadata`.
  - `$ASH_UNDER_TEST test fixtures/phase148-shards --shard 1/2 --format json` emitted `ash-shard-v1.0` with two selected rows and two skipped shard candidates.
  - `$ASH_UNDER_TEST test --merge-results /tmp/phase148-shard1.json /tmp/phase148-shard2.json --format json` emitted `ash-merge-v1.0` with four merged rows.
