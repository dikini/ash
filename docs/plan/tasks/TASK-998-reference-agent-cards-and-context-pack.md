# TASK-998: Reference agent cards and context pack

## Status: 📝 Planned

## Description

Create or update derivative agent cards, context-pack index entries, and common-confusion warnings for the Slice 2 stdlib, toolchain, and RuntimeKernel pages.

## Specification Reference

- [DESIGN-043](../../design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- [SPEC-075](../../spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- [PLAN-125](../PLAN-125-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- PLAN-INDEX Phase 130

## Dependencies

- ✅ TASK-993: Reference maintenance metadata and staleness substrate
- ✅ TASK-995: Ashgrove and CLI procedure pages
- ✅ TASK-996: RuntimeKernel pages
- 📝 TASK-997: Stdlib tower pages

## Requirements

1. Create `reference/agents/cards/stdlib-act.md`.
2. Create `reference/agents/cards/stdlib-proc.md`.
3. Create `reference/agents/cards/stdlib-workflow.md`.
4. Create `reference/agents/cards/stdlib-result.md`.
5. Create `reference/agents/cards/ash-cli.md`.
6. Create `reference/agents/cards/ashgrove.md`.
7. Create `reference/agents/cards/runtime-kernel.md`.
8. Update `reference/agents/context-pack-index.md`.
9. Update `reference/agents/common-confusions.md`.
10. Ensure every card has body-level `canonical_page` and `canonical_page_path` fields that resolve.

## Work Steps

1. Read the canonical page before writing each card.
2. Add compact syntax/usage snippets only when already present or evidenced in canonical pages.
3. Include forbidden stale claims for Ashgrove, RuntimeKernel, and tower semantics.
4. Include retrieval tags and must-check files.
5. Run the reference validator and repair card link-back failures.

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
  - python3 tools/reference/check_frontmatter.py
  - python3 tools/reference/check_frontmatter.py --pilot
  - |
    python3 - <<'PY'
    from pathlib import Path
    required = [
        'reference/agents/cards/stdlib-act.md',
        'reference/agents/cards/stdlib-proc.md',
        'reference/agents/cards/stdlib-workflow.md',
        'reference/agents/cards/stdlib-result.md',
        'reference/agents/cards/ash-cli.md',
        'reference/agents/cards/ashgrove.md',
        'reference/agents/cards/runtime-kernel.md',
    ]
    missing = [p for p in required if not Path(p).exists()]
    assert not missing, missing
    for rel in required:
        text = Path(rel).read_text()
        assert 'canonical_page:' in text, rel
        assert 'canonical_page_path:' in text, rel
        assert 'forbidden' in text.lower() or 'stale' in text.lower(), rel
    PY
checklist:
  - [ ] Agent cards created.
  - [ ] Cards resolve to canonical pages.
  - [ ] Common confusions updated.
  - [ ] Cards do not fork canonical page semantics.
```

## Dependencies for Next Task

TASK-999 closeout validates these derivative surfaces with the canonical pages.
