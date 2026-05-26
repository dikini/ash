# TASK-956: Callable syntax audit gate

## Status: 📝 Planned

## Description

Audit live parser/typechecker/renderer/module-summary/closure seams before Rust implementation so downstream tasks use exact files and tests.

## Specification Reference

- SPEC-072 §8
- SPEC-072 §12

## Dependencies

- ✅ TASK-955: Tower callable syntax packet

## Requirements

### Functional Requirements

1. Map current `Fn(...) -> ...`, unary `A -> B`, tuple type, closure, type rendering, and module-summary carrier code paths.
2. Produce `docs/plan/audits/TASK-956-callable-syntax-audit-gate.md` with exact files/functions/tests.
3. Replace downstream placeholder verification commands with exact focused non-zero test commands if current names differ.

### Audit Requirements

1. Record exact live parser seams: `crates/ash-parser/src/surface.rs` (`Type::Tuple`, `Type::Fn`), `crates/ash-parser/src/parse_module.rs` (`parse_surface_type_with_holes`, `parse_surface_type_atom_with_holes`, legacy `Fn` branch, unary `lhs -> rhs` branch, `convert_type_expr`), and `crates/ash-parser/src/parse_type_def.rs` (`parse_type_expr`, `parse_fn_type`, `parse_tuple_type`, synthetic `Constructor { name: "Fn" }` lowering).
2. Record exact typechecker seams: `crates/ash-typeck/src/types.rs` (`Type::Fn`, `Type::Fun`, `Display for Type`, `instantiate_fn_call` or successor), `crates/ash-typeck/src/check_expr.rs` callable application checking, and `crates/ash-typeck/src/lib.rs` signature conversion/registration paths.
3. Locate the live closure/fn-expression parser and lowering/runtime carrier for old `|args| => body`, or state that closure shorthand is not currently implemented and must fail closed.
4. Patch TASK-957 through TASK-960 verification blocks with exact focused non-zero test commands before implementation begins.
5. Add explicit audit rows for tuple-vs-n-ary callable-domain parsing in both parser paths and for current partial-application behavior.

### Non-goals

- Do not implement Act/Proc/Workflow callable runtime semantics unless this task explicitly says so.
- Do not introduce partial application or currying.
- Do not silently reinterpret higher-stratum arrows as pure functions returning computation values.

## Work Steps

1. Inspect the exact live files named by TASK-956 or this task.
2. Write focused RED tests or docs assertions before changing behavior.
3. Implement or document the minimal target behavior.
4. Run focused verification.
5. Update status surfaces and CHANGELOG.md if files beyond tests are changed.
6. Request independent review before marking complete.

## Dispatch

```yaml
agent: codex
reasoning: high
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - false # Replace with TASK-specific focused docs/audit/closeout verification command before marking complete.
checklist:
  - [ ] Required docs/audit artifacts updated.
  - [ ] Status surfaces reconciled.
  - [ ] Independent review completed where required.
```

## Dependencies for Next Task

This task contributes to PLAN-121 and SPEC-072 completion.

## Notes

Area: audit. Keep the callable-stratum axis separate from return type. Pure smart constructors returning `Act<T>`, `Proc<T>`, or `Workflow<T>` remain pure callables.
