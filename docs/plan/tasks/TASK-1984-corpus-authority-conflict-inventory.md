# TASK-1984: Corpus Authority and Conflict Inventory

**Status:** Complete
**Phase:** [PLAN-202](../PLAN-202-FORMAL-SEMANTICS-AND-VERIFICATION-PROGRAMME.md)

## Description

Inventory productive Ash documentation and identify every competing authority claim for grammar,
typing/effects, lowering, Core/CPS, runtime semantics, observability, and conformance.

## Requirements

- Extend SPEC-071/DESIGN-035/DESIGN-042 metadata rather than inventing an unrelated catalog.
- Freeze the PLAN-202 scope manifest, repository revision, dirty-worktree qualification, productive
  roots, semantic Rust classification, and explicit exclusions before claiming completeness.
- Record artifact identity, claimed authority, canonical subject, inbound productive links,
  current/target/historical status, conflicting claims, unique content, and proposed disposition.
- Include the known `docs/spec/README.md`, formalization-boundary, parser-to-Core, and Phase 201
  status conflicts from PLAN-202.
- Verify claims against live target Rust paths and tests where documentation disagrees.
- Record the Phase 201 handoff state; do not treat its top-level completion label as proof while
  TASK-1971/TASK-1972 remain unresolved.
- Produce an audit; do not promote, move, archive, or delete documents in this task.

## TDD Steps

1. Add inventory-schema self-tests and fixtures for duplicate ownership/conflict cases.
2. Generate the inventory and conflict report.
3. Review every proposed canonical subject for an owner or explicit unresolved status.
4. Run orientation, reference metadata, link, and docs gates.

## Completion Checklist

- [x] Every Markdown document in the frozen `docs/` and top-level `reference/` scope has an
  explicit A5/unresolved classification-overlay record; the final generator reports zero missing
  overlay classifications and zero unclassified artifacts.
- [x] Duplicate and missing semantic owners are explicit generator findings; all eight PLAN-202
  subjects are intentionally recorded as `unresolved` pending TASK-1986.
- [x] Existing Phase 201 and reference-corpus work is reconciled at handoff revision `c9294828`.
- [x] No semantic conflict is resolved by chronology alone.
- [x] The audit supplies actionable inputs to TASK-1985/TASK-1986.

## Completion Evidence

- [Frozen scope manifest](../audits/TASK-1984-corpus-authority-scope.json) records revision
  `c9294828`, qualified dirty paths, productive roots, exclusions with reasons, a 2,302-entry
  Markdown classification overlay, two linked data artifacts, all six semantic Rust crate roots,
  six symbol/executed-test realization records, four structured PLAN-202 conflict records, and the
  eight unresolved canonical subjects.
- [Human audit](../audits/TASK-1984-corpus-authority-inventory.md) records the conflict ledger,
  Phase 201 handoff resolution, and follow-on inputs without promoting, moving, archiving, or
  deleting any document.
- `python3 tools/docs/generate_corpus_authority_inventory.py --root . --scope
  docs/plan/audits/TASK-1984-corpus-authority-scope.json --output
  docs/plan/audits/TASK-1984-corpus-authority-inventory.json` intentionally exits nonzero while
  preserving the complete JSON audit; its final run reports 2,304 artifacts, 152 explicit
  `invalid_evidence_path` conflicts, and zero missing-overlay or unclassified-artifact findings.
  The metadata-only status check leaves this task audit and `PLAN-INDEX.md` free of false
  contradictory-status findings; the parser-to-Core conflict is attached to both its documentation
  artifact and the `crates/ash-parser/src/lower.rs` realization record.
