# TASK-1521: Effect-Level Type System Design

## Status: 📝 Planned

## Description

Design the `EffectLevel` enum, closure type extension, and capture analysis algorithm. This is the type-system design task for closure refinement.

## Specification Reference

- [SPEC-088: Closure Refinement and Effect-Safe Capture](../../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
- [PLAN-152: Closure Refinement and Tower Documentation](../PLAN-152-CLOSURE-REFINEMENT-AND-TOWER-DOCUMENTATION.md)

## Acceptance Criteria

- [ ] Design `EffectLevel` enum (Pure, Act, Proc, Workflow) with ordering
- [ ] Design closure type extension with `capture_effect` field
- [ ] Design `extract_effect_level` function for all types
- [ ] Design capture analysis algorithm
- [ ] Design diagnostic messages for violations
- [ ] Produce design document for review

## Verification

- Design document reviewed and approved
- Algorithm handles all types correctly
- Diagnostics are clear and actionable
