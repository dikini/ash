# TASK-997: Reference stdlib tower pages

## Status: 📝 Planned

## Description

Create stdlib tower reference pages for `Act`, `Proc`, `Workflow`, and `Result` as public library/API surfaces while preserving the existing `reference/language/` pages as language concept pages.

## Specification Reference

- [DESIGN-043](../../design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- [SPEC-075](../../spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- [PLAN-125](../PLAN-125-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- PLAN-INDEX Phase 130
- [SPEC-047](../../spec/SPEC-047-ACT-MONAD.md)
- [SPEC-048](../../spec/SPEC-048-PROC-LIBRARY.md)
- [SPEC-050](../../spec/SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md)
- [SPEC-051](../../spec/SPEC-051-WORKFLOW-SEMANTICS.md)
- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md)
- [SPEC-069](../../spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md)

## Dependencies

- ✅ TASK-993: Reference maintenance metadata and staleness substrate
- ✅ TASK-994: Reader journey link targets established

## Requirements

1. Create `reference/stdlib/README.md`.
2. Create `reference/stdlib/act.md`.
3. Create `reference/stdlib/proc.md`.
4. Create `reference/stdlib/workflow.md`.
5. Create `reference/stdlib/result.md`.
6. Cross-link existing `reference/language/effects-act.md`, `reference/language/processes-proc.md`, `reference/language/workflows.md`, and `reference/language/generalized-do.md` without collapsing concept pages into API pages.
7. Explain current public operations, examples, evidence, and limitations.
8. Preserve the strict tower: Pure < Act < Proc < Workflow.
9. Preserve `Result` domain failure vs operational bottom distinction.

## Work Steps

1. Inspect `std/src/act.ash`, `std/src/proc.ash`, `std/src/workflow.ash`, `std/src/result.ash`, current examples, and relevant tests before writing examples.
2. Make `reference/stdlib/README.md` teach the public tower map.
3. Make each page include current operations and limitations without inventing syntax.
4. Link language concept pages and stdlib pages both ways where appropriate.
5. Classify examples honestly.

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
  - python3 tools/reference/check_frontmatter.py
  - python3 tools/reference/check_frontmatter.py --pilot
  - |
    python3 - <<'PY'
    from pathlib import Path
    required = [
        'reference/stdlib/README.md',
        'reference/stdlib/act.md',
        'reference/stdlib/proc.md',
        'reference/stdlib/workflow.md',
        'reference/stdlib/result.md',
    ]
    missing = [p for p in required if not Path(p).exists()]
    assert not missing, missing
    text = '\n'.join(Path(p).read_text() for p in required)
    for term in ['Act', 'Proc', 'Workflow', 'Result', 'Pure']:
        assert term in text, term
    assert 'operational bottom' in text or 'Operational bottom' in text
    PY
checklist:
  - [ ] Stdlib tower pages created.
  - [ ] Language concept pages remain distinct.
  - [ ] Current examples are grounded in live stdlib/parser evidence.
  - [ ] Result/domain failure is not collapsed into operational bottom.
```

## Dependencies for Next Task

TASK-998 must create stdlib agent cards from these canonical pages after they exist.
