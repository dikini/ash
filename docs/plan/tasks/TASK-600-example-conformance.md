# TASK-600: Example Syntax Conformance Integration

## Status: ✅ Complete

## Description

Integrate `std::process` to run `ash check` on every `.ash` example file and aggregate parse/type errors.

## Specification Reference

- PLAN-090-SPEC-PROCESSOR.md — Track C
- DESIGN-NOTE-BATCH-CHECK-API.md

## Dependencies

- TASK-590 (file collector)
- TASK-598 (`std::process`)

## Requirements

1. Iterate over `example_files` from `FileTree`.
2. Spawn `ash check <file>` via `process::run`.
3. On non-zero exit, emit `ExampleFailure` finding with stderr.
4. Aggregate findings into the processor report.

## TDD Steps

### Step 1: Write failing test

Mock example directory with an invalid `.ash` file. Assert `ExampleFailure` finding.

### Step 2: Implement

Create `apps/spec_processor/src/examples.ash` with `check_examples(paths: List<String>) -> List<SpecFinding>`.

### Step 3: Verify

Run against real `examples/` directory. Every file should pass or produce a valid finding.

## Verification Steps

- [ ] Mock test passes
- [ ] Real repo run complete
- [ ] No deadlock on large stdout/stderr
- [ ] Codex verification: VERIFIED
