---
status: drafting
created: 2026-03-30
last-revised: 2026-04-05
related-plan-tasks: [TASK-394, TASK-395, TASK-396]
tags: [alignment, surface, ir, semantics, interpreter, consolidation]
---

# MCE-007: Full Layer Alignment

## Problem Statement

All layers of Ash must remain consistent:

surface syntax → canonical IR → big-step semantics → small-step semantics → interpreter/runtime.

This exploration is the full-stack consolidation point. It now treats:

- [MCE-004](MCE-004-BIG-STEP-ALIGNMENT.md) as resolved for the surface → IR → big-step side; and
- [MCE-005](MCE-005-SMALL-STEP.md) as materially defined for the canonical small-step backbone.

The heaviest remaining dependency is therefore no longer “what is the small-step backbone?” but “how is that backbone realized by the interpreter/runtime?” — the central concern of [MCE-006](MCE-006-SMALL-STEP-IR.md).

## Scope

In scope:

- all five layers: Surface, IR, Big-step, Small-step, Interpreter;
- cross-layer consistency checks;
- documentation of correspondence obligations between adjacent layers;
- identification of remaining drift after MCE-004 and Phase 61.

Out of scope:

- new feature design;
- optimization work;
- low-level runtime implementation changes.

Related but separate:

- [MCE-004](MCE-004-BIG-STEP-ALIGNMENT.md): accepted prerequisite for surface → IR → big-step
- [MCE-005](MCE-005-SMALL-STEP.md): accepted small-step planning/design backbone
- [MCE-006](MCE-006-SMALL-STEP-IR.md): remaining runtime/interpreter alignment prerequisite

## The Five Layers

```text
┌─────────────────────────────────────┐
│ Layer 1: Surface Syntax             │ User-facing Ash language
└──────────────┬──────────────────────┘
               │ lowering
               ▼
┌─────────────────────────────────────┐
│ Layer 2: Canonical IR               │ `SPEC-001`
└──────────────┬──────────────────────┘
               │ big-step meaning
               ▼
┌─────────────────────────────────────┐
│ Layer 3: Big-Step Semantics         │ `SPEC-004`
└──────────────┬──────────────────────┘
               │ small-step refinement
               ▼
┌─────────────────────────────────────┐
│ Layer 4: Small-Step Semantics       │ MCE-005 backbone
└──────────────┬──────────────────────┘
               │ runtime realization
               ▼
┌─────────────────────────────────────┐
│ Layer 5: Interpreter / Runtime      │ executable implementation
└─────────────────────────────────────┘
```

## Alignment Verification Obligations

For each canonical construct, MCE-007 ultimately needs:

1. Surface → IR: lowering contract defined.
2. IR → Big-step: big-step semantic rule defined.
3. Big-step → Small-step: correspondence argument defined.
4. Small-step → Interpreter: runtime realization argument defined.

## Current Alignment State

### Resolved inputs

The following are already fixed and should not be reopened here:

- `Workflow::Seq` remains primitive.
- `Expr::Match` remains primitive, and `if let` lowers to `Expr::Match`.
- `Par` big-step aggregation is helper-backed with successful branch-effect join.
- Spawn completion seals the child workflow's own authoritative terminal state in `CompletionPayload`.
- Small-step is workflow-first, uses the accepted configuration/label split from MCE-005, keeps expressions/patterns atomic in v1, and distinguishes blocked/suspended states from stuckness.

### Updated verification matrix

| Construct family | Surface→IR | IR→Big | Big→Small | Small→Interp | Status |
|---|---|---|---|---|---|
| Sequencing / binding / branching | ✅ | ✅ | Backbone fixed; per-form correspondence still to package | ❓ | Partial |
| Pattern-driven control | ✅ | ✅ | Backbone fixed; correspondence still to package | ❓ | Partial |
| Receive / blocking behavior | ✅ | ✅ | Backbone fixed; blocked-vs-stuck settled | ❓ | Partial |
| Parallel composition | ✅ | ✅ | Backbone fixed; interleaving + helper aggregation settled | ❓ | Partial |
| Capability / policy / obligation workflows | ✅ | ✅ | Backbone fixed; detailed correspondence still to package | ❓ | Partial |
| Spawn / completion observation contracts | ✅ | ✅ | Helper/runtime contract fixed as input | ❓ | Partial |

The remaining `❓` weight is concentrated in runtime/interpreter realization, not in the existence of a small-step backbone.

## Remaining Full-Stack Gaps

### Gap 1: Packaged big-step ↔ small-step correspondence

After Phase 61, the backbone is no longer the blocker. The remaining semantic packaging work is to make the correspondence explicit enough that later readers can see how:

- terminal outcomes are reconstructed from terminal configurations;
- traces/effects/provenance/obligations are preserved across repeated steps;
- blocked/suspended states refine, rather than contradict, the `SPEC-004` worldview.

This is the closeout space tracked by Phase 61 correspondence work and the relevant sections of MCE-005.

### Gap 2: Small-step ↔ interpreter correspondence

This is now the primary open dependency.

MCE-007 still needs MCE-006 to explain:

- how workflow configurations map to runtime structures;
- how `Par` interleaving is realized;
- how blocked states are represented operationally;
- how terminal observables are preserved by the interpreter/runtime.

### Gap 3: Ongoing drift prevention

Once MCE-006 matures, MCE-007 should also define how future changes are checked against the full five-layer stack so the repo does not drift back into layer disagreement.

## Deliverables

MCE-007 should eventually produce:

1. a construct-by-construct five-layer alignment matrix;
2. explicit correspondence notes for the remaining nontrivial constructs;
3. a runtime-realization summary consuming MCE-006;
4. a drift-prevention checklist for future language/runtime changes.

## Dependencies

This exploration depends on:

- MCE-002 (IR inventory known)
- MCE-004 (surface → IR → big-step alignment accepted)
- MCE-005 (small-step backbone accepted in Phase 61)
- MCE-006 (runtime/interpreter alignment still needed)

Recommendation: keep MCE-007 drafting until MCE-006 matures enough to close the interpreter side of the matrix.

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-30 | Exploration created | Need a complete alignment view |
| 2026-04-05 | Reframed after Phase 61 | MCE-005 is now materially defined; remaining dependency weight is on MCE-006 and runtime/interpreter correspondence |
