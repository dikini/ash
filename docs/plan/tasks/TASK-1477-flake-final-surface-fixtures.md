# TASK-1477: Add flaky/quarantine final-surface fixtures

## Status: ✅ Complete

## Description

Add no-Cargo fixtures that exercise retry, flake, and quarantine behavior through an Ash executable.

## Specification Reference

- [SPEC-084: Flaky-Test Quarantine and Distributed Orchestration](../../spec/SPEC-084-FLAKY-TEST-QUARANTINE-AND-DISTRIBUTED-ORCHESTRATION.md)
- [PLAN-148: Flaky-Test Quarantine and Distributed Orchestration](../PLAN-148-FLAKY-TEST-QUARANTINE-AND-DISTRIBUTED-ORCHESTRATION.md)

## Dependencies

- TASK-1476: prior Phase 148 task

## Requirements

### Functional Requirements

1. Create phase-owned fixture files under fixtures/phase148-flakes.
2. Record direct $ASH_UNDER_TEST final-surface commands.
3. Fixtures must not require Cargo as user-facing evidence.

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
  - ${ASH_UNDER_TEST:?set Ash candidate binary} test fixtures/phase148-flakes --retries 2 --format json
checklist:
  - [ ] Focused Rust tests pass with non-zero intended test count
  - [ ] Direct Ash-under-test fixture command passes or fails closed as specified
  - [ ] JSON output remains machine-readable and schema-versioned
  - [ ] Unsupported/malformed cases fail closed
```

## Notes

- Keep user-facing evidence on `$ASH_UNDER_TEST test ...`; Cargo tests are implementer bridge evidence.
- If implementation discovers a broader prerequisite, split it rather than widening this task silently.
