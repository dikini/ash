# Role Convergence Blocker Remediation Design

## Goal

Close the blocker-class gaps left after the initial role-convergence pass by replacing placeholder
role-obligation lowering with a lossless representation and by reconciling touched docs/examples
with the canonical surface contract.

## Problem Statement

The Phase 35 review found two remaining blocker areas:

1. Source role obligations are parsed as names but lowered through a placeholder bridge that turns
   them into unconditional core obligations, losing meaning and overstating end-to-end support.
2. Several touched user-facing docs/examples removed `supervises` but still do not match the
   canonical surface syntax, which blurs the line between historical/reference material and live
   examples.

There are also adjacent cleanup issues that should be closed in the same pass because they are
small, local, and directly connected to the blocked review outcome.

## Design Decision

### 1. Introduce a lossless core role-obligation carrier

Core role metadata should stop storing role-level obligations as full workflow `Obligation` values
when the source contract only preserves named obligation references.

The bounded fix is to introduce a dedicated core carrier for role-level obligation references and to
update the canonical IR spec accordingly. The carrier should preserve source intent exactly, rather
than fabricating unconditional obligations.

### 2. Make the parser/lowering path honest and observable

`role` definitions in inline modules should participate in a real parser/core path rather than a
task-local helper plus placeholder lowering function. The follow-up should either:

- wire lowered role definitions into an observable module-lowering path, or
- narrow the supported path explicitly and remove claims that imply broader end-to-end coverage.

The recommended direction is the first: add a small, explicit lowering path for inline-module role
definitions so the new role carrier is exercised by a real parser/core API.

### 3. Canonicalize touched docs/examples instead of partially cleaning them

Touched docs/examples should either become canonical-surface examples or be clearly marked as
historical/reference-only. This pass keeps scope narrow by only fixing the files touched during the
role-convergence branch and any directly related supporting text.

### 4. Keep workflow/process supervision separate

No part of this remediation reintroduces role hierarchy. Runtime/process supervision remains a
separate concept around workflow control and lifecycle authority.

## Proposed Task Shape

1. **TASK-221** — replace the placeholder core role-obligation bridge with a spec-honest carrier.
2. **TASK-222** — integrate role-definition lowering into a real inline-module parser/core path.
3. **TASK-223** — canonicalize the touched role docs/examples and fix adjacent local inconsistencies.
4. **TASK-224** — run a focused closeout audit, reconcile bookkeeping, and document any intentional
   residual references.

## Non-goals

This design does not:

- introduce role hierarchy or inherited approval semantics,
- redesign workflow/process supervision,
- broaden module lowering beyond the minimum path needed for honest role-definition support,
- attempt a repository-wide example modernization beyond the touched role-convergence files.

## Acceptance Criteria

This follow-up design is successful when:

1. core role metadata preserves named role obligations without placeholder semantics,
2. inline-module role definitions can be lowered through an observable parser/core path,
3. touched docs/examples are either canonical-surface-aligned or clearly historical,
4. the final review can describe the remaining role-convergence work as non-blocking.
