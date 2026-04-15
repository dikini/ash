# TASK-560: Track FnDef Type Annotation Handling

**Phase:** 80 (follow-up)
**Spec:** SPEC-031 §5.1
**Related:** TASK-558, TASK-553
**Estimate:** 4 hours
**Status:** ✅ Complete

## Description

Phase 80 code review noted that parameter type annotations on `Expr::FnDef` are handled via `annotation_name_to_type()` which maps known builtin type names to their `Type` variants, but falls back to `Type::Var(TypeVar::fresh())` for any unknown name. This means user-defined type names in annotations are silently ignored.

## What was done

Replaced `annotation_name_to_type(name: &str) -> Type` with `annotation_to_type(name: &str, env: &TypeEnv, span: Span, context: &str) -> Result<Type, ConstructorError>`:

1. **Added `ConstructorError::UnknownTypeAnnotation`** variant in `error.rs` with `name`, `context`, and `span` fields.
2. **New resolver consults `TypeEnv::resolve_type`** — primitives map directly, user-defined types registered in the TypeEnv resolve to `Type::Constructor { name, args: [], kind: Kind::Type }`, and unknown names produce a type error.
3. **Error recovery** — when an annotation fails to resolve, the error is accumulated and a fresh type variable is used as fallback so type checking can continue and report more errors.
4. **Both param and return type annotations** go through the same resolution path.

## Tests

- `task560_unknown_param_annotation_produces_error` — `fn(x: BogusType) { x }` produces `UnknownTypeAnnotation` error
- `task560_unknown_return_annotation_produces_error` — `fn(x) -> BogusRet { x }` produces `UnknownTypeAnnotation` error
- `task560_user_defined_type_annotation_resolves` — a type registered in TypeEnv resolves to `Type::Constructor` in the function signature

## Acceptance Criteria

- [x] `fn(x: KnownType) { ... }` correctly constrains `x` to the resolved type
- [x] `fn(x: UnknownType) { ... }` produces a type error (not silent fallback)
- [x] Return type annotations resolved through the same path
- [x] Tests: custom type annotation constrains, unknown annotation errors
- [x] `cargo test --all` passes
- [x] `cargo clippy --all` clean
