# TASK-1482: Document proof-producing synthesis landscape

## Status: ⏸️ Deferred / To-Spec

## Description

Write a deferred landscape note covering symbolic execution, SMT/solver proof, proof terms, replay/checking, and trust boundaries.

## Specification Reference

- [SPEC-085: Proof-Producing Synthesis Todo Spec](../../spec/SPEC-085-PROOF-PRODUCING-SYNTHESIS-TODO-SPEC.md)
- [PLAN-149: Proof-Producing Synthesis Todo Spec](../PLAN-149-PROOF-PRODUCING-SYNTHESIS-TODO-SPEC.md)

## Dependencies

- PLAN-145 complete
- Phase implementer must load `rust-skills`, `ash-language-feature-spec-writing`, and `verification-before-completion` before implementation or closeout work.

## Requirements

### Functional Requirements

1. Keep this task documentation-only; do not implement symbolic execution, solver calls, proof checking, or new parser syntax.
2. Clearly classify proof-producing synthesis as future non-test evidence rather than empirical `by test` evidence.
3. Record open trust-boundary questions and to-spec criteria for a later implementation-grade packet.
4. Keep syntax examples illustrative unless a later spec promotes them to normative syntax.

### Examples for Implementers

Future syntax is intentionally illustrative only:

```ash
proof associativity(...) {
    by solver z3
}

proof safety(...) {
    by symbolic { produce proof_artifact }
}
```

This phase must not implement these forms; it records the future evidence-family boundary.

## Implementation Guidance

### Expected files to inspect first

- `crates/ash-cli/src/test_runner/`
- `crates/ash-cli/src/test_runner/synthesized/`
- `crates/ash-parser/src/surface.rs` and `crates/ash-parser/src/parse_module.rs` if syntax changes are required
- `reference/tools/test.md`

### Documentation Steps

1. Inspect Phase 145 evidence-family wording and future-proof any references to symbolic/solver proof.
2. Write the deferred note/spec/task updates named by this task.
3. Verify no examples are phrased as currently supported Ash syntax.
4. Run scoped Markdown link/trailing-whitespace checks.
5. Update PLAN-INDEX/CHANGELOG only in the deferred closeout task unless this task owns that surface.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 10
toolsets: [terminal, file]
skills:
  - ash-language-feature-spec-writing
  - software-planning
  - verification-before-completion
```

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 -c "from pathlib import Path; assert Path('docs/spec/SPEC-085-PROOF-PRODUCING-SYNTHESIS-TODO-SPEC.md').exists(); assert Path('docs/plan/PLAN-149-PROOF-PRODUCING-SYNTHESIS-TODO-SPEC.md').exists()"
checklist:
  - [ ] Deferred status is explicit in spec/plan/task surfaces
  - [ ] No implementation syntax is claimed as currently supported
  - [ ] Future to-spec criteria are clear
  - [ ] CHANGELOG/PLAN-INDEX updates are present in closeout
```

## Notes

- Keep the task small; if implementation discovers a broader prerequisite, stop and create a prerequisite task instead of widening scope.
- Do not use ordinary installed `ash` as the only proof while tooling is under development; record `$ASH_UNDER_TEST` provenance.
- Rust tooling is required for implementers, but not accepted as the author/executor-facing proof path.
