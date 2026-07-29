# TASK-2050: Language Reference for Rows, Operations, Resources, Roles, and Authority

**Status:** Planned
**Phase:** [PLAN-206](../PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md)
**Depends on:** TASK-2045
**Owned feature IDs:** LANG-011, LANG-012, LANG-021.

## Description

Document effect-row annotations, aliases/groups, declared operation identities, resources, and
roles, with the mandatory distinction between static requirements and runtime authority.

## Requirements

- Create `docs/reference/language/effects/index.md`, `rows-aliases-groups-and-operations.md`, and
  `resources-roles-and-authority-boundaries.md`.
- State that rows, imported summaries, names, roles, and resource metadata do not install provider
  frames or grant admission/runtime authority.
- Document the whole computation-row grammar, not merely aliases/operations: policy, channel,
  process, fail, evidence, group, whole-row variables/tails, resource/channel modes, role,
  resource, and operation paths. Describe each family's parser/typecheck/Core/runtime status.
- Describe aliases/groups and operation identities only to their current parser/typeck/Core
  transport extent; identify all missing runtime clauses.
- Exclude top-level capability/policy source declarations and direct capability/provider grant
  forms unless a fresh parser branch proves their reintroduction.

## Handoffs and dependencies

- **Consumes:** parser effect/resource/role branches, `ash-typeck` row paths, Core row typing, and
  Engine row admission.
- **Evidence:** `cargo test -p ash-parser --test task_2001_effect_alias_group_surface`, `--test
  task_1809_computation_row_parser`; `cargo test -p ash-typeck --test
  task_1814_row_cross_boundary_non_authority`, `--test task_2001_local_effect_row_resolution`;
  `cargo test -p ash-engine --test task_1822_row_authority_neutrality`, `--test
  task_1829_1830_1831_1832_1833_row_admission`; `cargo test -p ash-typeck --test
  task_2013_handler_row_typing task_2013_every_nonempty_or_open_residual_keeps_resume_affine`.
- **Produces:** authority terminology and row evidence consumed by TASK-2051/2052.
- **Non-goals:** automatically admitted effects, row-derived frames, tower carrier APIs, or direct
  capability/policy declaration examples.

## TDD and verification steps

1. Write a row-to-authority non-grant checklist before prose.
2. Confirm positive row parsing and negative authority/admission controls with named tests.
3. Render grammar/semantic fences and classify all remaining clauses partial.

## Completion checklist

- [ ] Every authority claim names its separate admission evidence.
- [ ] Rows/aliases/groups/resource/role examples are status-labelled.
- [ ] Excluded declaration forms have no copyable examples.
- [ ] Removed forms never appear as current examples; indexes/changelog/PLAN-INDEX are updated.
