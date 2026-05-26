# TASK-963: Stdlib and reference callable syntax migration

## Status: ✅ Complete

## Description

Migrate Ash source examples and daily-use reference pages to the callable syntax implemented by PLAN-121. This task is the repository-facing migration step after parser/typechecker support lands: standard-library `.ash` surfaces and the `reference/` corpus must stop teaching or depending on legacy `Fn(...) -> ...` and pure `|args| => ...` syntax except where explicitly documenting compatibility.

## Specification Reference

- SPEC-072 §4
- SPEC-072 §5
- SPEC-072 §10
- SPEC-072 C72-2, C72-3, C72-8

## Dependencies

- ✅ TASK-955: Tower callable syntax packet
- ✅ TASK-956: Callable syntax audit gate
- ✅ TASK-957: Pure callable type parser
- ✅ TASK-958: Callable type typechecking and rendering
- ✅ TASK-959: Pure closure arrow syntax
- ✅ TASK-960: Reserved tower callable arrows
- ✅ TASK-961: Callable syntax reference docs

## Requirements

### Functional Requirements

1. Audit `std/` for callable type spellings and pure closure examples.
2. Migrate standard-library `.ash` signatures from legacy `Fn(A, B) -> C` spellings to preferred `(A, B) -> C` where the parser/typechecker now accepts them.
3. Migrate standard-library pure closure examples from `|args| => body` to `|args| -> body` where they are intended to be pure closures.
4. Audit the top-level `reference/` corpus, not only `docs/reference/`, and update all current examples to prefer `(A, B) -> C` and `|args| -> body`.
5. Preserve legacy syntax only in sections explicitly labeled as compatibility or migration guidance.
6. Keep higher-stratum callable syntax examples honest: `-*>`, `=>`, and `=*>` may appear as reserved/future syntax, but executable snippets must not rely on unimplemented runtime semantics.
7. Update agent cards under `reference/agents/` when they include callable syntax rules or snippets.

### Non-goals

- Do not implement Act/Proc/Workflow callable runtime semantics.
- Do not rewrite unrelated stdlib APIs or reference prose while performing syntax migration.
- Do not convert tuple-argument functions into n-ary callables without checking the intended domain shape.
- Do not delete compatibility documentation for `Fn(...) -> ...`; relabel it as legacy/migration guidance instead.

## Work Steps

1. Run a repository audit for callable syntax in `std/` and `reference/`.
2. Classify each hit as current syntax, legacy compatibility documentation, historical text, or unrelated `=>` syntax.
3. Patch standard-library `.ash` files first so parser/typechecker tests exercise the new syntax against real library surfaces.
4. Patch `reference/` pages and agent cards to teach the new syntax as the default.
5. Add or update focused verification that the migrated stdlib corpus parses/checks where the existing stdlib gate supports it.
6. Run the reference validator and focused syntax scans that fail if unlabelled legacy syntax remains in `std/` or current `reference/` pages.
7. Update PLAN-121, PLAN-INDEX, task status surfaces, and CHANGELOG.md when completing the task.
8. Request independent review before marking complete.

## Dispatch

```yaml
agent: codex
reasoning: medium
worktree: recommended
skills:
  - rust-skills
  - ash-documentation-style-guide
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
    path = Path('crates/ash-parser/tests/task_963_stdlib_reference_callable_syntax.rs')
    text = path.read_text()
    required = [
        'stdlib_callable_signatures_parse_with_preferred_syntax',
        'stdlib_contains_no_legacy_fn_callback_signatures',
        'stdlib_callback_signatures_do_not_use_bare_unary_arrow_domains',
        'reference_current_examples_prefer_callable_arrow_syntax',
        'reference_current_examples_prefer_pure_closure_arrow',
        'legacy_callable_examples_are_labeled_compatibility',
    ]
    missing = [name for name in required if name not in text]
    assert not missing, f'{path} missing tests: {missing}'
    PY
  - cargo test -p ash-parser --test task_963_stdlib_reference_callable_syntax -- --nocapture
  - python3 tools/reference/check_frontmatter.py
  - |
    python3 - <<'PY'
    import re
    from pathlib import Path
    roots = [Path('std/src'), Path('reference')]
    legacy_fn = re.compile(r'\bFn\s*\(')
    old_closure = re.compile(r'\|[^|\n]*\|\s*=>')
    allowed_markers = ('compatibility', 'legacy', 'migration', 'historical', 'reserved')
    violations = []
    for root in roots:
        for path in sorted(root.rglob('*')):
            if path.suffix not in {'.ash', '.md'}:
                continue
            for lineno, line in enumerate(path.read_text().splitlines(), 1):
                if (legacy_fn.search(line) or old_closure.search(line)) and not any(marker in line.lower() for marker in allowed_markers):
                    violations.append(f'{path}:{lineno}: {line.strip()}')
    assert not violations, 'unlabelled legacy callable syntax remains:\n' + '\n'.join(violations)
    PY
checklist:
  - [x] `std/` callable type and pure closure syntax audited.
  - [x] Standard-library callable signatures prefer `(A, B) -> C` where accepted.
  - [x] Standard-library pure closures use `|args| -> body` where accepted.
  - [x] `reference/` current examples prefer the new syntax.
  - [x] Compatibility-only legacy examples are explicitly labeled.
  - [x] Higher-stratum arrow examples are marked reserved/future unless implemented.
  - [x] Reference validator passes.
  - [x] Independent review completed.
```

## Dependencies for Next Task

TASK-962 closeout depends on this task so the final acceptance matrix covers both implementation behavior and migrated daily-use surfaces.

## Notes

Area: stdlib/reference migration. Treat `reference/` as the daily-use corpus and `docs/` as working/historical material. Do not mark PLAN-121 complete while current stdlib or reference examples still teach legacy callable syntax as the default.

## Completion Notes

- Audited `std/src` and `reference/` for legacy callable type spelling and old pure-closure shorthand. `std/src` contained callback signature hits in `act.ash`, `list.ash`, `option.ash`, `proc.ash`, `result.ash`, and `workflow.ash`; no stdlib pure-closure shorthand migration was required.
- Migrated stdlib callback signatures to preferred callable arrow syntax: `(a) -> Bool`, `(a) -> b`, `(T) -> U`, `(E) -> F`, `(T) -> Result<U, E>`, `(A) -> Act<B>`, `(A) -> Proc<B>`, and `(A) -> Workflow<B>`.
- Added `crates/ash-parser/tests/task_963_stdlib_reference_callable_syntax.rs` to parse migrated stdlib surfaces, enforce no stdlib legacy callback signatures, enforce current reference examples prefer callable-arrow and pure-closure-arrow syntax, and require compatibility/reserved labels for intentional legacy mentions.
- Verified that remaining top-level `reference/` hits are labeled compatibility or reserved/future guidance from TASK-961, not current executable examples.
