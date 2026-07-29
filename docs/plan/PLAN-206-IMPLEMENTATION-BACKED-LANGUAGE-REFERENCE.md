---
id: plan.206.implementation-backed-language-reference
title: Implementation-Backed Ash Language Reference
kind: planning-packet
status: complete
authority: planning
owner: language-reference
last_verified: 2026-07-29
---

# PLAN-206: Implementation-Backed Ash Language Reference

## Purpose

Plan and author a systematic, navigable language manual rooted at
`docs/reference/language/index.md`. The manual describes the language accepted by the checked
implementation now. It does not turn target prose, an AST carrier, a historical example, or an
internal Core/CPS term into a source-language feature.

The source of truth is the live path from parser acceptance through the surface AST, static
checking, lowering, admitted Engine execution, and executable tests. Existing specifications,
machine-readable indexes, plans, audits, and top-level `reference/` pages are routing and
conflict evidence. They are not authority when they disagree with the live route.

## Audited boundary and evidence rule

The initial census is [AUDIT-206](audits/AUDIT-206-implementation-backed-language-reference.md).
Every manual task starts by refreshing the rows it owns against the commit being documented.

For each claimed source feature, authors must record all of these independently:

1. **Grammar acceptance:** accepted, rejected, or parser-only.
2. **Static route:** checked, rejected after parse, partial, or not applicable.
3. **Lowering route:** lowered, bounded-only, rejected, or not applicable.
4. **Admission/runtime route:** admitted/executed, fixture-bounded, closed, or not applicable.
5. **Evidence and parity:** `proved`, `tested`, or `none`; and `matches_spec` or `below_spec`
   where a target rule exists.

`implemented` means the relevant full route is evidenced. `partial` means at least one required
layer is missing or bounded. `planned` is target-only. `excluded` means removed or unsupported
source syntax. An internal Rust or Core/CPS type with no accepted source spelling is
`internal-only`. Rows and summary metadata are requirements/evidence transport only; they never
grant a handler frame, provider, resource, role, or runtime authority.

The completed AUDIT-206 census found no target-only/planned source-language feature to place in
this manual. A partial, below-spec, bounded, or closed current route is not a planned feature; a
future target-only feature must be explicitly registered and labelled `planned` before inclusion.

## Source-of-truth map

| Topic | Primary implementation evidence | Supporting evidence/indexes | Manual rule |
|---|---|---|---|
| Module grammar and definitions | `crates/ash-parser/src/parse_module.rs::module_file`; `crates/ash-parser/src/surface.rs::Definition` | parser tests and `SPEC-095a`/`SPEC-095b` | Trace parser branches, not enum variants alone. |
| Expressions, patterns, blocks, `do`, handlers | `crates/ash-parser/src/parse_expr.rs`, `surface.rs`, `lower.rs` | parser/typeck tests; `SPEC-095*` | Separate parse acceptance from lowering. |
| Types/type computation | `ash-typeck`, parser type/module summary paths, `ash-core` typing | `SPEC-097*`, `SPEC-100`, traceability | Do not call an internal type carrier public without source/test evidence. |
| Effects/rows/handlers | `ash-typeck/src/handler_rows.rs`, `ash-core` CPS, Engine admission | `SPEC-096*`, `SPEC-099b`, task records | State handler and row runtime boundaries explicitly. |
| Execution and clients | `ash-engine/src/lib.rs`, CLI/REPL/daemon routes | `semantic-task-records.json`, terminal tests | Describe only admitted, Engine-only bounded routes. |
| Standard library | `std/src/**`, parser/typeck/Engine stdlib tests | checked corpus and module-loader tests | A public declaration/import is not execution evidence. |
| Historical/reference material | `reference/**`, `docs/reference/**`, old specs/plans | `CANONICAL-CORPUS.json`, `SEMANTIC-TRACEABILITY.json` | Cite only as context/conflict; never as primary syntax authority. |

Rust-analyzer activation failed for this worktree during the audit; PLAN-206 research is
baseline-only (`rg`, targeted source reads, and executable-test records). A documentation task
must retry language-aware navigation before broad source search and record whether it became
productive.

## Reference placement

SPEC-071 §3, rule 2 requires the reference corpus at top-level `reference/` unless it is
superseded.
TASK-2045 selected the first permitted outcome: the narrow policy amendment in
[SPEC-071 §3.1](../spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md#31-scoped-implementation-backed-language-manual-exception)
authorizes only `docs/reference/language/` as a separate implementation-backed language manual
and working surface. It remains outside the SPEC-071 top-level corpus and its frontmatter
validator. The amendment supplies the manual's authority, evidence, limitation, navigation, and
maintenance rules; this plan does not itself bypass the top-level policy.

Under that outcome, TASK-2045 owns the written placement decision:

- preserve `reference/` as canonical-adjacent historical/curated material; do not migrate it;
- preserve existing `docs/reference/` contracts as cross-cutting/formal planning material;
- give the new directory its own index, status/source map, and page evidence convention;
- link the plan from `docs/README.md`, then link the actual manual only once TASK-2045 creates it.

This avoids inheriting the old corpus validator, which scans `reference/**` and requires `ref.*`
frontmatter, while still using its useful stale-document discipline.

## Proposed manual hierarchy

The hierarchy is a destination, not a claim that every page is already implemented. Detailed
pages are created only for rows found implemented or materially partial in their owning task.

```text
docs/reference/language/
├── index.md
├── status.md
├── source-of-truth.md
├── conventions.md
├── lexical-and-modules/
│   ├── index.md
│   ├── source-files-names-and-literals.md
│   ├── modules-imports-and-visibility.md
│   └── notation-and-expression-macros.md
├── forms/
│   ├── index.md
│   ├── declarations-and-functions.md
│   ├── values-bindings-blocks-and-calls.md
│   └── control-flow-and-patterns.md
├── types/
│   ├── index.md
│   ├── data-newtypes-and-callables.md
│   ├── generics-kinds-interfaces-and-impls.md
│   └── type-level-domains-functions-families-and-propositions.md
├── effects/
│   ├── index.md
│   ├── rows-aliases-groups-and-operations.md
│   ├── resources-roles-and-authority-boundaries.md
│   ├── handlers-failure-and-do.md
│   └── comprehensions.md
├── execution/
│   ├── index.md
│   ├── entry-lowering-and-admission.md
│   └── clients-terminals-and-diagnostics.md
└── library/
    ├── index.md
    ├── modules-and-imports.md
    └── diagnostics-and-errors.md
```

## Standard feature-page template

1. **Status and evidence:** feature ID, reviewed revision, grammar/static/lowering/runtime status,
   exact code/tests, implementation/evidence/parity axes, and non-goals.
2. **What it is / use:** concise prose explaining observable source behavior.
3. **Examples:** minimal and realistic examples, each tied to parser/checker/runtime evidence and
   labelled static-only where it has no execution proof.
4. **Syntax:** an `ebnf` fence with only the feature's accepted grammar.
5. **Semantics:** `sequent` typing/transition rules only where the implementation gives a precise
   rule. Otherwise say which semantic layer is absent; never manufacture a formal rule.
6. **Diagnostics and boundaries:** rejection modes, bounded routes, limitations, and authority
   non-grants.
7. **Related evidence:** neighbouring reference links and exact implementation/test links.

## Grammar and sequent policy

`ebnf` fences must be compatible with `/home/dikini/Projects/railroad`: use `=`, terminate every
production with `;`, quote terminals, and never use `::=`. Use only supported sequences,
alternatives, grouping, optionals, repetitions, and postfix quantifiers.

`sequent` fences must follow `/home/dikini/Projects/sequent-md/README.md` and `grammar.md`: name
rules with `:=` or `::=`, put premises in bracket groups, and use `=>` or `===>` for the
conclusion. This syntax allowance does not authorize unsupported semantic claims. TASK-2054
parsed/rendered every current manual fence with both external projects; Ash's existing docs gate
does not validate either fence language.

## Work plan and dependencies

| Task | Scope | Depends on | Parallelism |
|---|---|---|---|
| TASK-2044 | Research packet, audit, task decomposition | — | Complete foundation |
| TASK-2045 | Placement decision, skeleton/index/status/source-map convention | 2044 | Complete — SPEC-071 §3.1 amendment and four-page skeleton established |
| TASK-2046 | Lexical/modules/imports/notation/macros | 2045 | Complete — implementation-backed lexical/module status, EBNF, and route boundaries verified |
| TASK-2047 | Declarations, functions, values, blocks, control, patterns | 2045 | Complete — implementation-backed forms status, EBNF/sequents, and route boundaries verified |
| TASK-2048 | Ordinary data/types/callables/generics/kinds/interfaces/impls | 2045 | Complete — implementation-backed type/interface boundaries, EBNF, and bounded sequents verified |
| TASK-2049 | Type-level domains/functions/families/propositions | 2045 | Complete — implementation-backed type-level parser/static boundaries, EBNF, and normalization sequent verified |
| TASK-2050 | Rows, aliases/groups, declared operations, resources/roles | 2045 | Complete — implementation-backed effects chapter, authority limits, EBNF, and non-granting sequent verified |
| TASK-2051 | Handlers, failure, `do`, comprehensions | 2045; coordinate 2050 terminology | Complete — bounded routes, EBNF, and typed/lowering sequents verified |
| TASK-2052 | Entry/lowering/admission/clients/terminals | 2045 | Complete — bounded `fn main`, Engine admission/request, selected client, and V1 terminal boundaries verified |
| TASK-2053 | Public stdlib modules/imports and diagnostics/errors | 2045 | Complete — 59-file corpus, bounded runtime registry, selected `time::sleep`, and diagnostic/terminal limits verified |
| TASK-2054 | Integration, evidence refresh, fence/link/index validation, closeout | 2046-2053 | Complete — 30 manual fences validated; navigation, documentation gates, and external consumers verified |

LANG-004 gives TASK-2047 only the active-declaration inventory and its cross-links; the detailed
feature documentation remains owned by TASK-2048 through TASK-2051 as their task scopes specify.

## Verification and non-goals

Every task ran its named parser/typeck/Engine/CLI tests and example checks. The closeout ran:

```bash
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
python3 tools/reference/validate.py --root .   # report existing legacy failures separately
git diff --check
```

It also ran the external railroad and sequent-md validation steps recorded by TASK-2054: the
helper found 16 EBNF and 14 sequent fences; its Node tests passed 23/23; railroad check passed
38/38 and built (two non-fatal vendor `-0` warnings); sequent-md tests passed 26/26 and built.
The orientation self-test, documentation gate (2,032 links and zero missing), and diff hygiene
also passed. No task may claim that the existing docs gate validates EBNF/sequent fences.

The narrow policy reconciliation in **Reference placement** is in scope: it may require an
approved SPEC/design/policy amendment or supersession, or an authority-approved separate-surface
classification, together with its required index/policy updates. Apart from that reconciliation,
non-goals are changing parser, checker, lowering, Engine, tests, target specifications, or legacy
`reference/`; resolving every historical document; documenting removed forms as language features;
or claiming general execution/parity from a fixture-bounded route.
