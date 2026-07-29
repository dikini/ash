# Language Reference Authoring Conventions

[Reference index](index.md) · [Status and coverage](status.md) ·
[Source of truth](source-of-truth.md)

## Page status

**Reviewed implementation revision:** `423f603c`. TASK-2054's closeout review confirms that this
convention governs the completed manual; future pages must refresh the implementation evidence
they inspect.

**Implementation:** not applicable for this authoring-convention page.
**Evidence:** [PLAN-206](../../plan/PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md) and
[TASK-2045](../../plan/tasks/TASK-2045-language-reference-placement-and-skeleton.md).
**Parity:** not applicable for this authoring-convention page.

## Required feature-page structure

Each feature page includes, in this order where applicable:

1. Status and evidence: reviewed revision; grammar, static, lowering, and admission/runtime
   status; implementation/evidence/parity axes; exact source/test links; and non-goals.
2. What it is and how to use it: concise prose about observable source behavior.
3. Examples: minimal checked examples, labelled static-only or fixture-bounded when that is all
   the evidence proves.
4. Syntax: accepted source grammar only, in an `ebnf` fence.
5. Semantics: precise `sequent` rules only when the implementation supplies the corresponding
   rule; otherwise name the absent layer or limitation.
6. Diagnostics and boundaries: rejection modes, bounded routes, authority non-grants, and related
   evidence.

## Evidence and examples

An example cannot establish behavior beyond its evidenced route. A public declaration/import is
not runtime proof, and an internal Core/CPS term is not a source-language feature. Never include
deprecated workflow/tower syntax as a copyable current-language example. Historical snippets may
only appear outside current guidance with an explicit historical label and fresh
implementation-backed reason.

## EBNF and sequent fences

Use an `ebnf` fence compatible with `/home/dikini/Projects/railroad`: productions use `=`, end in
`;`, quote terminals, and do not use `::=`. Use only supported grouping, alternatives, optionals,
repetitions, and postfix quantifiers.

Use a `sequent` fence compatible with `/home/dikini/Projects/sequent-md`: name rules with `:=` or
`::=`, put premises in bracket groups, and use `=>` or `===>` for a conclusion. Correct fence
syntax never authorizes an unsupported semantic claim.

TASK-2054 validated all current manual fences with those external projects (16 EBNF and 14
sequent). The repository documentation gate does not validate either language, so future manual
changes must run the task-owned fence validator as well as the documentation gate.

## Maintenance

Refresh a page when its parser, checker, lowering, Engine/runtime route, tests, or admitted
behavior changes. Update [the index](index.md) and [status map](status.md) with new pages or
material changes, then run the repository documentation navigation/link gate. Do not apply the
top-level `reference/` frontmatter validator to this separate manual surface.
