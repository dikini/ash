# TASK-507: Pure Functions Stdlib and Conformance Tests

## Status: ✅ Passed

## Description

Rewrite the pure stdlib surface around `fn` and add the conformance/failure-mode coverage needed to
prove the pure-functions phase contract end-to-end.

## Specification Reference

- [PLAN-023: Pure Functions Phase](../PLAN-023-PURE-FUNCTIONS-PHASE.md)
- [DESIGN-020: Pure Functions and the Three-Vertex Model](../../design/DESIGN-020-PURE-FUNCTIONS-THREE-VERTEX-MODEL.md)
- [SPEC-027: Pure Functions](../../spec/SPEC-027-PURE-FUNCTIONS.md)
- [SPEC-028: Function Constraint System](../../spec/SPEC-028-FUNCTION-CONSTRAINT-SYSTEM.md)

## Requirements

1. Rewrite pure stdlib modules such as `option`, `result`, and pure `io::path` helpers to use `fn`.
2. Replace string-matching parser checks with real parser/semantic validation.
3. Add conformance tests for imported fn calls, qualified fn success/failure, `Type::Null` one-armed
   `if`, recursion/panic boundary, undefined fn, wrong-target call diagnostics, and workflow
   precondition-call-site proofs.

## Dependencies

- [TASK-503](TASK-503-pure-functions-name-resolution-and-call-forms.md)
- [TASK-504](TASK-504-pure-functions-type-system-and-purity.md)
- [TASK-505](TASK-505-pure-functions-contract-lowering-and-stage1-constraints.md)
- [TASK-506](TASK-506-pure-functions-runtime-and-workflow-integration.md)

## Completion Checklist

- [x] pure stdlib modules rewritten to `fn`
- [x] parser/semantic tests updated to real conformance coverage
- [x] failure-mode tests added and passing
- [x] imported/qualified/wrong-target call coverage present
