# Ash Language Reference

## Manual status

**Manual status:** Complete. TASK-2054 integrated the shared skeleton and all six chapters,
then validated navigation, documentation gates, every manual EBNF/sequent fence, and the external
railroad and sequent renderers. No chapter inherits support claims from this navigation page.

**Reviewed implementation revision:** `423f603c`. The feature evidence was refreshed in the
current workspace during closeout; the documentation closeout itself is not assigned a commit
identifier until it is committed.

**Implementation:** not applicable for this navigation page.
**Evidence:** tested navigation and documentation-gate links.
**Parity:** not applicable for this navigation page.

## Start here

- [Status and coverage](status.md) — manual-wide status vocabulary, coverage boundary, and
  limitations.
- [Source of truth](source-of-truth.md) — evidence order, implementation routes, and the
  placement decision.
- [Authoring conventions](conventions.md) — required evidence fields, examples, EBNF, and
  sequent rules for feature pages.

## Chapters

- [Lexical structure and modules](lexical-and-modules/index.md) — current source-file, module,
  import-route, notation, macro, and operator-section boundaries (TASK-2046).
- [Forms: declarations and expressions](forms/index.md) — implementation-backed function,
  binding, control-flow, pattern, contract, and authoring-only law/proof boundaries (TASK-2047).
- [Types, callables, interfaces, and implementations](types/index.md) — ordinary types, nominal
  wrappers, callable and capability type spellings, bounded generic/interface evidence, and
  type-level domain/function/family/proposition boundaries (TASK-2048, TASK-2049).
- [Effects, rows, and authority boundaries](effects/index.md) — computation-row requirements,
  aliases/groups, declared operation identities, resource/role metadata, and the rule that none
  of them grants runtime authority; canonical source handlers, scoped failure, `do`, and
  comprehensions retain their separate bounded routes (TASK-2050, TASK-2051).
- [Entry, admission, clients, and terminal results](execution/index.md) — bounded `fn main`
  admission, Engine-issued requests, selected CLI/test/REPL/daemon routes, and normalized
  terminal observations without a direct-evaluator fallback (TASK-2052).
- [Library and diagnostics](library/index.md) — the 59-file `std/src` parser/static corpus,
  ordinary versus narrow runtime-entry imports, selected `time::sleep` evidence, and diagnostic/
  terminal limitations without a blanket standard-library runtime claim (TASK-2053).

## Scope

This is a separate implementation-backed language manual, authorized by
[SPEC-071 §3.1](../../spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md#31-scoped-implementation-backed-language-manual-exception).
It is not part of the top-level [`reference/`](../../../reference/INDEX.md) corpus and is not
validated by that corpus's frontmatter checker.

The manual is rooted in the live implementation and executable evidence recorded by
[PLAN-206](../../plan/PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md) and
[AUDIT-206](../../plan/audits/AUDIT-206-implementation-backed-language-reference.md). Each
chapter establishes its own claims from the rows it refreshed; the closeout validates the manual
as navigation and evidence infrastructure, not as a blanket support claim.

## Current-example boundary

Do not use deprecated workflow/tower syntax as a copyable current-language example in this manual.
Historical material remains preserved in its original locations and can be linked only as context
or conflict evidence.

## Maintenance

When a feature page is added or refreshed, link it from this index and update its status/evidence
record. Changed manual Markdown must pass the repository documentation navigation/link gate and
the manual-wide fence validator; TASK-2054 records the initial completed validation baseline.
