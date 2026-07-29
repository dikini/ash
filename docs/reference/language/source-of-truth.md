# Language Reference Source of Truth

[Reference index](index.md) · [Status and coverage](status.md) ·
[Authoring conventions](conventions.md)

## Page status

**Reviewed implementation revision:** `423f603c`. The closeout rechecked manual evidence in the
current workspace; a later feature refresh records the implementation revision it actually
inspects.

**Implementation:** not applicable for this evidence-routing page.
**Evidence:** [PLAN-206](../../plan/PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md),
[AUDIT-206](../../plan/audits/AUDIT-206-implementation-backed-language-reference.md), and the
documented repository checks.
**Parity:** not applicable for this evidence-routing page.

## Authority order

For a claim about current Ash source behavior, use the live route in this order:

1. Parser acceptance and source spelling.
2. Surface AST and name/type checking.
3. Core/CPS lowering.
4. Engine admission and runtime behavior.
5. Executable tests that demonstrate the claimed route.

Specifications, JSON indexes, plans, audits, and existing reference pages help locate evidence or
record conflicts. They do not override the live route. A source feature must be traced through the
layers that apply to its claim; a surface AST variant alone is not proof that syntax is accepted.

## Placement decision

[SPEC-071 §3.1](../../spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md#31-scoped-implementation-backed-language-manual-exception)
authorizes this directory as the one separate implementation-backed language manual/working
surface. It remains outside the SPEC-071 top-level `reference/` corpus and its frontmatter
validator. The policy retains the top-level corpus and historical material in place; this manual
does not rewrite them.

This directory still follows `docs/` navigation and changed-Markdown link checks. Every page must
be reachable from [the manual index](index.md). TASK-2054 completed the initial manual-specific
fence and closeout validation; future changes retain those checks.

## Evidence required on a feature page

State the reviewed revision, exact source paths or symbols, exact test files or commands, the
status axes in [Status and coverage](status.md), and any contradiction, bounded route, or missing
layer. When current implementation and a specification disagree, explain the limit and link to the
conflicting material; do not select the more convenient claim.

## Historical boundary

Top-level [`reference/`](../../../reference/INDEX.md) and older `docs/reference/` contracts are
preserved context and may contain stale or historical claims. They never authorize a current
feature description without fresh implementation-backed evidence. Removed workflow/tower syntax
is not a current-language example.
