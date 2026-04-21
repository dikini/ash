# TASK-603: CI Gate Integration

## Status: ✅ Complete

## Description

Wire the spec processor into `cargo test` and the CI workflow so that a failing processor blocks the pipeline.

## Specification Reference

- PLAN-090-SPEC-PROCESSOR.md — Track C
- AGENTS.md (manual CI policy)

## Dependencies

- All previous Phase 90 tasks

## Requirements

1. Create Rust integration test that runs the processor workflow.
2. Assert exit code is zero.
3. Document CI gate in relevant workflow/policy doc.

## TDD Steps

### Step 1: Write failing test

Create `tests/spec_processor_integration.rs` that runs `apps/spec_processor/src/main.ash`.

### Step 2: Implement

Use `ash-engine` API to execute the processor workflow in the integration test.

### Step 3: Verify

Run test against real repo. Review report for unexpected findings.

## Verification Steps

- [ ] `cargo test --test spec_processor_integration` passes
- [ ] Processor blocks CI on real findings
- [ ] CHANGELOG.md entry added for Phase 90
- [ ] Codex phase audit: VERIFIED
