# TASK-S57-7: Post-SPEC-Update Review of Phase 57B Tasks

## Status: ⬜ Pending

## Description

After Phase 57A SPEC updates complete, review all Phase 57B implementation tasks for validity against the updated specifications. Syntax and semantics changes in 57A may invalidate specific task content, assumptions, or acceptance criteria.

## Scope

Review each 57B task for alignment with completed 57A SPEC updates:

### Review Checklist

**For each 57B task (359-369):**

- [ ] **Syntax validity**: Does task use normative syntax from 57A?
- [ ] **API alignment**: Does task reference correct runtime APIs (Engine, not fictional Runtime)?
- [ ] **Spec citations**: Are "Spec:" fields updated from MCE-001 to actual SPEC citations?
- [ ] **Type consistency**: Do types match across tasks (especially RuntimeError style)?
- [ ] **Acceptance criteria**: Are tests implementable with current AST/surface?
- [ ] **Crate names**: Does task reference existing crates (ash-std, not ash-stdlib)?

### Tasks Likely Needing Updates

| Task | Likely Issues | Action |
|------|---------------|--------|
| TASK-359 | References ash-stdlib | Update to ash-std |
| TASK-360 | Cites MCE not SPEC | Update citations after 57A |
| TASK-361 | Uses cap Args, use runtime.Args | Use capability Args, use runtime::Args |
| TASK-362 | May reference await | Use "observe completion via control authority" |
| TASK-363 | Too large, fictional Runtime::new() | Split, use Engine per SPEC-010 |
| TASK-366 | Says "implement ash run" | Redefine as "redefine entry-point semantics" |
| TASK-367 | Cites MCE for error format | Anchor to SPEC-005/SPEC-021 |
| TASK-368 | Over-ambitious | Split into minimum vs extended |
| TASK-369 | References non-existent crates, MCE acceptance | Fix crate names, remove MCE lifecycle |

### Validation Gates

Before any 57B task begins implementation:

1. **SPEC completion verified**: All blocking 57A tasks show ✅ Complete
2. **This review task complete**: TASK-S57-7 shows ✅ Complete
3. **Task-specific validation**: Implementer verifies task content against updated SPEC

### Deliverables

- [ ] Review report: List of tasks requiring updates
- [ ] Updated task files: All 57B tasks aligned with 57A SPEC
- [ ] Validation checklist: Signed off by reviewer

## Related

- All TASK-S57-*: SPEC updates that may affect 57B
- All TASK-3[5-9]*: Implementation tasks to review

## Est. Hours: 2-3

## Blocking

- All 57B tasks should verify this task is complete before starting
- Or: Individual 57B tasks can proceed if they validate against 57A SPEC independently
