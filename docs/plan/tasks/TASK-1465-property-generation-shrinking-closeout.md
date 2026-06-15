# TASK-1465: Close out property generation/shrinking phase

## Status: ✅ Complete

## Description

Update reference docs, PLAN-INDEX, CHANGELOG, and record broad/focused verification evidence.

## Specification Reference

- [SPEC-082: Property Generation and Shrinking Substrate](../../spec/SPEC-082-PROPERTY-GENERATION-AND-SHRINKING-SUBSTRATE.md)
- [PLAN-146: Property Generation and Shrinking Substrate](../PLAN-146-PROPERTY-GENERATION-AND-SHRINKING-SUBSTRATE.md)

## Dependencies

- TASK-1464 complete or explicitly handed off
- Phase implementer must load `rust-skills`, `ash-language-feature-spec-writing`, and `verification-before-completion` before implementation or closeout work.

## Requirements

### Functional Requirements

1. Preserve Phase 145's no-Cargo final-surface rule: user-facing evidence must run through `$ASH_UNDER_TEST test ...`.
2. Fail closed for unsupported or malformed inputs; do not count unsupported behavior as passing.
3. Keep JSON output explicit enough for downstream tools and agents.
4. Add task-specific examples/fixtures rather than relying only on Rust unit tests.

### Examples for Implementers

```ash
law reverse_twice(xs: List<Int>): reverse(reverse(xs)) == xs
proof reverse_twice(xs: List<Int>) {
    by test property
}
```

Expected final-surface evidence example:

```bash
$ASH_UNDER_TEST test fixtures/phase146-property-shrinking --only-synthesized laws --format json --seed 42 --max-cases 50
```

A failing property row should include generated bindings and, after shrinking, a smaller counterexample.

## Implementation Guidance

### Expected files to inspect first

- `crates/ash-cli/src/test_runner/`
- `crates/ash-cli/src/test_runner/synthesized/`
- `crates/ash-parser/src/surface.rs` and `crates/ash-parser/src/parse_module.rs` if syntax changes are required
- `reference/tools/test.md`

### TDD Steps

1. Write focused failing tests for this task's schema/runner behavior.
2. Add or update Ash fixture files under `fixtures/phase146-...` when user-facing behavior changes.
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
  - ${ASH_UNDER_TEST:?set Ash candidate binary} test fixtures/phase146-... --format json
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
