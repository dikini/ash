---
id: language.reference.index
title: Ash Language Reference
kind: manual-index
status: current
audience: [human, agent]
reviewed_revision: 423f603c
evidence: tested
refresh_trigger: ["docs/reference/language/**", "docs/plan/PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md"]
---

# Ash Language Reference

## What this manual covers

This manual describes what the implementation accepted at `423f603c`. Each chapter says whether a
form parses, type-checks, lowers, or runs. A page does not claim support that its evidence does not
show.

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

This is an implementation-backed language manual, authorized by
[SPEC-071 §3.1](../../spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md#31-scoped-implementation-backed-language-manual-exception).
It is separate from the top-level [`reference/`](../../../reference/INDEX.md) corpus and uses its
own metadata.

The live implementation and executable tests are the evidence for this manual. Plans and older
reference pages help find that evidence, but do not define the current language.

## Maintenance

When a page changes, update its metadata and evidence, then run the documentation and fence checks
listed in [Authoring conventions](conventions.md).
