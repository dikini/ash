# TASK-1476: Implement quarantine metadata handling

## Status: ✅ Complete

## Description

Parse quarantine metadata, fail closed on malformed quarantine declarations, and keep quarantined rows visible but not ordinary passes.

## Specification Reference

- [SPEC-084: Flaky-Test Quarantine and Distributed Orchestration](../../spec/SPEC-084-FLAKY-TEST-QUARANTINE-AND-DISTRIBUTED-ORCHESTRATION.md)
- [PLAN-148: Flaky-Test Quarantine and Distributed Orchestration](../PLAN-148-FLAKY-TEST-QUARANTINE-AND-DISTRIBUTED-ORCHESTRATION.md)

## Dependencies

- TASK-1475: prior Phase 148 task

## Requirements

### Functional Requirements

1. Recognize test metadata that marks a test quarantined with a reason.
2. Malformed quarantine metadata must produce an explicit error.
3. Quarantined failing/flaky tests must be visible and not counted as ordinary passes.

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
  - cargo test -p ash-cli quarantine -- --nocapture
checklist:
  - [ ] Focused Rust tests pass with non-zero intended test count
  - [ ] Direct Ash-under-test fixture command passes or fails closed as specified
  - [ ] JSON output remains machine-readable and schema-versioned
  - [ ] Unsupported/malformed cases fail closed
```

## Notes

- Keep user-facing evidence on `$ASH_UNDER_TEST test ...`; Cargo tests are implementer bridge evidence.
- If implementation discovers a broader prerequisite, split it rather than widening this task silently.
