# TASK-183: Define Formalization Boundary and Proof Targets

## Status: ✅ Complete

## Description

Write a formalization-boundary note that tells future Lean work exactly which documents are
normative, which artifacts are migration-only, and which proof and bisimulation targets the
language definition is expected to support.

## Specification Reference

- SPEC-001: IR
- SPEC-003: Type System
- SPEC-004: Operational Semantics
- SPEC-020: ADT Types
- SPEC-021: Runtime Observable Behavior

## Requirements

### Functional Requirements

1. Define the normative document set for future Lean work
2. Separate canonical specs from migration/reference/task artifacts
3. List initial proof targets and bisimulation targets
4. State the intended relationship between Rust and Lean implementations
5. Treat recoverable failure as explicit `Result` dataflow, not exceptional `catch` semantics

## TDD Evidence

### Red

Before this task, the repository had no single formalization-boundary note. Lean-oriented guidance
was split between the hardening plan, the runtime/observable references, and an older Lean
reference interpreter document, so the canonical proof corpus was not mechanically obvious.

### Green

The formalization boundary is now explicit:

- the canonical Lean/Rust proof corpus is [SPEC-001](../../spec/SPEC-001-IR.md),
  [SPEC-003](../../spec/SPEC-003-TYPE-SYSTEM.md), [SPEC-004](../../spec/SPEC-004-SEMANTICS.md),
  [SPEC-020](../../spec/SPEC-020-ADT-TYPES.md), and
  [SPEC-021](../../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)
- [docs/reference/formalization-boundary.md](../../reference/formalization-boundary.md) separates
  the canonical semantic corpus, authoritative source/handoff contracts, and historical artifacts
- [docs/spec/SPEC-021-LEAN-REFERENCE.md](../../spec/SPEC-021-LEAN-REFERENCE.md) is now explicitly
  marked legacy instead of masquerading as a competing current spec
- recoverable failure is explicitly `Result`-based; `catch` is not part of the canonical contract
- proof and bisimulation targets are listed in the boundary note rather than being left implicit

## Files

- Create: `docs/reference/formalization-boundary.md`
- Modify: `docs/spec/SPEC-021-LEAN-REFERENCE.md`
- Modify: `docs/plan/2026-03-19-spec-hardening-design.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for:
- unclear normative document set,
- unclear proof targets,
- unclear Rust-vs-Lean contract relationship.

### Step 2: Verify RED

Expected failure conditions:
- no formalization-boundary note currently exists.

### Step 3: Implement the minimal reference (Green)

Write only the boundary and target-property note.

### Step 4: Verify GREEN

Expected pass conditions:
- the normative document set and proof targets are explicit.

### Step 5: Commit

```bash
git add docs/reference/formalization-boundary.md docs/spec/SPEC-021-LEAN-REFERENCE.md docs/plan/2026-03-19-spec-hardening-design.md docs/plan/PLAN-INDEX.md docs/plan/tasks/TASK-183-define-formalization-boundary-and-proof-targets.md CHANGELOG.md
git commit -m "docs: define formalization boundary and proof targets"
```

## Completion Checklist

- [x] formalization boundary documented
- [x] proof targets documented
- [x] Rust-vs-Lean contract relationship documented
- [x] `CHANGELOG.md` updated

## Non-goals

- No Lean implementation
- No proofs

## Dependencies

- Depends on: TASK-177, TASK-178, TASK-182, TASK-185
- Blocks: TASK-184
