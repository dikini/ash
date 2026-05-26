# TASK-962: Tower callable syntax closeout

## Status: ✅ Complete

## Description

Close out SPEC-072/PLAN-121 with acceptance evidence, broad gates, status reconciliation, and independent review remediation.

## Specification Reference

- SPEC-072 §11
- PLAN-121 §7

## Dependencies

- ✅ TASK-955: Tower callable syntax packet
- ✅ TASK-956: Callable syntax audit gate
- ✅ TASK-957: Pure callable type parser
- ✅ TASK-958: Callable type typecheck and rendering
- ✅ TASK-959: Pure closure arrow syntax
- ✅ TASK-960: Reserved tower callable arrows
- ✅ TASK-961: Callable syntax reference docs
- ✅ TASK-963: Stdlib and reference callable syntax migration

## Requirements

### Functional Requirements

1. Create an acceptance matrix for C72-1 through C72-8.
2. Run focused and broad gates.
3. Verify TASK-963 migrated current `std/` and top-level `reference/` examples to preferred syntax except explicitly labeled compatibility material.
4. Run independent review focused on parser ambiguity, stale docs, stdlib/reference migration coverage, and callable-stratum/return-type conflation.
5. Patch findings and reconcile all status surfaces.

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
    matrix = Path('docs/plan/audits/TASK-962-tower-callable-syntax-acceptance-matrix.md')
    text = matrix.read_text()
    for marker in [f'C72-{i}' for i in range(1, 9)]:
        assert marker in text, f'missing {marker}'
    for required in [
        'task_957_callable_type_parser',
        'task_958_callable_type_rendering',
        'task_959_pure_closure_arrow',
        'task_960_reserved_callable_arrows',
        'task_963_stdlib_reference_callable_syntax',
        'No space left on device',
    ]:
        assert required in text, f'missing evidence marker: {required}'
    PY
  - cargo fmt --check
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - cargo test --workspace
  - cargo doc --workspace --no-deps
  - python3 tools/reference/check_frontmatter.py
  - python3 tools/reference/check_frontmatter.py --pilot
checklist:
  - [x] Required docs/audit artifacts updated.
  - [x] TASK-963 stdlib/reference migration evidence included in the acceptance matrix.
  - [x] Status surfaces reconciled.
  - [x] Independent review completed where required.
```

## Dependencies for Next Task

This task contributes to PLAN-121 and SPEC-072 completion.

## Notes

Area: closeout. Keep the callable-stratum axis separate from return type. Pure smart constructors returning `Act<T>`, `Proc<T>`, or `Workflow<T>` remain pure callables.
