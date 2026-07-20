# TASK-1987: Archive, Redirect, and Context Migration

**Status:** Planned
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

- [ ] Every displaced artifact has a disposition and preserved provenance.
- [ ] Productive inbound links route to canonical/current material.
- [ ] Agent packs contain no archive authority leakage.
- [ ] No hand-maintained duplicate snapshot tree is introduced.
