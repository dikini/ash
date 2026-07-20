# TASK-1984: Corpus Authority and Conflict Inventory

**Status:** Planned
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

- [ ] Every productive document in scope is classified.
- [ ] Duplicate and missing semantic owners are explicit.
- [ ] Existing Phase 201 and reference-corpus work is reconciled.
- [ ] No semantic conflict is resolved by chronology alone.
- [ ] The audit supplies actionable inputs to TASK-1985/TASK-1986.
