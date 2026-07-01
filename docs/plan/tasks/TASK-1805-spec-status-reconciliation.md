# TASK-1805: Spec status reconciliation after target cleanup

## Status: ✅ Complete

## Description

Reconcile spec status surfaces after Phase 176 and the interphase NOTE-020/WorkflowForm cleanup. Several specs still said `Planned` or `Draft` in `docs/spec/SPEC-INDEX.md`, `docs/spec/README.md`, or their own headers even though their implementation phases are complete. This is an interphase documentation/status maintenance task, not a language/runtime implementation phase.

## Specification Reference

- [SPEC-081](../../spec/SPEC-081-LAW-TEST-EVIDENCE-SUBSTRATE.md): Phase 145 complete.
- [SPEC-082](../../spec/SPEC-082-PROPERTY-GENERATION-AND-SHRINKING-SUBSTRATE.md): Phase 146 complete.
- [SPEC-086](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md): Phase 150 complete.
- [SPEC-087](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md): Phase 151 complete with recursive bounded generation still fail-closed/deferred.
- [SPEC-088](../../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md): Phase 152 complete.
- [SPEC-089](../../spec/SPEC-089-LIST-BUILTIN-TO-STDLIB.md): Phase 153 complete, with runtime `Value::List` removal closed by Phase 176.
- [SPEC-091](../../spec/SPEC-091-LET-DESTRUCTORS.md): Phase 155 complete.
- [SPEC-092](../../spec/SPEC-092-PARSER-BLOCKER-RESOLUTION.md): Phase 156 complete.
- [SPEC-094](../../spec/SPEC-094-LANGUAGE-SURFACE-FIX.md): Phase 158 complete, with TASK-1580 closed by Phase 176.
- [SPEC-099](../../spec/SPEC-099-CORE-LANGUAGE.md): Phase 161 complete.
- [SPEC-100](../../spec/SPEC-100-CORE-TYPE-CHECKING.md): Phase 162 complete.
- [SPEC-101](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md): Phase 163 complete.
- [SPEC-102](../../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md): Phase 164 complete.

## Dependencies

- ✅ Phase 145 through Phase 164 implementation and closeout rows in `docs/plan/PLAN-INDEX.md`.
- ✅ Phase 176 cleanup and closeout, especially QuickCheck recursive-combinator fail-closed re-scope and list-runtime reconciliation.
- ✅ TASK-1803 and TASK-1804 interphase docs cleanup.

## Scope

### In scope

1. Update stale spec headers from `Planned`/`Draft` to honest implemented-MVP statuses.
2. Update `docs/spec/SPEC-INDEX.md` and `docs/spec/README.md` rows for the targeted specs.
3. Preserve caveats instead of overclaiming complete target behavior.
4. Add verification assertions that the old stale statuses are gone from the targeted surfaces.

### Out of scope

1. No Rust implementation changes.
2. No new `PLAN-177` or feature phase.
3. No reopening completed implementation phases.
4. No removal of explicitly deferred future work such as proof-producing synthesis, arbitrary user Monad execution, target inference, or bounded recursive QuickCheck generation.

## Reconciliation Decisions

| Spec | Prior stale status | Reconciled status | Caveat |
|---|---|---|---|
| SPEC-081 | Planned | Implemented MVP (Phase 145) | Solver/symbolic proof evidence remains future non-test evidence. |
| SPEC-082 | Planned | Implemented MVP (Phase 146) | Broader generator/shrinker expansion remains future work. |
| SPEC-086 | Planned | Implemented MVP (Phase 150) | Hardened by SPEC-087/Phase 151. |
| SPEC-087 | Planned | Implemented MVP (Phase 151; Phase 176 recursive cleanup) | Bounded recursive generation remains fail-closed/deferred. |
| SPEC-088 | Draft | Implemented MVP (Phase 152) | Cross-stratum serialization and broader capture features remain out of scope. |
| SPEC-089 | Draft | Implemented MVP (Phase 153; Phase 176 runtime cleanup) | `Value::List` removal completed by Phase 176. |
| SPEC-091 | Draft | Implemented MVP (Phase 155) | No new destructuring surface beyond scoped phase deliverable. |
| SPEC-092 | Draft | Implemented MVP (Phase 156) | Parser blockers resolved for list/surface migration scope. |
| SPEC-094 | Draft | Implemented MVP (Phase 158; Phase 176 tail closure) | Deferred list/language-surface tail reconciled by Phase 176. |
| SPEC-099 | Draft | Implemented MVP / formal spec still design-level | Core substrate implemented by Phase 161. |
| SPEC-100 | Draft | Implemented MVP / formal spec still design-level | Core type checker slice implemented by Phase 162. |
| SPEC-101 | Draft | Implemented MVP | Core lazy/memo mode substrate implemented by Phase 163. |
| SPEC-102 | Draft | Implemented MVP | Continuation multiplicity substrate implemented by Phase 164. |

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
    idx = (root / 'docs/spec/SPEC-INDEX.md').read_text()
    readme = (root / 'docs/spec/README.md').read_text()
    for spec in ['081','082','086','087','088','089','091','092','094','099','100','101','102']:
        assert f'SPEC-{spec}' in idx
    stale = [
        'SPEC-081-LAW-TEST-EVIDENCE-SUBSTRATE.md) | Planned',
        'SPEC-082-PROPERTY-GENERATION-AND-SHRINKING-SUBSTRATE.md) | Planned',
        'SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | Planned',
        'SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md) | Planned',
        'SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md) | Draft',
        'SPEC-089-LIST-BUILTIN-TO-STDLIB.md) | Draft',
        'SPEC-091-LET-DESTRUCTORS.md) | Draft',
        'SPEC-092-PARSER-BLOCKER-RESOLUTION.md) | Draft',
        'SPEC-094-LANGUAGE-SURFACE-FIX.md) | 📝 Draft',
    ]
    for needle in stale:
        assert needle not in idx, needle
    assert 'recursive bounded generation remains fail-closed/deferred' in idx
    assert 'Value::List' in idx
    assert 'Implemented MVP (Phase 163)' in idx
    assert 'Implemented MVP (Phase 164)' in idx
    assert 'recursive bounded generation remains fail-closed/deferred' in readme
    PY
checklist:
  - [x] Task record created.
  - [x] Spec headers updated where stale.
  - [x] SPEC-INDEX statuses updated with caveats.
  - [x] SPEC README statuses updated with caveats.
  - [x] CHANGELOG.md updated.
  - [x] Docs gates pass.
```

## Notes

This task intentionally leaves target-state specs such as `SPEC-095b`, `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099b`, and `SPEC-099c` as draft/target-state unless their specific implementation status was in scope. The aim is to remove clear stale drift, not to declare the whole target language finished.

While editing `docs/spec/README.md`, this task also refreshed adjacent SPEC-083 and SPEC-084 README rows from `Planned` to `Implemented MVP` because their spec headers and `SPEC-INDEX.md` already had the correct implemented status. No other SPEC-083/SPEC-084 surfaces were changed.
