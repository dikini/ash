# TASK-504: Pure Functions Type System and Purity

## Status: ✅ Passed

## Description

Add the type-system support for pure functions, including `Type::Fn`, generic instantiation,
purity checking, omitted-else `if` / `Type::Null`, and fn/call typing.

## Specification Reference

- [PLAN-023: Pure Functions Phase](../PLAN-023-PURE-FUNCTIONS-PHASE.md)
- [SPEC-003: Type System](../../spec/SPEC-003-TYPE-SYSTEM.md)
- [SPEC-027: Pure Functions](../../spec/SPEC-027-PURE-FUNCTIONS.md)

## Requirements

1. Introduce `Type::Fn(Vec<Type>, Box<Type>)` distinct from `Type::Fun(..., Effect)`.
2. Support generic fn instantiation at call sites.
3. Enforce purity rules for fn bodies, including `Expr::Call` and `Expr::InterfaceMethodCall`.
4. Enforce one-armed `if` typing/evaluation expectations around `Type::Null`.

## Dependencies

- [TASK-502](TASK-502-pure-functions-parser-and-ast-foundation.md)
- [TASK-503](TASK-503-pure-functions-name-resolution-and-call-forms.md)

## Likely Files

- Modify: `crates/ash-typeck/` type representation and inference code
- Modify: purity-checking logic
- Modify: tests for omitted-else `if`, fn typing, and generic instantiation

## Completion Checklist

- [x] `Type::Fn` introduced and distinguished from `Type::Fun`
- [x] generic fn call typing works
- [x] one-armed `if` / `Type::Null` checks enforced
- [x] purity checking covers ordinary and interface method calls
