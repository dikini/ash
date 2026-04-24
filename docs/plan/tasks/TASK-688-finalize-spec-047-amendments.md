# TASK-688: Finalize SPEC-047 amendments and targeted spec updates

## Status: ✅ Complete

## Description

Finalize the minimal normative amendments for SPEC-047 so the Act monad contract is frozen against the implementation landed in TASK-685 through TASK-687.

## Specification Reference

- SPEC-047
- SPEC-001
- SPEC-002
- SPEC-003
- SPEC-004
- SPEC-027
- SPEC-031
- SPEC-BUILTIN-FN

## Dependencies

- 📝 TASK-672: prerequisite task
- 📝 TASK-685: prerequisite task
- 📝 TASK-686: prerequisite task
- 📝 TASK-687: prerequisite task

## Requirements

### Functional Requirements

1. Keep SPEC-047 as the primary normative source for Phase 97 Act semantics.
2. Keep SPEC-047, PLAN-097, and the Phase 97 PLAN-INDEX packet aligned around the same additive architecture.
3. Amend only the targeted downstream specs needed to reflect the frozen contract.
4. Preserve the additive architecture: `Act<A>` is expression-level, `invoke` is the runtime primitive, and `unit`/`bind`/`then`/`guard` remain library functions.
5. Keep workflow-level `act` semantics unchanged while documenting expression-level `act { ... }` interop.
6. Document the existing `Type::Fun(...)` coexistence boundary rather than replacing it.
7. Avoid reopening SPEC-025 scope or expanding the builtin-function contract beyond the targeted SPEC-BUILTIN-FN clarification.

### Editorial Requirements

1. Use surgical insertions instead of rewrites.
2. Avoid duplicating SPEC-047 content verbatim across downstream specs.
3. Ensure cross-references point back to SPEC-047 for the normative semantics.

## TDD Steps

### Step 1: Audit the target specs

Cross-check the current text of SPEC-001, SPEC-002, SPEC-003, SPEC-004, SPEC-027, SPEC-031, and SPEC-BUILTIN-FN against SPEC-047.

### Step 2: Draft minimal amendments

Add only the necessary clauses, notes, or cross-references to align the frozen contract with existing spec boundaries.

### Step 3: Verify consistency

Confirm the amended specs do not introduce contradiction with workflow-first semantics, purity boundaries, or builtin-function policy.

### Step 4: Update phase artifacts

If any spec text changes, ensure the phase documentation and changelog reflect the amendment set.

## Verification Steps

- [x] SPEC-047 amendments are consistent with landed TASK-685 through TASK-687 behavior.
- [x] Targeted spec files contain only minimal surgical updates.
- [x] PLAN-INDEX.md reflects the task status accurately.
- [x] CHANGELOG.md includes the Phase 97 spec-amendment note.

## Notes

- Phase 97 remains additive.
- Keep the frozen boundary between workflow execution and expression-level Act semantics intact.
- Prefer cross-references over repeated normative prose.
