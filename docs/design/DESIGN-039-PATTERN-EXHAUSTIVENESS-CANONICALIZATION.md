# DESIGN-039: Pattern/Exhaustiveness Canonicalization

**Status:** Promoted to [SPEC-068](../spec/SPEC-068-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md) / [PLAN-117](../plan/PLAN-117-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md)
**Date:** 2026-05-14
**Origin:** [DESIGN-034 §16.9](DESIGN-034-TOTAL-TYPE-COMPUTATION.md#169-cross-packet-implementation-gaps-to-plan-explicitly), [TASK-890](../plan/tasks/TASK-890-pattern-exhaustiveness-alias-canonicalization-packet.md)
**Related:** [SPEC-057](../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md), [SPEC-058](../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md), [SPEC-059](../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md), [SPEC-060](../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md), [SPEC-064](../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)

## Summary

SPEC-058 and SPEC-060 gave Ash canonical type-expression and normalization/equality boundaries. They intentionally did not replace every consumer with canonical equality. Pattern checking and exhaustiveness are sensitive consumers because constructor resolution, ADT identity, alias transparency, projection rigidity, and diagnostics all affect user-visible behavior.

This design chooses an audit-first rollout. Pattern/exhaustiveness may consume a narrower canonicalization API than general definitional equality, and it must not solve under neutral computation heads merely because equality can normalize aliases at other boundaries.

## Core decisions

- Pattern checking may canonicalize transparent aliases and selected reducible projections only through a pattern-specific API.
- Exhaustiveness must use the same constructor universe as pattern typing after canonicalization.
- Neutral/stuck computation heads and rigid projections do not become matchable ADT constructors.
- Alias-equivalent positive cases need paired negative leakage tests for same visible names from unrelated modules.
- Existing ADT constructor resolution and current pattern diagnostics remain the baseline until SPEC-068 explicitly changes them.


## Non-goals

- Do not reopen SPEC-057 through SPEC-064 substrate decisions unless the new spec explicitly names an incompatibility.
- Do not encode type-level computation as ordinary `Type::Constructor` nodes.
- Do not broaden `do` notation or pattern behavior before the owning implementation phase lands.

## Handoff

The normative implementation contract is in [SPEC-068](../spec/SPEC-068-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md). The task order is in [PLAN-117](../plan/PLAN-117-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md).
