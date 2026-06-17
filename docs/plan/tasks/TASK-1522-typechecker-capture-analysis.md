# TASK-1522: Typechecker Capture Analysis

## Status: 📝 Planned

## Description

Implement typechecker capture analysis: extract effect level from types, check captures at closure creation, and emit diagnostics. This is the core implementation task.

## Specification Reference

- [SPEC-088: Closure Refinement and Effect-Safe Capture](../../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
- [PLAN-152: Closure Refinement and Tower Documentation](../PLAN-152-CLOSURE-REFINEMENT-AND-TOWER-DOCUMENTATION.md)
- [TASK-1521](TASK-1521-effect-level-type-system-design.md) — Design dependency

## Acceptance Criteria

- [ ] Implement `EffectLevel` enum in ash-core
- [ ] Implement `extract_effect_level` for all types
- [ ] Extend closure type with `capture_effect` field
- [ ] Implement capture analysis at closure creation
- [ ] Emit `CaptureEffectViolation` diagnostic with clear message
- [ ] Handle all edge cases (nested closures, generic types, etc.)

## Verification

- `cargo test -p ash-typeck` passes
- New tests for capture analysis pass
- Negative tests for violations pass
- Property tests for effect-level monotonicity pass
