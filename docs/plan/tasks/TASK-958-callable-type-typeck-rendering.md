# TASK-958: Callable type typecheck and rendering

## Status: ✅ Complete

## Description

Wire preferred pure callable syntax through typechecking, rendering, diagnostics, and module import/export surfaces without losing arity or return type data.

## Specification Reference

- SPEC-072 §7
- SPEC-072 §9
- SPEC-072 C72-1 through C72-3

## Dependencies

- ✅ TASK-955: Tower callable syntax packet
- ✅ TASK-956: Callable syntax audit gate
- ✅ TASK-957: Pure callable type parser

## Requirements

### Functional Requirements

1. Add RED typechecker/rendering tests for preferred pure callable syntax.
2. Verify module summary import/export preserves function signatures.
3. Update renderers to prefer `(A, B) -> C` while accepting legacy syntax.

### Typechecker/Rendering Requirements

1. Update `Display for Type` or successor renderers to prefer `(A, B) -> C` for `Type::Fn`.
2. Audit and update exact-arity behavior in `instantiate_fn_call` or successor call-checking helpers. SPEC-072 requires `f(1)` to fail for `f : (Int, Int) -> Int`.
3. Verify module export/import signature transport preserves callable arity and return type.
4. Add focused tests for unary, n-ary, nested-return callable rendering, exact arity, too few arguments, and too many arguments.

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
reasoning: medium
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - |
    python3 - <<'PY'
    from pathlib import Path
    checks = {
        Path('crates/ash-typeck/tests/task_958_callable_type_rendering.rs'): [
            'type_display_prefers_parenthesized_callable_domain',
            'nested_return_callable_renders_right_associative',
            'callable_application_requires_exact_arity',
            'too_few_arguments_are_not_partial_application',
            'too_many_arguments_report_exact_arity',
        ],
        Path('crates/ash-engine/tests/task_958_callable_module_summary.rs'): [
            'imported_pub_fn_signature_preserves_n_ary_callable_parameter',
            'imported_builtin_signature_preserves_preferred_callable_syntax',
            'workflow_returning_smart_constructor_remains_pure_callable',
        ],
    }
    for path, names in checks.items():
        text = path.read_text()
        missing = [name for name in names if name not in text]
        assert not missing, f'{path} missing tests: {missing}'
    PY
  - cargo test -p ash-typeck --test task_958_callable_type_rendering -- --nocapture
  - cargo test -p ash-engine --test task_958_callable_module_summary -- --nocapture
checklist:
  - [x] Focused tests pass.
  - [x] Formatting clean.
  - [x] Clippy clean if Rust touched.
  - [x] CHANGELOG.md updated for implementation or docs changes.
```

## Dependencies for Next Task

This task contributes to PLAN-121 and SPEC-072 completion.

## Notes

Area: typechecker. Keep the callable-stratum axis separate from return type. Pure smart constructors returning `Act<T>`, `Proc<T>`, or `Workflow<T>` remain pure callables.

Completion notes:

- RED evidence: after adding `crates/ash-typeck/tests/task_958_callable_type_rendering.rs`, `cargo test -p ash-typeck --test task_958_callable_type_rendering -- --nocapture` failed with 4 focused failures: `Type::Fn` rendered as legacy `Fn(...) -> ...`, too few arguments returned a partial callable type, and too many arguments reported `expected at most`. After adding `crates/ash-engine/tests/task_958_callable_module_summary.rs`, `cargo test -p ash-engine --test task_958_callable_module_summary -- --nocapture` failed with 3 focused failures where imported callable signatures rendered as legacy `Fn(...) -> ...`.
- GREEN evidence: the focused TASK-958 typechecker target passes 5 tests and the focused engine module-summary target passes 3 tests after updating pure callable rendering and exact-arity checked call behavior.
- Implementation scope: legacy `Fn(...) -> ...` parsing remains accepted from TASK-957; TASK-958 did not implement closure syntax or reserve higher-stratum arrows.
