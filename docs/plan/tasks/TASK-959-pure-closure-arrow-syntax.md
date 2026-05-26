# TASK-959: Pure closure arrow syntax

## Status: ✅ Complete

## Description

Implement pure closure shorthand `|args| -> body` and stop treating `|args| => body` as preferred pure syntax.

## Specification Reference

- SPEC-072 §6
- SPEC-072 C72-4

## Dependencies

- ✅ TASK-955: Tower callable syntax packet
- ✅ TASK-956: Callable syntax audit gate
- ✅ TASK-957: Pure callable type parser

## Requirements

### Functional Requirements

1. Add RED parser/typechecker/interpreter tests for `|x| -> x + 1` and `|x, y| -> x + y` in supported contexts.
2. Add migration or rejection test for old `|x| => x + 1` pure syntax.
3. Implement closure parsing/lowering updates without changing closure capture semantics beyond the spelling.

### Closure-Specific Requirements

1. Implement `|args| -> body` only in closure-literal context.
2. Do not reuse match-arm `=>` parsing for closure arrows.
3. Ensure `|args| -> body` remains Pure-stratum even in higher tower contexts; captures/body must satisfy pure-closure rules.
4. Add focused migration/rejection coverage for old `|args| => body` as pure syntax.

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
        Path('crates/ash-parser/tests/task_959_pure_closure_arrow.rs'): [
            'pure_closure_arrow_single_param_parses_as_fn_def',
            'pure_closure_arrow_two_params_parses_as_fn_def',
            'old_fat_arrow_closure_is_not_silent_pure_shorthand',
            'closure_arrow_does_not_steal_match_arm_fat_arrow',
        ],
        Path('crates/ash-typeck/tests/task_959_pure_closure_arrow.rs'): [
            'pure_closure_arrow_typechecks_as_type_fn_in_pure_context',
            'pure_closure_arrow_in_workflow_context_keeps_existing_boundary',
        ],
        Path('crates/ash-interp/tests/task_959_pure_closure_arrow.rs'): [
            'pure_closure_arrow_executes_existing_closure_runtime_path',
        ],
    }
    for path, names in checks.items():
        text = path.read_text()
        missing = [name for name in names if name not in text]
        assert not missing, f'{path} missing tests: {missing}'
    PY
  - cargo test -p ash-parser --test task_959_pure_closure_arrow -- --nocapture
  - cargo test -p ash-typeck --test task_959_pure_closure_arrow -- --nocapture
  - cargo test -p ash-interp --test task_959_pure_closure_arrow -- --nocapture
checklist:
  - [x] Focused tests pass.
  - [x] Formatting clean.
  - [x] Clippy clean if Rust touched.
  - [x] CHANGELOG.md updated for implementation or docs changes.
```

## Dependencies for Next Task

This task contributes to PLAN-121 and SPEC-072 completion.

## Notes

Area: parser/typechecker/runtime. Keep the callable-stratum axis separate from return type. Pure smart constructors returning `Act<T>`, `Proc<T>`, or `Workflow<T>` remain pure callables.

Completion notes:

- RED evidence: after adding `crates/ash-parser/tests/task_959_pure_closure_arrow.rs`, `cargo test -p ash-parser --test task_959_pure_closure_arrow -- --nocapture` failed with 3 focused failures: `|x| -> x + 1` and `|x, y| -> x + y` did not parse, while old `|x| => x + 1` still parsed as a pure `FnDef`. The typechecker and interpreter targets failed at parse time for the same missing `->` closure shorthand.
- GREEN evidence: the focused TASK-959 parser target passes 4 tests, the typechecker target passes 2 tests, and the interpreter target passes 1 test after switching the closure-literal parser separator from `=>` to `->` and keeping `Expr::FnDef` at the Pure `Type::Fn` stratum even in workflow contexts.
- Implementation scope: the parser still desugars pure closure shorthand to the existing surface `Expr::FnDef`, then the existing lowering/typechecker/runtime paths handle it. TASK-959 did not implement Act/Proc/Workflow callable semantics or reserve the higher-stratum closure arrows owned by TASK-960. Existing match-arm `=>` parsing remains covered as noninterference, and older TASK-558 workflow-context tests were updated to the SPEC-072 pure-closure boundary.
