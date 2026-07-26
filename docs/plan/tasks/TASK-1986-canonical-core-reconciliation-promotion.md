# TASK-1986: Canonical Core Reconciliation and Promotion

**Status:** Complete
**Phase:** [PLAN-202](../PLAN-202-FORMAL-SEMANTICS-AND-VERIFICATION-PROGRAMME.md)
**Depends on:** TASK-1985

## Description

Reconcile the candidate grammar/type/effect/lowering/Core/CPS/runtime/conformance sources and
promote one compact, coherent canonical core.

## Requirements

- Assign one canonical owner per subject listed in PLAN-202.
- Resolve workflow-first versus target function/Core/CPS authority explicitly.
- Preserve unique historical rationale through typed links or archive manifests.
- Update conformance cases and handoff contracts with each promoted rule.
- Generate the default human and agent read paths from the manifest.

## TDD Steps

1. Add failing ownership/read-path fixtures for the known authority conflicts.
2. Reconcile one vertical slice at a time: grammar, type/effect, lowering, Core/CPS, runtime,
   observable/conformance.
3. Run executable examples or conformance fixtures for each promoted slice.
4. Regenerate indexes/packs and run documentation gates.

## Completion Checklist

- [x] The canonical core covers all eight PLAN-202 subjects.
- [x] No unresolved conflict is presented as canonical.
- [x] Default agent paths exclude historical/research claims.
- [x] Handoff and conformance artifacts cite stable rule identities.

## Completion Evidence

- `docs/spec/CANONICAL-CORE.md` owns the compact target vocabulary, grammar, type/effect,
  Core/CPS, lowering, operational, observable, and implementation-conformance rules.
- `CANONICAL-CORPUS.json` assigns exactly one active A1/A2 owner to every PLAN-202 subject,
  classifies the former formalization-boundary and parser-to-Core sources as A5 superseded
  history, preserves them through typed supersession edges, and generates A5-free human/agent
  default paths.
- A2 handoffs and the A3 conformance node carry stable `LOWER-*`, `OBS-*`, and `CONF-*` trace IDs.
- Verification on 2026-07-24: `python3 -m unittest discover -s tools/docs/tests -p 'test_*.py'`
  (45 tests), `python3 tools/docs/validate_canonical_corpus.py --root . --manifest
  docs/spec/CANONICAL-CORPUS.json --require-promotion-completeness --check-reference-frontmatter`,
  `python3 tools/docs/validate_orientation_indexes.py --self-test`, and
  `bash scripts/check-docs-gate.sh` all passed.
