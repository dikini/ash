# TASK-1520: Closure Refinement Audit and Capture Channels

## Status: 📝 Planned

## Description

Audit all current closure creation points in the Ash codebase, identify all capture channels, and document effect leakage scenarios. This is the foundation for the capture-based effect rule.

## Specification Reference

- [SPEC-088: Closure Refinement and Effect-Safe Capture](../../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
- [PLAN-152: Closure Refinement and Tower Documentation](../PLAN-152-CLOSURE-REFINEMENT-AND-TOWER-DOCUMENTATION.md)

## Acceptance Criteria

- [ ] List all closure creation points in parser, typechecker, and runtime
- [ ] Identify all capture channels (lexical scope, environment frame, parameters)
- [ ] Document effect leakage scenarios with concrete examples
- [ ] Classify each scenario as: blocked by current rule, allowed by new rule, or requires further analysis
- [ ] Produce audit report for review

## Verification

- Audit report reviewed and approved
- All scenarios classified correctly
- No missing capture channels identified
