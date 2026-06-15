# TASK-1480: Implement distributed result merge

## Status: ✅ Complete

## Description

Merge shard JSON outputs deterministically and reject duplicate or missing shards before aggregating success.

## Specification Reference

- [SPEC-084: Flaky-Test Quarantine and Distributed Orchestration](../../spec/SPEC-084-FLAKY-TEST-QUARANTINE-AND-DISTRIBUTED-ORCHESTRATION.md)
- [PLAN-148: Flaky-Test Quarantine and Distributed Orchestration](../PLAN-148-FLAKY-TEST-QUARANTINE-AND-DISTRIBUTED-ORCHESTRATION.md)

## Dependencies

- TASK-1479: prior Phase 148 task

## Requirements

### Functional Requirements

1. --merge-results reads shard JSON files without re-running tests.
2. Duplicate shard IDs fail closed.
3. Missing shard IDs fail closed.
4. Merged output preserves test rows and evidence artifacts deterministically.

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
  - cargo test -p ash-cli --test phase148_flake_orchestration -- --nocapture
  - ${ASH_UNDER_TEST:?set Ash candidate binary} test --merge-results /tmp/phase148-shard-*.json --format json
checklist:
  - [ ] Focused Rust tests pass with non-zero intended test count
  - [ ] Direct Ash-under-test fixture command passes or fails closed as specified
  - [ ] JSON output remains machine-readable and schema-versioned
  - [ ] Unsupported/malformed cases fail closed
```

## Notes

- Keep user-facing evidence on `$ASH_UNDER_TEST test ...`; Cargo tests are implementer bridge evidence.
- If implementation discovers a broader prerequisite, split it rather than widening this task silently.
