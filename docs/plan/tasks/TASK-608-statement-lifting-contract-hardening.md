# TASK-608: Statement-Lifting Contract Hardening

## Status: ✅ Complete

## Description

Turn the current lifting pass into a production-honest subsystem by formalizing the conservative contract: lift where the workflow structure can honestly host synthetic bindings, and preserve the original expression elsewhere so downstream diagnostics remain user-facing and non-panicking.

## Specification Reference

- `docs/design/DESIGN-028-STATEMENT-LIFTING.md`
- `docs/spec/SPEC-002-SURFACE.md`

## Dependencies

- ✅ TASK-605: Statement Lifting and Pipe Operator Prototype

## Requirements

1. No user program may crash due to lifting assertions/panics.
2. Unsupported workflow positions must preserve original expressions instead of introducing hidden semantic distortion.
3. The supported-vs-preserved contract must be documented and regression-tested.
4. Regression tests must cover at least `Ret`, `If`, `ForEach`, `Set`, `Send`, `Spawn`, `Split`, and workflow-call arguments.

## TDD Steps

1. Add regression tests reproducing each formerly panic-prone workflow position.
2. Replace panic/assert behavior with explicit preserve-original behavior where bindings would otherwise be required.
3. Add CLI/end-to-end tests showing user-facing diagnostics are preserved.
4. Re-run parser/lowering and workspace tests.

## Verification Steps

- [ ] `cargo test -p ash-parser lift -- --nocapture`
- [ ] `cargo test -p ash-cli --test cli -- --nocapture`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`

## Notes

This task is about diagnostic honesty and contract stability, not broadening lifting semantics yet.