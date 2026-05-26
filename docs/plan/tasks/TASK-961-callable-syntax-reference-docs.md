# TASK-961: Callable syntax reference docs

## Status: ✅ Complete

## Description

Update reference pages, agent card, and amended legacy specs so daily readers learn the new syntax and do not copy stale examples. This task establishes the documentation guidance; TASK-963 performs the repository-wide `std/` and current `reference/` syntax migration after implementation support lands.

## Specification Reference

- SPEC-072 §10
- SPEC-072 C72-8
- SPEC-071

## Dependencies

- ✅ TASK-955: Tower callable syntax packet
- ✅ TASK-957 through TASK-960 completion for final implemented-behavior docs.

## Requirements

### Functional Requirements

1. Update `reference/language/functions.md` and sub-pages to prefer `(A, B) -> C` and `|args| -> body` in prose and canonical teaching examples.
2. Update `reference/agents/cards/functions.md`.
3. Patch SPEC-027/SPEC-031 notes or sections to point to SPEC-072 as the current callable syntax owner.
4. Hand off broad `std/` and top-level `reference/` syntax scans/migration to TASK-963 rather than closing the phase with stale examples.

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
  - git diff --check
  - python3 -m py_compile tools/reference/check_frontmatter.py
  - python3 tools/reference/check_frontmatter.py
  - python3 tools/reference/check_frontmatter.py --pilot
  - |
    python3 - <<'PY'
    from pathlib import Path
    required = {
        Path('reference/language/functions.md'): ['(T) -> T', '|x| -> x * 2', 'SPEC-072'],
        Path('reference/language/functions/declarations.md'): ['f: (T) -> U'],
        Path('reference/language/functions/calls-and-values.md'): ['f: (Int, Int) -> Int', 'reserved for future `Act`, `Proc`, and `Workflow` callable syntax'],
        Path('reference/language/functions/local-and-anonymous.md'): ['|args| =>', 'reserved and rejected'],
        Path('reference/language/functions/implementation-notes.md'): ['|params| -> expr', 'fail-closed reserved arrows'],
        Path('reference/agents/cards/functions.md'): ['f: (T) -> U', '|x| -> x + 1', 'legacy `Fn(...) -> ...` as compatibility syntax only'],
        Path('docs/spec/SPEC-027-PURE-FUNCTIONS.md'): ['SPEC-072 owns the current preferred callable source syntax'],
        Path('docs/spec/SPEC-031-FIRST-CLASS-FUNCTIONS.md'): ['preferred pure closure shorthand is `|args| -> body`'],
    }
    for path, needles in required.items():
        text = path.read_text()
        missing = [needle for needle in needles if needle not in text]
        assert not missing, f'{path} missing {missing}'
    forbidden = {
        Path('reference/language/functions.md'): ['|x| => x * 2', 'f: Fn(T) -> T'],
        Path('reference/language/functions/declarations.md'): ['f: Fn(T) -> U'],
        Path('reference/language/functions/calls-and-values.md'): ['f: Fn(Int)', 'f: Fn(Int, Int)', '|x| => x + 1'],
        Path('reference/language/functions/implementation-notes.md'): ['|params| => expr` desugars'],
        Path('reference/agents/cards/functions.md'): ['f: Fn(T) -> U', '|x| => x + 1', 'Use `Fn(T) -> U`'],
    }
    for path, needles in forbidden.items():
        text = path.read_text()
        present = [needle for needle in needles if needle in text]
        assert not present, f'{path} still contains stale snippets {present}'
    PY
checklist:
  - [x] Required docs/audit artifacts updated.
  - [x] Status surfaces reconciled.
  - [x] Independent review completed where required.
```

## Dependencies for Next Task

This task contributes to PLAN-121 and SPEC-072 completion.

## Notes

Area: reference documentation. Keep the callable-stratum axis separate from return type. Pure smart constructors returning `Act<T>`, `Proc<T>`, or `Workflow<T>` remain pure callables. Broad current-corpus migration for `std/` and `reference/` is owned by TASK-963.

## Completion Notes

- Updated `reference/language/functions.md`, focused functions sub-pages, and `reference/agents/cards/functions.md` to prefer `(A, B) -> C` callable types and `|args| -> body` pure closures.
- Added reader-facing guidance that `-*>, =>, =*>` are reserved in callable-type and closure-literal contexts, while pure smart constructors use `->` with tower values in return position.
- Replaced the placeholder TASK-961 verification command with reference frontmatter gates and focused stale-snippet assertions; broad corpus migration remains owned by TASK-963.
