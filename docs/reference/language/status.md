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

## Coverage boundary

The complete manual contains shared navigation/conventions, the TASK-2046
[lexical-and-modules chapter](lexical-and-modules/index.md), the TASK-2047
[forms chapter](forms/index.md), the TASK-2048/TASK-2049 [types chapter](types/index.md), the
TASK-2050/TASK-2051 [effects chapter](effects/index.md), and the TASK-2052
[execution chapter](execution/index.md). Those chapters make their own bounded claims for LANG-001
through LANG-016 and LANG-019 through LANG-023; they do not establish aggregate support for
general handler, effect, or client runtime routes.

TASK-2053 refreshed and documented LANG-017/LANG-018, including the `std/src` corpus, import
limits, and diagnostic boundaries. TASK-2054 completed the manual-wide closeout: every page is
reachable from the index, the manual has 16 EBNF and 14 sequent fences, and its 30 fences have
been validated by the task-owned helper and the respective external tools. This is validation of
the documentation artifacts, not evidence that every partial language route executes.

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

`implemented` is not shorthand for parser acceptance. A missing, rejected, or bounded applicable
layer remains a limitation and normally makes the implementation status `partial`.

## Planned and target-only register

**No target-only/planned source-language feature is known at this audited revision.** `planned` is
reserved for a feature explicitly described by a target specification but absent from the live
source route. The completed AUDIT-206 census found no such feature to add to this manual.

This does not make incomplete current routes future features: a live spelling with a missing or
bounded layer remains `partial`, `below_spec`, or `closed` as its page records. Removed syntax is
`excluded`, and a Rust/Core/CPS carrier with no source spelling is `internal-only`. A future
target-only feature must be added to this register with its target authority and an explicit
`planned` label before it is described in the manual.

## Exclusions and limitations

Removed workflow/tower source syntax is excluded from current examples and chapter claims. An
internal Core/CPS or Rust carrier is `internal-only` until a supported source spelling and its
route are evidenced. Planned target material is labelled `planned`; it is never presented as a
current language feature.

For the current exclusion register and feature census, see
[AUDIT-206](../../plan/audits/AUDIT-206-implementation-backed-language-reference.md#exclusion-register).
