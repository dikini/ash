# TASK-1987: Archive, Redirect, and Context Migration

**Status:** Complete
**Phase:** [PLAN-202](../PLAN-202-FORMAL-SEMANTICS-AND-VERIFICATION-PROGRAMME.md)
**Depends on:** TASK-1986

## Description

Quarantine displaced documentation using git-backed archive manifests, typed supersession, and
validated routing while keeping top-level `reference/` as the curated current corpus.

## Requirements

- Apply the PLAN-202 disposition taxonomy to every displaced artifact.
- Preserve unique content, original revision, reason, and replacement.
- Use redirect/tombstone files only where productive inbound links require them.
- Exclude archive/research material from default context packs and current examples.
- Measure retrieval quality before and after migration.

## TDD Steps

1. Add failing routing tests for archived sources leaking into productive paths.
2. Create snapshot/archive manifests and replacement routes.
3. Move or tombstone only after link and content-preservation checks pass.
4. Run link, metadata, orientation, context-pack, and docs gates.

## Completion Checklist

- [x] Every displaced artifact has a disposition and preserved provenance.
- [x] Productive inbound links route to canonical/current material.
- [x] Agent packs contain no archive authority leakage.
- [x] No hand-maintained duplicate snapshot tree is introduced.

## Completion evidence

- `reference/manifests/phase-202-archive.json` must record only genuinely historical artifacts
  with truthful Git provenance; active TASK-1984 audit/evidence remains live task evidence and is
  not an archived artifact.
- `reference/manifests/phase-202-redirects.json` routes the two productive workflow-first legacy
  contracts to their active canonical-core owners. The TASK-1984 audit evidence remains available
  as live, nonproductive task evidence without a redirect claim.
- `docs/plan/audits/TASK-1987-retrieval-quality.json` supplies stable before/after retrieval
  cases for every migrated A5 record, while `reference/agents/context-pack-index.md` routes
  current semantic work through the canonical-core default path.
- Verified 2026-07-24 after the migration validator distinguished active evidence from superseded
  or archived migration candidates:

  ```bash
  python3 tools/docs/validate_canonical_corpus.py --root . \
    --manifest docs/spec/CANONICAL-CORPUS.json --format json \
    --check-reference-frontmatter --require-promotion-completeness \
    --require-migration-completeness
  python3 -m unittest discover -s tools/docs/tests -p 'test_*.py'
  python3 tools/docs/validate_orientation_indexes.py --self-test
  bash scripts/check-docs-gate.sh
  git diff --check
  ```
