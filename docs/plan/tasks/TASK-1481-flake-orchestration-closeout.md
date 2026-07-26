# TASK-1481: Close out flake/orchestration phase

## Status: ✅ Complete

## Description

Reconcile status surfaces, changelog/reference docs, and broad verification for Phase 148.

## Specification Reference

- [SPEC-084: Flaky-Test Quarantine and Distributed Orchestration](../../spec/SPEC-084-FLAKY-TEST-QUARANTINE-AND-DISTRIBUTED-ORCHESTRATION.md)
- [PLAN-148: Flaky-Test Quarantine and Distributed Orchestration](../PLAN-148-FLAKY-TEST-QUARANTINE-AND-DISTRIBUTED-ORCHESTRATION.md)

## Dependencies

- TASK-1480: prior Phase 148 task

## Requirements

### Functional Requirements

1. Update PLAN-148, PLAN-INDEX, SPEC-084, CHANGELOG.md, and reference/tools/test.md.
2. Record focused and final-surface evidence.
3. Run focused and broad gates after the final diff.

### Non-Goals

- Remote cluster provisioning.
- Proof-producing synthesis.
- New generator/shrinker semantics.

## TDD Steps

1. Write focused failing Rust/CLI tests for this task's runner behavior.
2. Add or update phase-owned Ash fixtures when user-facing behavior changes.
3. Implement the smallest Rust slice that satisfies those tests.
4. Run focused tests and direct `$ASH_UNDER_TEST` evidence for user-facing behavior.
5. Leave status surfaces planned until closeout unless this task owns a required caveat.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 16
toolsets: [terminal, file]
skills:
  - rust-skills
  - ash-language-feature-spec-writing
  - test-driven-development
  - verification-before-completion
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-cli --test phase148_flake_orchestration -- --nocapture
  - cargo test -p ash-cli flake_orchestration -- --nocapture
  - cargo clippy -p ash-cli --all-targets -- -D warnings
  - cargo check --workspace
checklist:
  - [ ] Focused Rust tests pass with non-zero intended test count
  - [ ] Direct Ash-under-test fixture command passes or fails closed as specified
  - [ ] JSON output remains machine-readable and schema-versioned
  - [ ] Unsupported/malformed cases fail closed
```

## Notes

- Implementation evidence:
  - `cargo test -p ash-cli --test phase148_flake_orchestration -- --nocapture` passed 5/5 tests.
  - Historical fixture-run results below predate TASK-2014 Path B. Current authored source bodies
    fail closed with `no validated production typed lowering is available`; retries classify that
    error as a stable failure, while quarantine preserves it as the original `error` outcome.
    This is runner/orchestration evidence, not source execution.
  - `$ASH_UNDER_TEST test fixtures/phase148-flakes --retries 2 --format json` now emits an
    `ash-flake-v1.0` report with zero flaky rows and closed-admission stable failures.
  - `$ASH_UNDER_TEST test fixtures/phase148-quarantine --format json` emits a visible quarantined
    skip row preserving the original closed-admission error.
  - `$ASH_UNDER_TEST test fixtures/phase148-quarantine-malformed --format json` failed closed with malformed quarantine metadata.
  - `$ASH_UNDER_TEST test fixtures/phase148-shards --shard 1/2 --format json` retains the
    deterministic shard plan but its selected authored rows reject at admission.
  - A source-produced failed shard envelope is rejected by `--merge-results`. The successful
    `ash-merge-v1.0` control merges explicitly synthetic successful JSON shard envelopes with four
    rows; it verifies the merge protocol only and must not be presented as execution of source
    shard fixtures.
- Scope boundary: this phase implements local orchestration primitives only, not remote distributed worker lifecycle/provisioning.
- If implementation discovers a broader prerequisite, split it rather than widening this task silently.
