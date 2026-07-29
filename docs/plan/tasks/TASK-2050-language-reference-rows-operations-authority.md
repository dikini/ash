# TASK-2050: Language Reference for Rows, Operations, Resources, Roles, and Authority

**Status:** Complete
**Phase:** [PLAN-206](../PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md)
**Depends on:** TASK-2045
**Owned feature IDs:** LANG-011, LANG-012, LANG-021.
**Semantic task classification:** non-semantic-workflow-enforcement

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

- [x] Every authority claim names its separate admission evidence.
- [x] Rows/aliases/groups/resource/role examples are status-labelled.
- [x] Excluded declaration forms have no copyable examples.
- [x] Removed forms never appear as current examples; indexes/changelog/PLAN-INDEX are updated.

## Completion evidence

The required row-to-authority non-grant checklist was applied before authoring:

1. A parsed/lowered row records a requirement and does not create a provider, resource
   initializer, admitted role, policy discharge, handler frame, or runtime module.
2. An operation requirement is checked only against separately registered provider authority.
3. A resource requirement is checked only against a separately selected initializer.
4. A role requirement is checked only against a role already admitted on the request.
5. Policy/channel/process/failure/group requirements reject as unsupported, and evidence always
   rejects in the current checker, including a recognized family, because no record/discharge
   strategy route is implemented; none of these items is an authority grant.

The complete implementation-backed chapter is [Effects](../../reference/language/effects/index.md):
[rows, aliases, groups, and operations](../../reference/language/effects/rows-aliases-groups-and-operations.md)
and [resources, roles, and authority boundaries](../../reference/language/effects/resources-roles-and-authority-boundaries.md).

Verification completed against the reviewed implementation revision:

- `cargo test -p ash-parser --test task_2001_effect_alias_group_surface --test task_1809_computation_row_parser --test phase_101_resource_binding_parser` — 15 passed.
- `cargo test -p ash-parser --lib parse_visibility::tests::` — 8 passed, including `pub`,
  `pub(crate)`, `pub(super)`, `pub(self)`, and `pub(in path)` parsing.
- `cargo test -p ash-parser test_parse_inline_module_with_role_definition` — 1 selected parser
  unit test passed.
- `cargo test -p ash-typeck --test task_1814_row_cross_boundary_non_authority --test task_2001_local_effect_row_resolution --test task_2013_handler_row_typing` — 26 passed.
- `cargo test -p ash-engine --test task_1822_row_authority_neutrality --test task_1829_1830_1831_1832_1833_row_admission --test task_1896_1897_evidence_contract_discharge --test task_2011_declared_concrete_operation_source_call` — 31 passed.
- Rendered 2 `ebnf` fences with `/home/dikini/Projects/railroad/src/ebnf.js::compileEbnf`
  and 1 `sequent` fence with `/home/dikini/Projects/sequent-md/packages/core/src/index.js::render`;
  the renderer returned no diagnostics.
- `python3 tools/docs/validate_orientation_indexes.py --self-test`,
  `bash scripts/check-docs-gate.sh`, and `git diff --check` passed; the documentation gate checked
  1,903 links with none missing.
