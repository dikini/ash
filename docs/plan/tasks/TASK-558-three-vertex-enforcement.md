# TASK-558: Three-Vertex Enforcement via Type::Fn vs Type::Fun

**Phase:** 80
**Spec:** SPEC-031 §4.8, §6.3
**Depends on:** TASK-553, TASK-556
**Estimate:** 4 hours

## Description

Enforce the three-vertex boundary: closures defined in workflow context get `Type::Fun(params, ret, effect)` and are rejected where `Type::Fn(params, ret)` is expected.

## Requirements

### 1. Context-Aware FnDef Typing

In the type checker, when type-checking `Expr::FnDef`:

- If in pure `fn` context: type as `Type::Fn(param_types, ret_type)`
- If in `workflow` context: type as `Type::Fun(param_types, ret_type, effect)` where `effect` is derived from the workflow's effect level or the closure body's effects

### 2. Fn/Fun Unification Rejection

The type checker must reject passing `Type::Fun` where `Type::Fn` is expected:

- `Type::Fn(params, ret)` != `Type::Fun(params, ret, effect)` in unification
- This prevents workflow-captured closures from leaking into pure fn parameters

### 3. Escape Cases

Per SPEC-031 §4.8, enforce the five prohibited escape cases:

| Case | Enforcement |
|------|------------|
| Return Fun from workflow | typecheck: return type must not contain Fun |
| Store Fun in instance state | typecheck: state field typed Fn rejects Fun |
| Pass Fun to Fn parameter | typecheck: unification failure |
| Fun through container into pure context | typecheck: container element type propagation |
| Serialize Closure across boundary | runtime: serialization error (already in TASK-551) |

### 4. BoundaryViolation Error

Add `EvalError::BoundaryViolation` as a distinct variant (per SPEC-031 §4.8 implementation note). Use when a runtime check catches a boundary crossing that static typing missed.

## TDD Steps

1. Test: `fn(x) { x + 1 }` in pure context types as `Type::Fn([Int], Int)`
2. Test: `fn(x) { act ...; x }` in workflow context types as `Type::Fun([Int], Int, Operational)`
3. Test: passing `Type::Fun` to a `Type::Fn` parameter -> type error
4. Test: returning `Type::Fun` from workflow -> type error
5. Test: `List<Type::Fun>` passed where `List<Type::Fn>` expected -> type error
6. Verify `cargo test --all` passes

## Completion Checklist

- [ ] FnDef typed as Fn in pure context, Fun in workflow context
- [ ] Fn/Fun unification rejection
- [ ] All five escape cases enforced
- [ ] `BoundaryViolation` error variant
- [ ] Type checking tests
- [ ] `cargo test --all` passes
- [ ] `cargo clippy` clean
- [ ] CHANGELOG.md updated
