# TASK-1804: WorkflowForm deprecation cleanup

## Status: ✅ Complete

## Description

Reconcile WorkflowForm-era documentation after the target effect/type/Core/CPS redesign. The cleanup is docs-first and explicitly does not revive `WorkflowForm` as a primary syntax, type, IR, or runtime carrier. It marks older WorkflowForm-centric material as historical or superseded by the ambient computation model and points future work toward workflow facts represented by rows, Core/CPS carriers, trace/monitor sidecars, obligations, evidence, and provenance.

## Specification Reference

- [NOTE-010](../../notes/NOTE-010-WORKFLOW-FORM-PRECHECK-QUESTIONS.md): historical WorkflowForm Q&A backlog.
- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md): first-class workflow carrier MVP whose WorkflowForm-centric design language is now superseded for target planning.
- [SPEC-096b](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md): target computation-row/effect-system model.
- [SPEC-097b](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md): target type-system model.
- [SPEC-098b](../../spec/SPEC-098b-TARGET-IR.md): target IR model.
- [SPEC-099](../../spec/SPEC-099-CORE-LANGUAGE.md): Core language and ambient computation carriers.
- [SPEC-100](../../spec/SPEC-100-CORE-TYPE-CHECKING.md): Core type checking.

## Dependencies

- ✅ TASK-1803: NOTE-020 promoted taxonomy cleanup established the computation-row framing used by this reconciliation.
- ✅ Phase 176 closeout: repository has no active Phase 177 packet yet; this task is interphase docs/status maintenance.

## Scope

### In scope

1. Mark NOTE-010 as historical/superseded by the ambient computation model.
2. Add a no-revival clarification to SPEC-056 and related index summaries.
3. Preserve historical/current-state specs as references while preventing them from being read as future implementation mandates.
4. Update `docs/notes/NOTE-INDEX.md`, `docs/spec/SPEC-INDEX.md`, `docs/spec/README.md`, and `CHANGELOG.md`.

### Out of scope

1. No Rust implementation changes.
2. No parser/typechecker/runtime changes.
3. No new `WorkflowForm` carrier, AST node, Core term, CPS term, or runtime representation.
4. No deletion of frozen current-state semantics just because they contain historical workflow-form vocabulary.

## Requirements

### Functional Requirements

1. Documentation must say future work is workflow-fact reconciliation, not WorkflowForm revival.
2. Historical WorkflowForm terminology must be explicitly marked as historical or superseded where it is likely to mislead planning.
3. Index summaries must route agents toward ambient computation, rows, Core/CPS carriers, trace/monitor sidecars, obligations, evidence, and provenance.
4. Follow-up seeds may mention ambient workflow-fact integration only; they must not suggest implementing `WorkflowForm` as a primary carrier.

### Non-Functional Requirements

1. Preserve historical evidence and current-state specs.
2. Avoid broad rewrites unrelated to WorkflowForm deprecation.
3. Keep docs-index and changelog policy satisfied.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - python3 - <<'PY'
    from pathlib import Path
    root = Path('.')
    note = (root / 'docs/notes/NOTE-010-WORKFLOW-FORM-PRECHECK-QUESTIONS.md').read_text()
    spec = (root / 'docs/spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md').read_text()
    readme = (root / 'docs/spec/README.md').read_text()
    assert 'Historical / superseded' in note
    assert 'WorkflowForm is not revived' in spec
    assert 'ambient computation' in readme
    assert 'no new WorkflowForm implementation backlog' in note
    PY
checklist:
  - [x] NOTE-010 marked historical/superseded.
  - [x] SPEC-056 no-revival clarification added.
  - [x] Index/readme summaries route future work to ambient workflow facts.
  - [x] CHANGELOG.md updated.
  - [x] Docs gates pass.
```

## Follow-up Seeds

Future work, if any, belongs to ambient workflow-fact reconciliation:

- align workflow obligations/evidence/provenance with computation-row families;
- connect workflow trace/monitor facts to Core/CPS sidecars;
- clarify current-state historical workflow terminology versus target-state ambient facts;
- keep legacy workflow declaration translation as compatibility behavior only.

These seeds are not a mandate to implement `WorkflowForm`.

## Notes

This task intentionally leaves frozen/current-state specs intact when they document old workflow-form behavior. The cleanup only prevents stale terminology from being promoted into new target implementation work.
