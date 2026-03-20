# TASK-190: Synthesize Runtime-Reasoner Spec Delta Program

## Status: ✅ Complete

## Description

Merge the frozen separation rules and the two audit reports into one ordered spec-delta program
that states what to revise, what to leave untouched, and how the design-review outcome affects the
existing convergence roadmap.

## Specification Reference

- `docs/design/RUNTIME_REASONER_INTERACTION_MODEL.md`
- SPEC-001: IR
- SPEC-002: Surface Language
- SPEC-004: Operational Semantics
- SPEC-017: Capability Integration
- SPEC-018: Capability Runtime Verification Matrix
- SPEC-021: Runtime Observable Behavior

## Plan Reference

- `docs/plan/2026-03-20-runtime-reasoner-design-review-plan.md`

## Requirements

### Functional Requirements

1. Merge findings from TASK-187, TASK-188, and TASK-189
2. Produce one prioritized delta list by document and concern class
3. Distinguish framing-only deltas from normative contract deltas and new-document needs
4. State how the result affects existing planned convergence tasks and phases

## Files

- Create: `docs/plan/2026-03-20-runtime-reasoner-spec-delta-program.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for:
- audit outputs not yet synthesized,
- no single ordered delta list by document,
- unclear impact on existing convergence phases.

### Step 2: Verify RED

Expected failure conditions:
- the repository still lacks one authoritative review output explaining what spec work follows from the design-review phase.

Observed before implementation:
- the repository had the frozen separation rules and two completed audit reports, but no single
  document ordered the resulting deltas by document, concern class, and downstream planning impact.
- it was still unclear which follow-up work should be framing-only, which should be deferred, and
  which existing convergence phases were actually unaffected.

### Step 3: Implement the synthesis plan (Green)

Create only the synthesis and downstream planning output needed to guide later spec work.

### Step 4: Verify GREEN

Expected pass conditions:
- deltas are prioritized and scoped,
- runtime-only concerns are explicitly protected from interaction-layer overloading,
- downstream task impact is explicit.

Verified after implementation:
- [2026-03-20-runtime-reasoner-spec-delta-program.md](../2026-03-20-runtime-reasoner-spec-delta-program.md)
  now merges TASK-187 through TASK-189 into one prioritized delta program.
- the synthesis explicitly protects monitor views, `exposes`, workflow observability, capability
  verification, and approval routing as runtime-only concerns.
- the synthesis defines a concrete next-step ordering: interaction contract first, then minimal
  `SPEC-004` framing, then terminology tightening, then any later surface-language guidance.
- the impact on existing convergence phases is now explicit: current parser/runtime convergence work
  remains unchanged unless it begins touching runtime-to-reasoner interaction semantics.

### Step 5: Commit

```bash
git add docs/plan/2026-03-20-runtime-reasoner-spec-delta-program.md docs/plan/PLAN-INDEX.md CHANGELOG.md docs/plan/tasks/TASK-190-synthesize-runtime-reasoner-spec-delta-program.md
git commit -m "docs: synthesize runtime-reasoner spec delta program"
```

## Completion Checklist

- [x] audits synthesized
- [x] document-by-document delta list produced
- [x] concern classes preserved
- [x] convergence impact documented
- [x] `CHANGELOG.md` updated

## Non-goals

- No normative spec edits
- No runtime implementation changes
- No parser or interpreter convergence work

## Dependencies

- Depends on: TASK-187, TASK-188, TASK-189
- Blocks: follow-up spec revision tasks to be introduced after synthesis
