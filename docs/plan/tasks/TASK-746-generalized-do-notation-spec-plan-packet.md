# TASK-746: Generalized Do-Notation Spec/Plan Packet

## Status: ✅ Complete

## Description

Promote [DESIGN-031](../../design/DESIGN-031-GENERALIZED-DO-NOTATION.md) into a normative generalized typed do-notation specification and a Phase 105 implementation plan. This task is docs/planning only and intentionally does not implement parser, typechecker, lowering, or runtime behavior.

## Specification Reference

- [DESIGN-031](../../design/DESIGN-031-GENERALIZED-DO-NOTATION.md)
- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md)
- [PLAN-101](../PLAN-101-GENERALIZED-TYPED-DO-NOTATION.md)

## Dependencies

- ✅ Phase 97: Act substrate complete.
- ✅ Phase 98: Proc/process/failure/workflow boundary substrate complete.
- ✅ Phase 99: `proc::from_act` embedding boundary complete.
- 🟢 Phase 104: active/in-flight; Phase 105 implementation must wait for closeout unless explicitly authorized.

## Requirements

1. Create [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md) as the normative owner of `do:K` notation.
2. Create [PLAN-101](../PLAN-101-GENERALIZED-TYPED-DO-NOTATION.md) as the implementation plan for Phase 105.
3. Register Phase 105 and TASK-746 through TASK-753 in [PLAN-INDEX](../PLAN-INDEX.md).
4. Add the new spec to [docs/spec/README.md](../../spec/README.md).
5. Update [DESIGN-031](../../design/DESIGN-031-GENERALIZED-DO-NOTATION.md) to point at the promoted spec/plan.
6. Update [CHANGELOG.md](../../../CHANGELOG.md).
7. Record Phase 104 non-interference explicitly.

## Docs Steps

### Step 1: Inspect current state

Verify:

- current `act {}` grammar still uses `x = expr;` and `ret expr;`;
- no existing SPEC-054 / PLAN-101 / TASK-746 collision;
- Phase 104 owns TASK-741 through TASK-745.

### Step 2: Write spec and plan

Create:

- `docs/spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md`
- `docs/plan/PLAN-101-GENERALIZED-TYPED-DO-NOTATION.md`

### Step 3: Create implementation task files

Create task files:

- `docs/plan/tasks/TASK-746-generalized-do-notation-spec-plan-packet.md`
- `docs/plan/tasks/TASK-747-do-block-surface-ast-and-parser-substrate.md`
- `docs/plan/tasks/TASK-748-do-target-kinding-and-dictionary-resolution.md`
- `docs/plan/tasks/TASK-749-typed-do-elaboration-and-lowering.md`
- `docs/plan/tasks/TASK-750-act-block-compatibility-and-migration.md`
- `docs/plan/tasks/TASK-751-proc-do-integration-and-tower-behavior.md`
- `docs/plan/tasks/TASK-752-do-notation-diagnostics.md`
- `docs/plan/tasks/TASK-753-do-notation-docs-examples-closeout.md`

### Step 4: Update indexes and changelog

Patch:

- `docs/spec/README.md`
- `docs/plan/PLAN-INDEX.md`
- `docs/design/DESIGN-031-GENERALIZED-DO-NOTATION.md`
- `CHANGELOG.md`

## Verification Evidence

- [x] SPEC-054 created.
- [x] PLAN-101 created.
- [x] TASK-746 through TASK-753 files created.
- [x] SPEC index updated.
- [x] PLAN-INDEX Phase 105 updated.
- [x] CHANGELOG updated.
- [x] DESIGN-031 points to SPEC-054/PLAN-101.

## Dependencies for Next Task

Required by:

- TASK-747: surface AST/parser substrate.
- TASK-748: do-target kinding and dictionary resolution.
