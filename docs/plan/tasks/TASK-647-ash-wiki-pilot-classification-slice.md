# TASK-647: Ash Wiki Pilot Classification Slice

## Status: 📝 Planned

## Description

Validate the Ash wiki metadata model against a real pilot slice of the Ash corpus. This task classifies one representative document family using the new authority/status/health model, records explicit supersession where applicable, and produces the first pilot authority/supersession views. The goal is to test whether the schema is practical before broader rollout.

## Specification Reference

- [SPEC-045: Ash Wiki Knowledge Substrate](../../spec/SPEC-045-ASH-WIKI.md)
- [Ash Wiki Metadata Schema](../../reference/ash-wiki-metadata-schema.md)
- [2026-04-20 Ash Wiki Implementation Plan](../../plans/2026-04-20-ash-wiki-implementation-plan.md) — Phase 1 / Task 3

## Dependencies

- ✅ TASK-645: Ash Wiki Concept Packet
- ✅ TASK-646: Ash Wiki Metadata Carrier Schema

## Requirements

1. Select a representative pilot slice of the Ash corpus.
2. Assign `type`, `authority`, `status`, and `health` across that slice.
3. Record explicit supersession or state why supersession is not applicable.
4. Produce at least one human-readable authority map and one supersession map for the slice.
5. Record friction points or schema ambiguities discovered during classification.

## TDD Steps

### Step 1: Choose pilot scope (Red)

Pick a slice with enough variation to stress the schema. Preferred candidates:
- Ash wiki / AI-native workflow future notes
- tooling-facing specs/designs/plans
- one implementation-heavy subsystem with historical drift concerns

### Step 2: Create pilot maps (Green)

Create:
- `docs/wiki/indexes/pilot-authority-map.md`
- `docs/wiki/indexes/pilot-supersession-map.md`

Populate them with explicit per-artifact classifications and notes.

### Step 3: Verify (Green)

Check that:
- no artifact in the pilot slice lacks classification
- stale vs historical is not conflated
- supersession claims are explicit and scoped
- friction points are recorded for follow-on schema refinement

## Verification Steps

- [ ] Pilot slice is explicitly named and bounded
- [ ] Each pilot artifact has a normalized classification
- [ ] Partial supersession, if any, is scoped explicitly
- [ ] Friction points are captured for follow-on work
- [ ] `CHANGELOG.md` updated when the task is executed

## Dependencies for Next Task

This task outputs:
- a validated pilot slice for the metadata model
- first static authority/supersession views
- a concrete list of schema pain points for registry/lint work

Required by:
- Future registry generation and audit/lint tasks in the Ash wiki rollout
