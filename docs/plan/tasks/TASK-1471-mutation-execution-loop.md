# TASK-1471: Implement mutation execution loop

## Status: 📝 Planned

## Description

Run bounded mutants through selected Ash tests/properties, classify killed/survived/equivalent/deferred, and enforce limits/timeouts.

## Specification Reference

- [SPEC-083: Law Coverage and Mutation Testing](../../spec/SPEC-083-LAW-COVERAGE-AND-MUTATION-TESTING.md)
- [PLAN-147: Law Coverage and Mutation Testing](../PLAN-147-LAW-COVERAGE-AND-MUTATION-TESTING.md)

## Dependencies

- TASK-1470 complete or explicitly handed off
- Phase implementer must load `rust-skills`, `ash-language-feature-spec-writing`, and `verification-before-completion` before implementation or closeout work.

## Requirements

### Functional Requirements

1. Preserve Phase 145's no-Cargo final-surface rule: user-facing evidence must run through `$ASH_UNDER_TEST test ...`.
2. Fail closed for unsupported or malformed inputs; do not count unsupported behavior as passing.
3. Keep JSON output explicit enough for downstream tools and agents.
4. Add task-specific examples/fixtures rather than relying only on Rust unit tests.

### Examples for Implementers

```bash
$ASH_UNDER_TEST test fixtures/phase147-coverage --coverage --format json
$ASH_UNDER_TEST test fixtures/phase147-mutation --mutation --mutation-limit 20 --format json
```

Coverage should report uncovered law/proof declarations. Mutation output should distinguish killed, survived, equivalent/deferred, and errored mutants.

## Implementation Guidance

### Expected files to inspect first

- `crates/ash-cli/src/test_runner/`
- `crates/ash-cli/src/test_runner/synthesized/`
- `crates/ash-parser/src/surface.rs` and `crates/ash-parser/src/parse_module.rs` if syntax changes are required
- `reference/tools/test.md`

### TDD Steps

1. Write focused failing tests for this task's schema/runner behavior.
2. Add or update Ash fixture files under `fixtures/phase147-...` when user-facing behavior changes.
3. Implement the smallest Rust slice that satisfies the focused tests.
4. Run the direct Ash-under-test command and record its output in the task closeout notes.
5. Update docs/status surfaces only in the closeout task unless this task owns a public behavior caveat.

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
  - false # replace with exact focused cargo/Ash verification before marking complete
  - ${ASH_UNDER_TEST:?set Ash candidate binary} test fixtures/phase147-... --format json
checklist:
  - [ ] Focused Rust tests pass
  - [ ] Direct Ash-under-test fixture command passes or defers honestly
  - [ ] Unsupported cases fail closed
  - [ ] CHANGELOG/reference updates are present if public behavior changed
```

## Notes

- Keep the task small; if implementation discovers a broader prerequisite, stop and create a prerequisite task instead of widening scope.
- Do not use ordinary installed `ash` as the only proof while tooling is under development; record `$ASH_UNDER_TEST` provenance.
- Rust tooling is required for implementers, but not accepted as the author/executor-facing proof path.
