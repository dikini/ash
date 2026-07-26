# TASK-1985: Canonical Manifest, Metadata, and Validation

**Status:** Complete
**Phase:** [PLAN-202](../PLAN-202-FORMAL-SEMANTICS-AND-VERIFICATION-PROGRAMME.md)
**Depends on:** TASK-1984

## Description

Define and implement the A0-A5 authority manifest and metadata validation needed to make semantic
ownership, supersession, dependencies, and derived context packs machine-checkable.

## Requirements

- Define a versioned `canonical-corpus/v1` sidecar schema with `canonical_for`, authority level,
  controlled lifecycle, stable trace-node support, and inherited SPEC-071 evidence/relationship
  concepts.
- Keep SPEC-071 `reference/` frontmatter enums and required fields valid; document typed edges
  between the separate canonical and reference schemas instead of reusing incompatible enums.
- Validate unique ids/owners, acyclic supersession, valid paths, controlled enums, and generated
  artifact freshness.
- Keep top-level `reference/` derivative and preserve git-backed snapshot manifests.
- Reject authority conflicts rather than selecting a winner implicitly.

## TDD Steps

1. Add failing validator self-tests for duplicate ownership, supersession cycles, broken sources,
   and derivative authority leakage.
2. Implement the schema and validator changes.
3. Add the initial manifest entries without promoting unresolved conflicts.
4. Run validator self-tests and documentation gates.

## Completion Checklist

- [x] Manifest/schema is documented and validated.
- [x] Existing reference metadata remains compatible through an explicit separate-schema boundary.
- [x] Conflict and cycle fixtures fail closed.
- [x] Generated context packs cannot become authority.

## Completion Evidence

- [Canonical corpus sidecar documentation](../../spec/CANONICAL-CORPUS.md) and the adjacent
  [machine manifest](../../spec/CANONICAL-CORPUS.json) define the A0-A5 overlay, controlled
  lifecycles, PLAN-202 stable trace IDs/anchors, typed node/path edges, and source-hash freshness
  for the A4 context-pack derivative. The initial graph deliberately creates no A1/A2 semantic
  owner; all eight subjects and all four TASK-1984 conflicts remain unresolved for TASK-1986.
- The sidecar preserves top-level `reference/` as A4, validates a self-contained SPEC-071 CPS
  compatibility slice without altering frontmatter enums, and links the existing git-backed
  snapshot-manifest policy in SPEC-071 §12 rather than moving or replacing snapshots.
- `python3 tools/docs/validate_canonical_corpus.py --root . --manifest
  docs/spec/CANONICAL-CORPUS.json --format json --check-reference-frontmatter` reports
  `{"errors": [], "schema": "canonical-corpus-validation-report/v1"}`.
- `python3 -m unittest tools/docs/tests/test_validate_canonical_corpus.py` passes all 17
  validator contract tests; `python3 tools/docs/validate_orientation_indexes.py --self-test` and
  `bash scripts/check-docs-gate.sh` pass.
