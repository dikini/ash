# TASK-646: Ash Wiki Metadata Carrier Schema

## Status: ✅ Complete

## Description

Define the concrete metadata carrier format for the Ash wiki Phase 1 rollout. This task resolves the open question from the initial Ash wiki packet by choosing a hybrid frontmatter-plus-registry model, documenting the normalized logical schema, defining validation rules, and updating the wiki spec and implementation plan to use a concrete task identifier instead of placeholders.

## Specification Reference

- [SPEC-045: Ash Wiki Knowledge Substrate](../../spec/SPEC-045-ASH-WIKI.md) — §8 Required Metadata Contract
- [Ash Wiki Metadata Schema](../../reference/ash-wiki-metadata-schema.md)
- [2026-04-20 Ash Wiki Implementation Plan](../../plans/2026-04-20-ash-wiki-implementation-plan.md) — Phase 1 / Task 2

## Dependencies

- ✅ TASK-645: Ash Wiki Concept Packet

## Requirements

1. Decide the metadata carrier representation for the Ash wiki rollout.
2. Define the normalized logical schema independent of carrier.
3. Define legal state combinations and minimum validation rules.
4. Describe the adoption policy for legacy docs versus newly managed wiki artifacts.
5. Update the spec/plan corpus so the Phase 1 metadata task is concretely named and traceable.

## Completion Checklist

- [x] Document the carrier decision in `docs/reference/ash-wiki-metadata-schema.md`
- [x] Patch `SPEC-045` to name the concrete carrier model and schema-reference relationship
- [x] Update the implementation plan to replace placeholder references with `TASK-646`
- [x] Add explicit task-level traceability for the metadata-schema work
- [x] Update `CHANGELOG.md`

## Notes

This is a docs/planning task. No runtime or code substrate is implemented here. The output is the authoritative Phase 1 metadata-carrier decision that later registry/lint/query tasks will build on.
