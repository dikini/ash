# TASK-1896: Evidence Row Substrate

**Status:** Planned
**Plan:** [PLAN-194](../PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md)

## Description

Add evidence row records for tests, laws, proofs, runtime monitors, and observation evidence.

## Requirements

1. Represent evidence rows as requirement and record metadata, not authority grants.
2. Preserve evidence kinds for tests, laws, proof certificates, runtime monitors, and observation
   evidence.
3. Key evidence by predicate summary, boundary identity, snapshot environment, module identity, and
   evidence source.
4. Fail closed for missing, invalid, stale, or kind-incompatible evidence.
5. Preserve enough metadata for diagnostics and later evidence caching.

## TDD Steps

1. RED: add evidence-row parsing/summary or Core-carrier tests for each evidence kind supported by
   the current syntax layer.
2. RED: add stale/missing/kind-mismatch evidence rejection tests.
3. GREEN: implement evidence row carriers and validation hooks.
4. Verify evidence row mention does not install operation/resource/role authority.

## Completion Checklist

- [ ] Evidence kinds are represented distinctly.
- [ ] Evidence rows remain requirements/records.
- [ ] Missing or invalid evidence fails closed.
- [ ] Evidence identity includes predicate, boundary, snapshot, module, and source metadata.
- [ ] Authority-neutral evidence regressions pass.
