---
id: language.reference.conventions
title: Language Reference Authoring Conventions
kind: authoring-guide
status: current
audience: [human, agent]
reviewed_revision: 423f603c
evidence: tested
refresh_trigger: ["docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md", "docs/plan/PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md", "tools/docs/validate_language_reference_fences.mjs"]
---

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

## Manual metadata

Every page begins with YAML frontmatter. This is metadata for this manual, not the top-level
`reference/` schema.

```yaml
id: language.reference.<stable-page-name>
title: Human-readable title
kind: manual-index | chapter-index | feature-reference | status-map | methodology | authoring-guide
status: current | partial
audience: [human, agent]
reviewed_revision: <implementation commit>
evidence: tested | none
refresh_trigger: ["source/or/test/path/**"]
```

`reviewed_revision` identifies the implementation revision behind the page's claims; it is not a
claim that the page itself was committed at that revision. `refresh_trigger` names code, tests, or
policy that require a new review when they change. Keep identifiers stable so links and tooling can
refer to a page without depending on its title or path.

## Feature-page structure

Feature pages should lead with the reader's question, not the implementation census:

1. **Status and evidence** states the reviewed revision, the four route statuses, and the exact
   source/test evidence.
2. **What it is** explains the accepted source form and its practical use.
3. **Examples** show the smallest evidence-backed form. Label parser-only, static-only, or
   fixture-bounded examples at the point where the distinction matters.
4. **Syntax** contains accepted grammar only.
5. **Semantics and boundaries** explains the strongest supported behavior and names the next
   missing layer once.
6. **Related evidence** collects task links and commands.

Avoid repeating the same caveat after every example. State the boundary beside the first claim it
limits, then use ordinary prose unless a later claim needs a different boundary.

## Evidence and examples

An example cannot establish behavior beyond its evidenced route. A public declaration/import is
not runtime proof, and an internal Core/CPS term is not a source-language feature. Never include
deprecated workflow/tower syntax as a copyable current-language example. Historical snippets may
only appear outside current guidance with an explicit historical label and fresh
implementation-backed reason.

## Writing style

Use Orwell's rules:

1. Prefer a short familiar word to a long one.
2. Cut a word when it adds no meaning.
3. Use active voice when it names the actor.
4. Do not turn a simple verb into an abstract noun.
5. Do not use technical language to sound technical. Keep a term only when it names an exact Ash
   construct or implementation component.
6. Break these rules rather than write an ugly or misleading sentence.

Write the rule first. Then show the source form or result. Keep implementation details and test
paths in the support and evidence sections instead of using them to introduce every paragraph.
Say `the parser rejects this` rather than `this remains a parser boundary`; say `the Engine cannot
run this form yet` rather than `the route is closed`.

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

Refresh a page when a path in its `refresh_trigger` changes, or when its parser, checker,
lowering, Engine/runtime route, executable evidence, or admitted behavior changes. Update the
page's revision and evidence fields, then update [the index](index.md) or [status map](status.md)
when navigation or support status changed. Run the manual fence validator and the repository
documentation navigation/link gate. Do not apply the top-level `reference/` frontmatter validator
to this separate manual surface.
