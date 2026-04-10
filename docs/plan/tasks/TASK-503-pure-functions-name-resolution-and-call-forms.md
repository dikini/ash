# TASK-503: Pure Functions Name Resolution and Call Forms

## Status: 📝 Planned

## Description

Implement fn name binding/import/export and the call-form resolution rules for unqualified,
qualified, and wrong-target calls under the pure-functions model.

## Specification Reference

- [PLAN-023: Pure Functions Phase](../PLAN-023-PURE-FUNCTIONS-PHASE.md)
- [SPEC-009: Modules](../../spec/SPEC-009-MODULES.md)
- [SPEC-012: Imports](../../spec/SPEC-012-IMPORTS.md)
- [SPEC-027: Pure Functions](../../spec/SPEC-027-PURE-FUNCTIONS.md)

## Requirements

1. Resolve local and imported fn bindings in module scope.
2. Support `use path::fn_name`, `pub fn`, and `pub use`-based fn exports/re-exports.
3. Resolve `module::name(args)` to fn definitions only.
4. Emit clear wrong-target diagnostics when capability-only syntax and fn-call syntax are mixed.

## Dependencies

- [TASK-502](TASK-502-pure-functions-parser-and-ast-foundation.md)

## Likely Files

- Modify: resolver/name-resolution crates and tests
- Modify: import/export handling for function definitions
- Modify: diagnostics tests for wrong-target calls

## Completion Checklist

- [ ] local fn binding works
- [ ] imported fn binding works
- [ ] qualified fn calls work
- [ ] capability targets are rejected in fn-call syntax
- [ ] wrong-target diagnostics are covered by tests
