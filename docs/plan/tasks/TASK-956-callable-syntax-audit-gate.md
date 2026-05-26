# TASK-956: Callable syntax audit gate

## Status: ✅ Complete

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
  - |
    python3 - <<'PY'
    from pathlib import Path
    audit = Path('docs/plan/audits/TASK-956-callable-syntax-audit-gate.md')
    text = audit.read_text()
    required = [
        'Parser Seams',
        'Typechecker, Rendering, And Application Seams',
        'Closure And Runtime Seams',
        'Module Summary And Import/Export Seams',
        'Stdlib And Reference Exposure',
        'tuple-vs-n-ary',
        'Partial application',
        'TASK-957',
        'TASK-958',
        'TASK-959',
        'TASK-960',
        'TASK-963',
    ]
    missing = [needle for needle in required if needle not in text]
    assert not missing, f'{audit} missing required audit coverage: {missing}'
    for rel in [
        'docs/plan/tasks/TASK-957-pure-callable-type-parser.md',
        'docs/plan/tasks/TASK-958-callable-type-typeck-rendering.md',
        'docs/plan/tasks/TASK-959-pure-closure-arrow-syntax.md',
        'docs/plan/tasks/TASK-960-reserved-tower-callable-arrows.md',
        'docs/plan/tasks/TASK-963-stdlib-and-reference-callable-syntax-migration.md',
    ]:
        task_text = Path(rel).read_text()
        placeholder = 'false' + ' #'
        assert placeholder not in task_text, f'{rel} still contains a placeholder false command'
        assert 'cargo test -p ' in task_text, f'{rel} missing focused cargo verification'
    PY
checklist:
  - [x] Required docs/audit artifacts updated.
  - [x] Status surfaces reconciled.
  - [x] Independent review completed against the live tree; findings were cross-checked against this audit before closeout.
```

## Dependencies for Next Task

This task contributes to PLAN-121 and SPEC-072 completion.

## Notes

Area: audit. Keep the callable-stratum axis separate from return type. Pure smart constructors returning `Act<T>`, `Proc<T>`, or `Workflow<T>` remain pure callables.
