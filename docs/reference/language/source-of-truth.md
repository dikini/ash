---
id: language.reference.source-of-truth
title: Language Reference Source of Truth
kind: methodology
status: current
audience: [human, agent]
reviewed_revision: 423f603c
evidence: tested
refresh_trigger: ["docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md", "docs/plan/PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md"]
---

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

## How to check a claim

Check the implementation in this order:

1. Parser acceptance and source spelling.
2. Surface AST and name/type checking.
3. Core/CPS lowering.
4. Engine admission and runtime behavior.
5. Executable tests that demonstrate the claimed route.

Specifications, plans, audits, and older reference pages can point to evidence or record a
conflict. They do not override the implementation. An AST variant alone does not show that the
parser accepts its syntax.

## Placement decision

[SPEC-071 §3.1](../../spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md#31-scoped-implementation-backed-language-manual-exception)
authorizes this directory as the one separate implementation-backed language manual/working
surface. It remains outside the SPEC-071 top-level `reference/` corpus and its frontmatter
validator. The policy retains the top-level corpus and historical material in place; this manual
does not rewrite them.

Every page must be reachable from [the manual index](index.md). Changed pages must pass the
documentation link check and the manual fence check.

## Evidence required on a feature page

State the reviewed revision, source paths, tests, the status values from
[Status and coverage](status.md), and any missing step. When the implementation and a
specification disagree, describe the difference and link to both.

## Historical boundary

Top-level [`reference/`](../../../reference/INDEX.md) and older `docs/reference/` pages are
historical context. Do not use them as evidence for current syntax. Removed workflow/tower syntax
does not belong in current examples.
