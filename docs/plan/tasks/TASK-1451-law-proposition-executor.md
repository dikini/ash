# TASK-1451: Execute supported law propositions over explicit bindings

## Status: ✅ Complete

## Description

Execute supported law propositions over explicit bindings. This task is part of Phase 145 / PLAN-145 and implements a small, reviewable slice of [SPEC-081](../../spec/SPEC-081-LAW-TEST-EVIDENCE-SUBSTRATE.md). The task must preserve the core phase constraint: final user-facing law/test/proof behavior is runnable with an Ash-under-test executable (`$ASH_UNDER_TEST test ...`) and does not require the law/test/proof author or executor to use Cargo or Rust tooling. Do not assume the globally installed `ash` is current while Ash tooling is under development.

## Specification Reference

- [SPEC-081: Law Test Evidence Substrate](../../spec/SPEC-081-LAW-TEST-EVIDENCE-SUBSTRATE.md)
- [PLAN-145: Law Test Evidence Substrate](../PLAN-145-LAW-TEST-EVIDENCE-SUBSTRATE.md)

## Dependencies

- TASK-1447

## Estimated Effort

6 hours

## Requirements

### Functional Requirements

1. Define the supported executable law proposition subset.
2. Evaluate simple law propositions over explicit JSON/Value bindings.
3. Defer unsupported expression forms with a precise reason.
4. Record binding snapshots in repro artifacts.

### No-Rust User-Facing Requirement

Every user-facing behavior changed by this task must be demonstrated with a direct `$ASH_UNDER_TEST test ...` command. `$ASH_UNDER_TEST` may be a candidate binary produced or selected by the implementer, but its path, version/provenance, and release/install-parity handoff must be recorded. Do not count `cargo run -p ash-cli -- test ...` as final-surface evidence, and do not assume the globally installed `ash` is current unless TASK-1446 proves it.

### Honest Failure Requirements

- Missing tests, missing generators, missing domains, unsupported propositions, duplicate names, and skipped evidence must not satisfy a proof.
- Unsupported cases should report `invalid_evidence`, `deferred`, or a precise error rather than a misleading pass.
- Authored and synthesized rows must remain labeled distinctly in JSON output.

## TDD / Implementation Steps

### Step 1: RED — focused regression or fixture

Create a focused regression test or `.ash` fixture that demonstrates the current gap for this task. The fixture should initially fail, defer, or expose string-only/metadata-only behavior.

### Step 2: GREEN — minimal implementation

Implement only the slice described in this task. Prefer narrow data models, explicit diagnostics, and fail-closed behavior over broad inferred magic.

### Step 3: Final-surface Ash smoke

Run at least one direct Ash CLI command, for example:

```bash
${ASH_UNDER_TEST:?set Ash candidate binary} test <fixture>.ash --include-synthesized laws --format json
${ASH_UNDER_TEST:?set Ash candidate binary} test <fixture>.ash --only-synthesized laws --format json
${ASH_UNDER_TEST:?set Ash candidate binary} test <fixture-tests-dir> --format json
```

The exact fixture paths should be created or selected by the implementer and recorded in the task closeout notes.

### Step 4: Integration health

Run focused Rust implementation health gates after the Ash smoke passes.

## Dispatch

```yaml
agent: codex
reasoning: medium
max_turns: 12
toolsets: [terminal, file, coding]
```

Implementation guidance for the agent:

- Read `crates/ash-parser/src/surface.rs`, `crates/ash-parser/src/parse_module.rs`, `crates/ash-cli/src/test_runner/synthesized/schema.rs`, `crates/ash-cli/src/test_runner/synthesized.rs`, and relevant runner modules before editing.
- Use MCP/LSP for Rust symbol tracing where possible.
- Keep this task atomic; do not implement later Phase 145 tasks opportunistically.
- Update `CHANGELOG.md` under `[Unreleased]` when the task changes code, docs policy, CLI behavior, or reference behavior.

## Verification

```yaml
strictness: clean
commands:
  - ${ASH_UNDER_TEST:?set Ash candidate binary} test <task-specific-fixture>.ash --include-synthesized laws --format json
  - ${ASH_UNDER_TEST:?set Ash candidate binary} test <task-specific-fixture-or-dir> --format json
  - cargo test -p ash-cli <task_specific_filter> -- --nocapture
  - cargo clippy -p ash-cli --all-targets -- -D warnings
  - cargo fmt --check
  - git diff --check
checklist:
  - [ ] Direct `$ASH_UNDER_TEST test` final-surface command passed without Cargo.
  - [ ] Focused Rust tests passed.
  - [ ] Unsupported cases fail closed or defer explicitly.
  - [ ] JSON output labels evidence mode and status where this task owns output.
  - [ ] CHANGELOG.md updated if code/tooling/docs-policy changed.
```

## Orchestrator Notes

This task is the shared engine for property and small-world modes. Keep the subset narrow and honest.

Do not mark this task complete from Rust unit tests alone. The orchestrator must inspect the direct `ash test` output and verify that the law side and test side are both represented when this task touches proof/test linkage.
