# TASK-960: Reserved tower callable arrows

## Status: ✅ Complete

## Description

Reserve Act/Proc/Workflow callable and closure arrows with targeted diagnostics rather than silently accepting or misclassifying them.

## Specification Reference

- SPEC-072 §5.5-§5.7
- SPEC-072 §6.3
- SPEC-072 C72-5 through C72-7

## Dependencies

- ✅ TASK-955: Tower callable syntax packet
- ✅ TASK-956: Callable syntax audit gate
- ✅ TASK-957: Pure callable type parser
- ✅ TASK-958: Callable type typechecking and rendering
- ✅ TASK-959: Pure closure arrow syntax

## Requirements

### Functional Requirements

1. Add RED diagnostics tests for `(A) -*> B`, `(A) => B`, `(A) =*> B` in type contexts.
2. Add RED diagnostics tests for `|x| -*> { ... }`, `|x| => { ... }`, and `|x| =*> { ... }` closure contexts.
3. Implement fail-closed reservation diagnostics and smart-constructor distinction docs/tests.

### Reservation Requirements

1. Recognize reserved arrows with maximal munch before generic parse failure in callable-type and closure-literal contexts.
2. Add reserved diagnostics tests for type alias, function parameter, function return, interface/builtin signature if applicable, and closure contexts.
3. Add negative leakage tests proving match-arm `=>` remains legal where currently supported while closure `=>` is reserved for Proc closures.
4. Ensure reserved arrows do not lower to `Type::Fn`, `Type::Fun`, or pure callables returning `Act`/`Proc`/`Workflow`.

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
        Path('crates/ash-parser/tests/task_960_reserved_callable_arrows.rs'): [
            'act_callable_type_arrow_is_reserved',
            'proc_callable_type_arrow_is_reserved',
            'workflow_callable_type_arrow_is_reserved',
            'act_closure_arrow_is_reserved',
            'proc_closure_arrow_is_reserved',
            'workflow_closure_arrow_is_reserved',
            'match_arm_fat_arrow_remains_legal',
        ],
        Path('crates/ash-typeck/tests/task_960_reserved_callable_arrows.rs'): [
            'reserved_type_arrows_do_not_lower_to_type_fn_or_type_fun',
            'reserved_closure_arrows_do_not_typecheck_as_pure_or_effect_closures',
            'smart_constructor_returning_workflow_remains_pure_callable',
        ],
    }
    for path, names in checks.items():
        text = path.read_text()
        missing = [name for name in names if name not in text]
        assert not missing, f'{path} missing tests: {missing}'
    PY
  - cargo test -p ash-parser --test task_960_reserved_callable_arrows -- --nocapture
  - cargo test -p ash-typeck --test task_960_reserved_callable_arrows -- --nocapture
checklist:
  - [x] Focused tests pass.
  - [x] Formatting clean.
  - [x] Clippy clean if Rust touched.
  - [x] CHANGELOG.md updated for implementation or docs changes.
```

## Dependencies for Next Task

This task contributes to PLAN-121 and SPEC-072 completion.

## Notes

Area: parser/typechecker diagnostics. Keep the callable-stratum axis separate from return type. Pure smart constructors returning `Act<T>`, `Proc<T>`, or `Workflow<T>` remain pure callables.

## Completion Notes

- Added parser diagnostics coverage for reserved Act/Proc/Workflow callable arrows in type aliases, function parameters, function returns, builtin signatures, interface signatures, and closure literal contexts.
- Added typechecker boundary coverage proving reserved arrows are rejected before lowering to `Type::Fn`/`Type::Fun` or closure inference, while a pure smart constructor returning `Workflow<Int>` remains `Type::Fn`.
- Implemented fail-closed diagnostics for reserved callable arrows in callable-type-shaped and closure-literal-shaped parse-failure contexts without adding Act/Proc/Workflow callable runtime semantics, partial application, or currying.
- Review remediation added regressions for comment-separated reserved arrows, parenthesized match-arm `=>` false positives, and reserved-looking arrows in strings/comments; the diagnostic scan now skips lexical trivia and only classifies plausible callable-type/closure contexts.
