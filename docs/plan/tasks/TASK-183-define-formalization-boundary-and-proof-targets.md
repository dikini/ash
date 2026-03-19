# TASK-183: Define Formalization Boundary and Proof Targets

## Status: 📝 Planned

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

## Files

- Create: `docs/reference/formalization-boundary.md`
- Modify: `docs/plan/2026-03-19-spec-hardening-design.md`
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
git add docs/reference/formalization-boundary.md docs/plan/2026-03-19-spec-hardening-design.md CHANGELOG.md
git commit -m "docs: define formalization boundary and proof targets"
```

## Completion Checklist

- [ ] formalization boundary documented
- [ ] proof targets documented
- [ ] Rust-vs-Lean contract relationship documented
- [ ] `CHANGELOG.md` updated

## Non-goals

- No Lean implementation
- No proofs

## Dependencies

- Depends on: TASK-177, TASK-178, TASK-182, TASK-185
- Blocks: TASK-184
