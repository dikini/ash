# TASK-1972: Entry Artifact Carrier Alignment

**Status:** Planned
**Phase:** [PLAN-201 Semantic Cleanup Follow-up](../PLAN-201-SEMANTIC-CLEANUP-FOLLOWUP.md)
**Source audit:** [AUDIT-201 Semantic Removal Vs Rename](../audits/AUDIT-201-semantic-removal-vs-rename.md)

## Description

Align TCIR/AMIR entry-artifact carriers with target effect-row computation artifacts. Remove
workflow-artifact semantics that survived under entry/computation vocabulary unless they are proven
to be target-justified implementation details.

## Requirements

- Audit TCIR, AMIR, runtime artifact builder, and typechecker artifact carriers for preserved
  workflow-artifact semantics.
- Replace stale artifact semantics with effect-row computation artifacts over target Core/TCIR/AMIR.
- Rewrite artifact provenance tests to assert function/effect-row identity.
- Keep current core, typechecker, engine, and runtime artifact behavior green.
- Update Phase 201 semantic audit and closeout evidence.

## TDD Steps

1. Add or tighten tests that distinguish target effect-row artifacts from workflow-artifact
   compatibility semantics.
2. Refactor TCIR/AMIR/typechecker/runtime artifact carriers to the target computation model.
3. Rewrite focused artifact provenance tests around ordinary functions and effect rows.
4. Run core, typechecker, engine, Phase 201 gate, and docs/index checks together.

## Completion Checklist

- [ ] TCIR/AMIR entry artifacts no longer preserve workflow-artifact semantics.
- [ ] Artifact provenance tests assert target function/effect-row identity.
- [ ] Runtime artifact builder behavior remains green for target entries.
- [ ] Phase 201 gates block stale workflow/tower artifact names and variants.
- [ ] `CHANGELOG.md`, AUDIT-201, and relevant plan evidence are updated.

## Current Implementation Evidence (Verification Pending)

- `CheckedFunctionArtifact` now transports the selected checked Core function identity, effect
  row, result type, source anchor, and lowered body from engine checking to runtime artifact
  construction. Runtime artifact construction no longer fabricates a null-bodied application
  entry from a text-derived effect-row summary.
- TCIR computation provenance records checked function identity, row, and result type; AMIR
  provenance includes a deterministic fingerprint of the checked function body. The runtime
  summaries expose this checked-function provenance.
- Runtime artifact admission rejects a checked-function identity that does not match the selected
  entry point. Regression tests also distinguish artifacts built from different checked bodies and
  assert that their cache/artifact identities differ.
- Focused RED/GREEN coverage includes a real `ash run --dry-run` source with a non-pure
  `process spawn` row, engine identity-mismatch and body-provenance tests, and cache-identity
  coverage. Final cross-crate verification, Phase 201 gate confirmation, and closeout review are
  still pending; the completion checklist intentionally remains unchecked.
