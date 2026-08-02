---
id: language.reference.status
title: Language Reference Status and Coverage
kind: status-map
status: current
audience: [human, agent]
reviewed_revision: 423f603c
evidence: tested
refresh_trigger: ["docs/reference/language/**", "docs/plan/audits/AUDIT-206-implementation-backed-language-reference.md"]
---

# Language Reference Status and Coverage

[Reference index](index.md) · [Source of truth](source-of-truth.md) ·
[Authoring conventions](conventions.md)

## Page status

**Reviewed implementation revision:** `423f603c`. The closeout evidence below was produced in
the current uncommitted workspace and deliberately has no invented documentation commit ID.

**Implementation:** not applicable for this status map.
**Evidence:** [AUDIT-206](../../plan/audits/AUDIT-206-implementation-backed-language-reference.md)
and repository documentation-gate navigation checks.
**Parity:** not applicable for this status map.

## Coverage

The manual has shared navigation and conventions, then the TASK-2046
[lexical-and-modules chapter](lexical-and-modules/index.md), the TASK-2047
[forms chapter](forms/index.md), the TASK-2048/TASK-2049 [types chapter](types/index.md), the
TASK-2050/TASK-2051 [effects chapter](effects/index.md), and the TASK-2052
[execution chapter](execution/index.md), and the TASK-2053 [library chapter](library/index.md).
Those chapters make their own bounded claims for LANG-001 through LANG-016 and LANG-019 through
LANG-023.

TASK-2053 covers LANG-017/LANG-018: the `std/src` corpus, imports, and diagnostics. TASK-2054
checked navigation and all 30 grammar and rule fences. Those checks validate the documentation;
they do not mean that every form runs.

## Closeout evidence

- `node --test tools/docs/validate_language_reference_fences.test.mjs`: 23/23 tests passed.
- The manual helper validated 30 fences: 16 EBNF and 14 sequent.
- `/home/dikini/Projects/railroad`: `npm run check` passed 38/38 and `npm run build` passed; its
  two vendor `-0` warnings were non-fatal.
- `/home/dikini/Projects/sequent-md`: `npm test` passed 26/26 and `npm run build` passed.
- Repository orientation self-test, documentation gate (2,032 links, zero missing), and
  `git diff --check` passed. The repository gate does not replace either external fence check.

## Required status vocabulary for a feature page

Every feature page records these axes independently:

| Axis | Required values | Meaning |
|---|---|---|
| Grammar | `accepted`, `rejected`, `parser-only` | Source parser acceptance, not an AST carrier alone. |
| Static | `checked`, `rejected-after-parse`, `partial`, `not-applicable` | Name/type/static route. |
| Lowering | `lowered`, `bounded-only`, `rejected`, `not-applicable` | Source-to-Core/CPS realization. |
| Admission/runtime | `admitted-executed`, `fixture-bounded`, `closed`, `not-applicable` | Engine admission and execution evidence. |
| Implementation | `implemented`, `partial`, `planned`, `excluded`, `internal-only` | Claim scope across the applicable layers. |
| Evidence | `proved`, `tested`, `none` | Strength of the cited evidence. |
| Parity | `matches_spec`, `below_spec`, `not-applicable` | Relationship to a relevant target rule. |

`implemented` means more than parser acceptance. If a needed step is missing or rejected, use
`partial`.

## Planned and target-only register

**No target-only/planned source-language feature is known at this audited revision.** `planned` is
reserved for a feature explicitly described by a target specification but absent from the live
source route. The completed AUDIT-206 census found no such feature to add to this manual.

An incomplete current feature is still current. Mark it `partial`, `below_spec`, or `closed` as
needed. Mark removed syntax `excluded`, and implementation-only Rust/Core/CPS carriers
`internal-only`. Add a target-only feature here before describing it in the manual.

## Exclusions and limitations

Removed workflow/tower source syntax is excluded from current examples and chapter claims. An
internal Core/CPS or Rust carrier is `internal-only` until a supported source spelling and its
route are evidenced. Planned target material is labelled `planned`; it is never presented as a
current language feature.

For the current exclusion register and feature census, see
[AUDIT-206](../../plan/audits/AUDIT-206-implementation-backed-language-reference.md#exclusion-register).
