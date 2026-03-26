# TASK-299: Type Checker - Bind Workflow Parameters from Input

## Status: 📝 Planned

## Description

Enable the type checker to recognize workflow parameters as bound variables when executing with input bindings. Currently, workflows with parameters fail with `UnboundVariable` errors even when input values are provided via CLI `--input`.

## Current Broken State

```rust
// Workflow definition
workflow greet(name: String) {
    ret "Hello, " + name;  // ERROR: UnboundVariable("name")
}

// CLI invocation
ash run greet.ash --input '{"name": "World"}'
// Type error: Name resolution error: [UnboundVariable("name")]
```

The `execute_with_input()` path exists in the engine and correctly parses input JSON, but the type checker (`type_check_workflow`) runs on the surface workflow body without knowledge of the parameters.

## Specification Reference

- SPEC-005: CLI Specification (section on `--input` parameter binding)
- SPEC-003: Type System Specification

## Dependencies

- ✅ TASK-292: Make CLI --input functional (tests created, this is the implementation)

## Requirements

1. **Parameter Injection**: Before type checking a workflow body, inject parameter names into the type checker's name resolution scope
2. **Type Annotation**: Parameters have explicit type annotations that must be used for type checking
3. **Binding Semantics**: Parameters should be treated as `Let` bindings for linear obligation tracking
4. **Backwards Compatibility**: Workflows without parameters must continue to work unchanged

## TDD Steps

1. **Failing Test**: Create test showing `type_check_workflow` fails with `UnboundVariable` for parameters
2. **Parameter Passing**: Modify `type_check_workflow()` to accept optional parameter bindings
3. **Environment Setup**: Inject parameters into initial type environment before checking body
4. **Verify Tests**: Run all ash-typeck tests plus CLI integration tests

## Test Cases

- Single parameter workflow with input binding
- Multiple parameter workflow with partial input (should error)
- Multiple parameter workflow with complete input
- Parameter shadowing existing names
- Parameter type mismatch detection

## Files Likely to Change

- `crates/ash-typeck/src/lib.rs` - `type_check_workflow()` signature and implementation
- `crates/ash-typeck/src/check_expr.rs` - Environment handling
- `crates/ash-engine/src/lib.rs` - Bridge between engine and type checker

## Completion Checklist

- [ ] Type checker accepts parameter bindings for workflow type checking
- [ ] Parameters are properly typed in the environment
- [ ] Unbound parameter errors occur only when not provided
- [ ] All ash-typeck tests pass
- [ ] CLI input tests can be unignored and pass
- [ ] `cargo clippy --all-targets` clean
- [ ] CHANGELOG.md updated
- [ ] PLAN-INDEX.md updated
