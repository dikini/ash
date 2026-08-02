---
id: language.reference.effects
title: Effects, Rows, and Authority Boundaries
kind: chapter-index
status: partial
audience: [human, agent]
reviewed_revision: 423f603c
evidence: tested
refresh_trigger: ["crates/ash-parser/src/**", "crates/ash-typeck/src/**", "crates/ash-engine/src/row_admission.rs", "crates/ash-engine/src/**"]
---

# Effects, Rows, and Authority Boundaries

[Language reference](../index.md) · [Status and coverage](../status.md) ·
[Source of truth](../source-of-truth.md)

## Page status

**Reviewed revision:** `423f603c` (refresh AUDIT-206 rows LANG-011 through LANG-014, LANG-021,
and LANG-022 before changing a current-language claim).

**Implementation:** partial. Computation rows and their declarations are accepted and transported
as checked metadata, source handlers/failure/do forms have deliberately bounded routes, and a row
still describes requirements rather than creating runtime authority or a general executable effect
route.
**Evidence:** tested. The focused parser, typechecker, and Engine tests named on the child pages
cover acceptance, metadata transport, sealed handler evidence, and fail-closed admission controls.
**Parity:** below_spec. This chapter documents the implementation boundary rather than extending
older authority-oriented descriptions.

| Topic | Grammar | Static | Lowering | Admission/runtime | Implementation | Evidence | Parity |
|---|---|---|---|---|---|---|---|
| Computation rows and open tails | accepted | partial | lowered | closed | partial | tested | below_spec |
| Effect aliases and groups | accepted | checked | lowered | closed | partial | tested | below_spec |
| Declared concrete operation identities | accepted | checked | lowered | closed | partial | tested | below_spec |
| Resource-type declarations | accepted | checked | not-applicable | closed | partial | tested | below_spec |
| Role declarations | accepted | partial | not-applicable | closed | partial | tested | below_spec |
| Handlers and `handle … with` | accepted | partial | bounded-only | fixture-bounded | partial | tested | below_spec |
| Scoped failure | accepted | checked | lowered | closed | partial | tested | below_spec |
| Exact ambient `do` entry fixture | accepted | checked | lowered | fixture-bounded | partial | tested | below_spec |
| Ambient binding, target-annotated `do`, and comprehensions | accepted | partial | bounded-only | closed | partial | tested | below_spec |

`closed` does not mean that a row is ignored. The Engine derives an admission requirement from
lowered row metadata and checks it against authority already selected by the host/request. A
matching provider, resource initializer, or admitted role can satisfy that one check, but the
current source route then remains subject to the separate checked-Core/CPS admission boundary.
It is not a general effect executor.

## In this chapter

- [Rows, aliases, groups, and operations](rows-aliases-groups-and-operations.md) — row syntax,
  metadata transport, named aliases/groups, and concrete operation identities.
- [Resources, roles, and authority boundaries](resources-roles-and-authority-boundaries.md) —
  resource types, role metadata, and the independent host-side authority needed for admission.
- [Handlers, scoped failure, and `do`](handlers-failure-and-do.md) — canonical source handlers,
  fixture-bounded handler admission, legacy failure carriers, and ambient versus typed `do`.
- [Bracket comprehensions](comprehensions.md) — qualifier syntax and the explicit-target static
  elaboration boundary.

## Scope boundary

The source parser is `crates/ash-parser/src/parse_module.rs::{parse_computation_row_from_open_brace,
parse_effect_row_definition,parse_resource_type_definition,parse_role_definition}`. The checker
registers resource types and effect-row summaries in `ash-typeck`; the Engine maps a lowered
`CoreRow` to `RowAdmissionRequirement` values in `ash-engine/src/row_admission.rs`.

That mapping is deliberately non-granting. Rows, imported summaries, aliases, groups, resource
metadata, and role names do not register a provider, select a resource initializer, attach an
admitted role, install a handler frame, or invoke a host hook. The handler chapter documents only
sealed fixture admission, and the entry/admission route remains separately owned by TASK-2052.

The active source type spelling `capability Name` is covered by the
[types chapter](../types/data-newtypes-and-callables.md#capability-name-is-a-source-type-not-a-declaration).
It is not a substitute for an authority declaration or a provider grant.

## Related evidence

- [AUDIT-206 implementation census](../../../plan/audits/AUDIT-206-implementation-backed-language-reference.md#implementation-census)
- [TASK-2050](../../../plan/tasks/TASK-2050-language-reference-rows-operations-authority.md)
- `crates/ash-engine/src/row_admission.rs`
