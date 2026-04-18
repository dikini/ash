# TASK-604: Small-Step IR Compression Prototype

## Status: ✅ Complete

## Description

Prototype the compressed IR and small-step abstract machine described in `DESIGN-027-SMALL-STEP-IR-COMPRESSION.md`. This is an exploratory implementation to validate that the current `Workflow` AST can be decomposed into `Stmt` + `Frame` + `Config`, and that a non-recursive `step` loop can replace the big-step recursive evaluator without breaking observable semantics.

## Requirements

1. Add `Stmt`, `Frame`, and `Config` types to `ash-core` alongside the existing `Workflow` AST.
2. Implement a `step` function in `ash-interp/src/small_step.rs` that drives a `Config` through small-step transitions.
3. Provide a lowering function from current `Workflow` to `Stmt` sequence + initial `Config`.
4. Write unit tests covering at least:
   - `Done` / `Ret`
   - `Let` binding
   - `Seq` via statement sequencing
   - `If` branching
   - `Act` with guard and argument evaluation
5. Ensure `cargo check -p ash-core -p ash-interp` passes with no warnings.

## TDD Steps

1. Write the core types (`Stmt`, `Frame`, `Config`) with `Debug` and `Clone`.
2. Write a test that lowers `Workflow::Ret` into a `Config` and steps it to terminal value.
3. Implement the stepper to make the test pass.
4. Add `Workflow::Let` lowering and stepper test.
5. Add `Workflow::Seq` and `Workflow::If` tests.
6. Add `Workflow::Act` test with a mock capability context.

## Completion Checklist

- [ ] `Stmt`, `Frame`, `Config` types exist in `ash-core`
- [ ] `small_step.rs` module exists in `ash-interp`
- [ ] Lowering from `Workflow` to `Config` is implemented
- [ ] Stepper loop handles terminal, progress, and error outcomes
- [ ] Unit tests demonstrate parity with big-step for covered variants
- [ ] `cargo check` and `cargo clippy` clean for modified crates
- [ ] Prototype findings documented in a short report

## Related Documents

- `docs/design/DESIGN-027-SMALL-STEP-IR-COMPRESSION.md`
- `docs/spec/SPEC-001-IR.md`
- `docs/ideas/minimal-core/MCE-005-SMALL-STEP.md`
- `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md`
